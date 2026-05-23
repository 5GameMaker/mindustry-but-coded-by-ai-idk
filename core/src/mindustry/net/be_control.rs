//! Side-effect-free mirror of upstream `mindustry.net.BeControl`.
//!
//! Java `BeControl` mixes timers, HTTP requests, dialogs, file replacement and
//! process restarts with global `Vars` state.  This Rust layer keeps the
//! update-check decisions as pure plans plus explicit state transitions so
//! runtime adapters can provide the actual side effects.

pub const BE_UPDATE_INTERVAL_SECONDS: u32 = 60;
pub const BE_RELEASES_LATEST_URL: &str =
    "https://api.github.com/repos/Anuken/MindustryBuilds/releases/latest";
pub const BE_SERVER_ASSET_PREFIX: &str = "Mindustry-BE-Server";
pub const BE_DESKTOP_ASSET_PREFIX: &str = "Mindustry-BE-Desktop";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeRuntimeFlags {
    pub version_type: String,
    pub steam: bool,
    pub client_loaded: bool,
    pub headless: bool,
    pub mobile: bool,
    pub current_build: i32,
}

impl BeRuntimeFlags {
    pub fn new(
        version_type: impl Into<String>,
        steam: bool,
        client_loaded: bool,
        headless: bool,
        mobile: bool,
        current_build: i32,
    ) -> Self {
        Self {
            version_type: version_type.into(),
            steam,
            client_loaded,
            headless,
            mobile,
            current_build,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeInitPlan {
    Disabled,
    ScheduleChecks { interval_seconds: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeUpdateCheckPlan {
    Skip,
    FetchLatestRelease { url: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeReleaseInfo {
    pub tag_name: String,
    pub assets: Vec<BeReleaseAsset>,
}

impl BeReleaseInfo {
    pub fn build_number(&self) -> i32 {
        self.tag_name.trim().parse::<i32>().unwrap_or(0)
    }

    pub fn asset_prefix(headless: bool) -> &'static str {
        if headless {
            BE_SERVER_ASSET_PREFIX
        } else {
            BE_DESKTOP_ASSET_PREFIX
        }
    }

    pub fn find_asset(&self, headless: bool) -> Option<&BeReleaseAsset> {
        let prefix = Self::asset_prefix(headless);
        self.assets
            .iter()
            .find(|asset| asset.name.starts_with(prefix))
    }

    pub fn from_json_text(text: &str) -> Result<Self, BeReleaseParseError> {
        let tag_name = parse_json_string_value(text, "tag_name")?;
        let assets = parse_json_object_array(text, "assets")?
            .into_iter()
            .map(|asset_text| {
                Ok(BeReleaseAsset {
                    name: parse_json_string_value(asset_text, "name")?,
                    browser_download_url: parse_json_string_value(
                        asset_text,
                        "browser_download_url",
                    )?,
                })
            })
            .collect::<Result<Vec<_>, BeReleaseParseError>>()?;

        Ok(Self { tag_name, assets })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeUpdateAvailable {
    pub build: i32,
    pub download_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeDesktopUpdatePlan {
    pub build: i32,
    pub download_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeHeadlessUpdatePlan {
    pub build: i32,
    pub download_url: String,
    pub auto_update: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeUpdateDialogPlan {
    None,
    Desktop(BeDesktopUpdatePlan),
    Headless(BeHeadlessUpdatePlan),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeReleaseParseError {
    MissingString(String),
    MissingArray(String),
    UnterminatedString(String),
    UnterminatedArray(String),
    UnterminatedObject(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeReleaseApplyError {
    MissingAsset {
        expected_prefix: &'static str,
        build: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeReleaseError {
    Parse(BeReleaseParseError),
    Apply(BeReleaseApplyError),
}

impl From<BeReleaseParseError> for BeReleaseError {
    fn from(value: BeReleaseParseError) -> Self {
        Self::Parse(value)
    }
}

impl From<BeReleaseApplyError> for BeReleaseError {
    fn from(value: BeReleaseApplyError) -> Self {
        Self::Apply(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeControl {
    check_updates: bool,
    update_available: bool,
    update_url: Option<String>,
    update_build: i32,
}

impl Default for BeControl {
    fn default() -> Self {
        Self {
            check_updates: true,
            update_available: false,
            update_url: None,
            update_build: 0,
        }
    }
}

impl BeControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active(&self, runtime: &BeRuntimeFlags) -> bool {
        runtime.version_type == "bleeding-edge" && !runtime.steam
    }

    pub fn check_updates(&self) -> bool {
        self.check_updates
    }

    pub fn is_update_available(&self) -> bool {
        self.update_available
    }

    pub fn update_build(&self) -> i32 {
        self.update_build
    }

    pub fn update_url(&self) -> Option<&str> {
        self.update_url.as_deref()
    }

    pub fn init_plan(&self, runtime: &BeRuntimeFlags) -> BeInitPlan {
        if self.active(runtime) {
            BeInitPlan::ScheduleChecks {
                interval_seconds: BE_UPDATE_INTERVAL_SECONDS,
            }
        } else {
            BeInitPlan::Disabled
        }
    }

    pub fn update_check_plan(&self, runtime: &BeRuntimeFlags) -> BeUpdateCheckPlan {
        if self.active(runtime)
            && (runtime.client_loaded || runtime.headless)
            && self.check_updates
            && !runtime.mobile
        {
            BeUpdateCheckPlan::FetchLatestRelease {
                url: BE_RELEASES_LATEST_URL.to_string(),
            }
        } else {
            BeUpdateCheckPlan::Skip
        }
    }

    pub fn apply_release_text(
        &mut self,
        runtime: &BeRuntimeFlags,
        text: &str,
    ) -> Result<Option<BeUpdateAvailable>, BeReleaseError> {
        let release = BeReleaseInfo::from_json_text(text)?;
        Ok(self.apply_release_info(runtime, &release)?)
    }

    pub fn apply_release_info(
        &mut self,
        runtime: &BeRuntimeFlags,
        release: &BeReleaseInfo,
    ) -> Result<Option<BeUpdateAvailable>, BeReleaseApplyError> {
        let new_build = release.build_number();
        if new_build <= runtime.current_build {
            return Ok(None);
        }

        let asset = release.find_asset(runtime.headless).ok_or_else(|| {
            BeReleaseApplyError::MissingAsset {
                expected_prefix: BeReleaseInfo::asset_prefix(runtime.headless),
                build: new_build,
            }
        })?;

        let available = BeUpdateAvailable {
            build: new_build,
            download_url: asset.browser_download_url.clone(),
        };

        self.update_available = true;
        self.update_build = available.build;
        self.update_url = Some(available.download_url.clone());

        Ok(Some(available))
    }

    pub fn show_update_dialog_plan(
        &mut self,
        runtime: &BeRuntimeFlags,
        auto_update: bool,
    ) -> BeUpdateDialogPlan {
        if !self.update_available {
            return BeUpdateDialogPlan::None;
        }

        let Some(download_url) = self.update_url.clone() else {
            return BeUpdateDialogPlan::None;
        };

        self.check_updates = false;

        if runtime.headless {
            BeUpdateDialogPlan::Headless(BeHeadlessUpdatePlan {
                build: self.update_build,
                download_url,
                auto_update,
            })
        } else {
            BeUpdateDialogPlan::Desktop(BeDesktopUpdatePlan {
                build: self.update_build,
                download_url,
            })
        }
    }
}

fn parse_json_string_value(text: &str, key: &str) -> Result<String, BeReleaseParseError> {
    let value_index = find_json_value_index(text, key)
        .ok_or_else(|| BeReleaseParseError::MissingString(key.to_string()))?;
    if text.as_bytes().get(value_index) != Some(&b'"') {
        return Err(BeReleaseParseError::MissingString(key.to_string()));
    }
    parse_json_string_at(text, value_index, key).map(|(value, _)| value)
}

fn parse_json_object_array<'a>(
    text: &'a str,
    key: &str,
) -> Result<Vec<&'a str>, BeReleaseParseError> {
    let value_index = find_json_value_index(text, key)
        .ok_or_else(|| BeReleaseParseError::MissingArray(key.to_string()))?;
    if text.as_bytes().get(value_index) != Some(&b'[') {
        return Err(BeReleaseParseError::MissingArray(key.to_string()));
    }

    let end = find_matching_delimiter(text, value_index, b'[', b']', key)?;
    let body = &text[value_index + 1..end];
    let bytes = body.as_bytes();
    let mut objects = Vec::new();
    let mut index = 0usize;

    while index < bytes.len() {
        while index < bytes.len() && matches!(bytes[index], b' ' | b'\n' | b'\r' | b'\t' | b',') {
            index += 1;
        }

        if index >= bytes.len() {
            break;
        }

        if bytes[index] != b'{' {
            index += 1;
            continue;
        }

        let object_end = find_matching_delimiter(body, index, b'{', b'}', key)?;
        objects.push(&body[index..=object_end]);
        index = object_end + 1;
    }

    Ok(objects)
}

fn find_json_value_index(text: &str, key: &str) -> Option<usize> {
    let quoted_key = format!("\"{key}\"");
    let key_index = text.find(&quoted_key)?;
    let bytes = text.as_bytes();
    let mut index = key_index + quoted_key.len();

    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }

    if bytes.get(index) != Some(&b':') {
        return None;
    }
    index += 1;

    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }

    Some(index)
}

fn parse_json_string_at(
    text: &str,
    quote_index: usize,
    key: &str,
) -> Result<(String, usize), BeReleaseParseError> {
    let bytes = text.as_bytes();
    let mut index = quote_index + 1;
    let mut value = String::new();

    while index < bytes.len() {
        match bytes[index] {
            b'"' => return Ok((value, index + 1)),
            b'\\' => {
                index += 1;
                match bytes.get(index).copied() {
                    Some(b'"') => value.push('"'),
                    Some(b'\\') => value.push('\\'),
                    Some(b'/') => value.push('/'),
                    Some(b'b') => value.push('\u{0008}'),
                    Some(b'f') => value.push('\u{000c}'),
                    Some(b'n') => value.push('\n'),
                    Some(b'r') => value.push('\r'),
                    Some(b't') => value.push('\t'),
                    Some(other) => value.push(other as char),
                    None => return Err(BeReleaseParseError::UnterminatedString(key.to_string())),
                }
            }
            byte => value.push(byte as char),
        }
        index += 1;
    }

    Err(BeReleaseParseError::UnterminatedString(key.to_string()))
}

fn find_matching_delimiter(
    text: &str,
    open_index: usize,
    open: u8,
    close: u8,
    key: &str,
) -> Result<usize, BeReleaseParseError> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, byte) in bytes.iter().copied().enumerate().skip(open_index) {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }

            match byte {
                b'\\' => escaped = true,
                b'"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match byte {
            b'"' => in_string = true,
            value if value == open => depth += 1,
            value if value == close => {
                depth -= 1;
                if depth == 0 {
                    return Ok(index);
                }
            }
            _ => {}
        }
    }

    Err(match close {
        b']' => BeReleaseParseError::UnterminatedArray(key.to_string()),
        _ => BeReleaseParseError::UnterminatedObject(key.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_requires_bleeding_edge_and_non_steam_like_java() {
        let control = BeControl::new();

        assert!(control.active(&BeRuntimeFlags::new(
            "bleeding-edge",
            false,
            false,
            false,
            false,
            157
        )));
        assert!(!control.active(&BeRuntimeFlags::new(
            "release", false, false, false, false, 157
        )));
        assert!(!control.active(&BeRuntimeFlags::new(
            "bleeding-edge",
            true,
            false,
            false,
            false,
            157
        )));
    }

    #[test]
    fn init_and_tick_plans_follow_java_runtime_guards() {
        let control = BeControl::new();
        let mut runtime = BeRuntimeFlags::new("bleeding-edge", false, false, false, false, 157);

        assert_eq!(
            control.init_plan(&runtime),
            BeInitPlan::ScheduleChecks {
                interval_seconds: BE_UPDATE_INTERVAL_SECONDS,
            }
        );
        assert_eq!(control.update_check_plan(&runtime), BeUpdateCheckPlan::Skip);

        runtime.client_loaded = true;
        assert_eq!(
            control.update_check_plan(&runtime),
            BeUpdateCheckPlan::FetchLatestRelease {
                url: BE_RELEASES_LATEST_URL.into(),
            }
        );

        runtime.client_loaded = false;
        runtime.headless = true;
        assert_eq!(
            control.update_check_plan(&runtime),
            BeUpdateCheckPlan::FetchLatestRelease {
                url: BE_RELEASES_LATEST_URL.into(),
            }
        );

        runtime.mobile = true;
        assert_eq!(control.update_check_plan(&runtime), BeUpdateCheckPlan::Skip);
    }

    #[test]
    fn release_json_parser_keeps_tag_and_assets() {
        let release = BeReleaseInfo::from_json_text(
            r#"{
                "tag_name": "158",
                "assets": [
                    {
                        "name": "Mindustry-BE-Desktop-abc.jar",
                        "browser_download_url": "https://desktop"
                    },
                    {
                        "name": "Mindustry-BE-Server-abc.jar",
                        "browser_download_url": "https://server"
                    }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(release.tag_name, "158");
        assert_eq!(release.build_number(), 158);
        assert_eq!(release.assets.len(), 2);
        assert_eq!(
            release.find_asset(false).unwrap().browser_download_url,
            "https://desktop"
        );
        assert_eq!(
            release.find_asset(true).unwrap().browser_download_url,
            "https://server"
        );
    }

    #[test]
    fn apply_release_info_ignores_non_newer_builds_like_java() {
        let mut control = BeControl::new();
        let runtime = BeRuntimeFlags::new("bleeding-edge", false, true, false, false, 157);
        let release = BeReleaseInfo {
            tag_name: "157".into(),
            assets: vec![BeReleaseAsset {
                name: "Mindustry-BE-Desktop.jar".into(),
                browser_download_url: "https://desktop".into(),
            }],
        };

        assert_eq!(
            control.apply_release_info(&runtime, &release).unwrap(),
            None
        );
        assert!(!control.is_update_available());
        assert_eq!(control.update_url(), None);
    }

    #[test]
    fn apply_release_text_selects_headless_server_asset() {
        let mut control = BeControl::new();
        let runtime = BeRuntimeFlags::new("bleeding-edge", false, false, true, false, 157);

        let available = control
            .apply_release_text(
                &runtime,
                r#"{
                    "tag_name": "158",
                    "assets": [
                        {
                            "name": "Mindustry-BE-Desktop-abc.jar",
                            "browser_download_url": "https://desktop"
                        },
                        {
                            "name": "Mindustry-BE-Server-abc.jar",
                            "browser_download_url": "https://server"
                        }
                    ]
                }"#,
            )
            .unwrap()
            .unwrap();

        assert_eq!(
            available,
            BeUpdateAvailable {
                build: 158,
                download_url: "https://server".into(),
            }
        );
        assert!(control.is_update_available());
        assert_eq!(control.update_build(), 158);
        assert_eq!(control.update_url(), Some("https://server"));
    }

    #[test]
    fn apply_release_text_selects_desktop_asset() {
        let mut control = BeControl::new();
        let runtime = BeRuntimeFlags::new("bleeding-edge", false, true, false, false, 157);

        let available = control
            .apply_release_text(
                &runtime,
                r#"{
                    "tag_name": "158",
                    "assets": [
                        {
                            "name": "Mindustry-BE-Desktop-abc.jar",
                            "browser_download_url": "https://desktop"
                        },
                        {
                            "name": "Mindustry-BE-Server-abc.jar",
                            "browser_download_url": "https://server"
                        }
                    ]
                }"#,
            )
            .unwrap()
            .unwrap();

        assert_eq!(
            available,
            BeUpdateAvailable {
                build: 158,
                download_url: "https://desktop".into(),
            }
        );
        assert_eq!(control.update_url(), Some("https://desktop"));
    }

    #[test]
    fn show_update_dialog_plan_disables_checks_for_desktop_and_headless() {
        let desktop_runtime = BeRuntimeFlags::new("bleeding-edge", false, true, false, false, 157);
        let headless_runtime = BeRuntimeFlags::new("bleeding-edge", false, false, true, false, 157);

        let mut desktop = BeControl::new();
        desktop
            .apply_release_text(
                &desktop_runtime,
                r#"{
                    "tag_name": "158",
                    "assets": [{
                        "name": "Mindustry-BE-Desktop-abc.jar",
                        "browser_download_url": "https://desktop"
                    }]
                }"#,
            )
            .unwrap();
        assert_eq!(
            desktop.show_update_dialog_plan(&desktop_runtime, false),
            BeUpdateDialogPlan::Desktop(BeDesktopUpdatePlan {
                build: 158,
                download_url: "https://desktop".into(),
            })
        );
        assert!(!desktop.check_updates());

        let mut headless = BeControl::new();
        headless
            .apply_release_text(
                &headless_runtime,
                r#"{
                    "tag_name": "158",
                    "assets": [{
                        "name": "Mindustry-BE-Server-abc.jar",
                        "browser_download_url": "https://server"
                    }]
                }"#,
            )
            .unwrap();
        assert_eq!(
            headless.show_update_dialog_plan(&headless_runtime, true),
            BeUpdateDialogPlan::Headless(BeHeadlessUpdatePlan {
                build: 158,
                download_url: "https://server".into(),
                auto_update: true,
            })
        );
        assert!(!headless.check_updates());
    }

    #[test]
    fn show_update_dialog_without_update_leaves_checks_enabled() {
        let runtime = BeRuntimeFlags::new("bleeding-edge", false, true, false, false, 157);
        let mut control = BeControl::new();

        assert_eq!(
            control.show_update_dialog_plan(&runtime, false),
            BeUpdateDialogPlan::None
        );
        assert!(control.check_updates());
    }

    #[test]
    fn newer_build_without_matching_asset_returns_explicit_error() {
        let mut control = BeControl::new();
        let runtime = BeRuntimeFlags::new("bleeding-edge", false, false, true, false, 157);
        let release = BeReleaseInfo {
            tag_name: "158".into(),
            assets: vec![BeReleaseAsset {
                name: "Mindustry-BE-Desktop.jar".into(),
                browser_download_url: "https://desktop".into(),
            }],
        };

        assert_eq!(
            control.apply_release_info(&runtime, &release),
            Err(BeReleaseApplyError::MissingAsset {
                expected_prefix: BE_SERVER_ASSET_PREFIX,
                build: 158,
            })
        );
        assert!(!control.is_update_available());
    }
}
