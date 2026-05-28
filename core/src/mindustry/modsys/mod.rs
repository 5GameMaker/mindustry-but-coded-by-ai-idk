//! Incremental Rust mirror of upstream `mindustry.mod`.
//!
//! `mod` is a Rust keyword, so this crate exposes the package as `modsys`.
//! The first migrated pieces are the Java `Mod` base class hooks and the
//! `NoPatch` marker annotation.

use crate::mindustry::graphics::{
    MultiPackerPlan, PageType, RegionRequest, TextureAtlasRegionSource,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModConfigPaths {
    pub mods_dir: String,
    pub plugin_name: String,
}

impl ModConfigPaths {
    pub fn new(mods_dir: impl Into<String>, plugin_name: impl Into<String>) -> Self {
        Self {
            mods_dir: trim_slash(mods_dir.into()),
            plugin_name: plugin_name.into(),
        }
    }

    /// Java: `mods/[plugin-name]`.
    pub fn config_folder(&self) -> String {
        format!("{}/{}", self.mods_dir, self.plugin_name)
    }

    /// Java: `mods/[plugin-name]/config.json`.
    pub fn config(&self) -> String {
        format!("{}/config.json", self.config_folder())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: String,
    pub description: String,
}

impl CommandSpec {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandRegistry {
    commands: Vec<CommandSpec>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, name: impl Into<String>, description: impl Into<String>) {
        self.commands.push(CommandSpec::new(name, description));
    }

    pub fn commands(&self) -> &[CommandSpec] {
        &self.commands
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpritePackRequest {
    pub source_path: String,
    pub atlas_name: String,
    pub page_hint: String,
    pub r#override: bool,
}

impl SpritePackRequest {
    pub fn new(source_path: impl Into<String>, atlas_name: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            atlas_name: atlas_name.into(),
            page_hint: String::new(),
            r#override: false,
        }
    }

    pub fn with_page_hint(mut self, page_hint: impl Into<String>) -> Self {
        self.page_hint = page_hint.into();
        self
    }

    pub fn with_override(mut self, r#override: bool) -> Self {
        self.r#override = r#override;
        self
    }

    /// 解析该请求应该进入哪个 atlas page。
    ///
    /// 显式 `page_hint` 优先；`sprites` / `sprites-override` 这类上层目录
    /// 会回退到和 upstream Java packer 相同的路径推断规则。
    pub fn page_type(&self) -> PageType {
        resolve_sprite_page_type(&self.page_hint, &self.source_path)
    }

    /// 把请求转换成纯数据 region 请求。
    ///
    /// 当没有真实图片尺寸时，`width` / `height` 可以作为占位 metadata，
    /// 默认值为 `1x1`。
    pub fn to_region_request(&self) -> RegionRequest<TextureAtlasRegionSource<bool>> {
        self.to_region_request_with_size(1, 1)
    }

    /// 把请求转换成纯数据 region 请求，并显式提供占位尺寸。
    pub fn to_region_request_with_size(
        &self,
        width: u32,
        height: u32,
    ) -> RegionRequest<TextureAtlasRegionSource<bool>> {
        RegionRequest::new(
            self.atlas_name.clone(),
            width.max(1),
            height.max(1),
            TextureAtlasRegionSource::new(self.source_path.clone(), self.r#override),
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpritePacker {
    textures: Vec<String>,
    requests: Vec<SpritePackRequest>,
}

impl SpritePacker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_texture(&mut self, path: impl Into<String>) {
        let path = path.into();
        self.textures.push(path.clone());
        self.requests
            .push(SpritePackRequest::new(path.clone(), path));
    }

    pub fn textures(&self) -> &[String] {
        &self.textures
    }

    pub fn add_request(&mut self, request: SpritePackRequest) {
        self.requests.push(request);
    }

    pub fn requests(&self) -> &[SpritePackRequest] {
        &self.requests
    }

    /// 导出纯数据 atlas 计划。
    ///
    /// 没有真实图片尺寸时，默认使用 `1x1` 作为占位 metadata。
    pub fn to_multi_packer_plan(&self) -> MultiPackerPlan<TextureAtlasRegionSource<bool>> {
        self.to_multi_packer_plan_with_size(1, 1)
    }

    /// 导出纯数据 atlas 计划，并显式指定占位尺寸。
    pub fn to_multi_packer_plan_with_size(
        &self,
        width: u32,
        height: u32,
    ) -> MultiPackerPlan<TextureAtlasRegionSource<bool>> {
        let mut plan = MultiPackerPlan::new();

        for request in &self.requests {
            let page_type = request.page_type();
            let region = request.to_region_request_with_size(width, height);
            let _ = plan.insert_or_replace_request(page_type, region);
        }

        plan
    }
}

/// Rust equivalent of the Java abstract `Mod` base class.
///
/// Every hook defaults to a no-op, matching upstream.
pub trait Mod {
    fn get_config_folder(&self, paths: &ModConfigPaths) -> String {
        paths.config_folder()
    }

    fn get_config(&self, paths: &ModConfigPaths) -> String {
        paths.config()
    }

    /// Called after all plugins have been created and commands registered.
    fn init(&mut self) {}

    /// Called on clientside mods. Load content here.
    fn load_content(&mut self) {}

    /// Called during sprite packing to allow adding custom textures.
    fn pack_sprites(&mut self, _packer: &mut SpritePacker) {}

    /// Register commands to be used on the server side.
    fn register_server_commands(&mut self, _handler: &mut CommandRegistry) {}

    /// Register commands to be used on the client side.
    fn register_client_commands(&mut self, _handler: &mut CommandRegistry) {}
}

/// Rust equivalent of Java `Plugin extends Mod`.
///
/// Plugins are a special type of mod that is always hidden.
pub trait Plugin: Mod {
    fn hidden(&self) -> bool {
        true
    }
}

/// Marker equivalent to Java's runtime `@NoPatch` annotation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NoPatch;

pub trait NoPatchMarker {}

/// Lightweight class-loader handle used to mirror upstream
/// `mindustry.mod.ClassLoaderCloser`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassLoaderKind {
    /// Java `URLClassLoader`, the only loader kind closed by upstream.
    Url { closed: bool },
    /// Any other platform loader; Java leaves it untouched.
    Other,
}

impl ClassLoaderKind {
    pub fn url() -> Self {
        Self::Url { closed: false }
    }

    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Url { closed: true })
    }
}

/// Mirrors Java `ClassLoaderCloser.close(loader)`.
///
/// Upstream only calls `close()` when the provided loader is an
/// `URLClassLoader`; this preserves the Android workaround shape where other
/// loaders may not expose a close method.
pub fn close_class_loader(loader: &mut ClassLoaderKind) {
    if let ClassLoaderKind::Url { closed } = loader {
        *closed = true;
    }
}

fn trim_slash(mut path: String) -> String {
    while path.ends_with('/') || path.ends_with('\\') {
        path.pop();
    }
    if path.is_empty() {
        ".".into()
    } else {
        path.replace('\\', "/")
    }
}

fn resolve_sprite_page_type(page_hint: &str, source_path: &str) -> PageType {
    if let Some(page_type) = page_type_from_hint(page_hint) {
        return page_type;
    }

    page_type_from_source_path(source_path)
}

fn page_type_from_hint(page_hint: &str) -> Option<PageType> {
    let hint = normalize_sprite_hint(page_hint);

    match hint.as_str() {
        "" => None,
        "main" => Some(PageType::Main),
        "environment" => Some(PageType::Environment),
        "ui" => Some(PageType::Ui),
        "rubble" => Some(PageType::Rubble),
        "sprites" | "sprites-override" => None,
        _ if hint.contains("environment") => Some(PageType::Environment),
        _ if hint.contains("rubble") => Some(PageType::Rubble),
        _ if hint.contains("ui") => Some(PageType::Ui),
        _ if hint.contains("main") => Some(PageType::Main),
        _ => None,
    }
}

fn page_type_from_source_path(source_path: &str) -> PageType {
    let path = normalize_sprite_hint(source_path);

    if path.contains("sprites/blocks/environment")
        || path.contains("sprites-override/blocks/environment")
    {
        PageType::Environment
    } else if path.contains("sprites/rubble") || path.contains("sprites-override/rubble") {
        PageType::Rubble
    } else if path.contains("sprites/ui") || path.contains("sprites-override/ui") {
        PageType::Ui
    } else {
        PageType::Main
    }
}

fn normalize_sprite_hint(value: &str) -> String {
    value.trim().replace('\\', "/").to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct EmptyMod;

    impl Mod for EmptyMod {}

    #[derive(Default)]
    struct RecordingMod {
        initialized: bool,
        content_loaded: bool,
    }

    impl Mod for RecordingMod {
        fn init(&mut self) {
            self.initialized = true;
        }

        fn load_content(&mut self) {
            self.content_loaded = true;
        }

        fn pack_sprites(&mut self, packer: &mut SpritePacker) {
            packer.add_texture("sprites/custom.png");
        }

        fn register_server_commands(&mut self, handler: &mut CommandRegistry) {
            handler.register("status", "prints status");
        }

        fn register_client_commands(&mut self, handler: &mut CommandRegistry) {
            handler.register("ping", "client ping");
        }
    }

    struct LockedField;
    impl NoPatchMarker for LockedField {}

    #[derive(Default)]
    struct HiddenPlugin;
    impl Mod for HiddenPlugin {}
    impl Plugin for HiddenPlugin {}

    fn assert_no_patch<T: NoPatchMarker>() {}

    #[test]
    fn mod_config_paths_follow_java_mods_plugin_layout() {
        let paths = ModConfigPaths::new("mods\\", "example");

        assert_eq!(paths.config_folder(), "mods/example");
        assert_eq!(paths.config(), "mods/example/config.json");

        let empty_root = ModConfigPaths::new("", "plugin");
        assert_eq!(empty_root.config_folder(), "./plugin");
    }

    #[test]
    fn default_mod_hooks_are_noops_like_java_base_class() {
        let paths = ModConfigPaths::new("mods", "empty");
        let mut module = EmptyMod;
        let mut packer = SpritePacker::new();
        let mut server = CommandRegistry::new();
        let mut client = CommandRegistry::new();

        assert_eq!(module.get_config_folder(&paths), "mods/empty");
        assert_eq!(module.get_config(&paths), "mods/empty/config.json");

        module.init();
        module.load_content();
        module.pack_sprites(&mut packer);
        module.register_server_commands(&mut server);
        module.register_client_commands(&mut client);

        assert!(packer.textures().is_empty());
        assert!(server.is_empty());
        assert!(client.is_empty());
    }

    #[test]
    fn mod_hooks_can_register_content_sprites_and_commands() {
        let mut module = RecordingMod::default();
        let mut packer = SpritePacker::new();
        let mut server = CommandRegistry::new();
        let mut client = CommandRegistry::new();

        module.init();
        module.load_content();
        module.pack_sprites(&mut packer);
        module.register_server_commands(&mut server);
        module.register_client_commands(&mut client);

        assert!(module.initialized);
        assert!(module.content_loaded);
        assert_eq!(packer.textures(), &["sprites/custom.png".to_string()]);
        assert_eq!(
            packer.requests(),
            &[SpritePackRequest::new(
                "sprites/custom.png",
                "sprites/custom.png"
            )]
        );
        assert_eq!(
            server.commands(),
            &[CommandSpec::new("status", "prints status")]
        );
        assert_eq!(
            client.commands(),
            &[CommandSpec::new("ping", "client ping")]
        );
    }

    #[test]
    fn sprite_packer_legacy_api_keeps_textures_and_builds_default_requests() {
        let mut packer = SpritePacker::new();

        packer.add_texture("sprites/custom.png");

        assert_eq!(packer.textures(), &["sprites/custom.png".to_string()]);
        assert_eq!(
            packer.requests(),
            &[SpritePackRequest::new(
                "sprites/custom.png",
                "sprites/custom.png"
            )]
        );

        let plan = packer.to_multi_packer_plan();
        let region = plan
            .page(PageType::Main)
            .get("sprites/custom.png")
            .expect("default sprite texture should export to main page");
        assert_eq!(region.width, 1);
        assert_eq!(region.height, 1);
        assert_eq!(region.payload.source_path, "sprites/custom.png");
        assert!(!region.payload.payload);
    }

    #[test]
    fn sprite_packer_requests_can_describe_sprites_and_overrides() {
        let mut packer = SpritePacker::new();

        packer.add_request(
            SpritePackRequest::new("sprites/block.png", "block")
                .with_page_hint("sprites")
                .with_override(false),
        );
        packer.add_request(
            SpritePackRequest::new("sprites-override/ui/icon.png", "icon")
                .with_page_hint("sprites-override")
                .with_override(true),
        );

        assert_eq!(
            packer.requests(),
            &[
                SpritePackRequest {
                    source_path: "sprites/block.png".into(),
                    atlas_name: "block".into(),
                    page_hint: "sprites".into(),
                    r#override: false,
                },
                SpritePackRequest {
                    source_path: "sprites-override/ui/icon.png".into(),
                    atlas_name: "icon".into(),
                    page_hint: "sprites-override".into(),
                    r#override: true,
                },
            ]
        );
    }

    #[test]
    fn sprite_pack_request_page_type_uses_stable_hints_and_source_paths() {
        let main = SpritePackRequest::new("sprites/custom.png", "custom");
        let environment =
            SpritePackRequest::new("assets/ignored.png", "env").with_page_hint("environment");
        let ui = SpritePackRequest::new("sprites/ui/icon.png", "icon").with_page_hint("sprites");
        let rubble = SpritePackRequest::new("sprites-override/rubble/crack.png", "crack")
            .with_page_hint("sprites-override");

        assert_eq!(main.page_type(), PageType::Main);
        assert_eq!(environment.page_type(), PageType::Environment);
        assert_eq!(ui.page_type(), PageType::Ui);
        assert_eq!(rubble.page_type(), PageType::Rubble);
    }

    #[test]
    fn sprite_packer_exports_multi_packer_plan_with_placeholder_metadata() {
        let mut packer = SpritePacker::new();

        packer.add_request(
            SpritePackRequest::new("sprites/block.png", "block")
                .with_page_hint("sprites")
                .with_override(false),
        );
        packer.add_request(
            SpritePackRequest::new("sprites/blocks/environment/env.png", "env")
                .with_page_hint("environment")
                .with_override(true),
        );
        packer.add_request(
            SpritePackRequest::new("sprites/ui/icon.png", "icon")
                .with_page_hint("sprites-override"),
        );
        packer.add_request(
            SpritePackRequest::new("sprites/rubble/crack.png", "crack")
                .with_page_hint("rubble")
                .with_override(true),
        );

        let plan = packer.to_multi_packer_plan_with_size(8, 16);

        let main = plan.page(PageType::Main).get("block").unwrap();
        assert_eq!(main.width, 8);
        assert_eq!(main.height, 16);
        assert_eq!(main.payload.source_path, "sprites/block.png");
        assert!(!main.payload.payload);

        let environment = plan.page(PageType::Environment).get("env").unwrap();
        assert_eq!(environment.width, 8);
        assert_eq!(environment.height, 16);
        assert_eq!(
            environment.payload.source_path,
            "sprites/blocks/environment/env.png"
        );
        assert!(environment.payload.payload);

        let ui = plan.page(PageType::Ui).get("icon").unwrap();
        assert_eq!(ui.width, 8);
        assert_eq!(ui.height, 16);
        assert_eq!(ui.payload.source_path, "sprites/ui/icon.png");
        assert!(!ui.payload.payload);

        let rubble = plan.page(PageType::Rubble).get("crack").unwrap();
        assert_eq!(rubble.width, 8);
        assert_eq!(rubble.height, 16);
        assert_eq!(rubble.payload.source_path, "sprites/rubble/crack.png");
        assert!(rubble.payload.payload);
    }

    #[test]
    fn no_patch_marker_can_be_attached_to_rust_types() {
        assert_eq!(NoPatch, NoPatch);
        assert_no_patch::<LockedField>();
    }

    #[test]
    fn plugin_marker_extends_mod_and_is_hidden_by_default() {
        let plugin = HiddenPlugin;
        assert!(plugin.hidden());
    }

    #[test]
    fn class_loader_closer_only_closes_url_loaders_like_java() {
        let mut url = ClassLoaderKind::url();
        let mut other = ClassLoaderKind::Other;

        close_class_loader(&mut url);
        close_class_loader(&mut other);

        assert!(url.is_closed());
        assert_eq!(other, ClassLoaderKind::Other);
    }
}
