//! Side-effect-free mirror of upstream `mindustry.net.SteamAdmin`.
//!
//! Java performs asynchronous HTTP requests, schedules a periodic timer and
//! kicks newly banned players from global `Groups.player`.  This Rust layer
//! keeps the same decisions as explicit fetch plans and kick lists so runtime
//! adapters can provide the actual HTTP/timer/player side effects.

use super::{SteamAdminData, SteamAdminParseError};

pub const STEAM_ADMIN_CHECK_INTERVAL_SECONDS: f32 = 60.0 * 5.0;
pub const STEAM_ADMIN_DATA_PATH: &str = "data.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamAdminPlayer {
    pub connection_id: i32,
    pub steam_id: String,
}

impl SteamAdminPlayer {
    pub fn new(connection_id: i32, steam_id: impl Into<String>) -> Self {
        Self {
            connection_id,
            steam_id: steam_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SteamAdminFetchPlan {
    Disabled,
    FetchLatestCommit {
        commits_url: String,
        primary_url: String,
        fallback_url: String,
    },
    FetchData {
        url: String,
        fallback_url: Option<String>,
        schedule_interval_seconds: Option<f32>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamAdminApplyResult {
    pub kicked_connection_ids: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SteamAdmin {
    scheduled: bool,
    data: SteamAdminData,
}

impl SteamAdmin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scheduled(&self) -> bool {
        self.scheduled
    }

    pub fn data(&self) -> &SteamAdminData {
        &self.data
    }

    pub fn set_data(&mut self, data: SteamAdminData) {
        self.data = data;
    }

    pub fn fetch_plan(
        &self,
        steam_enabled: bool,
        cache_bust: bool,
        github_api: &str,
        primary_url: &str,
        fallback_url: &str,
    ) -> SteamAdminFetchPlan {
        if !steam_enabled {
            return SteamAdminFetchPlan::Disabled;
        }

        if cache_bust {
            SteamAdminFetchPlan::FetchLatestCommit {
                commits_url: latest_commit_url(github_api),
                primary_url: primary_url.to_string(),
                fallback_url: fallback_url.to_string(),
            }
        } else {
            self.fetch_data_plan(primary_url, Some(fallback_url))
        }
    }

    pub fn fetch_data_plan(&self, url: &str, fallback_url: Option<&str>) -> SteamAdminFetchPlan {
        SteamAdminFetchPlan::FetchData {
            url: url.to_string(),
            fallback_url: fallback_url.map(ToString::to_string),
            schedule_interval_seconds: (!self.scheduled)
                .then_some(STEAM_ADMIN_CHECK_INTERVAL_SECONDS),
        }
    }

    pub fn mark_scheduled(&mut self) {
        self.scheduled = true;
    }

    pub fn plan_after_latest_commit(
        &self,
        commit_sha: Option<&str>,
        primary_url: &str,
        fallback_url: &str,
    ) -> SteamAdminFetchPlan {
        let url = commit_sha
            .map(|sha| cache_busted_primary_url(primary_url, sha))
            .unwrap_or_else(|| primary_url.to_string());
        self.fetch_data_plan(&url, Some(fallback_url))
    }

    pub fn fallback_after_error(&self, plan: &SteamAdminFetchPlan) -> Option<SteamAdminFetchPlan> {
        match plan {
            SteamAdminFetchPlan::FetchData {
                fallback_url: Some(fallback_url),
                ..
            } => Some(self.fetch_data_plan(fallback_url, None)),
            _ => None,
        }
    }

    pub fn apply_data_text(
        &mut self,
        text: &str,
        players: &[SteamAdminPlayer],
    ) -> Result<SteamAdminApplyResult, SteamAdminParseError> {
        let data = SteamAdminData::from_json_text(text)?;
        let kicked_connection_ids = players
            .iter()
            .filter(|player| data.is_banned(&player.steam_id))
            .map(|player| player.connection_id)
            .collect();
        self.data = data;
        Ok(SteamAdminApplyResult {
            kicked_connection_ids,
        })
    }

    pub fn is_banned(&self, id: &str) -> bool {
        self.data.is_banned(id)
    }

    pub fn is_admin(&self, id: &str) -> bool {
        self.data.is_admin(id)
    }
}

pub fn latest_commit_url(github_api: &str) -> String {
    format!(
        "{}/repos/Anuken/MindustrySteamBans/commits?path={}&per_page=1",
        github_api.trim_end_matches('/'),
        STEAM_ADMIN_DATA_PATH
    )
}

pub fn cache_busted_primary_url(primary_url: &str, commit_sha: &str) -> String {
    primary_url.replacen("master", commit_sha, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_plan_ignores_non_steam_runtime_like_java() {
        let admin = SteamAdmin::new();

        assert_eq!(
            admin.fetch_plan(
                false,
                false,
                "https://api.github.com",
                "https://raw.githubusercontent.com/Anuken/MindustrySteamBans/master/data.json",
                "https://mirror/data.json",
            ),
            SteamAdminFetchPlan::Disabled
        );
    }

    #[test]
    fn fetch_plan_supports_cache_bust_commit_lookup() {
        let admin = SteamAdmin::new();
        let plan = admin.fetch_plan(
            true,
            true,
            "https://api.github.com/",
            "https://raw.githubusercontent.com/Anuken/MindustrySteamBans/master/data.json",
            "https://mirror/data.json",
        );

        assert_eq!(
            plan,
            SteamAdminFetchPlan::FetchLatestCommit {
                commits_url: "https://api.github.com/repos/Anuken/MindustrySteamBans/commits?path=data.json&per_page=1".into(),
                primary_url:
                    "https://raw.githubusercontent.com/Anuken/MindustrySteamBans/master/data.json"
                        .into(),
                fallback_url: "https://mirror/data.json".into(),
            }
        );
        assert_eq!(
            cache_busted_primary_url("https://host/master/data.json", "abc123"),
            "https://host/abc123/data.json"
        );
    }

    #[test]
    fn fetch_data_plan_schedules_only_first_fetch_impl_like_java() {
        let mut admin = SteamAdmin::new();

        assert_eq!(
            admin.fetch_data_plan("primary", Some("fallback")),
            SteamAdminFetchPlan::FetchData {
                url: "primary".into(),
                fallback_url: Some("fallback".into()),
                schedule_interval_seconds: Some(STEAM_ADMIN_CHECK_INTERVAL_SECONDS),
            }
        );

        admin.mark_scheduled();
        assert_eq!(
            admin.fetch_data_plan("primary", Some("fallback")),
            SteamAdminFetchPlan::FetchData {
                url: "primary".into(),
                fallback_url: Some("fallback".into()),
                schedule_interval_seconds: None,
            }
        );
    }

    #[test]
    fn failed_primary_fetch_falls_back_once() {
        let admin = SteamAdmin::new();
        let primary = admin.fetch_data_plan("primary", Some("fallback"));
        let fallback = admin.fallback_after_error(&primary).unwrap();

        assert_eq!(
            fallback,
            SteamAdminFetchPlan::FetchData {
                url: "fallback".into(),
                fallback_url: None,
                schedule_interval_seconds: Some(STEAM_ADMIN_CHECK_INTERVAL_SECONDS),
            }
        );
        assert!(admin.fallback_after_error(&fallback).is_none());
    }

    #[test]
    fn apply_data_text_updates_database_and_kicks_newly_banned_players() {
        let mut admin = SteamAdmin::new();
        let players = vec![
            SteamAdminPlayer::new(1, "steam:111"),
            SteamAdminPlayer::new(2, "steam:222"),
            SteamAdminPlayer::new(3, "not-steam:111"),
        ];
        let result = admin
            .apply_data_text(r#"{"bans":["111"],"admins":["222"]}"#, &players)
            .unwrap();

        assert_eq!(result.kicked_connection_ids, vec![1]);
        assert!(admin.is_banned("steam:111"));
        assert!(admin.is_admin("steam:222"));
        assert!(!admin.is_banned("111"));
    }

    #[test]
    fn latest_commit_failure_uses_uncached_primary_url() {
        let admin = SteamAdmin::new();

        assert_eq!(
            admin.plan_after_latest_commit(None, "https://host/master/data.json", "fallback"),
            SteamAdminFetchPlan::FetchData {
                url: "https://host/master/data.json".into(),
                fallback_url: Some("fallback".into()),
                schedule_interval_seconds: Some(STEAM_ADMIN_CHECK_INTERVAL_SECONDS),
            }
        );
    }
}
