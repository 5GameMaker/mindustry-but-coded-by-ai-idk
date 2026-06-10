//! Settings menu dialog model mirroring upstream `mindustry.ui.dialogs.SettingsMenuDialog`.

use crate::mindustry::ui::{
    upstream_menu_bundle_format_for_locale, upstream_menu_bundle_value_for_locale,
};

pub const SETTINGS_DIALOG_TITLE_KEY: &str = "settings";
pub const SETTINGS_DIALOG_TITLE_FALLBACK: &str = "Settings";
pub const SETTINGS_DATA_DIALOG_TITLE: &str = "@settings.data";
pub const SETTINGS_MAIN_TABLE_BACKGROUND: &str = "Tex.button";
pub const SETTINGS_MAIN_BUTTON_STYLE: &str = "Styles.flatt";
pub const SETTINGS_MAIN_BUTTON_SIZE: (f32, f32) = (300.0, 60.0);
pub const SETTINGS_MAIN_BUTTON_MARGIN_LEFT: f32 = 8.0;
pub const SETTINGS_MAIN_ICON_SIZE: f32 = 32.0;
pub const SETTINGS_PREFS_MARGIN: f32 = 14.0;
pub const SETTINGS_DATA_BUTTON_STYLE: &str = "Styles.flatt";
pub const SETTINGS_DATA_BUTTON_SIZE: (f32, f32) = (280.0, 60.0);
pub const SETTINGS_BACK_BUTTON_TEXT: &str = "@back";
pub const SETTINGS_BACK_BUTTON_ICON: &str = "left";
pub const SETTINGS_BACK_BUTTON_SIZE: (f32, f32) = (210.0, 64.0);
pub const SETTINGS_RESET_BUTTON_TEXT: &str = "settings.reset";
pub const SETTINGS_RESET_BUTTON_FALLBACK: &str = "Reset to Defaults";
pub const SETTINGS_RESET_BUTTON_WIDTH: f32 = 240.0;
pub const SETTINGS_RESET_BUTTON_MARGIN: f32 = 14.0;
pub const SETTINGS_RESET_BUTTON_PAD: f32 = 6.0;
pub const SETTINGS_PLANET_OPTION_COLUMNS: usize = 4;
pub const SETTINGS_PLANET_OPTION_SIZE: (f32, f32) = (110.0, 45.0);
pub const SETTINGS_IMPORT_TEMP_FILE: &str = "zipdata.zip";
pub const SETTINGS_IOS_EXPORT_FILE: &str = "mindustry-data-export.zip";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsMenuContext {
    pub mobile: bool,
    pub ios: bool,
    pub steam: bool,
    pub version_modifier_contains_beta: bool,
    pub mac: bool,
    pub shield_shader_available: bool,
    pub keyboard_setting: bool,
    pub is_windows: bool,
    pub ui_scale_at_open: i32,
}

