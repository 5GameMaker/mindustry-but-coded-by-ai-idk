//! Editor-maps dialog model mirroring upstream `mindustry.ui.dialogs.EditorMapsDialog`.

use crate::mindustry::maps::MapDescriptor;

use super::{
    MapListBottomButton, MapListDialog, MapListDialogAction, MapListDialogContext,
    MapListDialogModel, MapListMaps, MapListPlanet,
};

pub const EDITOR_MAPS_DIALOG_TITLE: &str = "@maps";
pub const EDITOR_MAPS_DIALOG_DISPLAY_TYPE: bool = true;
pub const EDITOR_MAPS_NEW_MAP_TEXT: &str = "@editor.newmap";
pub const EDITOR_MAPS_NEW_MAP_ICON: &str = "add";
pub const EDITOR_MAPS_IMPORT_MAP_TEXT: &str = "@editor.importmap";
pub const EDITOR_MAPS_IMPORT_MAP_ICON: &str = "upload";
pub const EDITOR_MAPS_BUTTON_SIZE: (f32, f32) = (210.0, 64.0);
pub const EDITOR_MAPS_TEXT_INPUT_TITLE: &str = "@editor.newmap";
pub const EDITOR_MAPS_TEXT_INPUT_MESSAGE: &str = "@editor.mapname";
pub const EDITOR_MAPS_TEXT_INPUT_DEFAULT: &str = "";
pub const EDITOR_MAPS_MAP_EXTENSION: &str = "msav";
pub const EDITOR_MAPS_EXISTS_ERROR: &str = "@editor.exists";
pub const EDITOR_MAPS_IMAGE_ERROR: &str = "@editor.errorimage";
pub const EDITOR_MAPS_NAME_ERROR: &str = "@editor.errorname";
pub const EDITOR_MAPS_IMPORT_EXISTS_KEY: &str = "editor.import.exists";
pub const EDITOR_MAPS_OVERWRITE_CONFIRM_KEY: &str = "editor.overwrite.confirm";
pub const EDITOR_MAPS_CONFIRM_TITLE: &str = "@confirm";
pub const EDITOR_MAPS_MAP_DELETE_KEY: &str = "map.delete";
pub const EDITOR_MAP_INFO_TITLE: &str = "@editor.mapinfo";
pub const EDITOR_MAP_INFO_PREVIEW_PORTRAIT_SIZE: f32 = 160.0;
pub const EDITOR_MAP_INFO_PREVIEW_DESKTOP_SIZE: f32 = 300.0;
pub const EDITOR_MAP_INFO_PREVIEW_SCALING: &str = "Scaling.fit";
pub const EDITOR_MAP_INFO_PANEL_STYLE: &str = "Styles.black";
pub const EDITOR_MAP_INFO_PANEL_MARGIN: f32 = 6.0;
pub const EDITOR_MAP_INFO_FIELD_PAD_TOP: f32 = 10.0;
pub const EDITOR_MAP_INFO_FIELD_LABEL_COLOR: &str = "Color.gray";
pub const EDITOR_MAP_INFO_NAME_LABEL: &str = "@editor.mapname";
pub const EDITOR_MAP_INFO_AUTHOR_LABEL: &str = "@editor.author";
pub const EDITOR_MAP_INFO_DESCRIPTION_LABEL: &str = "@editor.description";
pub const EDITOR_MAP_INFO_FALLBACK_AUTHOR: &str = "Anuke";
pub const EDITOR_MAP_INFO_OPEN_IN_TEXT: &str = "@editor.openin";
pub const EDITOR_MAP_INFO_OPEN_IN_ICON: &str = "export";
pub const EDITOR_MAP_INFO_VIEW_WORKSHOP_TEXT: &str = "@view.workshop";
pub const EDITOR_MAP_INFO_VIEW_WORKSHOP_ICON: &str = "link";
pub const EDITOR_MAP_INFO_DELETE_TEXT: &str = "@delete";
pub const EDITOR_MAP_INFO_DELETE_ICON: &str = "trash";
pub const EDITOR_MAP_INFO_ACTION_BUTTON_HEIGHT: f32 = 54.0;
pub const EDITOR_MAP_INFO_ACTION_BUTTON_MARGIN_LEFT: f32 = 10.0;
pub const EDITOR_MAP_INFO_MAP_NOT_FOUND_ERROR: &str = "@error.mapnotfound";

