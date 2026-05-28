use super::multi_packer::{MultiPackerPlan, PackPlan, PagePlan, PageSpec, PageType, RegionRequest};

impl PageType {
    /// 上游 `sprites*.png` 的默认页资源路径。
    pub const fn atlas_source_path(self) -> &'static str {
        match self {
            Self::Main => "sprites.png",
            Self::Environment => "sprites2.png",
            Self::Ui => "sprites3.png",
            Self::Rubble => "sprites4.png",
        }
    }
}

/// 一条 atlas region 的输入源：在 `RegionRequest` 上附加 source path。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureAtlasRegionSource<T = ()> {
    pub source_path: String,
    pub payload: T,
}

impl<T> TextureAtlasRegionSource<T> {
    pub fn new(source_path: impl Into<String>, payload: T) -> Self {
        Self {
            source_path: source_path.into(),
            payload,
        }
    }

    pub fn map_payload<U>(self, payload: U) -> TextureAtlasRegionSource<U> {
        TextureAtlasRegionSource {
            source_path: self.source_path,
            payload,
        }
    }
}

/// 纯数据化的 sprite 来源描述：
///
/// - `source_path`：真实或虚拟文件路径
/// - `atlas_name`：atlas 中的 region 名称
/// - `page_hint`：可选 page 提示，空字符串时会回退到路径推断
/// - `override`：是否覆盖已有同名条目
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureAtlasSpriteSourceDescriptor {
    pub source_path: String,
    pub atlas_name: String,
    pub page_hint: String,
    pub r#override: bool,
}

