#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GameStats {
    pub waves_lasted: i32,
    pub units_created: i32,
    pub enemies_destroyed: i32,
    pub buildings_built: i32,
    pub buildings_deconstructed: i32,
    pub buildings_destroyed: i32,
}