impl Default for SettingsMenuContext {
    fn default() -> Self {
        Self {
            mobile: false,
            ios: false,
            steam: false,
            version_modifier_contains_beta: false,
            mac: cfg!(target_os = "macos"),
            shield_shader_available: true,
            keyboard_setting: false,
            is_windows: cfg!(target_os = "windows"),
            ui_scale_at_open: 100,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsMenuDialogModel {
    pub title: String,
    pub should_pause: bool,
    pub page: SettingsMenuPage,
    pub prefs_margin: f32,
    pub main_menu: Option<SettingsMainMenuModel>,
    pub settings_table: Option<SettingsTableModel>,
    pub child_dialog: Option<SettingsChildDialogModel>,
    pub back_button: SettingsBackButton,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsMainMenuModel {
    pub background: &'static str,
    pub default_button_size: (f32, f32),
    pub entries: Vec<SettingsMainMenuEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsMainMenuEntry {
    pub label: String,
    pub icon: Option<String>,
    pub style: &'static str,
    pub icon_size: f32,
    pub margin_left: f32,
    pub target: SettingsMenuTarget,
    pub row: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsMenuTarget {
    Table(&'static str),
    LanguageDialog,
    ControlsDialog,
    DataDialog,
    CustomCategory(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsMenuPage {
    Main,
    Table(&'static str),
    CustomCategory(usize),
}

impl Default for SettingsMenuPage {
    fn default() -> Self {
        Self::Main
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsTableModel {
    pub table: String,
    pub left_aligned: bool,
    pub rows: Vec<SettingsPrefRow>,
    pub reset_button: SettingsResetButton,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsPrefRow {
    pub spec: SettingsPrefSpec,
    pub title: String,
    pub description: Option<String>,
    pub widget_width: f32,
    pub kind: SettingsPrefWidget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPrefWidget {
    Check {
        checked: bool,
    },
    Slider {
        value: i32,
        display: String,
        min: i32,
        max: i32,
        step: i32,
    },
    Text {
        value: String,
    },
    AreaText {
        value: String,
        rows: u8,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsResetButton {
    pub text: String,
    pub margin: f32,
    pub width: f32,
    pub pad: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsChildDialogModel {
    Data(SettingsDataDialogModel),
    PlanetData(SettingsPlanetDataDialogModel),
    PlanetSelect(SettingsPlanetSelectDialogModel),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsDataDialogModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub table_background: &'static str,
    pub buttons: Vec<SettingsDataButton>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingsDataButton {
    pub label: &'static str,
    pub icon: &'static str,
    pub style: &'static str,
    pub size: (f32, f32),
    pub margin_left: f32,
    pub action: SettingsDataActionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsDataActionKind {
    ClearAllData,
    ClearPlanetData,
    ClearSaves,
    ClearResearch,
    ClearCampaignSaves,
    ExportData,
    ImportData,
    OpenDataFolder,
    ExportCrashLogs,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsPlanetDataDialogModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub table_background: &'static str,
    pub planet_select_button: SettingsPlanetSelectButton,
    pub clear_research_button: SettingsDataButton,
    pub clear_campaign_saves_button: SettingsDataButton,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsPlanetSelectButton {
    pub label: String,
    pub icon: &'static str,
    pub style: &'static str,
    pub size: (f32, f32),
    pub margin_left: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsPlanetSelectDialogModel {
    pub title: &'static str,
    pub fill_parent: bool,
    pub close_button_added: bool,
    pub table_background: &'static str,
    pub columns: usize,
    pub options: Vec<SettingsPlanetOption>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsPlanetOption {
    pub id: String,
    pub text: String,
    pub checked: bool,
    pub size: (f32, f32),
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsBackButton {
    pub text: &'static str,
    pub icon: &'static str,
    pub size: (f32, f32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsCategory {
    pub name: String,
    pub icon: Option<String>,
    pub table: SettingsCustomTable,
}

impl SettingsCategory {
    pub fn new(
        name: impl Into<String>,
        icon: Option<impl Into<String>>,
        entries: Vec<SettingsPrefSpec>,
    ) -> Self {
        let name = name.into();
        Self {
            table: SettingsCustomTable {
                key: name.clone(),
                entries,
            },
            name,
            icon: icon.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsCustomTable {
    pub key: String,
    pub entries: Vec<SettingsPrefSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPlanet {
    pub id: String,
    pub localized_name: String,
    pub icon_color: String,
    pub generator_present: bool,
    pub sector_count: usize,
    pub accessible: bool,
}

impl SettingsPlanet {
    pub fn new(
        id: impl Into<String>,
        localized_name: impl Into<String>,
        icon_color: impl Into<String>,
        generator_present: bool,
        sector_count: usize,
        accessible: bool,
    ) -> Self {
        Self {
            id: id.into(),
            localized_name: localized_name.into(),
            icon_color: icon_color.into(),
            generator_present,
            sector_count,
            accessible,
        }
    }

    pub fn visible_in_planet_select(&self) -> bool {
        self.generator_present && self.sector_count > 0 && self.accessible
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPrefKind {
    Check,
    Slider,
    Text,
    AreaText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPrefDefaultValue {
    Bool(bool),
    BoolNotMobile,
    Int(i32),
    Text(&'static str),
}

impl SettingsPrefDefaultValue {
    pub fn bool_value(self, context: &SettingsMenuContext) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(value),
            Self::BoolNotMobile => Some(!context.mobile),
            Self::Int(_) | Self::Text(_) => None,
        }
    }

    pub fn int_value(self) -> Option<i32> {
        match self {
            Self::Int(value) => Some(value),
            Self::Bool(_) | Self::BoolNotMobile | Self::Text(_) => None,
        }
    }

    pub fn text_value(self, context: &SettingsMenuContext) -> String {
        match self {
            Self::Bool(value) => value.to_string(),
            Self::BoolNotMobile => (!context.mobile).to_string(),
            Self::Int(value) => value.to_string(),
            Self::Text(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsPrefRange {
    pub min: i32,
    pub max: i32,
    pub step: i32,
}

impl SettingsPrefRange {
    pub const fn new(min: i32, max: i32, step: i32) -> Self {
        Self { min, max, step }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsVisibility {
    Always,
    Mobile,
    NotMobile,
    NotIos,
    Steam,
    SteamAndNotBeta,
    MobileAndNotIos,
    Mac,
    ShieldShader,
}

impl SettingsVisibility {
    pub fn visible(self, context: &SettingsMenuContext) -> bool {
        match self {
            Self::Always => true,
            Self::Mobile => context.mobile,
            Self::NotMobile => !context.mobile,
            Self::NotIos => !context.ios,
            Self::Steam => context.steam,
            Self::SteamAndNotBeta => context.steam && !context.version_modifier_contains_beta,
            Self::MobileAndNotIos => context.mobile && !context.ios,
            Self::Mac => context.mac,
            Self::ShieldShader => context.shield_shader_available,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsPrefSpec {
    pub table: &'static str,
    pub key: &'static str,
    pub kind: SettingsPrefKind,
    pub default_value: SettingsPrefDefaultValue,
    pub range: Option<SettingsPrefRange>,
    pub visibility: SettingsVisibility,
    pub text_rows: u8,
}

impl SettingsPrefSpec {
    pub fn visible(self, context: &SettingsMenuContext) -> bool {
        self.visibility.visible(context)
    }

    pub fn default_text(self, context: &SettingsMenuContext) -> String {
        self.default_value.text_value(context)
    }

    pub fn summary(self, context: &SettingsMenuContext) -> String {
        let mut out = format!(
            "{}/{} {:?} default={}",
            self.table,
            self.key,
            self.kind,
            self.default_text(context)
        );
        if let Some(range) = self.range {
            out.push_str(&format!(
                " range={}..{} / {}",
                range.min, range.max, range.step
            ));
        }
        if self.visibility != SettingsVisibility::Always {
            out.push_str(&format!(" visibility={:?}", self.visibility));
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsMenuDialogAction {
    RebuildMenu,
    RebuildTable(&'static str),
    UpdateScrollFocus,
    BackToMain,
    Hide,
    ShowLanguageDialog,
    ShowControlsDialog,
    ShowDataDialog,
    ShowPlanetDataDialog,
    ShowPlanetSelectDialog,
    CloseChildDialog,
    SelectPlanet(String),
    ResetTable {
        table: String,
        remove_keys: Vec<&'static str>,
    },
    PutBool {
        key: &'static str,
        value: bool,
    },
    PutInt {
        key: &'static str,
        value: i32,
    },
    PutString {
        key: &'static str,
        value: String,
    },
    PutFloat {
        key: &'static str,
        value_milli: i32,
    },
    SetDesktopInput,
    SetMobileInput,
    SetUseKeyboard(bool),
    ClearDefaultServers,
    FetchCommunityServers,
    UpdateLobby,
    UpdateMarginsAndResizeScene,
    SetUiScaleChanged(bool),
    SetPreferredFps(i32),
    SetVsync(bool),
    SetFullscreen(bool),
    BeginForceLandscape,
    EndForceLandscape,
    ToggleBloom(bool),
    FireEnablePixelation,
    SetTextureFilterLinear(bool),
    Confirm {
        title: &'static str,
        message: &'static str,
        action: SettingsDataActionKind,
    },
    ConfirmPlanet {
        title: &'static str,
        message_key: &'static str,
        planet_id: String,
    },
    PreserveSettingsContaining(Vec<&'static str>),
    ClearSettings,
    DeleteDataDirectories,
    ExitApp,
    ShowFileChooser {
        open: bool,
        extension: &'static str,
        action: SettingsDataActionKind,
    },
    ExportDataToLocalFile(&'static str),
    ShareFile(String),
    ShowInfo(&'static str),
    ImportDataFromZip,
    ResetSave,
    ResetGameState,
    DeleteAllSaves,
    ClearLoadoutInfo,
    ResetTechTreeAll,
    ResetTechTreeForPlanet(String),
    ClearUnlockableContent,
    ClearUnlockableContentForPlanet(String),
    RemoveSetting(&'static str),
    ClearAllPlanetStats,
    ClearPlanetStats(String),
    DeleteAllCampaignSaves,
    DeleteCampaignSavesForPlanet(String),
    ReloadChangedPlanetMeshes,
    OpenDataFolder,
    ExportCrashLogs,
    ShowException,
    ShowError(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsDataImportError {
    MissingSettingsBin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsDataArchiveEntry {
    pub path: String,
    pub directory: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsMenuDialog {
    pub page: SettingsMenuPage,
    pub selected_planet: String,
    pub categories: Vec<SettingsCategory>,
    pub child_dialog: Option<SettingsChildDialogKind>,
    last_rebuild_size: (i32, i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsChildDialogKind {
    Data,
    PlanetData,
    PlanetSelect,
}

impl Default for SettingsMenuDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsMenuDialog {
    pub fn new() -> Self {
        Self::new_with_size(0, 0)
    }

    pub fn new_with_size(width: i32, height: i32) -> Self {
        Self {
            page: SettingsMenuPage::Main,
            selected_planet: "serpulo".to_string(),
            categories: Vec::new(),
            child_dialog: None,
            last_rebuild_size: (width, height),
        }
    }

    pub fn add_category(&mut self, category: SettingsCategory) {
        self.categories.push(category);
    }

    pub fn shown_plan(&mut self) -> Vec<SettingsMenuDialogAction> {
        self.back();
        vec![
            SettingsMenuDialogAction::BackToMain,
            SettingsMenuDialogAction::RebuildMenu,
        ]
    }

    pub fn on_resize_plan(&mut self, width: i32, height: i32) -> Vec<SettingsMenuDialogAction> {
        if self.last_rebuild_size == (width, height) {
            return Vec::new();
        }
        self.last_rebuild_size = (width, height);
        vec![
            SettingsMenuDialogAction::RebuildTable("graphics"),
            SettingsMenuDialogAction::RebuildTable("sound"),
            SettingsMenuDialogAction::RebuildTable("game"),
            SettingsMenuDialogAction::UpdateScrollFocus,
        ]
    }

    pub fn model(
        &self,
        context: &SettingsMenuContext,
        locale: &str,
        values: &[(String, String)],
        planets: &[SettingsPlanet],
        graphics_width: f32,
    ) -> SettingsMenuDialogModel {
        let settings_table = match self.page {
            SettingsMenuPage::Main => None,
            SettingsMenuPage::Table(table) => Some(settings_table_model(
                table,
                builtin_specs_for_table(table, context),
                context,
                locale,
                values,
                graphics_width,
            )),
            SettingsMenuPage::CustomCategory(index) => {
                let category = &self.categories[index];
                Some(settings_table_model(
                    &category.table.key,
                    category.table.entries.iter().copied().collect(),
                    context,
                    locale,
                    values,
                    graphics_width,
                ))
            }
        };

        SettingsMenuDialogModel {
            title: dialog_title(locale),
            should_pause: true,
            page: self.page,
            prefs_margin: SETTINGS_PREFS_MARGIN,
            main_menu: (self.page == SettingsMenuPage::Main).then(|| self.main_menu_model(context)),
            settings_table,
            child_dialog: self
                .child_dialog
                .map(|kind| self.child_dialog_model(kind, context, locale, planets)),
            back_button: SettingsBackButton {
                text: SETTINGS_BACK_BUTTON_TEXT,
                icon: SETTINGS_BACK_BUTTON_ICON,
                size: SETTINGS_BACK_BUTTON_SIZE,
            },
        }
    }

    pub fn main_menu_model(&self, context: &SettingsMenuContext) -> SettingsMainMenuModel {
        SettingsMainMenuModel {
            background: SETTINGS_MAIN_TABLE_BACKGROUND,
            default_button_size: SETTINGS_MAIN_BUTTON_SIZE,
            entries: self.main_menu_entries(context),
        }
    }

    pub fn main_menu_entries(&self, context: &SettingsMenuContext) -> Vec<SettingsMainMenuEntry> {
        let mut entries = Vec::new();
        let builtins = [
            (
                "@settings.game".to_string(),
                Some("settings".to_string()),
                SettingsMenuTarget::Table("game"),
                true,
            ),
            (
                "@settings.graphics".to_string(),
                Some("image".to_string()),
                SettingsMenuTarget::Table("graphics"),
                true,
            ),
            (
                "@settings.sound".to_string(),
                Some("filters".to_string()),
                SettingsMenuTarget::Table("sound"),
                true,
            ),
            (
                "@settings.language".to_string(),
                Some("chat".to_string()),
                SettingsMenuTarget::LanguageDialog,
                true,
            ),
            (
                "@settings.controls".to_string(),
                Some("move".to_string()),
                SettingsMenuTarget::ControlsDialog,
                !context.mobile || context.keyboard_setting,
            ),
            (
                "@settings.data".to_string(),
                Some("save".to_string()),
                SettingsMenuTarget::DataDialog,
                true,
            ),
        ];

        for (label, icon, target, visible) in builtins {
            if visible {
                entries.push(main_menu_entry(label, icon, target, entries.len()));
            }
        }

        for (index, category) in self.categories.iter().enumerate() {
            entries.push(main_menu_entry(
                category.name.clone(),
                category.icon.clone(),
                SettingsMenuTarget::CustomCategory(index),
                entries.len(),
            ));
        }

        entries
    }

    pub fn open_target_plan(
        &mut self,
        target: SettingsMenuTarget,
    ) -> Vec<SettingsMenuDialogAction> {
        match target {
            SettingsMenuTarget::Table(table) => {
                self.page = SettingsMenuPage::Table(table);
                Vec::new()
            }
            SettingsMenuTarget::LanguageDialog => {
                vec![SettingsMenuDialogAction::ShowLanguageDialog]
            }
            SettingsMenuTarget::ControlsDialog => {
                vec![SettingsMenuDialogAction::ShowControlsDialog]
            }
            SettingsMenuTarget::DataDialog => {
                self.child_dialog = Some(SettingsChildDialogKind::Data);
                vec![SettingsMenuDialogAction::ShowDataDialog]
            }
            SettingsMenuTarget::CustomCategory(index) => {
                self.page = SettingsMenuPage::CustomCategory(index);
                Vec::new()
            }
        }
    }

    pub fn press_back_or_escape_plan(&mut self) -> Vec<SettingsMenuDialogAction> {
        if self.page != SettingsMenuPage::Main {
            self.back();
            vec![
                SettingsMenuDialogAction::BackToMain,
                SettingsMenuDialogAction::RebuildMenu,
            ]
        } else {
            vec![SettingsMenuDialogAction::Hide]
        }
    }

    pub fn back(&mut self) {
        self.page = SettingsMenuPage::Main;
    }

    pub fn show_planet_data_plan(&mut self) -> Vec<SettingsMenuDialogAction> {
        self.child_dialog = Some(SettingsChildDialogKind::PlanetData);
        vec![SettingsMenuDialogAction::ShowPlanetDataDialog]
    }

    pub fn show_planet_select_plan(&mut self) -> Vec<SettingsMenuDialogAction> {
        self.child_dialog = Some(SettingsChildDialogKind::PlanetSelect);
        vec![SettingsMenuDialogAction::ShowPlanetSelectDialog]
    }

    pub fn select_planet_plan(
        &mut self,
        planet_id: impl Into<String>,
    ) -> Vec<SettingsMenuDialogAction> {
        self.selected_planet = planet_id.into();
        self.child_dialog = Some(SettingsChildDialogKind::PlanetData);
        vec![SettingsMenuDialogAction::SelectPlanet(
            self.selected_planet.clone(),
        )]
    }

    pub fn close_child_dialog_plan(&mut self) -> Vec<SettingsMenuDialogAction> {
        self.child_dialog = None;
        vec![SettingsMenuDialogAction::CloseChildDialog]
    }

    fn child_dialog_model(
        &self,
        kind: SettingsChildDialogKind,
        context: &SettingsMenuContext,
        locale: &str,
        planets: &[SettingsPlanet],
    ) -> SettingsChildDialogModel {
        match kind {
            SettingsChildDialogKind::Data => {
                SettingsChildDialogModel::Data(data_dialog_model(context))
            }
            SettingsChildDialogKind::PlanetData => SettingsChildDialogModel::PlanetData(
                planet_data_dialog_model(locale, &self.selected_planet, planets),
            ),
            SettingsChildDialogKind::PlanetSelect => SettingsChildDialogModel::PlanetSelect(
                planet_select_dialog_model(&self.selected_planet, planets),
            ),
        }
    }
}

pub const SETTINGS_PREF_GROUPS: &[SettingsPrefGroup] = &[
    SettingsPrefGroup {
        table: "game",
        entries: &[
            "saveinterval",
            "autotarget",
            "keyboard",
            "communityservers",
            "savecreate",
            "blockreplace",
            "conveyorpathfinding",
            "hints",
            "backgroundpause",
            "buildautopause",
            "distinctcontrolgroups",
            "doubletapmine",
            "commandmodehold",
            "modcrashdisable",
            "playerlimit",
            "steampublichost",
            "console",
        ],
    },
    SettingsPrefGroup {
        table: "graphics",
        entries: &[
            "uiEdgePadding",
            "uiscale",
            "screenshake",
            "bloomintensity",
            "bloomblur",
            "fpscap",
            "chatopacity",
            "lasersopacity",
            "unitlaseropacity",
            "bridgeopacity",
            "maxmagnificationmultiplierpercent",
            "minmagnificationmultiplierpercent",
            "vsync",
            "fullscreen",
            "landscape",
            "effects",
            "atmosphere",
            "drawlight",
            "destroyedblocks",
            "blockstatus",
            "playerchat",
            "coreitems",
            "minimap",
            "smoothcamera",
            "detach-camera",
            "position",
            "mouseposition",
            "fps",
            "playerindicators",
            "showpings",
            "showotherbuildplans",
            "indicators",
            "showweather",
            "animatedwater",
            "animatedshields",
            "bloom",
            "pixelate",
            "linear",
            "skipcoreanimation",
            "hidedisplays",
            "macnotch",
        ],
    },
    SettingsPrefGroup {
        table: "sound",
        entries: &["alwaysmusic", "musicvol", "sfxvol", "ambientvol"],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsPrefGroup {
    pub table: &'static str,
    pub entries: &'static [&'static str],
}

const fn check_pref(
    table: &'static str,
    key: &'static str,
    default_value: SettingsPrefDefaultValue,
    visibility: SettingsVisibility,
) -> SettingsPrefSpec {
    SettingsPrefSpec {
        table,
        key,
        kind: SettingsPrefKind::Check,
        default_value,
        range: None,
        visibility,
        text_rows: 1,
    }
}

const fn slider_pref(
    table: &'static str,
    key: &'static str,
    default_value: SettingsPrefDefaultValue,
    min: i32,
    max: i32,
    step: i32,
    visibility: SettingsVisibility,
) -> SettingsPrefSpec {
    SettingsPrefSpec {
        table,
        key,
        kind: SettingsPrefKind::Slider,
        default_value,
        range: Some(SettingsPrefRange::new(min, max, step)),
        visibility,
        text_rows: 1,
    }
}

pub const SETTINGS_PREF_SPECS: &[SettingsPrefSpec] = &[
    slider_pref(
        "game",
        "saveinterval",
        SettingsPrefDefaultValue::Int(60),
        10,
        600,
        10,
        SettingsVisibility::Always,
    ),
    check_pref(
        "game",
        "autotarget",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Mobile,
    ),
    check_pref(
        "game",
        "keyboard",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::Mobile,
    ),
    check_pref(
        "game",
        "communityservers",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "game",
        "savecreate",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "game",
        "blockreplace",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "game",
        "conveyorpathfinding",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "game",
        "hints",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "game",
        "backgroundpause",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::NotMobile,
    ),
    check_pref(
        "game",
        "buildautopause",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::NotMobile,
    ),
    check_pref(
        "game",
        "distinctcontrolgroups",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::NotMobile,
    ),
    check_pref(
        "game",
        "doubletapmine",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::Always,
    ),
    check_pref(
        "game",
        "commandmodehold",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "game",
        "modcrashdisable",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::NotIos,
    ),
    slider_pref(
        "game",
        "playerlimit",
        SettingsPrefDefaultValue::Int(16),
        2,
        32,
        1,
        SettingsVisibility::Steam,
    ),
    check_pref(
        "game",
        "steampublichost",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::SteamAndNotBeta,
    ),
    check_pref(
        "game",
        "console",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::Always,
    ),
    slider_pref(
        "graphics",
        "uiEdgePadding",
        SettingsPrefDefaultValue::Int(0),
        0,
        100,
        1,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "graphics",
        "uiscale",
        SettingsPrefDefaultValue::Int(100),
        25,
        300,
        5,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "graphics",
        "screenshake",
        SettingsPrefDefaultValue::Int(4),
        0,
        8,
        1,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "graphics",
        "bloomintensity",
        SettingsPrefDefaultValue::Int(6),
        0,
        16,
        1,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "graphics",
        "bloomblur",
        SettingsPrefDefaultValue::Int(2),
        1,
        16,
        1,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "graphics",
        "fpscap",
        SettingsPrefDefaultValue::Int(240),
        10,
        245,
        5,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "graphics",
        "chatopacity",
        SettingsPrefDefaultValue::Int(100),
        0,
        100,
        5,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "graphics",
        "lasersopacity",
        SettingsPrefDefaultValue::Int(100),
        0,
        100,
        5,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "graphics",
        "unitlaseropacity",
        SettingsPrefDefaultValue::Int(100),
        0,
        100,
        5,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "graphics",
        "bridgeopacity",
        SettingsPrefDefaultValue::Int(100),
        0,
        100,
        5,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "graphics",
        "maxmagnificationmultiplierpercent",
        SettingsPrefDefaultValue::Int(100),
        100,
        200,
        25,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "graphics",
        "minmagnificationmultiplierpercent",
        SettingsPrefDefaultValue::Int(100),
        100,
        300,
        25,
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "vsync",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::NotMobile,
    ),
    check_pref(
        "graphics",
        "fullscreen",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::NotMobile,
    ),
    check_pref(
        "graphics",
        "landscape",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::MobileAndNotIos,
    ),
    check_pref(
        "graphics",
        "effects",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "atmosphere",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "drawlight",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "destroyedblocks",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "blockstatus",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "playerchat",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "coreitems",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::NotMobile,
    ),
    check_pref(
        "graphics",
        "minimap",
        SettingsPrefDefaultValue::BoolNotMobile,
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "smoothcamera",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "detach-camera",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::NotMobile,
    ),
    check_pref(
        "graphics",
        "position",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "mouseposition",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::NotMobile,
    ),
    check_pref(
        "graphics",
        "fps",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "playerindicators",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "showpings",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "showotherbuildplans",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "indicators",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "showweather",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "animatedwater",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "animatedshields",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::ShieldShader,
    ),
    check_pref(
        "graphics",
        "bloom",
        SettingsPrefDefaultValue::Bool(true),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "pixelate",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "linear",
        SettingsPrefDefaultValue::BoolNotMobile,
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "skipcoreanimation",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "hidedisplays",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::Always,
    ),
    check_pref(
        "graphics",
        "macnotch",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::Mac,
    ),
    check_pref(
        "sound",
        "alwaysmusic",
        SettingsPrefDefaultValue::Bool(false),
        SettingsVisibility::Always,
    ),
    slider_pref(
        "sound",
        "musicvol",
        SettingsPrefDefaultValue::Int(100),
        0,
        100,
        1,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "sound",
        "sfxvol",
        SettingsPrefDefaultValue::Int(100),
        0,
        100,
        1,
        SettingsVisibility::Always,
    ),
    slider_pref(
        "sound",
        "ambientvol",
        SettingsPrefDefaultValue::Int(100),
        0,
        100,
        1,
        SettingsVisibility::Always,
    ),
];

pub const SETTINGS_DATA_ACTIONS: &[SettingsDataButton] = &[
    data_button(
        "@settings.cleardata",
        "trash",
        SettingsDataActionKind::ClearAllData,
    ),
    data_button(
        "@settings.clearplanetdata",
        "trash",
        SettingsDataActionKind::ClearPlanetData,
    ),
    data_button(
        "@settings.clearsaves",
        "trash",
        SettingsDataActionKind::ClearSaves,
    ),
    data_button(
        "@settings.clearresearch",
        "trash",
        SettingsDataActionKind::ClearResearch,
    ),
    data_button(
        "@settings.clearcampaignsaves",
        "trash",
        SettingsDataActionKind::ClearCampaignSaves,
    ),
    data_button("@data.export", "upload", SettingsDataActionKind::ExportData),
    data_button(
        "@data.import",
        "download",
        SettingsDataActionKind::ImportData,
    ),
    data_button(
        "@data.openfolder",
        "folder",
        SettingsDataActionKind::OpenDataFolder,
    ),
    data_button(
        "@crash.export",
        "upload",
        SettingsDataActionKind::ExportCrashLogs,
    ),
];

const fn data_button(
    label: &'static str,
    icon: &'static str,
    action: SettingsDataActionKind,
) -> SettingsDataButton {
    SettingsDataButton {
        label,
        icon,
        style: SETTINGS_DATA_BUTTON_STYLE,
        size: SETTINGS_DATA_BUTTON_SIZE,
        margin_left: 4.0,
        action,
    }
}

fn main_menu_entry(
    label: String,
    icon: Option<String>,
    target: SettingsMenuTarget,
    row: usize,
) -> SettingsMainMenuEntry {
    SettingsMainMenuEntry {
        label,
        icon,
        style: SETTINGS_MAIN_BUTTON_STYLE,
        icon_size: SETTINGS_MAIN_ICON_SIZE,
        margin_left: SETTINGS_MAIN_BUTTON_MARGIN_LEFT,
        target,
        row,
    }
}

pub fn dialog_title(locale: &str) -> String {
    upstream_menu_bundle_value_for_locale(locale, SETTINGS_DIALOG_TITLE_KEY)
        .unwrap_or(SETTINGS_DIALOG_TITLE_FALLBACK)
        .to_string()
}

pub fn builtin_specs_for_table(
    table: &str,
    context: &SettingsMenuContext,
) -> Vec<SettingsPrefSpec> {
    let group = SETTINGS_PREF_GROUPS
        .iter()
        .find(|group| group.table == table)
        .expect("settings table group must exist");
    group
        .entries
        .iter()
        .map(|key| settings_pref_spec(table, key).expect("settings pref spec must exist"))
        .filter(|spec| spec.visible(context))
        .collect()
}

pub fn settings_pref_spec(table: &str, key: &str) -> Option<SettingsPrefSpec> {
    SETTINGS_PREF_SPECS
        .iter()
        .copied()
        .find(|spec| spec.table == table && spec.key == key)
}

pub fn settings_table_model(
    table: &str,
    specs: Vec<SettingsPrefSpec>,
    context: &SettingsMenuContext,
    locale: &str,
    values: &[(String, String)],
    graphics_width: f32,
) -> SettingsTableModel {
    let widget_width = (graphics_width / 1.2).min(460.0);
    SettingsTableModel {
        table: table.to_string(),
        left_aligned: true,
        rows: specs
            .into_iter()
            .map(|spec| pref_row(spec, context, locale, values, widget_width))
            .collect(),
        reset_button: SettingsResetButton {
            text: upstream_menu_bundle_value_for_locale(locale, SETTINGS_RESET_BUTTON_TEXT)
                .unwrap_or(SETTINGS_RESET_BUTTON_FALLBACK)
                .to_string(),
            margin: SETTINGS_RESET_BUTTON_MARGIN,
            width: SETTINGS_RESET_BUTTON_WIDTH,
            pad: SETTINGS_RESET_BUTTON_PAD,
        },
    }
}

pub fn pref_row(
    spec: SettingsPrefSpec,
    context: &SettingsMenuContext,
    locale: &str,
    values: &[(String, String)],
    widget_width: f32,
) -> SettingsPrefRow {
    let value = setting_value(spec, context, values);
    let kind = match spec.kind {
        SettingsPrefKind::Check => SettingsPrefWidget::Check {
            checked: value == "true",
        },
        SettingsPrefKind::Slider => {
            let range = spec.range.expect("slider range must exist");
            let int_value = value
                .parse::<i32>()
                .unwrap_or_else(|_| spec.default_value.int_value().expect("slider default int"));
            SettingsPrefWidget::Slider {
                value: int_value,
                display: slider_display_value(spec.key, int_value, locale),
                min: range.min,
                max: range.max,
                step: range.step,
            }
        }
        SettingsPrefKind::Text => SettingsPrefWidget::Text { value },
        SettingsPrefKind::AreaText => SettingsPrefWidget::AreaText {
            value,
            rows: spec.text_rows,
        },
    };

    SettingsPrefRow {
        spec,
        title: setting_title(locale, spec.key, context.is_windows),
        description: setting_description(locale, spec.key),
        widget_width,
        kind,
    }
}

fn setting_value(
    spec: SettingsPrefSpec,
    context: &SettingsMenuContext,
    values: &[(String, String)],
) -> String {
    let storage_key = settings_storage_key(spec.table, spec.key);
    values
        .iter()
        .find(|(key, _)| key == &storage_key || key == spec.key)
        .map(|(_, value)| value.clone())
        .unwrap_or_else(|| spec.default_text(context))
}

pub fn settings_storage_key(table: &str, key: &str) -> String {
    format!("{table}.{key}")
}

pub fn setting_title(locale: &str, key: &str, is_windows: bool) -> String {
    let windows_key = format!("setting.{key}.name.windows");
    if is_windows {
        if let Some(value) = upstream_menu_bundle_value_for_locale(locale, &windows_key) {
            return value.to_string();
        }
    }
    let key_name = format!("setting.{key}.name");
    upstream_menu_bundle_value_for_locale(locale, &key_name)
        .unwrap_or(key)
        .to_string()
}

pub fn setting_description(locale: &str, key: &str) -> Option<String> {
    let desc_key = format!("setting.{key}.description");
    upstream_menu_bundle_value_for_locale(locale, &desc_key).map(ToOwned::to_owned)
}

pub fn slider_display_value(key: &str, value: i32, locale: &str) -> String {
    match key {
        "saveinterval" => {
            upstream_menu_bundle_format_for_locale(locale, "setting.seconds", &[&value.to_string()])
                .unwrap_or_else(|| format!("{value}"))
        }
        "playerlimit" => value.to_string(),
        "uiEdgePadding" => format!("{value}px"),
        "uiscale"
        | "chatopacity"
        | "lasersopacity"
        | "unitlaseropacity"
        | "bridgeopacity"
        | "maxmagnificationmultiplierpercent"
        | "minmagnificationmultiplierpercent"
        | "musicvol"
        | "sfxvol"
        | "ambientvol" => format!("{value}%"),
        "screenshake" => format!("{}x", java_float_label(value as f32 / 4.0)),
        "bloomintensity" => format!("{}%", (value as f32 / 4.0 * 100.0) as i32),
        "bloomblur" => format!("{value}x"),
        "fpscap" => {
            if value > 240 {
                upstream_menu_bundle_value_for_locale(locale, "setting.fpscap.none")
                    .unwrap_or("None")
                    .to_string()
            } else {
                upstream_menu_bundle_format_for_locale(
                    locale,
                    "setting.fpscap.text",
                    &[&value.to_string()],
                )
                .unwrap_or_else(|| value.to_string())
            }
        }
        _ => value.to_string(),
    }
}

pub fn java_float_label(value: f32) -> String {
    let mut out = format!("{value:.2}");
    while out.ends_with('0') && !out.ends_with(".0") {
        out.pop();
    }
    out
}

pub fn data_dialog_model(context: &SettingsMenuContext) -> SettingsDataDialogModel {
    SettingsDataDialogModel {
        title: SETTINGS_DATA_DIALOG_TITLE,
        close_button_added: true,
        table_background: SETTINGS_MAIN_TABLE_BACKGROUND,
        buttons: settings_visible_data_buttons(context),
    }
}

pub fn settings_visible_data_buttons(context: &SettingsMenuContext) -> Vec<SettingsDataButton> {
    SETTINGS_DATA_ACTIONS
        .iter()
        .copied()
        .filter(|button| button.action != SettingsDataActionKind::OpenDataFolder || !context.mobile)
        .collect()
}

pub fn planet_data_dialog_model(
    locale: &str,
    selected_planet_id: &str,
    planets: &[SettingsPlanet],
) -> SettingsPlanetDataDialogModel {
    let planet = planets
        .iter()
        .find(|planet| planet.id == selected_planet_id)
        .unwrap_or_else(|| {
            planets
                .first()
                .expect("planet data dialog requires planets")
        });
    SettingsPlanetDataDialogModel {
        title: SETTINGS_DATA_DIALOG_TITLE,
        close_button_added: true,
        table_background: SETTINGS_MAIN_TABLE_BACKGROUND,
        planet_select_button: SettingsPlanetSelectButton {
            label: planet_select_label(locale, planet),
            icon: "planet",
            style: SETTINGS_DATA_BUTTON_STYLE,
            size: SETTINGS_DATA_BUTTON_SIZE,
            margin_left: 4.0,
        },
        clear_research_button: data_button(
            "@settings.clearplanetresearch",
            "trash",
            SettingsDataActionKind::ClearPlanetData,
        ),
        clear_campaign_saves_button: data_button(
            "@settings.clearplanetcampaignsaves",
            "trash",
            SettingsDataActionKind::ClearCampaignSaves,
        ),
    }
}

pub fn planet_select_label(locale: &str, planet: &SettingsPlanet) -> String {
    let colored = format!("[#{}]{}", planet.icon_color, planet.localized_name);
    upstream_menu_bundle_format_for_locale(locale, "settings.planetselect", &[&colored])
        .unwrap_or(colored)
}

pub fn planet_select_dialog_model(
    selected_planet_id: &str,
    planets: &[SettingsPlanet],
) -> SettingsPlanetSelectDialogModel {
    let options = visible_planets(planets)
        .into_iter()
        .enumerate()
        .map(|(index, planet)| SettingsPlanetOption {
            id: planet.id.clone(),
            text: planet.localized_name.clone(),
            checked: planet.id == selected_planet_id,
            size: SETTINGS_PLANET_OPTION_SIZE,
            row: index / SETTINGS_PLANET_OPTION_COLUMNS,
            column: index % SETTINGS_PLANET_OPTION_COLUMNS,
        })
        .collect();

    SettingsPlanetSelectDialogModel {
        title: "",
        fill_parent: false,
        close_button_added: true,
        table_background: SETTINGS_MAIN_TABLE_BACKGROUND,
        columns: SETTINGS_PLANET_OPTION_COLUMNS,
        options,
    }
}

pub fn visible_planets(planets: &[SettingsPlanet]) -> Vec<SettingsPlanet> {
    planets
        .iter()
        .filter(|planet| planet.visible_in_planet_select())
        .cloned()
        .collect()
}

pub fn reset_table_plan(table: &str, context: &SettingsMenuContext) -> SettingsMenuDialogAction {
    SettingsMenuDialogAction::ResetTable {
        table: table.to_string(),
        remove_keys: builtin_specs_for_table(table, context)
            .into_iter()
            .map(|spec| spec.key)
            .collect(),
    }
}

pub fn setting_change_plan(
    spec: SettingsPrefSpec,
    value: &str,
    context: &SettingsMenuContext,
) -> Vec<SettingsMenuDialogAction> {
    let mut out = match spec.kind {
        SettingsPrefKind::Check => {
            let checked = value == "true";
            vec![SettingsMenuDialogAction::PutBool {
                key: spec.key,
                value: checked,
            }]
        }
        SettingsPrefKind::Slider => {
            let int_value = value.parse::<i32>().expect("slider value must be an int");
            vec![SettingsMenuDialogAction::PutInt {
                key: spec.key,
                value: int_value,
            }]
        }
        SettingsPrefKind::Text | SettingsPrefKind::AreaText => {
            vec![SettingsMenuDialogAction::PutString {
                key: spec.key,
                value: value.to_string(),
            }]
        }
    };

    match (spec.table, spec.key) {
        ("game", "keyboard") => {
            let enabled = value == "true";
            if enabled {
                out.push(SettingsMenuDialogAction::SetDesktopInput);
            } else {
                out.push(SettingsMenuDialogAction::SetMobileInput);
            }
            out.push(SettingsMenuDialogAction::SetUseKeyboard(enabled));
        }
        ("game", "communityservers") => {
            out.push(SettingsMenuDialogAction::ClearDefaultServers);
            if value == "true" {
                out.push(SettingsMenuDialogAction::FetchCommunityServers);
            }
        }
        ("game", "playerlimit") | ("game", "steampublichost") => {
            out.push(SettingsMenuDialogAction::UpdateLobby);
        }
        ("graphics", "uiEdgePadding") => {
            out.push(SettingsMenuDialogAction::UpdateMarginsAndResizeScene);
        }
        ("graphics", "uiscale") => {
            let int_value = value.parse::<i32>().expect("uiscale value must be an int");
            out.push(SettingsMenuDialogAction::SetUiScaleChanged(
                int_value != context.ui_scale_at_open,
            ));
        }
        ("graphics", "fpscap") if context.ios => {
            let int_value = value.parse::<i32>().expect("fpscap value must be an int");
            out.push(SettingsMenuDialogAction::SetPreferredFps(
                if int_value > 240 { 0 } else { int_value },
            ));
        }
        ("graphics", "lasersopacity") => {
            let int_value = value
                .parse::<i32>()
                .expect("laser opacity value must be an int");
            out.push(SettingsMenuDialogAction::PutInt {
                key: "preferredlaseropacity",
                value: int_value,
            });
        }
        ("graphics", "maxmagnificationmultiplierpercent") => {
            let int_value = value.parse::<i32>().expect("max zoom value must be an int");
            out.push(SettingsMenuDialogAction::PutFloat {
                key: "maxzoomingamemultiplier",
                value_milli: int_value * 10,
            });
        }
        ("graphics", "minmagnificationmultiplierpercent") => {
            let int_value = value.parse::<i32>().expect("min zoom value must be an int");
            out.push(SettingsMenuDialogAction::PutFloat {
                key: "minzoomingamemultiplier",
                value_milli: int_value * 10,
            });
        }
        ("graphics", "vsync") => {
            out.push(SettingsMenuDialogAction::SetVsync(value == "true"));
        }
        ("graphics", "fullscreen") => {
            out.push(SettingsMenuDialogAction::SetFullscreen(value == "true"));
        }
        ("graphics", "landscape") => {
            if value == "true" {
                out.push(SettingsMenuDialogAction::BeginForceLandscape);
            } else {
                out.push(SettingsMenuDialogAction::EndForceLandscape);
            }
        }
        ("graphics", "bloom") => {
            out.push(SettingsMenuDialogAction::ToggleBloom(value == "true"));
        }
        ("graphics", "pixelate") if value == "true" => {
            out.push(SettingsMenuDialogAction::FireEnablePixelation);
        }
        ("graphics", "linear") => {
            out.push(SettingsMenuDialogAction::SetTextureFilterLinear(
                value == "true",
            ));
        }
        _ => {}
    }

    out
}

pub fn initial_settings_side_effects_plan(
    context: &SettingsMenuContext,
    values: &[(String, String)],
) -> Vec<SettingsMenuDialogAction> {
    let mut out = Vec::new();

    if context.mobile && value_bool("game", "keyboard", context, values) {
        out.push(SettingsMenuDialogAction::SetDesktopInput);
        out.push(SettingsMenuDialogAction::SetUseKeyboard(true));
    }

    if !context.mobile {
        out.push(SettingsMenuDialogAction::SetVsync(value_bool(
            "graphics", "vsync", context, values,
        )));
        if value_bool("graphics", "fullscreen", context, values) {
            out.push(SettingsMenuDialogAction::SetFullscreen(true));
        }
        out.push(SettingsMenuDialogAction::PutBool {
            key: "swapdiagonal",
            value: false,
        });
    } else if !context.ios && value_bool("graphics", "landscape", context, values) {
        out.push(SettingsMenuDialogAction::BeginForceLandscape);
    }

    if context.ios {
        let fps = value_int("graphics", "fpscap", context, values);
        out.push(SettingsMenuDialogAction::SetPreferredFps(if fps > 240 {
            0
        } else {
            fps
        }));
    }

    if value_bool("graphics", "linear", context, values) {
        out.push(SettingsMenuDialogAction::SetTextureFilterLinear(true));
    }

    out
}

fn value_bool(
    table: &str,
    key: &str,
    context: &SettingsMenuContext,
    values: &[(String, String)],
) -> bool {
    let spec = settings_pref_spec(table, key).expect("settings bool spec must exist");
    setting_value(spec, context, values) == "true"
}

fn value_int(
    table: &str,
    key: &str,
    context: &SettingsMenuContext,
    values: &[(String, String)],
) -> i32 {
    let spec = settings_pref_spec(table, key).expect("settings int spec must exist");
    setting_value(spec, context, values)
        .parse::<i32>()
        .expect("settings int value must parse")
}

pub fn data_action_plan(
    action: SettingsDataActionKind,
    context: &SettingsMenuContext,
) -> Vec<SettingsMenuDialogAction> {
    match action {
        SettingsDataActionKind::ClearAllData => vec![
            SettingsMenuDialogAction::Confirm {
                title: "@confirm",
                message: "@settings.clearall.confirm",
                action,
            },
            SettingsMenuDialogAction::PreserveSettingsContaining(vec!["usid", "uuid"]),
            SettingsMenuDialogAction::ClearSettings,
            SettingsMenuDialogAction::DeleteDataDirectories,
            SettingsMenuDialogAction::ExitApp,
        ],
        SettingsDataActionKind::ClearPlanetData => {
            vec![SettingsMenuDialogAction::ShowPlanetDataDialog]
        }
        SettingsDataActionKind::ClearSaves => vec![
            SettingsMenuDialogAction::Confirm {
                title: "@confirm",
                message: "@settings.clearsaves.confirm",
                action,
            },
            SettingsMenuDialogAction::DeleteAllSaves,
        ],
        SettingsDataActionKind::ClearResearch => vec![
            SettingsMenuDialogAction::Confirm {
                title: "@confirm",
                message: "@settings.clearresearch.confirm",
                action,
            },
            SettingsMenuDialogAction::ClearLoadoutInfo,
            SettingsMenuDialogAction::ResetTechTreeAll,
            SettingsMenuDialogAction::ClearUnlockableContent,
            SettingsMenuDialogAction::RemoveSetting("unlocks"),
        ],
        SettingsDataActionKind::ClearCampaignSaves => vec![
            SettingsMenuDialogAction::Confirm {
                title: "@confirm",
                message: "@settings.clearcampaignsaves.confirm",
                action,
            },
            SettingsMenuDialogAction::ClearAllPlanetStats,
            SettingsMenuDialogAction::DeleteAllCampaignSaves,
            SettingsMenuDialogAction::ReloadChangedPlanetMeshes,
        ],
        SettingsDataActionKind::ExportData => {
            if context.ios {
                vec![
                    SettingsMenuDialogAction::ExportDataToLocalFile(SETTINGS_IOS_EXPORT_FILE),
                    SettingsMenuDialogAction::ShareFile(SETTINGS_IOS_EXPORT_FILE.to_string()),
                ]
            } else {
                vec![
                    SettingsMenuDialogAction::ShowFileChooser {
                        open: false,
                        extension: "zip",
                        action,
                    },
                    SettingsMenuDialogAction::ShowInfo("@data.exported"),
                ]
            }
        }
        SettingsDataActionKind::ImportData => vec![
            SettingsMenuDialogAction::Confirm {
                title: "@confirm",
                message: "@data.import.confirm",
                action,
            },
            SettingsMenuDialogAction::ShowFileChooser {
                open: true,
                extension: "zip",
                action,
            },
            SettingsMenuDialogAction::ImportDataFromZip,
            SettingsMenuDialogAction::ResetSave,
            SettingsMenuDialogAction::ResetGameState,
            SettingsMenuDialogAction::ExitApp,
        ],
        SettingsDataActionKind::OpenDataFolder => vec![SettingsMenuDialogAction::OpenDataFolder],
        SettingsDataActionKind::ExportCrashLogs => {
            vec![SettingsMenuDialogAction::ExportCrashLogs]
        }
    }
}

pub fn planet_data_action_plan(
    action: SettingsPlanetDataActionKind,
    planet_id: impl Into<String>,
) -> Vec<SettingsMenuDialogAction> {
    let planet_id = planet_id.into();
    match action {
        SettingsPlanetDataActionKind::ClearPlanetResearch => vec![
            SettingsMenuDialogAction::ConfirmPlanet {
                title: "@confirm",
                message_key: "settings.clearplanetresearch.confirm",
                planet_id: planet_id.clone(),
            },
            SettingsMenuDialogAction::ClearLoadoutInfo,
            SettingsMenuDialogAction::ResetTechTreeForPlanet(planet_id.clone()),
            SettingsMenuDialogAction::ClearUnlockableContentForPlanet(planet_id),
            SettingsMenuDialogAction::RemoveSetting("unlocks"),
        ],
        SettingsPlanetDataActionKind::ClearPlanetCampaignSaves => vec![
            SettingsMenuDialogAction::ConfirmPlanet {
                title: "@confirm",
                message_key: "settings.clearplanetcampaignsaves.confirm",
                planet_id: planet_id.clone(),
            },
            SettingsMenuDialogAction::ClearPlanetStats(planet_id.clone()),
            SettingsMenuDialogAction::DeleteCampaignSavesForPlanet(planet_id),
            SettingsMenuDialogAction::ReloadChangedPlanetMeshes,
        ],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPlanetDataActionKind {
    ClearPlanetResearch,
    ClearPlanetCampaignSaves,
}

pub fn export_data_archive_entries(
    data_dir: &str,
    settings_file: &str,
    custom_maps: &[&str],
    saves: &[&str],
    mods: &[&str],
    schematics: &[&str],
) -> Vec<SettingsDataArchiveEntry> {
    let base = normalize_archive_path(data_dir);
    let mut files: Vec<SettingsDataArchiveEntry> = Vec::new();
    for path in std::iter::once(settings_file)
        .chain(custom_maps.iter().copied())
        .chain(saves.iter().copied())
        .chain(mods.iter().copied())
        .chain(schematics.iter().copied())
    {
        files.push(SettingsDataArchiveEntry {
            path: relative_archive_path(&base, path),
            directory: path.ends_with('/') || path.ends_with('\\'),
        });
    }

    let mut index = 0;
    while index < files.len() {
        let mut parent = parent_archive_path(&files[index].path);
        while let Some(path) = parent {
            if !files.iter().any(|entry| entry.path == path) {
                files.push(SettingsDataArchiveEntry {
                    path: format!("{path}/"),
                    directory: true,
                });
            }
            parent = parent_archive_path(&path);
        }
        index += 1;
    }

    files
        .into_iter()
        .map(|mut entry| {
            entry.path = entry.path.trim_start_matches('/').to_string();
            if entry.directory && !entry.path.ends_with('/') {
                entry.path.push('/');
            }
            entry
        })
        .collect()
}

fn normalize_archive_path(path: &str) -> String {
    path.replace('\\', "/").trim_end_matches('/').to_string()
}

fn relative_archive_path(base: &str, path: &str) -> String {
    let normalized = normalize_archive_path(path);
    normalized
        .strip_prefix(base)
        .unwrap_or(&normalized)
        .trim_start_matches('/')
        .to_string()
}

fn parent_archive_path(path: &str) -> Option<String> {
    let trimmed = path.trim_end_matches('/');
    let index = trimmed.rfind('/')?;
    (index > 0).then(|| trimmed[..index].to_string())
}

pub fn import_data_plan(
    zip_entries: &[&str],
) -> Result<Vec<SettingsMenuDialogAction>, SettingsDataImportError> {
    if !zip_entries.iter().any(|entry| *entry == "settings.bin") {
        return Err(SettingsDataImportError::MissingSettingsBin);
    }
    Ok(vec![
        SettingsMenuDialogAction::ImportDataFromZip,
        SettingsMenuDialogAction::DeleteAllSaves,
        SettingsMenuDialogAction::ClearSettings,
    ])
}

pub fn crash_logs_text(crashes: &[(&str, &str)], last_log: Option<&str>) -> String {
    let mut out = String::new();
    for (name, contents) in crashes {
        out.push_str(name);
        out.push_str("\n\n");
        out.push_str(contents);
        out.push('\n');
    }
    if let Some(log) = last_log {
        out.push_str("\nlast log:\n");
        out.push_str(log);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(specs: &[SettingsPrefSpec]) -> Vec<&'static str> {
        specs.iter().map(|spec| spec.key).collect()
    }

    fn planets() -> Vec<SettingsPlanet> {
        vec![
            SettingsPlanet::new("serpulo", "Serpulo", "5f9aff", true, 10, true),
            SettingsPlanet::new("erekir", "Erekir", "ffaa5f", true, 8, true),
            SettingsPlanet::new("sun", "Sun", "ffffff", false, 0, true),
            SettingsPlanet::new("hidden", "Hidden", "000000", true, 1, false),
        ]
    }

    #[test]
    fn main_menu_matches_java_visibility_and_dynamic_categories() {
        let mut dialog = SettingsMenuDialog::new();
        let desktop = SettingsMenuContext::default();
        let labels: Vec<_> = dialog
            .main_menu_entries(&desktop)
            .into_iter()
            .map(|entry| entry.label)
            .collect();
        assert_eq!(
            labels,
            vec![
                "@settings.game",
                "@settings.graphics",
                "@settings.sound",
                "@settings.language",
                "@settings.controls",
                "@settings.data"
            ]
        );

        let mobile = SettingsMenuContext {
            mobile: true,
            ..SettingsMenuContext::default()
        };
        let labels: Vec<_> = dialog
            .main_menu_entries(&mobile)
            .into_iter()
            .map(|entry| entry.label)
            .collect();
        assert!(!labels.contains(&"@settings.controls".to_string()));

        let mobile_keyboard = SettingsMenuContext {
            mobile: true,
            keyboard_setting: true,
            ..SettingsMenuContext::default()
        };
        let labels: Vec<_> = dialog
            .main_menu_entries(&mobile_keyboard)
            .into_iter()
            .map(|entry| entry.label)
            .collect();
        assert!(labels.contains(&"@settings.controls".to_string()));

        dialog.add_category(SettingsCategory::new(
            "Mod Settings",
            Some("mod-icon"),
            vec![check_pref(
                "mod-settings",
                "mod-enabled",
                SettingsPrefDefaultValue::Bool(true),
                SettingsVisibility::Always,
            )],
        ));
        let entries = dialog.main_menu_entries(&desktop);
        let custom = entries.last().unwrap();
        assert_eq!(custom.label, "Mod Settings");
        assert_eq!(custom.icon.as_deref(), Some("mod-icon"));
        assert_eq!(custom.target, SettingsMenuTarget::CustomCategory(0));
        assert_eq!(custom.icon_size, SETTINGS_MAIN_ICON_SIZE);
    }

    #[test]
    fn builtin_setting_tables_match_upstream_platform_conditions() {
        let desktop = SettingsMenuContext::default();
        let game = keys(&builtin_specs_for_table("game", &desktop));
        assert!(game.contains(&"saveinterval"));
        assert!(game.contains(&"backgroundpause"));
        assert!(game.contains(&"modcrashdisable"));
        assert!(!game.contains(&"autotarget"));
        assert!(!game.contains(&"playerlimit"));

        let steam_beta = SettingsMenuContext {
            steam: true,
            version_modifier_contains_beta: true,
            ..SettingsMenuContext::default()
        };
        let game = keys(&builtin_specs_for_table("game", &steam_beta));
        assert!(game.contains(&"playerlimit"));
        assert!(!game.contains(&"steampublichost"));

        let mobile_ios = SettingsMenuContext {
            mobile: true,
            ios: true,
            ..SettingsMenuContext::default()
        };
        let game = keys(&builtin_specs_for_table("game", &mobile_ios));
        assert!(game.contains(&"autotarget"));
        assert!(game.contains(&"keyboard"));
        assert!(!game.contains(&"backgroundpause"));
        assert!(!game.contains(&"modcrashdisable"));

        let graphics = builtin_specs_for_table("graphics", &mobile_ios);
        assert!(!keys(&graphics).contains(&"landscape"));
        let minimap = graphics.iter().find(|spec| spec.key == "minimap").unwrap();
        assert_eq!(minimap.default_text(&mobile_ios), "false");
        let linear = graphics.iter().find(|spec| spec.key == "linear").unwrap();
        assert_eq!(linear.default_text(&mobile_ios), "false");
    }

    #[test]
    fn settings_table_rows_use_java_titles_slider_text_and_reset_row() {
        let context = SettingsMenuContext::default();
        let values = vec![
            (settings_storage_key("sound", "musicvol"), "15".to_string()),
            (
                settings_storage_key("graphics", "fpscap"),
                "245".to_string(),
            ),
        ];
        let sound = settings_table_model(
            "sound",
            builtin_specs_for_table("sound", &context),
            &context,
            "en",
            &values,
            1200.0,
        );
        assert_eq!(sound.reset_button.width, SETTINGS_RESET_BUTTON_WIDTH);
        assert_eq!(sound.rows[0].title, "Always Play Music");
        let music = sound
            .rows
            .iter()
            .find(|row| row.spec.key == "musicvol")
            .unwrap();
        assert_eq!(
            music.kind,
            SettingsPrefWidget::Slider {
                value: 15,
                display: "15%".to_string(),
                min: 0,
                max: 100,
                step: 1,
            }
        );

        assert_eq!(slider_display_value("saveinterval", 60, "en"), "60 seconds");
        assert_eq!(slider_display_value("screenshake", 4, "en"), "1.0x");
        assert_eq!(slider_display_value("bloomintensity", 6, "en"), "150%");
        assert_eq!(slider_display_value("fpscap", 245, "en"), "None");
        assert_eq!(slider_display_value("uiEdgePadding", 24, "en"), "24px");
    }

    #[test]
    fn data_and_planet_dialog_models_match_java_buttons() {
        let desktop = SettingsMenuContext::default();
        let data = data_dialog_model(&desktop);
        assert_eq!(data.title, "@settings.data");
        assert_eq!(data.buttons.len(), 9);
        assert_eq!(data.buttons[0].label, "@settings.cleardata");
        assert_eq!(data.buttons[5].action, SettingsDataActionKind::ExportData);

        let mobile = SettingsMenuContext {
            mobile: true,
            ..SettingsMenuContext::default()
        };
        let data = data_dialog_model(&mobile);
        assert!(!data
            .buttons
            .iter()
            .any(|button| button.action == SettingsDataActionKind::OpenDataFolder));

        let p = planets();
        let planet_data = planet_data_dialog_model("en", "serpulo", &p);
        assert!(planet_data
            .planet_select_button
            .label
            .contains("[#5f9aff]Serpulo"));
        assert_eq!(
            planet_data.clear_research_button.label,
            "@settings.clearplanetresearch"
        );

        let select = planet_select_dialog_model("erekir", &p);
        assert_eq!(select.columns, 4);
        assert_eq!(select.options.len(), 2);
        assert!(select.options[1].checked);
    }

    #[test]
    fn navigation_resize_and_reset_follow_java_state_machine() {
        let context = SettingsMenuContext::default();
        let mut dialog = SettingsMenuDialog::new_with_size(800, 600);
        assert!(dialog.on_resize_plan(800, 600).is_empty());
        assert_eq!(
            dialog.on_resize_plan(1024, 768),
            vec![
                SettingsMenuDialogAction::RebuildTable("graphics"),
                SettingsMenuDialogAction::RebuildTable("sound"),
                SettingsMenuDialogAction::RebuildTable("game"),
                SettingsMenuDialogAction::UpdateScrollFocus,
            ]
        );

        dialog.open_target_plan(SettingsMenuTarget::Table("graphics"));
        assert_eq!(dialog.page, SettingsMenuPage::Table("graphics"));
        assert_eq!(
            dialog.press_back_or_escape_plan(),
            vec![
                SettingsMenuDialogAction::BackToMain,
                SettingsMenuDialogAction::RebuildMenu
            ]
        );
        assert_eq!(
            dialog.press_back_or_escape_plan(),
            vec![SettingsMenuDialogAction::Hide]
        );

        let reset = reset_table_plan("sound", &context);
        assert_eq!(
            reset,
            SettingsMenuDialogAction::ResetTable {
                table: "sound".to_string(),
                remove_keys: vec!["alwaysmusic", "musicvol", "sfxvol", "ambientvol"],
            }
        );
    }

    #[test]
    fn setting_change_and_initial_side_effects_cover_runtime_callbacks() {
        let context = SettingsMenuContext {
            steam: true,
            ui_scale_at_open: 100,
            ..SettingsMenuContext::default()
        };
        let player_limit = settings_pref_spec("game", "playerlimit").unwrap();
        assert_eq!(
            setting_change_plan(player_limit, "8", &context),
            vec![
                SettingsMenuDialogAction::PutInt {
                    key: "playerlimit",
                    value: 8,
                },
                SettingsMenuDialogAction::UpdateLobby,
            ]
        );

        let ui_scale = settings_pref_spec("graphics", "uiscale").unwrap();
        assert_eq!(
            setting_change_plan(ui_scale, "125", &context),
            vec![
                SettingsMenuDialogAction::PutInt {
                    key: "uiscale",
                    value: 125,
                },
                SettingsMenuDialogAction::SetUiScaleChanged(true),
            ]
        );

        let pixelate = settings_pref_spec("graphics", "pixelate").unwrap();
        assert_eq!(
            setting_change_plan(pixelate, "true", &context),
            vec![
                SettingsMenuDialogAction::PutBool {
                    key: "pixelate",
                    value: true,
                },
                SettingsMenuDialogAction::FireEnablePixelation,
            ]
        );

        let values = vec![
            (
                settings_storage_key("graphics", "fullscreen"),
                "true".to_string(),
            ),
            (
                settings_storage_key("graphics", "linear"),
                "true".to_string(),
            ),
        ];
        let initial = initial_settings_side_effects_plan(&context, &values);
        assert!(initial.contains(&SettingsMenuDialogAction::SetVsync(true)));
        assert!(initial.contains(&SettingsMenuDialogAction::SetFullscreen(true)));
        assert!(initial.contains(&SettingsMenuDialogAction::PutBool {
            key: "swapdiagonal",
            value: false,
        }));
        assert!(initial.contains(&SettingsMenuDialogAction::SetTextureFilterLinear(true)));
    }

    #[test]
    fn data_action_plan_models_confirm_and_platform_branches() {
        let desktop = SettingsMenuContext::default();
        assert_eq!(
            data_action_plan(SettingsDataActionKind::ClearResearch, &desktop),
            vec![
                SettingsMenuDialogAction::Confirm {
                    title: "@confirm",
                    message: "@settings.clearresearch.confirm",
                    action: SettingsDataActionKind::ClearResearch,
                },
                SettingsMenuDialogAction::ClearLoadoutInfo,
                SettingsMenuDialogAction::ResetTechTreeAll,
                SettingsMenuDialogAction::ClearUnlockableContent,
                SettingsMenuDialogAction::RemoveSetting("unlocks"),
            ]
        );
        assert_eq!(
            data_action_plan(SettingsDataActionKind::ExportData, &desktop),
            vec![
                SettingsMenuDialogAction::ShowFileChooser {
                    open: false,
                    extension: "zip",
                    action: SettingsDataActionKind::ExportData,
                },
                SettingsMenuDialogAction::ShowInfo("@data.exported"),
            ]
        );

        let ios = SettingsMenuContext {
            ios: true,
            mobile: true,
            ..SettingsMenuContext::default()
        };
        assert_eq!(
            data_action_plan(SettingsDataActionKind::ExportData, &ios),
            vec![
                SettingsMenuDialogAction::ExportDataToLocalFile(SETTINGS_IOS_EXPORT_FILE),
                SettingsMenuDialogAction::ShareFile(SETTINGS_IOS_EXPORT_FILE.to_string()),
            ]
        );

        assert_eq!(
            planet_data_action_plan(SettingsPlanetDataActionKind::ClearPlanetResearch, "erekir"),
            vec![
                SettingsMenuDialogAction::ConfirmPlanet {
                    title: "@confirm",
                    message_key: "settings.clearplanetresearch.confirm",
                    planet_id: "erekir".to_string(),
                },
                SettingsMenuDialogAction::ClearLoadoutInfo,
                SettingsMenuDialogAction::ResetTechTreeForPlanet("erekir".to_string()),
                SettingsMenuDialogAction::ClearUnlockableContentForPlanet("erekir".to_string()),
                SettingsMenuDialogAction::RemoveSetting("unlocks"),
            ]
        );
    }

    #[test]
    fn export_import_and_crash_log_helpers_match_java_shape() {
        let entries = export_data_archive_entries(
            "D:/data",
            "D:/data/settings.bin",
            &["D:/data/maps/custom.msav"],
            &["D:/data/saves/slot1.msav"],
            &[],
            &["D:/data/schematics/foo.msch"],
        );
        let paths: Vec<_> = entries.iter().map(|entry| entry.path.as_str()).collect();
        assert!(paths.contains(&"settings.bin"));
        assert!(paths.contains(&"maps/custom.msav"));
        assert!(paths.contains(&"maps/"));
        assert!(paths.contains(&"saves/"));
        assert!(paths.contains(&"schematics/"));

        assert_eq!(
            import_data_plan(&["settings.bin", "saves/slot1.msav"]).unwrap(),
            vec![
                SettingsMenuDialogAction::ImportDataFromZip,
                SettingsMenuDialogAction::DeleteAllSaves,
                SettingsMenuDialogAction::ClearSettings,
            ]
        );
        assert_eq!(
            import_data_plan(&["saves/slot1.msav"]),
            Err(SettingsDataImportError::MissingSettingsBin)
        );

        assert_eq!(
            crash_logs_text(&[("crash1.txt", "boom")], Some("last")),
            "crash1.txt\n\nboom\n\nlast log:\nlast"
        );
    }
}