#[derive(Debug, Clone, PartialEq)]
pub struct EditorMapsDialogModel {
    pub title: String,
    pub display_type: bool,
    pub list: MapListDialogModel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorMapInfoDialogModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub map_file: String,
    pub map_name: String,
    pub preview: EditorMapInfoPreview,
    pub info_panel: EditorMapInfoPanel,
    pub fields: Vec<EditorMapInfoField>,
    pub open_in_button: EditorMapInfoButton,
    pub secondary_button: EditorMapInfoButton,
    pub shown: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorMapInfoPreview {
    pub map_file: String,
    pub image_scaling: &'static str,
    pub border_scaling: &'static str,
    pub size: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorMapInfoPanel {
    pub style: &'static str,
    pub width: f32,
    pub height: f32,
    pub margin: f32,
    pub scroll_pane_grow: bool,
    pub defaults_pad_top: f32,
    pub label_color: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorMapInfoField {
    pub label: &'static str,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMapInfoButtonAction {
    OpenInEditor,
    Delete,
    ViewWorkshop,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorMapInfoButton {
    pub text: &'static str,
    pub icon: &'static str,
    pub action: EditorMapInfoButtonAction,
    pub fill_x: bool,
    pub height: f32,
    pub margin_left: f32,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorMapsDialogAction {
    ShowTextInput {
        title: &'static str,
        message: &'static str,
        initial_text: &'static str,
    },
    ShowFileChooser {
        open: bool,
        extension: &'static str,
    },
    LoadAnd,
    TryCatchMapError,
    HideMapListDialog,
    ShowEditor,
    SetEditorTagName {
        name: String,
    },
    FireMapMakeEvent,
    ShowErrorMessage {
        message: &'static str,
    },
    ShowInfoFormatted {
        bundle_key: &'static str,
        value: String,
    },
    ShowConfirmFormatted {
        title: &'static str,
        bundle_key: &'static str,
        value: String,
        confirmed_actions: Vec<EditorMapsDialogAction>,
    },
    ImportMap {
        map_file: String,
        map_name: String,
    },
    RemoveMap {
        map_file: String,
        map_name: String,
    },
    SetupMapList,
    ShowMapInfoDialog {
        map_file: String,
        map_name: String,
    },
    BeginEditMap {
        map_file: String,
    },
    HideMapInfoDialog,
    ViewListing {
        map_file: String,
        map_name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorMapsDialog {
    pub list: MapListDialog,
}

impl Default for EditorMapsDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorMapsDialog {
    pub fn new() -> Self {
        Self {
            list: MapListDialog::new(EDITOR_MAPS_DIALOG_TITLE, EDITOR_MAPS_DIALOG_DISPLAY_TYPE),
        }
    }

    pub fn setup(
        &mut self,
        maps: &MapListMaps,
        planets: &[MapListPlanet],
        context: &MapListDialogContext,
    ) -> EditorMapsDialogModel {
        let model = self.list.setup(maps, planets, context);
        editor_maps_model(model)
    }

    pub fn model(
        &self,
        maps: &MapListMaps,
        context: &MapListDialogContext,
    ) -> EditorMapsDialogModel {
        editor_maps_model(self.list.model(maps, context))
    }

    pub fn on_resize(
        &mut self,
        maps: &MapListMaps,
        planets: &[MapListPlanet],
        context: &MapListDialogContext,
    ) -> (Vec<MapListDialogAction>, EditorMapsDialogModel) {
        let (actions, model) = self.list.on_resize(maps, planets, context);
        (actions, editor_maps_model(model))
    }

    pub fn show_plan(&self, context: &MapListDialogContext) -> Vec<MapListDialogAction> {
        self.list.show_plan(context)
    }

    pub fn back_plan() -> Vec<MapListDialogAction> {
        MapListDialog::back_plan()
    }

    pub fn search_changed_plan(&mut self, text: &str) -> Vec<MapListDialogAction> {
        self.list.search_changed_plan(text)
    }

    pub fn show_map_plan(&mut self, map: &MapDescriptor) -> EditorMapsDialogAction {
        self.list.active_dialog_open = true;
        EditorMapsDialogAction::ShowMapInfoDialog {
            map_file: map.file.clone(),
            map_name: map.name().to_string(),
        }
    }

    pub fn map_info_model(
        &mut self,
        map: &MapDescriptor,
        context: &MapListDialogContext,
        steam: bool,
    ) -> EditorMapInfoDialogModel {
        self.list.active_dialog_open = true;
        map_info_model(map, context, steam)
    }

    pub fn new_map_button_plan() -> Vec<EditorMapsDialogAction> {
        vec![EditorMapsDialogAction::ShowTextInput {
            title: EDITOR_MAPS_TEXT_INPUT_TITLE,
            message: EDITOR_MAPS_TEXT_INPUT_MESSAGE,
            initial_text: EDITOR_MAPS_TEXT_INPUT_DEFAULT,
        }]
    }

    pub fn new_map_submitted_plan(
        maps: &MapListMaps,
        name: impl Into<String>,
    ) -> Vec<EditorMapsDialogAction> {
        let name = name.into();
        if map_by_exact_name(maps, &name).is_some() {
            vec![EditorMapsDialogAction::ShowErrorMessage {
                message: EDITOR_MAPS_EXISTS_ERROR,
            }]
        } else {
            vec![
                EditorMapsDialogAction::LoadAnd,
                EditorMapsDialogAction::HideMapListDialog,
                EditorMapsDialogAction::ShowEditor,
                EditorMapsDialogAction::SetEditorTagName { name },
                EditorMapsDialogAction::FireMapMakeEvent,
            ]
        }
    }

    pub fn import_map_button_plan() -> Vec<EditorMapsDialogAction> {
        vec![EditorMapsDialogAction::ShowFileChooser {
            open: true,
            extension: EDITOR_MAPS_MAP_EXTENSION,
        }]
    }

    pub fn import_chosen_plan(
        maps: &MapListMaps,
        imported_map: &MapDescriptor,
        image: bool,
    ) -> Vec<EditorMapsDialogAction> {
        let resolved_name = imported_map
            .tags
            .get("name")
            .cloned()
            .or_else(|| Some(next_unknown_import_name(maps)));
        Self::import_chosen_plan_with_resolved_name(maps, imported_map, image, resolved_name)
    }

    pub fn import_chosen_plan_with_resolved_name(
        maps: &MapListMaps,
        imported_map: &MapDescriptor,
        image: bool,
        resolved_name: Option<String>,
    ) -> Vec<EditorMapsDialogAction> {
        let mut actions = vec![
            EditorMapsDialogAction::LoadAnd,
            EditorMapsDialogAction::TryCatchMapError,
        ];

        if image {
            actions.push(EditorMapsDialogAction::ShowErrorMessage {
                message: EDITOR_MAPS_IMAGE_ERROR,
            });
            return actions;
        }

        let Some(name) = resolved_name else {
            actions.push(EditorMapsDialogAction::ShowErrorMessage {
                message: EDITOR_MAPS_NAME_ERROR,
            });
            return actions;
        };

        if let Some(conflict) = map_by_case_insensitive_name(maps, &name) {
            if !conflict.custom {
                actions.push(EditorMapsDialogAction::ShowInfoFormatted {
                    bundle_key: EDITOR_MAPS_IMPORT_EXISTS_KEY,
                    value: name,
                });
            } else {
                actions.push(EditorMapsDialogAction::ShowConfirmFormatted {
                    title: EDITOR_MAPS_CONFIRM_TITLE,
                    bundle_key: EDITOR_MAPS_OVERWRITE_CONFIRM_KEY,
                    value: imported_map.name().to_string(),
                    confirmed_actions: import_overwrite_confirmed_plan(conflict, imported_map),
                });
            }
        } else {
            actions.extend(import_confirmed_plan(imported_map, name));
        }

        actions
    }

    pub fn open_in_editor_plan(
        map: &MapDescriptor,
        begin_edit_succeeds: bool,
    ) -> Vec<EditorMapsDialogAction> {
        let mut actions = vec![EditorMapsDialogAction::BeginEditMap {
            map_file: map.file.clone(),
        }];
        if begin_edit_succeeds {
            actions.extend([
                EditorMapsDialogAction::HideMapInfoDialog,
                EditorMapsDialogAction::HideMapListDialog,
            ]);
        } else {
            actions.push(EditorMapsDialogAction::ShowErrorMessage {
                message: EDITOR_MAP_INFO_MAP_NOT_FOUND_ERROR,
            });
        }
        actions
    }

    pub fn secondary_action_plan(map: &MapDescriptor, steam: bool) -> Vec<EditorMapsDialogAction> {
        if map.workshop && steam {
            vec![EditorMapsDialogAction::ViewListing {
                map_file: map.file.clone(),
                map_name: map.name().to_string(),
            }]
        } else {
            vec![EditorMapsDialogAction::ShowConfirmFormatted {
                title: EDITOR_MAPS_CONFIRM_TITLE,
                bundle_key: EDITOR_MAPS_MAP_DELETE_KEY,
                value: map.name().to_string(),
                confirmed_actions: delete_confirmed_plan(map),
            }]
        }
    }

    pub fn delete_confirmed_plan(map: &MapDescriptor) -> Vec<EditorMapsDialogAction> {
        delete_confirmed_plan(map)
    }
}

pub fn editor_maps_buttons() -> Vec<MapListBottomButton> {
    vec![
        MapListBottomButton {
            text: EDITOR_MAPS_NEW_MAP_TEXT,
            icon: EDITOR_MAPS_NEW_MAP_ICON,
            size: EDITOR_MAPS_BUTTON_SIZE,
            colspan: 1,
            row_after: false,
        },
        MapListBottomButton {
            text: EDITOR_MAPS_IMPORT_MAP_TEXT,
            icon: EDITOR_MAPS_IMPORT_MAP_ICON,
            size: EDITOR_MAPS_BUTTON_SIZE,
            colspan: 1,
            row_after: false,
        },
    ]
}

pub fn next_unknown_import_name(maps: &MapListMaps) -> String {
    let mut number = 0usize;
    loop {
        let candidate = format!("unknown{number}");
        number += 1;
        if map_by_exact_name(maps, &candidate).is_none() {
            return format!("unknown{number}");
        }
    }
}

fn editor_maps_model(mut list: MapListDialogModel) -> EditorMapsDialogModel {
    list.bottom_buttons.extend(editor_maps_buttons());
    EditorMapsDialogModel {
        title: list.title.clone(),
        display_type: list.display_type,
        list,
    }
}

fn map_info_model(
    map: &MapDescriptor,
    context: &MapListDialogContext,
    steam: bool,
) -> EditorMapInfoDialogModel {
    let size = if context.portrait {
        EDITOR_MAP_INFO_PREVIEW_PORTRAIT_SIZE
    } else {
        EDITOR_MAP_INFO_PREVIEW_DESKTOP_SIZE
    };
    let mut fields = vec![
        EditorMapInfoField {
            label: EDITOR_MAP_INFO_NAME_LABEL,
            value: map.name().to_string(),
        },
        EditorMapInfoField {
            label: EDITOR_MAP_INFO_AUTHOR_LABEL,
            value: author_text(map),
        },
    ];
    if raw_tag_non_empty(map, "description") {
        fields.push(EditorMapInfoField {
            label: EDITOR_MAP_INFO_DESCRIPTION_LABEL,
            value: map.description().to_string(),
        });
    }
    let secondary_view_workshop = map.workshop && steam;
    EditorMapInfoDialogModel {
        title: EDITOR_MAP_INFO_TITLE,
        close_button_added: true,
        map_file: map.file.clone(),
        map_name: map.name().to_string(),
        preview: EditorMapInfoPreview {
            map_file: map.file.clone(),
            image_scaling: EDITOR_MAP_INFO_PREVIEW_SCALING,
            border_scaling: EDITOR_MAP_INFO_PREVIEW_SCALING,
            size,
        },
        info_panel: EditorMapInfoPanel {
            style: EDITOR_MAP_INFO_PANEL_STYLE,
            width: size,
            height: size,
            margin: EDITOR_MAP_INFO_PANEL_MARGIN,
            scroll_pane_grow: true,
            defaults_pad_top: EDITOR_MAP_INFO_FIELD_PAD_TOP,
            label_color: EDITOR_MAP_INFO_FIELD_LABEL_COLOR,
        },
        fields,
        open_in_button: EditorMapInfoButton {
            text: EDITOR_MAP_INFO_OPEN_IN_TEXT,
            icon: EDITOR_MAP_INFO_OPEN_IN_ICON,
            action: EditorMapInfoButtonAction::OpenInEditor,
            fill_x: true,
            height: EDITOR_MAP_INFO_ACTION_BUTTON_HEIGHT,
            margin_left: EDITOR_MAP_INFO_ACTION_BUTTON_MARGIN_LEFT,
            disabled: false,
        },
        secondary_button: EditorMapInfoButton {
            text: if secondary_view_workshop {
                EDITOR_MAP_INFO_VIEW_WORKSHOP_TEXT
            } else {
                EDITOR_MAP_INFO_DELETE_TEXT
            },
            icon: if secondary_view_workshop {
                EDITOR_MAP_INFO_VIEW_WORKSHOP_ICON
            } else {
                EDITOR_MAP_INFO_DELETE_ICON
            },
            action: if secondary_view_workshop {
                EditorMapInfoButtonAction::ViewWorkshop
            } else {
                EditorMapInfoButtonAction::Delete
            },
            fill_x: true,
            height: EDITOR_MAP_INFO_ACTION_BUTTON_HEIGHT,
            margin_left: EDITOR_MAP_INFO_ACTION_BUTTON_MARGIN_LEFT,
            disabled: !map.workshop && !map.custom,
        },
        shown: true,
    }
}

fn import_confirmed_plan(
    imported_map: &MapDescriptor,
    map_name: String,
) -> Vec<EditorMapsDialogAction> {
    vec![
        EditorMapsDialogAction::ImportMap {
            map_file: imported_map.file.clone(),
            map_name,
        },
        EditorMapsDialogAction::SetupMapList,
    ]
}

fn import_overwrite_confirmed_plan(
    conflict: &MapDescriptor,
    imported_map: &MapDescriptor,
) -> Vec<EditorMapsDialogAction> {
    let mut actions = vec![
        EditorMapsDialogAction::TryCatchMapError,
        EditorMapsDialogAction::RemoveMap {
            map_file: conflict.file.clone(),
            map_name: conflict.name().to_string(),
        },
    ];
    actions.extend(import_confirmed_plan(
        imported_map,
        imported_map.name().to_string(),
    ));
    actions
}

fn delete_confirmed_plan(map: &MapDescriptor) -> Vec<EditorMapsDialogAction> {
    vec![
        EditorMapsDialogAction::RemoveMap {
            map_file: map.file.clone(),
            map_name: map.name().to_string(),
        },
        EditorMapsDialogAction::HideMapInfoDialog,
        EditorMapsDialogAction::SetupMapList,
    ]
}

fn author_text(map: &MapDescriptor) -> String {
    if !map.custom && raw_tag_empty(map, "author") {
        EDITOR_MAP_INFO_FALLBACK_AUTHOR.into()
    } else {
        map.author().to_string()
    }
}

fn raw_tag_empty(map: &MapDescriptor, key: &str) -> bool {
    map.tags
        .get(key)
        .map(|value| value.is_empty())
        .unwrap_or(true)
}

fn raw_tag_non_empty(map: &MapDescriptor, key: &str) -> bool {
    map.tags
        .get(key)
        .map(|value| !value.is_empty())
        .unwrap_or(false)
}

fn map_by_exact_name<'a>(maps: &'a MapListMaps, name: &str) -> Option<&'a MapDescriptor> {
    all_maps(maps).find(|map| map.name() == name)
}

fn map_by_case_insensitive_name<'a>(
    maps: &'a MapListMaps,
    name: &str,
) -> Option<&'a MapDescriptor> {
    all_maps(maps).find(|map| map.name().eq_ignore_ascii_case(name))
}

fn all_maps(maps: &MapListMaps) -> impl Iterator<Item = &MapDescriptor> {
    maps.custom
        .iter()
        .chain(maps.builtin.iter())
        .chain(maps.modded.iter())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn map(file: &str, name: &str, custom: bool) -> MapDescriptor {
        let mut tags = BTreeMap::new();
        tags.insert("name".into(), name.into());
        tags.insert("author".into(), "Mapper".into());
        tags.insert("description".into(), "A map".into());
        MapDescriptor::new(file, 120, 120, tags, custom, 11, 157)
    }

    fn nameless_map(file: &str) -> MapDescriptor {
        MapDescriptor::new(file, 64, 64, BTreeMap::new(), true, 11, 157)
    }

    fn maps() -> MapListMaps {
        MapListMaps {
            custom: vec![map("maps/custom/arena.msav", "Arena", true)],
            builtin: vec![map("maps/default/maze.msav", "Maze", false)],
            modded: Vec::new(),
        }
    }

    fn planets() -> Vec<MapListPlanet> {
        vec![MapListPlanet::new("serpulo", "Serpulo")]
    }

    #[test]
    fn constructor_and_setup_extend_map_list_with_editor_buttons_after_back() {
        let mut dialog = EditorMapsDialog::new();
        let model = dialog.setup(
            &maps(),
            &planets(),
            &MapListDialogContext {
                portrait: true,
                graphics_width: 460,
                ..MapListDialogContext::default()
            },
        );

        assert_eq!(dialog.list.title, "@maps");
        assert!(dialog.list.display_type);
        assert_eq!(model.title, "@maps");
        assert!(model.display_type);
        assert_eq!(
            model
                .list
                .bottom_buttons
                .iter()
                .map(|button| (
                    button.text,
                    button.icon,
                    button.size,
                    button.colspan,
                    button.row_after
                ))
                .collect::<Vec<_>>(),
            vec![
                ("@back", "left", (420.0, 64.0), 2, true),
                ("@editor.newmap", "add", (210.0, 64.0), 1, false),
                ("@editor.importmap", "upload", (210.0, 64.0), 1, false),
            ]
        );
    }

    #[test]
    fn new_map_prompt_and_submit_plan_match_java_order() {
        assert_eq!(
            EditorMapsDialog::new_map_button_plan(),
            vec![EditorMapsDialogAction::ShowTextInput {
                title: "@editor.newmap",
                message: "@editor.mapname",
                initial_text: "",
            }]
        );

        assert_eq!(
            EditorMapsDialog::new_map_submitted_plan(&maps(), "Arena"),
            vec![EditorMapsDialogAction::ShowErrorMessage {
                message: "@editor.exists",
            }]
        );

        assert_eq!(
            EditorMapsDialog::new_map_submitted_plan(&maps(), "Fresh"),
            vec![
                EditorMapsDialogAction::LoadAnd,
                EditorMapsDialogAction::HideMapListDialog,
                EditorMapsDialogAction::ShowEditor,
                EditorMapsDialogAction::SetEditorTagName {
                    name: "Fresh".into(),
                },
                EditorMapsDialogAction::FireMapMakeEvent,
            ]
        );
    }

    #[test]
    fn import_button_opens_map_file_chooser_and_rejects_images_inside_load_and_trycatch() {
        assert_eq!(
            EditorMapsDialog::import_map_button_plan(),
            vec![EditorMapsDialogAction::ShowFileChooser {
                open: true,
                extension: "msav",
            }]
        );

        assert_eq!(
            EditorMapsDialog::import_chosen_plan(&maps(), &map("image.png", "Image", true), true),
            vec![
                EditorMapsDialogAction::LoadAnd,
                EditorMapsDialogAction::TryCatchMapError,
                EditorMapsDialogAction::ShowErrorMessage {
                    message: "@editor.errorimage",
                },
            ]
        );
    }

    #[test]
    fn import_name_resolution_builtin_conflict_custom_confirm_and_success_match_java_branches() {
        let source = map("incoming/maze.msav", "maze", true);
        assert_eq!(
            EditorMapsDialog::import_chosen_plan(&maps(), &source, false),
            vec![
                EditorMapsDialogAction::LoadAnd,
                EditorMapsDialogAction::TryCatchMapError,
                EditorMapsDialogAction::ShowInfoFormatted {
                    bundle_key: "editor.import.exists",
                    value: "maze".into(),
                },
            ]
        );

        let overwrite = map("incoming/arena.msav", "Arena", true);
        let actions = EditorMapsDialog::import_chosen_plan(&maps(), &overwrite, false);
        assert_eq!(actions.len(), 3);
        match &actions[2] {
            EditorMapsDialogAction::ShowConfirmFormatted {
                title,
                bundle_key,
                value,
                confirmed_actions,
            } => {
                assert_eq!(*title, "@confirm");
                assert_eq!(*bundle_key, "editor.overwrite.confirm");
                assert_eq!(value, "Arena");
                assert_eq!(
                    confirmed_actions,
                    &vec![
                        EditorMapsDialogAction::TryCatchMapError,
                        EditorMapsDialogAction::RemoveMap {
                            map_file: "maps/custom/arena.msav".into(),
                            map_name: "Arena".into(),
                        },
                        EditorMapsDialogAction::ImportMap {
                            map_file: "incoming/arena.msav".into(),
                            map_name: "Arena".into(),
                        },
                        EditorMapsDialogAction::SetupMapList,
                    ]
                );
            }
            other => panic!("unexpected action: {other:?}"),
        }

        let fresh = map("incoming/fresh.msav", "Fresh", true);
        assert_eq!(
            EditorMapsDialog::import_chosen_plan(&maps(), &fresh, false),
            vec![
                EditorMapsDialogAction::LoadAnd,
                EditorMapsDialogAction::TryCatchMapError,
                EditorMapsDialogAction::ImportMap {
                    map_file: "incoming/fresh.msav".into(),
                    map_name: "Fresh".into(),
                },
                EditorMapsDialogAction::SetupMapList,
            ]
        );
    }

    #[test]
    fn import_missing_and_null_name_branches_follow_upstream_guard() {
        assert_eq!(next_unknown_import_name(&maps()), "unknown1");

        let nameless = nameless_map("incoming/save.msav");
        assert_eq!(
            EditorMapsDialog::import_chosen_plan(&maps(), &nameless, false),
            vec![
                EditorMapsDialogAction::LoadAnd,
                EditorMapsDialogAction::TryCatchMapError,
                EditorMapsDialogAction::ImportMap {
                    map_file: "incoming/save.msav".into(),
                    map_name: "unknown1".into(),
                },
                EditorMapsDialogAction::SetupMapList,
            ]
        );
        assert_eq!(
            EditorMapsDialog::import_chosen_plan_with_resolved_name(
                &maps(),
                &nameless,
                false,
                None,
            ),
            vec![
                EditorMapsDialogAction::LoadAnd,
                EditorMapsDialogAction::TryCatchMapError,
                EditorMapsDialogAction::ShowErrorMessage {
                    message: "@editor.errorname",
                },
            ]
        );
    }

    #[test]
    fn map_info_model_uses_preview_size_author_fallback_description_and_buttons() {
        let mut builtin_tags = BTreeMap::new();
        builtin_tags.insert("name".into(), "Builtin".into());
        builtin_tags.insert("description".into(), "Desc".into());
        let builtin = MapDescriptor::new(
            "maps/default/builtin.msav",
            100,
            100,
            builtin_tags,
            false,
            11,
            157,
        );
        let mut dialog = EditorMapsDialog::new();

        let portrait = dialog.map_info_model(
            &builtin,
            &MapListDialogContext {
                portrait: true,
                ..MapListDialogContext::default()
            },
            false,
        );

        assert!(dialog.list.active_dialog_open);
        assert_eq!(portrait.title, "@editor.mapinfo");
        assert!(portrait.close_button_added);
        assert_eq!(portrait.preview.size, 160.0);
        assert_eq!(portrait.preview.image_scaling, "Scaling.fit");
        assert_eq!(portrait.info_panel.style, "Styles.black");
        assert_eq!(portrait.info_panel.width, 160.0);
        assert_eq!(
            portrait.fields,
            vec![
                EditorMapInfoField {
                    label: "@editor.mapname",
                    value: "Builtin".into(),
                },
                EditorMapInfoField {
                    label: "@editor.author",
                    value: "Anuke".into(),
                },
                EditorMapInfoField {
                    label: "@editor.description",
                    value: "Desc".into(),
                },
            ]
        );
        assert_eq!(portrait.open_in_button.text, "@editor.openin");
        assert_eq!(portrait.open_in_button.icon, "export");
        assert_eq!(portrait.secondary_button.text, "@delete");
        assert_eq!(portrait.secondary_button.icon, "trash");
        assert_eq!(
            portrait.secondary_button.action,
            EditorMapInfoButtonAction::Delete
        );
        assert!(portrait.secondary_button.disabled);

        let mut workshop = map("maps/workshop/one.msav", "Workshop", false);
        workshop.workshop = true;
        let desktop = dialog.map_info_model(
            &workshop,
            &MapListDialogContext {
                portrait: false,
                ..MapListDialogContext::default()
            },
            true,
        );
        assert_eq!(desktop.preview.size, 300.0);
        assert_eq!(desktop.secondary_button.text, "@view.workshop");
        assert_eq!(desktop.secondary_button.icon, "link");
        assert_eq!(
            desktop.secondary_button.action,
            EditorMapInfoButtonAction::ViewWorkshop
        );
        assert!(!desktop.secondary_button.disabled);
    }

    #[test]
    fn map_info_action_plans_match_open_delete_and_workshop_order() {
        let mut custom = map("maps/custom/arena.msav", "Arena", true);
        assert_eq!(
            EditorMapsDialog::open_in_editor_plan(&custom, true),
            vec![
                EditorMapsDialogAction::BeginEditMap {
                    map_file: "maps/custom/arena.msav".into(),
                },
                EditorMapsDialogAction::HideMapInfoDialog,
                EditorMapsDialogAction::HideMapListDialog,
            ]
        );
        assert_eq!(
            EditorMapsDialog::open_in_editor_plan(&custom, false),
            vec![
                EditorMapsDialogAction::BeginEditMap {
                    map_file: "maps/custom/arena.msav".into(),
                },
                EditorMapsDialogAction::ShowErrorMessage {
                    message: "@error.mapnotfound",
                },
            ]
        );

        assert_eq!(
            EditorMapsDialog::secondary_action_plan(&custom, false),
            vec![EditorMapsDialogAction::ShowConfirmFormatted {
                title: "@confirm",
                bundle_key: "map.delete",
                value: "Arena".into(),
                confirmed_actions: vec![
                    EditorMapsDialogAction::RemoveMap {
                        map_file: "maps/custom/arena.msav".into(),
                        map_name: "Arena".into(),
                    },
                    EditorMapsDialogAction::HideMapInfoDialog,
                    EditorMapsDialogAction::SetupMapList,
                ],
            }]
        );

        custom.workshop = true;
        assert_eq!(
            EditorMapsDialog::secondary_action_plan(&custom, true),
            vec![EditorMapsDialogAction::ViewListing {
                map_file: "maps/custom/arena.msav".into(),
                map_name: "Arena".into(),
            }]
        );
    }

    #[test]
    fn show_map_and_resize_keep_map_list_active_dialog_lifecycle() {
        let mut dialog = EditorMapsDialog::new();
        let maps = maps();
        dialog.setup(&maps, &planets(), &MapListDialogContext::default());
        assert_eq!(
            dialog.show_map_plan(&maps.custom[0]),
            EditorMapsDialogAction::ShowMapInfoDialog {
                map_file: "maps/custom/arena.msav".into(),
                map_name: "Arena".into(),
            }
        );
        assert!(dialog.list.active_dialog_open);

        let (actions, resized) =
            dialog.on_resize(&maps, &planets(), &MapListDialogContext::default());
        assert_eq!(
            actions,
            vec![
                MapListDialogAction::HideActiveDialog,
                MapListDialogAction::RebuildMaps,
            ]
        );
        assert!(!dialog.list.active_dialog_open);
        assert_eq!(
            resized
                .list
                .bottom_buttons
                .iter()
                .map(|button| button.text)
                .collect::<Vec<_>>(),
            vec!["@back", "@editor.newmap", "@editor.importmap"]
        );
    }
}
