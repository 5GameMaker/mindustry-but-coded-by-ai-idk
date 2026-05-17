use crate::mindustry::game::Gamemode;

#[derive(Debug, Clone, PartialEq)]
pub struct Host {
    pub name: String,
    pub address: String,
    pub mapname: String,
    pub description: String,
    pub wave: i32,
    pub players: i32,
    pub player_limit: i32,
    pub version: i32,
    pub version_type: String,
    pub mode: Gamemode,
    pub mode_name: Option<String>,
    pub ping: i32,
    pub port: i32,
}

impl Host {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ping: i32,
        name: impl Into<String>,
        address: impl Into<String>,
        port: i32,
        mapname: impl Into<String>,
        wave: i32,
        players: i32,
        version: i32,
        version_type: impl Into<String>,
        mode: Gamemode,
        player_limit: i32,
        description: impl Into<String>,
        mode_name: Option<String>,
    ) -> Self {
        Self {
            ping,
            name: name.into(),
            address: address.into(),
            port,
            mapname: mapname.into(),
            wave,
            players,
            version,
            version_type: version_type.into(),
            mode,
            player_limit,
            description: description.into(),
            mode_name,
        }
    }
}