impl TextureAtlasSpriteSourceDescriptor {
    pub fn new(source_path: impl Into<String>, atlas_name: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            atlas_name: atlas_name.into(),
            page_hint: String::new(),
            r#override: false,
        }
    }

    pub fn from_source_path(source_path: impl Into<String>) -> Self {
        let source_path = source_path.into();
        let atlas_name = derive_atlas_name(&source_path);

        Self::new(source_path, atlas_name)
    }

    /// 从“虚拟目录/清单扫描路径”构建纯数据描述符。
    ///
    /// 该入口只解析路径结构，不触发任何 PNG/GPU 相关逻辑：
    /// - `sprites` / `sprites-override` 作为根目录时，会被保留为 page hint
    /// - `ui` / `environment` / `rubble` 顶层目录会直接路由到对应 page
    /// - `sprites-override` 根目录会自动标记为 override
    pub fn from_virtual_source_path(source_path: impl Into<String>) -> Self {
        let source_path = source_path.into();
        let atlas_name = derive_atlas_name(&source_path);
        let (page_hint, r#override) = infer_virtual_sprite_hint(&source_path);

        Self {
            source_path,
            atlas_name,
            page_hint,
            r#override,
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

    pub fn page_type(&self) -> PageType {
        resolve_sprite_page_type(&self.page_hint, &self.source_path)
    }

    pub fn to_region_request(&self) -> RegionRequest<TextureAtlasRegionSource<bool>> {
        self.to_region_request_with_size(1, 1)
    }

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

fn derive_atlas_name(source_path: &str) -> String {
    let normalized = normalize_sprite_hint(source_path);
    let file_name = normalized.rsplit('/').next().unwrap_or_default();

    if file_name.is_empty() {
        return normalized;
    }

    match file_name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() => stem.to_string(),
        _ => file_name.to_string(),
    }
}

fn infer_virtual_sprite_hint(source_path: &str) -> (String, bool) {
    let normalized = normalize_sprite_hint(source_path);
    let mut segments = normalized.split('/').filter(|segment| !segment.is_empty());
    let Some(root) = segments.next() else {
        return (String::new(), false);
    };

    match root {
        "sprites-override" => {
            let page_hint = match segments.next() {
                Some("ui") => "sprites-override/ui".to_string(),
                Some("environment") => "sprites-override/environment".to_string(),
                Some("rubble") => "sprites-override/rubble".to_string(),
                _ => "sprites-override".to_string(),
            };
            (page_hint, true)
        }
        "sprites" => {
            let page_hint = match segments.next() {
                Some("ui") => "ui".to_string(),
                Some("environment") => "environment".to_string(),
                Some("rubble") => "rubble".to_string(),
                _ => "sprites".to_string(),
            };
            (page_hint, false)
        }
        "ui" | "environment" | "rubble" => (root.to_string(), false),
        _ => (String::new(), false),
    }
}

/// atlas 中的一条 region：纯数据，不绑定任何后端纹理对象。
#[derive(Debug, Clone, PartialEq)]
pub struct TextureAtlasRegion<T = ()> {
    pub page_type: PageType,
    pub name: String,
    pub source_path: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub splits: Option<[i32; 4]>,
    pub pads: Option<[i32; 4]>,
    pub u: f32,
    pub v: f32,
    pub u2: f32,
    pub v2: f32,
    pub payload: T,
}

impl<T> TextureAtlasRegion<T> {
    pub fn new(
        page_type: PageType,
        name: impl Into<String>,
        source_path: impl Into<String>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        payload: T,
    ) -> Self {
        Self {
            page_type,
            name: name.into(),
            source_path: source_path.into(),
            x,
            y,
            width,
            height,
            splits: None,
            pads: None,
            u: 0.0,
            v: 0.0,
            u2: 0.0,
            v2: 0.0,
            payload,
        }
    }

    pub fn from_request(
        page_type: PageType,
        request: RegionRequest<TextureAtlasRegionSource<T>>,
    ) -> Self {
        let RegionRequest {
            name,
            width,
            height,
            splits,
            pads,
            payload,
        } = request;

        Self {
            page_type,
            name,
            source_path: payload.source_path,
            x: 0,
            y: 0,
            width,
            height,
            splits,
            pads,
            u: 0.0,
            v: 0.0,
            u2: 0.0,
            v2: 0.0,
            payload: payload.payload,
        }
    }

    pub fn with_position(mut self, x: u32, y: u32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_splits(mut self, splits: [i32; 4]) -> Self {
        self.splits = Some(splits);
        self
    }

    pub fn with_pads(mut self, pads: [i32; 4]) -> Self {
        self.pads = Some(pads);
        self
    }

    pub fn with_meta(mut self, splits: Option<[i32; 4]>, pads: Option<[i32; 4]>) -> Self {
        self.splits = splits;
        self.pads = pads;
        self
    }

    pub fn with_uv(mut self, u: f32, v: f32, u2: f32, v2: f32) -> Self {
        self.u = u;
        self.v = v;
        self.u2 = u2;
        self.v2 = v2;
        self
    }

    pub fn sync_uv(&mut self, page_width: u32, page_height: u32) {
        if page_width == 0 || page_height == 0 {
            self.u = 0.0;
            self.v = 0.0;
            self.u2 = 0.0;
            self.v2 = 0.0;
            return;
        }

        let page_width = page_width as f32;
        let page_height = page_height as f32;
        self.u = self.x as f32 / page_width;
        self.v = self.y as f32 / page_height;
        self.u2 = self.x.saturating_add(self.width) as f32 / page_width;
        self.v2 = self.y.saturating_add(self.height) as f32 / page_height;
    }
}

/// 单个 atlas page 的装载结果：保存 page 本身与其 region 列表。
#[derive(Debug, Clone, PartialEq)]
pub struct TextureAtlasPage<T = ()> {
    pub page_type: PageType,
    pub source_path: String,
    pub spec: PageSpec,
    pub regions: Vec<TextureAtlasRegion<T>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureAtlasInsertError<T> {
    pub page_type: PageType,
    pub region: TextureAtlasRegion<T>,
}

impl<T> TextureAtlasPage<T> {
    pub fn new(page_type: PageType) -> Self {
        Self {
            page_type,
            source_path: page_type.atlas_source_path().to_string(),
            spec: page_type.spec(),
            regions: Vec::new(),
        }
    }

    pub fn with_source_path(mut self, source_path: impl Into<String>) -> Self {
        self.source_path = source_path.into();
        self
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    pub fn len(&self) -> usize {
        self.regions.len()
    }

    pub fn get(&self, name: &str) -> Option<&TextureAtlasRegion<T>> {
        self.regions.iter().find(|region| region.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut TextureAtlasRegion<T>> {
        self.regions.iter_mut().find(|region| region.name == name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn insert_region(
        &mut self,
        mut region: TextureAtlasRegion<T>,
    ) -> Result<(), TextureAtlasInsertError<T>> {
        if self.contains(&region.name) {
            return Err(TextureAtlasInsertError {
                page_type: self.page_type,
                region,
            });
        }

        region.page_type = self.page_type;
        region.sync_uv(self.spec.width, self.spec.height);
        self.regions.push(region);
        Ok(())
    }

    pub fn replace_region(
        &mut self,
        mut region: TextureAtlasRegion<T>,
    ) -> Option<TextureAtlasRegion<T>> {
        if let Some(index) = self
            .regions
            .iter()
            .position(|item| item.name == region.name)
        {
            region.page_type = self.page_type;
            region.sync_uv(self.spec.width, self.spec.height);
            Some(std::mem::replace(&mut self.regions[index], region))
        } else {
            let _ = self.insert_region(region);
            None
        }
    }

    pub fn insert_or_replace_region(
        &mut self,
        region: TextureAtlasRegion<T>,
    ) -> Option<TextureAtlasRegion<T>> {
        self.replace_region(region)
    }

    pub fn remove_region(&mut self, name: &str) -> Option<TextureAtlasRegion<T>> {
        let index = self.regions.iter().position(|region| region.name == name)?;
        Some(self.regions.remove(index))
    }

    pub fn refresh_uvs(&mut self) {
        for region in &mut self.regions {
            region.sync_uv(self.spec.width, self.spec.height);
        }
    }

    pub fn from_page_plan(page_plan: PagePlan<TextureAtlasRegionSource<T>>) -> Self {
        let mut page = Self::new(page_plan.page_type);
        page.spec = page_plan.spec;

        for request in page_plan.requests {
            if page
                .insert_region(TextureAtlasRegion::from_request(page.page_type, request))
                .is_err()
            {
                panic!("page plan should not contain duplicate region names");
            }
        }

        page
    }
}

impl<T> Default for TextureAtlasPage<T> {
    fn default() -> Self {
        Self::new(PageType::Main)
    }
}

/// 查找结果：保留 page 信息，便于后续后端做精确的定位和调试。
#[derive(Debug, Clone, Copy)]
pub struct LocatedTextureAtlasRegion<'a, T> {
    pub page_type: PageType,
    pub page_source_path: &'a str,
    pub region: &'a TextureAtlasRegion<T>,
}

/// 查找失败时保留的最小信息，方便区分全局 miss 与 page 内 miss。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureAtlasLookupMiss {
    pub page_type: Option<PageType>,
    pub page_source_path: Option<String>,
    pub name: String,
}

impl TextureAtlasLookupMiss {
    pub fn global(name: impl Into<String>) -> Self {
        Self {
            page_type: None,
            page_source_path: None,
            name: name.into(),
        }
    }

    pub fn in_page(
        page_type: PageType,
        page_source_path: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            page_type: Some(page_type),
            page_source_path: Some(page_source_path.into()),
            name: name.into(),
        }
    }
}

/// backend-neutral 的 atlas plan/registry：按 Mindustry page 顺序保存并可直接查找。
#[derive(Debug, Clone, PartialEq)]
pub struct TextureAtlasPlan<T = ()> {
    pub pages: [TextureAtlasPage<T>; 4],
}

impl<T> TextureAtlasPlan<T> {
    pub fn new() -> Self {
        Self {
            pages: [
                TextureAtlasPage::new(PageType::Main),
                TextureAtlasPage::new(PageType::Environment),
                TextureAtlasPage::new(PageType::Ui),
                TextureAtlasPage::new(PageType::Rubble),
            ],
        }
    }

    pub fn page(&self, page_type: PageType) -> &TextureAtlasPage<T> {
        &self.pages[page_type.index()]
    }

    pub fn page_mut(&mut self, page_type: PageType) -> &mut TextureAtlasPage<T> {
        &mut self.pages[page_type.index()]
    }

    pub fn get(&self, name: &str) -> Option<LocatedTextureAtlasRegion<'_, T>> {
        self.pages.iter().find_map(|page| {
            page.get(name).map(|region| LocatedTextureAtlasRegion {
                page_type: page.page_type,
                page_source_path: &page.source_path,
                region,
            })
        })
    }

    pub fn lookup(
        &self,
        name: &str,
    ) -> Result<LocatedTextureAtlasRegion<'_, T>, TextureAtlasLookupMiss> {
        self.get(name)
            .ok_or_else(|| TextureAtlasLookupMiss::global(name))
    }

    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn get_in_page(&self, page_type: PageType, name: &str) -> Option<&TextureAtlasRegion<T>> {
        self.page(page_type).get(name)
    }

    pub fn lookup_in_page(
        &self,
        page_type: PageType,
        name: &str,
    ) -> Result<&TextureAtlasRegion<T>, TextureAtlasLookupMiss> {
        let page = self.page(page_type);
        page.get(name).ok_or_else(|| {
            TextureAtlasLookupMiss::in_page(page_type, page.source_path.clone(), name)
        })
    }

    pub fn has_in_page(&self, page_type: PageType, name: &str) -> bool {
        self.get_in_page(page_type, name).is_some()
    }

    pub fn insert_region(
        &mut self,
        page_type: PageType,
        region: TextureAtlasRegion<T>,
    ) -> Result<(), TextureAtlasInsertError<T>> {
        self.page_mut(page_type).insert_region(region)
    }

    pub fn replace_region(
        &mut self,
        page_type: PageType,
        region: TextureAtlasRegion<T>,
    ) -> Option<TextureAtlasRegion<T>> {
        self.page_mut(page_type).replace_region(region)
    }

    pub fn insert_or_replace_region(
        &mut self,
        page_type: PageType,
        region: TextureAtlasRegion<T>,
    ) -> Option<TextureAtlasRegion<T>> {
        self.page_mut(page_type).insert_or_replace_region(region)
    }

    pub fn remove_region(
        &mut self,
        page_type: PageType,
        name: &str,
    ) -> Option<TextureAtlasRegion<T>> {
        self.page_mut(page_type).remove_region(name)
    }

    pub fn clear_page(&mut self, page_type: PageType) {
        self.page_mut(page_type).regions.clear();
    }

    pub fn refresh_uvs(&mut self) {
        for page in &mut self.pages {
            page.refresh_uvs();
        }
    }

    pub fn from_pack_plan(pack_plan: PackPlan<TextureAtlasRegionSource<T>>) -> Self {
        let mut atlas = Self::new();

        for page_plan in pack_plan.pages {
            let page_type = page_plan.page_type;
            atlas.pages[page_type.index()] = TextureAtlasPage::from_page_plan(page_plan);
        }

        atlas
    }
}

impl TextureAtlasPlan<bool> {
    pub fn from_sprite_sources<I>(sources: I) -> Self
    where
        I: IntoIterator<Item = TextureAtlasSpriteSourceDescriptor>,
    {
        let mut atlas = Self::new();

        for source in sources {
            let page_type = source.page_type();
            let region = TextureAtlasRegion::new(
                page_type,
                source.atlas_name,
                source.source_path,
                0,
                0,
                1,
                1,
                source.r#override,
            );
            let _ = atlas.insert_or_replace_region(page_type, region);
        }

        atlas
    }

    pub fn from_source_paths<I, S>(paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::from_sprite_sources(
            paths
                .into_iter()
                .map(TextureAtlasSpriteSourceDescriptor::from_source_path),
        )
    }

    pub fn from_virtual_source_paths<I, S>(paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::from_sprite_sources(
            paths
                .into_iter()
                .map(TextureAtlasSpriteSourceDescriptor::from_virtual_source_path),
        )
    }
}

impl PackPlan<TextureAtlasRegionSource<bool>> {
    pub fn from_sprite_sources<I>(sources: I) -> Self
    where
        I: IntoIterator<Item = TextureAtlasSpriteSourceDescriptor>,
    {
        let mut plan = MultiPackerPlan::new();

        for source in sources {
            let page_type = source.page_type();
            let request = source.to_region_request();
            let _ = plan.insert_or_replace_request(page_type, request);
        }

        plan.into_pack_plan()
    }

    pub fn from_source_paths<I, S>(paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::from_sprite_sources(
            paths
                .into_iter()
                .map(TextureAtlasSpriteSourceDescriptor::from_source_path),
        )
    }

    pub fn from_virtual_source_paths<I, S>(paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::from_sprite_sources(
            paths
                .into_iter()
                .map(TextureAtlasSpriteSourceDescriptor::from_virtual_source_path),
        )
    }
}

impl<T> Default for TextureAtlasPlan<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_f32_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-6,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn texture_atlas_page_type_source_paths_match_mindustry_sprite_pages() {
        assert_eq!(PageType::Main.atlas_source_path(), "sprites.png");
        assert_eq!(PageType::Environment.atlas_source_path(), "sprites2.png");
        assert_eq!(PageType::Ui.atlas_source_path(), "sprites3.png");
        assert_eq!(PageType::Rubble.atlas_source_path(), "sprites4.png");
    }

    #[test]
    fn texture_atlas_sprite_source_descriptor_derives_name_and_page_type() {
        let main =
            TextureAtlasSpriteSourceDescriptor::from_source_path("sprites/blocks/stone-wall.png");
        let environment =
            TextureAtlasSpriteSourceDescriptor::new("sprites/blocks/environment/ore.png", "ore")
                .with_page_hint("environment")
                .with_override(true);
        let ui = TextureAtlasSpriteSourceDescriptor::new("assets/ignored/icon.png", "icon")
            .with_page_hint("sprites/ui");

        assert_eq!(main.atlas_name, "stone-wall");
        assert_eq!(main.page_type(), PageType::Main);
        assert_eq!(environment.page_type(), PageType::Environment);
        assert!(environment.r#override);
        assert_eq!(ui.page_type(), PageType::Ui);

        let request = environment.to_region_request_with_size(0, 8);
        assert_eq!(request.name, "ore");
        assert_eq!(request.width, 1);
        assert_eq!(request.height, 8);
        assert_eq!(
            request.payload.source_path,
            "sprites/blocks/environment/ore.png"
        );
        assert!(request.payload.payload);
    }

    #[test]
    fn texture_atlas_sprite_source_descriptor_from_virtual_source_path_routes_pages_and_override() {
        let main =
            TextureAtlasSpriteSourceDescriptor::from_virtual_source_path("sprites/block.png");
        let ui = TextureAtlasSpriteSourceDescriptor::from_virtual_source_path(
            "sprites-override/ui/icon.png",
        );
        let environment =
            TextureAtlasSpriteSourceDescriptor::from_virtual_source_path("environment/ore.png");
        let rubble = TextureAtlasSpriteSourceDescriptor::from_virtual_source_path(
            "sprites-override/rubble/crack.png",
        );

        assert_eq!(main.atlas_name, "block");
        assert_eq!(main.page_type(), PageType::Main);
        assert!(!main.r#override);

        assert_eq!(ui.atlas_name, "icon");
        assert_eq!(ui.page_type(), PageType::Ui);
        assert!(ui.r#override);

        assert_eq!(environment.atlas_name, "ore");
        assert_eq!(environment.page_type(), PageType::Environment);
        assert!(!environment.r#override);

        assert_eq!(rubble.atlas_name, "crack");
        assert_eq!(rubble.page_type(), PageType::Rubble);
        assert!(rubble.r#override);
    }

    #[test]
    fn texture_atlas_plan_can_be_built_from_source_paths_without_image_loading() {
        let plan = TextureAtlasPlan::from_source_paths([
            "sprites/blocks/stone-wall.png",
            "sprites/ui/button.png",
            "sprites-override/rubble/crack.png",
        ]);

        let main = plan.page(PageType::Main).get("stone-wall").unwrap();
        assert_eq!(main.source_path, "sprites/blocks/stone-wall.png");
        assert_eq!(main.payload, false);
        assert_eq!(main.width, 1);
        assert_eq!(main.height, 1);

        let ui = plan.page(PageType::Ui).get("button").unwrap();
        assert_eq!(ui.source_path, "sprites/ui/button.png");
        assert_eq!(ui.payload, false);

        let rubble = plan.page(PageType::Rubble).get("crack").unwrap();
        assert_eq!(rubble.source_path, "sprites-override/rubble/crack.png");
        assert_eq!(rubble.payload, false);
    }

    #[test]
    fn texture_atlas_plan_can_be_built_from_virtual_source_paths_without_image_loading() {
        let plan = TextureAtlasPlan::from_virtual_source_paths([
            "sprites/blocks/stone-wall.png",
            "sprites-override/ui/button.png",
            "environment/ore.png",
            "sprites-override/blocks/environment/ore-override.png",
            "rubble/crack.png",
        ]);

        let main = plan.page(PageType::Main).get("stone-wall").unwrap();
        assert_eq!(main.source_path, "sprites/blocks/stone-wall.png");
        assert_eq!(main.payload, false);

        let ui = plan.page(PageType::Ui).get("button").unwrap();
        assert_eq!(ui.source_path, "sprites-override/ui/button.png");
        assert!(ui.payload);

        let environment = plan.page(PageType::Environment).get("ore").unwrap();
        assert_eq!(environment.source_path, "environment/ore.png");
        assert_eq!(environment.payload, false);

        let environment_override = plan
            .page(PageType::Environment)
            .get("ore-override")
            .unwrap();
        assert_eq!(
            environment_override.source_path,
            "sprites-override/blocks/environment/ore-override.png"
        );
        assert!(environment_override.payload);

        let rubble = plan.page(PageType::Rubble).get("crack").unwrap();
        assert_eq!(rubble.source_path, "rubble/crack.png");
        assert_eq!(rubble.payload, false);
    }

    #[test]
    fn texture_atlas_page_insert_region_recomputes_uv_and_rejects_duplicates() {
        let mut page = TextureAtlasPage::new(PageType::Main);

        page.insert_region(
            TextureAtlasRegion::new(
                PageType::Main,
                "core",
                "sprites/core.png",
                16,
                32,
                64,
                128,
                "payload",
            )
            .with_splits([1, 2, 3, 4])
            .with_pads([5, 6, 7, 8]),
        )
        .unwrap();

        let region = page.get("core").unwrap();
        assert_eq!(region.page_type, PageType::Main);
        assert_eq!(region.source_path, "sprites/core.png");
        assert_eq!(region.width, 64);
        assert_eq!(region.height, 128);
        assert_eq!(region.splits, Some([1, 2, 3, 4]));
        assert_eq!(region.pads, Some([5, 6, 7, 8]));
        assert_f32_close(region.u, 16.0 / 4096.0);
        assert_f32_close(region.v, 32.0 / 4096.0);
        assert_f32_close(region.u2, 80.0 / 4096.0);
        assert_f32_close(region.v2, 160.0 / 4096.0);

        let duplicate = page
            .insert_region(TextureAtlasRegion::new(
                PageType::Main,
                "core",
                "sprites/core-dup.png",
                0,
                0,
                1,
                1,
                "dup",
            ))
            .unwrap_err();
        assert_eq!(duplicate.page_type, PageType::Main);
        assert_eq!(duplicate.region.name, "core");
    }

    #[test]
    fn texture_atlas_plan_from_pack_plan_is_searchable_and_reports_lookup_misses() {
        let mut main = PagePlan::new(PageType::Main);
        main.insert_request(RegionRequest::new(
            "white",
            16,
            16,
            TextureAtlasRegionSource::new("sprites/white.png", "main"),
        ))
        .unwrap();

        let mut ui = PagePlan::new(PageType::Ui);
        ui.insert_request(
            RegionRequest::new(
                "whiteui",
                32,
                32,
                TextureAtlasRegionSource::new("sprites/ui/whiteui.png", "ui"),
            )
            .with_splits([1, 2, 3, 4])
            .with_pads([5, 6, 7, 8]),
        )
        .unwrap();

        let plan = TextureAtlasPlan::from_pack_plan(PackPlan::new(vec![ui, main]));

        assert_eq!(plan.pages.len(), 4);
        assert_eq!(plan.pages[0].page_type, PageType::Main);
        assert_eq!(plan.pages[0].source_path, "sprites.png");
        assert_eq!(plan.pages[1].page_type, PageType::Environment);
        assert!(plan.pages[1].is_empty());
        assert_eq!(plan.pages[2].page_type, PageType::Ui);
        assert_eq!(plan.pages[3].page_type, PageType::Rubble);

        let white = plan.get("white").expect("white should be searchable");
        assert_eq!(white.page_type, PageType::Main);
        assert_eq!(white.page_source_path, "sprites.png");
        assert_eq!(white.region.source_path, "sprites/white.png");
        assert_eq!(white.region.payload, "main");
        assert_eq!(white.region.width, 16);
        assert_eq!(white.region.height, 16);

        let whiteui = plan.lookup_in_page(PageType::Ui, "whiteui").unwrap();
        assert_eq!(whiteui.source_path, "sprites/ui/whiteui.png");
        assert_eq!(whiteui.payload, "ui");
        assert_eq!(whiteui.splits, Some([1, 2, 3, 4]));
        assert_eq!(whiteui.pads, Some([5, 6, 7, 8]));

        let miss = plan.lookup("missing").unwrap_err();
        assert_eq!(
            miss,
            TextureAtlasLookupMiss {
                page_type: None,
                page_source_path: None,
                name: "missing".to_string(),
            }
        );

        let page_miss = plan
            .lookup_in_page(PageType::Rubble, "missing")
            .unwrap_err();
        assert_eq!(
            page_miss,
            TextureAtlasLookupMiss {
                page_type: Some(PageType::Rubble),
                page_source_path: Some("sprites4.png".to_string()),
                name: "missing".to_string(),
            }
        );
    }

    #[test]
    fn texture_atlas_pack_plan_can_be_built_from_source_descriptors() {
        let plan = PackPlan::from_sprite_sources([
            TextureAtlasSpriteSourceDescriptor::from_source_path("sprites/block.png")
                .with_override(false),
            TextureAtlasSpriteSourceDescriptor::new("sprites/ui/icon.png", "icon")
                .with_page_hint("ui")
                .with_override(true),
            TextureAtlasSpriteSourceDescriptor::new("sprites-override/rubble/crack.png", "crack")
                .with_page_hint("sprites-override"),
        ]);

        let block = plan.page(PageType::Main).unwrap().get("block").unwrap();
        assert_eq!(block.payload.source_path, "sprites/block.png");
        assert!(!block.payload.payload);

        let icon = plan.page(PageType::Ui).unwrap().get("icon").unwrap();
        assert_eq!(icon.payload.source_path, "sprites/ui/icon.png");
        assert!(icon.payload.payload);

        let crack = plan.page(PageType::Rubble).unwrap().get("crack").unwrap();
        assert_eq!(
            crack.payload.source_path,
            "sprites-override/rubble/crack.png"
        );
        assert!(!crack.payload.payload);
    }

    #[test]
    fn texture_atlas_pack_plan_can_be_built_from_virtual_source_paths() {
        let plan = PackPlan::from_virtual_source_paths([
            "sprites/block.png",
            "sprites-override/ui/icon.png",
            "environment/ore.png",
            "sprites-override/rubble/crack.png",
        ]);

        let block = plan.page(PageType::Main).unwrap().get("block").unwrap();
        assert_eq!(block.payload.source_path, "sprites/block.png");
        assert!(!block.payload.payload);

        let icon = plan.page(PageType::Ui).unwrap().get("icon").unwrap();
        assert_eq!(icon.payload.source_path, "sprites-override/ui/icon.png");
        assert!(icon.payload.payload);

        let ore = plan
            .page(PageType::Environment)
            .unwrap()
            .get("ore")
            .unwrap();
        assert_eq!(ore.payload.source_path, "environment/ore.png");
        assert!(!ore.payload.payload);

        let crack = plan.page(PageType::Rubble).unwrap().get("crack").unwrap();
        assert_eq!(
            crack.payload.source_path,
            "sprites-override/rubble/crack.png"
        );
        assert!(crack.payload.payload);
    }

    #[test]
    fn texture_atlas_plan_mutation_refreshes_uvs_in_page_order() {
        let mut plan = TextureAtlasPlan::new();
        plan.insert_region(
            PageType::Rubble,
            TextureAtlasRegion::new(
                PageType::Rubble,
                "rubble",
                "sprites/rubble.png",
                0,
                0,
                128,
                64,
                (),
            ),
        )
        .unwrap();

        assert!(plan.has("rubble"));
        assert!(plan.has_in_page(PageType::Rubble, "rubble"));

        {
            let region = plan.page_mut(PageType::Rubble).get_mut("rubble").unwrap();
            region.x = 8;
            region.y = 16;
        }
        plan.refresh_uvs();

        let region = plan.get_in_page(PageType::Rubble, "rubble").unwrap();
        assert_f32_close(region.u, 8.0 / 4096.0);
        assert_f32_close(region.v, 16.0 / 2048.0);
        assert_f32_close(region.u2, 136.0 / 4096.0);
        assert_f32_close(region.v2, 80.0 / 2048.0);
    }
}
