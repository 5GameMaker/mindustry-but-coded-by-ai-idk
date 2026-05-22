//! Mirrors upstream `mindustry.logic.CutsceneAction`.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CutsceneAction {
    Pan,
    Zoom,
    Stop,
}

impl CutsceneAction {
    pub const ALL: [CutsceneAction; 3] = [
        CutsceneAction::Pan,
        CutsceneAction::Zoom,
        CutsceneAction::Stop,
    ];
    pub const WIRE_NAMES: [&'static str; 3] = ["pan", "zoom", "stop"];

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
    use super::CutsceneAction;

    #[test]
    fn cutscene_action_order_and_wire_names_match_java_enum() {
        assert_eq!(
            CutsceneAction::ALL,
            [
                CutsceneAction::Pan,
                CutsceneAction::Zoom,
                CutsceneAction::Stop
            ]
        );
        assert_eq!(CutsceneAction::Pan.ordinal(), 0);
        assert_eq!(CutsceneAction::Zoom.ordinal(), 1);
        assert_eq!(CutsceneAction::Stop.ordinal(), 2);
        assert_eq!(CutsceneAction::Stop.wire_name(), "stop");
        assert_eq!(CutsceneAction::from_ordinal(3), None);
        assert_eq!(
            CutsceneAction::by_wire_name("zoom"),
            Some(CutsceneAction::Zoom)
        );
        assert_eq!(CutsceneAction::by_wire_name("missing"), None);
    }
}
