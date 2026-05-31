//! Incremental Rust mirror of upstream `mindustry.mod`.
//!
//! `mod` is a Rust keyword, so this crate exposes the package as `modsys`.
//! The first migrated pieces are the Java `Mod` base class hooks and the
//! `NoPatch` marker annotation.

use crate::mindustry::core::{AssetFile, FileTree};
use crate::mindustry::graphics::{
    png_dimensions_from_path, MultiPackerPlan, PageType, RegionRequest, TextureAtlasRegionSource,
    TextureScale,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModMetadata {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub repo: Option<String>,
    pub source_path: Option<String>,
}

impl ModMetadata {
    pub fn from_directory(
        fallback_name: impl AsRef<str>,
        root: impl AsRef<Path>,
    ) -> io::Result<Self> {
        let fallback_name = fallback_name.as_ref();
        let root = resolve_mod_root(root)?;
        for file_name in ["mod.hjson", "mod.json", "plugin.hjson", "plugin.json"] {
            let path = root.join(file_name);
            if !path.is_file() {
                continue;
            }

            let bytes = fs::read(&path)?;
            let source = String::from_utf8_lossy(&bytes);
            return Ok(Self::from_source_text(
                fallback_name,
                Some(file_name),
                source.as_ref(),
            ));
        }

        Ok(Self {
            name: Some(fallback_name.to_string()),
            ..Self::default()
        })
    }

    pub fn from_source_text(
        fallback_name: impl AsRef<str>,
        source_path: Option<&str>,
        source: &str,
    ) -> Self {
        let fallback_name = fallback_name.as_ref();
        let name = extract_mod_metadata_value(source, "name")
            .or_else(|| (!fallback_name.is_empty()).then(|| fallback_name.to_string()));
        Self {
            name,
            display_name: extract_mod_metadata_value(source, "displayName")
                .or_else(|| extract_mod_metadata_value(source, "display-name"))
                .or_else(|| extract_mod_metadata_value(source, "display")),
            author: extract_mod_metadata_value(source, "author"),
            version: extract_mod_metadata_value(source, "version"),
            description: extract_mod_metadata_value(source, "description"),
            repo: extract_mod_metadata_value(source, "repo"),
            source_path: source_path.map(str::to_string),
        }
    }

    pub fn display_name_or_name(&self) -> Option<&str> {
        self.display_name
            .as_deref()
            .or(self.name.as_deref())
            .filter(|value| !value.trim().is_empty())
    }

    pub fn author_or_unknown(&self) -> &str {
        self.author.as_deref().unwrap_or("@unknown")
    }

    pub fn version_or_unknown(&self) -> &str {
        self.version.as_deref().unwrap_or("@unknown")
    }
}

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
    pub texture_scale: TextureScale,
}

