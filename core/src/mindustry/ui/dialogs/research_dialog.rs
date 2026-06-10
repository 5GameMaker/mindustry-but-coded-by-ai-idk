//! Research dialog model mirroring upstream `mindustry.ui.dialogs.ResearchDialog`.

use std::collections::BTreeMap;

use crate::mindustry::{
    ctype::ContentType,
    game::{ObjectiveKind, TechNode, TechNodeId, TechTree},
    r#type::Item,
    ui::{format_amount, BranchTreeLayout, TreeArena, TreeLayout, TreeLocation},
};

pub const RESEARCH_DIALOG_TITLE: &str = "";
pub const RESEARCH_DEBUG_SHOW_REQUIREMENTS: bool = false;
pub const RESEARCH_NODE_SIZE: f32 = 60.0;
pub const RESEARCH_NODE_IMAGE_SIZE: f32 = 32.0;
pub const RESEARCH_NODE_IMAGE_SCALING: &str = "Scaling.fit";
pub const RESEARCH_TREE_SPACING: f32 = 20.0;
pub const RESEARCH_INITIAL_PAN_X: f32 = 0.0;
pub const RESEARCH_INITIAL_PAN_Y: f32 = -200.0;
pub const RESEARCH_INITIAL_SCALE: f32 = 1.0;
pub const RESEARCH_MIN_SCALE: f32 = 0.25;
pub const RESEARCH_MAX_SCALE: f32 = 1.0;
pub const RESEARCH_TITLE_MIN_WIDTH: f32 = 300.0;
pub const RESEARCH_TITLE_ICON_PAD_RIGHT: f32 = 8.0;
pub const RESEARCH_TITLE_COLOR: &str = "Pal.accent";
pub const RESEARCH_SELECT_TITLE: &str = "@techtree.select";
pub const RESEARCH_SELECT_ROW_SIZE: (f32, f32) = (300.0, 60.0);
pub const RESEARCH_SELECT_ROW_STYLE: &str = "Styles.flatTogglet";
pub const RESEARCH_SELECT_ROW_ICON_SIZE: f32 = 32.0;
pub const RESEARCH_SELECT_ROW_MARGIN_LEFT: f32 = 12.0;
pub const RESEARCH_DATABASE_TEXT: &str = "@database";
pub const RESEARCH_DATABASE_ICON: &str = "book";
pub const RESEARCH_DATABASE_BUTTON_SIZE: (f32, f32) = (210.0, 64.0);
pub const RESEARCH_DATABASE_BUTTON_NAME: &str = "database";
pub const RESEARCH_ITEM_DISPLAY_PORTRAIT_MARGIN_TOP: f32 = 60.0;
pub const RESEARCH_ITEM_DISPLAY_DEFAULT_MARGIN_TOP: f32 = 0.0;
pub const RESEARCH_FALLBACK_PLANET: &str = "serpulo";
pub const RESEARCH_NODE_STYLE_UP_UNLOCKED: &str = "Tex.buttonOver";
pub const RESEARCH_NODE_STYLE_UP_LOCKED: &str = "Tex.button";
pub const RESEARCH_NODE_STYLE_UP_UNAVAILABLE: &str = "Tex.buttonRed";
pub const RESEARCH_NODE_IMAGE_LOCKED: &str = "lock";
pub const RESEARCH_NODE_IMAGE_COLOR_UNLOCKED: &str = "Color.white";
pub const RESEARCH_NODE_IMAGE_COLOR_SELECTABLE_LOCKED: &str = "Color.gray";
pub const RESEARCH_NODE_IMAGE_COLOR_UNSELECTABLE: &str = "Pal.gray";
pub const RESEARCH_LINE_COLOR_LOCKED: &str = "Pal.gray";
pub const RESEARCH_LINE_COLOR_UNLOCKED: &str = "Pal.accent";
pub const RESEARCH_LINE_Z_LOCKED: f32 = 1.0;
pub const RESEARCH_LINE_Z_UNLOCKED: f32 = 2.0;
pub const RESEARCH_LINE_STROKE: f32 = 4.0;
pub const RESEARCH_INFO_BACKGROUND: &str = "Tex.button";
pub const RESEARCH_INFO_MARGIN: f32 = 8.0;
pub const RESEARCH_INFO_BUTTON_ICON: &str = "info";
pub const RESEARCH_INFO_BUTTON_STYLE: &str = "Styles.flati";
pub const RESEARCH_INFO_BUTTON_WIDTH: f32 = 50.0;
pub const RESEARCH_INFO_DESC_PAD: f32 = 9.0;
pub const RESEARCH_LOCKED_TEXT: &str = "@locked";
pub const RESEARCH_LOCKED_COLOR: &str = "Pal.remove";
pub const RESEARCH_COMPLETE_TEXT: &str = "@complete";
pub const RESEARCH_COMPLETED_TEXT: &str = "@completed";
pub const RESEARCH_PROGRESS_KEY: &str = "research.progress";
pub const RESEARCH_PROGRESS_MAX: i32 = 99;
pub const RESEARCH_PROGRESS_COLOR: &str = "Color.lightGray";
pub const RESEARCH_PROGRESS_SHINE_COLOR: &str = "Pal.accent";
pub const RESEARCH_REQUIREMENT_TEXT_COLOR: &str = "Color.lightGray";
pub const RESEARCH_REQUIREMENT_MISSING_COLOR: &str = "Color.scarlet";
pub const RESEARCH_REQUIREMENT_ICON_SIZE: f32 = 24.0;
pub const RESEARCH_REQUIREMENT_ICON_PAD_RIGHT: f32 = 3.0;
pub const RESEARCH_OBJECTIVE_PREFIX: &str = "> ";
pub const RESEARCH_OBJECTIVE_OK_ICON: &str = "ok";
pub const RESEARCH_OBJECTIVE_CANCEL_ICON: &str = "cancel";
pub const RESEARCH_OBJECTIVE_ICON_PAD_LEFT: f32 = 3.0;
pub const RESEARCH_MOBILE_RESEARCH_TEXT: &str = "@research";
pub const RESEARCH_MOBILE_RESEARCH_ICON: &str = "ok";
pub const RESEARCH_MOBILE_RESEARCH_HEIGHT: f32 = 44.0;
pub const RESEARCH_MOBILE_RESEARCH_COLSPAN: usize = 3;
pub const RESEARCH_MOBILE_RESEARCH_DISABLED_DRAWABLE: &str = "Tex.button";
pub const RESEARCH_MOBILE_RESEARCH_UP_DRAWABLE: &str = "Tex.buttonOver";
pub const RESEARCH_MOBILE_RESEARCH_OVER_DRAWABLE: &str = "Tex.buttonDown";
pub const RESEARCH_INLINE_DESCRIPTION_COLOR: &str = "Color.lightGray";
pub const RESEARCH_INLINE_DESCRIPTION_MARGIN: f32 = 3.0;
pub const RESEARCH_INLINE_DESCRIPTION_LONG_MIN_WIDTH: f32 = 270.0;
pub const RESEARCH_INLINE_DESCRIPTION_LONG_THRESHOLD: usize = 20;

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchDialogContext {
    pub net_client: bool,
    pub mobile: bool,
    pub portrait: bool,
    pub scl: f32,
    pub debug_show_requirements: bool,
    pub planet_dialog_shown: bool,
    pub planet_dialog_tree: Option<TechNodeId>,
    pub campaign_tree: Option<TechNodeId>,
    pub item_catalog: Vec<ResearchItemInfo>,
    pub content_details: Vec<ResearchContentDetails>,
}

impl Default for ResearchDialogContext {
    fn default() -> Self {
        Self {
            net_client: false,
            mobile: false,
            portrait: false,
            scl: 1.0,
            debug_show_requirements: RESEARCH_DEBUG_SHOW_REQUIREMENTS,
            planet_dialog_shown: false,
            planet_dialog_tree: None,
            campaign_tree: None,
            item_catalog: Vec::new(),
            content_details: Vec::new(),
        }
    }
}

