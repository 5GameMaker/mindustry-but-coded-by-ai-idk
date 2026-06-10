//! Schematics dialog model mirroring upstream `mindustry.ui.dialogs.SchematicsDialog`.

use std::collections::BTreeMap;

use crate::mindustry::{game::Schematic, r#type::ItemStack, ui::format_amount};

pub const SCHEMATICS_DIALOG_TITLE: &str = "@schematics";
pub const SCHEMATICS_TAG_HEIGHT: f32 = 42.0;
pub const SCHEMATICS_SEARCH_ICON: &str = "zoom";
pub const SCHEMATICS_SEARCH_MESSAGE: &str = "@schematic.search";
pub const SCHEMATICS_SEARCH_PAD_BOTTOM: f32 = 4.0;
pub const SCHEMATICS_TAGS_TEXT: &str = "@schematic.tags";
pub const SCHEMATICS_TAGS_PAD_RIGHT: f32 = 4.0;
pub const SCHEMATICS_TAG_BUTTON_STYLE: &str = "Styles.togglet";
pub const SCHEMATICS_TAG_BUTTON_PAD: f32 = 2.0;
pub const SCHEMATICS_TAG_EDIT_ICON: &str = "pencilSmall";
pub const SCHEMATICS_TAG_EDIT_TOOLTIP: &str = "@schematic.edittags";
pub const SCHEMATICS_IMPORT_TEXT: &str = "@schematic.import";
pub const SCHEMATICS_IMPORT_ICON: &str = "download";
pub const SCHEMATICS_CARD_COLUMN_WIDTH: f32 = 230.0;
pub const SCHEMATICS_CARD_PAD: f32 = 4.0;
pub const SCHEMATICS_CARD_STYLE: &str = "Styles.flati";
pub const SCHEMATICS_CARD_UP: &str = "Tex.pane";
pub const SCHEMATICS_CARD_BUTTON_SIZE: f32 = 50.0;
pub const SCHEMATICS_CARD_BUTTON_STYLE: &str = "Styles.emptyi";
pub const SCHEMATICS_CARD_PREVIEW_SIZE: f32 = 200.0;
pub const SCHEMATICS_CARD_NAME_BACKGROUND: &str = "Styles.black3";
pub const SCHEMATICS_CARD_NAME_STYLE: &str = "Styles.outlineLabel";
pub const SCHEMATICS_CARD_NAME_COLOR: &str = "Color.white";
pub const SCHEMATICS_CARD_NAME_MAX_WIDTH: f32 = 192.0;
pub const SCHEMATICS_CARD_NAME_PAD: f32 = 4.0;
pub const SCHEMATICS_IMAGE_BACKGROUND: &str = "sprites/schematic-background.png";
pub const SCHEMATICS_IMAGE_SCALING: &str = "Scaling.fit";
pub const SCHEMATICS_IMAGE_REPEAT_SCALING: f32 = 16.0;
pub const SCHEMATICS_IMAGE_BORDER_THICKNESS: f32 = 4.0;
pub const SCHEMATICS_IMAGE_BORDER_COLOR: &str = "Pal.gray";
pub const SCHEMATICS_IMAGE_BORDER_HOVER_COLOR: &str = "Pal.accent";
pub const SCHEMATICS_IMAGE_LOADING_ICON: &str = "refresh";
pub const SCHEMATICS_NONE_FOUND: &str = "@none.found";
pub const SCHEMATICS_NONE: &str = "@none";
pub const SCHEMATICS_NONE_COLOR: &str = "Color.lightGray";
pub const SCHEMATICS_DISABLED_TEXT: &str = "@schematic.disabled";
pub const SCHEMATICS_BASE64_PREFIX: &str = "bXNjaA";
pub const SCHEMATICS_EXTENSION: &str = "msch";
pub const SCHEMATICS_SAVED_TEXT: &str = "@schematic.saved";
pub const SCHEMATICS_COPIED_TEXT: &str = "@copied";
pub const SCHEMATICS_CONFIRM_TITLE: &str = "@confirm";
pub const SCHEMATICS_DELETE_CONFIRM: &str = "@schematic.delete.confirm";
pub const SCHEMATICS_IMPORT_DIALOG_TITLE: &str = "@editor.import";
pub const SCHEMATICS_EXPORT_DIALOG_TITLE: &str = "@editor.export";
pub const SCHEMATICS_EDIT_DIALOG_TITLE: &str = "@schematic.edit";
pub const SCHEMATICS_COPY_IMPORT_TEXT: &str = "@schematic.copy.import";
pub const SCHEMATICS_COPY_TEXT: &str = "@schematic.copy";
pub const SCHEMATICS_IMPORT_FILE_TEXT: &str = "@schematic.importfile";
pub const SCHEMATICS_EXPORT_FILE_TEXT: &str = "@schematic.exportfile";
pub const SCHEMATICS_BROWSE_WORKSHOP_TEXT: &str = "@schematic.browseworkshop";
pub const SCHEMATICS_SHARE_WORKSHOP_TEXT: &str = "@schematic.shareworkshop";
pub const SCHEMATICS_WORKSHOP_ICON: &str = "book";
pub const SCHEMATICS_COPY_ICON: &str = "copy";
pub const SCHEMATICS_EXPORT_ICON: &str = "export";
pub const SCHEMATICS_DOWNLOAD_ICON: &str = "download";
pub const SCHEMATICS_IMPORT_EXPORT_BUTTON_SIZE: (f32, f32) = (280.0, 60.0);
pub const SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE: &str = "Styles.flatt";
pub const SCHEMATICS_IMPORT_EXPORT_MARGIN_LEFT: f32 = 12.0;
pub const SCHEMATICS_EDIT_FILL_PARENT: bool = true;
pub const SCHEMATICS_EDIT_MARGIN: f32 = 30.0;
pub const SCHEMATICS_EDIT_NAME_LABEL: &str = "@name";
pub const SCHEMATICS_EDIT_DESCRIPTION_LABEL: &str = "@editor.description";
pub const SCHEMATICS_EDIT_FIELD_SIZE: (f32, f32) = (400.0, 55.0);
pub const SCHEMATICS_EDIT_AREA_SIZE: (f32, f32) = (400.0, 140.0);
pub const SCHEMATICS_EDIT_TAGS_MAX_WIDTH: f32 = 400.0;
pub const SCHEMATICS_EDIT_BUTTON_SIZE: (f32, f32) = (210.0, 64.0);
pub const SCHEMATICS_EDIT_BUTTON_PAD: f32 = 4.0;
pub const SCHEMATICS_OK_TEXT: &str = "@ok";
pub const SCHEMATICS_OK_ICON: &str = "ok";
pub const SCHEMATICS_CANCEL_TEXT: &str = "@cancel";
pub const SCHEMATICS_CANCEL_ICON: &str = "cancel";
pub const SCHEMATICS_ADD_TAG_TITLE: &str = "@schematic.addtag";
pub const SCHEMATICS_TAG_EXISTS_TEXT: &str = "@schematic.tagexists";
pub const SCHEMATICS_RENAME_TAG_TITLE: &str = "@schematic.renametag";
pub const SCHEMATICS_TAG_DELETE_CONFIRM: &str = "@schematic.tagdelconfirm";
pub const SCHEMATICS_TEXT_TAG_TEXT: &str = "@schematic.texttag";
pub const SCHEMATICS_ICON_TAG_TEXT: &str = "@schematic.icontag";
pub const SCHEMATICS_ADD_ICON: &str = "add";
pub const SCHEMATICS_ADD_SMALL_ICON: &str = "addSmall";
pub const SCHEMATICS_CANCEL_SMALL_ICON: &str = "cancelSmall";
pub const SCHEMATICS_MOVE_UP_ICON: &str = "upOpen";
pub const SCHEMATICS_MOVE_DOWN_ICON: &str = "downOpen";
pub const SCHEMATICS_MOVE_UP_TOOLTIP: &str = "@editor.moveup";
pub const SCHEMATICS_MOVE_DOWN_TOOLTIP: &str = "@editor.movedown";
pub const SCHEMATICS_RENAME_ICON: &str = "pencil";
pub const SCHEMATICS_DELETE_ICON: &str = "trash";
pub const SCHEMATICS_DELETE_TOOLTIP: &str = "@save.delete";
pub const SCHEMATICS_TAGGED_KEY: &str = "schematic.tagged";
pub const SCHEMATICS_TAG_ENTRY_BACKGROUND: &str = "Tex.whiteui";
pub const SCHEMATICS_TAG_ENTRY_COLOR: &str = "Pal.gray";
pub const SCHEMATICS_TAG_ENTRY_MARGIN: f32 = 5.0;
pub const SCHEMATICS_TAG_ENTRY_MIN_WIDTH: f32 = 210.0;
pub const SCHEMATICS_TAG_ENTRY_PAD: f32 = 4.0;
pub const SCHEMATICS_INFO_TITLE_PREFIX: &str = "[[@schematic] ";
pub const SCHEMATICS_INFO_FILL_PARENT: bool = true;
pub const SCHEMATICS_INFO_TEXT_KEY: &str = "schematic.info";
pub const SCHEMATICS_INFO_TEXT_COLOR: &str = "Color.lightGray";
pub const SCHEMATICS_INFO_TAGS_PAD: f32 = 6.0;
pub const SCHEMATICS_INFO_IMAGE_MAX_SIZE: f32 = 800.0;
pub const SCHEMATICS_INFO_REQUIREMENTS_PAD: f32 = 6.0;
pub const SCHEMATICS_INFO_REQUIREMENTS_COLUMNS: usize = 4;
pub const SCHEMATICS_INFO_REQUIREMENT_ICON_SIZE: f32 = 32.0;
pub const SCHEMATICS_INFO_REQUIREMENT_PAD_LEFT: f32 = 2.0;
pub const SCHEMATICS_INFO_REQUIREMENT_PAD_RIGHT: f32 = 4.0;
pub const SCHEMATICS_POWER_ICON: &str = "powerSmall";
pub const SCHEMATICS_POWER_PRODUCTION_COLOR: &str = "Pal.powerLight";
pub const SCHEMATICS_POWER_CONSUMPTION_COLOR: &str = "Pal.remove";
pub const SCHEMATICS_POWER_SPACER: f32 = 15.0;
pub const SCHEMATICS_DESCRIPTION_COLOR_PREFIX: &str = "[lightgray]";
pub const SCHEMATICS_DESCRIPTION_PAD_TOP: f32 = 20.0;
pub const SCHEMATICS_DESCRIPTION_MAX_WIDTH: f32 = 500.0;
pub const SCHEMATICS_DESCRIPTION_PAD_LEFT: f32 = 8.0;
pub const SCHEMATICS_DESCRIPTION_PAD_RIGHT: f32 = 8.0;
pub const SCHEMATICS_INFO_BUTTON_DESKTOP_SIZE: (f32, f32) = (210.0, 64.0);
pub const SCHEMATICS_INFO_BUTTON_PORTRAIT_SIZE: (f32, f32) = (150.0, 64.0);
pub const SCHEMATICS_BACK_TEXT: &str = "@back";
pub const SCHEMATICS_BACK_ICON: &str = "left";
pub const SCHEMATICS_EDIT_TEXT: &str = "@edit";
pub const SCHEMATICS_EDIT_ICON: &str = "edit";

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsDialogContext {
    pub graphics_width: f32,
    pub scl: f32,
    pub steam: bool,
    pub desktop: bool,
    pub portrait: bool,
    pub state_is_menu: bool,
    pub schematics_allowed: bool,
    pub search_field_focused: bool,
    pub clipboard_text: Option<String>,
    pub core_items: BTreeMap<String, i32>,
    pub infinite_resources: bool,
}

impl Default for SchematicsDialogContext {
    fn default() -> Self {
        Self {
            graphics_width: 960.0,
            scl: 1.0,
            steam: false,
            desktop: true,
            portrait: false,
            state_is_menu: true,
            schematics_allowed: true,
            search_field_focused: false,
            clipboard_text: None,
            core_items: BTreeMap::new(),
            infinite_resources: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsDialogSchematic {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub labels: Vec<String>,
    pub width: i32,
    pub height: i32,
    pub tile_count: usize,
    pub has_steam_id: bool,
    pub mod_display_name: Option<String>,
    pub requirements: Vec<ItemStack>,
    pub power_consumption: f32,
    pub power_production: f32,
    pub preview_ready: bool,
}

impl SchematicsDialogSchematic {
    pub fn from_schematic(id: usize, schematic: &Schematic) -> Self {
        Self {
            id,
            name: schematic.name(),
            description: schematic.description(),
            labels: schematic.labels.clone(),
            width: schematic.width,
            height: schematic.height,
            tile_count: schematic.tiles.len(),
            has_steam_id: schematic.tags.contains_key("steamid"),
            mod_display_name: schematic.r#mod.clone(),
            requirements: Vec::new(),
            power_consumption: 0.0,
            power_production: 0.0,
            preview_ready: false,
        }
    }

    pub fn with_requirements(mut self, requirements: Vec<ItemStack>) -> Self {
        self.requirements = requirements;
        self
    }

    pub fn with_power(mut self, production: f32, consumption: f32) -> Self {
        self.power_production = production;
        self.power_consumption = consumption;
        self
    }

    pub fn preview_ready(mut self, preview_ready: bool) -> Self {
        self.preview_ready = preview_ready;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsDialogModel {
    pub title: &'static str,
    pub should_pause: bool,
    pub close_button_added: bool,
    pub import_button: SchematicsTopButton,
    pub search: SchematicsSearchModel,
    pub tag_bar: SchematicsTagBarModel,
    pub pane: SchematicsPaneModel,
    pub focus_search_on_desktop_show: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchematicsTopButton {
    pub text: &'static str,
    pub icon: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsSearchModel {
    pub icon: &'static str,
    pub text: String,
    pub message_text: &'static str,
    pub pad_bottom: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsTagBarModel {
    pub label: &'static str,
    pub label_pad_right: f32,
    pub height: f32,
    pub edit_button: SchematicsIconButton,
    pub chips: Vec<SchematicsTagChip>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsTagChip {
    pub tag: String,
    pub style: &'static str,
    pub checked: bool,
    pub pad: f32,
    pub height: f32,
    pub wrap_label: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsIconButton {
    pub icon: &'static str,
    pub size: f32,
    pub pad: f32,
    pub tooltip: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsPaneModel {
    pub columns: usize,
    pub cards: Vec<SchematicsCardModel>,
    pub first_schematic: Option<usize>,
    pub empty_text: Option<&'static str>,
    pub empty_color: Option<&'static str>,
    pub scroll_x: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsCardModel {
    pub schematic: usize,
    pub name: String,
    pub labels: Vec<String>,
    pub row: usize,
    pub column: usize,
    pub pad: f32,
    pub style: &'static str,
    pub up: &'static str,
    pub top_aligned: bool,
    pub margin: f32,
    pub action_buttons: Vec<SchematicsCardActionButton>,
    pub image: SchematicImageModel,
    pub name_overlay: SchematicsNameOverlay,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsCardActionButton {
    pub kind: SchematicsCardActionKind,
    pub icon: &'static str,
    pub style: &'static str,
    pub size: f32,
    pub tooltip: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchematicsCardActionKind {
    Info,
    Export,
    Edit,
    ViewWorkshop,
    Delete,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicImageModel {
    pub background: &'static str,
    pub scaling: &'static str,
    pub repeat_scaling: f32,
    pub thickness: f32,
    pub border_color: &'static str,
    pub hover_border_color: &'static str,
    pub preview_ready: bool,
    pub loading_icon: &'static str,
    pub size: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsNameOverlay {
    pub background: &'static str,
    pub label_style: &'static str,
    pub color: &'static str,
    pub max_width: f32,
    pub pad: f32,
    pub ellipsis: bool,
    pub alignment: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsImportDialogModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub buttons: Vec<SchematicsDialogButton>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsExportDialogModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub buttons: Vec<SchematicsDialogButton>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsDialogButton {
    pub text: &'static str,
    pub icon: &'static str,
    pub style: &'static str,
    pub size: (f32, f32),
    pub margin_left: f32,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsEditDialogModel {
    pub title: &'static str,
    pub fill_parent: bool,
    pub close_listener_added: bool,
    pub margin: f32,
    pub tags: SchematicsSchematicTagsModel,
    pub name_label: &'static str,
    pub name_text: String,
    pub name_field_size: (f32, f32),
    pub description_label: &'static str,
    pub description_text: String,
    pub description_area_size: (f32, f32),
    pub buttons: Vec<SchematicsDialogButton>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsSchematicTagsModel {
    pub show_label: bool,
    pub label: &'static str,
    pub label_pad_right: f32,
    pub chips: Vec<SchematicsSchematicTagChip>,
    pub add_button: SchematicsIconButton,
    pub max_width: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsSchematicTagChip {
    pub tag: String,
    pub background: &'static str,
    pub cancel_icon: &'static str,
    pub height: f32,
    pub pad_right: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsAllTagsDialogModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub rows: Vec<SchematicsAllTagRow>,
    pub add_buttons: Vec<SchematicsDialogButton>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsAllTagRow {
    pub tag: String,
    pub background: &'static str,
    pub color: &'static str,
    pub margin: f32,
    pub min_width: f32,
    pub pad: f32,
    pub tagged_count_key: &'static str,
    pub tagged_count: usize,
    pub move_up: SchematicsIconButton,
    pub move_down: SchematicsIconButton,
    pub rename: SchematicsIconButton,
    pub delete: SchematicsIconButton,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsInfoDialogModel {
    pub title: String,
    pub fill_parent: bool,
    pub close_listener_added: bool,
    pub info: SchematicsInfoLine,
    pub tags: SchematicsSchematicTagsModel,
    pub image: SchematicImageModel,
    pub requirements: Vec<SchematicsRequirementRow>,
    pub power: Option<SchematicsPowerModel>,
    pub description: Option<SchematicsDescriptionModel>,
    pub buttons: Vec<SchematicsDialogButton>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchematicsInfoLine {
    pub bundle_key: &'static str,
    pub width: i32,
    pub height: i32,
    pub tiles: usize,
    pub color: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsRequirementRow {
    pub item: String,
    pub icon: String,
    pub icon_size: f32,
    pub amount_text: String,
    pub pad_left: f32,
    pub pad_right: f32,
    pub row_after: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsPowerModel {
    pub production: Option<SchematicsPowerEntry>,
    pub consumption: Option<SchematicsPowerEntry>,
    pub spacer_width: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsPowerEntry {
    pub icon: &'static str,
    pub color: &'static str,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicsDescriptionModel {
    pub text: String,
    pub pad_top: f32,
    pub max_width: f32,
    pub pad_left: f32,
    pub pad_right: f32,
    pub wrap: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchematicsDialogAction {
    RebuildPane,
    RebuildTags,
    Setup,
    HideDialog,
    ShowImportDialog,
    ShowExportDialog {
        schematic: usize,
    },
    ShowEditDialog {
        schematic: usize,
    },
    ShowInfoDialog {
        schematic: usize,
    },
    ViewWorkshop {
        schematic: usize,
    },
    ShowInfo {
        text: String,
    },
    ShowInfoFade {
        text: &'static str,
    },
    ShowException {
        message: String,
    },
    ShowConfirm {
        title: &'static str,
        text: &'static str,
        schematic: usize,
    },
    DeleteSchematic {
        schematic: usize,
    },
    UseSchematic {
        schematic: usize,
    },
    SetClipboardSchematic {
        schematic: usize,
    },
    ImportClipboardSchematic,
    ImportFileChooser {
        extension: &'static str,
    },
    ExportFile {
        schematic: usize,
        extension: &'static str,
    },
    PublishWorkshop {
        schematic: usize,
    },
    OpenWorkshop,
    SaveSchematic {
        schematic: usize,
    },
    SaveTags {
        tags: Vec<String>,
    },
    ShowTextInput {
        title: &'static str,
        message: &'static str,
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchematicsDialog {
    pub search: String,
    pub tags: Vec<String>,
    pub selected_tags: Vec<String>,
    pub checked_tags: bool,
    pub first_schematic: Option<usize>,
}

impl Default for SchematicsDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SchematicsDialog {
    pub fn new() -> Self {
        Self {
            search: String::new(),
            tags: Vec::new(),
            selected_tags: Vec::new(),
            checked_tags: false,
            first_schematic: None,
        }
    }

    pub fn with_tags(tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            tags: tags.into_iter().map(Into::into).collect(),
            ..Self::new()
        }
    }

    pub fn setup(
        &mut self,
        schematics: &[SchematicsDialogSchematic],
        context: &SchematicsDialogContext,
    ) -> SchematicsDialogModel {
        if !self.checked_tags {
            self.check_tags(schematics);
            self.checked_tags = true;
        }
        self.search.clear();
        self.model(schematics, context)
    }

    pub fn model(
        &mut self,
        schematics: &[SchematicsDialogSchematic],
        context: &SchematicsDialogContext,
    ) -> SchematicsDialogModel {
        let pane = self.pane_model(schematics, context);
        SchematicsDialogModel {
            title: SCHEMATICS_DIALOG_TITLE,
            should_pause: true,
            close_button_added: true,
            import_button: SchematicsTopButton {
                text: SCHEMATICS_IMPORT_TEXT,
                icon: SCHEMATICS_IMPORT_ICON,
            },
            search: SchematicsSearchModel {
                icon: SCHEMATICS_SEARCH_ICON,
                text: self.search.clone(),
                message_text: SCHEMATICS_SEARCH_MESSAGE,
                pad_bottom: SCHEMATICS_SEARCH_PAD_BOTTOM,
            },
            tag_bar: self.tag_bar_model(),
            pane,
            focus_search_on_desktop_show: context.desktop,
        }
    }

    pub fn set_search_plan(&mut self, search: impl Into<String>) -> Vec<SchematicsDialogAction> {
        self.search = search.into();
        vec![SchematicsDialogAction::RebuildPane]
    }

    pub fn clear_search_plan(&mut self) -> Vec<SchematicsDialogAction> {
        if self.search.is_empty() {
            Vec::new()
        } else {
            self.search.clear();
            vec![SchematicsDialogAction::RebuildPane]
        }
    }

    pub fn toggle_tag_plan(&mut self, tag: &str) -> Vec<SchematicsDialogAction> {
        if let Some(index) = self.selected_tags.iter().position(|value| value == tag) {
            self.selected_tags.remove(index);
        } else {
            self.selected_tags.push(tag.to_string());
        }
        vec![SchematicsDialogAction::RebuildPane]
    }

    pub fn search_submit_plan(
        &self,
        context: &SchematicsDialogContext,
    ) -> Vec<SchematicsDialogAction> {
        if !context.search_field_focused {
            return Vec::new();
        }
        let Some(schematic) = self.first_schematic else {
            return Vec::new();
        };
        if !context.schematics_allowed {
            vec![SchematicsDialogAction::ShowInfo {
                text: SCHEMATICS_DISABLED_TEXT.into(),
            }]
        } else {
            vec![
                SchematicsDialogAction::UseSchematic { schematic },
                SchematicsDialogAction::HideDialog,
            ]
        }
    }

    pub fn card_click_plan(
        &self,
        schematic: &SchematicsDialogSchematic,
        children_pressed: bool,
        context: &SchematicsDialogContext,
    ) -> Vec<SchematicsDialogAction> {
        if children_pressed {
            return Vec::new();
        }
        if context.state_is_menu {
            vec![SchematicsDialogAction::ShowInfoDialog {
                schematic: schematic.id,
            }]
        } else if !context.schematics_allowed {
            vec![SchematicsDialogAction::ShowInfo {
                text: SCHEMATICS_DISABLED_TEXT.into(),
            }]
        } else {
            vec![
                SchematicsDialogAction::UseSchematic {
                    schematic: schematic.id,
                },
                SchematicsDialogAction::HideDialog,
            ]
        }
    }

    pub fn card_action_plan(
        &self,
        schematic: &SchematicsDialogSchematic,
        kind: SchematicsCardActionKind,
    ) -> Vec<SchematicsDialogAction> {
        match kind {
            SchematicsCardActionKind::Info => vec![SchematicsDialogAction::ShowInfoDialog {
                schematic: schematic.id,
            }],
            SchematicsCardActionKind::Export => vec![SchematicsDialogAction::ShowExportDialog {
                schematic: schematic.id,
            }],
            SchematicsCardActionKind::Edit => vec![SchematicsDialogAction::ShowEditDialog {
                schematic: schematic.id,
            }],
            SchematicsCardActionKind::ViewWorkshop => {
                vec![SchematicsDialogAction::ViewWorkshop {
                    schematic: schematic.id,
                }]
            }
            SchematicsCardActionKind::Delete => {
                if let Some(mod_name) = &schematic.mod_display_name {
                    vec![SchematicsDialogAction::ShowInfo {
                        text: format!("mod.item.remove:{mod_name}"),
                    }]
                } else {
                    vec![SchematicsDialogAction::ShowConfirm {
                        title: SCHEMATICS_CONFIRM_TITLE,
                        text: SCHEMATICS_DELETE_CONFIRM,
                        schematic: schematic.id,
                    }]
                }
            }
        }
    }

    pub fn confirm_delete_plan(&self, schematic: usize) -> Vec<SchematicsDialogAction> {
        vec![
            SchematicsDialogAction::DeleteSchematic { schematic },
            SchematicsDialogAction::RebuildPane,
        ]
    }

    pub fn show_import_model(
        &self,
        context: &SchematicsDialogContext,
    ) -> SchematicsImportDialogModel {
        let clipboard_valid = context
            .clipboard_text
            .as_deref()
            .is_some_and(|text| text.starts_with(SCHEMATICS_BASE64_PREFIX));
        let mut buttons = vec![
            SchematicsDialogButton {
                text: SCHEMATICS_COPY_IMPORT_TEXT,
                icon: SCHEMATICS_COPY_ICON,
                style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                size: SCHEMATICS_IMPORT_EXPORT_BUTTON_SIZE,
                margin_left: SCHEMATICS_IMPORT_EXPORT_MARGIN_LEFT,
                disabled: !clipboard_valid,
            },
            SchematicsDialogButton {
                text: SCHEMATICS_IMPORT_FILE_TEXT,
                icon: SCHEMATICS_DOWNLOAD_ICON,
                style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                size: SCHEMATICS_IMPORT_EXPORT_BUTTON_SIZE,
                margin_left: SCHEMATICS_IMPORT_EXPORT_MARGIN_LEFT,
                disabled: false,
            },
        ];
        if context.steam {
            buttons.push(SchematicsDialogButton {
                text: SCHEMATICS_BROWSE_WORKSHOP_TEXT,
                icon: SCHEMATICS_WORKSHOP_ICON,
                style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                size: SCHEMATICS_IMPORT_EXPORT_BUTTON_SIZE,
                margin_left: SCHEMATICS_IMPORT_EXPORT_MARGIN_LEFT,
                disabled: false,
            });
        }
        SchematicsImportDialogModel {
            title: SCHEMATICS_IMPORT_DIALOG_TITLE,
            close_button_added: true,
            buttons,
        }
    }

    pub fn import_button_plan(&self) -> Vec<SchematicsDialogAction> {
        vec![SchematicsDialogAction::ShowImportDialog]
    }

    pub fn import_copy_plan(&mut self) -> Vec<SchematicsDialogAction> {
        vec![
            SchematicsDialogAction::HideDialog,
            SchematicsDialogAction::ImportClipboardSchematic,
            SchematicsDialogAction::Setup,
            SchematicsDialogAction::ShowInfoFade {
                text: SCHEMATICS_SAVED_TEXT,
            },
        ]
    }

    pub fn import_file_plan(&self) -> Vec<SchematicsDialogAction> {
        vec![
            SchematicsDialogAction::HideDialog,
            SchematicsDialogAction::ImportFileChooser {
                extension: SCHEMATICS_EXTENSION,
            },
        ]
    }

    pub fn browse_workshop_plan(&self) -> Vec<SchematicsDialogAction> {
        vec![
            SchematicsDialogAction::HideDialog,
            SchematicsDialogAction::OpenWorkshop,
        ]
    }

    pub fn show_export_model(
        &self,
        schematic: &SchematicsDialogSchematic,
        context: &SchematicsDialogContext,
    ) -> SchematicsExportDialogModel {
        let mut buttons = Vec::new();
        if context.steam && !schematic.has_steam_id {
            buttons.push(SchematicsDialogButton {
                text: SCHEMATICS_SHARE_WORKSHOP_TEXT,
                icon: SCHEMATICS_WORKSHOP_ICON,
                style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                size: SCHEMATICS_IMPORT_EXPORT_BUTTON_SIZE,
                margin_left: SCHEMATICS_IMPORT_EXPORT_MARGIN_LEFT,
                disabled: false,
            });
        }
        buttons.extend([
            SchematicsDialogButton {
                text: SCHEMATICS_COPY_TEXT,
                icon: SCHEMATICS_COPY_ICON,
                style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                size: SCHEMATICS_IMPORT_EXPORT_BUTTON_SIZE,
                margin_left: SCHEMATICS_IMPORT_EXPORT_MARGIN_LEFT,
                disabled: false,
            },
            SchematicsDialogButton {
                text: SCHEMATICS_EXPORT_FILE_TEXT,
                icon: SCHEMATICS_EXPORT_ICON,
                style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                size: SCHEMATICS_IMPORT_EXPORT_BUTTON_SIZE,
                margin_left: SCHEMATICS_IMPORT_EXPORT_MARGIN_LEFT,
                disabled: false,
            },
        ]);
        SchematicsExportDialogModel {
            title: SCHEMATICS_EXPORT_DIALOG_TITLE,
            close_button_added: true,
            buttons,
        }
    }

    pub fn export_action_plan(
        &self,
        schematic: &SchematicsDialogSchematic,
        text: &'static str,
    ) -> Vec<SchematicsDialogAction> {
        match text {
            SCHEMATICS_SHARE_WORKSHOP_TEXT => vec![
                SchematicsDialogAction::HideDialog,
                SchematicsDialogAction::PublishWorkshop {
                    schematic: schematic.id,
                },
            ],
            SCHEMATICS_COPY_TEXT => vec![
                SchematicsDialogAction::HideDialog,
                SchematicsDialogAction::ShowInfoFade {
                    text: SCHEMATICS_COPIED_TEXT,
                },
                SchematicsDialogAction::SetClipboardSchematic {
                    schematic: schematic.id,
                },
            ],
            SCHEMATICS_EXPORT_FILE_TEXT => vec![
                SchematicsDialogAction::HideDialog,
                SchematicsDialogAction::ExportFile {
                    schematic: schematic.id,
                    extension: SCHEMATICS_EXTENSION,
                },
            ],
            _ => Vec::new(),
        }
    }

    pub fn show_edit_model(
        &self,
        schematic: &SchematicsDialogSchematic,
    ) -> SchematicsEditDialogModel {
        SchematicsEditDialogModel {
            title: SCHEMATICS_EDIT_DIALOG_TITLE,
            fill_parent: SCHEMATICS_EDIT_FILL_PARENT,
            close_listener_added: true,
            margin: SCHEMATICS_EDIT_MARGIN,
            tags: self.schematic_tags_model(schematic, false, Some(SCHEMATICS_EDIT_TAGS_MAX_WIDTH)),
            name_label: SCHEMATICS_EDIT_NAME_LABEL,
            name_text: schematic.name.clone(),
            name_field_size: SCHEMATICS_EDIT_FIELD_SIZE,
            description_label: SCHEMATICS_EDIT_DESCRIPTION_LABEL,
            description_text: schematic.description.clone(),
            description_area_size: SCHEMATICS_EDIT_AREA_SIZE,
            buttons: vec![
                SchematicsDialogButton {
                    text: SCHEMATICS_OK_TEXT,
                    icon: SCHEMATICS_OK_ICON,
                    style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                    size: SCHEMATICS_EDIT_BUTTON_SIZE,
                    margin_left: 0.0,
                    disabled: schematic.name.is_empty(),
                },
                SchematicsDialogButton {
                    text: SCHEMATICS_CANCEL_TEXT,
                    icon: SCHEMATICS_CANCEL_ICON,
                    style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                    size: SCHEMATICS_EDIT_BUTTON_SIZE,
                    margin_left: 0.0,
                    disabled: false,
                },
            ],
        }
    }

    pub fn accept_edit_plan(
        &self,
        schematic: &mut SchematicsDialogSchematic,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Vec<SchematicsDialogAction> {
        schematic.name = name.into();
        schematic.description = description.into();
        vec![
            SchematicsDialogAction::SaveSchematic {
                schematic: schematic.id,
            },
            SchematicsDialogAction::HideDialog,
            SchematicsDialogAction::RebuildPane,
        ]
    }

    pub fn edit_enter_plan(
        &self,
        name_text: &str,
        description_focused: bool,
        schematic: usize,
    ) -> Vec<SchematicsDialogAction> {
        if !name_text.is_empty() && !description_focused {
            vec![SchematicsDialogAction::SaveSchematic { schematic }]
        } else {
            Vec::new()
        }
    }

    pub fn check_tags(&mut self, schematics: &[SchematicsDialogSchematic]) {
        for schematic in schematics {
            self.check_tags_for_schematic(schematic);
        }
    }

    pub fn check_tags_for_schematic(&mut self, schematic: &SchematicsDialogSchematic) -> bool {
        let mut any = false;
        for tag in &schematic.labels {
            if !self.tags.contains(tag) {
                self.tags.push(tag.clone());
                any = true;
            }
        }
        any
    }

    pub fn add_tag_to_schematic_plan(
        &mut self,
        schematic: &mut SchematicsDialogSchematic,
        tag: impl Into<String>,
    ) -> Vec<SchematicsDialogAction> {
        let tag = tag.into();
        schematic.labels.push(tag);
        sort_labels_by_global_tags(&mut schematic.labels, &self.tags);
        self.tags_changed_plan(true, schematic.id)
    }

    pub fn remove_tag_from_schematic_plan(
        &mut self,
        schematic: &mut SchematicsDialogSchematic,
        tag: &str,
    ) -> Vec<SchematicsDialogAction> {
        schematic.labels.retain(|value| value != tag);
        self.tags_changed_plan(true, schematic.id)
    }

    pub fn show_new_tag_plan(&self, title: &'static str) -> Vec<SchematicsDialogAction> {
        vec![SchematicsDialogAction::ShowTextInput {
            title,
            message: "",
            text: String::new(),
        }]
    }

    pub fn add_global_tag_plan(&mut self, tag: impl Into<String>) -> Vec<SchematicsDialogAction> {
        let tag = tag.into();
        if self.tags.contains(&tag) {
            vec![SchematicsDialogAction::ShowInfo {
                text: SCHEMATICS_TAG_EXISTS_TEXT.into(),
            }]
        } else {
            self.tags.push(tag);
            vec![
                SchematicsDialogAction::RebuildTags,
                SchematicsDialogAction::SaveTags {
                    tags: self.tags.clone(),
                },
            ]
        }
    }

    pub fn show_all_tags_model(
        &self,
        schematics: &[SchematicsDialogSchematic],
    ) -> SchematicsAllTagsDialogModel {
        let rows = self
            .tags
            .iter()
            .map(|tag| SchematicsAllTagRow {
                tag: tag.clone(),
                background: SCHEMATICS_TAG_ENTRY_BACKGROUND,
                color: SCHEMATICS_TAG_ENTRY_COLOR,
                margin: SCHEMATICS_TAG_ENTRY_MARGIN,
                min_width: SCHEMATICS_TAG_ENTRY_MIN_WIDTH,
                pad: SCHEMATICS_TAG_ENTRY_PAD,
                tagged_count_key: SCHEMATICS_TAGGED_KEY,
                tagged_count: schematics
                    .iter()
                    .filter(|schematic| schematic.labels.contains(tag))
                    .count(),
                move_up: tag_icon_button(SCHEMATICS_MOVE_UP_ICON, SCHEMATICS_MOVE_UP_TOOLTIP),
                move_down: tag_icon_button(SCHEMATICS_MOVE_DOWN_ICON, SCHEMATICS_MOVE_DOWN_TOOLTIP),
                rename: tag_icon_button(SCHEMATICS_RENAME_ICON, SCHEMATICS_RENAME_TAG_TITLE),
                delete: tag_icon_button(SCHEMATICS_DELETE_ICON, SCHEMATICS_DELETE_TOOLTIP),
            })
            .collect();
        SchematicsAllTagsDialogModel {
            title: SCHEMATICS_TAG_EDIT_TOOLTIP,
            close_button_added: true,
            rows,
            add_buttons: vec![
                SchematicsDialogButton {
                    text: SCHEMATICS_TEXT_TAG_TEXT,
                    icon: SCHEMATICS_ADD_ICON,
                    style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                    size: (0.0, SCHEMATICS_TAG_HEIGHT),
                    margin_left: 0.0,
                    disabled: false,
                },
                SchematicsDialogButton {
                    text: SCHEMATICS_ICON_TAG_TEXT,
                    icon: SCHEMATICS_ADD_ICON,
                    style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                    size: (0.0, SCHEMATICS_TAG_HEIGHT),
                    margin_left: 0.0,
                    disabled: false,
                },
            ],
        }
    }

    pub fn move_tag_plan(&mut self, tag: &str, direction: i32) -> Vec<SchematicsDialogAction> {
        let Some(index) = self.tags.iter().position(|value| value == tag) else {
            return Vec::new();
        };
        let next = index as i32 + direction;
        if next < 0 || next >= self.tags.len() as i32 {
            return Vec::new();
        }
        self.tags.swap(index, next as usize);
        vec![
            SchematicsDialogAction::RebuildTags,
            SchematicsDialogAction::SaveTags {
                tags: self.tags.clone(),
            },
        ]
    }

    pub fn rename_tag_plan(
        &mut self,
        schematics: &mut [SchematicsDialogSchematic],
        tag: &str,
        result: impl Into<String>,
    ) -> Vec<SchematicsDialogAction> {
        let result = result.into();
        if result == tag {
            return Vec::new();
        }
        if self.tags.contains(&result) {
            return vec![SchematicsDialogAction::ShowInfo {
                text: SCHEMATICS_TAG_EXISTS_TEXT.into(),
            }];
        }
        let mut actions = Vec::new();
        for schematic in schematics {
            let mut changed = false;
            for label in &mut schematic.labels {
                if label == tag {
                    *label = result.clone();
                    changed = true;
                }
            }
            if changed {
                actions.push(SchematicsDialogAction::SaveSchematic {
                    schematic: schematic.id,
                });
            }
        }
        for selected in &mut self.selected_tags {
            if selected == tag {
                *selected = result.clone();
            }
        }
        if let Some(index) = self.tags.iter().position(|value| value == tag) {
            self.tags[index] = result;
        }
        actions.extend([
            SchematicsDialogAction::RebuildTags,
            SchematicsDialogAction::SaveTags {
                tags: self.tags.clone(),
            },
        ]);
        actions
    }

    pub fn delete_tag_plan(
        &mut self,
        schematics: &mut [SchematicsDialogSchematic],
        tag: &str,
    ) -> Vec<SchematicsDialogAction> {
        let mut actions = Vec::new();
        for schematic in schematics {
            let before = schematic.labels.len();
            schematic.labels.retain(|label| label != tag);
            if schematic.labels.len() != before {
                actions.push(SchematicsDialogAction::SaveSchematic {
                    schematic: schematic.id,
                });
            }
        }
        self.selected_tags.retain(|selected| selected != tag);
        self.tags.retain(|value| value != tag);
        actions.extend([
            SchematicsDialogAction::RebuildTags,
            SchematicsDialogAction::RebuildPane,
            SchematicsDialogAction::SaveTags {
                tags: self.tags.clone(),
            },
        ]);
        actions
    }

    pub fn info_model(
        &self,
        schematic: &SchematicsDialogSchematic,
        context: &SchematicsDialogContext,
    ) -> SchematicsInfoDialogModel {
        SchematicsInfoDialogModel {
            title: format!("{SCHEMATICS_INFO_TITLE_PREFIX}{}", schematic.name),
            fill_parent: SCHEMATICS_INFO_FILL_PARENT,
            close_listener_added: true,
            info: SchematicsInfoLine {
                bundle_key: SCHEMATICS_INFO_TEXT_KEY,
                width: schematic.width,
                height: schematic.height,
                tiles: schematic.tile_count,
                color: SCHEMATICS_INFO_TEXT_COLOR,
            },
            tags: self.schematic_tags_model(schematic, true, None),
            image: schematic_image(schematic),
            requirements: requirement_rows(schematic, context),
            power: power_model(schematic),
            description: (!schematic.description.is_empty()).then(|| SchematicsDescriptionModel {
                text: format!(
                    "{SCHEMATICS_DESCRIPTION_COLOR_PREFIX}{}",
                    schematic.description
                ),
                pad_top: SCHEMATICS_DESCRIPTION_PAD_TOP,
                max_width: SCHEMATICS_DESCRIPTION_MAX_WIDTH,
                pad_left: SCHEMATICS_DESCRIPTION_PAD_LEFT,
                pad_right: SCHEMATICS_DESCRIPTION_PAD_RIGHT,
                wrap: true,
            }),
            buttons: vec![
                SchematicsDialogButton {
                    text: SCHEMATICS_BACK_TEXT,
                    icon: SCHEMATICS_BACK_ICON,
                    style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                    size: if context.portrait {
                        SCHEMATICS_INFO_BUTTON_PORTRAIT_SIZE
                    } else {
                        SCHEMATICS_INFO_BUTTON_DESKTOP_SIZE
                    },
                    margin_left: 0.0,
                    disabled: false,
                },
                SchematicsDialogButton {
                    text: SCHEMATICS_EXPORT_DIALOG_TITLE,
                    icon: "upload",
                    style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                    size: if context.portrait {
                        SCHEMATICS_INFO_BUTTON_PORTRAIT_SIZE
                    } else {
                        SCHEMATICS_INFO_BUTTON_DESKTOP_SIZE
                    },
                    margin_left: 0.0,
                    disabled: false,
                },
                SchematicsDialogButton {
                    text: SCHEMATICS_EDIT_TEXT,
                    icon: SCHEMATICS_EDIT_ICON,
                    style: SCHEMATICS_IMPORT_EXPORT_BUTTON_STYLE,
                    size: if context.portrait {
                        SCHEMATICS_INFO_BUTTON_PORTRAIT_SIZE
                    } else {
                        SCHEMATICS_INFO_BUTTON_DESKTOP_SIZE
                    },
                    margin_left: 0.0,
                    disabled: false,
                },
            ],
        }
    }

    fn pane_model(
        &mut self,
        schematics: &[SchematicsDialogSchematic],
        context: &SchematicsDialogContext,
    ) -> SchematicsPaneModel {
        let columns = schematic_columns(context.graphics_width, context.scl);
        let search_key = schematic_search_key(&self.search);
        self.first_schematic = None;
        let mut cards = Vec::new();
        for schematic in schematics {
            if !self.selected_tags.is_empty()
                && !self
                    .selected_tags
                    .iter()
                    .all(|tag| schematic.labels.contains(tag))
            {
                continue;
            }
            if !self.search.is_empty()
                && !schematic_search_key(&schematic.name).contains(&search_key)
            {
                continue;
            }
            if self.first_schematic.is_none() {
                self.first_schematic = Some(schematic.id);
            }
            let index = cards.len();
            cards.push(SchematicsCardModel {
                schematic: schematic.id,
                name: schematic.name.clone(),
                labels: schematic.labels.clone(),
                row: index / columns,
                column: index % columns,
                pad: SCHEMATICS_CARD_PAD,
                style: SCHEMATICS_CARD_STYLE,
                up: SCHEMATICS_CARD_UP,
                top_aligned: true,
                margin: 0.0,
                action_buttons: card_action_buttons(schematic),
                image: schematic_image(schematic),
                name_overlay: SchematicsNameOverlay {
                    background: SCHEMATICS_CARD_NAME_BACKGROUND,
                    label_style: SCHEMATICS_CARD_NAME_STYLE,
                    color: SCHEMATICS_CARD_NAME_COLOR,
                    max_width: SCHEMATICS_CARD_NAME_MAX_WIDTH * context.scl,
                    pad: SCHEMATICS_CARD_NAME_PAD,
                    ellipsis: true,
                    alignment: "Align.center",
                },
            });
        }

        let empty_text = if cards.is_empty() {
            if !search_key.is_empty() || !self.selected_tags.is_empty() {
                Some(SCHEMATICS_NONE_FOUND)
            } else {
                Some(SCHEMATICS_NONE)
            }
        } else {
            None
        };
        SchematicsPaneModel {
            columns,
            cards,
            first_schematic: self.first_schematic,
            empty_text,
            empty_color: (empty_text == Some(SCHEMATICS_NONE)).then_some(SCHEMATICS_NONE_COLOR),
            scroll_x: false,
        }
    }

    fn tag_bar_model(&self) -> SchematicsTagBarModel {
        SchematicsTagBarModel {
            label: SCHEMATICS_TAGS_TEXT,
            label_pad_right: SCHEMATICS_TAGS_PAD_RIGHT,
            height: SCHEMATICS_TAG_HEIGHT,
            edit_button: SchematicsIconButton {
                icon: SCHEMATICS_TAG_EDIT_ICON,
                size: SCHEMATICS_TAG_HEIGHT,
                pad: SCHEMATICS_TAG_BUTTON_PAD,
                tooltip: SCHEMATICS_TAG_EDIT_TOOLTIP,
            },
            chips: self
                .tags
                .iter()
                .map(|tag| SchematicsTagChip {
                    tag: tag.clone(),
                    style: SCHEMATICS_TAG_BUTTON_STYLE,
                    checked: self.selected_tags.contains(tag),
                    pad: SCHEMATICS_TAG_BUTTON_PAD,
                    height: SCHEMATICS_TAG_HEIGHT,
                    wrap_label: false,
                })
                .collect(),
        }
    }

    fn schematic_tags_model(
        &self,
        schematic: &SchematicsDialogSchematic,
        show_label: bool,
        max_width: Option<f32>,
    ) -> SchematicsSchematicTagsModel {
        let mut labels = schematic.labels.clone();
        sort_labels_by_global_tags(&mut labels, &self.tags);
        SchematicsSchematicTagsModel {
            show_label,
            label: SCHEMATICS_TAGS_TEXT,
            label_pad_right: SCHEMATICS_TAGS_PAD_RIGHT,
            chips: labels
                .into_iter()
                .map(|tag| SchematicsSchematicTagChip {
                    tag,
                    background: SCHEMATICS_CARD_UP,
                    cancel_icon: SCHEMATICS_CANCEL_SMALL_ICON,
                    height: SCHEMATICS_TAG_HEIGHT,
                    pad_right: 4.0,
                })
                .collect(),
            add_button: SchematicsIconButton {
                icon: SCHEMATICS_ADD_SMALL_ICON,
                size: SCHEMATICS_TAG_HEIGHT,
                pad: 0.0,
                tooltip: SCHEMATICS_ADD_TAG_TITLE,
            },
            max_width,
        }
    }

    fn tags_changed_plan(
        &self,
        rebuild_tags: bool,
        schematic: usize,
    ) -> Vec<SchematicsDialogAction> {
        let mut actions = Vec::new();
        if rebuild_tags {
            actions.push(SchematicsDialogAction::RebuildTags);
        }
        if !self.selected_tags.is_empty() {
            actions.push(SchematicsDialogAction::RebuildPane);
        }
        actions.push(SchematicsDialogAction::SaveSchematic { schematic });
        actions.push(SchematicsDialogAction::SaveTags {
            tags: self.tags.clone(),
        });
        actions
    }
}

pub fn schematic_columns(graphics_width: f32, scl: f32) -> usize {
    ((graphics_width / (SCHEMATICS_CARD_COLUMN_WIDTH * scl)) as usize).max(1)
}

pub fn schematic_search_key(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|ch| {
            !matches!(
                ch,
                '`' | '~'
                    | '!'
                    | '@'
                    | '#'
                    | '$'
                    | '%'
                    | '^'
                    | '&'
                    | '*'
                    | '('
                    | ')'
                    | '-'
                    | '_'
                    | '='
                    | '+'
                    | '{'
                    | '}'
                    | '|'
                    | ';'
                    | ':'
                    | '\''
                    | '"'
                    | ','
                    | '<'
                    | '.'
                    | '>'
                    | '/'
                    | '?'
            )
        })
        .collect()
}

fn card_action_buttons(schematic: &SchematicsDialogSchematic) -> Vec<SchematicsCardActionButton> {
    let mut buttons = vec![
        SchematicsCardActionButton {
            kind: SchematicsCardActionKind::Info,
            icon: "info",
            style: SCHEMATICS_CARD_BUTTON_STYLE,
            size: SCHEMATICS_CARD_BUTTON_SIZE,
            tooltip: "@info.title",
        },
        SchematicsCardActionButton {
            kind: SchematicsCardActionKind::Export,
            icon: "upload",
            style: SCHEMATICS_CARD_BUTTON_STYLE,
            size: SCHEMATICS_CARD_BUTTON_SIZE,
            tooltip: "@editor.export",
        },
        SchematicsCardActionButton {
            kind: SchematicsCardActionKind::Edit,
            icon: "pencil",
            style: SCHEMATICS_CARD_BUTTON_STYLE,
            size: SCHEMATICS_CARD_BUTTON_SIZE,
            tooltip: "@schematic.edit",
        },
    ];
    if schematic.has_steam_id {
        buttons.push(SchematicsCardActionButton {
            kind: SchematicsCardActionKind::ViewWorkshop,
            icon: "link",
            style: SCHEMATICS_CARD_BUTTON_STYLE,
            size: SCHEMATICS_CARD_BUTTON_SIZE,
            tooltip: "@view.workshop",
        });
    } else {
        buttons.push(SchematicsCardActionButton {
            kind: SchematicsCardActionKind::Delete,
            icon: SCHEMATICS_DELETE_ICON,
            style: SCHEMATICS_CARD_BUTTON_STYLE,
            size: SCHEMATICS_CARD_BUTTON_SIZE,
            tooltip: SCHEMATICS_DELETE_TOOLTIP,
        });
    }
    buttons
}

fn schematic_image(schematic: &SchematicsDialogSchematic) -> SchematicImageModel {
    SchematicImageModel {
        background: SCHEMATICS_IMAGE_BACKGROUND,
        scaling: SCHEMATICS_IMAGE_SCALING,
        repeat_scaling: SCHEMATICS_IMAGE_REPEAT_SCALING,
        thickness: SCHEMATICS_IMAGE_BORDER_THICKNESS,
        border_color: SCHEMATICS_IMAGE_BORDER_COLOR,
        hover_border_color: SCHEMATICS_IMAGE_BORDER_HOVER_COLOR,
        preview_ready: schematic.preview_ready,
        loading_icon: SCHEMATICS_IMAGE_LOADING_ICON,
        size: SCHEMATICS_CARD_PREVIEW_SIZE,
    }
}

fn tag_icon_button(icon: &'static str, tooltip: &'static str) -> SchematicsIconButton {
    SchematicsIconButton {
        icon,
        size: 40.0,
        pad: 0.0,
        tooltip,
    }
}

fn sort_labels_by_global_tags(labels: &mut [String], tags: &[String]) {
    labels.sort_by_key(|label| {
        tags.iter()
            .position(|tag| tag == label)
            .unwrap_or(usize::MAX)
    });
}

fn requirement_rows(
    schematic: &SchematicsDialogSchematic,
    context: &SchematicsDialogContext,
) -> Vec<SchematicsRequirementRow> {
    schematic
        .requirements
        .iter()
        .enumerate()
        .map(|(index, stack)| {
            let owned = context.core_items.get(&stack.item).copied().unwrap_or(0);
            let enough =
                context.infinite_resources || context.state_is_menu || owned >= stack.amount;
            let amount_text = if enough {
                format!("[lightgray]{}", format_amount(stack.amount as i64))
            } else {
                format!(
                    "[scarlet]{}[lightgray]/{}",
                    format_amount(owned.min(stack.amount) as i64),
                    format_amount(stack.amount as i64)
                )
            };
            SchematicsRequirementRow {
                item: stack.item.clone(),
                icon: format!("item-{}-ui", stack.item),
                icon_size: SCHEMATICS_INFO_REQUIREMENT_ICON_SIZE,
                amount_text,
                pad_left: SCHEMATICS_INFO_REQUIREMENT_PAD_LEFT,
                pad_right: SCHEMATICS_INFO_REQUIREMENT_PAD_RIGHT,
                row_after: (index + 1) % SCHEMATICS_INFO_REQUIREMENTS_COLUMNS == 0,
            }
        })
        .collect()
}

fn power_model(schematic: &SchematicsDialogSchematic) -> Option<SchematicsPowerModel> {
    let prod = schematic.power_production * 60.0;
    let cons = schematic.power_consumption * 60.0;
    if prod.abs() <= f32::EPSILON && cons.abs() <= f32::EPSILON {
        return None;
    }
    Some(SchematicsPowerModel {
        production: (prod.abs() > f32::EPSILON).then(|| SchematicsPowerEntry {
            icon: SCHEMATICS_POWER_ICON,
            color: SCHEMATICS_POWER_PRODUCTION_COLOR,
            text: format!("+{}", fixed2(prod)),
        }),
        consumption: (cons.abs() > f32::EPSILON).then(|| SchematicsPowerEntry {
            icon: SCHEMATICS_POWER_ICON,
            color: SCHEMATICS_POWER_CONSUMPTION_COLOR,
            text: format!("-{}", fixed2(cons)),
        }),
        spacer_width: SCHEMATICS_POWER_SPACER,
    })
}

fn fixed2(value: f32) -> String {
    let mut out = format!("{value:.2}");
    while out.contains('.') && out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn schematic(id: usize, name: &str, labels: &[&str]) -> SchematicsDialogSchematic {
        let mut tags = HashMap::new();
        tags.insert("name".into(), name.into());
        tags.insert("description".into(), format!("{name} description"));
        let mut schematic = Schematic::new(Vec::new(), tags, 4, 5);
        schematic.labels = labels.iter().map(|label| (*label).to_string()).collect();
        SchematicsDialogSchematic::from_schematic(id, &schematic).preview_ready(true)
    }

    #[test]
    fn setup_checks_tags_clears_search_and_builds_shell_like_java() {
        let schematics = vec![
            schematic(0, "Core-Factory!", &["core"]),
            schematic(1, "Drill", &["ore"]),
        ];
        let mut dialog = SchematicsDialog::with_tags(["starter"]);
        dialog.search = "old".into();
        let model = dialog.setup(&schematics, &SchematicsDialogContext::default());

        assert!(dialog.checked_tags);
        assert_eq!(dialog.search, "");
        assert_eq!(dialog.tags, vec!["starter", "core", "ore"]);
        assert_eq!(model.title, "@schematics");
        assert!(model.should_pause);
        assert!(model.close_button_added);
        assert_eq!(model.import_button.text, "@schematic.import");
        assert_eq!(model.search.icon, "zoom");
        assert_eq!(model.search.message_text, "@schematic.search");
        assert_eq!(model.tag_bar.height, 42.0);
        assert_eq!(model.tag_bar.chips.len(), 3);
        assert_eq!(model.pane.columns, 4);
        assert_eq!(model.pane.cards.len(), 2);
        assert_eq!(model.pane.first_schematic, Some(0));
        assert!(model.focus_search_on_desktop_show);
    }

    #[test]
    fn search_and_selected_tags_filter_with_ignore_symbols_and_empty_texts() {
        let schematics = vec![
            schematic(0, "Core-Factory!", &["core"]),
            schematic(1, "Drill Array", &["ore"]),
        ];
        let mut dialog = SchematicsDialog::with_tags(["core", "ore"]);
        dialog.selected_tags.push("core".into());
        dialog.set_search_plan("core_factory");
        let model = dialog.model(&schematics, &SchematicsDialogContext::default());

        assert_eq!(schematic_search_key("Core-Factory!"), "corefactory");
        assert_eq!(model.pane.cards.len(), 1);
        assert_eq!(model.pane.cards[0].schematic, 0);
        assert_eq!(dialog.first_schematic, Some(0));

        dialog.set_search_plan("missing");
        let model = dialog.model(&schematics, &SchematicsDialogContext::default());
        assert!(model.pane.cards.is_empty());
        assert_eq!(model.pane.empty_text, Some("@none.found"));
        assert_eq!(model.pane.empty_color, None);

        dialog.search.clear();
        dialog.selected_tags.clear();
        let model = dialog.model(&[], &SchematicsDialogContext::default());
        assert_eq!(model.pane.empty_text, Some("@none"));
        assert_eq!(model.pane.empty_color, Some("Color.lightGray"));
    }

    #[test]
    fn cards_use_java_buttons_preview_overlay_and_click_actions() {
        let mut steam = schematic(7, "Steam", &[]);
        steam.has_steam_id = true;
        let local = schematic(8, "Local", &[]);
        let mut dialog = SchematicsDialog::new();
        let model = dialog.model(
            &[steam.clone(), local.clone()],
            &SchematicsDialogContext::default(),
        );

        assert_eq!(
            model.pane.cards[0]
                .action_buttons
                .iter()
                .map(|button| button.kind)
                .collect::<Vec<_>>(),
            vec![
                SchematicsCardActionKind::Info,
                SchematicsCardActionKind::Export,
                SchematicsCardActionKind::Edit,
                SchematicsCardActionKind::ViewWorkshop,
            ]
        );
        assert_eq!(
            model.pane.cards[1].action_buttons[3].kind,
            SchematicsCardActionKind::Delete
        );
        assert_eq!(
            model.pane.cards[0].image.background,
            "sprites/schematic-background.png"
        );
        assert_eq!(model.pane.cards[0].name_overlay.max_width, 192.0);

        let menu = SchematicsDialogContext::default();
        assert_eq!(
            dialog.card_click_plan(&local, false, &menu),
            vec![SchematicsDialogAction::ShowInfoDialog { schematic: 8 }]
        );
        let game = SchematicsDialogContext {
            state_is_menu: false,
            ..SchematicsDialogContext::default()
        };
        assert_eq!(
            dialog.card_click_plan(&local, false, &game),
            vec![
                SchematicsDialogAction::UseSchematic { schematic: 8 },
                SchematicsDialogAction::HideDialog,
            ]
        );
        let disabled = SchematicsDialogContext {
            state_is_menu: false,
            schematics_allowed: false,
            ..SchematicsDialogContext::default()
        };
        assert_eq!(
            dialog.card_click_plan(&local, false, &disabled),
            vec![SchematicsDialogAction::ShowInfo {
                text: "@schematic.disabled".into()
            }]
        );
        assert!(dialog.card_click_plan(&local, true, &game).is_empty());
    }

    #[test]
    fn import_and_export_dialogs_match_java_buttons_and_disabled_clipboard_gate() {
        let dialog = SchematicsDialog::new();
        let context = SchematicsDialogContext {
            steam: true,
            clipboard_text: Some("bXNjaA-valid".into()),
            ..SchematicsDialogContext::default()
        };
        let import = dialog.show_import_model(&context);
        assert_eq!(import.title, "@editor.import");
        assert_eq!(import.buttons.len(), 3);
        assert!(!import.buttons[0].disabled);
        assert_eq!(import.buttons[2].text, "@schematic.browseworkshop");

        let invalid = SchematicsDialogContext {
            clipboard_text: Some("not-a-schematic".into()),
            ..SchematicsDialogContext::default()
        };
        assert!(dialog.show_import_model(&invalid).buttons[0].disabled);
        assert_eq!(
            dialog.import_file_plan(),
            vec![
                SchematicsDialogAction::HideDialog,
                SchematicsDialogAction::ImportFileChooser { extension: "msch" },
            ]
        );

        let local = schematic(2, "Local", &[]);
        let export = dialog.show_export_model(&local, &context);
        assert_eq!(
            export
                .buttons
                .iter()
                .map(|button| button.text)
                .collect::<Vec<_>>(),
            vec![
                "@schematic.shareworkshop",
                "@schematic.copy",
                "@schematic.exportfile"
            ]
        );
        assert_eq!(
            dialog.export_action_plan(&local, "@schematic.copy"),
            vec![
                SchematicsDialogAction::HideDialog,
                SchematicsDialogAction::ShowInfoFade { text: "@copied" },
                SchematicsDialogAction::SetClipboardSchematic { schematic: 2 },
            ]
        );
    }

    #[test]
    fn edit_and_tag_mutations_update_schematic_and_global_tags_like_java() {
        let mut dialog = SchematicsDialog::with_tags(["core", "ore"]);
        let mut schem = schematic(3, "Old", &["ore"]);

        let edit = dialog.show_edit_model(&schem);
        assert_eq!(edit.title, "@schematic.edit");
        assert_eq!(edit.name_text, "Old");
        assert_eq!(edit.tags.chips[0].tag, "ore");
        assert_eq!(edit.buttons[0].text, "@ok");

        let actions = dialog.accept_edit_plan(&mut schem, "New", "Desc");
        assert_eq!(schem.name, "New");
        assert_eq!(schem.description, "Desc");
        assert_eq!(
            actions,
            vec![
                SchematicsDialogAction::SaveSchematic { schematic: 3 },
                SchematicsDialogAction::HideDialog,
                SchematicsDialogAction::RebuildPane,
            ]
        );

        let actions = dialog.add_tag_to_schematic_plan(&mut schem, "core");
        assert_eq!(schem.labels, vec!["core", "ore"]);
        assert!(actions.contains(&SchematicsDialogAction::SaveSchematic { schematic: 3 }));

        let actions = dialog.add_global_tag_plan("core");
        assert_eq!(
            actions,
            vec![SchematicsDialogAction::ShowInfo {
                text: "@schematic.tagexists".into()
            }]
        );
        let actions = dialog.add_global_tag_plan("power");
        assert_eq!(dialog.tags, vec!["core", "ore", "power"]);
        assert!(actions.contains(&SchematicsDialogAction::RebuildTags));
    }

    #[test]
    fn all_tags_dialog_move_rename_delete_keep_counts_and_save_actions() {
        let mut dialog = SchematicsDialog::with_tags(["core", "ore", "power"]);
        dialog.selected_tags.push("ore".into());
        let mut schematics = vec![
            schematic(0, "A", &["core", "ore"]),
            schematic(1, "B", &["ore"]),
        ];

        let model = dialog.show_all_tags_model(&schematics);
        assert_eq!(model.title, "@schematic.edittags");
        assert_eq!(model.rows[1].tag, "ore");
        assert_eq!(model.rows[1].tagged_count, 2);
        assert_eq!(model.add_buttons[0].text, "@schematic.texttag");

        let actions = dialog.move_tag_plan("ore", -1);
        assert_eq!(dialog.tags, vec!["ore", "core", "power"]);
        assert!(actions.contains(&SchematicsDialogAction::RebuildTags));

        let actions = dialog.rename_tag_plan(&mut schematics, "ore", "drill");
        assert_eq!(dialog.tags, vec!["drill", "core", "power"]);
        assert_eq!(dialog.selected_tags, vec!["drill"]);
        assert!(schematics[0].labels.contains(&"drill".into()));
        assert!(actions.contains(&SchematicsDialogAction::SaveSchematic { schematic: 0 }));
        assert!(actions.contains(&SchematicsDialogAction::SaveSchematic { schematic: 1 }));

        let actions = dialog.delete_tag_plan(&mut schematics, "drill");
        assert_eq!(dialog.tags, vec!["core", "power"]);
        assert!(dialog.selected_tags.is_empty());
        assert!(!schematics[0].labels.contains(&"drill".into()));
        assert!(actions.contains(&SchematicsDialogAction::RebuildPane));
    }

    #[test]
    fn info_dialog_formats_requirements_power_description_and_buttons() {
        let dialog = SchematicsDialog::with_tags(["core"]);
        let schem = schematic(4, "Info", &["core"])
            .with_requirements(vec![
                ItemStack::new("copper", 1500),
                ItemStack::new("lead", 50),
            ])
            .with_power(1.25, 0.5);
        let context = SchematicsDialogContext {
            state_is_menu: false,
            core_items: BTreeMap::from([("copper".into(), 1000), ("lead".into(), 50)]),
            ..SchematicsDialogContext::default()
        };

        let info = dialog.info_model(&schem, &context);
        assert_eq!(info.title, "[[@schematic] Info");
        assert_eq!(info.info.bundle_key, "schematic.info");
        assert_eq!(info.tags.chips[0].tag, "core");
        assert_eq!(info.image.size, 200.0);
        assert_eq!(
            info.requirements[0].amount_text,
            "[scarlet]1.0[gray]k[][lightgray]/1.5[gray]k[]"
        );
        assert_eq!(info.requirements[1].amount_text, "[lightgray]50");
        let power = info.power.unwrap();
        assert_eq!(power.production.unwrap().text, "+75");
        assert_eq!(power.consumption.unwrap().text, "-30");
        assert!(info.description.unwrap().text.starts_with("[lightgray]"));
        assert_eq!(
            info.buttons
                .iter()
                .map(|button| button.text)
                .collect::<Vec<_>>(),
            vec!["@back", "@editor.export", "@edit"]
        );
    }
}
