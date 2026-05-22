//! Mirrors upstream `mindustry.logic.LUnitControl`.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LUnitControl {
    Idle,
    Stop,
    Move,
    Approach,
    Pathfind,
    AutoPathfind,
    Boost,
    Target,
    Targetp,
    ItemDrop,
    ItemTake,
    PayDrop,
    PayTake,
    PayEnter,
    Mine,
    Flag,
    Build,
    Deconstruct,
    GetBlock,
    Within,
    Unbind,
}

impl LUnitControl {
    pub const ALL: [LUnitControl; 21] = [
        LUnitControl::Idle,
        LUnitControl::Stop,
        LUnitControl::Move,
        LUnitControl::Approach,
        LUnitControl::Pathfind,
        LUnitControl::AutoPathfind,
        LUnitControl::Boost,
        LUnitControl::Target,
        LUnitControl::Targetp,
        LUnitControl::ItemDrop,
        LUnitControl::ItemTake,
        LUnitControl::PayDrop,
        LUnitControl::PayTake,
        LUnitControl::PayEnter,
        LUnitControl::Mine,
        LUnitControl::Flag,
        LUnitControl::Build,
        LUnitControl::Deconstruct,
        LUnitControl::GetBlock,
        LUnitControl::Within,
        LUnitControl::Unbind,
    ];

    pub const WIRE_NAMES: [&'static str; 21] = [
        "idle",
        "stop",
        "move",
        "approach",
        "pathfind",
        "autoPathfind",
        "boost",
        "target",
        "targetp",
        "itemDrop",
        "itemTake",
        "payDrop",
        "payTake",
        "payEnter",
        "mine",
        "flag",
        "build",
        "deconstruct",
        "getBlock",
        "within",
        "unbind",
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

    pub const fn params(self) -> &'static [&'static str] {
        match self {
            LUnitControl::Move => &["x", "y"],
            LUnitControl::Approach => &["x", "y", "radius"],
            LUnitControl::Pathfind => &["x", "y"],
            LUnitControl::Boost => &["enable"],
            LUnitControl::Target => &["x", "y", "shoot"],
            LUnitControl::Targetp => &["unit", "shoot"],
            LUnitControl::ItemDrop => &["to", "amount"],
            LUnitControl::ItemTake => &["from", "item", "amount"],
            LUnitControl::PayTake => &["takeUnits"],
            LUnitControl::Mine => &["x", "y"],
            LUnitControl::Flag => &["value"],
            LUnitControl::Build => &["x", "y", "block", "rotation", "config"],
            LUnitControl::Deconstruct => &["x", "y"],
            LUnitControl::GetBlock => &["x", "y", "type", "building", "floor"],
            LUnitControl::Within => &["x", "y", "radius", "result"],
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LUnitControl;

    #[test]
    fn l_unit_control_order_names_and_params_match_java_enum() {
        assert_eq!(LUnitControl::ALL.len(), 21);
        assert_eq!(LUnitControl::Idle.ordinal(), 0);
        assert_eq!(LUnitControl::AutoPathfind.ordinal(), 5);
        assert_eq!(LUnitControl::Unbind.ordinal(), 20);
        assert_eq!(LUnitControl::from_ordinal(21), None);
        assert_eq!(
            LUnitControl::ALL
                .iter()
                .map(|value| value.wire_name())
                .collect::<Vec<_>>(),
            LUnitControl::WIRE_NAMES.to_vec()
        );
        assert_eq!(
            LUnitControl::by_wire_name("autoPathfind"),
            Some(LUnitControl::AutoPathfind)
        );
        assert_eq!(
            LUnitControl::by_wire_name("getBlock"),
            Some(LUnitControl::GetBlock)
        );
        assert_eq!(LUnitControl::by_wire_name("missing"), None);
        assert_eq!(LUnitControl::Move.params(), &["x", "y"]);
        assert_eq!(LUnitControl::Targetp.params(), &["unit", "shoot"]);
        assert_eq!(
            LUnitControl::Build.params(),
            &["x", "y", "block", "rotation", "config"]
        );
        assert_eq!(
            LUnitControl::GetBlock.params(),
            &["x", "y", "type", "building", "floor"]
        );
        assert_eq!(
            LUnitControl::Within.params(),
            &["x", "y", "radius", "result"]
        );
        assert_eq!(LUnitControl::PayDrop.params(), &[] as &[&str]);
    }
}
