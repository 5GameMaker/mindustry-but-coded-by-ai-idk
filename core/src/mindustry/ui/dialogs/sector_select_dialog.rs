//! Sector-select dialog model mirroring upstream `mindustry.ui.dialogs.SectorSelectDialog`.

use crate::mindustry::{ctype::ContentId, r#type::SectorPreset};

pub const SECTOR_SELECT_DIALOG_TITLE: &str = "@database-category.sector";
pub const SECTOR_SELECT_SEARCH_ICON: &str = "zoom";
pub const SECTOR_SELECT_SEARCH_WIDTH: f32 = 300.0;
pub const SECTOR_SELECT_ROW_STYLE: &str = "Styles.grayt";
pub const SECTOR_SELECT_ROW_ICON_SIZE: f32 = 32.0;
pub const SECTOR_SELECT_ROW_WIDTH: f32 = 400.0;
pub const SECTOR_SELECT_ROW_HEIGHT: f32 = 50.0;
pub const SECTOR_SELECT_ROW_MARGIN: f32 = 4.0;
pub const SECTOR_SELECT_ROW_PAD: f32 = 3.0;

#[derive(Debug, Clone, PartialEq)]
pub struct SectorSelectDialogModel {
    pub title: &'static str,
    pub content_top_aligned: bool,
    pub search_icon: &'static str,
    pub search_width: f32,
    pub search_text: String,
    pub pane_grow: bool,
    pub pane_top_aligned: bool,
    pub close_button_added: bool,
    pub rows: Vec<SectorSelectRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectorSelectRow {
    pub id: ContentId,
    pub name: String,
    pub localized_name: String,
    pub planet_name: String,
    pub ui_icon: String,
    pub style: &'static str,
    pub icon_size: f32,
    pub size: (f32, f32),
    pub margin: f32,
    pub pad: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectorSelectDialogAction {
    ClearSearch,
    RequestKeyboard,
    PostRequestKeyboard,
    Rebuild,
    AcceptSectorPreset {
        id: ContentId,
        name: String,
        localized_name: String,
        planet_name: String,
    },
    HideDialog,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorSelectDialog {
    pub planet_name: String,
    pub search_text: String,
}

impl Default for SectorSelectDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SectorSelectDialog {
    pub fn new() -> Self {
        Self {
            planet_name: "serpulo".into(),
            search_text: String::new(),
        }
    }

    pub fn show(
        &mut self,
        planet_name: impl Into<String>,
        sectors: &[SectorPreset],
    ) -> SectorSelectDialogModel {
        self.planet_name = planet_name.into();
        self.search_text.clear();
        self.model(sectors)
    }

    pub fn shown(
        &mut self,
        sectors: &[SectorPreset],
    ) -> (Vec<SectorSelectDialogAction>, SectorSelectDialogModel) {
        self.search_text.clear();
        (
            vec![
                SectorSelectDialogAction::ClearSearch,
                SectorSelectDialogAction::RequestKeyboard,
                SectorSelectDialogAction::PostRequestKeyboard,
                SectorSelectDialogAction::Rebuild,
            ],
            self.model(sectors),
        )
    }

    pub fn search_changed(
        &mut self,
        text: impl Into<String>,
        sectors: &[SectorPreset],
    ) -> SectorSelectDialogModel {
        self.search_text = text.into();
        self.model(sectors)
    }

    pub fn enter_plan(&self, sectors: &[SectorPreset]) -> Vec<SectorSelectDialogAction> {
        let text = normalized_search(&self.search_text);
        sectors
            .iter()
            .find(|sector| matches_sector(sector, &self.planet_name, &text))
            .map(select_sector_plan)
            .unwrap_or_default()
    }

    pub fn row_click_plan(row: &SectorSelectRow) -> Vec<SectorSelectDialogAction> {
        vec![
            SectorSelectDialogAction::AcceptSectorPreset {
                id: row.id,
                name: row.name.clone(),
                localized_name: row.localized_name.clone(),
                planet_name: row.planet_name.clone(),
            },
            SectorSelectDialogAction::HideDialog,
        ]
    }

    pub fn model(&self, sectors: &[SectorPreset]) -> SectorSelectDialogModel {
        let text = normalized_search(&self.search_text);
        SectorSelectDialogModel {
            title: SECTOR_SELECT_DIALOG_TITLE,
            content_top_aligned: true,
            search_icon: SECTOR_SELECT_SEARCH_ICON,
            search_width: SECTOR_SELECT_SEARCH_WIDTH,
            search_text: self.search_text.clone(),
            pane_grow: true,
            pane_top_aligned: true,
            close_button_added: true,
            rows: sectors
                .iter()
                .filter(|sector| matches_sector(sector, &self.planet_name, &text))
                .map(sector_row)
                .collect(),
        }
    }
}

pub fn matches_sector(sector: &SectorPreset, planet_name: &str, normalized_text: &str) -> bool {
    sector.planet_name.as_deref() == Some(planet_name)
        && sector.require_unlock
        && (normalized_text.is_empty()
            || sector.name.to_lowercase().contains(normalized_text)
            || sector
                .localized_name
                .to_lowercase()
                .contains(normalized_text))
}

pub fn normalized_search(text: &str) -> String {
    text.to_lowercase()
}

fn sector_row(sector: &SectorPreset) -> SectorSelectRow {
    SectorSelectRow {
        id: sector.id,
        name: sector.name.clone(),
        localized_name: sector.localized_name.clone(),
        planet_name: sector.planet_name.clone().unwrap_or_default(),
        ui_icon: sector.name.clone(),
        style: SECTOR_SELECT_ROW_STYLE,
        icon_size: SECTOR_SELECT_ROW_ICON_SIZE,
        size: (SECTOR_SELECT_ROW_WIDTH, SECTOR_SELECT_ROW_HEIGHT),
        margin: SECTOR_SELECT_ROW_MARGIN,
        pad: SECTOR_SELECT_ROW_PAD,
    }
}

fn select_sector_plan(sector: &SectorPreset) -> Vec<SectorSelectDialogAction> {
    vec![
        SectorSelectDialogAction::AcceptSectorPreset {
            id: sector.id,
            name: sector.name.clone(),
            localized_name: sector.localized_name.clone(),
            planet_name: sector.planet_name.clone().unwrap_or_default(),
        },
        SectorSelectDialogAction::HideDialog,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preset(
        id: ContentId,
        name: &str,
        localized: &str,
        planet: &str,
        require_unlock: bool,
    ) -> SectorPreset {
        SectorPreset::with_planet_sector(name, planet, id as i32)
            .with_id(id)
            .localized(localized)
            .require_unlock(require_unlock)
    }

    fn sectors() -> Vec<SectorPreset> {
        vec![
            preset(15, "groundZero", "Ground Zero", "serpulo", true),
            preset(86, "frozenForest", "Frozen Forest", "serpulo", true),
            preset(10, "onset", "Onset", "erekir", true),
            preset(99, "hidden", "Hidden", "serpulo", false),
        ]
    }

    #[test]
    fn constructor_defaults_to_serpulo_and_database_sector_title() {
        let dialog = SectorSelectDialog::new();
        let model = dialog.model(&sectors());

        assert_eq!(dialog.planet_name, "serpulo");
        assert_eq!(model.title, "@database-category.sector");
        assert!(model.content_top_aligned);
        assert_eq!(model.search_icon, "zoom");
        assert_eq!(model.search_width, 300.0);
        assert!(model.pane_grow);
        assert!(model.pane_top_aligned);
        assert!(model.close_button_added);
    }

    #[test]
    fn shown_clears_search_requests_keyboard_posts_request_and_rebuilds() {
        let mut dialog = SectorSelectDialog::new();
        dialog.search_text = "forest".into();

        let (actions, model) = dialog.shown(&sectors());

        assert_eq!(
            actions,
            vec![
                SectorSelectDialogAction::ClearSearch,
                SectorSelectDialogAction::RequestKeyboard,
                SectorSelectDialogAction::PostRequestKeyboard,
                SectorSelectDialogAction::Rebuild,
            ]
        );
        assert_eq!(dialog.search_text, "");
        assert_eq!(model.search_text, "");
        assert_eq!(model.rows.len(), 2);
    }

    #[test]
    fn rebuild_filters_by_planet_require_unlock_and_case_insensitive_name_or_localized_name() {
        let mut dialog = SectorSelectDialog::new();

        let model = dialog.search_changed("FOREST", &sectors());

        assert_eq!(model.rows.len(), 1);
        assert_eq!(model.rows[0].name, "frozenForest");
        assert_eq!(model.rows[0].localized_name, "Frozen Forest");
        assert_eq!(model.rows[0].planet_name, "serpulo");
        assert_eq!(model.rows[0].ui_icon, "frozenForest");
        assert_eq!(model.rows[0].style, "Styles.grayt");
        assert_eq!(model.rows[0].icon_size, 32.0);
        assert_eq!(model.rows[0].size, (400.0, 50.0));
        assert_eq!(model.rows[0].margin, 4.0);
        assert_eq!(model.rows[0].pad, 3.0);

        let onset = dialog.show("erekir", &sectors());
        assert_eq!(onset.rows.len(), 1);
        assert_eq!(onset.rows[0].name, "onset");
    }

    #[test]
    fn matches_sector_follows_java_predicate() {
        let visible = preset(15, "groundZero", "Ground Zero", "serpulo", true);
        let hidden = preset(99, "hidden", "Hidden", "serpulo", false);
        let other = preset(10, "onset", "Onset", "erekir", true);

        assert!(matches_sector(&visible, "serpulo", ""));
        assert!(matches_sector(&visible, "serpulo", "zero"));
        assert!(matches_sector(&visible, "serpulo", "ground"));
        assert!(!matches_sector(&visible, "serpulo", "onset"));
        assert!(!matches_sector(&hidden, "serpulo", ""));
        assert!(!matches_sector(&other, "serpulo", ""));
    }

    #[test]
    fn enter_accepts_first_matching_sector_and_hides_dialog() {
        let mut dialog = SectorSelectDialog::new();
        dialog.search_changed("ground", &sectors());

        assert_eq!(
            dialog.enter_plan(&sectors()),
            vec![
                SectorSelectDialogAction::AcceptSectorPreset {
                    id: 15,
                    name: "groundZero".into(),
                    localized_name: "Ground Zero".into(),
                    planet_name: "serpulo".into(),
                },
                SectorSelectDialogAction::HideDialog,
            ]
        );
    }

    #[test]
    fn enter_without_match_does_nothing_like_null_found_branch() {
        let mut dialog = SectorSelectDialog::new();
        dialog.search_changed("does-not-exist", &sectors());

        assert!(dialog.enter_plan(&sectors()).is_empty());
    }

    #[test]
    fn row_click_accepts_clicked_sector_then_hides() {
        let dialog = SectorSelectDialog::new();
        let model = dialog.model(&sectors());

        assert_eq!(
            SectorSelectDialog::row_click_plan(&model.rows[1]),
            vec![
                SectorSelectDialogAction::AcceptSectorPreset {
                    id: 86,
                    name: "frozenForest".into(),
                    localized_name: "Frozen Forest".into(),
                    planet_name: "serpulo".into(),
                },
                SectorSelectDialogAction::HideDialog,
            ]
        );
    }
}
