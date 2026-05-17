#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CampaignStats {
    pub units_destroyed: i32,
    pub units_lost: i32,
    pub buildings_built: i32,
    pub buildings_destroyed: i32,
    pub sectors_captured: i32,
    pub play_time: i64,
}
