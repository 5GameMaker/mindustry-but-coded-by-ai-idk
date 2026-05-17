#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorldParams {
    pub seed_offset: i32,
    pub save_info: bool,
    pub core_position_override: i32,
}

impl WorldParams {
    pub fn new() -> Self {
        Self {
            seed_offset: 0,
            save_info: true,
            core_position_override: 0,
        }
    }
}
