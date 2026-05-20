use crate::mindustry::{
    ctype::ContentId,
    r#type::{unit::erekir_unit_type::apply_erekir_unit_type_defaults, UnitType},
    world::meta::Env,
};

pub fn tank_unit_type(id: ContentId, name: impl Into<String>) -> UnitType {
    let mut unit = UnitType::new(id, name);
    apply_tank_unit_type_defaults(&mut unit);
    unit
}

pub fn apply_tank_unit_type_defaults(unit: &mut UnitType) {
    apply_erekir_unit_type_defaults(unit);
    unit.square_shape = true;
    unit.omni_movement = false;
    unit.rotate_move_first = true;
    unit.rotate_speed = 1.3;
    unit.env_disabled = Env::NONE;
    unit.speed = 0.8;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::{
        ctype::{Content, ContentType},
        r#type::unit::erekir_unit_type::PAL_DARK_OUTLINE_RGBA,
    };

    #[test]
    fn tank_unit_type_constructor_matches_java_preset_fields() {
        let unit = tank_unit_type(39, "stell");

        assert_eq!(unit.id(), 39);
        assert_eq!(unit.content_type(), ContentType::Unit);
        assert_eq!(unit.name(), "stell");
        assert_eq!(unit.outline_color_rgba, PAL_DARK_OUTLINE_RGBA);
        assert_eq!(unit.ammo_type, "item:beryllium");
        assert_eq!(unit.research_cost_multiplier, 10.0);
        assert!(unit.square_shape);
        assert!(!unit.omni_movement);
        assert!(unit.rotate_move_first);
        assert_eq!(unit.rotate_speed, 1.3);
        assert_eq!(unit.env_disabled, Env::NONE);
        assert_eq!(unit.speed, 0.8);
    }

    #[test]
    fn tank_defaults_override_erekir_environment_and_motion_fields() {
        let mut unit = UnitType::new(0, "custom-tank");
        apply_tank_unit_type_defaults(&mut unit);

        assert_eq!(unit.env_disabled, Env::NONE);
        assert_eq!(unit.speed, 0.8);
        assert_eq!(unit.rotate_speed, 1.3);
        assert!(unit.square_shape);
        assert!(unit.rotate_move_first);
    }
}
