//! Effect state component mirroring upstream
//! `mindustry.entities.comp.EffectStateComp`.

use super::{
    child::{ChildComp, ChildParent},
    decal::DecalColor,
};
use crate::mindustry::{
    entities::{Effect, DEFAULT_EFFECT_CLIP},
    io::{EffectStateSyncWire, TypeValue},
};

#[derive(Debug, Clone, PartialEq)]
pub struct EffectRenderInput<'a> {
    pub id: i32,
    pub effect_id: Option<u16>,
    pub color: DecalColor,
    pub time: f32,
    pub lifetime: f32,
    pub clip: f32,
    pub rotation: f32,
    pub x: f32,
    pub y: f32,
    pub data: &'a TypeValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectStateComp {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub time: f32,
    pub lifetime: f32,
    pub color: DecalColor,
    pub effect_clip: f32,
    pub data: TypeValue,
    pub effect_id: Option<u16>,
    pub offset_x: f32,
    pub offset_y: f32,
    pub offset_pos: f32,
    pub offset_rot: f32,
    pub parent_id: Option<i32>,
    pub rot_with_parent: bool,
}

impl EffectStateComp {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            time: 0.0,
            lifetime: 0.0,
            color: DecalColor::WHITE,
            effect_clip: DEFAULT_EFFECT_CLIP,
            data: TypeValue::Null,
            effect_id: None,
            offset_x: 0.0,
            offset_y: 0.0,
            offset_pos: 0.0,
            offset_rot: 0.0,
            parent_id: None,
            rot_with_parent: false,
        }
    }

    /// Java: `lifetime = effect.render(...)`.
    pub fn draw_with<F>(&mut self, render: F)
    where
        F: FnOnce(EffectRenderInput<'_>) -> f32,
    {
        self.lifetime = render(EffectRenderInput {
            id: self.id,
            effect_id: self.effect_id,
            color: self.color,
            time: self.time,
            lifetime: self.lifetime,
            clip: self.effect_clip,
            rotation: self.rotation,
            x: self.x,
            y: self.y,
            data: &self.data,
        });
    }

    /// Java `EffectState` inherits the normal entity lifetime tick.  Keep the
    /// time advancement local to the state so callers can materialize packet
    /// ingress into states first, then run a real tick/cull/draw pass instead
    /// of treating packets as renderable entities.
    pub fn tick(&mut self, delta: f32) {
        self.time += delta.max(0.0);
    }

    pub fn is_expired(&self) -> bool {
        self.time >= self.lifetime
    }

    pub fn update_parent_transform(&mut self, parent: Option<ChildParent>) -> bool {
        if self.parent_id.is_none() {
            return false;
        }
        let Some(parent) = parent else {
            return false;
        };

        let mut child = ChildComp::new(self.x, self.y, self.rotation);
        child.parent = Some(parent);
        child.rot_with_parent = self.rot_with_parent;
        child.offset_x = self.offset_x;
        child.offset_y = self.offset_y;
        child.offset_pos = self.offset_pos;
        child.offset_rot = self.offset_rot;
        child.update();

        self.x = child.x;
        self.y = child.y;
        self.rotation = child.rotation;
        true
    }

    pub fn clip_size(&self) -> f32 {
        self.effect_clip
    }

    pub fn apply_sync_wire(&mut self, sync: &EffectStateSyncWire, effect: Option<&Effect>) {
        self.color = decal_color_from_rgba(sync.color.rgba() as u32);
        self.data = sync.data.clone();
        self.effect_id = Some(sync.effect_id);
        self.lifetime = sync.lifetime;
        self.effect_clip = effect
            .map(|effect| effect.clip)
            .unwrap_or(DEFAULT_EFFECT_CLIP);
        self.offset_pos = sync.offset_pos;
        self.offset_rot = sync.offset_rot;
        self.offset_x = sync.offset_x;
        self.offset_y = sync.offset_y;
        self.parent_id = sync.parent_id;
        self.rot_with_parent = sync.rot_with_parent;
        self.rotation = sync.rotation;
        self.time = sync.time;
        self.x = sync.x;
        self.y = sync.y;
    }
}

