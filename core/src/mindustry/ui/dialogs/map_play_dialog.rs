//! Map-play dialog model mirroring upstream `mindustry.ui.dialogs.MapPlayDialog`.

use crate::mindustry::{
    game::{Gamemode, Rules},
    maps::MapDescriptor,
    ui::{upstream_menu_bundle_format_for_locale, upstream_menu_bundle_value_for_locale},
};

pub const MAP_PLAY_DIALOG_INITIAL_TITLE: &str = "";
pub const MAP_PLAY_MODE_TABLE_BACKGROUND: &str = "Tex.button";
pub const MAP_PLAY_MODE_BUTTON_STYLE: &str = "Styles.flatToggleMenut";
pub const MAP_PLAY_MODE_BUTTON_WIDTH: f32 = 140.0;
pub const MAP_PLAY_MODE_BUTTON_DESKTOP_HEIGHT: f32 = 54.0;
pub const MAP_PLAY_MODE_BUTTON_MOBILE_HEIGHT: f32 = 44.0;
pub const MAP_PLAY_MODE_COLUMNS: usize = 2;
pub const MAP_PLAY_HELP_BUTTON_TEXT: &str = "?";
pub const MAP_PLAY_HELP_BUTTON_WIDTH: f32 = 50.0;
pub const MAP_PLAY_HELP_BUTTON_PAD_LEFT: f32 = 18.0;
pub const MAP_PLAY_HELP_ENTRY_WIDTH: f32 = 400.0;
pub const MAP_PLAY_CUSTOMIZE_BUTTON_TEXT: &str = "@customize";
pub const MAP_PLAY_CUSTOMIZE_ICON: &str = "settings";
pub const MAP_PLAY_CUSTOMIZE_BUTTON_SIZE: (f32, f32) = (230.0, 50.0);
pub const MAP_PLAY_PREVIEW_BORDER_STROKE: f32 = 3.0;
pub const MAP_PLAY_PREVIEW_DESKTOP_SIZE: f32 = 250.0;
pub const MAP_PLAY_PREVIEW_MOBILE_LANDSCAPE_SIZE: f32 = 150.0;
pub const MAP_PLAY_PREVIEW_SCALING: &str = "Scaling.fit";
pub const MAP_PLAY_PLAY_BUTTON_TEXT: &str = "@play";
pub const MAP_PLAY_PLAY_ICON: &str = "play";
pub const MAP_PLAY_PLAY_BUTTON_SIZE: (f32, f32) = (210.0, 64.0);
pub const MAP_PLAY_HELP_OK_BUTTON_TEXT: &str = "@ok";
pub const MAP_PLAY_HELP_OK_BUTTON_SIZE: (f32, f32) = (110.0, 50.0);
pub const MAP_PLAY_HELP_OK_BUTTON_PAD: f32 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapPlayDialogContext {
    pub mobile: bool,
    pub portrait: bool,
    pub high_score: i32,
    pub play_listener_present: bool,
}

