//! Map-list dialog model mirroring upstream `mindustry.ui.dialogs.MapListDialog`.

use std::collections::BTreeSet;

use crate::mindustry::{
    game::Gamemode, maps::MapDescriptor, ui::upstream_menu_bundle_value_for_locale,
};

pub const MAP_LIST_BACK_BUTTON_TEXT: &str = "@back";
pub const MAP_LIST_BACK_ICON: &str = "left";
pub const MAP_LIST_BACK_BUTTON_SIZE: (f32, f32) = (210.0, 64.0);
pub const MAP_LIST_PORTRAIT_BACK_BUTTON_SIZE: (f32, f32) = (420.0, 64.0);
pub const MAP_LIST_SEARCH_ICON: &str = "zoom";
pub const MAP_LIST_SEARCH_MESSAGE: &str = "@editor.search";
pub const MAP_LIST_SEARCH_MAX_LENGTH: usize = 50;
pub const MAP_LIST_FILTER_ICON: &str = "filter";
pub const MAP_LIST_FILTER_STYLE: &str = "Styles.emptyi";
pub const MAP_LIST_FILTER_TOOLTIP: &str = "@editor.filters";
pub const MAP_LIST_PANE_PAD_LEFT: f32 = 28.0;
pub const MAP_LIST_PANE_PAD_BOTTOM: f32 = 64.0;
pub const MAP_LIST_TABLE_MARGIN_RIGHT: f32 = 12.0;
pub const MAP_LIST_CARD_WIDTH: f32 = 200.0;
pub const MAP_LIST_CARD_PAD: f32 = 8.0;
pub const MAP_LIST_CARD_MARGIN: f32 = 9.0;
pub const MAP_LIST_CARD_PREVIEW_SIZE: f32 = 180.0;
pub const MAP_LIST_CARD_TITLE_WIDTH: f32 = 182.0;
pub const MAP_LIST_CARD_DIVIDER_PAD: f32 = 4.0;
pub const MAP_LIST_CARD_DIVIDER_COLOR: &str = "Pal.gray";
pub const MAP_LIST_CARD_MODE_ICON_SIZE: f32 = 16.0;
pub const MAP_LIST_CARD_MODE_ICON_PAD: f32 = 4.0;
pub const MAP_LIST_CARD_TYPE_PAD_TOP: f32 = 3.0;
pub const MAP_LIST_CARD_TYPE_COLOR: &str = "Color.gray";
pub const MAP_LIST_NO_MAPS_TEXT: &str = "@maps.none";
pub const MAP_LIST_FILTER_DIALOG_TITLE: &str = "@editor.filters";
pub const MAP_LIST_FILTER_SECTION_MODE: &str = "@editor.filters.mode";
pub const MAP_LIST_FILTER_SECTION_PRIORITIES: &str = "@editor.filters.priorities";
pub const MAP_LIST_FILTER_SECTION_TYPE: &str = "@editor.filters.type";
pub const MAP_LIST_FILTER_SECTION_SEARCH: &str = "@editor.filters.search";
pub const MAP_LIST_FILTER_BUTTON_SIZE: f32 = 60.0;
pub const MAP_LIST_FILTER_BUTTON_STYLE: &str = "Styles.emptyTogglei";
pub const MAP_LIST_FILTER_TABLE_BACKGROUND: &str = "Tex.button";
pub const MAP_LIST_FILTER_TYPE_BUTTON_SIZE: (f32, f32) = (150.0, 60.0);
pub const MAP_LIST_FILTER_TYPE_BUTTON_STYLE: &str = "Styles.flatTogglet";
pub const MAP_LIST_PLANET_FILTER_TITLE: &str = "@editor.filters.planetselect";
pub const MAP_LIST_PLANET_ANY_TEXT: &str = "@rules.anyenv";
pub const MAP_LIST_PLANET_ANY_NAME: &str = "sun";
pub const MAP_LIST_PLANET_BUTTON_TOOLTIP: &str = "@editor.filters.planetselect";
pub const MAP_LIST_PLANET_OPTION_SIZE: (f32, f32) = (300.0, 60.0);
pub const MAP_LIST_PLANET_OPTION_STYLE: &str = "Styles.flatTogglet";
pub const MAP_LIST_SETTING_SHOW_BUILTIN: &str = "editorshowbuiltinmaps";
pub const MAP_LIST_SETTING_SHOW_CUSTOM: &str = "editorshowcustommaps";
pub const MAP_LIST_SETTING_SHOW_MODDED: &str = "editorshowmoddedmaps";
pub const MAP_LIST_SETTING_SEARCH_AUTHOR: &str = "editorsearchauthor";
pub const MAP_LIST_SETTING_SEARCH_DESCRIPTION: &str = "editorsearchdescription";
pub const MAP_LIST_SETTING_SEARCH_MOD_NAME: &str = "editorsearchmodname";
pub const MAP_LIST_SETTING_PRIORITIZE_CUSTOM: &str = "editorprioritizecustom";
pub const MAP_LIST_SETTING_PRIORITIZE_MODDED: &str = "editorprioritizemodded";
pub const MAP_LIST_SETTING_FILTER_PLANETS: &str = "editorfilterplanets";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapListPlanet {
    pub name: String,
    pub localized_name: String,
    pub accessible: bool,
}