impl SpritePackRequest {
    pub fn new(source_path: impl Into<String>, atlas_name: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            atlas_name: atlas_name.into(),
            page_hint: String::new(),
            r#override: false,
            texture_scale: TextureScale::default(),
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

    pub fn with_texture_scale(mut self, texture_scale: f32) -> Self {
        self.texture_scale = TextureScale::new(texture_scale);
        self
    }

    pub fn texture_scale(&self) -> f32 {
        self.texture_scale.value()
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
        let (width, height) = png_dimensions_from_path(&self.source_path).unwrap_or((1, 1));
        self.to_region_request_with_size(width, height)
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
            TextureAtlasRegionSource::new(self.source_path.clone(), self.r#override)
                .with_texture_scale(self.texture_scale()),
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

    pub fn add_mod_sprite_source(&mut self, source: ModSpritePackSource) -> bool {
        if let Some(request) = source.to_request() {
            self.add_request(request);
            true
        } else {
            false
        }
    }

    pub fn extend_mod_sprite_sources<I>(&mut self, sources: I) -> usize
    where
        I: IntoIterator<Item = ModSpritePackSource>,
    {
        let mut imported = 0;
        for source in sources {
            if self.add_mod_sprite_source(source) {
                imported += 1;
            }
        }
        imported
    }

    pub fn requests(&self) -> &[SpritePackRequest] {
        &self.requests
    }

    /// 导出纯数据 atlas 计划。
    ///
    /// 真实 PNG 文件会读取 IHDR 宽高；虚拟路径或读取失败时回退到 `1x1`
    /// 占位 metadata。
    pub fn to_multi_packer_plan(&self) -> MultiPackerPlan<TextureAtlasRegionSource<bool>> {
        let mut plan = MultiPackerPlan::new();

        for request in &self.requests {
            let page_type = request.page_type();
            let region = request.to_region_request();
            let _ = plan.insert_or_replace_request(page_type, region);
        }

        plan
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

/// Pure-data mirror of one file discovered by Java `Mods.packSprites(...)`.
///
/// Upstream scans both `sprites` and `sprites-override`; the former prefixes
/// atlas names with the mod name, while the latter keeps the original name and
/// replaces an existing atlas region. This struct is intentionally filesystem
/// agnostic so the real directory scanner can feed it later without forcing PNG
/// decoding or GPU texture upload into the planning layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModSpritePackSource {
    pub mod_name: String,
    pub source_path: String,
    pub prefix_with_mod_name: bool,
}

impl ModSpritePackSource {
    pub fn new(
        mod_name: impl Into<String>,
        source_path: impl Into<String>,
        prefix_with_mod_name: bool,
    ) -> Self {
        Self {
            mod_name: mod_name.into(),
            source_path: source_path.into(),
            prefix_with_mod_name,
        }
    }

    pub fn sprite(mod_name: impl Into<String>, source_path: impl Into<String>) -> Self {
        Self::new(mod_name, source_path, true)
    }

    pub fn override_sprite(mod_name: impl Into<String>, source_path: impl Into<String>) -> Self {
        Self::new(mod_name, source_path, false)
    }

    pub fn atlas_name(&self) -> Option<String> {
        mod_sprite_atlas_name(&self.mod_name, &self.source_path, self.prefix_with_mod_name)
    }

    pub fn page_hint(&self) -> &'static str {
        if self.prefix_with_mod_name {
            "sprites"
        } else {
            "sprites-override"
        }
    }

    pub fn to_request(&self) -> Option<SpritePackRequest> {
        let atlas_name = self.atlas_name()?;
        Some(
            SpritePackRequest::new(self.source_path.clone(), atlas_name)
                .with_page_hint(self.page_hint())
                .with_override(!self.prefix_with_mod_name),
        )
    }

    pub fn from_scanned_path(
        mod_name: impl Into<String>,
        source_path: impl Into<String>,
    ) -> Option<Self> {
        let mod_name = mod_name.into();
        let source_path = normalize_mod_resource_path(source_path.into());
        if !source_path.to_ascii_lowercase().ends_with(".png") {
            return None;
        }

        let mut saw_sprites = false;
        for segment in source_path.split('/') {
            match segment {
                "sprites-override" => {
                    return Some(Self::override_sprite(mod_name, source_path));
                }
                "sprites" => saw_sprites = true,
                _ => {}
            }
        }

        saw_sprites.then(|| Self::sprite(mod_name, source_path))
    }
}

/// Pure-data plan for mod icons.
///
/// Headless runtimes never try to load an icon. Non-headless runtimes keep the
/// v158.1 candidate order: `icon.png` first, then `preview.png`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModIconLoadPlan {
    pub headless: bool,
    pub candidates: Vec<String>,
}

impl ModIconLoadPlan {
    pub fn new(headless: bool) -> Self {
        Self {
            headless,
            candidates: mod_icon_candidates(headless),
        }
    }
}

impl Default for ModIconLoadPlan {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Pure-data mod resource plan that keeps icon lookup and sprite packing data
/// separate from real file system or GPU work.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModResourcePlan {
    pub icon: ModIconLoadPlan,
    pub sprite_sources: Vec<ModSpritePackSource>,
}

impl ModResourcePlan {
    pub fn new(headless: bool) -> Self {
        Self {
            icon: ModIconLoadPlan::new(headless),
            sprite_sources: Vec::new(),
        }
    }

    pub fn with_sprite_sources(
        mut self,
        sources: impl IntoIterator<Item = ModSpritePackSource>,
    ) -> Self {
        self.sprite_sources.extend(sources);
        self
    }

    pub fn from_file_paths(
        mod_name: impl Into<String>,
        headless: bool,
        paths: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let mod_name = mod_name.into();
        let mut regular_sources = Vec::new();
        let mut override_sources = Vec::new();

        for source in paths
            .into_iter()
            .filter_map(|path| ModSpritePackSource::from_scanned_path(mod_name.clone(), path))
        {
            if source.prefix_with_mod_name {
                regular_sources.push(source);
            } else {
                override_sources.push(source);
            }
        }

        regular_sources.sort_by(|left, right| left.source_path.cmp(&right.source_path));
        override_sources.sort_by(|left, right| left.source_path.cmp(&right.source_path));
        regular_sources.extend(override_sources);

        Self {
            icon: ModIconLoadPlan::new(headless),
            sprite_sources: regular_sources,
        }
    }

    /// 真实目录入口：先做 root unwrap，再扫描 `sprites/**/*.png` 与
    /// `sprites-override/**/*.png`，最后复用 `from_file_paths(...)` 生成纯数据计划。
    pub fn from_directory(
        mod_name: impl Into<String>,
        headless: bool,
        root: impl AsRef<Path>,
    ) -> io::Result<Self> {
        let sprite_paths = scan_mod_sprite_paths(root)?;
        Ok(Self::from_file_paths(mod_name, headless, sprite_paths))
    }

    pub fn sprite_requests(&self) -> Vec<SpritePackRequest> {
        self.sprite_sources
            .iter()
            .filter_map(ModSpritePackSource::to_request)
            .collect()
    }
}

/// 单个被发现的 mod 目录计划。
///
/// `mod_name` 保留容器顶层目录名，`root` 则保存经过单子目录 unwrap 后的
/// 实际资源根。这样既能在容器级 discovery 中保持稳定命名，又不会丢失
/// 真实资源扫描锚点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModResourceDirectoryPlan {
    pub mod_name: String,
    pub root: PathBuf,
    pub meta: ModMetadata,
    pub file_tree: FileTree,
    pub resource_plan: ModResourcePlan,
}

impl ModResourceDirectoryPlan {
    pub fn from_directory(
        mod_name: impl Into<String>,
        headless: bool,
        root: impl AsRef<Path>,
    ) -> io::Result<Self> {
        let mod_name = mod_name.into();
        let root = resolve_mod_root(root)?;
        let meta = ModMetadata::from_directory(&mod_name, &root)?;
        let file_tree = mod_file_tree_from_directory(&root)?;
        let resource_plan = ModResourcePlan::from_directory(mod_name.clone(), headless, &root)?;

        Ok(Self {
            mod_name,
            root,
            meta,
            file_tree,
            resource_plan,
        })
    }
}

/// `data/mods` 容器级纯数据 discovery 结果。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModResourceContainerPlan {
    pub mods: Vec<ModResourceDirectoryPlan>,
}

