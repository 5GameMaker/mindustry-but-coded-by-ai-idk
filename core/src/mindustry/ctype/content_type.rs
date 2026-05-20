#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ContentType {
    Item = 0,
    Block = 1,
    MechUnused = 2,
    Bullet = 3,
    Liquid = 4,
    Status = 5,
    Unit = 6,
    Weather = 7,
    EffectUnused = 8,
    Sector = 9,
    LoadoutUnused = 10,
    TypeidUnused = 11,
    Error = 12,
    Planet = 13,
    AmmoUnused = 14,
    Team = 15,
    UnitCommand = 16,
    UnitStance = 17,
}

impl ContentType {
    pub const ALL: [ContentType; 18] = [
        ContentType::Item,
        ContentType::Block,
        ContentType::MechUnused,
        ContentType::Bullet,
        ContentType::Liquid,
        ContentType::Status,
        ContentType::Unit,
        ContentType::Weather,
        ContentType::EffectUnused,
        ContentType::Sector,
        ContentType::LoadoutUnused,
        ContentType::TypeidUnused,
        ContentType::Error,
        ContentType::Planet,
        ContentType::AmmoUnused,
        ContentType::Team,
        ContentType::UnitCommand,
        ContentType::UnitStance,
    ];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn from_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|content_type| content_type.wire_name() == name)
    }

    pub const fn wire_name(self) -> &'static str {
        match self {
            ContentType::Item => "item",
            ContentType::Block => "block",
            ContentType::MechUnused => "mech_UNUSED",
            ContentType::Bullet => "bullet",
            ContentType::Liquid => "liquid",
            ContentType::Status => "status",
            ContentType::Unit => "unit",
            ContentType::Weather => "weather",
            ContentType::EffectUnused => "effect_UNUSED",
            ContentType::Sector => "sector",
            ContentType::LoadoutUnused => "loadout_UNUSED",
            ContentType::TypeidUnused => "typeid_UNUSED",
            ContentType::Error => "error",
            ContentType::Planet => "planet",
            ContentType::AmmoUnused => "ammo_UNUSED",
            ContentType::Team => "team",
            ContentType::UnitCommand => "unitCommand",
            ContentType::UnitStance => "unitStance",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ContentType;

    #[test]
    fn content_type_ordinals_match_upstream_order() {
        for (index, content_type) in ContentType::ALL.iter().copied().enumerate() {
            assert_eq!(content_type.ordinal(), index as u8);
            assert_eq!(ContentType::from_ordinal(index as u8), Some(content_type));
        }
        assert_eq!(ContentType::from_ordinal(18), None);
    }

    #[test]
    fn content_type_wire_names_roundtrip() {
        for content_type in ContentType::ALL {
            assert_eq!(
                ContentType::from_wire_name(content_type.wire_name()),
                Some(content_type)
            );
        }
        assert_eq!(
            ContentType::from_wire_name("unitCommand"),
            Some(ContentType::UnitCommand)
        );
        assert_eq!(
            ContentType::from_wire_name("mech_UNUSED"),
            Some(ContentType::MechUnused)
        );
        assert_eq!(ContentType::from_wire_name("missing"), None);
    }
}
