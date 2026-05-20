use crate::mindustry::{ctype::ContentId, r#type::UnitType, world::meta::Env};

pub const PAL_NEOPLASM_OUTLINE_RGBA: u32 = 0x6b2b2bff;
pub const PAL_NEOPLASM1_RGBA: u32 = 0xc33e2bff;

pub fn neoplasm_unit_type(id: ContentId, name: impl Into<String>) -> UnitType {
    let mut unit = UnitType::new(id, name);
    apply_neoplasm_unit_type_defaults(&mut unit);
    unit
}

pub fn apply_neoplasm_unit_type_defaults(unit: &mut UnitType) {
    unit.outline_color_rgba = PAL_NEOPLASM_OUTLINE_RGBA;
    push_unique(&mut unit.immunities, "burning");
    push_unique(&mut unit.immunities, "melting");
    unit.env_disabled = Env::NONE;
    unit.draw_cell = false;

    push_unique(&mut unit.abilities, "RegenAbility");
    push_unique(&mut unit.abilities, "LiquidExplodeAbility:neoplasm");
    push_unique(
        &mut unit.abilities,
        "LiquidRegenAbility:neoplasm:neoplasmHeal",
    );

    unit.heal_flash = true;
    unit.heal_color_rgba = PAL_NEOPLASM1_RGBA;
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::ctype::{Content, ContentType};

    #[test]
    fn neoplasm_unit_type_constructor_matches_java_preset_fields() {
        let unit = neoplasm_unit_type(54, "renale");

        assert_eq!(unit.id(), 54);
        assert_eq!(unit.content_type(), ContentType::Unit);
        assert_eq!(unit.name(), "renale");
        assert_eq!(unit.outline_color_rgba, PAL_NEOPLASM_OUTLINE_RGBA);
        assert_eq!(unit.env_disabled, Env::NONE);
        assert!(!unit.draw_cell);
        assert!(unit.heal_flash);
        assert_eq!(unit.heal_color_rgba, PAL_NEOPLASM1_RGBA);
        assert!(unit.immunities.iter().any(|entry| entry == "burning"));
        assert!(unit.immunities.iter().any(|entry| entry == "melting"));
        assert!(unit.abilities.iter().any(|entry| entry == "RegenAbility"));
        assert!(unit
            .abilities
            .iter()
            .any(|entry| entry == "LiquidExplodeAbility:neoplasm"));
        assert!(unit
            .abilities
            .iter()
            .any(|entry| entry == "LiquidRegenAbility:neoplasm:neoplasmHeal"));
    }

    #[test]
    fn neoplasm_defaults_do_not_duplicate_marker_entries() {
        let mut unit = UnitType::new(0, "latum");
        apply_neoplasm_unit_type_defaults(&mut unit);
        apply_neoplasm_unit_type_defaults(&mut unit);

        assert_eq!(
            unit.immunities
                .iter()
                .filter(|entry| entry.as_str() == "burning")
                .count(),
            1
        );
        assert_eq!(
            unit.abilities
                .iter()
                .filter(|entry| entry.as_str() == "RegenAbility")
                .count(),
            1
        );
    }
}
