pub mod ai;
pub mod r#async;
pub mod audio;
pub mod client_launcher;
pub mod content;
pub mod core;
pub mod ctype;
pub mod editor;
pub mod entities;
pub mod game;
pub mod graphics;
pub mod input;
pub mod io;
pub mod logic;
pub mod maps;
pub mod modsys;
pub mod net;
pub mod service;
pub mod r#type;
pub mod ui;
pub mod vars;
pub mod world;

pub const UPSTREAM_BASELINE: &str = "mindustry-upstream-v158.1";
pub const UPSTREAM_VERSION_TYPE: &str = "official";
pub const UPSTREAM_VERSION_MODIFIER: &str = "release";
pub const UPSTREAM_VERSION_BUILD: i32 = 158;
pub const UPSTREAM_VERSION_REVISION: i32 = 1;
pub const UPSTREAM_VERSION_COMMIT_HASH: &str = "unknown";
pub const UPSTREAM_MENU_VERSION_TEXT_OVERRIDE: Option<&str> = None;

pub fn upstream_version_info() -> core::version::VersionInfo {
    core::version::VersionInfo {
        build_type: UPSTREAM_VERSION_TYPE.to_string(),
        modifier: UPSTREAM_VERSION_MODIFIER.to_string(),
        commit_hash: UPSTREAM_VERSION_COMMIT_HASH.to_string(),
        build_date: "unknown".to_string(),
        number: 8,
        build: UPSTREAM_VERSION_BUILD,
        revision: UPSTREAM_VERSION_REVISION,
        is_steam: UPSTREAM_VERSION_MODIFIER.contains("steam"),
        enabled: true,
    }
}

pub fn upstream_menu_version_text() -> String {
    UPSTREAM_MENU_VERSION_TEXT_OVERRIDE
        .map(str::to_string)
        .unwrap_or_else(|| upstream_version_info().combined())
}

pub fn upstream_menu_version_color() -> [f32; 4] {
    if UPSTREAM_MENU_VERSION_TEXT_OVERRIDE == Some("custom build") || UPSTREAM_VERSION_BUILD == -1 {
        [0.988, 0.506, 0.251, 0.667]
    } else {
        [1.0, 1.0, 1.0, 0.729]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_menu_version_uses_java_version_combined_shape() {
        let info = upstream_version_info();

        assert_eq!(info.build_type, "official");
        assert_eq!(info.modifier, "release");
        assert_eq!(info.build, 158);
        assert_eq!(info.revision, 1);
        assert_eq!(info.build_string(), "158.1");
        assert_eq!(upstream_menu_version_text(), "release build 158.1");
        assert_eq!(upstream_menu_version_color(), [1.0, 1.0, 1.0, 0.729]);
    }
}
