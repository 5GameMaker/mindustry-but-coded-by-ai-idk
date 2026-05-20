use crate::mindustry::{
    ctype::ContentId,
    r#type::{unit::erekir_unit_type::PAL_DARK_OUTLINE_RGBA, UnitType},
    world::meta::Env,
};

pub fn missile_unit_type(id: ContentId, name: impl Into<String>) -> UnitType {
    let mut unit = UnitType::new(id, name);
    apply_missile_unit_type_defaults(&mut unit);
    unit
}

pub fn apply_missile_unit_type_defaults(unit: &mut UnitType) {
    unit.player_controllable = false;
    unit.create_wreck = false;
    unit.create_scorch = false;
    unit.logic_controllable = false;
    unit.is_enemy = false;
    unit.use_unit_cap = false;
    unit.draw_cell = false;
    unit.allowed_in_payloads = false;
    unit.flying = true;
    unit.env_enabled = Env::ANY;
    unit.env_disabled = Env::NONE;
    unit.physics = false;
    unit.bounded = false;
    unit.trail_length = 7;
    unit.hidden = true;
    unit.hoverable = false;
    unit.speed = 4.0;
    unit.lifetime = 60.0 * 1.7;
    unit.rotate_speed = 2.5;
    unit.range = 6.0;
    unit.target_priority = -1.0;
    unit.outline_color_rgba = PAL_DARK_OUTLINE_RGBA;
    unit.fog_radius = 2.0;
    unit.loop_sound = "loopMissileTrail".into();
    unit.loop_sound_volume = 0.05;
    unit.draw_minimap = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::ctype::{Content, ContentType};

    #[test]
    fn missile_unit_type_constructor_matches_java_preset_fields() {
        let unit = missile_unit_type(90, "quell-missile");

        assert_eq!(unit.id(), 90);
        assert_eq!(unit.content_type(), ContentType::Unit);
        assert_eq!(unit.name(), "quell-missile");
        assert!(!unit.player_controllable);
        assert!(!unit.logic_controllable);
        assert!(!unit.create_wreck);
        assert!(!unit.create_scorch);
        assert!(!unit.is_enemy);
        assert!(!unit.use_unit_cap);
        assert!(!unit.draw_cell);
        assert!(!unit.allowed_in_payloads);
        assert!(unit.flying);
        assert_eq!(unit.env_enabled, Env::ANY);
        assert_eq!(unit.env_disabled, Env::NONE);
        assert!(!unit.physics);
        assert!(!unit.bounded);
        assert_eq!(unit.trail_length, 7);
        assert!(unit.hidden);
        assert!(!unit.hoverable);
        assert_eq!(unit.speed, 4.0);
        assert_eq!(unit.lifetime, 102.0);
        assert_eq!(unit.rotate_speed, 2.5);
        assert_eq!(unit.range, 6.0);
        assert_eq!(unit.target_priority, -1.0);
        assert_eq!(unit.outline_color_rgba, PAL_DARK_OUTLINE_RGBA);
        assert_eq!(unit.fog_radius, 2.0);
        assert_eq!(unit.loop_sound, "loopMissileTrail");
        assert_eq!(unit.loop_sound_volume, 0.05);
        assert!(!unit.draw_minimap);
    }

    #[test]
    fn missile_defaults_override_base_unit_runtime_flags() {
        let mut unit = UnitType::new(0, "custom-missile");
        unit.allowed_in_payloads = true;
        unit.physics = true;
        unit.bounded = true;

        apply_missile_unit_type_defaults(&mut unit);

        assert!(!unit.allowed_in_payloads);
        assert!(!unit.physics);
        assert!(!unit.bounded);
        assert_eq!(unit.env_enabled, Env::ANY);
    }
}
