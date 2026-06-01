use base64::Engine;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformInfo {
    pub name: String,
    pub headless: bool,
}

impl PlatformInfo {
    pub fn headless() -> Self {
        Self {
            name: "headless".into(),
            headless: true,
        }
    }

    pub fn desktop() -> Self {
        Self {
            name: "desktop".into(),
            headless: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetProviderKind {
    ArcNet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptRuntimeKind {
    Rhino,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlatformSettings {
    uuid: String,
}

impl PlatformSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_uuid(uuid: impl Into<String>) -> Self {
        Self { uuid: uuid.into() }
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn set_uuid(&mut self, uuid: impl Into<String>) {
        self.uuid = uuid.into();
    }
}

/// Rust counterpart of upstream `Platform` default methods.
///
/// Side-effect-heavy operations intentionally no-op by default, matching Java.
pub trait Platform {
    fn update_lobby(&mut self) {}

    fn invite_friends(&mut self) {}

    fn publish(&mut self, _publishable_name: &str) {}

    fn view_listing(&mut self, _publishable_name: &str) {}

    fn view_listing_id(&mut self, _map_id: &str) {}

    fn get_workshop_content(&self, _publishable_type: &str) -> Vec<String> {
        Vec::new()
    }

    fn open_workshop(&mut self) {}

    fn open_uri(&mut self, _uri: &str) -> bool {
        false
    }

    fn set_clipboard_text(&mut self, _text: &str) {}

    fn get_clipboard_text(&mut self) -> Option<String> {
        None
    }

    fn get_net(&self) -> NetProviderKind {
        NetProviderKind::ArcNet
    }

    fn create_scripts(&self) -> ScriptRuntimeKind {
        ScriptRuntimeKind::Rhino
    }

    fn update_rpc(&mut self) {}

    /// Returns an existing persisted UUID or creates an 8-byte base64 value.
    fn get_uuid(&mut self, settings: &mut PlatformSettings) -> String {
        if !settings.uuid.is_empty() {
            return settings.uuid.clone();
        }
        let uuid = encode_uuid_bytes(default_uuid_bytes());
        settings.uuid = uuid.clone();
        uuid
    }

    fn share_file(&mut self, _file: &str) {}

    fn show_file_chooser(
        &mut self,
        open: bool,
        title: &str,
        extension: &str,
    ) -> FileChooserRequest {
        FileChooserRequest::new(open, title, extension)
    }

    fn show_multi_file_chooser(&mut self, extensions: &[&str]) -> MultiFileChooserRequest {
        MultiFileChooserRequest::new(extensions)
    }

    fn hide(&mut self) {}

    fn begin_force_landscape(&mut self) {}

    fn end_force_landscape(&mut self) {}
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DefaultPlatform;

impl Platform for DefaultPlatform {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChooserRequest {
    pub open: bool,
    pub title: String,
    pub extension: String,
}

impl FileChooserRequest {
    pub fn new(open: bool, title: impl Into<String>, extension: impl Into<String>) -> Self {
        Self {
            open,
            title: title.into(),
            extension: normalize_extension(extension),
        }
    }

    pub fn accepts(&self, file: &str) -> bool {
        extension_of(file)
            .map(|extension| extension.eq_ignore_ascii_case(&self.extension))
            .unwrap_or(false)
    }

    /// Java save mode writes to `parent/nameWithoutExtension.extension`.
    pub fn selected_result(&self, selected: &str) -> String {
        if self.open {
            selected.replace('\\', "/")
        } else {
            replace_extension(selected, &self.extension)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiFileChooserRequest {
    pub extensions: Vec<String>,
}

impl MultiFileChooserRequest {
    pub fn new(extensions: &[&str]) -> Self {
        Self {
            extensions: extensions
                .iter()
                .map(|extension| normalize_extension(*extension))
                .collect(),
        }
    }

    pub fn accepts(&self, file: &str) -> bool {
        extension_of(file)
            .map(|extension| {
                self.extensions
                    .iter()
                    .any(|allowed| extension.eq_ignore_ascii_case(allowed))
            })
            .unwrap_or(false)
    }
}

pub trait FileWriter {
    fn write(&mut self, file: &str) -> Result<(), String>;
}

pub fn encode_uuid_bytes(bytes: [u8; 8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn default_uuid_bytes() -> [u8; 8] {
    // Deterministic fallback for headless/test contexts. Real platform backends
    // can override `get_uuid` and use OS randomness.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0x6d69_6e64_7573_7479);
    splitmix64(nanos).to_be_bytes()
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn normalize_extension(extension: impl Into<String>) -> String {
    extension
        .into()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

fn extension_of(file: &str) -> Option<String> {
    file.rsplit(['/', '\\']).next().and_then(|name| {
        name.rsplit_once('.')
            .map(|(_, extension)| extension.to_ascii_lowercase())
    })
}

fn replace_extension(file: &str, extension: &str) -> String {
    let normalized = file.replace('\\', "/");
    let (parent, name) = normalized
        .rsplit_once('/')
        .map(|(parent, name)| (Some(parent), name))
        .unwrap_or((None, normalized.as_str()));
    let stem = name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name);
    match parent {
        Some(parent) => format!("{parent}/{stem}.{extension}"),
        None => format!("{stem}.{extension}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_info_factories_keep_headless_and_desktop_defaults() {
        assert_eq!(
            PlatformInfo::headless(),
            PlatformInfo {
                name: "headless".into(),
                headless: true
            }
        );
        assert_eq!(
            PlatformInfo::desktop(),
            PlatformInfo {
                name: "desktop".into(),
                headless: false
            }
        );
    }

    #[test]
    fn default_platform_hooks_match_java_noop_and_provider_defaults() {
        let mut platform = DefaultPlatform;

        platform.update_lobby();
        platform.invite_friends();
        platform.publish("map");
        platform.view_listing("map");
        platform.view_listing_id("123");
        platform.open_workshop();
        assert!(!platform.open_uri("https://example.invalid"));
        platform.set_clipboard_text("https://example.invalid");
        platform.update_rpc();
        platform.share_file("map.msav");
        platform.hide();
        platform.begin_force_landscape();
        platform.end_force_landscape();

        assert!(platform.get_workshop_content("map").is_empty());
        assert_eq!(platform.get_net(), NetProviderKind::ArcNet);
        assert_eq!(platform.create_scripts(), ScriptRuntimeKind::Rhino);
    }

    #[test]
    fn get_uuid_reuses_existing_or_persists_generated_base64_eight_bytes() {
        let mut platform = DefaultPlatform;
        let mut existing = PlatformSettings::with_uuid("AQIDBAUGBwg=");
        assert_eq!(platform.get_uuid(&mut existing), "AQIDBAUGBwg=");

        let mut empty = PlatformSettings::new();
        let generated = platform.get_uuid(&mut empty);
        assert_eq!(generated, empty.uuid());
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(generated.as_bytes())
                .unwrap()
                .len(),
            8
        );
        assert_eq!(encode_uuid_bytes([1, 2, 3, 4, 5, 6, 7, 8]), "AQIDBAUGBwg=");
    }

    #[test]
    fn file_chooser_requests_filter_and_rewrite_save_extension_like_java_dialog() {
        let open = FileChooserRequest::new(true, "@open", "msav");
        assert!(open.accepts("maps/test.MSAV"));
        assert!(!open.accepts("maps/test.zip"));
        assert_eq!(open.selected_result("maps\\test.msav"), "maps/test.msav");

        let save = FileChooserRequest::new(false, "@save", ".msav");
        assert_eq!(save.selected_result("maps/save.old"), "maps/save.msav");
        assert_eq!(save.selected_result("save"), "save.msav");
    }

    #[test]
    fn multi_file_chooser_accepts_any_configured_extension_case_insensitively() {
        let request = MultiFileChooserRequest::new(&["zip", ".jar"]);
        assert!(request.accepts("mods/foo.ZIP"));
        assert!(request.accepts("mods/foo.jar"));
        assert!(!request.accepts("mods/foo.msav"));
    }
}
