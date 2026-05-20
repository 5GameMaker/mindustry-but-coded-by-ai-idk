#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockStatus {
    Active,
    NoOutput,
    NoInput,
    LogicDisable,
    Inactive,
}

impl BlockStatus {
    pub const ALL: [BlockStatus; 5] = [
        BlockStatus::Active,
        BlockStatus::NoOutput,
        BlockStatus::NoInput,
        BlockStatus::LogicDisable,
        BlockStatus::Inactive,
    ];

    pub fn ordinal(self) -> u8 {
        Self::ALL
            .iter()
            .position(|value| *value == self)
            .expect("BlockStatus::ALL must contain every variant") as u8
    }

    pub const fn color_rgba(self) -> u32 {
        match self {
            BlockStatus::Active => 0x5ce677ff,
            BlockStatus::NoOutput => 0xffa500ff,
            BlockStatus::NoInput => 0xff5555ff,
            BlockStatus::LogicDisable => 0x8a73c6ff,
            BlockStatus::Inactive => 0xd3d3d3ff,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BlockStatus;

    #[test]
    fn block_status_order_and_colors_match_java_enum() {
        assert_eq!(BlockStatus::ALL.len(), 5);
        assert_eq!(BlockStatus::Active.ordinal(), 0);
        assert_eq!(BlockStatus::NoOutput.ordinal(), 1);
        assert_eq!(BlockStatus::NoInput.ordinal(), 2);
        assert_eq!(BlockStatus::LogicDisable.ordinal(), 3);
        assert_eq!(BlockStatus::Inactive.ordinal(), 4);

        assert_eq!(BlockStatus::Active.color_rgba(), 0x5ce677ff);
        assert_eq!(BlockStatus::NoOutput.color_rgba(), 0xffa500ff);
        assert_eq!(BlockStatus::NoInput.color_rgba(), 0xff5555ff);
        assert_eq!(BlockStatus::LogicDisable.color_rgba(), 0x8a73c6ff);
        assert_eq!(BlockStatus::Inactive.color_rgba(), 0xd3d3d3ff);
    }
}
