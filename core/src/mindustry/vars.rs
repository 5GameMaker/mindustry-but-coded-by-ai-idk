pub const APP_NAME: &str = "Mindustry";
pub const MIN_MOD_GAME_VERSION: i32 = 136;
pub const MIN_JAVA_MOD_GAME_VERSION: i32 = 154;
pub const SCHEMATIC_BASE_START: &str = "bXNjaA";
pub const BUFFER_SIZE: usize = 8192;
pub const MAX_TCP_SIZE: usize = 1100;
pub const DEFAULT_PORT: u16 = 6567;
pub const MULTICAST_PORT: u16 = 20151;
pub const MULTICAST_GROUP: &str = "227.2.7.7";
pub const MAX_TEXT_LENGTH: usize = 150;
pub const MAX_PING_TEXT_LENGTH: usize = 40;
pub const MAX_NAME_LENGTH: usize = 40;
pub const MAX_PLAYER_PREVIEW_PLANS: usize = 1000;
pub const TILE_SIZE: i32 = 8;
pub const DARK_RADIUS: i32 = 4;
pub const BUILDING_RANGE: f32 = 220.0;
pub const MINE_TRANSFER_RANGE: f32 = 220.0;
pub const ITEM_TRANSFER_RANGE: f32 = 220.0;
pub const LOGIC_ITEM_TRANSFER_RANGE: f32 = 45.0;
pub const SERVER_CACHE_FILE_NAME: &str = "server_list.json";

pub const SERVER_JSON_URLS: [&str; 2] = [
    "https://raw.githubusercontent.com/Anuken/MindustryServerList/master/servers_v8.json",
    "https://cdn.jsdelivr.net/gh/anuken/mindustryserverlist/servers_v8.json",
];

pub const SERVER_JSON_BE_URLS: [&str; 2] = [
    "https://raw.githubusercontent.com/Anuken/MindustryServerList/master/servers_be.json",
    "https://cdn.jsdelivr.net/gh/anuken/mindustryserverlist/servers_be.json",
];

pub const MOD_JSON_URLS: [&str; 2] = [
    "https://raw.githubusercontent.com/Anuken/MindustryMods/master/mods.json",
    "https://cdn.jsdelivr.net/gh/anuken/mindustrymods/mods.json",
];

pub const STEAM_BANS_URLS: [&str; 2] = [
    "https://raw.githubusercontent.com/Anuken/MindustrySteamBans/master/data.json",
    "https://cdn.jsdelivr.net/gh/anuken/mindustrysteambans/data.json",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeMode {
    Client,
    Server,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFlags {
    pub mode: RuntimeMode,
    pub load_locales: bool,
    pub force_be_servers: bool,
    pub skip_mod_code: bool,
    pub headless: bool,
    pub mobile: bool,
    pub android: bool,
    pub ios: bool,
    pub steam: bool,
    pub disable_ui: bool,
    pub disable_save: bool,
    pub test_mobile: bool,
}

impl Default for RuntimeFlags {
    fn default() -> Self {
        Self {
            mode: RuntimeMode::Client,
            load_locales: true,
            force_be_servers: false,
            skip_mod_code: false,
            headless: false,
            mobile: false,
            android: false,
            ios: false,
            steam: false,
            disable_ui: false,
            disable_save: false,
            test_mobile: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePaths {
    pub data_dir: String,
    pub screenshot_dir: String,
    pub map_dir: String,
    pub save_dir: String,
    pub mod_dir: String,
    pub schematic_dir: String,
}

impl RuntimePaths {
    pub fn from_data_dir(data_dir: impl Into<String>) -> Self {
        let data_dir = data_dir.into();
        Self {
            screenshot_dir: format!("{data_dir}/screenshots"),
            map_dir: format!("{data_dir}/maps"),
            save_dir: format!("{data_dir}/saves"),
            mod_dir: format!("{data_dir}/mods"),
            schematic_dir: format!("{data_dir}/schematics"),
            data_dir,
        }
    }

    pub fn server_cache_file(&self) -> String {
        format!("{}/{}", self.data_dir, SERVER_CACHE_FILE_NAME)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppContext {
    pub flags: RuntimeFlags,
    pub paths: RuntimePaths,
    pub port: u16,
    pub steam_player_name: String,
}

impl AppContext {
    pub fn new(data_dir: impl Into<String>) -> Self {
        Self {
            flags: RuntimeFlags::default(),
            paths: RuntimePaths::from_data_dir(data_dir),
            port: DEFAULT_PORT,
            steam_player_name: String::new(),
        }
    }

    pub fn server(data_dir: impl Into<String>) -> Self {
        let mut context = Self::new(data_dir);
        context.flags.mode = RuntimeMode::Server;
        context.flags.headless = true;
        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_paths_use_java_server_cache_filename() {
        let paths = RuntimePaths::from_data_dir("data");

        assert_eq!(SERVER_CACHE_FILE_NAME, "server_list.json");
        assert_eq!(paths.server_cache_file(), "data/server_list.json");
    }
}
