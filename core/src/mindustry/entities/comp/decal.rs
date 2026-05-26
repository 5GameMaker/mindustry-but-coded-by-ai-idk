//! Decal component mirroring upstream `mindustry.entities.comp.DecalComp`.

use crate::mindustry::io::DecalSyncWire;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecalColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl DecalColor {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn from_rgba(rgba: u32) -> Self {
        Self {
            r: ((rgba >> 24) & 0xff) as f32 / 255.0,
            g: ((rgba >> 16) & 0xff) as f32 / 255.0,
            b: ((rgba >> 8) & 0xff) as f32 / 255.0,
            a: (rgba & 0xff) as f32 / 255.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecalRegion {
    pub name: String,
    pub width: f32,
}

impl DecalRegion {
    pub fn new(name: impl Into<String>, width: f32) -> Self {
        Self {
            name: name.into(),
            width,
        }
    }

    pub fn unknown() -> Self {
        Self::new("unknown", 0.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecalDrawPlan {
    pub layer: f32,
    pub mix_color: DecalColor,
    pub mix_alpha: f32,
    pub alpha: f32,
    pub region: DecalRegion,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecalComp {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub time: f32,
    pub lifetime: f32,
    pub color: DecalColor,
    pub region: DecalRegion,
}

impl DecalComp {
    pub const LAYER_SCORCH: f32 = 10.0;

    pub fn new(region: DecalRegion) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            time: 0.0,
            lifetime: 1.0,
            color: DecalColor::WHITE,
            region,
        }
    }

    pub fn fin(&self) -> f32 {
        self.time / self.lifetime
    }

    pub fn draw(&self) -> DecalDrawPlan {
        DecalDrawPlan {
            layer: Self::LAYER_SCORCH,
            mix_color: self.color,
            mix_alpha: self.color.a,
            alpha: 1.0 - curve(self.fin(), 0.98),
            region: self.region.clone(),
            x: self.x,
            y: self.y,
            rotation: self.rotation,
        }
    }

    pub fn clip_size(&self) -> f32 {
        self.region.width * 2.0
    }

    pub fn apply_sync_wire(&mut self, sync: &DecalSyncWire) {
        self.color = DecalColor::from_rgba(sync.color.rgba() as u32);
        self.lifetime = sync.lifetime;
        self.rotation = sync.rotation;
        self.time = sync.time;
        self.x = sync.x;
        self.y = sync.y;
        // Upstream sync cannot serialize TextureRegion; preserve the existing
        // region assigned by the creator/renderer side instead of replacing it.
    }
}

fn curve(value: f32, start: f32) -> f32 {
    if value <= start {
        0.0
    } else {
        ((value - start) / (1.0 - start)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decal_component_draw_plan_matches_java_layer_alpha_and_region() {
        let mut decal = DecalComp::new(DecalRegion::new("scorch", 12.0));
        decal.x = 3.0;
        decal.y = 4.0;
        decal.rotation = 90.0;
        decal.time = 0.99;
        decal.lifetime = 1.0;

        let plan = decal.draw();

        assert_eq!(plan.layer, DecalComp::LAYER_SCORCH);
        assert_eq!(plan.mix_color, DecalColor::WHITE);
        assert_eq!(plan.mix_alpha, 1.0);
        assert!((plan.alpha - 0.5).abs() < 0.0001);
        assert_eq!(plan.region.name, "scorch");
        assert_eq!((plan.x, plan.y, plan.rotation), (3.0, 4.0, 90.0));
    }

    #[test]
    fn decal_component_clip_size_is_double_region_width() {
        let decal = DecalComp::new(DecalRegion::new("mark", 16.0));

        assert_eq!(decal.clip_size(), 32.0);
    }

    #[test]
    fn decal_component_applies_sync_wire_and_preserves_region() {
        let mut decal = DecalComp::new(DecalRegion::new("scorch", 12.0));
        let sync = DecalSyncWire {
            color: crate::mindustry::io::type_io::RgbaColor::new(0x336699cc),
            lifetime: 45.0,
            rotation: 180.0,
            time: 9.0,
            x: 20.0,
            y: 40.0,
        };

        decal.apply_sync_wire(&sync);

        assert!((decal.color.r - 0x33 as f32 / 255.0).abs() < 0.0001);
        assert!((decal.color.g - 0x66 as f32 / 255.0).abs() < 0.0001);
        assert!((decal.color.b - 0x99 as f32 / 255.0).abs() < 0.0001);
        assert!((decal.color.a - 0xcc as f32 / 255.0).abs() < 0.0001);
        assert_eq!(decal.lifetime, 45.0);
        assert_eq!(decal.rotation, 180.0);
        assert_eq!(decal.time, 9.0);
        assert_eq!((decal.x, decal.y), (20.0, 40.0));
        assert_eq!(decal.region.name, "scorch");
        assert_eq!(decal.region.width, 12.0);
    }
}
