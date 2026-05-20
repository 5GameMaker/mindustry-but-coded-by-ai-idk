use crate::mindustry::{ctype::ContentId, r#type::UnitType, world::meta::Env};

pub const PAL_DARK_OUTLINE_RGBA: u32 = 0x2b2f36ff;

pub fn erekir_unit_type(id: ContentId, name: impl Into<String>) -> UnitType {
    let mut unit = UnitType::new(id, name);
    apply_erekir_unit_type_defaults(&mut unit);
    unit
}

pub fn apply_erekir_unit_type_defaults(unit: &mut UnitType) {
    unit.outline_color_rgba = PAL_DARK_OUTLINE_RGBA;
    unit.env_disabled = Env::SPACE;
    unit.ammo_type = "item:beryllium".into();
    unit.research_cost_multiplier = 10.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::ctype::{Content, ContentType};

    #[test]
    fn erekir_unit_type_constructor_matches_java_preset_fields() {
        let unit = erekir_unit_type(7, "merui");

        assert_eq!(unit.id(), 7);
        assert_eq!(unit.content_type(), ContentType::Unit);
        assert_eq!(unit.name(), "merui");
        assert_eq!(unit.outline_color_rgba, PAL_DARK_OUTLINE_RGBA);
        assert_eq!(unit.env_disabled, Env::SPACE);
        assert_eq!(unit.ammo_type, "item:beryllium");
        assert_eq!(unit.research_cost_multiplier, 10.0);
    }

    #[test]
    fn erekir_defaults_can_be_applied_to_existing_unit_type() {
        let mut unit = UnitType::new(0, "custom");
        unit.research_cost_multiplier = 0.0;
        unit.ammo_type = "item:copper".into();

        apply_erekir_unit_type_defaults(&mut unit);

        assert_eq!(unit.outline_color_rgba, PAL_DARK_OUTLINE_RGBA);
        assert_eq!(unit.env_disabled, Env::SPACE);
        assert_eq!(unit.ammo_type, "item:beryllium");
        assert_eq!(unit.research_cost_multiplier, 10.0);
    }
}