fn decal_color_from_rgba(rgba: u32) -> DecalColor {
    DecalColor {
        r: ((rgba >> 24) & 0xff) as f32 / 255.0,
        g: ((rgba >> 16) & 0xff) as f32 / 255.0,
        b: ((rgba >> 8) & 0xff) as f32 / 255.0,
        a: (rgba & 0xff) as f32 / 255.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_state_draw_updates_lifetime_from_effect_renderer() {
        let mut state = EffectStateComp::new(9);
        state.x = 1.0;
        state.y = 2.0;
        state.rotation = 45.0;
        state.time = 3.0;
        state.lifetime = 4.0;
        state.effect_id = Some(17);
        state.effect_clip = 11.0;
        state.data = TypeValue::String("payload".into());

        state.draw_with(|input| {
            assert_eq!(input.id, 9);
            assert_eq!(input.effect_id, Some(17));
            assert_eq!((input.x, input.y, input.rotation), (1.0, 2.0, 45.0));
            assert_eq!((input.time, input.lifetime), (3.0, 4.0));
            assert_eq!(input.clip, 11.0);
            assert_eq!(input.data, &TypeValue::String("payload".into()));
            8.0
        });

        assert_eq!(state.lifetime, 8.0);
    }

    #[test]
    fn effect_state_clip_size_matches_effect_clip() {
        let mut state = EffectStateComp::new(1);
        state.effect_clip = 32.0;

        assert_eq!(state.clip_size(), 32.0);
    }

    #[test]
    fn effect_state_sync_without_effect_falls_back_to_default_clip() {
        let mut state = EffectStateComp::new(11);
        state.effect_clip = 1.0;
        let sync = EffectStateSyncWire {
            color: crate::mindustry::io::type_io::RgbaColor::new(0xff00ff00u32 as i32),
            data: TypeValue::Null,
            effect_id: 99,
            lifetime: 10.0,
            offset_pos: 0.0,
            offset_rot: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            parent_id: None,
            rot_with_parent: false,
            rotation: 0.0,
            time: 0.0,
            x: 0.0,
            y: 0.0,
        };

        state.apply_sync_wire(&sync, None);

        assert_eq!(state.effect_clip, DEFAULT_EFFECT_CLIP);
        assert_eq!(state.clip_size(), DEFAULT_EFFECT_CLIP);
    }

    #[test]
    fn effect_state_ticks_and_reports_expiry_like_lifetime_entity() {
        let mut state = EffectStateComp::new(2);
        state.lifetime = 3.0;

        state.tick(1.25);
        assert_eq!(state.time, 1.25);
        assert!(!state.is_expired());

        state.tick(-10.0);
        assert_eq!(
            state.time, 1.25,
            "negative render deltas must not rewind pooled effect state time"
        );

        state.tick(1.75);
        assert_eq!(state.time, 3.0);
        assert!(state.is_expired());
    }

    #[test]
    fn effect_state_updates_parent_transform_like_child_component() {
        let mut state = EffectStateComp::new(3);
        state.parent_id = Some(99);
        state.rot_with_parent = true;
        state.offset_x = 2.0;
        state.offset_y = 0.0;
        state.offset_pos = 0.0;
        state.offset_rot = 15.0;
        state.x = 0.0;
        state.y = 0.0;
        state.rotation = 0.0;

        assert!(state.update_parent_transform(Some(ChildParent {
            x: 10.0,
            y: 20.0,
            rotation: Some(90.0),
        })));

        assert!((state.x - 10.0).abs() < 0.0001);
        assert!((state.y - 22.0).abs() < 0.0001);
        assert_eq!(state.rotation, 105.0);

        state.parent_id = None;
        assert!(!state.update_parent_transform(Some(ChildParent {
            x: 0.0,
            y: 0.0,
            rotation: Some(0.0),
        })));
    }

    #[test]
    fn effect_state_applies_sync_wire_fields_and_preserves_effect_clip() {
        let mut state = EffectStateComp::new(10);
        let effect = Effect::with_lifetime(7, 50.0, 32.0);
        let sync = EffectStateSyncWire {
            color: crate::mindustry::io::type_io::RgbaColor::new(0x336699cc),
            data: TypeValue::String("spark".into()),
            effect_id: 7,
            lifetime: 50.0,
            offset_pos: 1.25,
            offset_rot: -2.5,
            offset_x: 3.0,
            offset_y: 4.0,
            parent_id: Some(1234),
            rot_with_parent: true,
            rotation: 90.0,
            time: 12.0,
            x: 100.0,
            y: 200.0,
        };

        state.apply_sync_wire(&sync, Some(&effect));

        assert_eq!(state.effect_id, Some(7));
        assert_eq!(state.data, TypeValue::String("spark".into()));
        assert_eq!(state.lifetime, 50.0);
        assert_eq!(state.effect_clip, 32.0);
        assert!((state.color.r - 0x33 as f32 / 255.0).abs() < 0.0001);
        assert!((state.color.g - 0x66 as f32 / 255.0).abs() < 0.0001);
        assert!((state.color.b - 0x99 as f32 / 255.0).abs() < 0.0001);
        assert!((state.color.a - 0xcc as f32 / 255.0).abs() < 0.0001);
        assert_eq!(state.offset_pos, 1.25);
        assert_eq!(state.offset_rot, -2.5);
        assert_eq!(state.offset_x, 3.0);
        assert_eq!(state.offset_y, 4.0);
        assert_eq!(state.parent_id, Some(1234));
        assert!(state.rot_with_parent);
        assert_eq!(state.rotation, 90.0);
        assert_eq!(state.time, 12.0);
        assert_eq!(state.x, 100.0);
        assert_eq!(state.y, 200.0);
    }
}
