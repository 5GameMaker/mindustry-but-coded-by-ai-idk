#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockStatus {
    Active,
    NoOutput,
    NoInput,
    LogicDisable,
    Inactive,
}

impl BlockStatus {
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
