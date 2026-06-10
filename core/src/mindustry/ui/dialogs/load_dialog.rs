//! Load-game dialog model mirroring upstream `mindustry.ui.dialogs.LoadDialog`.

use crate::mindustry::{
    game::Gamemode,
    io::SAVE_EXTENSION,
    ui::{upstream_menu_bundle_format_for_locale, upstream_menu_bundle_value_for_locale},
};

pub const LOAD_DIALOG_TITLE: &str = "@loadgame";
pub const LOAD_DIALOG_SEARCH_MESSAGE: &str = "@save.search";
pub const LOAD_DIALOG_SLOT_TITLE_WIDTH: f32 = 230.0;
pub const LOAD_DIALOG_PREVIEW_SIZE: (f32, f32) = (160.0, 160.0);
pub const LOAD_DIALOG_META_WIDTH: f32 = 290.0;
pub const LOAD_DIALOG_MODE_BUTTON_SIZE: f32 = 60.0;
pub const LOAD_DIALOG_SLOT_PAD: f32 = 4.0;
pub const LOAD_DIALOG_SLOT_MARGIN: f32 = 10.0;
pub const LOAD_DIALOG_COLUMN_WIDTH: f32 = 470.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadSaveSlot {
    pub id: String,
    pub name: String,
    pub map_name: Option<String>,
    pub mode: Gamemode,
    pub wave: i32,
    pub autosave: bool,
    pub playtime: String,
    pub date: String,
    pub timestamp: i64,
    pub hidden: bool,
    pub preview_region: Option<String>,
}

