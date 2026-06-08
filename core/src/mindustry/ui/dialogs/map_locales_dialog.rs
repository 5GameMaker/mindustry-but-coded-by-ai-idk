//! Map locales dialog shell mirroring upstream `mindustry.editor.MapLocalesDialog`.

use std::collections::BTreeMap;

use super::BaseDialog;
use crate::mindustry::{
    ctype::ContentType,
    game::TeamRegistry,
    graphics::Pal,
    r#type::MapLocales,
    ui::fonts::{UpstreamContentIcon, UPSTREAM_UI_ICON_GLYPHS},
};

pub const MAP_LOCALES_CARD_WIDTH: f32 = 400.0;
pub const MAP_LOCALES_LOCALE_ITEM_WIDTH: f32 = 200.0;
pub const MAP_LOCALES_LOCALE_EDIT_BUTTON_WIDTH: f32 = 50.0;
pub const MAP_LOCALES_LOCALE_DELETE_BUTTON_WIDTH: f32 = 50.0;
pub const MAP_LOCALES_LOCALE_ADD_BUTTON_WIDTH: f32 = 250.0;
pub const MAP_LOCALES_LOCALE_ADD_BUTTON_HEIGHT: f32 = 50.0;
pub const MAP_LOCALES_MAIN_PROPERTY_COLLAPSE_BUTTON_SIZE: f32 = 35.0;
pub const MAP_LOCALES_MAIN_PROPERTY_FIELD_WIDTH: f32 = MAP_LOCALES_CARD_WIDTH - 125.0;
pub const MAP_LOCALES_MAIN_PROPERTY_REMOVE_BUTTON_SIZE: f32 = 35.0;
pub const MAP_LOCALES_MAIN_PROPERTY_EDIT_BUTTON_SIZE: f32 = 35.0;
pub const MAP_LOCALES_MAIN_PROPERTY_VALUE_HEIGHT: f32 = 140.0;
pub const MAP_LOCALES_PROPERTY_VIEW_ADD_BUTTON_WIDTH: f32 = 160.0;
pub const MAP_LOCALES_PROPERTY_VIEW_ADD_BUTTON_HEIGHT: f32 = 50.0;
pub const MAP_LOCALES_PROPERTY_VIEW_VALUE_HEIGHT: f32 = 140.0;
pub const MAP_LOCALES_ICON_BUTTON_SIZE: f32 = 48.0;
pub const MAP_LOCALES_ICON_CELL_WIDTH: f32 = 52.0;
pub const MAP_LOCALES_ICON_MAX_COLUMNS: usize = 20;
pub const MAP_LOCALES_FILTER_BUTTON_WIDTH: f32 = 450.0;
pub const MAP_LOCALES_FILTER_BUTTON_HEIGHT: f32 = 100.0;
pub const MAP_LOCALES_FILTER_BUTTON_PAD_TOP: f32 = 65.0;
pub const MAP_LOCALES_FILTER_STYLE_NAME: &str = "filterStyle";
pub const MAP_LOCALES_CONTENT_ICON_TYPES: [ContentType; 5] = [
    ContentType::Item,
    ContentType::Block,
    ContentType::Liquid,
    ContentType::Status,
    ContentType::Unit,
];
pub const MAP_LOCALES_MISSING_PLACEHOLDER: &str = "moai";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapLocalesDialogFilterStyle {
    pub java_name: &'static str,
    pub font_color: &'static str,
    pub over_font_color: &'static str,
    pub disabled_font_color: &'static str,
    pub disabled_drawable: &'static str,
}

pub const MAP_LOCALES_FILTER_STYLE: MapLocalesDialogFilterStyle = MapLocalesDialogFilterStyle {
    java_name: MAP_LOCALES_FILTER_STYLE_NAME,
    font_color: "lightGray",
    over_font_color: "accent",
    disabled_font_color: "gray",
    disabled_drawable: "black",
};

#[derive(Debug, Clone, PartialEq)]
pub struct MapLocalesDialogFilterButton {
    pub label: &'static str,
    pub checked: bool,
    pub style: MapLocalesDialogFilterStyle,
    pub width: f32,
    pub height: f32,
    pub pad_top: f32,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapLocalesDialogPropertyStatus {
    Correct,
    Missing,
    Same,
}

impl MapLocalesDialogPropertyStatus {
    pub fn color_rgba(self) -> [f32; 4] {
        match self {
            Self::Correct => [Pal::GRAY.r, Pal::GRAY.g, Pal::GRAY.b, Pal::GRAY.a],
            Self::Missing => [Pal::ACCENT.r, Pal::ACCENT.g, Pal::ACCENT.b, Pal::ACCENT.a],
            Self::Same => [
                Pal::TECH_BLUE.r,
                Pal::TECH_BLUE.g,
                Pal::TECH_BLUE.b,
                Pal::TECH_BLUE.a,
            ],
        }
    }

