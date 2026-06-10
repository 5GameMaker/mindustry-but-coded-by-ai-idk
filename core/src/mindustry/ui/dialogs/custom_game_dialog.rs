//! Custom-game dialog model mirroring upstream `mindustry.ui.dialogs.CustomGameDialog`.

use crate::mindustry::maps::MapDescriptor;

pub const CUSTOM_GAME_DIALOG_TITLE: &str = "@customgame";
pub const CUSTOM_GAME_DIALOG_CAMPAIGN: bool = false;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomGameDialogModel {
    pub title: &'static str,
    pub campaign: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomGameDialogAction {
    ShowMapPlay { map_file: String, map_name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CustomGameDialog;

impl CustomGameDialog {
    pub fn new() -> Self {
        Self
    }

    pub fn model(&self) -> CustomGameDialogModel {
        CustomGameDialogModel {
            title: CUSTOM_GAME_DIALOG_TITLE,
            campaign: CUSTOM_GAME_DIALOG_CAMPAIGN,
        }
    }

    pub fn show_map(&self, map: &MapDescriptor) -> CustomGameDialogAction {
        CustomGameDialogAction::ShowMapPlay {
            map_file: map.file.clone(),
            map_name: map.name().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn custom_game_dialog_is_non_campaign_map_list_with_custom_title() {
        let model = CustomGameDialog::new().model();

        assert_eq!(model.title, "@customgame");
        assert!(!model.campaign);
    }

    #[test]
    fn show_map_delegates_to_map_play_dialog_like_java_override() {
        let mut tags = BTreeMap::new();
        tags.insert("name".into(), "Maze".into());
        let map = MapDescriptor::new("maps/maze.msav", 100, 100, tags, true, 11, 157);

        assert_eq!(
            CustomGameDialog::new().show_map(&map),
            CustomGameDialogAction::ShowMapPlay {
                map_file: "maps/maze.msav".into(),
                map_name: "Maze".into(),
            }
        );
    }
}
