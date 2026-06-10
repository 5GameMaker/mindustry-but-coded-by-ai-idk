//! Save-game dialog model mirroring upstream `mindustry.ui.dialogs.SaveDialog`.

use super::load_dialog::{LoadDialog, LoadDialogModel, LoadDialogSlotRow, LoadSaveSlot};

pub const SAVE_DIALOG_TITLE: &str = "@savegame";
pub const SAVE_DIALOG_NEW_BUTTON: &str = "@save.new";
pub const SAVE_DIALOG_NEW_TITLE: &str = "@save";
pub const SAVE_DIALOG_NEW_TEXT: &str = "@save.newslot";
pub const SAVE_DIALOG_NEW_MAX_LENGTH: usize = 30;
pub const SAVE_DIALOG_OVERWRITE_TITLE: &str = "@overwrite";
pub const SAVE_DIALOG_OVERWRITE_TEXT: &str = "@save.overwrite";
pub const SAVE_DIALOG_LOADING_TEXT: &str = "@saving";
pub const SAVE_DIALOG_SAVE_DELAY_FRAMES: f32 = 5.0;

#[derive(Debug, Clone, PartialEq)]
pub struct SaveDialogModel {
    pub load_model: LoadDialogModel,
    pub new_button_text: &'static str,
    pub new_prompt_title: &'static str,
    pub new_prompt_text: &'static str,
    pub new_prompt_max_length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveDialogAction {
    NewSavePrompt {
        title: &'static str,
        text: &'static str,
        max_length: usize,
    },
    AddSave {
        name: String,
        loading_text: &'static str,
    },
    ConfirmOverwrite {
        slot_id: String,
        title: &'static str,
        text: &'static str,
    },
    ShowLoading {
        text: &'static str,
    },
    DelaySave {
        frames: i32,
    },
    HideDialog,
    HideLoading,
    SaveSlot {
        slot_id: String,
    },
    ShowSaveException,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveDialog {
    load: LoadDialog,
}

impl Default for SaveDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveDialog {
    pub fn new() -> Self {
        Self {
            load: LoadDialog::with_title(SAVE_DIALOG_TITLE),
        }
    }

    pub fn load_dialog(&self) -> &LoadDialog {
        &self.load
    }

    pub fn load_dialog_mut(&mut self) -> &mut LoadDialog {
        &mut self.load
    }

    pub fn setup(
        &self,
        slots: &[LoadSaveSlot],
        graphics_width: f32,
        scale: f32,
        locale: &str,
    ) -> SaveDialogModel {
        SaveDialogModel {
            load_model: self.load.setup(slots, graphics_width, scale, locale),
            new_button_text: SAVE_DIALOG_NEW_BUTTON,
            new_prompt_title: SAVE_DIALOG_NEW_TITLE,
            new_prompt_text: SAVE_DIALOG_NEW_TEXT,
            new_prompt_max_length: SAVE_DIALOG_NEW_MAX_LENGTH,
        }
    }

    pub fn update_should_hide(state_is_menu: bool, is_shown: bool) -> bool {
        state_is_menu && is_shown
    }

    pub fn new_save_prompt_action() -> SaveDialogAction {
        SaveDialogAction::NewSavePrompt {
            title: SAVE_DIALOG_NEW_TITLE,
            text: SAVE_DIALOG_NEW_TEXT,
            max_length: SAVE_DIALOG_NEW_MAX_LENGTH,
        }
    }

    pub fn add_save_action(name: impl Into<String>) -> SaveDialogAction {
        SaveDialogAction::AddSave {
            name: name.into(),
            loading_text: SAVE_DIALOG_LOADING_TEXT,
        }
    }

    pub fn slot_click_action(
        row: &LoadDialogSlotRow,
        children_pressed: bool,
    ) -> Option<SaveDialogAction> {
        if children_pressed {
            None
        } else {
            Some(SaveDialogAction::ConfirmOverwrite {
                slot_id: row.slot_id.clone(),
                title: SAVE_DIALOG_OVERWRITE_TITLE,
                text: SAVE_DIALOG_OVERWRITE_TEXT,
            })
        }
    }

    pub fn save_plan(slot_id: impl Into<String>) -> Vec<SaveDialogAction> {
        let slot_id = slot_id.into();
        vec![
            SaveDialogAction::ShowLoading {
                text: SAVE_DIALOG_LOADING_TEXT,
            },
            SaveDialogAction::DelaySave {
                frames: SAVE_DIALOG_SAVE_DELAY_FRAMES as i32,
            },
            SaveDialogAction::HideDialog,
            SaveDialogAction::HideLoading,
            SaveDialogAction::SaveSlot { slot_id },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::game::Gamemode;

    fn slots() -> Vec<LoadSaveSlot> {
        vec![LoadSaveSlot::new("slot", "Slot", Gamemode::Survival)]
    }

    #[test]
    fn save_dialog_reuses_load_dialog_with_save_title_and_new_button() {
        let dialog = SaveDialog::new();
        let model = dialog.setup(&slots(), 800.0, 1.0, "en");

        assert_eq!(model.load_model.title, "@savegame");
        assert_eq!(model.new_button_text, "@save.new");
        assert_eq!(model.new_prompt_title, "@save");
        assert_eq!(model.new_prompt_text, "@save.newslot");
        assert_eq!(model.new_prompt_max_length, 30);
    }

    #[test]
    fn update_hides_save_dialog_when_game_returns_to_menu() {
        assert!(SaveDialog::update_should_hide(true, true));
        assert!(!SaveDialog::update_should_hide(true, false));
        assert!(!SaveDialog::update_should_hide(false, true));
    }

    #[test]
    fn save_click_confirms_overwrite_unless_child_button_pressed() {
        let model = SaveDialog::new().setup(&slots(), 800.0, 1.0, "en");
        let row = &model.load_model.rows[0];

        assert_eq!(SaveDialog::slot_click_action(row, true), None);
        assert_eq!(
            SaveDialog::slot_click_action(row, false),
            Some(SaveDialogAction::ConfirmOverwrite {
                slot_id: "slot".into(),
                title: "@overwrite",
                text: "@save.overwrite",
            })
        );
    }

    #[test]
    fn save_plan_matches_java_loading_delay_hide_and_slot_save_order() {
        assert_eq!(
            SaveDialog::save_plan("slot"),
            vec![
                SaveDialogAction::ShowLoading { text: "@saving" },
                SaveDialogAction::DelaySave { frames: 5 },
                SaveDialogAction::HideDialog,
                SaveDialogAction::HideLoading,
                SaveDialogAction::SaveSlot {
                    slot_id: "slot".into()
                },
            ]
        );
    }
}