impl ResearchDialogContext {
    pub fn preferred_root(&self) -> Option<TechNodeId> {
        if self.planet_dialog_shown {
            self.planet_dialog_tree
        } else {
            self.campaign_tree
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchItemInfo {
    pub name: String,
    pub localized_name: String,
    pub icon: String,
    pub cost: f32,
}

impl ResearchItemInfo {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            localized_name: name.clone(),
            icon: format!("item-{name}-ui"),
            name,
            cost: 1.0,
        }
    }

    pub fn from_item(item: &Item) -> Self {
        Self {
            name: item.name().to_string(),
            localized_name: item.localized_name().to_string(),
            icon: item
                .base
                .icon_candidates(None)
                .ui_candidates
                .first()
                .cloned()
                .unwrap_or_else(|| format!("item-{}-ui", item.name())),
            cost: item.cost,
        }
    }

    pub fn from_items(items: &[Item]) -> Vec<Self> {
        items.iter().map(Self::from_item).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchContentDetails {
    pub content_type: ContentType,
    pub name: String,
    pub description: Option<String>,
    pub inline_description: bool,
}

impl ResearchContentDetails {
    pub fn new(content_type: ContentType, name: impl Into<String>) -> Self {
        Self {
            content_type,
            name: name.into(),
            description: None,
            inline_description: false,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn inline_description(mut self, inline_description: bool) -> Self {
        self.inline_description = inline_description;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResearchItemSource {
    pub planets: Vec<ResearchPlanetInventory>,
}

impl ResearchItemSource {
    pub fn from_totals<I, S>(items: I) -> Self
    where
        I: IntoIterator<Item = (S, i32)>,
        S: Into<String>,
    {
        Self {
            planets: vec![ResearchPlanetInventory {
                name: RESEARCH_FALLBACK_PLANET.into(),
                tech_tree: None,
                sectors: vec![ResearchSectorInventory {
                    sector_id: 0,
                    has_base: true,
                    frozen: false,
                    items: items
                        .into_iter()
                        .map(|(item, amount)| (item.into(), amount))
                        .collect(),
                }],
            }],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchPlanetInventory {
    pub name: String,
    pub tech_tree: Option<TechNodeId>,
    pub sectors: Vec<ResearchSectorInventory>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchSectorInventory {
    pub sector_id: i32,
    pub has_base: bool,
    pub frozen: bool,
    pub items: BTreeMap<String, i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResearchItemsModel {
    pub values: BTreeMap<String, i32>,
    pub total: i32,
    pub root_planets: Vec<String>,
    pub sector_cache: Vec<ResearchSectorItemCache>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchSectorItemCache {
    pub planet_name: String,
    pub sector_id: i32,
    pub items: BTreeMap<String, i32>,
}

impl ResearchItemsModel {
    pub fn rebuild(tree: &TechTree, root: TechNodeId, source: &ResearchItemSource) -> Self {
        let root_node = tree.node(root).expect("research root node must exist");
        let mut selected_planets = source
            .planets
            .iter()
            .filter(|planet| {
                planet.tech_tree == Some(root) || root_node.planet.as_deref() == Some(&planet.name)
            })
            .collect::<Vec<_>>();

        if selected_planets.is_empty() {
            selected_planets.extend(
                source
                    .planets
                    .iter()
                    .filter(|planet| planet.name == RESEARCH_FALLBACK_PLANET),
            );
        }

        let mut out = Self {
            root_planets: selected_planets
                .iter()
                .map(|planet| planet.name.clone())
                .collect(),
            ..Self::default()
        };

        for planet in selected_planets {
            for sector in &planet.sectors {
                if !sector.has_base || sector.frozen {
                    continue;
                }

                let mut cached = BTreeMap::new();
                for (item, amount) in &sector.items {
                    let amount = (*amount).max(0);
                    cached.insert(item.clone(), amount);
                    *out.values.entry(item.clone()).or_insert(0) += amount;
                    out.total += amount;
                }
                out.sector_cache.push(ResearchSectorItemCache {
                    planet_name: planet.name.clone(),
                    sector_id: sector.sector_id,
                    items: cached,
                });
            }
        }

        out
    }

    pub fn get(&self, item: &str) -> i32 {
        self.values.get(item).copied().unwrap_or(0)
    }

    pub fn has(&self, item: &str) -> bool {
        self.get(item) > 0
    }

    pub fn remove(&mut self, item: &str, amount: i32) -> Vec<ResearchDialogAction> {
        if amount <= 0 {
            return Vec::new();
        }

        let current = self.get(item);
        let percentage = amount as f64 / current as f64;
        let mut counter = amount;
        let mut actions = Vec::new();

        for sector in &mut self.sector_cache {
            if counter == 0 {
                break;
            }

            let sector_amount = sector.items.get(item).copied().unwrap_or(0);
            let to_remove = ((percentage * sector_amount as f64).ceil() as i32).min(counter);
            if to_remove <= 0 {
                continue;
            }

            *sector.items.entry(item.to_string()).or_insert(0) -= to_remove;
            counter -= to_remove;
            actions.push(ResearchDialogAction::RemoveSectorItem {
                planet_name: sector.planet_name.clone(),
                sector_id: sector.sector_id,
                item: item.to_string(),
                amount: to_remove,
            });
        }

        *self.values.entry(item.to_string()).or_insert(0) -= amount;
        self.total -= amount;
        actions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResearchDialogKey {
    Research,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResearchDialogAction {
    ShowDialog,
    HideDialog,
    PostHideDialog,
    HideTechSelectDialog,
    ShowDatabase,
    CheckMargin,
    PostCheckMargin,
    RebuildItems,
    RebuildAll,
    RebuildInfo {
        shine: Vec<bool>,
    },
    RebuildItemDisplay {
        used_shine_items: Vec<String>,
    },
    TreeLayout,
    ClearHover,
    ClearInfo,
    SwitchTree {
        root: TechNodeId,
    },
    SetPan {
        pan_x: f32,
        pan_y: f32,
    },
    SetScale {
        scale: f32,
    },
    SaveNode {
        node: TechNodeId,
    },
    UnlockContent {
        node: TechNodeId,
        content_name: String,
    },
    RemoveSectorItem {
        planet_name: String,
        sector_id: i32,
        item: String,
        amount: i32,
    },
    ShowContent {
        node: TechNodeId,
        content_type: ContentType,
        content_name: String,
    },
    SceneAct,
    PlayUnlockSound,
    FireResearchEvent {
        content_type: ContentType,
        content_name: String,
    },
    PlanetSetup,
    DelayedClientUnlockRebuild,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchDialogModel {
    pub title: &'static str,
    pub should_pause: bool,
    pub close_button_added: bool,
    pub node_size: f32,
    pub title_button: ResearchTitleButtonModel,
    pub bottom_button: ResearchBottomButtonModel,
    pub item_display: ResearchItemDisplayModel,
    pub view: ResearchViewModel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchTitleButtonModel {
    pub visible: bool,
    pub min_width: f32,
    pub icon: String,
    pub icon_pad_right: f32,
    pub icon_size: f32,
    pub label: String,
    pub label_color: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchBottomButtonModel {
    pub text: &'static str,
    pub icon: &'static str,
    pub size: (f32, f32),
    pub name: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchRootPickerModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub rows: Vec<ResearchRootPickerRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchRootPickerRow {
    pub node: TechNodeId,
    pub text: String,
    pub icon: String,
    pub style: &'static str,
    pub icon_size: f32,
    pub size: (f32, f32),
    pub margin_left: f32,
    pub checked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchItemDisplayModel {
    pub visible: bool,
    pub margin_top: f32,
    pub rows: Vec<ResearchItemDisplayRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchItemDisplayRow {
    pub item: String,
    pub localized_name: String,
    pub icon: String,
    pub amount: i32,
    pub amount_label: String,
    pub shine: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchViewModel {
    pub pan_x: f32,
    pub pan_y: f32,
    pub scale: f32,
    pub last_zoom: f32,
    pub moved: bool,
    pub bounds: ResearchBounds,
    pub nodes: Vec<ResearchTreeNodeModel>,
    pub links: Vec<ResearchLinkModel>,
    pub hover_node: Option<TechNodeId>,
    pub info_panel: Option<ResearchInfoPanelModel>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResearchBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for ResearchBounds {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchTreeNodeModel {
    pub id: TechNodeId,
    pub parent: Option<TechNodeId>,
    pub children: Vec<TechNodeId>,
    pub content_type: ContentType,
    pub content_name: String,
    pub localized_name: String,
    pub visible: bool,
    pub selectable: bool,
    pub locked: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub button: ResearchNodeButtonModel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchNodeButtonModel {
    pub style_up: &'static str,
    pub icon: String,
    pub image_color: &'static str,
    pub image_size: f32,
    pub image_scaling: &'static str,
    pub disabled: bool,
    pub touchable: bool,
    pub size: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchLinkModel {
    pub parent: TechNodeId,
    pub child: TechNodeId,
    pub color: &'static str,
    pub z: f32,
    pub stroke: f32,
    pub diagonal: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchInfoPanelModel {
    pub node: TechNodeId,
    pub background: &'static str,
    pub margin: f32,
    pub selectable: bool,
    pub title: String,
    pub info_button: Option<ResearchInfoButtonModel>,
    pub body: ResearchInfoBody,
    pub mobile_research_button: Option<ResearchMobileResearchButton>,
    pub inline_description: Option<ResearchInlineDescription>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchInfoButtonModel {
    pub icon: &'static str,
    pub style: &'static str,
    pub width: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResearchInfoBody {
    LockedClient {
        text: &'static str,
        color: &'static str,
    },
    Requirements {
        progress: Option<ResearchProgressModel>,
        rows: Vec<ResearchRequirementModel>,
    },
    Objectives {
        title: &'static str,
        rows: Vec<ResearchObjectiveModel>,
    },
    Completed {
        text: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchProgressModel {
    pub bundle_key: &'static str,
    pub percent: i32,
    pub color: &'static str,
    pub shine: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchRequirementModel {
    pub item: String,
    pub localized_name: String,
    pub icon: String,
    pub icon_size: f32,
    pub icon_pad_right: f32,
    pub required_amount: i32,
    pub available_amount: i32,
    pub amount_label: String,
    pub amount_color: &'static str,
    pub shine: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchObjectiveModel {
    pub text: String,
    pub icon: &'static str,
    pub color: &'static str,
    pub icon_pad_left: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchMobileResearchButton {
    pub text: &'static str,
    pub icon: &'static str,
    pub disabled: bool,
    pub height: f32,
    pub colspan: usize,
    pub disabled_drawable: &'static str,
    pub up_drawable: &'static str,
    pub over_drawable: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchInlineDescription {
    pub text: String,
    pub color: &'static str,
    pub margin: f32,
    pub min_width: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResearchDialog {
    pub root: Option<TechNodeId>,
    pub last_node: Option<TechNodeId>,
    pub nodes: BTreeMap<TechNodeId, ResearchTreeNodeModel>,
    pub items: ResearchItemsModel,
    pub bounds: ResearchBounds,
    pub pan_x: f32,
    pub pan_y: f32,
    pub scale: f32,
    pub last_zoom: f32,
    pub moved: bool,
    pub hover_node: Option<TechNodeId>,
    pub show_tech_select: bool,
    pub needs_rebuild: bool,
}

impl Default for ResearchDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ResearchDialog {
    pub fn new() -> Self {
        Self {
            root: None,
            last_node: None,
            nodes: BTreeMap::new(),
            items: ResearchItemsModel::default(),
            bounds: ResearchBounds::default(),
            pan_x: RESEARCH_INITIAL_PAN_X,
            pan_y: RESEARCH_INITIAL_PAN_Y,
            scale: RESEARCH_INITIAL_SCALE,
            last_zoom: -1.0,
            moved: false,
            hover_node: None,
            show_tech_select: false,
            needs_rebuild: false,
        }
    }

    pub fn shown_plan(
        &mut self,
        tree: &TechTree,
        item_source: &ResearchItemSource,
        context: &ResearchDialogContext,
    ) -> Vec<ResearchDialogAction> {
        self.ensure_root(tree, item_source);
        let mut actions = vec![
            ResearchDialogAction::CheckMargin,
            ResearchDialogAction::PostCheckMargin,
        ];

        if let Some(preferred) = context.preferred_root() {
            actions.extend(self.switch_tree_plan(tree, preferred, item_source));
        }

        self.rebuild_items(tree, item_source);
        actions.push(ResearchDialogAction::RebuildItems);
        self.check_nodes(tree);
        actions.push(ResearchDialogAction::RebuildItemDisplay {
            used_shine_items: Vec::new(),
        });
        self.tree_layout(tree, context);
        actions.push(ResearchDialogAction::TreeLayout);
        self.hover_node = None;
        actions.push(ResearchDialogAction::ClearHover);
        actions.push(ResearchDialogAction::ClearInfo);
        actions
    }

    pub fn show(
        &mut self,
        tree: &TechTree,
        item_source: &ResearchItemSource,
        context: &ResearchDialogContext,
    ) -> (Vec<ResearchDialogAction>, ResearchDialogModel) {
        let mut actions = self.shown_plan(tree, item_source, context);
        actions.push(ResearchDialogAction::ShowDialog);
        (actions, self.model(tree, context))
    }

    pub fn model(
        &mut self,
        tree: &TechTree,
        context: &ResearchDialogContext,
    ) -> ResearchDialogModel {
        self.ensure_root(tree, &ResearchItemSource::default());
        self.show_tech_select = self.can_select_root_like_java(tree);
        let root = self.root_id();
        let root_node = tree.node(root).expect("research root node must exist");
        ResearchDialogModel {
            title: RESEARCH_DIALOG_TITLE,
            should_pause: true,
            close_button_added: true,
            node_size: self.node_size(context),
            title_button: ResearchTitleButtonModel {
                visible: self.show_tech_select,
                min_width: RESEARCH_TITLE_MIN_WIDTH,
                icon: node_icon(root_node),
                icon_pad_right: RESEARCH_TITLE_ICON_PAD_RIGHT,
                icon_size: 32.0 * context.scl,
                label: root_node.localized_name().to_string(),
                label_color: RESEARCH_TITLE_COLOR,
            },
            bottom_button: ResearchBottomButtonModel {
                text: RESEARCH_DATABASE_TEXT,
                icon: RESEARCH_DATABASE_ICON,
                size: RESEARCH_DATABASE_BUTTON_SIZE,
                name: RESEARCH_DATABASE_BUTTON_NAME,
            },
            item_display: self.item_display_model(context, &[]),
            view: self.view_model(tree, context, &[]),
        }
    }

    pub fn model_with_shine(
        &mut self,
        tree: &TechTree,
        context: &ResearchDialogContext,
        used_shine_items: &[String],
        requirement_shine: &[bool],
    ) -> ResearchDialogModel {
        self.ensure_root(tree, &ResearchItemSource::default());
        self.show_tech_select = self.can_select_root_like_java(tree);
        let root = self.root_id();
        let root_node = tree.node(root).expect("research root node must exist");
        ResearchDialogModel {
            title: RESEARCH_DIALOG_TITLE,
            should_pause: true,
            close_button_added: true,
            node_size: self.node_size(context),
            title_button: ResearchTitleButtonModel {
                visible: self.show_tech_select,
                min_width: RESEARCH_TITLE_MIN_WIDTH,
                icon: node_icon(root_node),
                icon_pad_right: RESEARCH_TITLE_ICON_PAD_RIGHT,
                icon_size: 32.0 * context.scl,
                label: root_node.localized_name().to_string(),
                label_color: RESEARCH_TITLE_COLOR,
            },
            bottom_button: ResearchBottomButtonModel {
                text: RESEARCH_DATABASE_TEXT,
                icon: RESEARCH_DATABASE_ICON,
                size: RESEARCH_DATABASE_BUTTON_SIZE,
                name: RESEARCH_DATABASE_BUTTON_NAME,
            },
            item_display: self.item_display_model(context, used_shine_items),
            view: self.view_model(tree, context, requirement_shine),
        }
    }

    pub fn root_picker_model(
        &mut self,
        tree: &TechTree,
        context: &ResearchDialogContext,
    ) -> ResearchRootPickerModel {
        self.ensure_root(tree, &ResearchItemSource::default());
        let preferred = context.preferred_root();
        let rows = tree
            .roots()
            .iter()
            .copied()
            .filter_map(|id| {
                let node = tree.node(id).expect("research root node must exist");
                if node.requires_unlock && !node.content.unlocked_host && Some(id) != preferred {
                    return None;
                }
                Some(ResearchRootPickerRow {
                    node: id,
                    text: node.localized_name().to_string(),
                    icon: node_icon(node),
                    style: RESEARCH_SELECT_ROW_STYLE,
                    icon_size: RESEARCH_SELECT_ROW_ICON_SIZE,
                    size: RESEARCH_SELECT_ROW_SIZE,
                    margin_left: RESEARCH_SELECT_ROW_MARGIN_LEFT,
                    checked: Some(id) == self.last_node,
                })
            })
            .collect();
        ResearchRootPickerModel {
            title: RESEARCH_SELECT_TITLE,
            close_button_added: true,
            rows,
        }
    }

    pub fn select_root_plan(
        &mut self,
        tree: &TechTree,
        root: TechNodeId,
        item_source: &ResearchItemSource,
        context: &ResearchDialogContext,
    ) -> Vec<ResearchDialogAction> {
        if Some(root) == self.last_node {
            return Vec::new();
        }

        let mut actions = self.rebuild_tree_plan(tree, root, item_source, context);
        actions.push(ResearchDialogAction::HideTechSelectDialog);
        actions
    }

    pub fn switch_tree_plan(
        &mut self,
        tree: &TechTree,
        root: TechNodeId,
        item_source: &ResearchItemSource,
    ) -> Vec<ResearchDialogAction> {
        if Some(root) == self.last_node {
            return Vec::new();
        }

        self.nodes.clear();
        self.root = Some(root);
        self.last_node = Some(root);
        self.rebuild_node_views(tree);
        self.rebuild_items(tree, item_source);
        vec![
            ResearchDialogAction::SwitchTree { root },
            ResearchDialogAction::RebuildAll,
            ResearchDialogAction::RebuildItems,
        ]
    }

    pub fn rebuild_tree_plan(
        &mut self,
        tree: &TechTree,
        root: TechNodeId,
        item_source: &ResearchItemSource,
        context: &ResearchDialogContext,
    ) -> Vec<ResearchDialogAction> {
        let mut actions = self.switch_tree_plan(tree, root, item_source);
        self.pan_x = RESEARCH_INITIAL_PAN_X;
        self.pan_y = RESEARCH_INITIAL_PAN_Y;
        self.scale = RESEARCH_INITIAL_SCALE;
        self.hover_node = None;
        self.check_nodes(tree);
        self.tree_layout(tree, context);
        actions.extend([
            ResearchDialogAction::SetPan {
                pan_x: RESEARCH_INITIAL_PAN_X,
                pan_y: RESEARCH_INITIAL_PAN_Y,
            },
            ResearchDialogAction::SetScale {
                scale: RESEARCH_INITIAL_SCALE,
            },
            ResearchDialogAction::ClearHover,
            ResearchDialogAction::ClearInfo,
            ResearchDialogAction::TreeLayout,
        ]);
        actions
    }

    pub fn reset_event_plan() -> Vec<ResearchDialogAction> {
        vec![ResearchDialogAction::HideDialog]
    }

    pub fn hidden_plan() -> Vec<ResearchDialogAction> {
        vec![ResearchDialogAction::PlanetSetup]
    }

    pub fn key_down_plan(key: ResearchDialogKey) -> Vec<ResearchDialogAction> {
        match key {
            ResearchDialogKey::Research => vec![ResearchDialogAction::PostHideDialog],
            ResearchDialogKey::Other => Vec::new(),
        }
    }

    pub fn database_button_plan() -> Vec<ResearchDialogAction> {
        vec![
            ResearchDialogAction::HideDialog,
            ResearchDialogAction::ShowDatabase,
        ]
    }

    pub fn unlock_event_plan(
        &mut self,
        context: &ResearchDialogContext,
    ) -> Vec<ResearchDialogAction> {
        if context.net_client && !self.needs_rebuild {
            self.needs_rebuild = true;
            vec![ResearchDialogAction::DelayedClientUnlockRebuild]
        } else {
            Vec::new()
        }
    }

    pub fn finish_delayed_unlock_rebuild_plan(
        &mut self,
        tree: &TechTree,
        context: &ResearchDialogContext,
    ) -> Vec<ResearchDialogAction> {
        self.needs_rebuild = false;
        self.check_nodes(tree);
        self.hover_node = None;
        self.tree_layout(tree, context);
        vec![
            ResearchDialogAction::ClearHover,
            ResearchDialogAction::TreeLayout,
            ResearchDialogAction::RebuildInfo { shine: Vec::new() },
            ResearchDialogAction::SceneAct,
        ]
    }

    pub fn node_click_plan(
        &mut self,
        tree: &mut TechTree,
        node: TechNodeId,
        context: &ResearchDialogContext,
    ) -> Vec<ResearchDialogAction> {
        if context.mobile {
            self.hover_node = Some(node);
            return vec![ResearchDialogAction::RebuildInfo { shine: Vec::new() }];
        }

        let spend = {
            let tech_node = tree.node(node).expect("research node must exist");
            locked(tech_node) && can_spend(tree, node, &self.items, context)
        };
        if spend {
            self.spend_plan(tree, node, context)
        } else {
            self.hover_node = Some(node);
            vec![ResearchDialogAction::RebuildInfo { shine: Vec::new() }]
        }
    }

    pub fn hover_plan(
        &mut self,
        node: Option<TechNodeId>,
        context: &ResearchDialogContext,
    ) -> Vec<ResearchDialogAction> {
        if context.mobile {
            return Vec::new();
        }
        if self.hover_node != node {
            self.hover_node = node;
            vec![ResearchDialogAction::RebuildInfo { shine: Vec::new() }]
        } else {
            Vec::new()
        }
    }

    pub fn open_content_plan(
        &self,
        tree: &TechTree,
        node: TechNodeId,
    ) -> Vec<ResearchDialogAction> {
        let node = tree.node(node).expect("research node must exist");
        vec![ResearchDialogAction::ShowContent {
            node: node.id,
            content_type: node.content.content_type,
            content_name: node.content.name.clone(),
        }]
    }

    pub fn spend_plan(
        &mut self,
        tree: &mut TechTree,
        node: TechNodeId,
        context: &ResearchDialogContext,
    ) -> Vec<ResearchDialogAction> {
        if context.net_client {
            return Vec::new();
        }

        let mut complete = true;
        let mut shine = Vec::new();
        let mut used_shine_items = Vec::new();
        let mut actions = Vec::new();

        {
            let tech_node = tree.node_mut(node).expect("research node must exist");
            shine.resize(tech_node.requirements.len(), false);
            for (index, req) in tech_node.requirements.iter().cloned().enumerate() {
                let completed = &mut tech_node.finished_requirements[index];
                let used = (req.amount - completed.amount)
                    .min(self.items.get(&req.item))
                    .max(0);
                actions.extend(self.items.remove(&req.item, used));
                completed.amount += used;

                if used > 0 {
                    shine[index] = true;
                    used_shine_items.push(req.item.clone());
                }

                if completed.amount < req.amount {
                    complete = false;
                }
            }
        }

        if complete {
            actions.extend(self.unlock_plan(tree, node, context));
        }

        tree.save_node(node);
        actions.extend([
            ResearchDialogAction::SaveNode { node },
            ResearchDialogAction::SceneAct,
            ResearchDialogAction::RebuildInfo {
                shine: shine.clone(),
            },
            ResearchDialogAction::RebuildItemDisplay { used_shine_items },
            ResearchDialogAction::CheckMargin,
        ]);
        actions
    }

    pub fn scroll_plan(&mut self, amount_y: f32) -> Vec<ResearchDialogAction> {
        self.scale = (self.scale - amount_y / 10.0 * self.scale)
            .clamp(RESEARCH_MIN_SCALE, RESEARCH_MAX_SCALE);
        vec![ResearchDialogAction::SetScale { scale: self.scale }]
    }

    pub fn zoom_plan(&mut self, initial_distance: f32, distance: f32) -> Vec<ResearchDialogAction> {
        if self.last_zoom < 0.0 {
            self.last_zoom = self.scale;
        }
        self.scale = (distance / initial_distance * self.last_zoom)
            .clamp(RESEARCH_MIN_SCALE, RESEARCH_MAX_SCALE);
        vec![ResearchDialogAction::SetScale { scale: self.scale }]
    }

    pub fn zoom_touch_up_plan(&mut self) -> Vec<ResearchDialogAction> {
        self.last_zoom = self.scale;
        Vec::new()
    }

    pub fn pan_plan(
        &mut self,
        delta_x: f32,
        delta_y: f32,
        graphics_width: f32,
        graphics_height: f32,
        context: &ResearchDialogContext,
    ) -> Vec<ResearchDialogAction> {
        self.pan_x += delta_x / self.scale;
        self.pan_y += delta_y / self.scale;
        self.moved = true;
        self.clamp_pan(graphics_width, graphics_height, context);
        vec![ResearchDialogAction::SetPan {
            pan_x: self.pan_x,
            pan_y: self.pan_y,
        }]
    }

    pub fn release_plan(&mut self) -> Vec<ResearchDialogAction> {
        self.moved = false;
        Vec::new()
    }

    fn unlock_plan(
        &mut self,
        tree: &mut TechTree,
        node: TechNodeId,
        context: &ResearchDialogContext,
    ) -> Vec<ResearchDialogAction> {
        let mut actions = Vec::new();
        let mut parent = Some(node);
        let mut event_content = None;

        while let Some(id) = parent {
            let tech_node = tree.node_mut(id).expect("research node must exist");
            tech_node.content.unlocked_host = true;
            if id == node {
                event_content = Some((
                    tech_node.content.content_type,
                    tech_node.content.name.clone(),
                ));
            }
            actions.push(ResearchDialogAction::UnlockContent {
                node: id,
                content_name: tech_node.content.name.clone(),
            });
            parent = tech_node.parent;
        }

        self.check_nodes(tree);
        self.hover_node = None;
        self.tree_layout(tree, context);
        actions.extend([
            ResearchDialogAction::ClearHover,
            ResearchDialogAction::TreeLayout,
            ResearchDialogAction::RebuildInfo { shine: Vec::new() },
            ResearchDialogAction::SceneAct,
            ResearchDialogAction::PlayUnlockSound,
        ]);
        if let Some((content_type, content_name)) = event_content {
            actions.push(ResearchDialogAction::FireResearchEvent {
                content_type,
                content_name,
            });
        }
        actions
    }

    fn ensure_root(&mut self, tree: &TechTree, item_source: &ResearchItemSource) {
        if self.root.is_none() {
            let root = *tree
                .roots()
                .first()
                .expect("research dialog requires at least one tech tree root");
            self.root = Some(root);
            self.last_node = Some(root);
            self.rebuild_node_views(tree);
            self.rebuild_items(tree, item_source);
        }
    }

    fn rebuild_items(&mut self, tree: &TechTree, item_source: &ResearchItemSource) {
        self.items = ResearchItemsModel::rebuild(tree, self.root_id(), item_source);
    }

    fn rebuild_node_views(&mut self, tree: &TechTree) {
        self.nodes.clear();
        let node_size = RESEARCH_NODE_SIZE;
        for id in tree.each_from(self.root_id()) {
            let node = tree.node(id).expect("research node must exist");
            self.nodes.insert(
                id,
                ResearchTreeNodeModel {
                    id,
                    parent: node.parent,
                    children: node.children.clone(),
                    content_type: node.content.content_type,
                    content_name: node.content.name.clone(),
                    localized_name: node.localized_name().to_string(),
                    visible: true,
                    selectable: true,
                    locked: locked(node),
                    x: 0.0,
                    y: 0.0,
                    width: node_size,
                    height: node_size,
                    button: ResearchNodeButtonModel {
                        style_up: RESEARCH_NODE_STYLE_UP_LOCKED,
                        icon: node_content_icon(node),
                        image_color: RESEARCH_NODE_IMAGE_COLOR_UNLOCKED,
                        image_size: RESEARCH_NODE_IMAGE_SIZE,
                        image_scaling: RESEARCH_NODE_IMAGE_SCALING,
                        disabled: false,
                        touchable: true,
                        size: node_size,
                    },
                },
            );
        }
    }

    fn check_nodes(&mut self, tree: &TechTree) {
        self.check_node(tree, self.root_id());
    }

    fn check_node(&mut self, tree: &TechTree, node_id: TechNodeId) {
        let node = tree.node(node_id).expect("research node must exist");
        let is_locked = locked(node);
        let parent_visible = node
            .parent
            .and_then(|parent| self.nodes.get(&parent).map(|model| model.visible))
            .unwrap_or(true);
        if !is_locked && parent_visible {
            self.nodes
                .get_mut(&node_id)
                .expect("node model must exist")
                .visible = true;
        }

        let selectable = selectable(node);
        {
            let model = self.nodes.get_mut(&node_id).expect("node model must exist");
            model.selectable = selectable;
            model.locked = is_locked;
        }

        let visible = self
            .nodes
            .get(&node_id)
            .expect("node model must exist")
            .visible;
        for child in node.children.iter().copied() {
            if let Some(model) = self.nodes.get_mut(&child) {
                model.visible = !is_locked && visible;
            }
            self.check_node(tree, child);
        }
    }

    fn tree_layout(&mut self, tree: &TechTree, context: &ResearchDialogContext) {
        let root = self.root_id();
        let root_node = tree.node(root).expect("research root node must exist");
        let children = root_node.children.clone();
        let split = (children.len() + 1) / 2;
        let left = &children[..split];
        let right = &children[split..];

        let mut coords = BTreeMap::new();
        let (left_coords, left_root_y) =
            layout_research_half(tree, root, left, TreeLocation::Top, self.node_size(context));
        if right.is_empty() {
            coords.extend(left_coords);
        } else {
            let (right_coords, right_root_y) = layout_research_half(
                tree,
                root,
                right,
                TreeLocation::Bottom,
                self.node_size(context),
            );
            let shift = right_root_y - left_root_y;
            for (id, (x, y)) in left_coords {
                coords.insert(id, (x, y + shift));
            }
            coords.extend(right_coords);
        }

        for (id, (x, y)) in coords {
            let node_size = self.node_size(context);
            if let Some(model) = self.nodes.get_mut(&id) {
                model.x = x;
                model.y = y;
                model.width = node_size;
                model.height = node_size;
            }
        }

        let mut min_x = 0.0_f32;
        let mut min_y = 0.0_f32;
        let mut max_x = 0.0_f32;
        let mut max_y = 0.0_f32;
        for node in self.nodes.values().filter(|node| node.visible) {
            min_x = min_x.min(node.x - node.width / 2.0);
            max_x = max_x.max(node.x + node.width / 2.0);
            min_y = min_y.min(node.y - node.height / 2.0);
            max_y = max_y.max(node.y + node.height / 2.0);
        }
        self.bounds = ResearchBounds {
            x: min_x,
            y: min_y + self.node_size(context) * 1.5,
            width: max_x - min_x,
            height: max_y - min_y,
        };
    }

    fn can_select_root_like_java(&self, tree: &TechTree) -> bool {
        tree.roots()
            .iter()
            .copied()
            .filter(|&id| {
                let node = tree.node(id).expect("research root node must exist");
                !(node.requires_unlock && !node.content.unlocked_host)
            })
            .count()
            > 1
    }

    fn item_display_model(
        &self,
        context: &ResearchDialogContext,
        used_shine_items: &[String],
    ) -> ResearchItemDisplayModel {
        ResearchItemDisplayModel {
            visible: !context.net_client,
            margin_top: if context.portrait && self.show_tech_select {
                RESEARCH_ITEM_DISPLAY_PORTRAIT_MARGIN_TOP
            } else {
                RESEARCH_ITEM_DISPLAY_DEFAULT_MARGIN_TOP
            },
            rows: context
                .item_catalog
                .iter()
                .filter_map(|item| {
                    let amount = self.items.get(&item.name);
                    (amount > 0).then(|| ResearchItemDisplayRow {
                        item: item.name.clone(),
                        localized_name: item.localized_name.clone(),
                        icon: item.icon.clone(),
                        amount,
                        amount_label: format_amount(amount as i64),
                        shine: used_shine_items.iter().any(|used| used == &item.name),
                    })
                })
                .collect(),
        }
    }

    fn view_model(
        &self,
        tree: &TechTree,
        context: &ResearchDialogContext,
        requirement_shine: &[bool],
    ) -> ResearchViewModel {
        ResearchViewModel {
            pan_x: self.pan_x,
            pan_y: self.pan_y,
            scale: self.scale,
            last_zoom: self.last_zoom,
            moved: self.moved,
            bounds: self.bounds,
            nodes: self
                .nodes
                .values()
                .map(|node| self.node_model_with_button(tree, node.id, context))
                .collect(),
            links: self.link_models(tree, context),
            hover_node: self.hover_node,
            info_panel: self
                .hover_node
                .map(|node| self.info_panel_model(tree, node, context, requirement_shine)),
        }
    }

    fn node_model_with_button(
        &self,
        tree: &TechTree,
        node_id: TechNodeId,
        context: &ResearchDialogContext,
    ) -> ResearchTreeNodeModel {
        let mut model = self
            .nodes
            .get(&node_id)
            .expect("node model must exist")
            .clone();
        let node = tree.node(node_id).expect("research node must exist");
        let can_spend = can_spend(tree, node_id, &self.items, context);
        model.button = ResearchNodeButtonModel {
            style_up: if !model.locked {
                RESEARCH_NODE_STYLE_UP_UNLOCKED
            } else if !model.selectable || (!can_spend && !context.net_client) {
                RESEARCH_NODE_STYLE_UP_UNAVAILABLE
            } else {
                RESEARCH_NODE_STYLE_UP_LOCKED
            },
            icon: if model.selectable {
                node_content_icon(node)
            } else {
                RESEARCH_NODE_IMAGE_LOCKED.to_string()
            },
            image_color: if !model.locked {
                RESEARCH_NODE_IMAGE_COLOR_UNLOCKED
            } else if model.selectable {
                RESEARCH_NODE_IMAGE_COLOR_SELECTABLE_LOCKED
            } else {
                RESEARCH_NODE_IMAGE_COLOR_UNSELECTABLE
            },
            image_size: RESEARCH_NODE_IMAGE_SIZE,
            image_scaling: RESEARCH_NODE_IMAGE_SCALING,
            disabled: context.net_client && !context.mobile,
            touchable: model.visible,
            size: self.node_size(context),
        };
        model
    }

    fn link_models(
        &self,
        tree: &TechTree,
        context: &ResearchDialogContext,
    ) -> Vec<ResearchLinkModel> {
        let mut links = Vec::new();
        for node in self.nodes.values().filter(|node| node.visible) {
            let parent = tree.node(node.id).expect("research node must exist");
            for &child in &node.children {
                let Some(child_model) = self.nodes.get(&child) else {
                    continue;
                };
                if !child_model.visible {
                    continue;
                }
                let child_node = tree.node(child).expect("research child node must exist");
                let lock = locked(parent) || locked(child_node);
                let dx = (node.x - child_model.x).abs();
                let dy = (node.y - child_model.y).abs();
                let distance = (dx * dx + dy * dy).sqrt();
                links.push(ResearchLinkModel {
                    parent: node.id,
                    child,
                    color: if lock {
                        RESEARCH_LINE_COLOR_LOCKED
                    } else {
                        RESEARCH_LINE_COLOR_UNLOCKED
                    },
                    z: if lock {
                        RESEARCH_LINE_Z_LOCKED
                    } else {
                        RESEARCH_LINE_Z_UNLOCKED
                    },
                    stroke: RESEARCH_LINE_STROKE * context.scl,
                    diagonal: (dy - dx).abs() <= 1.0 && distance <= node.width * 3.0,
                });
            }
        }
        links
    }

    fn info_panel_model(
        &self,
        tree: &TechTree,
        node_id: TechNodeId,
        context: &ResearchDialogContext,
        requirement_shine: &[bool],
    ) -> ResearchInfoPanelModel {
        let node = tree.node(node_id).expect("research node must exist");
        let selectable = selectable(node);
        let is_locked = locked(node);
        let body = if is_locked || (context.debug_show_requirements && !context.net_client) {
            if context.net_client {
                ResearchInfoBody::LockedClient {
                    text: RESEARCH_LOCKED_TEXT,
                    color: RESEARCH_LOCKED_COLOR,
                }
            } else if selectable {
                ResearchInfoBody::Requirements {
                    progress: progress_model(node, context, requirement_shine),
                    rows: requirement_rows(node, &self.items, context, requirement_shine),
                }
            } else if !node.objectives.is_empty() {
                ResearchInfoBody::Objectives {
                    title: RESEARCH_COMPLETE_TEXT,
                    rows: objective_rows(&node.objectives),
                }
            } else {
                ResearchInfoBody::Requirements {
                    progress: None,
                    rows: Vec::new(),
                }
            }
        } else {
            ResearchInfoBody::Completed {
                text: RESEARCH_COMPLETED_TEXT,
            }
        };

        ResearchInfoPanelModel {
            node: node_id,
            background: RESEARCH_INFO_BACKGROUND,
            margin: RESEARCH_INFO_MARGIN,
            selectable,
            title: if selectable {
                node.localized_name().to_string()
            } else {
                "[accent]???".to_string()
            },
            info_button: selectable.then_some(ResearchInfoButtonModel {
                icon: RESEARCH_INFO_BUTTON_ICON,
                style: RESEARCH_INFO_BUTTON_STYLE,
                width: RESEARCH_INFO_BUTTON_WIDTH,
            }),
            body,
            mobile_research_button: (context.mobile && is_locked && !context.net_client).then(
                || ResearchMobileResearchButton {
                    text: RESEARCH_MOBILE_RESEARCH_TEXT,
                    icon: RESEARCH_MOBILE_RESEARCH_ICON,
                    disabled: !can_spend(tree, node_id, &self.items, context),
                    height: RESEARCH_MOBILE_RESEARCH_HEIGHT,
                    colspan: RESEARCH_MOBILE_RESEARCH_COLSPAN,
                    disabled_drawable: RESEARCH_MOBILE_RESEARCH_DISABLED_DRAWABLE,
                    up_drawable: RESEARCH_MOBILE_RESEARCH_UP_DRAWABLE,
                    over_drawable: RESEARCH_MOBILE_RESEARCH_OVER_DRAWABLE,
                },
            ),
            inline_description: inline_description(node, context),
        }
    }

    fn clamp_pan(
        &mut self,
        graphics_width: f32,
        graphics_height: f32,
        context: &ResearchDialogContext,
    ) {
        let pad = self.node_size(context);
        let ox = graphics_width / 2.0;
        let oy = graphics_height / 2.0;
        let rx =
            (self.bounds.x + self.pan_x + ox).clamp(-self.bounds.width + pad, graphics_width - pad);
        let ry = (self.pan_y + oy + self.bounds.y)
            .clamp(-self.bounds.height + pad, graphics_height - pad);
        self.pan_x = rx - self.bounds.x - ox;
        self.pan_y = ry - self.bounds.y - oy;
    }

    fn root_id(&self) -> TechNodeId {
        self.root.expect("research root must be initialized")
    }

    fn node_size(&self, context: &ResearchDialogContext) -> f32 {
        RESEARCH_NODE_SIZE * context.scl
    }
}

pub fn selectable(node: &TechNode) -> bool {
    node.content.unlocked_host || node.objectives.iter().all(ObjectiveKind::complete)
}

pub fn locked(node: &TechNode) -> bool {
    !node.content.unlocked_host
}

pub fn can_spend(
    tree: &TechTree,
    node: TechNodeId,
    items: &ResearchItemsModel,
    context: &ResearchDialogContext,
) -> bool {
    let node = tree.node(node).expect("research node must exist");
    if !selectable(node) || context.net_client {
        return false;
    }

    if node.requirements.is_empty() {
        return true;
    }

    for (index, req) in node.requirements.iter().enumerate() {
        let finished = node.finished_requirements[index].amount;
        if finished < req.amount && items.has(&req.item) {
            return true;
        }
    }

    locked(node)
}

fn progress_model(
    node: &TechNode,
    context: &ResearchDialogContext,
    shine: &[bool],
) -> Option<ResearchProgressModel> {
    if node.requirements.is_empty()
        || !node
            .finished_requirements
            .iter()
            .any(|stack| stack.amount > 0)
    {
        return None;
    }

    let mut sum = 0.0;
    let mut used = 0.0;
    let mut shiny = false;
    for (index, req) in node.requirements.iter().enumerate() {
        let cost = item_cost(&context.item_catalog, &req.item);
        sum += cost * req.amount as f32;
        used += cost * node.finished_requirements[index].amount as f32;
        shiny |= shine.get(index).copied().unwrap_or(false);
    }

    if sum <= 0.0 {
        return None;
    }

    Some(ResearchProgressModel {
        bundle_key: RESEARCH_PROGRESS_KEY,
        percent: (((used / sum) * 100.0) as i32).min(RESEARCH_PROGRESS_MAX),
        color: if shiny {
            RESEARCH_PROGRESS_SHINE_COLOR
        } else {
            RESEARCH_PROGRESS_COLOR
        },
        shine: shiny,
    })
}

fn requirement_rows(
    node: &TechNode,
    items: &ResearchItemsModel,
    context: &ResearchDialogContext,
    shine: &[bool],
) -> Vec<ResearchRequirementModel> {
    node.requirements
        .iter()
        .enumerate()
        .filter_map(|(index, req)| {
            let completed = node.finished_requirements[index].amount;
            if req.amount <= completed && !context.debug_show_requirements {
                return None;
            }
            let req_amount = if context.debug_show_requirements {
                req.amount
            } else {
                req.amount - completed
            };
            let item = item_info(&context.item_catalog, &req.item);
            let available = items.get(&req.item).min(req_amount);
            Some(ResearchRequirementModel {
                item: req.item.clone(),
                localized_name: item.localized_name,
                icon: item.icon,
                icon_size: RESEARCH_REQUIREMENT_ICON_SIZE,
                icon_pad_right: RESEARCH_REQUIREMENT_ICON_PAD_RIGHT,
                required_amount: req_amount,
                available_amount: available,
                amount_label: format!(
                    "{} / {}",
                    format_amount(available as i64),
                    format_amount(req_amount as i64)
                ),
                amount_color: if items.has(&req.item) {
                    RESEARCH_REQUIREMENT_TEXT_COLOR
                } else {
                    RESEARCH_REQUIREMENT_MISSING_COLOR
                },
                shine: shine.get(index).copied().unwrap_or(false),
            })
        })
        .collect()
}

fn objective_rows(objectives: &[ObjectiveKind]) -> Vec<ResearchObjectiveModel> {
    objectives
        .iter()
        .filter(|objective| !objective.complete())
        .map(|objective| ResearchObjectiveModel {
            text: format!("{RESEARCH_OBJECTIVE_PREFIX}{}", objective.display_token()),
            icon: if objective.complete() {
                RESEARCH_OBJECTIVE_OK_ICON
            } else {
                RESEARCH_OBJECTIVE_CANCEL_ICON
            },
            color: if objective.complete() {
                RESEARCH_REQUIREMENT_TEXT_COLOR
            } else {
                RESEARCH_REQUIREMENT_MISSING_COLOR
            },
            icon_pad_left: RESEARCH_OBJECTIVE_ICON_PAD_LEFT,
        })
        .collect()
}

fn inline_description(
    node: &TechNode,
    context: &ResearchDialogContext,
) -> Option<ResearchInlineDescription> {
    if !selectable(node) {
        return None;
    }

    let details = context.content_details.iter().find(|details| {
        details.content_type == node.content.content_type && details.name == node.content.name
    })?;
    if !details.inline_description {
        return None;
    }
    let description = details.description.clone()?;
    Some(ResearchInlineDescription {
        min_width: if description.chars().count() > RESEARCH_INLINE_DESCRIPTION_LONG_THRESHOLD {
            RESEARCH_INLINE_DESCRIPTION_LONG_MIN_WIDTH
        } else {
            0.0
        },
        text: description,
        color: RESEARCH_INLINE_DESCRIPTION_COLOR,
        margin: RESEARCH_INLINE_DESCRIPTION_MARGIN,
    })
}

fn item_info(catalog: &[ResearchItemInfo], name: &str) -> ResearchItemInfo {
    catalog
        .iter()
        .find(|item| item.name == name)
        .cloned()
        .unwrap_or_else(|| ResearchItemInfo::new(name))
}

fn item_cost(catalog: &[ResearchItemInfo], name: &str) -> f32 {
    catalog
        .iter()
        .find(|item| item.name == name)
        .map(|item| item.cost)
        .unwrap_or(1.0)
}

fn node_icon(node: &TechNode) -> String {
    node.icon.clone().unwrap_or_else(|| node_content_icon(node))
}

fn node_content_icon(node: &TechNode) -> String {
    format!(
        "{}-{}-ui",
        node.content.content_type.wire_name(),
        node.content.name
    )
}

trait ResearchNodeName {
    fn localized_name(&self) -> &str;
}

impl ResearchNodeName for TechNode {
    fn localized_name(&self) -> &str {
        self.name
            .as_deref()
            .unwrap_or(self.content.localized_name.as_str())
    }
}

fn layout_research_half(
    tree: &TechTree,
    root: TechNodeId,
    children: &[TechNodeId],
    location: TreeLocation,
    node_size: f32,
) -> (BTreeMap<TechNodeId, (f32, f32)>, f32) {
    let mut arena = TreeArena::new();
    let mut map = BTreeMap::new();
    let root_index = arena.add_node(node_size, node_size);
    map.insert(root, root_index);
    for &child in children {
        let child_index = add_layout_subtree(tree, &mut arena, &mut map, child, node_size);
        arena.add_child(root_index, child_index);
    }

    let mut layout = BranchTreeLayout::new();
    layout.gap_between_levels = RESEARCH_TREE_SPACING;
    layout.gap_between_nodes = RESEARCH_TREE_SPACING;
    layout.root_location = location;
    layout.layout(&mut arena, root_index);

    let mut coords = BTreeMap::new();
    for (id, index) in map {
        coords.insert(id, (arena.node(index).x, arena.node(index).y));
    }
    let root_y = coords.get(&root).expect("layout root must exist").1;
    (coords, root_y)
}

fn add_layout_subtree(
    tree: &TechTree,
    arena: &mut TreeArena,
    map: &mut BTreeMap<TechNodeId, usize>,
    node: TechNodeId,
    node_size: f32,
) -> usize {
    let index = arena.add_node(node_size, node_size);
    map.insert(node, index);
    let tech_node = tree.node(node).expect("research node must exist");
    for &child in &tech_node.children {
        let child_index = add_layout_subtree(tree, arena, map, child, node_size);
        arena.add_child(index, child_index);
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::{
        game::{ObjectiveContent, ObjectiveKind, TechContentRef},
        r#type::{Item, ItemStack},
    };

    fn stack(item: &str, amount: i32) -> ItemStack {
        ItemStack::new(item, amount)
    }

    fn item(name: &str, id: i16, localized: &str, cost: f32) -> Item {
        let mut item = Item::new(id, name);
        item.base.localized_name = Some(localized.into());
        item.cost = cost;
        item
    }

    fn context() -> ResearchDialogContext {
        ResearchDialogContext {
            item_catalog: ResearchItemInfo::from_items(&[
                item("copper", 0, "Copper", 1.0),
                item("lead", 1, "Lead", 2.0),
                item("graphite", 2, "Graphite", 4.0),
            ]),
            ..ResearchDialogContext::default()
        }
    }

    fn sample_tree() -> (TechTree, TechNodeId, TechNodeId, TechNodeId) {
        let mut tree = TechTree::new();
        let mut conveyor = None;
        let mut router = None;
        let root = tree.node_root(
            "serpulo",
            TechContentRef::block("core-shard")
                .localized("Core: Shard")
                .unlocked_host(true),
            Vec::new(),
            |tree| {
                conveyor = Some(tree.node_leaf(
                    TechContentRef::block("conveyor").localized("Conveyor"),
                    vec![stack("copper", 10), stack("lead", 5)],
                ));
                router = Some(tree.node_with_objectives(
                    TechContentRef::block("router").localized("Router"),
                    vec![stack("copper", 20)],
                    vec![ObjectiveKind::Produce(
                        ObjectiveContent::new("graphite").localized("Graphite"),
                    )],
                    |_| {},
                ));
            },
        );
        (tree, root, conveyor.unwrap(), router.unwrap())
    }

    #[test]
    fn shown_initializes_root_items_visibility_layout_and_shell_controls_like_java() {
        let (tree, root, conveyor, router) = sample_tree();
        let source = ResearchItemSource::from_totals([("copper", 15), ("lead", 3)]);
        let mut dialog = ResearchDialog::new();
        let mut context = context();
        context.portrait = true;

        let actions = dialog.shown_plan(&tree, &source, &context);
        let model = dialog.model(&tree, &context);

        assert_eq!(
            actions,
            vec![
                ResearchDialogAction::CheckMargin,
                ResearchDialogAction::PostCheckMargin,
                ResearchDialogAction::RebuildItems,
                ResearchDialogAction::RebuildItemDisplay {
                    used_shine_items: Vec::new()
                },
                ResearchDialogAction::TreeLayout,
                ResearchDialogAction::ClearHover,
                ResearchDialogAction::ClearInfo,
            ]
        );
        assert_eq!(dialog.root, Some(root));
        assert_eq!(model.title, "");
        assert!(model.should_pause);
        assert!(model.close_button_added);
        assert_eq!(model.node_size, 60.0);
        assert_eq!(model.title_button.label, "serpulo");
        assert!(!model.title_button.visible);
        assert_eq!(model.bottom_button.text, "@database");
        assert_eq!(model.bottom_button.icon, "book");
        assert_eq!(model.bottom_button.size, (210.0, 64.0));
        assert!(model.item_display.visible);
        assert_eq!(model.item_display.margin_top, 0.0);
        assert_eq!(model.item_display.rows[0].item, "copper");
        assert_eq!(model.item_display.rows[0].amount_label, "15");

        let root_model = model
            .view
            .nodes
            .iter()
            .find(|node| node.id == root)
            .unwrap();
        let conveyor_model = model
            .view
            .nodes
            .iter()
            .find(|node| node.id == conveyor)
            .unwrap();
        let router_model = model
            .view
            .nodes
            .iter()
            .find(|node| node.id == router)
            .unwrap();
        assert!(root_model.visible);
        assert!(!root_model.locked);
        assert!(conveyor_model.visible);
        assert!(conveyor_model.selectable);
        assert!(conveyor_model.locked);
        assert_eq!(conveyor_model.button.style_up, "Tex.button");
        assert!(router_model.visible);
        assert!(!router_model.selectable);
        assert_eq!(router_model.button.icon, "lock");
        assert_eq!(router_model.button.style_up, "Tex.buttonRed");
        assert!(!model.view.links.is_empty());
        assert!(model.view.bounds.y >= 0.0);
    }

    #[test]
    fn root_picker_filters_required_locked_roots_but_keeps_preferred_root_like_java() {
        let mut tree = TechTree::new();
        let serpulo = tree.node_root(
            "serpulo",
            TechContentRef::planet("serpulo")
                .localized("Serpulo")
                .unlocked_host(true),
            Vec::new(),
            |_| {},
        );
        let erekir = tree.node_root_with_unlock(
            "erekir",
            TechContentRef::planet("erekir").localized("Erekir"),
            true,
            Vec::new(),
            |_| {},
        );
        let tantros = tree.node_root_with_unlock(
            "tantros",
            TechContentRef::planet("tantros")
                .localized("Tantros")
                .unlocked_host(true),
            true,
            Vec::new(),
            |_| {},
        );
        let mut dialog = ResearchDialog::new();
        dialog.shown_plan(&tree, &ResearchItemSource::default(), &context());

        let mut context = context();
        context.campaign_tree = Some(erekir);
        let picker = dialog.root_picker_model(&tree, &context);
        assert_eq!(
            picker.rows.iter().map(|row| row.node).collect::<Vec<_>>(),
            vec![serpulo, erekir, tantros]
        );
        assert!(dialog.can_select_root_like_java(&tree));
        assert!(picker.rows[0].checked);

        context.campaign_tree = None;
        let picker = dialog.root_picker_model(&tree, &context);
        assert_eq!(
            picker.rows.iter().map(|row| row.node).collect::<Vec<_>>(),
            vec![serpulo, tantros]
        );
    }

    #[test]
    fn select_rebuilds_tree_resets_pan_scale_hover_and_hides_picker() {
        let mut tree = TechTree::new();
        let root_a = tree.node_root(
            "a",
            TechContentRef::planet("a")
                .localized("A")
                .unlocked_host(true),
            Vec::new(),
            |_| {},
        );
        let root_b = tree.node_root(
            "b",
            TechContentRef::planet("b")
                .localized("B")
                .unlocked_host(true),
            Vec::new(),
            |_| {},
        );
        let mut dialog = ResearchDialog::new();
        dialog.shown_plan(&tree, &ResearchItemSource::default(), &context());
        dialog.pan_x = 99.0;
        dialog.pan_y = 88.0;
        dialog.scale = 0.5;
        dialog.hover_node = Some(root_a);

        let actions =
            dialog.select_root_plan(&tree, root_b, &ResearchItemSource::default(), &context());

        assert_eq!(dialog.root, Some(root_b));
        assert_eq!(dialog.pan_x, 0.0);
        assert_eq!(dialog.pan_y, -200.0);
        assert_eq!(dialog.scale, 1.0);
        assert_eq!(dialog.hover_node, None);
        assert_eq!(
            actions,
            vec![
                ResearchDialogAction::SwitchTree { root: root_b },
                ResearchDialogAction::RebuildAll,
                ResearchDialogAction::RebuildItems,
                ResearchDialogAction::SetPan {
                    pan_x: 0.0,
                    pan_y: -200.0,
                },
                ResearchDialogAction::SetScale { scale: 1.0 },
                ResearchDialogAction::ClearHover,
                ResearchDialogAction::ClearInfo,
                ResearchDialogAction::TreeLayout,
                ResearchDialogAction::HideTechSelectDialog,
            ]
        );
    }

    #[test]
    fn can_spend_matches_java_locked_selectable_client_and_inventory_rules() {
        let (mut tree, _root, conveyor, router) = sample_tree();
        let mut dialog = ResearchDialog::new();
        let source = ResearchItemSource::from_totals([("copper", 0), ("lead", 0)]);
        let context = context();
        dialog.shown_plan(&tree, &source, &context);

        assert!(can_spend(&tree, conveyor, &dialog.items, &context));
        assert!(!can_spend(&tree, router, &dialog.items, &context));

        tree.node_mut(conveyor).unwrap().content.unlocked_host = true;
        assert!(!can_spend(&tree, conveyor, &dialog.items, &context));

        let source = ResearchItemSource::from_totals([("copper", 1), ("lead", 0)]);
        dialog.shown_plan(&tree, &source, &context);
        assert!(can_spend(&tree, conveyor, &dialog.items, &context));

        let client = ResearchDialogContext {
            net_client: true,
            ..context
        };
        assert!(!can_spend(&tree, conveyor, &dialog.items, &client));
    }

    #[test]
    fn spend_partially_consumes_sector_items_and_saves_progress_without_unlocking() {
        let (mut tree, _root, conveyor, _router) = sample_tree();
        let source = ResearchItemSource {
            planets: vec![ResearchPlanetInventory {
                name: "serpulo".into(),
                tech_tree: None,
                sectors: vec![
                    ResearchSectorInventory {
                        sector_id: 1,
                        has_base: true,
                        frozen: false,
                        items: BTreeMap::from([("copper".into(), 5), ("lead".into(), 1)]),
                    },
                    ResearchSectorInventory {
                        sector_id: 2,
                        has_base: true,
                        frozen: false,
                        items: BTreeMap::from([("copper".into(), 5), ("lead".into(), 2)]),
                    },
                ],
            }],
        };
        let mut dialog = ResearchDialog::new();
        let context = context();
        dialog.shown_plan(&tree, &source, &context);

        let actions = dialog.spend_plan(&mut tree, conveyor, &context);

        assert_eq!(
            tree.node(conveyor).unwrap().finished_requirements,
            vec![stack("copper", 10), stack("lead", 3)]
        );
        assert!(!tree.node(conveyor).unwrap().content.unlocked_host);
        assert_eq!(tree.progress()["req-conveyor-copper"], 10);
        assert_eq!(tree.progress()["req-conveyor-lead"], 3);
        assert_eq!(dialog.items.get("copper"), 0);
        assert_eq!(dialog.items.get("lead"), 0);
        assert!(actions.contains(&ResearchDialogAction::RemoveSectorItem {
            planet_name: "serpulo".into(),
            sector_id: 1,
            item: "copper".into(),
            amount: 5,
        }));
        assert!(actions.contains(&ResearchDialogAction::SaveNode { node: conveyor }));
        assert!(!actions.iter().any(|action| {
            matches!(action, ResearchDialogAction::FireResearchEvent { content_name, .. } if content_name == "conveyor")
        }));
    }

    #[test]
    fn spend_unlocks_node_and_parent_chain_when_requirements_complete() {
        let mut tree = TechTree::new();
        let mut child = None;
        let mut grandchild = None;
        let root = tree.node_root("root", TechContentRef::block("root"), Vec::new(), |tree| {
            child = Some(tree.node_with_objectives(
                TechContentRef::block("child"),
                Vec::new(),
                Vec::new(),
                |tree| {
                    grandchild = Some(tree.node_leaf(
                        TechContentRef::block("grandchild"),
                        vec![stack("copper", 1)],
                    ));
                },
            ));
        });
        let child = child.unwrap();
        let grandchild = grandchild.unwrap();
        let mut dialog = ResearchDialog::new();
        let context = context();
        dialog.shown_plan(
            &tree,
            &ResearchItemSource::from_totals([("copper", 1)]),
            &context,
        );

        let actions = dialog.spend_plan(&mut tree, grandchild, &context);

        assert!(tree.node(root).unwrap().content.unlocked_host);
        assert!(tree.node(child).unwrap().content.unlocked_host);
        assert!(tree.node(grandchild).unwrap().content.unlocked_host);
        assert!(actions.contains(&ResearchDialogAction::UnlockContent {
            node: grandchild,
            content_name: "grandchild".into(),
        }));
        assert!(actions.contains(&ResearchDialogAction::UnlockContent {
            node: child,
            content_name: "child".into(),
        }));
        assert!(actions.contains(&ResearchDialogAction::UnlockContent {
            node: root,
            content_name: "root".into(),
        }));
        assert!(actions.iter().any(|action| {
            matches!(action, ResearchDialogAction::FireResearchEvent { content_name, .. } if content_name == "grandchild")
        }));
    }

    #[test]
    fn info_panel_matches_locked_requirements_objectives_completed_and_inline_description() {
        let (mut tree, _root, conveyor, router) = sample_tree();
        tree.node_mut(conveyor).unwrap().finished_requirements[0].amount = 5;
        let mut dialog = ResearchDialog::new();
        let mut context = context();
        context.content_details.push(
            ResearchContentDetails::new(ContentType::Block, "conveyor")
                .description("Moves items forward very quickly.")
                .inline_description(true),
        );
        dialog.shown_plan(
            &tree,
            &ResearchItemSource::from_totals([("copper", 4), ("lead", 0)]),
            &context,
        );
        dialog.hover_node = Some(conveyor);
        let model = dialog.model_with_shine(&tree, &context, &[], &[true, false]);
        let panel = model.view.info_panel.unwrap();

        assert!(panel.selectable);
        assert_eq!(panel.title, "Conveyor");
        assert!(panel.info_button.is_some());
        assert!(matches!(
            panel.body,
            ResearchInfoBody::Requirements {
                progress: Some(ResearchProgressModel {
                    percent: 25,
                    shine: true,
                    ..
                }),
                ..
            }
        ));
        if let ResearchInfoBody::Requirements { rows, .. } = panel.body {
            assert_eq!(rows[0].item, "copper");
            assert_eq!(rows[0].amount_label, "4 / 5");
            assert_eq!(rows[0].amount_color, "Color.lightGray");
            assert!(rows[0].shine);
            assert_eq!(rows[1].item, "lead");
            assert_eq!(rows[1].amount_label, "0 / 5");
            assert_eq!(rows[1].amount_color, "Color.scarlet");
        }
        assert_eq!(
            panel.inline_description.unwrap().text,
            "Moves items forward very quickly."
        );

        dialog.hover_node = Some(router);
        let panel = dialog.model(&tree, &context).view.info_panel.unwrap();
        assert_eq!(panel.title, "[accent]???");
        assert!(matches!(panel.body, ResearchInfoBody::Objectives { .. }));

        tree.node_mut(conveyor).unwrap().content.unlocked_host = true;
        dialog.hover_node = Some(conveyor);
        let panel = dialog.model(&tree, &context).view.info_panel.unwrap();
        assert_eq!(
            panel.body,
            ResearchInfoBody::Completed { text: "@completed" }
        );
    }

    #[test]
    fn mobile_and_client_info_panel_follow_java_visibility_rules() {
        let (tree, _root, conveyor, _router) = sample_tree();
        let mut dialog = ResearchDialog::new();
        let mut context = context();
        context.mobile = true;
        dialog.shown_plan(
            &tree,
            &ResearchItemSource::from_totals([("copper", 0), ("lead", 0)]),
            &context,
        );
        dialog.hover_node = Some(conveyor);
        let panel = dialog.model(&tree, &context).view.info_panel.unwrap();
        assert_eq!(
            panel.mobile_research_button.unwrap(),
            ResearchMobileResearchButton {
                text: "@research",
                icon: "ok",
                disabled: false,
                height: 44.0,
                colspan: 3,
                disabled_drawable: "Tex.button",
                up_drawable: "Tex.buttonOver",
                over_drawable: "Tex.buttonDown",
            }
        );

        context.net_client = true;
        let panel = dialog.model(&tree, &context).view.info_panel.unwrap();
        assert!(matches!(
            panel.body,
            ResearchInfoBody::LockedClient {
                text: "@locked",
                color: "Pal.remove"
            }
        ));
        assert!(panel.mobile_research_button.is_none());
        assert!(!dialog.model(&tree, &context).item_display.visible);
    }

    #[test]
    fn keys_database_events_zoom_and_pan_actions_match_java_shell() {
        let (tree, _root, _conveyor, _router) = sample_tree();
        let mut dialog = ResearchDialog::new();
        let context = context();
        dialog.shown_plan(&tree, &ResearchItemSource::default(), &context);

        assert_eq!(
            ResearchDialog::key_down_plan(ResearchDialogKey::Research),
            vec![ResearchDialogAction::PostHideDialog]
        );
        assert_eq!(
            ResearchDialog::database_button_plan(),
            vec![
                ResearchDialogAction::HideDialog,
                ResearchDialogAction::ShowDatabase,
            ]
        );
        assert_eq!(
            ResearchDialog::reset_event_plan(),
            vec![ResearchDialogAction::HideDialog]
        );
        assert_eq!(
            ResearchDialog::hidden_plan(),
            vec![ResearchDialogAction::PlanetSetup]
        );

        let mut client_context = context.clone();
        client_context.net_client = true;
        assert_eq!(
            dialog.unlock_event_plan(&client_context),
            vec![ResearchDialogAction::DelayedClientUnlockRebuild]
        );
        assert!(dialog.needs_rebuild);
        assert!(dialog.unlock_event_plan(&client_context).is_empty());
        let actions = dialog.finish_delayed_unlock_rebuild_plan(&tree, &client_context);
        assert_eq!(actions[0], ResearchDialogAction::ClearHover);
        assert!(!dialog.needs_rebuild);

        dialog.scale = 1.0;
        assert_eq!(
            dialog.scroll_plan(20.0),
            vec![ResearchDialogAction::SetScale { scale: 0.25 }]
        );
        assert_eq!(
            dialog.zoom_plan(100.0, 1000.0),
            vec![ResearchDialogAction::SetScale { scale: 1.0 }]
        );
        dialog.zoom_touch_up_plan();
        assert_eq!(dialog.last_zoom, 1.0);

        let pan = dialog.pan_plan(20.0, -10.0, 800.0, 600.0, &context);
        assert!(matches!(pan[..], [ResearchDialogAction::SetPan { .. }]));
        assert!(dialog.moved);
        dialog.release_plan();
        assert!(!dialog.moved);
    }
}