impl ModResourceContainerPlan {
    pub fn discover_from_mods_directory(
        mods_dir: impl AsRef<Path>,
        headless: bool,
    ) -> io::Result<Self> {
        let mods_dir = mods_dir.as_ref();
        let mut mods = Vec::new();

        if !mods_dir.is_dir() {
            return Ok(Self { mods });
        }

        let mut entries = fs::read_dir(mods_dir)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by(|left, right| {
            left.file_name()
                .to_string_lossy()
                .cmp(&right.file_name().to_string_lossy())
        });

        for entry in entries {
            let path = entry.path();
            let folder = entry.file_name();
            if !path.is_dir() || is_skipped_mod_container_entry(&folder) {
                continue;
            }

            let mod_name = folder.to_string_lossy().into_owned();
            mods.push(ModResourceDirectoryPlan::from_directory(
                mod_name, headless, path,
            )?);
        }

        Ok(Self { mods })
    }

    pub fn is_empty(&self) -> bool {
        self.mods.is_empty()
    }
}

/// 解析真实 mod 目录的 root：如果目录下只有一个子目录，则自动展开到该子目录。
///
/// 这和 upstream Java `resolveRoot(...)` 的行为一致，便于处理“外层包了一层目录”
/// 的导入资源。
pub fn resolve_mod_root(root: impl AsRef<Path>) -> io::Result<PathBuf> {
    let root = root.as_ref();
    if !root.is_dir() {
        return Ok(root.to_path_buf());
    }

    let entries: Vec<_> = fs::read_dir(root)?.collect::<Result<_, _>>()?;
    let mut visible = entries
        .into_iter()
        .filter(|entry| entry.file_name() != ".DS_Store")
        .collect::<Vec<_>>();

    if visible.len() == 1 {
        let candidate = visible.remove(0).path();
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }

    Ok(root.to_path_buf())
}

/// 扫描真实 mod 目录中的普通文件，行为参考 Java `buildFiles(...)`：
/// 跳过顶层 `bundles/`、`sprites/`、`sprites-override/` 和 `.git/`。
pub fn scan_mod_file_paths(root: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let root = resolve_mod_root(root)?;
    let mut paths = Vec::new();

    if !root.is_dir() {
        return Ok(paths);
    }

    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let folder = entry.file_name();
        if is_special_top_level_folder(&folder) {
            continue;
        }

        walk_relative_files(&root, &path, &mut paths, accept_all_files)?;
    }

    paths.sort();
    Ok(paths)
}

/// 单独扫描 sprite 资源，严格收集 `sprites/**/*.png` 和
/// `sprites-override/**/*.png`。
pub fn scan_mod_sprite_paths(root: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let root = resolve_mod_root(root)?;
    let mut regular_paths = Vec::new();
    let mut override_paths = Vec::new();

    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let regular_dir = root.join("sprites");
    if regular_dir.is_dir() {
        walk_relative_files(&root, &regular_dir, &mut regular_paths, accept_png_file)?;
    }

    let override_dir = root.join("sprites-override");
    if override_dir.is_dir() {
        walk_relative_files(&root, &override_dir, &mut override_paths, accept_png_file)?;
    }

    regular_paths.sort();
    override_paths.sort();
    regular_paths.extend(override_paths);
    Ok(regular_paths)
}

/// Build a generic `FileTree` overlay from the files discovered by
/// [`scan_mod_file_paths`].
pub fn mod_file_tree_from_directory(root: impl AsRef<Path>) -> io::Result<FileTree> {
    let mut tree = FileTree::new();
    for path in scan_mod_file_paths(root)? {
        tree.add_file(path.clone(), AssetFile::new(path, true));
    }
    Ok(tree)
}

