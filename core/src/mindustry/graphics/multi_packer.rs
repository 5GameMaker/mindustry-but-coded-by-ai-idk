//! 纯数据化的 MultiPacker 计划层。
//!
//! 这个模块只描述“要打包什么、放到哪个 page、如何插入/替换、最终导出什么计划”，
//! 不依赖 Pixmap、Texture 或任何 GPU 资源。

use std::fmt;

/// 与 upstream `MultiPacker.PageType` 对齐的页面类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PageType {
    Main,
    Environment,
    Ui,
    Rubble,
}

impl PageType {
    /// 固定 page 顺序，和 upstream 保持一致。
    pub const ALL: [Self; 4] = [Self::Main, Self::Environment, Self::Ui, Self::Rubble];

    /// page 在 `MultiPackerPlan.pages` 中的索引。
    pub const fn index(self) -> usize {
        match self {
            Self::Main => 0,
            Self::Environment => 1,
            Self::Ui => 2,
            Self::Rubble => 3,
        }
    }

    /// page 的稳定字符串名，便于日志与测试。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Environment => "environment",
            Self::Ui => "ui",
            Self::Rubble => "rubble",
        }
    }

    /// page 的静态配置。
    pub const fn spec(self) -> PageSpec {
        match self {
            Self::Main | Self::Environment | Self::Ui => PageSpec::new(4096, 4096, 2, true),
            Self::Rubble => PageSpec::new(4096, 2048, 2, true),
        }
    }
}

impl fmt::Display for PageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// page 的静态参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageSpec {
    pub width: u32,
    pub height: u32,
    pub padding: u32,
    pub duplicate_border: bool,
}

impl PageSpec {
    pub const fn new(width: u32, height: u32, padding: u32, duplicate_border: bool) -> Self {
        Self {
            width,
            height,
            padding,
            duplicate_border,
        }
    }
}

/// 一个 region 的打包请求：只记录数据，不绑定图像资源。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegionRequest<T = ()> {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub splits: Option<[i32; 4]>,
    pub pads: Option<[i32; 4]>,
    pub payload: T,
}

impl<T> RegionRequest<T> {
    pub fn new(name: impl Into<String>, width: u32, height: u32, payload: T) -> Self {
        Self {
            name: name.into(),
            width,
            height,
            splits: None,
            pads: None,
            payload,
        }
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

    pub fn map_payload<U>(self, payload: U) -> RegionRequest<U> {
        RegionRequest {
            name: self.name,
            width: self.width,
            height: self.height,
            splits: self.splits,
            pads: self.pads,
            payload,
        }
    }
}

/// 插入失败：同名 request 已存在于该 page。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertRequestError<T> {
    pub page_type: PageType,
    pub request: RegionRequest<T>,
}

/// 单个 page 的计划。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PagePlan<T = ()> {
    pub page_type: PageType,
    pub spec: PageSpec,
    pub requests: Vec<RegionRequest<T>>,
}

