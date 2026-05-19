#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockFlag {
    Core,
    Storage,
    Generator,
    Turret,
    Factory,
    Repair,
    Battery,
    Reactor,
    Extinguisher,
    Drill,
    Shield,
    LaunchPad,
    UnitCargoUnloadPoint,
    UnitAssembler,
    HasFogRadius,
    SteamVent,
    BlockRepair,
    Synced,
}

impl BlockFlag {
    pub const ALL: [BlockFlag; 18] = [
        BlockFlag::Core,
        BlockFlag::Storage,
        BlockFlag::Generator,
        BlockFlag::Turret,
        BlockFlag::Factory,
        BlockFlag::Repair,
        BlockFlag::Battery,
        BlockFlag::Reactor,
        BlockFlag::Extinguisher,
        BlockFlag::Drill,
        BlockFlag::Shield,
        BlockFlag::LaunchPad,
        BlockFlag::UnitCargoUnloadPoint,
        BlockFlag::UnitAssembler,
        BlockFlag::HasFogRadius,
        BlockFlag::SteamVent,
        BlockFlag::BlockRepair,
        BlockFlag::Synced,
    ];
    pub const ALL_LOGIC: [BlockFlag; 10] = [
        BlockFlag::Core,
        BlockFlag::Storage,
        BlockFlag::Generator,
        BlockFlag::Turret,
        BlockFlag::Factory,
        BlockFlag::Repair,
        BlockFlag::Battery,
        BlockFlag::Reactor,
        BlockFlag::Drill,
        BlockFlag::Shield,
    ];

    pub const WIRE_NAMES: [&'static str; 18] = [
        "core",
        "storage",
        "generator",
        "turret",
        "factory",
        "repair",
        "battery",
        "reactor",
        "extinguisher",
        "drill",
        "shield",
        "launchPad",
        "unitCargoUnloadPoint",
        "unitAssembler",
        "hasFogRadius",
        "steamVent",
        "blockRepair",
        "synced",
    ];

    pub fn ordinal(self) -> u8 {
        Self::ALL
            .iter()
            .position(|value| *value == self)
            .expect("BlockFlag::ALL must contain every variant") as u8
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

    pub fn is_logic(self) -> bool {
        Self::ALL_LOGIC.contains(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_flag_wire_names_and_logic_subset_match_java_order() {
        assert_eq!(BlockFlag::ALL.len(), 18);
        assert_eq!(BlockFlag::ALL_LOGIC.len(), 10);
        assert_eq!(BlockFlag::Core.ordinal(), 0);
        assert_eq!(BlockFlag::Synced.ordinal(), 17);
        assert_eq!(BlockFlag::from_ordinal(18), None);

        assert_eq!(BlockFlag::Core.wire_name(), "core");
        assert_eq!(BlockFlag::LaunchPad.wire_name(), "launchPad");
        assert_eq!(BlockFlag::BlockRepair.wire_name(), "blockRepair");
        assert_eq!(BlockFlag::by_wire_name("shield"), Some(BlockFlag::Shield));
        assert_eq!(
            BlockFlag::by_wire_name("unitCargoUnloadPoint"),
            Some(BlockFlag::UnitCargoUnloadPoint)
        );
        assert_eq!(BlockFlag::by_wire_name("missing"), None);

        assert!(BlockFlag::Core.is_logic());
        assert!(BlockFlag::Shield.is_logic());
        assert!(!BlockFlag::LaunchPad.is_logic());
        assert!(!BlockFlag::Synced.is_logic());
    }
}