impl Default for MapPlayDialogContext {
    fn default() -> Self {
        Self {
            mobile: false,
            portrait: true,
            high_score: 0,
            play_listener_present: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapPlayDialogModel {
    pub title: String,
    pub fill_parent: bool,
    pub mode_label: String,
    pub mode_table_background: &'static str,
    pub mode_buttons: Vec<MapPlayModeButton>,
    pub help_button: MapPlayHelpButton,
    pub customize_button: MapPlayButton,
    pub preview: MapPlayPreview,
    pub high_score_label: Option<String>,
    pub close_button_added: bool,
    pub play_button: MapPlayButton,
    pub selected_gamemode: Gamemode,
    pub rules: Rules,
    pub map_file: String,
    pub map_name: String,
    pub playtesting: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapPlayModeButton {
    pub mode: Gamemode,
    pub text: String,
    pub style: &'static str,
    pub checked: bool,
    pub disabled: bool,
    pub size: (f32, f32),
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapPlayHelpButton {
    pub text: &'static str,
    pub width: f32,
    pub fill_y: bool,
    pub pad_left: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapPlayButton {
    pub text: &'static str,
    pub icon: Option<&'static str>,
    pub size: (f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapPlayPreview {
    pub map_file: String,
    pub border_stroke: f32,
    pub size: f32,
    pub scaling: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapPlayHelpDialogModel {
    pub title: String,
    pub fill_parent: bool,
    pub fade_scroll_bars: bool,
    pub entry_pad: f32,
    pub entries: Vec<MapPlayHelpEntry>,
    pub ok_button: MapPlayButton,
    pub ok_button_pad: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapPlayHelpEntry {
    pub mode: Gamemode,
    pub text: String,
    pub width: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapPlayDialogAction {
    SetSelectedGamemode {
        mode: Gamemode,
    },
    ApplyRules {
        mode: Gamemode,
        rules: Rules,
    },
    ShowCustomRules {
        rules: Rules,
        reset_mode: Gamemode,
    },
    RunPlayListener,
    PlayMap {
        map_file: String,
        map_name: String,
        rules: Rules,
        playtesting: bool,
    },
    HideDialog,
    HideCustomGameDialog,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapPlayDialog {
    pub play_listener_present: bool,
    pub rules: Rules,
    pub selected_gamemode: Gamemode,
    pub last_map_file: Option<String>,
    pub last_map_name: Option<String>,
}

impl Default for MapPlayDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl MapPlayDialog {
    pub fn new() -> Self {
        Self {
            play_listener_present: false,
            rules: Rules::default(),
            selected_gamemode: Gamemode::Survival,
            last_map_file: None,
            last_map_name: None,
        }
    }

    pub fn set_play_listener_present(&mut self, present: bool) {
        self.play_listener_present = present;
    }

    pub fn show(
        &mut self,
        map: &MapDescriptor,
        context: &MapPlayDialogContext,
        locale: &str,
    ) -> MapPlayDialogModel {
        self.show_with_playtesting(map, context, locale, false)
    }

    pub fn show_with_playtesting(
        &mut self,
        map: &MapDescriptor,
        context: &MapPlayDialogContext,
        locale: &str,
        playtesting: bool,
    ) -> MapPlayDialogModel {
        self.last_map_file = Some(map.file.clone());
        self.last_map_name = Some(map.name().to_string());

        if !self.selected_gamemode.valid(map) {
            self.selected_gamemode = Gamemode::ALL
                .iter()
                .copied()
                .find(|mode| mode.valid(map))
                .unwrap_or(Gamemode::Survival);
        }

        self.rules = map.apply_rules(self.selected_gamemode);
        self.model(map, context, locale, playtesting)
    }

    pub fn on_resize(
        &mut self,
        last_map: Option<&MapDescriptor>,
        context: &MapPlayDialogContext,
        locale: &str,
        playtesting: bool,
    ) -> Option<MapPlayDialogModel> {
        let map = last_map?;
        let rules = self.rules.clone();
        let mut model = self.show_with_playtesting(map, context, locale, playtesting);
        self.rules = rules;
        model.rules = self.rules.clone();
        Some(model)
    }

    pub fn select_gamemode_plan(
        &mut self,
        map: &MapDescriptor,
        mode: Gamemode,
    ) -> Vec<MapPlayDialogAction> {
        self.selected_gamemode = mode;
        self.rules = map.apply_rules(mode);
        vec![
            MapPlayDialogAction::SetSelectedGamemode { mode },
            MapPlayDialogAction::ApplyRules {
                mode,
                rules: self.rules.clone(),
            },
        ]
    }

    pub fn customize_plan(&self) -> MapPlayDialogAction {
        MapPlayDialogAction::ShowCustomRules {
            rules: self.rules.clone(),
            reset_mode: self.selected_gamemode,
        }
    }

    pub fn reset_custom_rules_plan(&mut self, map: &MapDescriptor) -> MapPlayDialogAction {
        self.rules = map.apply_rules(self.selected_gamemode);
        MapPlayDialogAction::ApplyRules {
            mode: self.selected_gamemode,
            rules: self.rules.clone(),
        }
    }

    pub fn play_plan(
        &self,
        map: &MapDescriptor,
        context: &MapPlayDialogContext,
        playtesting: bool,
    ) -> Vec<MapPlayDialogAction> {
        let mut actions = Vec::new();
        let has_listener = context.play_listener_present || self.play_listener_present;
        if has_listener {
            actions.push(MapPlayDialogAction::RunPlayListener);
        }
        actions.extend([
            MapPlayDialogAction::PlayMap {
                map_file: map.file.clone(),
                map_name: map.name().to_string(),
                rules: self.rules.clone(),
                playtesting,
            },
            MapPlayDialogAction::HideDialog,
            MapPlayDialogAction::HideCustomGameDialog,
        ]);
        actions
    }

    pub fn display_gamemode_help(locale: &str) -> MapPlayHelpDialogModel {
        MapPlayHelpDialogModel {
            title: bundle_value(locale, "mode.help.title"),
            fill_parent: false,
            fade_scroll_bars: false,
            entry_pad: 1.0,
            entries: Gamemode::ALL
                .iter()
                .copied()
                .filter(|mode| !mode.hidden())
                .map(|mode| MapPlayHelpEntry {
                    mode,
                    text: format!(
                        "[accent]{}:[] [lightgray]{}",
                        bundle_value(locale, &mode.name_bundle_key()),
                        bundle_value(locale, &mode.description_bundle_key())
                    ),
                    width: MAP_PLAY_HELP_ENTRY_WIDTH,
                })
                .collect(),
            ok_button: MapPlayButton {
                text: MAP_PLAY_HELP_OK_BUTTON_TEXT,
                icon: None,
                size: MAP_PLAY_HELP_OK_BUTTON_SIZE,
            },
            ok_button_pad: MAP_PLAY_HELP_OK_BUTTON_PAD,
        }
    }

    fn model(
        &self,
        map: &MapDescriptor,
        context: &MapPlayDialogContext,
        locale: &str,
        playtesting: bool,
    ) -> MapPlayDialogModel {
        MapPlayDialogModel {
            title: map.name().to_string(),
            fill_parent: false,
            mode_label: bundle_value(locale, "level.mode"),
            mode_table_background: MAP_PLAY_MODE_TABLE_BACKGROUND,
            mode_buttons: mode_buttons(map, self.selected_gamemode, context.mobile, locale),
            help_button: MapPlayHelpButton {
                text: MAP_PLAY_HELP_BUTTON_TEXT,
                width: MAP_PLAY_HELP_BUTTON_WIDTH,
                fill_y: true,
                pad_left: MAP_PLAY_HELP_BUTTON_PAD_LEFT,
            },
            customize_button: MapPlayButton {
                text: MAP_PLAY_CUSTOMIZE_BUTTON_TEXT,
                icon: Some(MAP_PLAY_CUSTOMIZE_ICON),
                size: MAP_PLAY_CUSTOMIZE_BUTTON_SIZE,
            },
            preview: MapPlayPreview {
                map_file: map.file.clone(),
                border_stroke: MAP_PLAY_PREVIEW_BORDER_STROKE,
                size: preview_size(context.mobile, context.portrait),
                scaling: MAP_PLAY_PREVIEW_SCALING,
            },
            high_score_label: Gamemode::Survival.valid(map).then(|| {
                bundle_format(
                    locale,
                    "level.highscore",
                    &[&context.high_score.to_string()],
                )
            }),
            close_button_added: true,
            play_button: MapPlayButton {
                text: MAP_PLAY_PLAY_BUTTON_TEXT,
                icon: Some(MAP_PLAY_PLAY_ICON),
                size: MAP_PLAY_PLAY_BUTTON_SIZE,
            },
            selected_gamemode: self.selected_gamemode,
            rules: self.rules.clone(),
            map_file: map.file.clone(),
            map_name: map.name().to_string(),
            playtesting,
        }
    }
}

pub fn mode_buttons(
    map: &MapDescriptor,
    selected_gamemode: Gamemode,
    mobile: bool,
    locale: &str,
) -> Vec<MapPlayModeButton> {
    let height = if mobile {
        MAP_PLAY_MODE_BUTTON_MOBILE_HEIGHT
    } else {
        MAP_PLAY_MODE_BUTTON_DESKTOP_HEIGHT
    };

    Gamemode::ALL
        .iter()
        .copied()
        .filter(|mode| !mode.hidden())
        .enumerate()
        .map(|(index, mode)| MapPlayModeButton {
            mode,
            text: bundle_value(locale, &mode.name_bundle_key()),
            style: MAP_PLAY_MODE_BUTTON_STYLE,
            checked: selected_gamemode == mode,
            disabled: !mode.valid(map),
            size: (MAP_PLAY_MODE_BUTTON_WIDTH, height),
            row: index / MAP_PLAY_MODE_COLUMNS,
            column: index % MAP_PLAY_MODE_COLUMNS,
        })
        .collect()
}

pub fn preview_size(mobile: bool, portrait: bool) -> f32 {
    if mobile && !portrait {
        MAP_PLAY_PREVIEW_MOBILE_LANDSCAPE_SIZE
    } else {
        MAP_PLAY_PREVIEW_DESKTOP_SIZE
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map_with(name: &str, spawns: i32, teams: &[i32]) -> MapDescriptor {
        let mut tags = BTreeMap::new();
        tags.insert("name".into(), name.into());
        let mut map = MapDescriptor::new(
            format!("maps/{}.msav", name.to_lowercase()),
            120,
            90,
            tags,
            true,
            11,
            157,
        );
        map.spawns = spawns;
        map.teams = teams.to_vec();
        map
    }

    #[test]
    fn map_play_dialog_initial_state_matches_java_constructor() {
        let dialog = MapPlayDialog::new();

        assert_eq!(MAP_PLAY_DIALOG_INITIAL_TITLE, "");
        assert_eq!(dialog.selected_gamemode, Gamemode::Survival);
        assert_eq!(dialog.last_map_file, None);
        assert!(!dialog.play_listener_present);
    }

    #[test]
    fn show_builds_mode_buttons_preview_highscore_and_rules_for_survival_map() {
        let map = map_with("Maze", 2, &[1, 2]);
        let mut dialog = MapPlayDialog::new();
        let context = MapPlayDialogContext {
            high_score: 42,
            ..MapPlayDialogContext::default()
        };

        let model = dialog.show(&map, &context, "en");

        assert_eq!(model.title, "Maze");
        assert!(!model.fill_parent);
        assert_eq!(model.mode_label, "Gamemode:");
        assert_eq!(model.mode_table_background, "Tex.button");
        assert_eq!(model.mode_buttons.len(), 4);
        assert_eq!(model.mode_buttons[0].mode, Gamemode::Survival);
        assert!(model.mode_buttons[0].checked);
        assert!(!model.mode_buttons[0].disabled);
        assert_eq!(model.mode_buttons[0].size, (140.0, 54.0));
        assert_eq!(
            (model.mode_buttons[1].row, model.mode_buttons[1].column),
            (0, 1)
        );
        assert_eq!(
            (model.mode_buttons[2].row, model.mode_buttons[2].column),
            (1, 0)
        );
        assert_eq!(model.help_button.text, "?");
        assert!(model.help_button.fill_y);
        assert_eq!(model.customize_button.text, "@customize");
        assert_eq!(model.customize_button.icon, Some("settings"));
        assert_eq!(model.preview.map_file, "maps/maze.msav");
        assert_eq!(model.preview.size, 250.0);
        assert_eq!(model.preview.scaling, "Scaling.fit");
        assert_eq!(
            model.high_score_label,
            Some("High Score: [accent]42".into())
        );
        assert!(model.close_button_added);
        assert_eq!(model.play_button.text, "@play");
        assert_eq!(model.play_button.icon, Some("play"));
        assert!(model.rules.waves);
        assert!(model.rules.wave_timer);
        assert_eq!(dialog.last_map_file, Some("maps/maze.msav".into()));
        assert_eq!(dialog.last_map_name, Some("Maze".into()));
    }

    #[test]
    fn show_resets_invalid_selected_mode_to_first_valid_gamemode_like_java_structs_find() {
        let map = map_with("Duel", 0, &[1, 2]);
        let mut dialog = MapPlayDialog::new();

        let model = dialog.show(&map, &MapPlayDialogContext::default(), "en");

        assert_eq!(model.selected_gamemode, Gamemode::Sandbox);
        assert!(!model.rules.wave_timer);
        assert!(model.rules.infinite_resources);
        assert!(model.high_score_label.is_none());
        assert!(model.mode_buttons[0].disabled);
        assert!(model.mode_buttons[1].checked);
        assert!(!model.mode_buttons[2].disabled);
        assert!(!model
            .mode_buttons
            .iter()
            .any(|button| button.mode == Gamemode::Editor));
    }

    #[test]
    fn selecting_mode_updates_selected_mode_and_reapplies_rules() {
        let map = map_with("Arena", 1, &[1, 2]);
        let mut dialog = MapPlayDialog::new();
        dialog.show(&map, &MapPlayDialogContext::default(), "en");

        let actions = dialog.select_gamemode_plan(&map, Gamemode::Pvp);

        assert_eq!(
            actions[0],
            MapPlayDialogAction::SetSelectedGamemode {
                mode: Gamemode::Pvp
            }
        );
        match &actions[1] {
            MapPlayDialogAction::ApplyRules { mode, rules } => {
                assert_eq!(*mode, Gamemode::Pvp);
                assert!(rules.pvp);
                assert!(rules.attack_mode);
                assert_eq!(rules.enemy_core_build_radius, 600.0);
            }
            other => panic!("unexpected action: {other:?}"),
        }
        assert_eq!(dialog.selected_gamemode, Gamemode::Pvp);
        assert!(dialog.rules.pvp);
    }

    #[test]
    fn customize_and_reset_actions_match_custom_rules_dialog_callback() {
        let map = map_with("Arena", 1, &[1, 2]);
        let mut dialog = MapPlayDialog::new();
        dialog.show(&map, &MapPlayDialogContext::default(), "en");
        dialog.select_gamemode_plan(&map, Gamemode::Attack);

        let customize = dialog.customize_plan();
        match customize {
            MapPlayDialogAction::ShowCustomRules { reset_mode, rules } => {
                assert_eq!(reset_mode, Gamemode::Attack);
                assert!(rules.attack_mode);
                assert!(!rules.pvp);
            }
            other => panic!("unexpected action: {other:?}"),
        }

        dialog.rules.pvp = true;
        let reset = dialog.reset_custom_rules_plan(&map);
        match reset {
            MapPlayDialogAction::ApplyRules { mode, rules } => {
                assert_eq!(mode, Gamemode::Attack);
                assert!(rules.attack_mode);
                assert!(!rules.pvp);
            }
            other => panic!("unexpected action: {other:?}"),
        }
    }

    #[test]
    fn play_plan_runs_optional_listener_then_plays_map_and_hides_both_dialogs() {
        let map = map_with("Maze", 2, &[1, 2]);
        let mut dialog = MapPlayDialog::new();
        dialog.show(&map, &MapPlayDialogContext::default(), "en");
        let context = MapPlayDialogContext {
            play_listener_present: true,
            ..MapPlayDialogContext::default()
        };

        let actions = dialog.play_plan(&map, &context, true);

        assert_eq!(actions[0], MapPlayDialogAction::RunPlayListener);
        match &actions[1] {
            MapPlayDialogAction::PlayMap {
                map_file,
                map_name,
                rules,
                playtesting,
            } => {
                assert_eq!(map_file, "maps/maze.msav");
                assert_eq!(map_name, "Maze");
                assert!(rules.waves);
                assert!(*playtesting);
            }
            other => panic!("unexpected action: {other:?}"),
        }
        assert_eq!(actions[2], MapPlayDialogAction::HideDialog);
        assert_eq!(actions[3], MapPlayDialogAction::HideCustomGameDialog);
    }

    #[test]
    fn resize_rebuilds_with_last_map_but_preserves_customized_rules() {
        let map = map_with("Maze", 2, &[1, 2]);
        let mut dialog = MapPlayDialog::new();
        dialog.show(&map, &MapPlayDialogContext::default(), "en");
        dialog.rules.pvp = true;

        let model = dialog
            .on_resize(Some(&map), &MapPlayDialogContext::default(), "en", false)
            .unwrap();

        assert!(dialog.rules.pvp);
        assert!(model.rules.pvp);
        assert_eq!(model.title, "Maze");
    }

    #[test]
    fn mobile_landscape_uses_smaller_preview_and_shorter_mode_buttons() {
        let map = map_with("Maze", 2, &[1, 2]);
        let mut dialog = MapPlayDialog::new();
        let context = MapPlayDialogContext {
            mobile: true,
            portrait: false,
            ..MapPlayDialogContext::default()
        };

        let model = dialog.show(&map, &context, "en");

        assert_eq!(model.preview.size, 150.0);
        assert_eq!(model.mode_buttons[0].size, (140.0, 44.0));
    }

    #[test]
    fn help_dialog_lists_visible_modes_with_localized_descriptions() {
        let help = MapPlayDialog::display_gamemode_help("en");

        assert_eq!(help.title, "Description of modes");
        assert!(!help.fill_parent);
        assert!(!help.fade_scroll_bars);
        assert_eq!(help.entries.len(), 4);
        assert_eq!(help.entries[0].mode, Gamemode::Survival);
        assert!(help.entries[0]
            .text
            .starts_with("[accent]Survival:[] [lightgray]"));
        assert_eq!(help.entries[0].width, 400.0);
        assert_eq!(help.ok_button.text, "@ok");
        assert_eq!(help.ok_button.size, (110.0, 50.0));
        assert!(!help
            .entries
            .iter()
            .any(|entry| entry.mode == Gamemode::Editor));
    }
}