    pub fn color_name(self) -> &'static str {
        match self {
            Self::Correct => "gray",
            Self::Missing => "accent",
            Self::Same => "techBlue",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapLocalesDialogLocaleRow {
    pub locale: String,
    pub display_name: String,
    pub is_selected: bool,
    pub item_width: f32,
    pub edit_button_width: f32,
    pub delete_button_width: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapLocalesDialogLocaleAddRow {
    pub label: &'static str,
    pub button_width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapLocalesDialogLocaleEntry {
    Locale(MapLocalesDialogLocaleRow),
    Add(MapLocalesDialogLocaleAddRow),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapLocalesDialogMainCard {
    pub key: String,
    pub value: String,
    pub status: MapLocalesDialogPropertyStatus,
    pub color: [f32; 4],
    pub card_width: f32,
    pub collapse_button_size: f32,
    pub field_width: f32,
    pub remove_button_size: f32,
    pub edit_button_size: f32,
    pub value_area_height: f32,
    pub expanded: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapLocalesDialogPropertyViewCard {
    pub locale: String,
    pub display_name: String,
    pub value: Option<String>,
    pub status: MapLocalesDialogPropertyStatus,
    pub color: [f32; 4],
    pub card_width: f32,
    pub add_button_width: Option<f32>,
    pub add_button_height: Option<f32>,
    pub value_area_height: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapLocalesDialogIconSectionKind {
    Iconc,
    Content(ContentType),
    Team,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapLocalesDialogIconCandidate {
    pub section: MapLocalesDialogIconSectionKind,
    pub name: String,
    pub tooltip: String,
    pub output: String,
    pub drawable_symbol: Option<String>,
    pub button_size: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapLocalesDialogContentIconSource {
    pub content_type: ContentType,
    pub name: String,
    pub localized_name: String,
    pub output: String,
    pub drawable_symbol: String,
    pub ui_icon_found: bool,
}

impl MapLocalesDialogContentIconSource {
    pub fn from_upstream_content_icon(
        content_type: ContentType,
        localized_name: impl Into<String>,
        icon: &UpstreamContentIcon,
        ui_icon_found: bool,
    ) -> Self {
        Self {
            content_type,
            name: icon.name.clone(),
            localized_name: localized_name.into(),
            output: icon.emoji_string().unwrap_or_default(),
            drawable_symbol: icon.atlas_symbol.clone(),
            ui_icon_found,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapLocalesDialogTeamIconSource {
    pub name: String,
    pub localized_name: String,
    pub output: String,
    pub drawable_symbol: String,
    pub atlas_found: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapLocalesDialog {
    pub base: BaseDialog,
    pub locales: MapLocales,
    pub last_saved: MapLocales,
    pub selected_locale: String,
    pub settings_locale: String,
    pub saved: bool,
    pub apply_to_all: bool,
    pub collapsed: bool,
    pub search_string: String,
    pub search_by_value: bool,
    pub show_correct: bool,
    pub show_missing: bool,
    pub show_same: bool,
}

impl Default for MapLocalesDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl MapLocalesDialog {
    pub fn new() -> Self {
        Self {
            base: BaseDialog::new("@editor.locales"),
            locales: MapLocales::new(),
            last_saved: MapLocales::new(),
            selected_locale: MapLocales::current_locale(),
            settings_locale: MapLocales::current_locale(),
            saved: true,
            apply_to_all: true,
            collapsed: false,
            search_string: String::new(),
            search_by_value: false,
            show_correct: true,
            show_missing: true,
            show_same: true,
        }
    }

    pub fn new_for_game_setting(locale: &str) -> Self {
        let mut dialog = Self::new();
        dialog.settings_locale = locale.to_string();
        dialog.selected_locale = MapLocales::current_locale_from_game_setting(locale);
        dialog
    }

    /// Mirrors Java `show(MapLocales locales)`: load data and snapshot the
    /// saved state. The row-building helpers materialize the selected locale on
    /// demand, matching `buildTables()`.
    pub fn show(&mut self, locales: MapLocales) {
        self.locales = locales;
        self.last_saved = self.locales.copy();
        self.saved = true;
    }

    pub fn is_dirty(&self) -> bool {
        !self.saved
    }

    pub fn apply_changes(&mut self) {
        self.last_saved = self.locales.copy();
        self.saved = true;
    }

    pub fn rollback_all(&mut self) {
        self.locales = self.last_saved.copy();
        self.saved = true;
        self.ensure_selected_locale_present();
    }

    pub fn rollback_locale(&mut self, locale: &str) -> bool {
        let Some(snapshot) = self.last_saved.locales.get(locale).cloned() else {
            return false;
        };
        self.locales.locales.insert(locale.to_string(), snapshot);
        self.saved = self.locales == self.last_saved;
        self.ensure_selected_locale_present();
        true
    }

    pub fn set_search_string(&mut self, search: impl Into<String>) {
        self.search_string = search.into();
    }

    pub fn clear_search(&mut self) {
        self.search_string.clear();
    }

    pub fn set_search_by_value(&mut self, by_value: bool) {
        self.search_by_value = by_value;
    }

    pub fn set_filter_flags(&mut self, show_correct: bool, show_missing: bool, show_same: bool) {
        self.show_correct = show_correct;
        self.show_missing = show_missing;
        self.show_same = show_same;
    }

    pub fn set_apply_to_all(&mut self, apply_to_all: bool) {
        self.apply_to_all = apply_to_all;
    }

    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.collapsed = collapsed;
    }

    pub fn main_search_message_key(&self) -> &'static str {
        if self.search_by_value {
            "@locales.searchvalue"
        } else {
            "@locales.searchname"
        }
    }

    pub fn property_view_search_message_key(&self) -> &'static str {
        if self.search_by_value {
            "@locales.searchvalue"
        } else {
            "@locales.searchlocale"
        }
    }

    pub fn filter_buttons_like_java(&self) -> [MapLocalesDialogFilterButton; 3] {
        [
            MapLocalesDialogFilterButton {
                label: "@locales.showcorrect",
                checked: self.show_correct,
                style: MAP_LOCALES_FILTER_STYLE,
                width: MAP_LOCALES_FILTER_BUTTON_WIDTH,
                height: MAP_LOCALES_FILTER_BUTTON_HEIGHT,
                pad_top: MAP_LOCALES_FILTER_BUTTON_PAD_TOP,
                color: [Pal::ACCENT.r, Pal::ACCENT.g, Pal::ACCENT.b, Pal::ACCENT.a],
            },
            MapLocalesDialogFilterButton {
                label: "@locales.showmissing",
                checked: self.show_missing,
                style: MAP_LOCALES_FILTER_STYLE,
                width: MAP_LOCALES_FILTER_BUTTON_WIDTH,
                height: MAP_LOCALES_FILTER_BUTTON_HEIGHT,
                pad_top: MAP_LOCALES_FILTER_BUTTON_PAD_TOP,
                color: [Pal::ACCENT.r, Pal::ACCENT.g, Pal::ACCENT.b, Pal::ACCENT.a],
            },
            MapLocalesDialogFilterButton {
                label: "@locales.showsame",
                checked: self.show_same,
                style: MAP_LOCALES_FILTER_STYLE,
                width: MAP_LOCALES_FILTER_BUTTON_WIDTH,
                height: MAP_LOCALES_FILTER_BUTTON_HEIGHT,
                pad_top: MAP_LOCALES_FILTER_BUTTON_PAD_TOP,
                color: [Pal::ACCENT.r, Pal::ACCENT.g, Pal::ACCENT.b, Pal::ACCENT.a],
            },
        ]
    }

    pub fn main_column_count(screen_width: f32, scale: f32) -> usize {
        Self::column_count_with_offsets(screen_width, scale, 410.0, true)
    }

    pub fn property_view_column_count(screen_width: f32, scale: f32) -> usize {
        Self::column_count_with_offsets(screen_width, scale, 100.0, false)
    }

    pub fn add_icon_column_count(screen_width: f32, scale: f32) -> usize {
        if !screen_width.is_finite() || !scale.is_finite() || scale <= 0.0 {
            return 1;
        }

        ((screen_width / (MAP_LOCALES_ICON_CELL_WIDTH * scale)) as usize)
            .min(MAP_LOCALES_ICON_MAX_COLUMNS)
            .max(1)
    }

    pub fn add_icon_search_key(search: &str) -> String {
        search.replace(' ', "").to_lowercase()
    }

    pub fn team_icon_sources_from_registry<F, G>(
        teams: &TeamRegistry,
        mut localized_name: F,
        mut atlas_has: G,
    ) -> Vec<MapLocalesDialogTeamIconSource>
    where
        F: FnMut(&str) -> String,
        G: FnMut(&str) -> bool,
    {
        teams
            .base_teams()
            .iter()
            .map(|team| {
                let drawable_symbol = format!("team-{}", team.name);
                MapLocalesDialogTeamIconSource {
                    name: team.name.clone(),
                    localized_name: localized_name(&team.name),
                    output: team.emoji.clone(),
                    atlas_found: atlas_has(&drawable_symbol),
                    drawable_symbol,
                }
            })
            .collect()
    }

    pub fn add_icon_dialog_candidates(
        search: &str,
        content_icons: &[MapLocalesDialogContentIconSource],
        team_icons: &[MapLocalesDialogTeamIconSource],
    ) -> Vec<MapLocalesDialogIconCandidate> {
        let search = Self::add_icon_search_key(search);
        let mut out = Vec::new();

        for glyph in UPSTREAM_UI_ICON_GLYPHS {
            if !glyph.java_name.to_lowercase().contains(&search) {
                continue;
            }
            if let Some(output) = glyph.glyph_string() {
                out.push(MapLocalesDialogIconCandidate {
                    section: MapLocalesDialogIconSectionKind::Iconc,
                    name: glyph.java_name.to_string(),
                    tooltip: glyph.java_name.to_string(),
                    output,
                    drawable_symbol: Some(glyph.java_name.to_string()),
                    button_size: MAP_LOCALES_ICON_BUTTON_SIZE,
                });
            }
        }

        for content_type in MAP_LOCALES_CONTENT_ICON_TYPES {
            for icon in content_icons
                .iter()
                .filter(|icon| icon.content_type == content_type)
            {
                if !icon.ui_icon_found {
                    continue;
                }
                if !Self::add_icon_search_key(&icon.localized_name).contains(&search) {
                    continue;
                }
                out.push(MapLocalesDialogIconCandidate {
                    section: MapLocalesDialogIconSectionKind::Content(content_type),
                    name: icon.name.clone(),
                    tooltip: icon.localized_name.clone(),
                    output: icon.output.clone(),
                    drawable_symbol: Some(icon.drawable_symbol.clone()),
                    button_size: MAP_LOCALES_ICON_BUTTON_SIZE,
                });
            }
        }

        for team in team_icons {
            if !team.atlas_found {
                continue;
            }
            if !team.localized_name.to_lowercase().contains(&search) {
                continue;
            }
            out.push(MapLocalesDialogIconCandidate {
                section: MapLocalesDialogIconSectionKind::Team,
                name: team.name.clone(),
                tooltip: team.localized_name.clone(),
                output: team.output.clone(),
                drawable_symbol: Some(team.drawable_symbol.clone()),
                button_size: MAP_LOCALES_ICON_BUTTON_SIZE,
            });
        }

        out
    }

    pub fn locale_rows<F>(&mut self, mut display_name: F) -> Vec<MapLocalesDialogLocaleEntry>
    where
        F: FnMut(&str) -> String,
    {
        self.ensure_selected_locale_present();
        let mut rows = Vec::new();
        for locale in self.locales.locale_codes() {
            rows.push(MapLocalesDialogLocaleEntry::Locale(
                MapLocalesDialogLocaleRow {
                    display_name: display_name(&locale),
                    is_selected: locale == self.selected_locale,
                    locale,
                    item_width: MAP_LOCALES_LOCALE_ITEM_WIDTH,
                    edit_button_width: MAP_LOCALES_LOCALE_EDIT_BUTTON_WIDTH,
                    delete_button_width: MAP_LOCALES_LOCALE_DELETE_BUTTON_WIDTH,
                },
            ));
        }
        rows.push(MapLocalesDialogLocaleEntry::Add(
            MapLocalesDialogLocaleAddRow {
                label: "@add",
                button_width: MAP_LOCALES_LOCALE_ADD_BUTTON_WIDTH,
                height: MAP_LOCALES_LOCALE_ADD_BUTTON_HEIGHT,
            },
        ));
        rows
    }

    pub fn main_cards(&mut self, screen_width: f32, scale: f32) -> Vec<MapLocalesDialogMainCard> {
        self.ensure_selected_locale_present();
        let Some(props) = self.selected_locale_map() else {
            return Vec::new();
        };

        let mut out = Vec::new();
        let needle = self.search_string.to_lowercase();
        for (key, value) in props {
            let comparison = if self.search_by_value {
                value.to_lowercase()
            } else {
                key.to_lowercase()
            };
            if !needle.is_empty() && !comparison.contains(&needle) {
                continue;
            }

            let status = self.property_status_for_main(key, value);
            if (status == MapLocalesDialogPropertyStatus::Correct && !self.show_correct)
                || (status == MapLocalesDialogPropertyStatus::Missing && !self.show_missing)
                || (status == MapLocalesDialogPropertyStatus::Same && !self.show_same)
            {
                continue;
            }

            out.push(MapLocalesDialogMainCard {
                key: key.clone(),
                value: value.clone(),
                status,
                color: status.color_rgba(),
                card_width: MAP_LOCALES_CARD_WIDTH,
                collapse_button_size: MAP_LOCALES_MAIN_PROPERTY_COLLAPSE_BUTTON_SIZE,
                field_width: MAP_LOCALES_MAIN_PROPERTY_FIELD_WIDTH,
                remove_button_size: MAP_LOCALES_MAIN_PROPERTY_REMOVE_BUTTON_SIZE,
                edit_button_size: MAP_LOCALES_MAIN_PROPERTY_EDIT_BUTTON_SIZE,
                value_area_height: MAP_LOCALES_MAIN_PROPERTY_VALUE_HEIGHT,
                expanded: !self.collapsed,
            });
        }

        let _ = screen_width;
        let _ = scale;
        out
    }

    pub fn property_view_cards<F>(
        &mut self,
        key: &str,
        screen_width: f32,
        scale: f32,
        mut display_name: F,
    ) -> Vec<MapLocalesDialogPropertyViewCard>
    where
        F: FnMut(&str) -> String,
    {
        self.ensure_selected_locale_present();
        let mut out = Vec::new();
        let needle = self.search_string.to_lowercase();

        for locale in self.locales.locale_codes() {
            let Some(values) = self.locales.locales.get(&locale) else {
                continue;
            };
            let value = values.get(key).cloned();
            let display_name_value = display_name(&locale);
            let status = self.property_status_for_view(key, &locale, value.as_deref());

            if (status == MapLocalesDialogPropertyStatus::Correct && !self.show_correct)
                || (status == MapLocalesDialogPropertyStatus::Missing && !self.show_missing)
                || (status == MapLocalesDialogPropertyStatus::Same && !self.show_same)
            {
                continue;
            }

            if status != MapLocalesDialogPropertyStatus::Missing {
                let comparison = if self.search_by_value {
                    value.as_deref().unwrap_or("").to_lowercase()
                } else {
                    display_name_value.to_lowercase()
                };
                if !needle.is_empty() && !comparison.contains(&needle) {
                    continue;
                }
            }

            out.push(MapLocalesDialogPropertyViewCard {
                locale: locale.clone(),
                display_name: display_name_value,
                value,
                status,
                color: status.color_rgba(),
                card_width: MAP_LOCALES_CARD_WIDTH,
                add_button_width: (status == MapLocalesDialogPropertyStatus::Missing)
                    .then_some(MAP_LOCALES_PROPERTY_VIEW_ADD_BUTTON_WIDTH),
                add_button_height: (status == MapLocalesDialogPropertyStatus::Missing)
                    .then_some(MAP_LOCALES_PROPERTY_VIEW_ADD_BUTTON_HEIGHT),
                value_area_height: MAP_LOCALES_PROPERTY_VIEW_VALUE_HEIGHT,
            });
        }

        let _ = screen_width;
        let _ = scale;
        out
    }

    pub fn property_status_for_main(
        &self,
        key: &str,
        value: &str,
    ) -> MapLocalesDialogPropertyStatus {
        for (locale, values) in &self.locales.locales {
            if locale == &self.selected_locale {
                continue;
            }
            match values.get(key) {
                None => return MapLocalesDialogPropertyStatus::Missing,
                Some(other) if other == value => return MapLocalesDialogPropertyStatus::Same,
                Some(_) => {}
            }
        }
        MapLocalesDialogPropertyStatus::Correct
    }

    pub fn property_status_for_view(
        &self,
        key: &str,
        locale: &str,
        value: Option<&str>,
    ) -> MapLocalesDialogPropertyStatus {
        let Some(value) = value else {
            return MapLocalesDialogPropertyStatus::Missing;
        };

        for (other_locale, values) in &self.locales.locales {
            if other_locale == locale {
                continue;
            }
            if let Some(other) = values.get(key) {
                if other == value {
                    return MapLocalesDialogPropertyStatus::Same;
                }
            }
        }

        MapLocalesDialogPropertyStatus::Correct
    }

    pub fn add_locale(&mut self, locale: impl Into<String>) -> bool {
        let locale = locale.into();
        if self.locales.locales.contains_key(&locale) {
            return false;
        }

        self.locales.locales.insert(locale.clone(), BTreeMap::new());
        self.selected_locale = locale;
        self.saved = false;
        true
    }

    pub fn delete_locale(&mut self, locale: &str) -> Option<BTreeMap<String, String>> {
        let removed = self.locales.locales.remove(locale)?;
        self.saved = false;

        if !self.locales.locales.contains_key(&self.selected_locale) {
            self.selected_locale = self
                .locales
                .locale_codes()
                .into_iter()
                .next()
                .unwrap_or_else(|| self.settings_locale.clone());
            self.ensure_selected_locale_present();
        }

        Some(removed)
    }

    pub fn add_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();

        if self.apply_to_all {
            for bundle in self.locales.locales.values_mut() {
                bundle.insert(key.clone(), value.clone());
            }
        } else {
            self.selected_locale_map_mut().insert(key, value);
        }
        self.saved = false;
    }

    pub fn rename_property(&mut self, old_key: &str, new_key: &str) -> bool {
        if old_key == new_key {
            return false;
        }
        if self
            .selected_locale_map()
            .is_some_and(|props| props.contains_key(new_key))
        {
            return false;
        }

        let mut changed = false;
        if self.apply_to_all {
            for bundle in self.locales.locales.values_mut() {
                if bundle.contains_key(old_key) && !bundle.contains_key(new_key) {
                    if let Some(value) = bundle.remove(old_key) {
                        bundle.insert(new_key.to_string(), value);
                        changed = true;
                    }
                }
            }
        } else if let Some(props) = self.locales.locales.get_mut(&self.selected_locale) {
            if let Some(value) = props.remove(old_key) {
                props.insert(new_key.to_string(), value);
                changed = true;
            }
        }

        if changed {
            self.saved = false;
        }

        changed
    }

    pub fn remove_property(&mut self, key: &str) -> bool {
        let mut changed = false;
        if self.apply_to_all {
            for bundle in self.locales.locales.values_mut() {
                changed |= bundle.remove(key).is_some();
            }
        } else if let Some(props) = self.locales.locales.get_mut(&self.selected_locale) {
            changed = props.remove(key).is_some();
        }

        if changed {
            self.saved = false;
        }

        changed
    }

    pub fn set_selected_locale_value(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.selected_locale_map_mut()
            .insert(key.into(), value.into());
        self.saved = false;
    }

    pub fn add_property_to_missing_locales_like_java(
        &mut self,
        key: &str,
        value: impl Into<String>,
    ) -> usize {
        let value = value.into();
        let mut added = 0;
        for bundle in self.locales.locales.values_mut() {
            if !bundle.contains_key(key) {
                bundle.insert(key.to_string(), value.clone());
                added += 1;
            }
        }
        if added > 0 {
            self.saved = false;
        }
        added
    }

    pub fn append_icon_to_selected_locale_property_like_java(
        &mut self,
        key: &str,
        icon: &str,
    ) -> bool {
        let Some(value) = self.selected_locale_map_mut().get_mut(key) else {
            return false;
        };
        value.push_str(icon);
        self.saved = false;
        true
    }

    pub fn can_rollback_selected_property_like_java(&self, key: &str) -> bool {
        let Some(saved_map) = self.last_saved.locales.get(&self.selected_locale) else {
            return false;
        };
        let Some(saved_value) = saved_map.get(key) else {
            return false;
        };
        self.locales
            .locales
            .get(&self.selected_locale)
            .and_then(|props| props.get(key))
            .is_some_and(|value| value != saved_value)
    }

    pub fn rollback_selected_property_like_java(&mut self, key: &str) -> bool {
        let Some(saved_value) = self
            .last_saved
            .locales
            .get(&self.selected_locale)
            .and_then(|props| props.get(key))
            .cloned()
        else {
            return false;
        };
        if !self.can_rollback_selected_property_like_java(key) {
            return false;
        }
        self.selected_locale_map_mut()
            .insert(key.to_string(), saved_value);
        true
    }

    pub fn set_property_value(
        &mut self,
        locale: &str,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> bool {
        let Some(props) = self.locales.locales.get_mut(locale) else {
            return false;
        };
        props.insert(key.into(), value.into());
        self.saved = false;
        true
    }

    pub fn add_missing_property(&mut self, locale: &str, key: impl Into<String>) -> bool {
        self.set_property_value(locale, key, MAP_LOCALES_MISSING_PLACEHOLDER)
    }

    pub fn export_locale_text(&self, locale: &str) -> Option<String> {
        let values = self.locales.locales.get(locale)?;
        let mut data = String::new();
        for (key, value) in values {
            data.push_str(key);
            data.push_str(" = ");
            data.push_str(&escape_java_locale_value(value));
            data.push('\n');
        }
        Some(data)
    }

    pub fn export_all_text(&self) -> String {
        let mut data = String::new();
        for locale in self.locales.locale_codes() {
            data.push_str(&locale);
            data.push_str(":\n");
            if let Some(text) = self.export_locale_text(&locale) {
                data.push_str(&text);
            }
        }
        data
    }

    pub fn import_locale_text(&mut self, locale: impl Into<String>, data: &str) {
        let locale = locale.into();
        let mut map = BTreeMap::new();
        for line in split_java_style_lines(data) {
            if let Some(sep_index) = line.find(" = ") {
                let key = line[..sep_index].to_string();
                let value = unescape_java_locale_value(&line[sep_index + 3..]);
                map.insert(key, value);
            }
        }
        self.locales.locales.insert(locale, map);
        self.saved = false;
    }

    pub fn import_all_text(&mut self, data: &str) {
        let mut bundles = MapLocales::new();
        let mut current_locale = String::new();

        for line in split_java_style_lines(data) {
            if line.ends_with(':') && !line.contains('=') {
                current_locale = line[..line.len() - 1].to_string();
                bundles
                    .locales
                    .insert(current_locale.clone(), BTreeMap::new());
            } else if let Some(sep_index) = line.find(" = ") {
                if !current_locale.is_empty() {
                    let value = unescape_java_locale_value(&line[sep_index + 3..]);
                    bundles
                        .locales
                        .entry(current_locale.clone())
                        .or_default()
                        .insert(line[..sep_index].to_string(), value);
                }
            }
        }

        self.locales = bundles;
        self.saved = false;
        self.ensure_selected_locale_present();
    }

    pub fn selected_locale_map(&self) -> Option<&BTreeMap<String, String>> {
        self.locales.locales.get(&self.selected_locale)
    }

    pub fn selected_locale_map_mut(&mut self) -> &mut BTreeMap<String, String> {
        self.ensure_selected_locale_present();
        self.locales
            .locales
            .get_mut(&self.selected_locale)
            .expect("selected locale must exist after ensure_selected_locale_present")
    }

    fn ensure_selected_locale_present(&mut self) {
        self.locales
            .locales
            .entry(self.selected_locale.clone())
            .or_insert_with(BTreeMap::new);
    }

    fn column_count_with_offsets(
        screen_width: f32,
        scale: f32,
        padding: f32,
        subtract_one: bool,
    ) -> usize {
        if !screen_width.is_finite() || !scale.is_finite() || scale <= 0.0 {
            return 1;
        }

        let scaled_width = screen_width / scale;
        let raw = ((scaled_width - padding) / MAP_LOCALES_CARD_WIDTH) as i32;
        let raw = if subtract_one { raw - 1 } else { raw };
        raw.max(1) as usize
    }
}

fn split_java_style_lines(data: &str) -> impl Iterator<Item = &str> {
    data.split(|ch| ch == '\n' || ch == '\r')
        .filter(|line| !line.is_empty())
}

fn escape_java_locale_value(value: &str) -> String {
    value.replace("\\n", "\\\\n").replace('\n', "\\n")
}

fn unescape_java_locale_value(value: &str) -> String {
    value.replace("\\n", "\n").replace("\\\n", "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_locales() -> MapLocales {
        let mut locales = MapLocales::new();
        locales.put_property("en", "alpha", "same");
        locales.put_property("en", "bravo", "only-en");
        locales.put_property("en", "charlie", "correct-en");
        locales.put_property("fr", "alpha", "same");
        locales.put_property("fr", "bravo", "value-fr");
        locales.put_property("fr", "delta", "fr-only");
        locales.put_property("zh", "alpha", "zh-value");
        locales.put_property("zh", "bravo", "value-zh");
        locales.put_property("zh", "charlie", "zh-different");
        locales
    }

    #[test]
    fn constructor_uses_java_title_and_default_filter_state() {
        let dialog = MapLocalesDialog::new();

        assert_eq!(dialog.base.title, "@editor.locales");
        assert_eq!(dialog.base.style, Default::default());
        assert_eq!(dialog.selected_locale, MapLocales::current_locale());
        assert_eq!(dialog.settings_locale, MapLocales::current_locale());
        assert!(dialog.saved);
        assert!(dialog.apply_to_all);
        assert!(!dialog.collapsed);
        assert_eq!(dialog.main_search_message_key(), "@locales.searchname");
        assert_eq!(
            dialog.property_view_search_message_key(),
            "@locales.searchlocale"
        );
    }

    #[test]
    fn constructor_for_game_setting_uses_settings_locale_like_java_current_locale() {
        let dialog = MapLocalesDialog::new_for_game_setting("zh_CN");

        assert_eq!(
            dialog.selected_locale, "zh_CN",
            "Java MapLocalesDialog selectedLocale comes from MapLocales.currentLocale(), which reads settings.locale before the OS locale"
        );
        assert_eq!(
            dialog.settings_locale, "zh_CN",
            "Java delete-locale fallback uses the raw Core.settings locale value, so Rust must keep it separate from selectedLocale"
        );
        assert_eq!(dialog.base.title, "@editor.locales");
        assert!(dialog.saved);
        assert!(dialog.apply_to_all);
    }

    #[test]
    fn delete_last_locale_falls_back_to_raw_settings_locale_like_java() {
        let mut dialog = MapLocalesDialog::new_for_game_setting("default");
        let mut locales = MapLocales::new();
        locales.put_property("en", "name", "Only Locale");
        dialog.show(locales);
        dialog.selected_locale = "en".into();

        let removed = dialog
            .delete_locale("en")
            .expect("locale should be removed");

        assert_eq!(removed["name"], "Only Locale");
        assert_eq!(
            dialog.selected_locale, "default",
            "Java MapLocalesDialog delete path falls back to Core.settings.getString(\"locale\"), preserving the raw default sentinel"
        );
        assert!(
            dialog.locales.locales.contains_key("default"),
            "buildTables() immediately materializes the raw fallback selectedLocale as an empty bundle"
        );
        assert!(dialog.is_dirty());

        let mut non_empty = MapLocalesDialog::new_for_game_setting("default");
        let mut locales = MapLocales::new();
        locales.put_property("en", "name", "English");
        locales.put_property("fr", "name", "French");
        non_empty.show(locales);
        non_empty.selected_locale = "fr".into();

        non_empty.delete_locale("fr");
        assert_eq!(
            non_empty.selected_locale, "en",
            "Java selects the first remaining locale instead of the raw settings fallback while the map still has locales"
        );
        assert!(!non_empty.locales.locales.contains_key("default"));
    }

    #[test]
    fn show_snapshots_locales_and_keeps_selected_locale_row_real() {
        let mut dialog = MapLocalesDialog::new();
        dialog.selected_locale = "it".into();

        let mut locales = MapLocales::new();
        locales.put_property("en", "hello", "world");

        dialog.show(locales.clone());

        assert_eq!(dialog.locales, locales);
        assert_eq!(dialog.last_saved, locales);
        assert!(dialog.saved);
        assert_eq!(dialog.selected_locale, "it");
    }

    #[test]
    fn locale_rows_include_java_width_semantics_and_add_row() {
        let mut dialog = MapLocalesDialog::new();
        dialog.show(sample_locales());
        dialog.selected_locale = "fr".into();

        let rows = dialog.locale_rows(|code| match code {
            "en" => "English".into(),
            "fr" => "Français".into(),
            "zh" => "中文".into(),
            other => other.into(),
        });

        assert_eq!(rows.len(), 4);
        match &rows[0] {
            MapLocalesDialogLocaleEntry::Locale(row) => {
                assert_eq!(row.locale, "en");
                assert_eq!(row.display_name, "English");
                assert!(!row.is_selected);
                assert_eq!(row.item_width, MAP_LOCALES_LOCALE_ITEM_WIDTH);
                assert_eq!(row.edit_button_width, MAP_LOCALES_LOCALE_EDIT_BUTTON_WIDTH);
                assert_eq!(
                    row.delete_button_width,
                    MAP_LOCALES_LOCALE_DELETE_BUTTON_WIDTH
                );
            }
            other => panic!("expected locale row, got {other:?}"),
        }
        match &rows[1] {
            MapLocalesDialogLocaleEntry::Locale(row) => {
                assert_eq!(row.locale, "fr");
                assert!(row.is_selected);
            }
            other => panic!("expected selected locale row, got {other:?}"),
        }
        match &rows[3] {
            MapLocalesDialogLocaleEntry::Add(row) => {
                assert_eq!(row.label, "@add");
                assert_eq!(row.button_width, MAP_LOCALES_LOCALE_ADD_BUTTON_WIDTH);
                assert_eq!(row.height, MAP_LOCALES_LOCALE_ADD_BUTTON_HEIGHT);
            }
            other => panic!("expected add row, got {other:?}"),
        }
    }

    #[test]
    fn main_column_formula_matches_java_offsets_and_card_width() {
        assert_eq!(MapLocalesDialog::main_column_count(800.0, 1.0), 1);
        assert_eq!(MapLocalesDialog::main_column_count(2400.0, 1.0), 3);
        assert_eq!(MapLocalesDialog::main_column_count(2399.0, 1.0), 3);
        assert_eq!(MapLocalesDialog::property_view_column_count(80.0, 1.0), 1);
        assert_eq!(MapLocalesDialog::property_view_column_count(900.0, 1.0), 2);
        assert_eq!(MapLocalesDialog::add_icon_column_count(1920.0, 1.0), 20);
        assert_eq!(MapLocalesDialog::add_icon_column_count(520.0, 1.0), 10);
        assert_eq!(MAP_LOCALES_CARD_WIDTH, 400.0);
        assert_eq!(MAP_LOCALES_ICON_BUTTON_SIZE, 48.0);
        assert_eq!(MAP_LOCALES_ICON_CELL_WIDTH, 52.0);
    }

    #[test]
    fn add_icon_dialog_candidates_follow_java_icons_table_sources_and_search() {
        let copper = UpstreamContentIcon {
            unicode: 0xf8ff,
            name: "copper".into(),
            atlas_symbol: "item-copper".into(),
        };
        let hidden = UpstreamContentIcon {
            unicode: 0xf8fe,
            name: "hidden-wall".into(),
            atlas_symbol: "block-hidden-wall".into(),
        };
        let dagger = UpstreamContentIcon {
            unicode: 0xf8fd,
            name: "dagger".into(),
            atlas_symbol: "unit-dagger".into(),
        };
        let content_icons = vec![
            MapLocalesDialogContentIconSource::from_upstream_content_icon(
                ContentType::Item,
                "Copper Ore",
                &copper,
                true,
            ),
            MapLocalesDialogContentIconSource::from_upstream_content_icon(
                ContentType::Block,
                "Hidden Wall",
                &hidden,
                false,
            ),
            MapLocalesDialogContentIconSource::from_upstream_content_icon(
                ContentType::Unit,
                "Dagger",
                &dagger,
                true,
            ),
        ];
        let mut teams = crate::mindustry::game::vanilla_teams();
        teams.base_teams_mut()[2].emoji = "⚔".into();
        let team_icons = MapLocalesDialog::team_icon_sources_from_registry(
            &teams,
            |name| match name {
                "crux" => "Crux".into(),
                other => other.into(),
            },
            |symbol| symbol == "team-crux",
        );

        let iconc = MapLocalesDialog::add_icon_dialog_candidates("right open", &[], &[]);
        assert!(iconc.iter().any(|candidate| {
            candidate.section == MapLocalesDialogIconSectionKind::Iconc
                && candidate.name == "rightOpen"
                && candidate.tooltip == "rightOpen"
                && candidate.button_size == MAP_LOCALES_ICON_BUTTON_SIZE
        }));

        let content =
            MapLocalesDialog::add_icon_dialog_candidates("copper ore", &content_icons, &[]);
        assert_eq!(content.len(), 1);
        assert_eq!(
            content[0].section,
            MapLocalesDialogIconSectionKind::Content(ContentType::Item)
        );
        assert_eq!(content[0].name, "copper");
        assert_eq!(content[0].tooltip, "Copper Ore");
        assert_eq!(content[0].output, "\u{f8ff}");
        assert_eq!(content[0].drawable_symbol.as_deref(), Some("item-copper"));

        let missing_hidden =
            MapLocalesDialog::add_icon_dialog_candidates("hidden wall", &content_icons, &[]);
        assert!(
            missing_hidden.is_empty(),
            "Java iconsTable filters content icons with u.uiIcon.found()"
        );

        let team = MapLocalesDialog::add_icon_dialog_candidates("crux", &[], &team_icons);
        assert_eq!(team.len(), 1);
        assert_eq!(team[0].section, MapLocalesDialogIconSectionKind::Team);
        assert_eq!(team[0].name, "crux");
        assert_eq!(team[0].output, "⚔");
        assert_eq!(team[0].drawable_symbol.as_deref(), Some("team-crux"));
    }

    #[test]
    fn filter_buttons_use_java_filter_style_and_metrics() {
        let mut dialog = MapLocalesDialog::new();
        dialog.set_filter_flags(true, false, true);
        let buttons = dialog.filter_buttons_like_java();

        assert_eq!(buttons[0].label, "@locales.showcorrect");
        assert_eq!(buttons[1].label, "@locales.showmissing");
        assert_eq!(buttons[2].label, "@locales.showsame");
        assert!(buttons[0].checked);
        assert!(!buttons[1].checked);
        assert!(buttons[2].checked);

        for button in buttons {
            assert_eq!(button.style, MAP_LOCALES_FILTER_STYLE);
            assert_eq!(button.style.java_name, "filterStyle");
            assert_eq!(button.style.font_color, "lightGray");
            assert_eq!(button.style.over_font_color, "accent");
            assert_eq!(button.style.disabled_font_color, "gray");
            assert_eq!(button.style.disabled_drawable, "black");
            assert_eq!(button.width, 450.0);
            assert_eq!(button.height, 100.0);
            assert_eq!(button.pad_top, 65.0);
            assert_eq!(
                button.color,
                [Pal::ACCENT.r, Pal::ACCENT.g, Pal::ACCENT.b, Pal::ACCENT.a]
            );
        }
    }

    #[test]
    fn main_cards_apply_search_status_and_color_semantics_like_java() {
        let mut dialog = MapLocalesDialog::new();
        dialog.show(sample_locales());
        dialog.selected_locale = "en".into();

        let all_cards = dialog.main_cards(1920.0, 1.0);
        let keys: Vec<_> = all_cards.iter().map(|card| card.key.as_str()).collect();
        assert_eq!(keys, vec!["alpha", "bravo", "charlie"]);

        let alpha = &all_cards[0];
        assert_eq!(alpha.status, MapLocalesDialogPropertyStatus::Same);
        assert_eq!(
            alpha.color,
            MapLocalesDialogPropertyStatus::Same.color_rgba()
        );
        assert_eq!(
            alpha.collapse_button_size,
            MAP_LOCALES_MAIN_PROPERTY_COLLAPSE_BUTTON_SIZE
        );
        assert_eq!(alpha.field_width, MAP_LOCALES_MAIN_PROPERTY_FIELD_WIDTH);
        assert_eq!(
            alpha.remove_button_size,
            MAP_LOCALES_MAIN_PROPERTY_REMOVE_BUTTON_SIZE
        );
        assert_eq!(
            alpha.edit_button_size,
            MAP_LOCALES_MAIN_PROPERTY_EDIT_BUTTON_SIZE
        );
        assert_eq!(
            alpha.value_area_height,
            MAP_LOCALES_MAIN_PROPERTY_VALUE_HEIGHT
        );
        assert!(alpha.expanded);

        let charlie = &all_cards[2];
        assert_eq!(charlie.status, MapLocalesDialogPropertyStatus::Missing);

        let mut filtered = dialog.clone();
        filtered.set_search_string("correct");
        filtered.set_search_by_value(true);
        let filtered_cards = filtered.main_cards(1920.0, 1.0);
        assert_eq!(filtered_cards.len(), 1);
        assert_eq!(filtered_cards[0].key, "charlie");

        let mut by_value = dialog.clone();
        by_value.set_search_string("only-en");
        by_value.set_search_by_value(true);
        let by_value_cards = by_value.main_cards(1920.0, 1.0);
        assert_eq!(by_value_cards.len(), 1);
        assert_eq!(by_value_cards[0].key, "bravo");

        let mut hidden = dialog.clone();
        hidden.set_filter_flags(false, true, false);
        let hidden_cards = hidden.main_cards(1920.0, 1.0);
        assert_eq!(hidden_cards.len(), 1);
        assert_eq!(
            hidden_cards[0].status,
            MapLocalesDialogPropertyStatus::Missing
        );
    }

    #[test]
    fn property_view_cards_match_missing_same_correct_semantics_and_search_rules() {
        let mut dialog = MapLocalesDialog::new();
        dialog.show(sample_locales());
        dialog.selected_locale = "en".into();

        let cards = dialog.property_view_cards("alpha", 1920.0, 1.0, |locale| match locale {
            "en" => "English".into(),
            "fr" => "Français".into(),
            "zh" => "中文".into(),
            other => other.into(),
        });

        assert_eq!(cards.len(), 3);
        assert_eq!(cards[0].locale, "en");
        assert_eq!(cards[0].status, MapLocalesDialogPropertyStatus::Same);
        assert_eq!(
            cards[0].color,
            MapLocalesDialogPropertyStatus::Same.color_rgba()
        );
        assert_eq!(cards[0].card_width, MAP_LOCALES_CARD_WIDTH);
        assert_eq!(
            cards[0].add_button_width, None,
            "same rows should expose the plain area, not the add button"
        );

        assert_eq!(cards[1].status, MapLocalesDialogPropertyStatus::Same);
        assert_eq!(cards[2].status, MapLocalesDialogPropertyStatus::Correct);

        let delta_cards =
            dialog.property_view_cards("delta", 1920.0, 1.0, |locale| locale.to_string());
        assert_eq!(delta_cards.len(), 3);
        let en_missing = delta_cards.iter().find(|card| card.locale == "en").unwrap();
        assert_eq!(en_missing.status, MapLocalesDialogPropertyStatus::Missing);
        assert_eq!(
            en_missing.add_button_width,
            Some(MAP_LOCALES_PROPERTY_VIEW_ADD_BUTTON_WIDTH)
        );
        assert_eq!(
            en_missing.add_button_height,
            Some(MAP_LOCALES_PROPERTY_VIEW_ADD_BUTTON_HEIGHT)
        );

        let mut by_name = dialog.clone();
        by_name.set_search_string("fran");
        let by_name_cards =
            by_name.property_view_cards("bravo", 1920.0, 1.0, |locale| match locale {
                "en" => "English".into(),
                "fr" => "Français".into(),
                "zh" => "中文".into(),
                other => other.into(),
            });
        assert!(by_name_cards.iter().all(|card| card.locale != "en"));

        let mut by_value = dialog.clone();
        by_value.set_search_string("value-zh");
        by_value.set_search_by_value(true);
        let by_value_cards =
            by_value.property_view_cards("bravo", 1920.0, 1.0, |locale| locale.to_string());
        assert_eq!(by_value_cards.len(), 1);
        assert_eq!(by_value_cards[0].locale, "zh");
    }

    #[test]
    fn apply_to_all_mutations_and_rollbacks_follow_java_semantics() {
        let mut dialog = MapLocalesDialog::new();
        dialog.show(sample_locales());
        dialog.selected_locale = "en".into();

        dialog.set_apply_to_all(true);
        assert!(dialog.rename_property("bravo", "beta"));
        assert!(dialog.locales.locales["en"].contains_key("beta"));
        assert!(dialog.locales.locales["fr"].contains_key("beta"));
        assert!(dialog.locales.locales["zh"].contains_key("beta"));
        assert!(!dialog.locales.locales["en"].contains_key("bravo"));
        assert!(dialog.is_dirty());

        dialog.add_property("shared", "same-value");
        assert_eq!(dialog.locales.locales["fr"]["shared"], "same-value");

        dialog.set_apply_to_all(false);
        dialog.set_selected_locale_value("en-only", "value");
        assert_eq!(dialog.locales.locales["en"]["en-only"], "value");
        assert!(!dialog.locales.locales["fr"].contains_key("en-only"));

        assert!(dialog.remove_property("en-only"));
        assert!(!dialog.locales.locales["en"].contains_key("en-only"));

        let exported = dialog.export_locale_text("en").unwrap();
        assert!(exported.contains("alpha = same"));
        assert!(exported.contains("beta = only-en"));

        let all_text = dialog.export_all_text();
        assert!(all_text.starts_with("en:\n"));
        assert!(all_text.contains("fr:\n"));

        let mut imported = MapLocalesDialog::new();
        imported.import_locale_text("en", "line = first\\nsecond\nslash = \\\n");
        assert_eq!(imported.locales.locales["en"]["line"], "first\nsecond");
        assert_eq!(imported.locales.locales["en"]["slash"], "\\");

        imported.import_all_text("en:\nname = hello\\nworld\nfr:\nname = bonjour\n");
        assert_eq!(imported.locales.locales["en"]["name"], "hello\nworld");
        assert_eq!(imported.locales.locales["fr"]["name"], "bonjour");

        imported.apply_changes();
        assert!(imported.saved);
        imported.locales.put_property("en", "changed", "x");
        imported.rollback_all();
        assert!(!imported.locales.locales["en"].contains_key("changed"));
    }

    #[test]
    fn property_edit_actions_match_java_addtoother_addicon_and_rollback() {
        let mut dialog = MapLocalesDialog::new();
        dialog.show(sample_locales());
        dialog.selected_locale = "en".into();

        let added = dialog.add_property_to_missing_locales_like_java("charlie", "correct-en");
        assert_eq!(added, 1);
        assert_eq!(dialog.locales.locales["fr"]["charlie"], "correct-en");
        assert_eq!(
            dialog.locales.locales["zh"]["charlie"], "zh-different",
            "Java @locales.addtoother only fills bundles that do not contain the key"
        );
        assert!(dialog.is_dirty());

        assert!(dialog.append_icon_to_selected_locale_property_like_java("charlie", "★"));
        assert_eq!(dialog.locales.locales["en"]["charlie"], "correct-en★");
        assert!(dialog.can_rollback_selected_property_like_java("charlie"));
        assert!(dialog.rollback_selected_property_like_java("charlie"));
        assert_eq!(dialog.locales.locales["en"]["charlie"], "correct-en");
        assert!(!dialog.can_rollback_selected_property_like_java("charlie"));
        assert!(
            dialog.is_dirty(),
            "Java propEditDialog rollback restores the value but does not flip saved=true; only full editDialog rollback does"
        );

        assert!(!dialog.append_icon_to_selected_locale_property_like_java("missing", "★"));
        assert!(!dialog.rollback_selected_property_like_java("missing"));
    }
}