impl MapListPlanet {
    pub fn new(name: impl Into<String>, localized_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            localized_name: localized_name.into(),
            accessible: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MapListMaps {
    pub custom: Vec<MapDescriptor>,
    pub builtin: Vec<MapDescriptor>,
    pub modded: Vec<MapDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapListDialogContext {
    pub portrait: bool,
    pub graphics_width: i32,
    pub desktop: bool,
    pub mobile: bool,
}

impl Default for MapListDialogContext {
    fn default() -> Self {
        Self {
            portrait: false,
            graphics_width: 800,
            desktop: false,
            mobile: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapListDialogModel {
    pub title: String,
    pub display_type: bool,
    pub available_planets: Vec<String>,
    pub bottom_buttons: Vec<MapListBottomButton>,
    pub search: MapListSearchModel,
    pub pane: MapListPaneModel,
    pub maps: MapListMapsModel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapListBottomButton {
    pub text: &'static str,
    pub icon: &'static str,
    pub size: (f32, f32),
    pub colspan: i32,
    pub row_after: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapListSearchModel {
    pub icon: &'static str,
    pub text: String,
    pub message_text: &'static str,
    pub max_length: usize,
    pub grow_x: bool,
    pub filter_icon: &'static str,
    pub filter_style: &'static str,
    pub filter_tooltip: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapListPaneModel {
    pub fade_scroll_bars: bool,
    pub scrolling_disabled_x: bool,
    pub pad_left: f32,
    pub uniform_x: bool,
    pub grow: bool,
    pub pad_bottom: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapListMapsModel {
    pub table_margin_right: f32,
    pub max_width: usize,
    pub map_size: f32,
    pub cards: Vec<MapListCard>,
    pub empty_text: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapListCard {
    pub map_file: String,
    pub map_name: String,
    pub plain_name: String,
    pub width: f32,
    pub style: &'static str,
    pub bottom_aligned: bool,
    pub pad: f32,
    pub margin: f32,
    pub row_start: bool,
    pub mode_icons: Vec<MapListModeIcon>,
    pub title_width: f32,
    pub title_centered: bool,
    pub title_ellipsis: bool,
    pub divider_color: &'static str,
    pub divider_pad: f32,
    pub preview_scaling: &'static str,
    pub preview_size: f32,
    pub type_label: Option<String>,
    pub type_color: &'static str,
    pub type_pad_top: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapListModeIcon {
    pub mode: Option<Gamemode>,
    pub icon: String,
    pub size: f32,
    pub pad: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapListFiltersModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub mode_section: &'static str,
    pub priority_section: &'static str,
    pub type_section: &'static str,
    pub search_section: &'static str,
    pub mode_buttons: Vec<MapListModeFilterButton>,
    pub priority_buttons: Vec<MapListPriorityFilterButton>,
    pub planet_button: MapListPlanetFilterButton,
    pub type_buttons: Vec<MapListToggleFilterButton>,
    pub search_buttons: Vec<MapListToggleFilterButton>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapListModeFilterButton {
    pub mode: Gamemode,
    pub icon: String,
    pub style: &'static str,
    pub size: i32,
    pub checked: bool,
    pub tooltip: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapListPriorityKind {
    Custom,
    Modded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapListPriorityFilterButton {
    pub kind: MapListPriorityKind,
    pub icon: &'static str,
    pub style: &'static str,
    pub size: i32,
    pub checked: bool,
    pub disabled: bool,
    pub tooltip: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapListPlanetFilterButton {
    pub icon: &'static str,
    pub style: &'static str,
    pub size: i32,
    pub checked: bool,
    pub tooltip: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapListToggleKind {
    ShowCustom,
    ShowBuiltin,
    ShowModded,
    SearchAuthor,
    SearchDescription,
    SearchModName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapListToggleFilterButton {
    pub kind: MapListToggleKind,
    pub text: &'static str,
    pub style: &'static str,
    pub size: (i32, i32),
    pub checked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapListPlanetFiltersModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub options: Vec<MapListPlanetFilterOption>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapListPlanetFilterOption {
    pub name: String,
    pub text: String,
    pub icon: String,
    pub style: &'static str,
    pub size: (i32, i32),
    pub margin_left: f32,
    pub checked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapListFilterAction {
    ToggleMode(Gamemode),
    ToggleShowCustom,
    ToggleShowBuiltin,
    ToggleShowModded,
    TogglePlanet(usize),
    ClearPlanets,
    ToggleSearchAuthor,
    ToggleSearchDescription,
    ToggleSearchModName,
    TogglePrioritizeCustom,
    TogglePrioritizeModded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapListDialogAction {
    HideDialog,
    HideActiveDialog,
    ShowDialog,
    FocusSearch,
    RebuildMaps,
    ShowMap {
        map_file: String,
        map_name: String,
    },
    ShowMapFilters,
    ShowPlanetFilters,
    PersistBool {
        key: &'static str,
        value: bool,
    },
    PersistPlanets {
        key: &'static str,
        planets: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapListDialog {
    pub title: String,
    pub display_type: bool,
    pub search_string: Option<String>,
    pub modes: Vec<Gamemode>,
    pub planets: Vec<String>,
    pub available_planets: Vec<String>,
    pub show_builtin: bool,
    pub show_custom: bool,
    pub show_modded: bool,
    pub search_author: bool,
    pub search_description: bool,
    pub search_mod_name: bool,
    pub prioritize_modded: bool,
    pub prioritize_custom: bool,
    pub active_dialog_open: bool,
}

impl MapListDialog {
    pub fn new(title: impl Into<String>, display_type: bool) -> Self {
        Self {
            title: title.into(),
            display_type,
            search_string: None,
            modes: Vec::new(),
            planets: Vec::new(),
            available_planets: Vec::new(),
            show_builtin: true,
            show_custom: true,
            show_modded: true,
            search_author: false,
            search_description: false,
            search_mod_name: false,
            prioritize_modded: false,
            prioritize_custom: false,
            active_dialog_open: false,
        }
    }

    pub fn setup(
        &mut self,
        maps: &MapListMaps,
        planets: &[MapListPlanet],
        context: &MapListDialogContext,
    ) -> MapListDialogModel {
        self.available_planets = available_planets(planets);
        self.search_string = None;
        self.model(maps, context)
    }

    pub fn model(&self, maps: &MapListMaps, context: &MapListDialogContext) -> MapListDialogModel {
        MapListDialogModel {
            title: self.title.clone(),
            display_type: self.display_type,
            available_planets: self.available_planets.clone(),
            bottom_buttons: bottom_buttons(self.display_type, context),
            search: MapListSearchModel {
                icon: MAP_LIST_SEARCH_ICON,
                text: self.search_string.clone().unwrap_or_default(),
                message_text: MAP_LIST_SEARCH_MESSAGE,
                max_length: MAP_LIST_SEARCH_MAX_LENGTH,
                grow_x: true,
                filter_icon: MAP_LIST_FILTER_ICON,
                filter_style: MAP_LIST_FILTER_STYLE,
                filter_tooltip: MAP_LIST_FILTER_TOOLTIP,
            },
            pane: MapListPaneModel {
                fade_scroll_bars: false,
                scrolling_disabled_x: true,
                pad_left: MAP_LIST_PANE_PAD_LEFT,
                uniform_x: true,
                grow: true,
                pad_bottom: MAP_LIST_PANE_PAD_BOTTOM,
            },
            maps: self.rebuild_maps_model(maps, context),
        }
    }

    pub fn on_resize(
        &mut self,
        maps: &MapListMaps,
        planets: &[MapListPlanet],
        context: &MapListDialogContext,
    ) -> (Vec<MapListDialogAction>, MapListDialogModel) {
        let mut actions = Vec::new();
        if self.active_dialog_open {
            self.active_dialog_open = false;
            actions.push(MapListDialogAction::HideActiveDialog);
        }
        let model = self.setup(maps, planets, context);
        actions.push(MapListDialogAction::RebuildMaps);
        (actions, model)
    }

    pub fn show_plan(&self, context: &MapListDialogContext) -> Vec<MapListDialogAction> {
        let mut actions = vec![MapListDialogAction::ShowDialog];
        if context.desktop {
            actions.push(MapListDialogAction::FocusSearch);
        }
        actions
    }

    pub fn back_plan() -> Vec<MapListDialogAction> {
        vec![MapListDialogAction::HideDialog]
    }

    pub fn search_changed_plan(&mut self, text: &str) -> Vec<MapListDialogAction> {
        self.search_string = (!text.is_empty()).then(|| text.to_lowercase());
        vec![MapListDialogAction::RebuildMaps]
    }

    pub fn show_map_plan(card: &MapListCard) -> MapListDialogAction {
        MapListDialogAction::ShowMap {
            map_file: card.map_file.clone(),
            map_name: card.map_name.clone(),
        }
    }

    pub fn show_map_filters_plan(&mut self) -> MapListDialogAction {
        self.active_dialog_open = true;
        MapListDialogAction::ShowMapFilters
    }

    pub fn show_planet_filters_plan(&mut self) -> MapListDialogAction {
        self.active_dialog_open = true;
        MapListDialogAction::ShowPlanetFilters
    }

    pub fn filters_model(&self) -> MapListFiltersModel {
        MapListFiltersModel {
            title: MAP_LIST_FILTER_DIALOG_TITLE,
            close_button_added: true,
            mode_section: MAP_LIST_FILTER_SECTION_MODE,
            priority_section: MAP_LIST_FILTER_SECTION_PRIORITIES,
            type_section: MAP_LIST_FILTER_SECTION_TYPE,
            search_section: MAP_LIST_FILTER_SECTION_SEARCH,
            mode_buttons: Gamemode::ALL
                .iter()
                .copied()
                .map(|mode| MapListModeFilterButton {
                    mode,
                    icon: format!("mode{}", capitalize(mode.wire_name())),
                    style: MAP_LIST_FILTER_BUTTON_STYLE,
                    size: MAP_LIST_FILTER_BUTTON_SIZE as i32,
                    checked: self.modes.contains(&mode),
                    tooltip: format!("@mode.{}.name", mode.wire_name()),
                })
                .collect(),
            priority_buttons: vec![
                MapListPriorityFilterButton {
                    kind: MapListPriorityKind::Custom,
                    icon: "players",
                    style: MAP_LIST_FILTER_BUTTON_STYLE,
                    size: MAP_LIST_FILTER_BUTTON_SIZE as i32,
                    checked: self.show_custom && self.prioritize_custom,
                    disabled: !self.show_custom,
                    tooltip: "@editor.filters.prioritizecustom",
                },
                MapListPriorityFilterButton {
                    kind: MapListPriorityKind::Modded,
                    icon: "hammer",
                    style: MAP_LIST_FILTER_BUTTON_STYLE,
                    size: MAP_LIST_FILTER_BUTTON_SIZE as i32,
                    checked: self.show_modded && self.prioritize_modded,
                    disabled: !self.show_modded,
                    tooltip: "@editor.filters.prioritizemod",
                },
            ],
            planet_button: MapListPlanetFilterButton {
                icon: "planet",
                style: MAP_LIST_FILTER_BUTTON_STYLE,
                size: MAP_LIST_FILTER_BUTTON_SIZE as i32,
                checked: self
                    .planets
                    .iter()
                    .any(|planet| self.available_planets.iter().any(|name| name == planet)),
                tooltip: MAP_LIST_PLANET_BUTTON_TOOLTIP,
            },
            type_buttons: vec![
                MapListToggleFilterButton {
                    kind: MapListToggleKind::ShowCustom,
                    text: "@custom",
                    style: MAP_LIST_FILTER_TYPE_BUTTON_STYLE,
                    size: (
                        MAP_LIST_FILTER_TYPE_BUTTON_SIZE.0 as i32,
                        MAP_LIST_FILTER_TYPE_BUTTON_SIZE.1 as i32,
                    ),
                    checked: self.show_custom,
                },
                MapListToggleFilterButton {
                    kind: MapListToggleKind::ShowBuiltin,
                    text: "@builtin",
                    style: MAP_LIST_FILTER_TYPE_BUTTON_STYLE,
                    size: (
                        MAP_LIST_FILTER_TYPE_BUTTON_SIZE.0 as i32,
                        MAP_LIST_FILTER_TYPE_BUTTON_SIZE.1 as i32,
                    ),
                    checked: self.show_builtin,
                },
                MapListToggleFilterButton {
                    kind: MapListToggleKind::ShowModded,
                    text: "@modded",
                    style: MAP_LIST_FILTER_TYPE_BUTTON_STYLE,
                    size: (
                        MAP_LIST_FILTER_TYPE_BUTTON_SIZE.0 as i32,
                        MAP_LIST_FILTER_TYPE_BUTTON_SIZE.1 as i32,
                    ),
                    checked: self.show_modded,
                },
            ],
            search_buttons: vec![
                MapListToggleFilterButton {
                    kind: MapListToggleKind::SearchAuthor,
                    text: "@editor.filters.author",
                    style: MAP_LIST_FILTER_TYPE_BUTTON_STYLE,
                    size: (
                        MAP_LIST_FILTER_TYPE_BUTTON_SIZE.0 as i32,
                        MAP_LIST_FILTER_TYPE_BUTTON_SIZE.1 as i32,
                    ),
                    checked: self.search_author,
                },
                MapListToggleFilterButton {
                    kind: MapListToggleKind::SearchDescription,
                    text: "@editor.filters.description",
                    style: MAP_LIST_FILTER_TYPE_BUTTON_STYLE,
                    size: (
                        MAP_LIST_FILTER_TYPE_BUTTON_SIZE.0 as i32,
                        MAP_LIST_FILTER_TYPE_BUTTON_SIZE.1 as i32,
                    ),
                    checked: self.search_description,
                },
                MapListToggleFilterButton {
                    kind: MapListToggleKind::SearchModName,
                    text: "@editor.filters.modname",
                    style: MAP_LIST_FILTER_TYPE_BUTTON_STYLE,
                    size: (
                        MAP_LIST_FILTER_TYPE_BUTTON_SIZE.0 as i32,
                        MAP_LIST_FILTER_TYPE_BUTTON_SIZE.1 as i32,
                    ),
                    checked: self.search_mod_name,
                },
            ],
        }
    }

    pub fn planet_filters_model(
        &self,
        planets: &[MapListPlanet],
        locale: &str,
    ) -> MapListPlanetFiltersModel {
        MapListPlanetFiltersModel {
            title: MAP_LIST_PLANET_FILTER_TITLE,
            close_button_added: true,
            options: planet_filter_options(planets, &self.planets, locale),
        }
    }

    pub fn dispatch_filter_action(
        &mut self,
        action: MapListFilterAction,
        planets: &[MapListPlanet],
    ) -> Vec<MapListDialogAction> {
        let mut actions = Vec::new();
        match action {
            MapListFilterAction::ToggleMode(mode) => toggle_value(&mut self.modes, mode),
            MapListFilterAction::ToggleShowCustom => {
                self.show_custom = !self.show_custom;
                actions.push(MapListDialogAction::PersistBool {
                    key: MAP_LIST_SETTING_SHOW_CUSTOM,
                    value: self.show_custom,
                });
                if !self.show_custom {
                    self.prioritize_custom = false;
                    actions.push(MapListDialogAction::PersistBool {
                        key: MAP_LIST_SETTING_PRIORITIZE_CUSTOM,
                        value: false,
                    });
                }
            }
            MapListFilterAction::ToggleShowBuiltin => {
                self.show_builtin = !self.show_builtin;
                actions.push(MapListDialogAction::PersistBool {
                    key: MAP_LIST_SETTING_SHOW_BUILTIN,
                    value: self.show_builtin,
                });
            }
            MapListFilterAction::ToggleShowModded => {
                self.show_modded = !self.show_modded;
                actions.push(MapListDialogAction::PersistBool {
                    key: MAP_LIST_SETTING_SHOW_MODDED,
                    value: self.show_modded,
                });
                if !self.show_modded {
                    self.prioritize_modded = false;
                    actions.push(MapListDialogAction::PersistBool {
                        key: MAP_LIST_SETTING_PRIORITIZE_MODDED,
                        value: false,
                    });
                }
            }
            MapListFilterAction::TogglePlanet(index) => {
                if let Some(option) = planet_filter_options(planets, &self.planets, "en").get(index)
                {
                    toggle_string(&mut self.planets, &option.name);
                    actions.push(MapListDialogAction::PersistPlanets {
                        key: MAP_LIST_SETTING_FILTER_PLANETS,
                        planets: self.planets.clone(),
                    });
                }
            }
            MapListFilterAction::ClearPlanets => {
                let available = self
                    .available_planets
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>();
                self.planets.retain(|planet| !available.contains(planet));
                actions.push(MapListDialogAction::PersistPlanets {
                    key: MAP_LIST_SETTING_FILTER_PLANETS,
                    planets: self.planets.clone(),
                });
            }
            MapListFilterAction::ToggleSearchAuthor => {
                self.search_author = !self.search_author;
                actions.push(MapListDialogAction::PersistBool {
                    key: MAP_LIST_SETTING_SEARCH_AUTHOR,
                    value: self.search_author,
                });
            }
            MapListFilterAction::ToggleSearchDescription => {
                self.search_description = !self.search_description;
                actions.push(MapListDialogAction::PersistBool {
                    key: MAP_LIST_SETTING_SEARCH_DESCRIPTION,
                    value: self.search_description,
                });
            }
            MapListFilterAction::ToggleSearchModName => {
                self.search_mod_name = !self.search_mod_name;
                actions.push(MapListDialogAction::PersistBool {
                    key: MAP_LIST_SETTING_SEARCH_MOD_NAME,
                    value: self.search_mod_name,
                });
            }
            MapListFilterAction::TogglePrioritizeCustom => {
                if self.show_custom {
                    self.prioritize_custom = !self.prioritize_custom;
                    actions.push(MapListDialogAction::PersistBool {
                        key: MAP_LIST_SETTING_PRIORITIZE_CUSTOM,
                        value: self.prioritize_custom,
                    });
                    if self.prioritize_custom {
                        self.prioritize_modded = false;
                        actions.push(MapListDialogAction::PersistBool {
                            key: MAP_LIST_SETTING_PRIORITIZE_MODDED,
                            value: false,
                        });
                    }
                }
            }
            MapListFilterAction::TogglePrioritizeModded => {
                if self.show_modded {
                    self.prioritize_modded = !self.prioritize_modded;
                    actions.push(MapListDialogAction::PersistBool {
                        key: MAP_LIST_SETTING_PRIORITIZE_MODDED,
                        value: self.prioritize_modded,
                    });
                    if self.prioritize_modded {
                        self.prioritize_custom = false;
                        actions.push(MapListDialogAction::PersistBool {
                            key: MAP_LIST_SETTING_PRIORITIZE_CUSTOM,
                            value: false,
                        });
                    }
                }
            }
        }
        actions.push(MapListDialogAction::RebuildMaps);
        actions
    }

    pub fn rebuild_maps_model(
        &self,
        maps: &MapListMaps,
        context: &MapListDialogContext,
    ) -> MapListMapsModel {
        let max_width = map_columns(context.graphics_width);
        let cards = filtered_maps(self, maps)
            .into_iter()
            .enumerate()
            .map(|(index, map)| map_card(map, index, max_width, self.display_type))
            .collect::<Vec<_>>();
        MapListMapsModel {
            table_margin_right: MAP_LIST_TABLE_MARGIN_RIGHT,
            max_width,
            map_size: MAP_LIST_CARD_WIDTH,
            empty_text: cards.is_empty().then_some(MAP_LIST_NO_MAPS_TEXT),
            cards,
        }
    }
}

pub fn map_columns(graphics_width: i32) -> usize {
    (graphics_width / 230).max(1) as usize
}

pub fn filtered_maps<'a>(dialog: &MapListDialog, maps: &'a MapListMaps) -> Vec<&'a MapDescriptor> {
    let mut map_list: Vec<&MapDescriptor> = Vec::new();
    if dialog.show_custom {
        extend_distinct(&mut map_list, &maps.custom);
    }
    if dialog.show_builtin {
        extend_distinct(&mut map_list, &maps.builtin);
    }
    if dialog.show_modded {
        extend_distinct(&mut map_list, &maps.modded);
    }

    let active_planets = dialog
        .planets
        .iter()
        .filter(|planet| {
            dialog
                .available_planets
                .iter()
                .any(|available| available == *planet)
        })
        .cloned()
        .collect::<Vec<_>>();
    let query = dialog.search_string.as_deref();

    map_list.retain(|map| {
        dialog.modes.iter().all(|mode| mode.valid(map))
            && planet_allowed(map, &active_planets)
            && query
                .map(|query| map_matches_search(dialog, map, query))
                .unwrap_or(true)
    });
    sort_maps(dialog, &mut map_list);
    map_list
}

fn extend_distinct<'a>(out: &mut Vec<&'a MapDescriptor>, maps: &'a [MapDescriptor]) {
    for map in maps {
        if !out.iter().any(|existing| *existing == map) {
            out.push(map);
        }
    }
}

fn sort_maps(dialog: &MapListDialog, maps: &mut Vec<&MapDescriptor>) {
    if dialog.prioritize_modded {
        maps.sort_by(|left, right| {
            let modded = right.mod_name.is_some().cmp(&left.mod_name.is_some());
            if modded != std::cmp::Ordering::Equal {
                return modded;
            }
            if left.mod_name.is_some() && right.mod_name.is_some() {
                let by_mod = stripped_mod_name(left).cmp(&stripped_mod_name(right));
                if by_mod != std::cmp::Ordering::Equal {
                    return by_mod;
                }
            }
            left.plain_name().cmp(&right.plain_name())
        });
    } else if dialog.prioritize_custom {
        maps.sort_by(|left, right| {
            let custom = right.custom.cmp(&left.custom);
            if custom != std::cmp::Ordering::Equal {
                return custom;
            }
            left.plain_name().cmp(&right.plain_name())
        });
    } else {
        maps.sort_by_key(|map| map.plain_name());
    }
}

fn planet_allowed(map: &MapDescriptor, active_planets: &[String]) -> bool {
    active_planets.is_empty()
        || active_planets
            .iter()
            .any(|planet| planet == &map.rules().planet)
}

fn map_matches_search(dialog: &MapListDialog, map: &MapDescriptor, query: &str) -> bool {
    map.plain_name().to_lowercase().contains(query)
        || (dialog.search_author && map.plain_author().to_lowercase().contains(query))
        || (dialog.search_description && map.plain_description().to_lowercase().contains(query))
        || (dialog.search_mod_name && stripped_mod_name(map).to_lowercase().contains(query))
}

fn map_card(
    map: &MapDescriptor,
    index: usize,
    max_width: usize,
    display_type: bool,
) -> MapListCard {
    let mut mode_icons = Gamemode::ALL
        .iter()
        .copied()
        .filter(|mode| mode.valid(map))
        .map(|mode| MapListModeIcon {
            mode: Some(mode),
            icon: format!("mode{}Small", capitalize(mode.wire_name())),
            size: MAP_LIST_CARD_MODE_ICON_SIZE,
            pad: MAP_LIST_CARD_MODE_ICON_PAD,
        })
        .collect::<Vec<_>>();
    if mode_icons.is_empty() {
        mode_icons.push(MapListModeIcon {
            mode: None,
            icon: String::new(),
            size: MAP_LIST_CARD_MODE_ICON_SIZE,
            pad: MAP_LIST_CARD_MODE_ICON_PAD,
        });
    }

    MapListCard {
        map_file: map.file.clone(),
        map_name: map.name().to_string(),
        plain_name: map.plain_name(),
        width: MAP_LIST_CARD_WIDTH,
        style: "Styles.grayt",
        bottom_aligned: true,
        pad: MAP_LIST_CARD_PAD,
        margin: MAP_LIST_CARD_MARGIN,
        row_start: index % max_width == 0,
        mode_icons,
        title_width: MAP_LIST_CARD_TITLE_WIDTH,
        title_centered: true,
        title_ellipsis: true,
        divider_color: MAP_LIST_CARD_DIVIDER_COLOR,
        divider_pad: MAP_LIST_CARD_DIVIDER_PAD,
        preview_scaling: "Scaling.fit",
        preview_size: MAP_LIST_CARD_PREVIEW_SIZE,
        type_label: display_type.then(|| map_type_label(map)),
        type_color: MAP_LIST_CARD_TYPE_COLOR,
        type_pad_top: MAP_LIST_CARD_TYPE_PAD_TOP,
    }
}

fn map_type_label(map: &MapDescriptor) -> String {
    if map.custom {
        "@custom".into()
    } else if map.workshop {
        "@workshop".into()
    } else if let Some(mod_name) = map
        .mod_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        format!("[lightgray]{mod_name}")
    } else {
        "@builtin".into()
    }
}

fn bottom_buttons(display_type: bool, context: &MapListDialogContext) -> Vec<MapListBottomButton> {
    let portrait_wide = context.portrait && display_type;
    vec![MapListBottomButton {
        text: MAP_LIST_BACK_BUTTON_TEXT,
        icon: MAP_LIST_BACK_ICON,
        size: if portrait_wide {
            MAP_LIST_PORTRAIT_BACK_BUTTON_SIZE
        } else {
            MAP_LIST_BACK_BUTTON_SIZE
        },
        colspan: if portrait_wide { 2 } else { 1 },
        row_after: portrait_wide,
    }]
}

fn available_planets(planets: &[MapListPlanet]) -> Vec<String> {
    let mut out = planets
        .iter()
        .filter(|planet| planet.accessible)
        .map(|planet| planet.name.clone())
        .collect::<Vec<_>>();
    out.push(MAP_LIST_PLANET_ANY_NAME.into());
    out
}

fn planet_filter_options(
    planets: &[MapListPlanet],
    selected: &[String],
    locale: &str,
) -> Vec<MapListPlanetFilterOption> {
    let mut options = vec![MapListPlanetFilterOption {
        name: MAP_LIST_PLANET_ANY_NAME.into(),
        text: bundle_value(locale, "rules.anyenv"),
        icon: "planet".into(),
        style: MAP_LIST_PLANET_OPTION_STYLE,
        size: (
            MAP_LIST_PLANET_OPTION_SIZE.0 as i32,
            MAP_LIST_PLANET_OPTION_SIZE.1 as i32,
        ),
        margin_left: 12.0,
        checked: selected
            .iter()
            .any(|planet| planet == MAP_LIST_PLANET_ANY_NAME),
    }];
    options.extend(
        planets
            .iter()
            .filter(|planet| planet.accessible)
            .map(|planet| MapListPlanetFilterOption {
                name: planet.name.clone(),
                text: planet.localized_name.clone(),
                icon: planet.name.clone(),
                style: MAP_LIST_PLANET_OPTION_STYLE,
                size: (
                    MAP_LIST_PLANET_OPTION_SIZE.0 as i32,
                    MAP_LIST_PLANET_OPTION_SIZE.1 as i32,
                ),
                margin_left: 12.0,
                checked: selected.iter().any(|selected| selected == &planet.name),
            }),
    );
    options
}

fn bundle_value(locale: &str, key: &str) -> String {
    upstream_menu_bundle_value_for_locale(locale, key)
        .or_else(|| upstream_menu_bundle_value_for_locale("en", key))
        .unwrap_or(key)
        .to_string()
}

fn toggle_value<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if let Some(index) = values.iter().position(|selected| *selected == value) {
        values.remove(index);
    } else {
        values.push(value);
    }
}

fn toggle_string(values: &mut Vec<String>, value: &str) {
    if let Some(index) = values.iter().position(|selected| selected == value) {
        values.remove(index);
    } else {
        values.push(value.to_string());
    }
}

fn stripped_mod_name(map: &MapDescriptor) -> String {
    map.mod_name
        .as_deref()
        .map(strip_colors)
        .unwrap_or_default()
}

fn strip_colors(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '[' {
            for next in chars.by_ref() {
                if next == ']' {
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn map(file: &str, name: &str, custom: bool) -> MapDescriptor {
        let mut tags = BTreeMap::new();
        tags.insert("name".into(), name.into());
        tags.insert("author".into(), "Anuke".into());
        tags.insert("description".into(), "A test map".into());
        let mut map = MapDescriptor::new(file, 100, 100, tags, custom, 11, 157);
        map.spawns = 1;
        map
    }

    fn attack_map(file: &str, name: &str, planet: &str) -> MapDescriptor {
        let mut map = map(file, name, false);
        map.spawns = 0;
        map.teams = vec![1, 2];
        map.tags
            .insert("rules".into(), format!("{{planet:\"{planet}\"}}"));
        map
    }

    fn maps() -> MapListMaps {
        let custom = map("custom/zeta.msav", "[accent]Zeta[]", true);
        let builtin = attack_map("builtin/alpha.msav", "Alpha", "serpulo");
        let mut modded = map("mods/beta.msav", "Beta", false);
        modded.mod_name = Some("[scarlet]Modded[] Pack".into());
        MapListMaps {
            custom: vec![custom],
            builtin: vec![builtin],
            modded: vec![modded],
        }
    }

    fn planets() -> Vec<MapListPlanet> {
        vec![
            MapListPlanet::new("serpulo", "Serpulo"),
            MapListPlanet::new("erekir", "Erekir"),
            MapListPlanet {
                name: "hidden".into(),
                localized_name: "Hidden".into(),
                accessible: false,
            },
        ]
    }

    #[test]
    fn setup_builds_buttons_search_pane_available_planets_and_resets_search() {
        let mut dialog = MapListDialog::new("@customgame", true);
        dialog.search_string = Some("old".into());
        let model = dialog.setup(
            &maps(),
            &planets(),
            &MapListDialogContext {
                portrait: true,
                graphics_width: 460,
                desktop: true,
                mobile: false,
            },
        );

        assert_eq!(dialog.search_string, None);
        assert_eq!(model.available_planets, vec!["serpulo", "erekir", "sun"]);
        assert_eq!(
            model.bottom_buttons,
            vec![MapListBottomButton {
                text: "@back",
                icon: "left",
                size: (420.0, 64.0),
                colspan: 2,
                row_after: true,
            }]
        );
        assert_eq!(model.search.icon, "zoom");
        assert_eq!(model.search.message_text, "@editor.search");
        assert_eq!(model.search.max_length, 50);
        assert_eq!(model.search.filter_tooltip, "@editor.filters");
        assert!(!model.pane.fade_scroll_bars);
        assert!(model.pane.scrolling_disabled_x);
        assert_eq!(model.pane.pad_left, 28.0);
        assert_eq!(model.maps.max_width, 2);
    }

    #[test]
    fn rebuild_maps_filters_by_modes_planets_type_and_search_scopes() {
        let mut dialog = MapListDialog::new("@customgame", true);
        dialog.setup(&maps(), &planets(), &MapListDialogContext::default());
        dialog.modes = vec![Gamemode::Attack];
        dialog.planets = vec!["erekir".into(), "missing".into()];

        let model = dialog.model(&maps(), &MapListDialogContext::default());

        assert_eq!(model.maps.cards.len(), 1);
        assert_eq!(model.maps.cards[0].map_name, "Alpha");

        dialog.search_changed_plan("anuke");
        assert!(dialog
            .model(&maps(), &MapListDialogContext::default())
            .maps
            .cards
            .is_empty());
        dialog.search_author = true;
        assert_eq!(
            dialog
                .model(&maps(), &MapListDialogContext::default())
                .maps
                .cards[0]
                .map_name,
            "Alpha"
        );

        dialog.show_builtin = false;
        assert_eq!(
            dialog
                .model(&maps(), &MapListDialogContext::default())
                .maps
                .empty_text,
            Some("@maps.none")
        );
    }

    #[test]
    fn sorting_matches_plain_name_custom_priority_and_modded_priority_branches() {
        let mut maps = maps();
        maps.custom.push(map("custom/aa.msav", "Aardvark", true));
        maps.builtin.push(map("builtin/mm.msav", "Middle", false));
        let mut zmod = map("mods/zz.msav", "Zoo", false);
        zmod.mod_name = Some("Another Mod".into());
        maps.modded.push(zmod);
        let mut dialog = MapListDialog::new("@maps", true);
        dialog.setup(&maps, &planets(), &MapListDialogContext::default());

        assert_eq!(
            filtered_maps(&dialog, &maps)
                .iter()
                .map(|map| map.plain_name())
                .collect::<Vec<_>>(),
            vec!["Aardvark", "Alpha", "Beta", "Middle", "Zeta", "Zoo"]
        );

        dialog.prioritize_custom = true;
        assert_eq!(
            filtered_maps(&dialog, &maps)
                .iter()
                .take(2)
                .map(|map| map.plain_name())
                .collect::<Vec<_>>(),
            vec!["Aardvark", "Zeta"]
        );

        dialog.prioritize_custom = false;
        dialog.prioritize_modded = true;
        assert_eq!(
            filtered_maps(&dialog, &maps)
                .iter()
                .take(2)
                .map(|map| stripped_mod_name(map))
                .collect::<Vec<_>>(),
            vec!["Another Mod", "Modded Pack"]
        );
    }

    #[test]
    fn cards_include_mode_icons_preview_type_footer_and_row_starts() {
        let mut dialog = MapListDialog::new("@maps", true);
        let model = dialog.setup(
            &maps(),
            &planets(),
            &MapListDialogContext {
                graphics_width: 460,
                ..MapListDialogContext::default()
            },
        );

        let first = &model.maps.cards[0];
        assert!(first.row_start);
        assert_eq!(first.width, 200.0);
        assert_eq!(first.style, "Styles.grayt");
        assert_eq!(first.pad, 8.0);
        assert_eq!(first.margin, 9.0);
        assert_eq!(first.title_width, 182.0);
        assert!(first.title_centered);
        assert!(first.title_ellipsis);
        assert_eq!(first.preview_scaling, "Scaling.fit");
        assert_eq!(first.preview_size, 180.0);
        assert_eq!(first.type_color, "Color.gray");
        assert!(
            first
                .mode_icons
                .iter()
                .any(|icon| icon.icon == "modeAttackSmall")
                || first
                    .mode_icons
                    .iter()
                    .any(|icon| icon.icon == "modeSurvivalSmall")
        );

        let labels = model
            .maps
            .cards
            .iter()
            .map(|card| card.type_label.clone().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            vec!["@builtin", "[lightgray][scarlet]Modded[] Pack", "@custom"]
        );
    }

    #[test]
    fn filter_dialog_model_exposes_modes_priorities_type_search_and_planet_button() {
        let mut dialog = MapListDialog::new("@maps", true);
        dialog.setup(&maps(), &planets(), &MapListDialogContext::default());
        dialog.modes.push(Gamemode::Survival);
        dialog.show_custom = false;
        dialog.prioritize_custom = true;
        dialog.planets.push("serpulo".into());
        dialog.search_mod_name = true;

        let model = dialog.filters_model();

        assert_eq!(model.title, "@editor.filters");
        assert!(model.close_button_added);
        assert_eq!(model.mode_section, "@editor.filters.mode");
        assert_eq!(model.mode_buttons.len(), 5);
        assert_eq!(model.mode_buttons[0].icon, "modeSurvival");
        assert!(model.mode_buttons[0].checked);
        assert_eq!(model.priority_buttons[0].kind, MapListPriorityKind::Custom);
        assert!(model.priority_buttons[0].disabled);
        assert!(!model.priority_buttons[0].checked);
        assert!(model.planet_button.checked);
        assert_eq!(
            model
                .type_buttons
                .iter()
                .map(|button| button.checked)
                .collect::<Vec<_>>(),
            vec![false, true, true]
        );
        assert_eq!(
            model
                .search_buttons
                .iter()
                .map(|button| (button.kind, button.checked))
                .collect::<Vec<_>>(),
            vec![
                (MapListToggleKind::SearchAuthor, false),
                (MapListToggleKind::SearchDescription, false),
                (MapListToggleKind::SearchModName, true),
            ]
        );
    }

    #[test]
    fn planet_filter_options_include_any_environment_first_and_only_accessible_planets() {
        let mut dialog = MapListDialog::new("@maps", true);
        dialog.setup(&maps(), &planets(), &MapListDialogContext::default());
        dialog.planets = vec!["sun".into(), "erekir".into()];

        let model = dialog.planet_filters_model(&planets(), "en");

        assert_eq!(model.title, "@editor.filters.planetselect");
        assert_eq!(
            model
                .options
                .iter()
                .map(|option| option.name.as_str())
                .collect::<Vec<_>>(),
            vec!["sun", "serpulo", "erekir"]
        );
        assert_eq!(model.options[0].text, "<Any>");
        assert_eq!(model.options[0].icon, "planet");
        assert!(model.options[0].checked);
        assert!(!model.options[1].checked);
        assert!(model.options[2].checked);
        assert_eq!(model.options[2].size, (300, 60));
    }

    #[test]
    fn dispatch_filter_actions_mutate_state_persist_settings_and_rebuild() {
        let mut dialog = MapListDialog::new("@maps", true);
        dialog.setup(&maps(), &planets(), &MapListDialogContext::default());

        assert_eq!(
            dialog.dispatch_filter_action(
                MapListFilterAction::ToggleMode(Gamemode::Attack),
                &planets()
            ),
            vec![MapListDialogAction::RebuildMaps]
        );
        assert_eq!(dialog.modes, vec![Gamemode::Attack]);

        assert_eq!(
            dialog.dispatch_filter_action(MapListFilterAction::ToggleShowCustom, &planets()),
            vec![
                MapListDialogAction::PersistBool {
                    key: MAP_LIST_SETTING_SHOW_CUSTOM,
                    value: false,
                },
                MapListDialogAction::PersistBool {
                    key: MAP_LIST_SETTING_PRIORITIZE_CUSTOM,
                    value: false,
                },
                MapListDialogAction::RebuildMaps,
            ]
        );
        assert!(!dialog.show_custom);
        assert!(!dialog.prioritize_custom);

        dialog.dispatch_filter_action(MapListFilterAction::TogglePlanet(1), &planets());
        assert_eq!(dialog.planets, vec!["serpulo"]);
        let actions = dialog.dispatch_filter_action(MapListFilterAction::ClearPlanets, &planets());
        assert_eq!(dialog.planets, Vec::<String>::new());
        assert_eq!(
            actions,
            vec![
                MapListDialogAction::PersistPlanets {
                    key: MAP_LIST_SETTING_FILTER_PLANETS,
                    planets: Vec::new(),
                },
                MapListDialogAction::RebuildMaps,
            ]
        );
    }

    #[test]
    fn priority_toggles_are_mutually_exclusive_and_disabled_when_source_hidden() {
        let mut dialog = MapListDialog::new("@maps", true);
        dialog.setup(&maps(), &planets(), &MapListDialogContext::default());

        let actions =
            dialog.dispatch_filter_action(MapListFilterAction::TogglePrioritizeCustom, &planets());
        assert!(dialog.prioritize_custom);
        assert!(!dialog.prioritize_modded);
        assert_eq!(
            actions,
            vec![
                MapListDialogAction::PersistBool {
                    key: MAP_LIST_SETTING_PRIORITIZE_CUSTOM,
                    value: true,
                },
                MapListDialogAction::PersistBool {
                    key: MAP_LIST_SETTING_PRIORITIZE_MODDED,
                    value: false,
                },
                MapListDialogAction::RebuildMaps,
            ]
        );

        dialog.dispatch_filter_action(MapListFilterAction::TogglePrioritizeModded, &planets());
        assert!(!dialog.prioritize_custom);
        assert!(dialog.prioritize_modded);

        dialog.show_modded = false;
        let actions =
            dialog.dispatch_filter_action(MapListFilterAction::TogglePrioritizeModded, &planets());
        assert!(dialog.prioritize_modded);
        assert_eq!(actions, vec![MapListDialogAction::RebuildMaps]);
    }

    #[test]
    fn search_resize_show_and_map_click_plans_match_java_lifecycle() {
        let mut dialog = MapListDialog::new("@maps", true);
        let model = dialog.setup(&maps(), &planets(), &MapListDialogContext::default());
        assert_eq!(
            MapListDialog::show_map_plan(&model.maps.cards[0]),
            MapListDialogAction::ShowMap {
                map_file: "builtin/alpha.msav".into(),
                map_name: "Alpha".into(),
            }
        );
        assert_eq!(
            dialog.search_changed_plan("AL"),
            vec![MapListDialogAction::RebuildMaps]
        );
        assert_eq!(dialog.search_string, Some("al".into()));
        assert_eq!(
            dialog.show_plan(&MapListDialogContext {
                desktop: true,
                ..MapListDialogContext::default()
            }),
            vec![
                MapListDialogAction::ShowDialog,
                MapListDialogAction::FocusSearch
            ]
        );
        assert_eq!(
            MapListDialog::back_plan(),
            vec![MapListDialogAction::HideDialog]
        );

        dialog.show_map_filters_plan();
        let (actions, resized) =
            dialog.on_resize(&maps(), &planets(), &MapListDialogContext::default());
        assert_eq!(
            actions,
            vec![
                MapListDialogAction::HideActiveDialog,
                MapListDialogAction::RebuildMaps,
            ]
        );
        assert!(!dialog.active_dialog_open);
        assert_eq!(resized.search.text, "");
    }
}
