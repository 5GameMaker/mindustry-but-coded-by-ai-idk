use crate::mindustry::ctype::{ContentId, ContentType, UnlockableContentBase};

#[derive(Debug, Clone, PartialEq)]
pub struct Liquid {
    pub base: UnlockableContentBase,
    pub gas: bool,
    pub color_rgba: u32,
    pub gas_color_rgba: u32,
    pub bar_color_rgba: Option<u32>,
    pub light_color_rgba: u32,
    pub flammability: f32,
    pub temperature: f32,
    pub heat_capacity: f32,
    pub viscosity: f32,
    pub explosiveness: f32,
    pub block_reactive: bool,
    pub coolant: bool,
    pub move_through_blocks: bool,
    pub incinerable: bool,
    pub effect: Option<String>,
    pub particle_effect: String,
    pub particle_spacing: f32,
    pub boil_point: f32,
    pub vapor_effect: String,
    pub cap_puddles: bool,
    pub hidden: bool,
    pub can_stay_on: Vec<String>,
    pub cell_spread_target: Option<String>,
    pub cell_max_spread: f32,
    pub cell_spread_conversion: f32,
    pub cell_spread_damage: f32,
    pub cell_remove_scaling: f32,
}

impl Liquid {
    pub const ANIMATION_FRAMES: i32 = 50;
    pub const ANIMATION_SCALE_GAS: f32 = 190.0;
    pub const ANIMATION_SCALE_LIQUID: f32 = 230.0;

    pub fn new(id: ContentId, name: impl Into<String>) -> Self {
        Self {
            base: UnlockableContentBase::new(id, ContentType::Liquid, name),
            gas: false,
            color_rgba: 0x000000ff,
            gas_color_rgba: 0xbfbfbfff,
            bar_color_rgba: None,
            light_color_rgba: 0x00000000,
            flammability: 0.0,
            temperature: 0.5,
            heat_capacity: 0.5,
            viscosity: 0.5,
            explosiveness: 0.0,
            block_reactive: true,
            coolant: true,
            move_through_blocks: false,
            incinerable: true,
            effect: Some("none".to_string()),
            particle_effect: "none".to_string(),
            particle_spacing: 60.0,
            boil_point: 2.0,
            vapor_effect: "vapor".to_string(),
            cap_puddles: true,
            hidden: false,
            can_stay_on: Vec::new(),
            cell_spread_target: None,
            cell_max_spread: 0.75,
            cell_spread_conversion: 1.2,
            cell_spread_damage: 0.11,
            cell_remove_scaling: 0.25,
        }
    }

    pub fn name(&self) -> &str {
        &self.base.mappable.name
    }

    pub fn localized_name(&self) -> &str {
        self.base
            .localized_name
            .as_deref()
            .unwrap_or_else(|| self.name())
    }

    pub fn init(&mut self) {
        if self.gas {
            self.boil_point = -1.0;
            self.color_rgba = with_alpha(self.color_rgba, 0x99);
            self.gas_color_rgba = self.color_rgba;
            if self.bar_color_rgba.is_none() {
                self.bar_color_rgba = Some(with_alpha(self.color_rgba, 0xff));
            }
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn animation_frame(&self, time: f32) -> i32 {
        let scale = if self.gas {
            Self::ANIMATION_SCALE_GAS
        } else {
            Self::ANIMATION_SCALE_LIQUID
        };
        ((time / scale * Self::ANIMATION_FRAMES as f32 + self.base.mappable.base.id as f32 * 5.0)
            as i32)
            .rem_euclid(Self::ANIMATION_FRAMES)
    }

    pub fn will_boil(&self, heat_env: f32) -> bool {
        heat_env >= self.boil_point
    }

    pub fn can_extinguish(&self) -> bool {
        self.flammability < 0.1 && self.temperature <= 0.5
    }

    pub fn bar_color(&self) -> u32 {
        self.bar_color_rgba.unwrap_or(self.color_rgba)
    }

    pub fn react(&self, _other: &Liquid, _amount: f32) -> f32 {
        0.0
    }

    pub fn logic_id(&self) -> i32 {
        self.base.mappable.base.id as i32
    }

    pub fn sense_color(&self) -> f64 {
        crate::mindustry::logic::rgba_u32_to_double_bits(self.color_rgba)
    }

    pub fn sense_id(&self) -> f64 {
        self.logic_id() as f64
    }

    pub fn sense_name(&self) -> &str {
        self.name()
    }
}

impl std::fmt::Display for Liquid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.localized_name())
    }
}