fn walk_relative_files(
    root: &Path,
    current: &Path,
    out: &mut Vec<String>,
    accept: fn(&Path) -> bool,
) -> io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_relative_files(root, &path, out, accept)?;
            continue;
        }

        if accept(&path) {
            out.push(normalize_mod_resource_path(
                path.strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .into_owned(),
            ));
        }
    }

    Ok(())
}

fn accept_all_files(_: &Path) -> bool {
    true
}

fn accept_png_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
}

fn extract_mod_metadata_value(source: &str, key: &str) -> Option<String> {
    source
        .char_indices()
        .find_map(|(index, _)| metadata_value_start(source, index, key))
        .and_then(|value_start| parse_metadata_value(source, value_start))
        .filter(|value| !value.trim().is_empty())
}

fn metadata_value_start(source: &str, index: usize, key: &str) -> Option<usize> {
    if !is_metadata_key_boundary(source, index.checked_sub(1)) {
        return None;
    }

    let rest = source.get(index..)?;
    let after_key = if let Some(rest) = rest.strip_prefix('"') {
        let rest = rest.strip_prefix(key)?;
        rest.strip_prefix('"')?
    } else if let Some(rest) = rest.strip_prefix('\'') {
        let rest = rest.strip_prefix(key)?;
        rest.strip_prefix('\'')?
    } else {
        let rest = rest.strip_prefix(key)?;
        if rest.chars().next().is_some_and(is_metadata_identifier_char) {
            return None;
        }
        rest
    };

    let separator_index = after_key.char_indices().find_map(|(offset, ch)| {
        if ch.is_whitespace() {
            None
        } else {
            matches!(ch, ':' | '=').then_some(offset + ch.len_utf8())
        }
    })?;
    Some(index + rest.len() - after_key.len() + separator_index)
}

fn parse_metadata_value(source: &str, mut index: usize) -> Option<String> {
    while let Some(ch) = source.get(index..)?.chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        index += ch.len_utf8();
    }

    let first = source.get(index..)?.chars().next()?;
    if matches!(first, '"' | '\'') {
        let quote = first;
        let mut out = String::new();
        let mut escaped = false;
        for ch in source.get(index + quote.len_utf8()..)?.chars() {
            if escaped {
                out.push(match ch {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '"' => '"',
                    '\'' => '\'',
                    '\\' => '\\',
                    other => other,
                });
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote {
                return Some(out.trim().to_string());
            }
            out.push(ch);
        }
        return None;
    }

    let raw = source
        .get(index..)?
        .split(|ch| matches!(ch, ',' | '\n' | '\r' | '}'))
        .next()?
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string();
    Some(raw)
}

fn is_metadata_key_boundary(source: &str, previous_index: Option<usize>) -> bool {
    match previous_index.and_then(|index| source.get(index..)?.chars().next()) {
        Some(ch) => !is_metadata_identifier_char(ch),
        None => true,
    }
}

fn is_metadata_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

fn is_skipped_mod_container_entry(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_str(),
        Some(name)
            if name.starts_with('.')
                || matches!(name, "bundles" | "sprites" | "sprites-override")
    )
}

fn is_special_top_level_folder(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_str(),
        Some("bundles") | Some("sprites") | Some("sprites-override") | Some(".git")
    )
}

pub fn mod_sprite_atlas_name(
    mod_name: &str,
    source_path: &str,
    prefix_with_mod_name: bool,
) -> Option<String> {
    let base_name = sprite_file_base_name(source_path)?;
    if !prefix_with_mod_name || mod_sprite_name_is_category_prefixed(&base_name, mod_name) {
        return Some(base_name);
    }

    Some(format!("{mod_name}-{base_name}"))
}