impl<T> PagePlan<T> {
    pub fn new(page_type: PageType) -> Self {
        Self {
            page_type,
            spec: page_type.spec(),
            requests: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    pub fn len(&self) -> usize {
        self.requests.len()
    }

    pub fn get(&self, name: &str) -> Option<&RegionRequest<T>> {
        self.requests.iter().find(|request| request.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut RegionRequest<T>> {
        self.requests
            .iter_mut()
            .find(|request| request.name == name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn insert_request(
        &mut self,
        request: RegionRequest<T>,
    ) -> Result<(), InsertRequestError<T>> {
        if self.contains(&request.name) {
            return Err(InsertRequestError {
                page_type: self.page_type,
                request,
            });
        }
        self.requests.push(request);
        Ok(())
    }

    pub fn replace_request(&mut self, request: RegionRequest<T>) -> Option<RegionRequest<T>> {
        if let Some(index) = self
            .requests
            .iter()
            .position(|item| item.name == request.name)
        {
            Some(std::mem::replace(&mut self.requests[index], request))
        } else {
            self.requests.push(request);
            None
        }
    }

    pub fn insert_or_replace_request(
        &mut self,
        request: RegionRequest<T>,
    ) -> Option<RegionRequest<T>> {
        self.replace_request(request)
    }

    pub fn remove(&mut self, name: &str) -> Option<RegionRequest<T>> {
        let index = self
            .requests
            .iter()
            .position(|request| request.name == name)?;
        Some(self.requests.remove(index))
    }
}

impl<T> Default for PagePlan<T> {
    fn default() -> Self {
        Self::new(PageType::Main)
    }
}

/// `get(name)` 的定位结果，保留 page 信息。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocatedRegionRequest<'a, T> {
    pub page_type: PageType,
    pub request: &'a RegionRequest<T>,
}

/// 导出的 pack plan：固定 page 顺序，方便后续执行器直接消费。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackPlan<T = ()> {
    pub pages: Vec<PagePlan<T>>,
}

impl<T> PackPlan<T> {
    pub fn new(pages: Vec<PagePlan<T>>) -> Self {
        Self { pages }
    }

    pub fn page(&self, page_type: PageType) -> Option<&PagePlan<T>> {
        self.pages.iter().find(|page| page.page_type == page_type)
    }
}

/// Multi page 的计划 builder。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MultiPackerPlan<T = ()> {
    pages: [PagePlan<T>; 4],
}

impl<T> MultiPackerPlan<T> {
    pub fn new() -> Self {
        Self {
            pages: [
                PagePlan::new(PageType::Main),
                PagePlan::new(PageType::Environment),
                PagePlan::new(PageType::Ui),
                PagePlan::new(PageType::Rubble),
            ],
        }
    }

    pub fn page(&self, page_type: PageType) -> &PagePlan<T> {
        &self.pages[page_type.index()]
    }

    pub fn page_mut(&mut self, page_type: PageType) -> &mut PagePlan<T> {
        &mut self.pages[page_type.index()]
    }

    pub fn get(&self, name: &str) -> Option<LocatedRegionRequest<'_, T>> {
        self.pages.iter().find_map(|page| {
            page.get(name).map(|request| LocatedRegionRequest {
                page_type: page.page_type,
                request,
            })
        })
    }

    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn get_in_page(&self, page_type: PageType, name: &str) -> Option<&RegionRequest<T>> {
        self.page(page_type).get(name)
    }

    pub fn has_in_page(&self, page_type: PageType, name: &str) -> bool {
        self.get_in_page(page_type, name).is_some()
    }

    pub fn insert_request(
        &mut self,
        page_type: PageType,
        request: RegionRequest<T>,
    ) -> Result<(), InsertRequestError<T>> {
        self.page_mut(page_type).insert_request(request)
    }

    pub fn replace_request(
        &mut self,
        page_type: PageType,
        request: RegionRequest<T>,
    ) -> Option<RegionRequest<T>> {
        self.page_mut(page_type).replace_request(request)
    }

    pub fn insert_or_replace_request(
        &mut self,
        page_type: PageType,
        request: RegionRequest<T>,
    ) -> Option<RegionRequest<T>> {
        self.page_mut(page_type).insert_or_replace_request(request)
    }

    pub fn remove_request(&mut self, page_type: PageType, name: &str) -> Option<RegionRequest<T>> {
        self.page_mut(page_type).remove(name)
    }

    pub fn clear_page(&mut self, page_type: PageType) {
        self.page_mut(page_type).requests.clear();
    }

    pub fn into_pack_plan(self) -> PackPlan<T> {
        PackPlan::new(self.pages.into_iter().collect())
    }
}

impl<T> Default for MultiPackerPlan<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<MultiPackerPlan<T>> for PackPlan<T> {
    fn from(value: MultiPackerPlan<T>) -> Self {
        value.into_pack_plan()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_types_match_upstream_layout() {
        assert_eq!(
            PageType::ALL,
            [
                PageType::Main,
                PageType::Environment,
                PageType::Ui,
                PageType::Rubble
            ]
        );
        assert_eq!(PageType::Main.as_str(), "main");
        assert_eq!(PageType::Environment.as_str(), "environment");
        assert_eq!(PageType::Ui.as_str(), "ui");
        assert_eq!(PageType::Rubble.as_str(), "rubble");

        assert_eq!(PageType::Main.spec(), PageSpec::new(4096, 4096, 2, true));
        assert_eq!(
            PageType::Environment.spec(),
            PageSpec::new(4096, 4096, 2, true)
        );
        assert_eq!(PageType::Ui.spec(), PageSpec::new(4096, 4096, 2, true));
        assert_eq!(PageType::Rubble.spec(), PageSpec::new(4096, 2048, 2, true));
    }

    #[test]
    fn insert_replace_and_lookup_follow_page_order() {
        let mut plan = MultiPackerPlan::new();

        plan.insert_request(
            PageType::Ui,
            RegionRequest::new("whiteui", 32, 32, "ui-payload").with_splits([1, 2, 3, 4]),
        )
        .unwrap();
        plan.insert_request(
            PageType::Main,
            RegionRequest::new("white", 16, 16, "main-payload").with_pads([5, 6, 7, 8]),
        )
        .unwrap();

        assert!(plan.has("whiteui"));
        assert!(plan.has_in_page(PageType::Ui, "whiteui"));

        let located = plan.get("whiteui").unwrap();
        assert_eq!(located.page_type, PageType::Ui);
        assert_eq!(located.request.name, "whiteui");
        assert_eq!(located.request.payload, "ui-payload");
        assert_eq!(located.request.splits, Some([1, 2, 3, 4]));

        let duplicate = plan
            .insert_request(PageType::Main, RegionRequest::new("white", 8, 8, "dup"))
            .unwrap_err();
        assert_eq!(duplicate.page_type, PageType::Main);
        assert_eq!(duplicate.request.name, "white");
        assert_eq!(plan.get_in_page(PageType::Main, "white").unwrap().width, 16);

        let old = plan
            .replace_request(
                PageType::Main,
                RegionRequest::new("white", 64, 64, "replaced"),
            )
            .unwrap();
        assert_eq!(old.width, 16);
        assert_eq!(old.payload, "main-payload");
        assert_eq!(plan.get_in_page(PageType::Main, "white").unwrap().width, 64);
        assert_eq!(
            plan.get_in_page(PageType::Main, "white").unwrap().payload,
            "replaced"
        );
    }

    #[test]
    fn replace_keeps_relative_order_and_pack_plan_keeps_all_pages() {
        let mut plan = MultiPackerPlan::new();

        plan.insert_request(PageType::Main, RegionRequest::new("a", 1, 1, "A"))
            .unwrap();
        plan.insert_request(PageType::Main, RegionRequest::new("b", 2, 2, "B"))
            .unwrap();
        plan.insert_request(PageType::Rubble, RegionRequest::new("r", 3, 3, "R"))
            .unwrap();

        let old = plan
            .replace_request(PageType::Main, RegionRequest::new("a", 9, 9, "A2"))
            .unwrap();
        assert_eq!(old.width, 1);
        assert_eq!(plan.page(PageType::Main).requests[0].name, "a");
        assert_eq!(plan.page(PageType::Main).requests[1].name, "b");
        assert_eq!(plan.page(PageType::Main).requests[0].width, 9);

        let pack_plan = plan.into_pack_plan();
        assert_eq!(pack_plan.pages.len(), 4);
        assert_eq!(pack_plan.pages[0].page_type, PageType::Main);
        assert_eq!(pack_plan.pages[1].page_type, PageType::Environment);
        assert_eq!(pack_plan.pages[2].page_type, PageType::Ui);
        assert_eq!(pack_plan.pages[3].page_type, PageType::Rubble);

        assert_eq!(pack_plan.pages[0].requests.len(), 2);
        assert_eq!(pack_plan.pages[0].requests[0].name, "a");
        assert_eq!(pack_plan.pages[0].requests[0].payload, "A2");
        assert_eq!(pack_plan.pages[0].requests[1].name, "b");
        assert!(pack_plan.pages[1].requests.is_empty());
        assert!(pack_plan.pages[2].requests.is_empty());
        assert_eq!(pack_plan.pages[3].requests[0].name, "r");
        assert_eq!(pack_plan.pages[3].spec.height, 2048);
    }
}
