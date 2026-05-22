//! Mirrors upstream `mindustry.logic.FetchType`.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FetchType {
    Unit,
    UnitCount,
    Player,
    PlayerCount,
    Core,
    CoreCount,
    Build,
    BuildCount,
}

impl FetchType {
    pub const ALL: [FetchType; 8] = [
        FetchType::Unit,
        FetchType::UnitCount,
        FetchType::Player,
        FetchType::PlayerCount,
        FetchType::Core,
        FetchType::CoreCount,
        FetchType::Build,
        FetchType::BuildCount,
    ];

    pub const WIRE_NAMES: [&'static str; 8] = [
        "unit",
        "unitCount",
        "player",
        "playerCount",
        "core",
        "coreCount",
        "build",
        "buildCount",
    ];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::FetchType;

    #[test]
    fn fetch_type_order_and_wire_names_match_java_enum() {
        assert_eq!(
            FetchType::ALL,
            [
                FetchType::Unit,
                FetchType::UnitCount,
                FetchType::Player,
                FetchType::PlayerCount,
                FetchType::Core,
                FetchType::CoreCount,
                FetchType::Build,
                FetchType::BuildCount
            ]
        );
        assert_eq!(FetchType::Unit.ordinal(), 0);
        assert_eq!(FetchType::BuildCount.ordinal(), 7);
        assert_eq!(FetchType::PlayerCount.wire_name(), "playerCount");
        assert_eq!(FetchType::from_ordinal(8), None);
        assert_eq!(FetchType::by_wire_name("core"), Some(FetchType::Core));
        assert_eq!(FetchType::by_wire_name("missing"), None);
    }
}