fn mod_icon_candidates(headless: bool) -> Vec<String> {
    if headless {
        Vec::new()
    } else {
        vec!["icon.png".into(), "preview.png".into()]
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

fn normalize_mod_resource_path(path: String) -> String {
    path.trim()
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string()
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

fn sprite_file_base_name(source_path: &str) -> Option<String> {
    let normalized = source_path.trim().replace('\\', "/");
    let file_name = normalized.rsplit('/').next().unwrap_or_default();
    let base_name = file_name
        .rsplit_once('.')
        .map_or(file_name, |(base, _extension)| base)
        .trim();

    (!base_name.is_empty()).then(|| base_name.to_string())
}

fn mod_sprite_name_is_category_prefixed(base_name: &str, mod_name: &str) -> bool {
    base_name
        .split_once('-')
        .is_some_and(|(_category, rest)| rest.starts_with(&format!("{mod_name}-")))
}

#[cfg(test)]
mod tests {
    use crate::mindustry::graphics::TextureAtlasPlan;

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
                    texture_scale: TextureScale::default(),
                },
                SpritePackRequest {
                    source_path: "sprites-override/ui/icon.png".into(),
                    atlas_name: "icon".into(),
                    page_hint: "sprites-override".into(),
                    r#override: true,
                    texture_scale: TextureScale::default(),
                },
            ]
        );
    }

    #[test]
    fn mod_sprite_sources_follow_java_pack_sprites_naming_rules() {
        assert_eq!(
            mod_sprite_atlas_name("example", "sprites/blocks/router.png", true),
            Some("example-router".into())
        );
        assert_eq!(
            mod_sprite_atlas_name("example", "sprites/block-example-router.png", true),
            Some("block-example-router".into())
        );
        assert_eq!(
            mod_sprite_atlas_name("example", "sprites-override/router.png", false),
            Some("router".into())
        );
        assert_eq!(mod_sprite_atlas_name("example", "sprites/.png", true), None);
    }

    #[test]
    fn sprite_packer_imports_mod_sprite_sources_into_page_aware_requests() {
        let mut packer = SpritePacker::new();

        let imported = packer.extend_mod_sprite_sources([
            ModSpritePackSource::sprite("example", "mods/example/sprites/router.png"),
            ModSpritePackSource::sprite(
                "example",
                "mods/example/sprites/blocks/environment/ore.png",
            ),
            ModSpritePackSource::override_sprite(
                "example",
                "mods/example/sprites-override/ui/icon.png",
            ),
        ]);

        assert_eq!(imported, 3);
        assert_eq!(
            packer.requests(),
            &[
                SpritePackRequest {
                    source_path: "mods/example/sprites/router.png".into(),
                    atlas_name: "example-router".into(),
                    page_hint: "sprites".into(),
                    r#override: false,
                    texture_scale: TextureScale::default(),
                },
                SpritePackRequest {
                    source_path: "mods/example/sprites/blocks/environment/ore.png".into(),
                    atlas_name: "example-ore".into(),
                    page_hint: "sprites".into(),
                    r#override: false,
                    texture_scale: TextureScale::default(),
                },
                SpritePackRequest {
                    source_path: "mods/example/sprites-override/ui/icon.png".into(),
                    atlas_name: "icon".into(),
                    page_hint: "sprites-override".into(),
                    r#override: true,
                    texture_scale: TextureScale::default(),
                },
            ]
        );

        let plan = packer.to_multi_packer_plan();
        assert!(plan.page(PageType::Main).get("example-router").is_some());
        assert!(plan
            .page(PageType::Environment)
            .get("example-ore")
            .is_some());
        assert!(plan.page(PageType::Ui).get("icon").is_some());
    }

    #[test]
    fn mod_resource_plan_skips_icons_headless_and_prefers_icon_then_preview() {
        let headless = ModResourcePlan::new(true);
        let desktop = ModResourcePlan::new(false);

        assert!(headless.icon.headless);
        assert!(headless.icon.candidates.is_empty());
        assert!(!desktop.icon.headless);
        assert_eq!(
            desktop.icon.candidates,
            vec!["icon.png".to_string(), "preview.png".to_string()]
        );
    }

    #[test]
    fn mod_resource_plan_keeps_sprite_sources_connected_to_pack_requests() {
        let plan = ModResourcePlan::new(false).with_sprite_sources([
            ModSpritePackSource::sprite("example", "mods/example/sprites/router.png"),
            ModSpritePackSource::override_sprite(
                "example",
                "mods/example/sprites-override/ui/icon.png",
            ),
        ]);

        assert_eq!(
            plan.sprite_requests(),
            vec![
                SpritePackRequest {
                    source_path: "mods/example/sprites/router.png".into(),
                    atlas_name: "example-router".into(),
                    page_hint: "sprites".into(),
                    r#override: false,
                    texture_scale: TextureScale::default(),
                },
                SpritePackRequest {
                    source_path: "mods/example/sprites-override/ui/icon.png".into(),
                    atlas_name: "icon".into(),
                    page_hint: "sprites-override".into(),
                    r#override: true,
                    texture_scale: TextureScale::default(),
                },
            ]
        );
    }

    #[test]
    fn mod_resource_plan_scans_file_paths_into_sprite_sources() {
        let plan = ModResourcePlan::from_file_paths(
            "example",
            false,
            [
                "mods/example/sprites/router.png",
                "mods/example/sprites/ui/badge.png",
                "mods\\example\\sprites\\blocks\\environment\\ore.png",
                "mods/example/sprites-override/router.png",
                "mods/example/sprites-override/rubble/crack.png",
                "mods/example/icon.png",
                "mods/example/preview.png",
                "mods/example/sprites/readme.txt",
            ],
        );

        assert_eq!(
            plan.icon.candidates,
            vec!["icon.png".to_string(), "preview.png".to_string()]
        );
        assert_eq!(
            plan.sprite_requests(),
            vec![
                SpritePackRequest {
                    source_path: "mods/example/sprites/blocks/environment/ore.png".into(),
                    atlas_name: "example-ore".into(),
                    page_hint: "sprites".into(),
                    r#override: false,
                    texture_scale: TextureScale::default(),
                },
                SpritePackRequest {
                    source_path: "mods/example/sprites/router.png".into(),
                    atlas_name: "example-router".into(),
                    page_hint: "sprites".into(),
                    r#override: false,
                    texture_scale: TextureScale::default(),
                },
                SpritePackRequest {
                    source_path: "mods/example/sprites/ui/badge.png".into(),
                    atlas_name: "example-badge".into(),
                    page_hint: "sprites".into(),
                    r#override: false,
                    texture_scale: TextureScale::default(),
                },
                SpritePackRequest {
                    source_path: "mods/example/sprites-override/router.png".into(),
                    atlas_name: "router".into(),
                    page_hint: "sprites-override".into(),
                    r#override: true,
                    texture_scale: TextureScale::default(),
                },
                SpritePackRequest {
                    source_path: "mods/example/sprites-override/rubble/crack.png".into(),
                    atlas_name: "crack".into(),
                    page_hint: "sprites-override".into(),
                    r#override: true,
                    texture_scale: TextureScale::default(),
                },
            ]
        );

        let requests = plan.sprite_requests();
        assert_eq!(requests[0].page_type(), PageType::Environment);
        assert_eq!(requests[2].page_type(), PageType::Ui);
        assert_eq!(requests[4].page_type(), PageType::Rubble);
    }

    fn minimal_png_bytes(width: u32, height: u32) -> Vec<u8> {
        const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&PNG_SIGNATURE);
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes
    }

    fn temp_png_path(stem: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "mindustry-rust-{stem}-{}-{nanos}",
                std::process::id()
            ))
            .join("sprites")
            .join(format!("{stem}.png"))
    }

    fn write_minimal_png(path: &Path, width: u32, height: u32) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, minimal_png_bytes(width, height)).unwrap();
    }

    fn temp_mod_root(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!("{prefix}-{}-{nanos}", std::process::id()))
            .join("outer")
    }

    #[test]
    fn mod_resource_plan_to_texture_atlas_pipeline_reads_real_png_dimensions() {
        let path = temp_png_path("mod-resource-plan");
        let temp_root = path.parent().unwrap().parent().unwrap().to_path_buf();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, minimal_png_bytes(48, 24)).unwrap();
        let source_path = path.to_string_lossy().replace('\\', "/");

        let plan = ModResourcePlan::from_file_paths("example", false, [source_path.clone()]);
        let mut packer = SpritePacker::new();
        for request in plan.sprite_requests() {
            packer.add_request(request);
        }

        let multi_plan = packer.to_multi_packer_plan();
        let multi_region = multi_plan
            .page(PageType::Main)
            .get("example-mod-resource-plan")
            .unwrap();
        assert_eq!(multi_region.width, 48);
        assert_eq!(multi_region.height, 24);
        assert_eq!(multi_region.payload.source_path, source_path);

        let atlas = TextureAtlasPlan::from_pack_plan(multi_plan.into_pack_plan());
        let atlas_region = atlas.lookup("example-mod-resource-plan").unwrap().region;
        assert_eq!(atlas_region.width, 48);
        assert_eq!(atlas_region.height, 24);
        assert_eq!(atlas_region.source_path, source_path);

        let missing_region = SpritePackRequest::new("mods/example/sprites/missing.png", "missing")
            .to_region_request();
        assert_eq!(missing_region.width, 1);
        assert_eq!(missing_region.height, 1);

        std::fs::remove_dir_all(temp_root).unwrap();
    }

    #[test]
    fn mod_resource_plan_from_directory_unwraps_single_child_root_and_scans_sprites() {
        let outer_root = temp_mod_root("mod-root-scan");
        let mod_root = outer_root.join("example-pack");

        std::fs::create_dir_all(&mod_root).unwrap();
        std::fs::write(
            mod_root.join("mod.hjson"),
            br#"
name: example
displayName: "Example Pack"
author: "Rust Tester"
version: "1.2.3"
description: "Adds routers."
repo: "Anon/example"
"#,
        )
        .unwrap();
        write_minimal_png(&mod_root.join("sprites/router.png"), 16, 16);
        write_minimal_png(&mod_root.join("sprites-override/ui/icon.png"), 32, 32);
        write_minimal_png(&mod_root.join("sprites/blocks/environment/ore.png"), 24, 24);
        std::fs::create_dir_all(mod_root.join("bundles")).unwrap();
        std::fs::write(mod_root.join("bundles/bundle.properties"), b"hello=world").unwrap();
        std::fs::create_dir_all(mod_root.join(".git")).unwrap();
        std::fs::write(mod_root.join(".git/HEAD"), b"ref: refs/heads/main").unwrap();
        std::fs::create_dir_all(mod_root.join("assets/nested")).unwrap();
        std::fs::write(mod_root.join("assets/nested/keep.txt"), b"keep").unwrap();
        std::fs::write(mod_root.join("assets/readme.md"), b"readme").unwrap();

        let file_paths = scan_mod_file_paths(&outer_root).unwrap();
        assert_eq!(
            file_paths,
            vec![
                "assets/nested/keep.txt".to_string(),
                "assets/readme.md".to_string()
            ]
        );

        let sprite_paths = scan_mod_sprite_paths(&outer_root).unwrap();
        assert_eq!(
            sprite_paths,
            vec![
                "sprites/blocks/environment/ore.png".to_string(),
                "sprites/router.png".to_string(),
                "sprites-override/ui/icon.png".to_string(),
            ]
        );

        let plan = ModResourcePlan::from_directory("example", false, &outer_root).unwrap();
        assert_eq!(
            plan.icon.candidates,
            vec!["icon.png".to_string(), "preview.png".to_string()]
        );
        assert_eq!(
            plan.sprite_requests(),
            vec![
                SpritePackRequest {
                    source_path: "sprites/blocks/environment/ore.png".into(),
                    atlas_name: "example-ore".into(),
                    page_hint: "sprites".into(),
                    r#override: false,
                    texture_scale: TextureScale::default(),
                },
                SpritePackRequest {
                    source_path: "sprites/router.png".into(),
                    atlas_name: "example-router".into(),
                    page_hint: "sprites".into(),
                    r#override: false,
                    texture_scale: TextureScale::default(),
                },
                SpritePackRequest {
                    source_path: "sprites-override/ui/icon.png".into(),
                    atlas_name: "icon".into(),
                    page_hint: "sprites-override".into(),
                    r#override: true,
                    texture_scale: TextureScale::default(),
                },
            ]
        );

        let directory_plan =
            ModResourceDirectoryPlan::from_directory("example", false, &outer_root).unwrap();
        assert_eq!(directory_plan.meta.name.as_deref(), Some("example"));
        assert_eq!(
            directory_plan.meta.display_name.as_deref(),
            Some("Example Pack")
        );
        assert_eq!(directory_plan.meta.author.as_deref(), Some("Rust Tester"));
        assert_eq!(directory_plan.meta.version.as_deref(), Some("1.2.3"));
        assert_eq!(
            directory_plan.meta.description.as_deref(),
            Some("Adds routers.")
        );
        assert_eq!(directory_plan.meta.repo.as_deref(), Some("Anon/example"));
        assert_eq!(
            directory_plan.meta.source_path.as_deref(),
            Some("mod.hjson")
        );

        let _ = std::fs::remove_dir_all(&outer_root);
    }

    #[test]
    fn mod_file_tree_from_directory_unwraps_root_and_keeps_generic_assets_only() {
        let outer_root = temp_mod_root("mod-file-tree");
        let mod_root = outer_root.join("example-pack");

        std::fs::create_dir_all(mod_root.join("assets/nested")).unwrap();
        std::fs::write(mod_root.join("assets/foo.txt"), b"foo").unwrap();
        std::fs::write(mod_root.join("assets/nested/bar.txt"), b"bar").unwrap();

        std::fs::create_dir_all(mod_root.join("sprites/blocks")).unwrap();
        std::fs::write(mod_root.join("sprites/blocks/router.png"), b"sprite").unwrap();
        std::fs::create_dir_all(mod_root.join("bundles")).unwrap();
        std::fs::write(mod_root.join("bundles/messages.properties"), b"hello=world").unwrap();
        std::fs::create_dir_all(mod_root.join(".git")).unwrap();
        std::fs::write(mod_root.join(".git/HEAD"), b"ref: refs/heads/main").unwrap();

        let file_paths = scan_mod_file_paths(&outer_root).unwrap();
        assert_eq!(
            file_paths,
            vec![
                "assets/foo.txt".to_string(),
                "assets/nested/bar.txt".to_string()
            ]
        );

        let file_tree = mod_file_tree_from_directory(&outer_root).unwrap();
        assert_eq!(file_tree.file_count(), 2);
        assert_eq!(
            file_tree.resolve("assets/foo.txt"),
            AssetFile::new("assets/foo.txt", true)
        );
        assert_eq!(
            file_tree.resolve("assets\\nested\\bar.txt"),
            AssetFile::new("assets/nested/bar.txt", true)
        );
        assert_eq!(
            file_tree.get("sprites/blocks/router.png"),
            AssetFile::missing("sprites/blocks/router.png")
        );
        assert_eq!(
            file_tree.get("bundles/messages.properties"),
            AssetFile::missing("bundles/messages.properties")
        );
        assert_eq!(file_tree.get(".git/HEAD"), AssetFile::missing(".git/HEAD"));

        let _ = std::fs::remove_dir_all(&outer_root);
    }

    #[test]
    fn mod_resource_container_plan_discovers_multiple_mods_and_skips_non_mod_entries() {
        let outer_root = temp_mod_root("mod-container-plan");
        let alpha_root = outer_root.join("alpha");
        let beta_outer_root = outer_root.join("beta");
        let beta_inner_root = beta_outer_root.join("example-pack");

        std::fs::create_dir_all(alpha_root.join("assets/nested")).unwrap();
        std::fs::write(alpha_root.join("assets/nested/alpha.txt"), b"alpha").unwrap();
        write_minimal_png(&alpha_root.join("sprites/router.png"), 16, 16);
        write_minimal_png(&alpha_root.join("sprites-override/ui/icon.png"), 32, 32);

        std::fs::create_dir_all(beta_inner_root.join("assets")).unwrap();
        std::fs::write(beta_inner_root.join("assets/beta.txt"), b"beta").unwrap();
        write_minimal_png(
            &beta_inner_root.join("sprites-override/rubble/crack.png"),
            24,
            24,
        );

        std::fs::write(outer_root.join("README.txt"), b"ignore").unwrap();
        std::fs::create_dir_all(outer_root.join(".git")).unwrap();
        std::fs::write(outer_root.join(".git/HEAD"), b"ref: refs/heads/main").unwrap();
        std::fs::create_dir_all(outer_root.join(".hidden-mod")).unwrap();
        std::fs::create_dir_all(outer_root.join("bundles")).unwrap();
        std::fs::write(
            outer_root.join("bundles/messages.properties"),
            b"hello=world",
        )
        .unwrap();
        std::fs::create_dir_all(outer_root.join("sprites/ui")).unwrap();
        write_minimal_png(&outer_root.join("sprites/ui/root.png"), 8, 8);
        std::fs::create_dir_all(outer_root.join("sprites-override/ui")).unwrap();
        write_minimal_png(&outer_root.join("sprites-override/ui/root.png"), 8, 8);

        let container =
            ModResourceContainerPlan::discover_from_mods_directory(&outer_root, false).unwrap();

        assert_eq!(
            container
                .mods
                .iter()
                .map(|plan| plan.mod_name.clone())
                .collect::<Vec<_>>(),
            vec!["alpha".to_string(), "beta".to_string()]
        );

        let alpha = &container.mods[0];
        assert_eq!(
            alpha.root.file_name().and_then(|name| name.to_str()),
            Some("alpha")
        );
        assert_eq!(alpha.file_tree.file_count(), 1);
        assert_eq!(
            alpha.file_tree.resolve("assets/nested/alpha.txt"),
            AssetFile::new("assets/nested/alpha.txt", true)
        );
        assert_eq!(
            alpha.resource_plan.sprite_requests(),
            vec![
                SpritePackRequest {
                    source_path: "sprites/router.png".into(),
                    atlas_name: "alpha-router".into(),
                    page_hint: "sprites".into(),
                    r#override: false,
                    texture_scale: TextureScale::default(),
                },
                SpritePackRequest {
                    source_path: "sprites-override/ui/icon.png".into(),
                    atlas_name: "icon".into(),
                    page_hint: "sprites-override".into(),
                    r#override: true,
                    texture_scale: TextureScale::default(),
                },
            ]
        );

        let beta = &container.mods[1];
        assert_eq!(
            beta.root.file_name().and_then(|name| name.to_str()),
            Some("example-pack")
        );
        assert_eq!(beta.file_tree.file_count(), 1);
        assert_eq!(
            beta.file_tree.resolve("assets/beta.txt"),
            AssetFile::new("assets/beta.txt", true)
        );
        assert_eq!(
            beta.resource_plan.sprite_requests(),
            vec![SpritePackRequest {
                source_path: "sprites-override/rubble/crack.png".into(),
                atlas_name: "crack".into(),
                page_hint: "sprites-override".into(),
                r#override: true,
                texture_scale: TextureScale::default(),
            }]
        );

        assert_eq!(container.mods.len(), 2);
        let _ = std::fs::remove_dir_all(&outer_root);
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
    fn sprite_pack_request_with_texture_scale_round_trips_through_packer_and_atlas() {
        let mut packer = SpritePacker::new();
        packer.add_request(
            SpritePackRequest::new("sprites/scaled.png", "scaled")
                .with_page_hint("sprites")
                .with_texture_scale(2.0),
        );

        let plan = packer.to_multi_packer_plan_with_size(8, 16);
        let request = plan.page(PageType::Main).get("scaled").unwrap();
        assert_eq!(request.width, 8);
        assert_eq!(request.height, 16);
        assert_eq!(request.payload.source_path, "sprites/scaled.png");
        assert_eq!(request.payload.texture_scale, TextureScale::new(2.0));

        let atlas = TextureAtlasPlan::from_pack_plan(plan.into_pack_plan());
        let region = atlas.lookup("scaled").unwrap().region;
        assert_eq!(region.source_path, "sprites/scaled.png");
        assert_eq!(region.width, 8);
        assert_eq!(region.height, 16);
        assert_eq!(region.scale, 2.0);
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
