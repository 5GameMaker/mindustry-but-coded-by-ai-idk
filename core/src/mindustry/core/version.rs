#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    pub build_type: String,
    pub modifier: String,
    pub commit_hash: String,
    pub build_date: String,
    pub number: i32,
    pub build: i32,
    pub revision: i32,
    pub is_steam: bool,
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self {
            build_type: "unknown".into(),
            modifier: "unknown".into(),
            commit_hash: "unknown".into(),
            build_date: "unknown".into(),
            number: 0,
            build: 0,
            revision: 0,
            is_steam: false,
        }
    }
}
