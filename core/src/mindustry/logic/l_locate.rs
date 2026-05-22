//! Mirrors upstream `mindustry.logic.LLocate`.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LLocate {
    Ore,
    Building,
    Spawn,
    Damaged,
}

impl LLocate {
    pub const ALL: [LLocate; 4] = [
        LLocate::Ore,
        LLocate::Building,
        LLocate::Spawn,
        LLocate::Damaged,
    ];
    pub const WIRE_NAMES: [&'static str; 4] = ["ore", "building", "spawn", "damaged"];

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
    use super::LLocate;

    #[test]
    fn l_locate_order_and_wire_names_match_java_enum() {
        assert_eq!(
            LLocate::ALL,
            [
                LLocate::Ore,
                LLocate::Building,
                LLocate::Spawn,
                LLocate::Damaged,
            ]
        );
        assert_eq!(LLocate::WIRE_NAMES, ["ore", "building", "spawn", "damaged"]);
        assert_eq!(LLocate::Damaged.ordinal(), 3);
        assert_eq!(LLocate::from_ordinal(3), Some(LLocate::Damaged));
        assert_eq!(LLocate::from_ordinal(4), None);
        assert_eq!(LLocate::by_wire_name("spawn"), Some(LLocate::Spawn));
        assert_eq!(LLocate::by_wire_name("missing"), None);
    }
}