fn with_alpha(rgba: u32, alpha: u8) -> u32 {
    (rgba & 0xffff_ff00) | alpha as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::logic::double_bits_to_rgba;

    #[test]
    fn liquid_defaults_match_java_constructor_shape() {
        let liquid = Liquid::new(2, "water");
        assert_eq!(liquid.name(), "water");
        assert!(!liquid.gas);
        assert_eq!(liquid.color_rgba, 0x000000ff);
        assert_eq!(liquid.gas_color_rgba, 0xbfbfbfff);
        assert_eq!(liquid.bar_color_rgba, None);
        assert_eq!(liquid.light_color_rgba, 0x00000000);
        assert_eq!(liquid.temperature, 0.5);
        assert_eq!(liquid.heat_capacity, 0.5);
        assert_eq!(liquid.viscosity, 0.5);
        assert!(liquid.block_reactive);
        assert!(liquid.coolant);
        assert!(!liquid.move_through_blocks);
        assert!(liquid.incinerable);
        assert_eq!(liquid.effect.as_deref(), Some("none"));
        assert_eq!(liquid.particle_effect, "none");
        assert_eq!(liquid.particle_spacing, 60.0);
        assert_eq!(liquid.boil_point, 2.0);
        assert!(liquid.cap_puddles);
        assert!(!liquid.hidden);
        assert!(liquid.can_stay_on.is_empty());
        assert_eq!(liquid.cell_spread_target, None);
        assert_eq!(liquid.cell_max_spread, 0.75);
        assert_eq!(liquid.cell_spread_conversion, 1.2);
        assert_eq!(liquid.cell_spread_damage, 0.11);
        assert_eq!(liquid.cell_remove_scaling, 0.25);
        assert_eq!(liquid.base.mappable.base.content_type, ContentType::Liquid);
    }

    #[test]
    fn liquid_gas_init_matches_java_side_effects() {
        let mut gas = Liquid::new(1, "ozone");
        gas.gas = true;
        gas.color_rgba = 0xfc81ddff;
        gas.init();

        assert_eq!(gas.boil_point, -1.0);
        assert_eq!(gas.color_rgba, 0xfc81dd99);
        assert_eq!(gas.gas_color_rgba, 0xfc81dd99);
        assert_eq!(gas.bar_color_rgba, Some(0xfc81ddff));
    }

    #[test]
    fn liquid_basic_helpers_follow_java_logic_contract() {
        let mut liquid = Liquid::new(4, "oil");
        liquid.base.localized_name = Some("Oil".into());
        liquid.color_rgba = 0x313131ff;
        liquid.flammability = 1.2;
        liquid.temperature = 0.6;
        liquid.boil_point = 0.65;
        liquid.bar_color_rgba = Some(0x6b675fff);

        assert_eq!(liquid.to_string(), "Oil");
        assert_eq!(liquid.sense_name(), "oil");
        assert_eq!(liquid.sense_id(), 4.0);
        assert_eq!(double_bits_to_rgba(liquid.sense_color()), 0x313131ff);
        assert_eq!(liquid.bar_color(), 0x6b675fff);
        assert!(!liquid.can_extinguish());
        assert!(!liquid.will_boil(0.64));
        assert!(liquid.will_boil(0.65));
        assert_eq!(liquid.react(&Liquid::new(0, "water"), 10.0), 0.0);

        liquid.hidden = true;
        assert!(liquid.is_hidden());
    }

    #[test]
    fn liquid_animation_frame_uses_gas_or_liquid_scale_and_id_offset() {
        let mut liquid = Liquid::new(2, "water");
        assert_eq!(liquid.animation_frame(0.0), 10);
        assert_eq!(liquid.animation_frame(Liquid::ANIMATION_SCALE_LIQUID), 10);

        liquid.gas = true;
        assert_eq!(liquid.animation_frame(0.0), 10);
        assert_eq!(
            liquid.animation_frame(Liquid::ANIMATION_SCALE_GAS / 2.0),
            35
        );
    }
}