impl LoadSaveSlot {
    pub fn new(id: impl Into<String>, name: impl Into<String>, mode: Gamemode) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            map_name: None,
            mode,
            wave: 1,
            autosave: false,
            playtime: "0:00".into(),
            date: String::new(),
            timestamp: 0,
            hidden: false,
            preview_region: None,
        }
    }

    pub fn with_map(mut self, map_name: impl Into<String>) -> Self {
        self.map_name = Some(map_name.into());
        self
    }

    pub fn with_wave(mut self, wave: i32) -> Self {
        self.wave = wave;
        self
    }

    pub fn with_autosave(mut self, autosave: bool) -> Self {
        self.autosave = autosave;
        self
    }

    pub fn with_playtime(mut self, playtime: impl Into<String>) -> Self {
        self.playtime = playtime.into();
        self
    }

    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.date = date.into();
        self
    }

    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadDialogModeFilter {
    pub mode: Gamemode,
    pub icon_name: String,
    pub checked: bool,
    pub tooltip: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadDialogSlotRow {
    pub slot_id: String,
    pub title: String,
    pub name: String,
    pub column: usize,
    pub row: usize,
    pub title_width: f32,
    pub preview_size: (f32, f32),
    pub preview_region: String,
    pub map_line: String,
    pub mode_wave_line: String,
    pub autosave_line: String,
    pub playtime_line: String,
    pub date_line: String,
    pub autosave_checked: bool,
    pub delete_confirm_title: &'static str,
    pub delete_confirm_text: &'static str,
    pub rename_title: &'static str,
    pub rename_text: &'static str,
    pub export_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadDialogModel {
    pub title: String,
    pub search_text: String,
    pub search_message: &'static str,
    pub mode_filters: Vec<LoadDialogModeFilter>,
    pub max_columns: usize,
    pub rows: Vec<LoadDialogSlotRow>,
    pub empty_label: Option<&'static str>,
    pub import_button_text: &'static str,
    pub import_extension: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadDialogAction {
    ToggleAutosave {
        slot_id: String,
        autosave: bool,
    },
    DeleteConfirm {
        slot_id: String,
        title: &'static str,
        text: &'static str,
    },
    RenamePrompt {
        slot_id: String,
        title: &'static str,
        text: &'static str,
        current_name: String,
    },
    Export {
        slot_id: String,
        name: String,
        extension: &'static str,
    },
    ImportPrompt {
        extension: &'static str,
    },
    ImportAccepted,
    ImportRejectedCampaign,
    ImportInvalid,
    CautiousLoad {
        slot_id: String,
    },
    HideLoadDialog,
    HidePausedDialog,
    ResetNet,
    LoadSlot {
        slot_id: String,
    },
    ClearEditorRules,
    SetPlaying,
    ShowCorruptedSave,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadDialog {
    title: String,
    search_string: Option<String>,
    hidden_modes: Vec<Gamemode>,
}

impl Default for LoadDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadDialog {
    pub fn new() -> Self {
        Self::with_title(LOAD_DIALOG_TITLE)
    }

    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            search_string: None,
            hidden_modes: Vec::new(),
        }
    }

    pub fn shown(&mut self) {
        self.search_string = Some(String::new());
        self.hidden_modes.clear();
    }

    pub fn search_string(&self) -> Option<&str> {
        self.search_string.as_deref()
    }

    pub fn hidden_modes(&self) -> &[Gamemode] {
        &self.hidden_modes
    }

    pub fn set_search_text(&mut self, text: impl Into<String>) {
        let text = text.into();
        self.search_string = if text.is_empty() {
            None
        } else {
            Some(text.to_lowercase())
        };
    }

    pub fn toggle_mode_filter(&mut self, mode: Gamemode) {
        if let Some(index) = self.hidden_modes.iter().position(|hidden| *hidden == mode) {
            self.hidden_modes.remove(index);
        } else {
            self.hidden_modes.push(mode);
        }
    }

    pub fn setup(
        &self,
        slots: &[LoadSaveSlot],
        graphics_width: f32,
        scale: f32,
        locale: &str,
    ) -> LoadDialogModel {
        let max_columns = ((graphics_width / (scale * LOAD_DIALOG_COLUMN_WIDTH)) as usize).max(1);
        let rows = self.rebuild_rows(slots, max_columns, locale);
        LoadDialogModel {
            title: self.title.clone(),
            search_text: self.search_string.clone().unwrap_or_default(),
            search_message: LOAD_DIALOG_SEARCH_MESSAGE,
            mode_filters: self.mode_filters(),
            max_columns,
            empty_label: rows.is_empty().then_some("@save.none"),
            rows,
            import_button_text: "@save.import",
            import_extension: SAVE_EXTENSION,
        }
    }

    pub fn mode_filters(&self) -> Vec<LoadDialogModeFilter> {
        Gamemode::ALL
            .into_iter()
            .map(|mode| LoadDialogModeFilter {
                mode,
                icon_name: if mode == Gamemode::Sandbox {
                    "terrain".into()
                } else {
                    format!("mode{}", capitalize_ascii(mode.wire_name()))
                },
                checked: !self.hidden_modes.contains(&mode),
                tooltip: format!("@mode.{}.name", mode.wire_name()),
            })
            .collect()
    }

    pub fn rebuild_rows(
        &self,
        slots: &[LoadSaveSlot],
        max_columns: usize,
        locale: &str,
    ) -> Vec<LoadDialogSlotRow> {
        let mut sorted = slots.to_vec();
        sorted.sort_by(|slot, other| other.timestamp.cmp(&slot.timestamp));

        let mut rows = Vec::new();
        for slot in sorted {
            if slot.hidden
                || self
                    .search_string
                    .as_ref()
                    .is_some_and(|search| !strip_colors(&slot.name).to_lowercase().contains(search))
                || (!self.hidden_modes.is_empty() && self.hidden_modes.contains(&slot.mode))
            {
                continue;
            }

            let index = rows.len();
            rows.push(slot_row(slot, index, max_columns, locale));
        }
        rows
    }

    pub fn toggle_autosave_action(slot: &LoadSaveSlot) -> LoadDialogAction {
        LoadDialogAction::ToggleAutosave {
            slot_id: slot.id.clone(),
            autosave: !slot.autosave,
        }
    }

    pub fn delete_action(row: &LoadDialogSlotRow) -> LoadDialogAction {
        LoadDialogAction::DeleteConfirm {
            slot_id: row.slot_id.clone(),
            title: "@confirm",
            text: row.delete_confirm_text,
        }
    }

    pub fn rename_action(row: &LoadDialogSlotRow) -> LoadDialogAction {
        LoadDialogAction::RenamePrompt {
            slot_id: row.slot_id.clone(),
            title: row.rename_title,
            text: row.rename_text,
            current_name: row.name.clone(),
        }
    }

    pub fn export_action(row: &LoadDialogSlotRow) -> LoadDialogAction {
        LoadDialogAction::Export {
            slot_id: row.slot_id.clone(),
            name: row.export_name.clone(),
            extension: SAVE_EXTENSION,
        }
    }

    pub fn import_action() -> LoadDialogAction {
        LoadDialogAction::ImportPrompt {
            extension: SAVE_EXTENSION,
        }
    }

    pub fn import_result_action(valid: bool, has_campaign_sector: bool) -> LoadDialogAction {
        if !valid {
            LoadDialogAction::ImportInvalid
        } else if has_campaign_sector {
            LoadDialogAction::ImportRejectedCampaign
        } else {
            LoadDialogAction::ImportAccepted
        }
    }

    pub fn slot_click_actions(
        row: &LoadDialogSlotRow,
        children_pressed: bool,
    ) -> Vec<LoadDialogAction> {
        if children_pressed {
            Vec::new()
        } else {
            Self::run_load_save_plan(&row.slot_id)
        }
    }

    pub fn run_load_save_plan(slot_id: &str) -> Vec<LoadDialogAction> {
        vec![
            LoadDialogAction::CautiousLoad {
                slot_id: slot_id.into(),
            },
            LoadDialogAction::HideLoadDialog,
            LoadDialogAction::HidePausedDialog,
            LoadDialogAction::ResetNet,
            LoadDialogAction::LoadSlot {
                slot_id: slot_id.into(),
            },
            LoadDialogAction::ClearEditorRules,
            LoadDialogAction::SetPlaying,
        ]
    }
}

fn slot_row(
    slot: LoadSaveSlot,
    index: usize,
    max_columns: usize,
    locale: &str,
) -> LoadDialogSlotRow {
    let color = "[lightgray]";
    let map = slot
        .map_name
        .as_deref()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| bundle_value(locale, "unknown"));
    let wave = bundle_format(locale, "save.wave", &[&format!("{color}{}", slot.wave)]);
    let autosave = bundle_format(
        locale,
        "save.autosave",
        &[&format!(
            "{color}{}",
            bundle_value(locale, if slot.autosave { "on" } else { "off" })
        )],
    );

    LoadDialogSlotRow {
        slot_id: slot.id.clone(),
        title: format!("[accent]{}", slot.name),
        name: slot.name.clone(),
        column: index % max_columns,
        row: index / max_columns,
        title_width: LOAD_DIALOG_SLOT_TITLE_WIDTH,
        preview_size: LOAD_DIALOG_PREVIEW_SIZE,
        preview_region: slot.preview_region.unwrap_or_else(|| "nomap".into()),
        map_line: bundle_format(locale, "save.map", &[&format!("{color}{map}")]),
        mode_wave_line: format!("{} /{} {}", mode_label(locale, slot.mode), color, wave),
        autosave_line: autosave,
        playtime_line: bundle_format(
            locale,
            "save.playtime",
            &[&format!("{color}{}", slot.playtime)],
        ),
        date_line: format!("{color}{}", slot.date),
        autosave_checked: slot.autosave,
        delete_confirm_title: "@confirm",
        delete_confirm_text: "@save.delete.confirm",
        rename_title: "@save.rename",
        rename_text: "@save.rename.text",
        export_name: format!("save-{}", slot.name),
    }
}

fn mode_label(locale: &str, mode: Gamemode) -> String {
    bundle_value(locale, &format!("mode.{}.name", mode.wire_name()))
}

fn bundle_value(locale: &str, key: &str) -> String {
    upstream_menu_bundle_value_for_locale(locale, key)
        .unwrap_or(key)
        .to_string()
}

fn bundle_format(locale: &str, key: &str, args: &[&str]) -> String {
    upstream_menu_bundle_format_for_locale(locale, key, args)
        .unwrap_or_else(|| replace_placeholders(key, args))
}

fn replace_placeholders(text: &str, args: &[&str]) -> String {
    let mut value = text.to_string();
    for (index, arg) in args.iter().enumerate() {
        value = value.replace(&format!("{{{index}}}"), arg);
    }
    value
}

fn strip_colors(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars();
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

fn capitalize_ascii(value: &str) -> String {
    let mut chars = value.chars();
    let mut out = String::new();
    if let Some(first) = chars.next() {
        out.push(first.to_ascii_uppercase());
    }
    out.extend(chars);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slots() -> Vec<LoadSaveSlot> {
        vec![
            LoadSaveSlot::new("old", "[scarlet]Alpha", Gamemode::Survival)
                .with_map("Ground Zero")
                .with_wave(3)
                .with_playtime("1:00")
                .with_date("2026-01-01")
                .with_timestamp(10),
            LoadSaveSlot::new("new", "Beta", Gamemode::Sandbox)
                .with_wave(9)
                .with_autosave(true)
                .with_playtime("2:00")
                .with_date("2026-01-02")
                .with_timestamp(20),
            LoadSaveSlot::new("hidden", "Hidden", Gamemode::Attack)
                .with_timestamp(30)
                .hidden(),
        ]
    }

    #[test]
    fn setup_sorts_visible_slots_by_timestamp_desc_and_builds_java_meta_lines() {
        let model = LoadDialog::new().setup(&slots(), 960.0, 1.0, "en");

        assert_eq!(model.title, "@loadgame");
        assert_eq!(model.max_columns, 2);
        assert_eq!(model.search_message, "@save.search");
        assert_eq!(model.empty_label, None);
        assert_eq!(model.rows.len(), 2);
        assert_eq!(model.rows[0].slot_id, "new");
        assert_eq!(model.rows[0].column, 0);
        assert_eq!(model.rows[0].row, 0);
        assert_eq!(model.rows[0].title, "[accent]Beta");
        assert_eq!(model.rows[0].map_line, "Map: [lightgray]Unknown");
        assert_eq!(
            model.rows[0].mode_wave_line,
            "Sandbox /[lightgray] Wave [lightgray]9"
        );
        assert_eq!(model.rows[0].autosave_line, "Autosave: [lightgray]On");
        assert_eq!(model.rows[0].playtime_line, "Playtime: [lightgray]2:00");
        assert_eq!(model.rows[1].slot_id, "old");
    }

    #[test]
    fn search_uses_stripped_lowercase_names_and_empty_state_matches_java_label() {
        let mut dialog = LoadDialog::new();
        dialog.set_search_text("alpha");
        let model = dialog.setup(&slots(), 400.0, 1.0, "en");
        assert_eq!(model.max_columns, 1);
        assert_eq!(model.rows.len(), 1);
        assert_eq!(model.rows[0].slot_id, "old");

        dialog.set_search_text("missing");
        let model = dialog.setup(&slots(), 400.0, 1.0, "en");
        assert!(model.rows.is_empty());
        assert_eq!(model.empty_label, Some("@save.none"));
    }

    #[test]
    fn mode_filter_toggle_hides_matching_modes_and_updates_checked_buttons() {
        let mut dialog = LoadDialog::new();
        dialog.toggle_mode_filter(Gamemode::Sandbox);
        let model = dialog.setup(&slots(), 960.0, 1.0, "en");

        assert_eq!(model.rows.len(), 1);
        assert_eq!(model.rows[0].slot_id, "old");
        let sandbox = model
            .mode_filters
            .iter()
            .find(|filter| filter.mode == Gamemode::Sandbox)
            .unwrap();
        assert_eq!(sandbox.icon_name, "terrain");
        assert!(!sandbox.checked);
    }

    #[test]
    fn slot_buttons_return_java_side_effect_actions() {
        let model = LoadDialog::new().setup(&slots(), 960.0, 1.0, "en");
        let row = &model.rows[0];

        assert_eq!(
            LoadDialog::delete_action(row),
            LoadDialogAction::DeleteConfirm {
                slot_id: "new".into(),
                title: "@confirm",
                text: "@save.delete.confirm",
            }
        );
        assert_eq!(
            LoadDialog::rename_action(row),
            LoadDialogAction::RenamePrompt {
                slot_id: "new".into(),
                title: "@save.rename",
                text: "@save.rename.text",
                current_name: "Beta".into(),
            }
        );
        assert_eq!(
            LoadDialog::export_action(row),
            LoadDialogAction::Export {
                slot_id: "new".into(),
                name: "save-Beta".into(),
                extension: "msav",
            }
        );
        assert!(LoadDialog::slot_click_actions(row, true).is_empty());
        assert_eq!(
            LoadDialog::slot_click_actions(row, false),
            LoadDialog::run_load_save_plan("new")
        );
    }

    #[test]
    fn import_result_action_matches_validity_and_campaign_rejection_branches() {
        assert_eq!(
            LoadDialog::import_result_action(false, false),
            LoadDialogAction::ImportInvalid
        );
        assert_eq!(
            LoadDialog::import_result_action(true, true),
            LoadDialogAction::ImportRejectedCampaign
        );
        assert_eq!(
            LoadDialog::import_result_action(true, false),
            LoadDialogAction::ImportAccepted
        );
    }
}
