//! Mirrors upstream `mindustry.logic.TileLayer`.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileLayer {
    Floor,
    Ore,
    Block,
    Building,
}

impl TileLayer {
    pub const ALL: [TileLayer; 4] = [
        TileLayer::Floor,
        TileLayer::Ore,
        TileLayer::Block,
        TileLayer::Building,
    ];
    pub const SETTABLE: [TileLayer; 3] = [TileLayer::Floor, TileLayer::Ore, TileLayer::Block];
    pub const WIRE_NAMES: [&'static str; 4] = ["floor", "ore", "block", "building"];

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

    pub const fn is_settable(self) -> bool {
        matches!(self, TileLayer::Floor | TileLayer::Ore | TileLayer::Block)
    }
}

#[cfg(test)]
mod tests {
    use super::TileLayer;

    #[test]
    fn tile_layer_order_settable_and_wire_names_match_java_enum() {
        assert_eq!(
            TileLayer::ALL,
            [
                TileLayer::Floor,
                TileLayer::Ore,
                TileLayer::Block,
                TileLayer::Building
            ]
        );
        assert_eq!(
            TileLayer::SETTABLE,
            [TileLayer::Floor, TileLayer::Ore, TileLayer::Block]
        );
        assert_eq!(TileLayer::Floor.ordinal(), 0);
        assert_eq!(TileLayer::Building.ordinal(), 3);
        assert!(TileLayer::Floor.is_settable());
        assert!(TileLayer::Block.is_settable());
        assert!(!TileLayer::Building.is_settable());
        assert_eq!(TileLayer::Building.wire_name(), "building");
        assert_eq!(TileLayer::from_ordinal(4), None);
        assert_eq!(TileLayer::by_wire_name("ore"), Some(TileLayer::Ore));
        assert_eq!(TileLayer::by_wire_name("missing"), None);
    }
}
