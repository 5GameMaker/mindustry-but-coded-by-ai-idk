//! Mods dialog model mirroring upstream `mindustry.ui.dialogs.ModsDialog`.

use crate::mindustry::ui::{
    upstream_menu_bundle_format_for_locale, upstream_menu_bundle_value_for_locale,
};

pub const MODS_DIALOG_TITLE: &str = "@mods";
pub const MODS_BROWSER_TITLE: &str = "@mods.browser";
pub const MODS_GUIDE_BUTTON_TEXT: &str = "@mods.guide";
pub const MODS_OPEN_FOLDER_TEXT: &str = "@mods.openfolder";
pub const MODS_IMPORT_BUTTON_TEXT: &str = "@mod.import";
pub const MODS_IMPORT_FILE_TEXT: &str = "@mod.import.file";
pub const MODS_IMPORT_GITHUB_TEXT: &str = "@mod.import.github";
pub const MODS_BROWSER_BUTTON_TEXT: &str = "@mods.browser";
pub const MODS_RELOAD_REQUIRED_TEXT: &str = "@mod.reloadrequired";
pub const MODS_NONE_TEXT: &str = "@mods.none";
pub const MODS_NONE_FOUND_TEXT: &str = "@none.found";
pub const MODS_LOADING_TEXT: &str = "@loading";
pub const MODS_DOWNLOADING_TEXT: &str = "@downloading";
pub const MODS_BROWSER_FETCHING_TEXT: &str = "mods.browser.fetching";
pub const MODS_BROWSER_NO_RELEASES_TEXT: &str = "@mods.browser.noreleases";
pub const MODS_CARD_HEIGHT: f32 = 110.0;
pub const MODS_CARD_MAX_WIDTH: f32 = 520.0;
pub const MODS_CARD_WIDTH_PAD: f32 = 28.0;
pub const MODS_MAIN_MAX_WIDTH: f32 = 556.0;
pub const MODS_MAIN_WIDTH_SCALE: f32 = 1.05;
pub const MODS_BUTTON_HEIGHT: f32 = 60.0;
pub const MODS_TOP_BUTTON_MARGIN: f32 = 12.0;
pub const MODS_IMPORT_DIALOG_BUTTON_SIZE: (f32, f32) = (300.0, 70.0);
pub const MODS_IMPORT_DIALOG_MARGIN: f32 = 12.0;
pub const MODS_LIST_PANE_MARGIN: f32 = 10.0;
pub const MODS_DISABLED_DIVIDER_HEIGHT: f32 = 4.0;
pub const MODS_MOD_ACTION_BUTTON_SIZE: f32 = 50.0;
pub const MODS_BROWSER_ICON_SIZE: f32 = 64.0;
pub const MODS_BROWSER_CARD_WIDTH: f32 = 438.0;
pub const MODS_BROWSER_CARD_HEIGHT: f32 = 80.0;
pub const MODS_BROWSER_COLUMNS_WIDTH: f32 = 480.0;
pub const MODS_BROWSER_INFO_WIDTH: f32 = 358.0;
pub const MODS_SELECTION_BUTTON_SIZE: (f32, f32) = (150.0, 54.0);
pub const MODS_SELECTION_DESKTOP_WIDTH: f32 = 500.0;
pub const MODS_SELECTION_MOBILE_WIDTH: f32 = 400.0;
pub const MODS_DETAILS_WIDTH: f32 = 400.0;
pub const MODS_VIEW_CONTENT_BUTTON_SIZE: (f32, f32) = (300.0, 50.0);
pub const MODS_CONTENT_ICON_SIZE: f32 = 50.0;
pub const MODS_GITHUB_PREFIX: &str = "https://github.com/";
pub const MODS_GITHUB_API_PREFIX: &str = "https://api.github.com";
pub const MODS_ICON_CACHE_PREFIX: &str =
    "https://raw.githubusercontent.com/Anuken/MindustryMods/master/icons/";
pub const MODS_MIN_JAVA_MOD_GAME_VERSION: f64 = 154.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModsDialogContext {
    pub mobile: bool,
    pub portrait: bool,
    pub steam: bool,
    pub ios: bool,
    pub graphics_width: f32,
    pub scl: f32,
    pub current_game_version: f64,
    pub min_java_mod_game_version: f64,
}

impl Default for ModsDialogContext {
    fn default() -> Self {
        Self {
            mobile: false,
            portrait: false,
            steam: false,
            ios: false,
            graphics_width: 800.0,
            scl: 1.0,
            current_game_version: 157.4,
            min_java_mod_game_version: MODS_MIN_JAVA_MOD_GAME_VERSION,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsDialogModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub guide_button: ModsTopButton,
    pub open_folder_button: Option<ModsTopButton>,
    pub reload_required_visible: bool,
    pub import_button: ModsTopButton,
    pub browser_button: ModsTopButton,
    pub search_visible: bool,
    pub empty_text: Option<&'static str>,
    pub cards: Vec<ModsInstalledCard>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsTopButton {
    pub text: &'static str,
    pub icon: &'static str,
    pub height: f32,
    pub margin: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsInstalledCard {
    pub index: usize,
    pub display_name: String,
    pub short_description: Option<String>,
    pub icon: ModsInstalledIcon,
    pub state_text: Option<&'static str>,
    pub state_details: Option<String>,
    pub disabled_text_visible: bool,
    pub divider_before: bool,
    pub enable_button: ModsInstalledActionButton,
    pub delete_or_link_button: ModsInstalledActionButton,
    pub publish_button: Option<ModsInstalledActionButton>,
    pub size: (f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsInstalledIcon {
    pub drawable: &'static str,
    pub border: &'static str,
    pub size: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsInstalledActionButton {
    pub icon: &'static str,
    pub size: f32,
    pub disabled: bool,
    pub action: ModsDialogAction,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsImportDialogModel {
    pub title: &'static str,
    pub table_background: &'static str,
    pub buttons: Vec<ModsImportButton>,
    pub close_button_added: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsImportButton {
    pub text: &'static str,
    pub icon: &'static str,
    pub size: (f32, f32),
    pub margin: f32,
    pub action: ModsDialogAction,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsLoadedMod {
    pub internal_name: String,
    pub display_name: String,
    pub author: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub repo: Option<String>,
    pub min_game_version: Option<String>,
    pub short_description: Option<String>,
    pub enabled: bool,
    pub state: ModsLoadedModState,
    pub missing_dependencies: Vec<String>,
    pub has_steam_id: bool,
    pub has_icon_texture: bool,
    pub hidden: bool,
    pub is_java: bool,
    pub content_count: usize,
    pub file_path: String,
}

impl ModsLoadedMod {
    pub fn supported(&self) -> bool {
        self.state != ModsLoadedModState::IncompatibleGame
            && self.state != ModsLoadedModState::IncompatibleMod
            && self.state != ModsLoadedModState::Blacklisted
    }

    pub fn has_unmet_dependencies(&self) -> bool {
        self.state == ModsLoadedModState::UnmetDependencies
    }

    pub fn has_content_errors(&self) -> bool {
        self.state == ModsLoadedModState::ErroredContent
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ModsLoadedModState {
    #[default]
    Loaded,
    IncompatibleGame,
    IncompatibleMod,
    Blacklisted,
    CircularDependencies,
    IncompleteDependencies,
    UnmetDependencies,
    ErroredContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModsListing {
    pub name: String,
    pub repo: String,
    pub internal_name: String,
    pub author: String,
    pub description: String,
    pub stars: i32,
    pub last_updated: String,
    pub min_game_version: String,
    pub has_java: bool,
    pub has_scripts: bool,
    pub ios_compatible: bool,
    pub legacy_compatible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsBrowserModel {
    pub title: &'static str,
    pub search_text: String,
    pub order_date: bool,
    pub sort_icon: &'static str,
    pub sort_tooltip: &'static str,
    pub columns: i32,
    pub loading: bool,
    pub cards: Vec<ModsBrowserCard>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsBrowserCard {
    pub listing: ModsListing,
    pub installed: bool,
    pub icon_url: String,
    pub icon_cache_key: String,
    pub compatibility: ModsBrowserCompatibility,
    pub size: (f32, f32),
    pub icon_size: f32,
    pub info_width: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModsBrowserCompatibility {
    Compatible,
    RequiresVersion(String),
    IncompatibleJavaMod,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsSelectionDialogModel {
    pub title: String,
    pub description_text: String,
    pub width: f32,
    pub buttons: Vec<ModsSelectionButton>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsSelectionButton {
    pub text: &'static str,
    pub icon: &'static str,
    pub size: (f32, f32),
    pub action: ModsDialogAction,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsReleaseDialogModel {
    pub title: &'static str,
    pub entries: Vec<ModsReleaseRow>,
    pub back_button: ModsSelectionButton,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsReleaseRow {
    pub name: String,
    pub date: String,
    pub latest: bool,
    pub open_url: String,
    pub release_id: String,
    pub buttons: Vec<ModsSelectionButton>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsRelease {
    pub name: String,
    pub published_at: String,
    pub html_url: String,
    pub api_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsModDetailsModel {
    pub title: String,
    pub width: f32,
    pub buttons: Vec<ModsSelectionButton>,
    pub fields: Vec<ModsModDetailsField>,
    pub disabled_details: Vec<String>,
    pub view_content_button: Option<ModsSelectionButton>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModsModDetailsField {
    pub label: &'static str,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModsDialogAction {
    OpenGuide,
    OpenModsFolder,
    ShowImportDialog,
    ShowBrowser,
    RebuildBrowser,
    ToggleBrowserSort,
    SetBrowserSearch(String),
    ShowFileChooser {
        extensions: Vec<&'static str>,
    },
    ShowGithubTextInput {
        last_value: String,
    },
    StoreLastGithubMod(String),
    ToggleModEnabled(String),
    ConfirmRemoveMod(String),
    RemoveMod(String),
    ViewSteamListing(String),
    PublishMod(String),
    OpenModFolder(String),
    OpenGithub(String),
    ReinstallGithub {
        repo: String,
        is_java: bool,
    },
    ShowMod(String),
    ShowModContent(String),
    ShowContent(String),
    HideCurrentContent,
    ReloadMods,
    ShowLoading(&'static str),
    HideLoading,
    SetImportProgressAvailable(bool),
    SetImportProgress(f32),
    SetImportCancelButton,
    CancelImport,
    GithubProbeRepo {
        repo: String,
        release: Option<String>,
    },
    GithubFetchJavaRelease {
        repo: String,
        release: Option<String>,
    },
    GithubFetchBranchZip {
        repo: String,
        branch: String,
        release: Option<String>,
    },
    GithubFetchReleaseZipball {
        repo: String,
        release: String,
    },
    GithubDownload {
        repo: String,
        url: String,
    },
    HandleDownloadedMod {
        repo: String,
        temp_file: String,
    },
    ImportDownloadedMod {
        temp_file: String,
        repo: String,
    },
    DeleteTempFile(String),
    Setup,
    ImportDependency {
        internal_name: String,
        repo: String,
        is_java: bool,
    },
    DependenciesDone {
        remaining: Vec<String>,
    },
    ShowErrorMessage(String),
    ShowException {
        title: Option<&'static str>,
        message: String,
    },
    ShowInfo(&'static str),
    OpenReleaseUrl(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModsImportErrorInput {
    NoSuchMethod,
    HttpStatus(String),
    Message(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModsImportErrorKind {
    FeatureUnsupported,
    ConnectFail(String),
    WritableDex,
    Exception(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModsDialog {
    pub search_text: String,
    pub mod_list: Option<Vec<ModsListing>>,
    pub order_date: bool,
    pub mod_import_progress: f32,
    pub cancelled_import: bool,
    pub current_content_open: bool,
    pub browser_visible: bool,
    pub scroll: f32,
}

impl Default for ModsDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ModsDialog {
    pub fn new() -> Self {
        Self {
            search_text: String::new(),
            mod_list: None,
            order_date: true,
            mod_import_progress: 0.0,
            cancelled_import: false,
            current_content_open: false,
            browser_visible: false,
            scroll: 0.0,
        }
    }

    pub fn setup_model(
        &self,
        mods: &[ModsLoadedMod],
        requires_reload: bool,
        query: &str,
        context: &ModsDialogContext,
        locale: &str,
    ) -> ModsDialogModel {
        let cards = installed_cards(mods, query, context, locale);
        ModsDialogModel {
            title: MODS_DIALOG_TITLE,
            close_button_added: true,
            guide_button: ModsTopButton {
                text: MODS_GUIDE_BUTTON_TEXT,
                icon: "link",
                height: 64.0,
                margin: 0.0,
            },
            open_folder_button: (!context.mobile).then_some(ModsTopButton {
                text: MODS_OPEN_FOLDER_TEXT,
                icon: "link",
                height: 64.0,
                margin: 0.0,
            }),
            reload_required_visible: requires_reload,
            import_button: ModsTopButton {
                text: MODS_IMPORT_BUTTON_TEXT,
                icon: "add",
                height: MODS_BUTTON_HEIGHT,
                margin: MODS_TOP_BUTTON_MARGIN,
            },
            browser_button: ModsTopButton {
                text: MODS_BROWSER_BUTTON_TEXT,
                icon: "menu",
                height: MODS_BUTTON_HEIGHT,
                margin: MODS_TOP_BUTTON_MARGIN,
            },
            search_visible: !mods.is_empty() && (!context.mobile || context.portrait),
            empty_text: mods.is_empty().then_some(MODS_NONE_TEXT),
            cards,
        }
    }

    pub fn import_dialog_model(&self) -> ModsImportDialogModel {
        ModsImportDialogModel {
            title: MODS_IMPORT_BUTTON_TEXT,
            table_background: "Tex.button",
            close_button_added: true,
            buttons: vec![
                ModsImportButton {
                    text: MODS_IMPORT_FILE_TEXT,
                    icon: "file",
                    size: MODS_IMPORT_DIALOG_BUTTON_SIZE,
                    margin: MODS_IMPORT_DIALOG_MARGIN,
                    action: ModsDialogAction::ShowFileChooser {
                        extensions: vec!["zip", "jar"],
                    },
                },
                ModsImportButton {
                    text: MODS_IMPORT_GITHUB_TEXT,
                    icon: "github",
                    size: MODS_IMPORT_DIALOG_BUTTON_SIZE,
                    margin: MODS_IMPORT_DIALOG_MARGIN,
                    action: ModsDialogAction::ShowGithubTextInput {
                        last_value: String::new(),
                    },
                },
            ],
        }
    }

    pub fn show_mod_browser_plan(&mut self) -> Vec<ModsDialogAction> {
        self.browser_visible = true;
        vec![
            ModsDialogAction::RebuildBrowser,
            ModsDialogAction::ShowBrowser,
        ]
    }

    pub fn toggle_browser_sort_plan(&mut self) -> Vec<ModsDialogAction> {
        self.order_date = !self.order_date;
        vec![
            ModsDialogAction::ToggleBrowserSort,
            ModsDialogAction::RebuildBrowser,
        ]
    }

    pub fn browser_model(
        &self,
        installed_mods: &[ModsLoadedMod],
        context: &ModsDialogContext,
    ) -> ModsBrowserModel {
        match &self.mod_list {
            Some(listings) => browser_model_from_listings(
                listings,
                installed_mods,
                &self.search_text,
                self.order_date,
                context,
            ),
            None => ModsBrowserModel {
                title: MODS_BROWSER_TITLE,
                search_text: self.search_text.clone(),
                order_date: self.order_date,
                sort_icon: if self.order_date { "list" } else { "star" },
                sort_tooltip: if self.order_date {
                    "@mods.browser.sortdate"
                } else {
                    "@mods.browser.sortstars"
                },
                columns: browser_columns(context),
                loading: true,
                cards: Vec::new(),
            },
        }
    }

    pub fn set_mod_list(&mut self, listings: Vec<ModsListing>) {
        self.mod_list = Some(sort_listings_by_date(listings));
    }

    pub fn hidden_plan(&self, requires_reload: bool) -> Vec<ModsDialogAction> {
        if requires_reload {
            vec![ModsDialogAction::ReloadMods]
        } else {
            Vec::new()
        }
    }

    pub fn resize_event_plan(&mut self) -> Vec<ModsDialogAction> {
        if self.current_content_open {
            self.current_content_open = false;
            vec![ModsDialogAction::HideCurrentContent]
        } else {
            Vec::new()
        }
    }

    pub fn github_import_mod_plan(
        &mut self,
        repo: impl Into<String>,
        is_java: bool,
        release: Option<String>,
    ) -> Vec<ModsDialogAction> {
        self.mod_import_progress = 0.0;
        self.cancelled_import = false;
        github_import_mod_plan(repo.into(), is_java, release)
    }
}

pub fn installed_cards(
    mods: &[ModsLoadedMod],
    query: &str,
    context: &ModsDialogContext,
    locale: &str,
) -> Vec<ModsInstalledCard> {
    let mut cards = Vec::new();
    let mut any_disabled = false;
    for (index, item) in mods.iter().enumerate() {
        if !mods_matches(query, &item.display_name) {
            continue;
        }

        let divider_before = !item.enabled && !any_disabled && !mods.is_empty();
        if divider_before {
            any_disabled = true;
        }

        let hide_disabled =
            !item.supported() || item.has_unmet_dependencies() || item.has_content_errors();
        cards.push(ModsInstalledCard {
            index,
            display_name: strip_colors(&item.display_name),
            short_description: item
                .short_description
                .clone()
                .filter(|value| !value.is_empty()),
            icon: ModsInstalledIcon {
                drawable: if item.has_icon_texture {
                    "TextureRegion(mod.iconTexture)"
                } else {
                    "Tex.nomap"
                },
                border: "Pal.accent",
                size: MODS_CARD_HEIGHT - 8.0,
            },
            state_text: state_text_key(item),
            state_details: state_details_text(item, locale),
            disabled_text_visible: !item.enabled && !hide_disabled,
            divider_before,
            enable_button: ModsInstalledActionButton {
                icon: if item.enabled { "downOpen" } else { "upOpen" },
                size: MODS_MOD_ACTION_BUTTON_SIZE,
                disabled: !item.supported(),
                action: ModsDialogAction::ToggleModEnabled(item.internal_name.clone()),
            },
            delete_or_link_button: ModsInstalledActionButton {
                icon: if item.has_steam_id { "link" } else { "trash" },
                size: MODS_MOD_ACTION_BUTTON_SIZE,
                disabled: false,
                action: if item.has_steam_id {
                    ModsDialogAction::ViewSteamListing(item.internal_name.clone())
                } else {
                    ModsDialogAction::ConfirmRemoveMod(item.internal_name.clone())
                },
            },
            publish_button: (context.steam && !item.has_steam_id).then(|| {
                ModsInstalledActionButton {
                    icon: "export",
                    size: MODS_MOD_ACTION_BUTTON_SIZE,
                    disabled: false,
                    action: ModsDialogAction::PublishMod(item.internal_name.clone()),
                }
            }),
            size: (installed_card_width(context), MODS_CARD_HEIGHT),
        });
    }
    cards
}

pub fn installed_card_width(context: &ModsDialogContext) -> f32 {
    (context.graphics_width / context.scl / MODS_MAIN_WIDTH_SCALE - MODS_CARD_WIDTH_PAD)
        .min(MODS_CARD_MAX_WIDTH)
}

pub fn state_text_key(item: &ModsLoadedMod) -> Option<&'static str> {
    match item.state {
        ModsLoadedModState::IncompatibleMod => Some("@mod.incompatiblemod"),
        ModsLoadedModState::Blacklisted => Some("@mod.blacklisted"),
        ModsLoadedModState::IncompatibleGame => Some("@mod.incompatiblegame"),
        ModsLoadedModState::CircularDependencies => Some("@mod.circulardependencies"),
        ModsLoadedModState::IncompleteDependencies => Some("@mod.incompletedependencies"),
        ModsLoadedModState::UnmetDependencies => Some("@mod.unmetdependencies"),
        ModsLoadedModState::ErroredContent => Some("@mod.erroredcontent"),
        ModsLoadedModState::Loaded if item.hidden => Some("@mod.multiplayer.compatible"),
        ModsLoadedModState::Loaded => None,
    }
}

pub fn state_details_text(item: &ModsLoadedMod, locale: &str) -> Option<String> {
    match item.state {
        ModsLoadedModState::IncompatibleMod => Some("@mod.incompatiblemod.details".to_string()),
        ModsLoadedModState::Blacklisted => Some("@mod.blacklisted.details".to_string()),
        ModsLoadedModState::IncompatibleGame => Some(
            upstream_menu_bundle_format_for_locale(
                locale,
                "mod.requiresversion.details",
                &[item.min_game_version.as_deref().unwrap_or("")],
            )
            .unwrap_or_else(|| {
                format!(
                    "mod.requiresversion.details:{}",
                    item.min_game_version.as_deref().unwrap_or("")
                )
            }),
        ),
        ModsLoadedModState::CircularDependencies => {
            Some("@mod.circulardependencies.details".to_string())
        }
        ModsLoadedModState::IncompleteDependencies => Some(format_dependency_details(
            locale,
            "mod.incompletedependencies.details",
            &item.missing_dependencies,
        )),
        ModsLoadedModState::UnmetDependencies => Some(format_dependency_details(
            locale,
            "mod.missingdependencies.details",
            &item.missing_dependencies,
        )),
        ModsLoadedModState::ErroredContent => Some("@mod.erroredcontent.details".to_string()),
        ModsLoadedModState::Loaded => None,
    }
}

fn format_dependency_details(locale: &str, key: &str, dependencies: &[String]) -> String {
    let joined = dependencies.join(", ");
    upstream_menu_bundle_format_for_locale(locale, key, &[&joined])
        .unwrap_or_else(|| format!("{key}:{joined}"))
}

pub fn show_mod_model(
    item: &ModsLoadedMod,
    context: &ModsDialogContext,
    locale: &str,
) -> ModsModDetailsModel {
    let mut buttons = Vec::new();
    if !context.mobile {
        buttons.push(ModsSelectionButton {
            text: MODS_OPEN_FOLDER_TEXT,
            icon: "link",
            size: MODS_SELECTION_BUTTON_SIZE,
            action: ModsDialogAction::OpenModFolder(item.file_path.clone()),
        });
    }
    if let Some(repo) = &item.repo {
        buttons.push(ModsSelectionButton {
            text: "@mods.github.open",
            icon: "link",
            size: MODS_SELECTION_BUTTON_SIZE,
            action: ModsDialogAction::OpenGithub(format!("{MODS_GITHUB_PREFIX}{repo}")),
        });
        if !item.has_steam_id {
            buttons.push(ModsSelectionButton {
                text: "@mods.browser.reinstall",
                icon: "download",
                size: MODS_SELECTION_BUTTON_SIZE,
                action: ModsDialogAction::ReinstallGithub {
                    repo: repo.clone(),
                    is_java: item.is_java,
                },
            });
        }
    }

    let mut fields = vec![ModsModDetailsField {
        label: "@editor.name",
        value: item.display_name.clone(),
    }];
    if let Some(author) = &item.author {
        fields.push(ModsModDetailsField {
            label: "@editor.author",
            value: author.clone(),
        });
    }
    if let Some(version) = &item.version {
        fields.push(ModsModDetailsField {
            label: "@mod.version",
            value: version.clone(),
        });
    }
    if let Some(description) = &item.description {
        fields.push(ModsModDetailsField {
            label: "@editor.description",
            value: description.clone(),
        });
    }

    ModsModDetailsModel {
        title: item.display_name.clone(),
        width: MODS_DETAILS_WIDTH,
        buttons,
        fields,
        disabled_details: state_details_text(item, locale).into_iter().collect(),
        view_content_button: (item.content_count > 0).then(|| ModsSelectionButton {
            text: "@mods.viewcontent",
            icon: "book",
            size: MODS_VIEW_CONTENT_BUTTON_SIZE,
            action: ModsDialogAction::ShowModContent(item.internal_name.clone()),
        }),
    }
}

pub fn browser_model_from_listings(
    listings: &[ModsListing],
    installed_mods: &[ModsLoadedMod],
    search: &str,
    order_date: bool,
    context: &ModsDialogContext,
) -> ModsBrowserModel {
    let mut listings = listings.to_vec();
    if order_date {
        listings = sort_listings_by_date(listings);
    } else {
        listings.sort_by(|left, right| right.stars.cmp(&left.stars));
    }

    let installed_repos: Vec<&str> = installed_mods
        .iter()
        .filter_map(|item| item.repo.as_deref())
        .collect();
    let cards = listings
        .into_iter()
        .filter(|listing| listing_visible(listing, search, context))
        .map(|listing| {
            let installed = installed_repos.iter().any(|repo| *repo == listing.repo);
            let compatibility = browser_compatibility(&listing, context);
            ModsBrowserCard {
                icon_url: browser_icon_url(&listing.repo),
                icon_cache_key: listing.repo.replace('/', "_"),
                size: (MODS_BROWSER_CARD_WIDTH, MODS_BROWSER_CARD_HEIGHT),
                icon_size: MODS_BROWSER_ICON_SIZE,
                info_width: MODS_BROWSER_INFO_WIDTH,
                listing,
                installed,
                compatibility,
            }
        })
        .collect();

    ModsBrowserModel {
        title: MODS_BROWSER_TITLE,
        search_text: search.to_string(),
        order_date,
        sort_icon: if order_date { "list" } else { "star" },
        sort_tooltip: if order_date {
            "@mods.browser.sortdate"
        } else {
            "@mods.browser.sortstars"
        },
        columns: browser_columns(context),
        loading: false,
        cards,
    }
}

pub fn sort_listings_by_date(mut listings: Vec<ModsListing>) -> Vec<ModsListing> {
    listings.sort_by(|left, right| listing_date_key(right).cmp(&listing_date_key(left)));
    listings
}

fn listing_date_key(listing: &ModsListing) -> String {
    let value = listing.last_updated.trim();
    if value.len() >= 20 && value.as_bytes().get(4) == Some(&b'-') {
        value.to_string()
    } else {
        "9999-12-31T23:59:59Z".to_string()
    }
}

pub fn browser_columns(context: &ModsDialogContext) -> i32 {
    (context.graphics_width / (context.scl * MODS_BROWSER_COLUMNS_WIDTH)).max(1.0) as i32
}

pub fn listing_visible(listing: &ModsListing, search: &str, context: &ModsDialogContext) -> bool {
    if ((listing.has_java || (listing.has_scripts && !listing.ios_compatible)) && context.ios)
        || (!mods_matches(search, &listing.name) && !mods_matches(search, &listing.repo))
    {
        return false;
    }
    true
}

pub fn browser_compatibility(
    listing: &ModsListing,
    context: &ModsDialogContext,
) -> ModsBrowserCompatibility {
    if !version_is_at_least(context.current_game_version, &listing.min_game_version) {
        ModsBrowserCompatibility::RequiresVersion(listing.min_game_version.clone())
    } else if listing.has_java
        && parse_version_number(&listing.min_game_version) < context.min_java_mod_game_version
        && !listing.legacy_compatible
    {
        ModsBrowserCompatibility::IncompatibleJavaMod
    } else {
        ModsBrowserCompatibility::Compatible
    }
}

pub fn browser_icon_url(repo: &str) -> String {
    format!("{MODS_ICON_CACHE_PREFIX}{}", repo.replace('/', "_"))
}

pub fn selection_dialog_model(
    listing: &ModsListing,
    installed: bool,
    context: &ModsDialogContext,
) -> ModsSelectionDialogModel {
    ModsSelectionDialogModel {
        title: listing.name.clone(),
        description_text: format!(
            "{}\n\n[accent]{}[lightgray] {}",
            listing.description,
            upstream_menu_bundle_value_for_locale("en", "editor.author").unwrap_or("Author:"),
            listing.author
        ),
        width: if context.mobile {
            MODS_SELECTION_MOBILE_WIDTH
        } else {
            MODS_SELECTION_DESKTOP_WIDTH
        },
        buttons: vec![
            ModsSelectionButton {
                text: "@back",
                icon: "left",
                size: MODS_SELECTION_BUTTON_SIZE,
                action: ModsDialogAction::RebuildBrowser,
            },
            ModsSelectionButton {
                text: if installed {
                    "@mods.browser.reinstall"
                } else {
                    "@mods.browser.add"
                },
                icon: "download",
                size: MODS_SELECTION_BUTTON_SIZE,
                action: ModsDialogAction::ReinstallGithub {
                    repo: listing.repo.clone(),
                    is_java: listing.has_java,
                },
            },
            ModsSelectionButton {
                text: "@mods.github.open",
                icon: "link",
                size: MODS_SELECTION_BUTTON_SIZE,
                action: ModsDialogAction::OpenGithub(format!(
                    "{}{}",
                    MODS_GITHUB_PREFIX, listing.repo
                )),
            },
            ModsSelectionButton {
                text: "@mods.browser.view-releases",
                icon: "zoom",
                size: MODS_SELECTION_BUTTON_SIZE,
                action: ModsDialogAction::GithubFetchJavaRelease {
                    repo: listing.repo.clone(),
                    release: Some("releases".to_string()),
                },
            },
        ],
    }
}

pub fn release_dialog_model(
    repo: &str,
    releases: &[ModsRelease],
) -> Option<ModsReleaseDialogModel> {
    if releases.is_empty() {
        return None;
    }
    Some(ModsReleaseDialogModel {
        title: "@mods.browser.releases",
        entries: releases
            .iter()
            .enumerate()
            .map(|(index, release)| {
                let release_id = release
                    .api_url
                    .rsplit('/')
                    .next()
                    .expect("release url must have id")
                    .to_string();
                ModsReleaseRow {
                    name: release.name.clone(),
                    date: release_date_label(&release.published_at),
                    latest: index == 0,
                    open_url: release.html_url.clone(),
                    release_id: release_id.clone(),
                    buttons: vec![
                        ModsSelectionButton {
                            text: "@mods.github.open-release",
                            icon: "link",
                            size: MODS_SELECTION_BUTTON_SIZE,
                            action: ModsDialogAction::OpenReleaseUrl(release.html_url.clone()),
                        },
                        ModsSelectionButton {
                            text: "@mods.browser.add",
                            icon: "download",
                            size: MODS_SELECTION_BUTTON_SIZE,
                            action: ModsDialogAction::ReinstallGithub {
                                repo: repo.to_string(),
                                is_java: false,
                            },
                        },
                    ],
                }
            })
            .collect(),
        back_button: ModsSelectionButton {
            text: "@back",
            icon: "left",
            size: MODS_SELECTION_BUTTON_SIZE,
            action: ModsDialogAction::RebuildBrowser,
        },
    })
}

pub fn release_date_label(published_at: &str) -> String {
    published_at
        .get(0..10)
        .unwrap_or(published_at)
        .replace('-', "/")
}

pub fn clean_github_repo_text(text: &str) -> String {
    let trimmed = text.trim().replace(' ', "");
    trimmed
        .strip_prefix(MODS_GITHUB_PREFIX)
        .unwrap_or(&trimmed)
        .to_string()
}

pub fn github_import_mod_plan(
    repo: String,
    is_java: bool,
    release: Option<String>,
) -> Vec<ModsDialogAction> {
    let mut out = vec![
        ModsDialogAction::ShowLoading(MODS_DOWNLOADING_TEXT),
        ModsDialogAction::SetImportProgress(0.0),
        ModsDialogAction::SetImportCancelButton,
    ];
    if is_java {
        out.push(ModsDialogAction::GithubFetchJavaRelease { repo, release });
    } else {
        out.push(ModsDialogAction::GithubProbeRepo { repo, release });
    }
    out
}

pub fn github_probe_result_plan(
    repo: impl Into<String>,
    default_branch: impl Into<String>,
    language: impl Into<String>,
    release: Option<String>,
) -> Vec<ModsDialogAction> {
    let repo = repo.into();
    let language = language.into();
    if is_java_like_language(&language) {
        vec![ModsDialogAction::GithubFetchJavaRelease { repo, release }]
    } else {
        vec![ModsDialogAction::GithubFetchBranchZip {
            repo,
            branch: default_branch.into(),
            release,
        }]
    }
}

pub fn is_java_like_language(language: &str) -> bool {
    matches!(language, "Java" | "Kotlin" | "Groovy" | "Scala")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModsGithubAsset {
    pub name: String,
    pub browser_download_url: String,
}

pub fn select_java_release_asset(assets: &[ModsGithubAsset]) -> Option<ModsGithubAsset> {
    assets
        .iter()
        .find(|asset| asset.name.starts_with("dexed") && asset.name.ends_with(".jar"))
        .or_else(|| assets.iter().find(|asset| asset.name.ends_with(".jar")))
        .cloned()
}

pub fn java_release_assets_plan(
    repo: impl Into<String>,
    assets: &[ModsGithubAsset],
) -> Result<Vec<ModsDialogAction>, String> {
    let repo = repo.into();
    let asset = select_java_release_asset(assets).ok_or_else(|| {
        "No JAR file found in releases. Make sure you have a valid jar file in the mod's latest Github Release.".to_string()
    })?;
    Ok(vec![ModsDialogAction::GithubDownload {
        repo,
        url: asset.browser_download_url,
    }])
}

pub fn branch_zip_plan(
    repo: impl Into<String>,
    branch: impl Into<String>,
    release: Option<String>,
    location: Option<String>,
) -> Vec<ModsDialogAction> {
    let repo = repo.into();
    match (release, location) {
        (Some(release), None) => {
            vec![ModsDialogAction::GithubFetchReleaseZipball { repo, release }]
        }
        (_, Some(url)) => vec![ModsDialogAction::GithubDownload { repo, url }],
        (None, None) => vec![ModsDialogAction::GithubDownload {
            repo: repo.clone(),
            url: format!(
                "{MODS_GITHUB_API_PREFIX}/repos/{repo}/zipball/{}",
                branch.into()
            ),
        }],
    }
}

pub fn handle_mod_plan(
    repo: impl Into<String>,
    content_length: i64,
    cancelled: bool,
) -> Vec<ModsDialogAction> {
    if cancelled {
        return Vec::new();
    }
    let repo = repo.into();
    let temp_file = format!("{}.zip", repo.replace('/', ""));
    vec![
        ModsDialogAction::SetImportProgressAvailable(content_length > 0),
        ModsDialogAction::HandleDownloadedMod {
            repo: repo.clone(),
            temp_file: temp_file.clone(),
        },
        ModsDialogAction::ImportDownloadedMod {
            temp_file: temp_file.clone(),
            repo,
        },
        ModsDialogAction::DeleteTempFile(temp_file),
        ModsDialogAction::Setup,
        ModsDialogAction::HideLoading,
    ]
}

pub fn import_dependencies_plan(
    mut dependencies: Vec<String>,
    listings: &[ModsListing],
) -> Vec<ModsDialogAction> {
    let mut out = Vec::new();
    for listing in listings {
        if dependencies.iter().any(|dep| dep == &listing.internal_name) {
            dependencies.retain(|dep| dep != &listing.internal_name);
            out.push(ModsDialogAction::ImportDependency {
                internal_name: listing.internal_name.clone(),
                repo: listing.repo.clone(),
                is_java: listing.has_java,
            });
        }
    }
    out.push(ModsDialogAction::DependenciesDone {
        remaining: dependencies,
    });
    out
}

pub fn mod_error_kind(error: ModsImportErrorInput) -> ModsImportErrorKind {
    match error {
        ModsImportErrorInput::NoSuchMethod => ModsImportErrorKind::FeatureUnsupported,
        ModsImportErrorInput::HttpStatus(status) => ModsImportErrorKind::ConnectFail(status),
        ModsImportErrorInput::Message(message) => {
            let lower = message.to_lowercase();
            if lower.contains("trust anchor") || lower.contains("ssl") || lower.contains("protocol")
            {
                ModsImportErrorKind::FeatureUnsupported
            } else if lower.contains("writable dex") {
                ModsImportErrorKind::WritableDex
            } else {
                ModsImportErrorKind::Exception(message)
            }
        }
    }
}

pub fn mod_error_actions(error: ModsImportErrorInput) -> Vec<ModsDialogAction> {
    let kind = mod_error_kind(error);
    let mut out = vec![ModsDialogAction::HideLoading];
    match kind {
        ModsImportErrorKind::FeatureUnsupported => {
            out.push(ModsDialogAction::ShowErrorMessage(
                "@feature.unsupported".to_string(),
            ));
        }
        ModsImportErrorKind::ConnectFail(status) => {
            out.push(ModsDialogAction::ShowErrorMessage(format!(
                "connectfail:{status}"
            )));
        }
        ModsImportErrorKind::WritableDex => out.push(ModsDialogAction::ShowException {
            title: Some("@error.moddex"),
            message: "writable dex".to_string(),
        }),
        ModsImportErrorKind::Exception(message) => {
            out.push(ModsDialogAction::ShowException {
                title: None,
                message,
            });
        }
    }
    out
}

pub fn mods_matches(query: &str, text: &str) -> bool {
    let query = query.trim();
    query.is_empty() || text.to_lowercase().contains(&query.to_lowercase())
}

fn strip_colors(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
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

pub fn version_is_at_least(current: f64, required: &str) -> bool {
    current >= parse_version_number(required)
}

pub fn parse_version_number(value: &str) -> f64 {
    value
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>()
        .parse::<f64>()
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loaded(name: &str) -> ModsLoadedMod {
        ModsLoadedMod {
            internal_name: name.to_string(),
            display_name: name.to_string(),
            author: Some("author".to_string()),
            version: Some("1.0".to_string()),
            description: Some("description".to_string()),
            repo: Some(format!("Anuken/{name}")),
            min_game_version: Some("157".to_string()),
            short_description: Some("short".to_string()),
            enabled: true,
            state: ModsLoadedModState::Loaded,
            missing_dependencies: Vec::new(),
            has_steam_id: false,
            has_icon_texture: false,
            hidden: false,
            is_java: false,
            content_count: 0,
            file_path: format!("mods/{name}"),
        }
    }

    fn listing(name: &str, stars: i32, updated: &str) -> ModsListing {
        ModsListing {
            name: name.to_string(),
            repo: format!("Anuken/{name}"),
            internal_name: name.to_string(),
            author: "author".to_string(),
            description: "description".to_string(),
            stars,
            last_updated: updated.to_string(),
            min_game_version: "157".to_string(),
            has_java: false,
            has_scripts: false,
            ios_compatible: true,
            legacy_compatible: false,
        }
    }

    #[test]
    fn installed_list_matches_java_visibility_state_and_buttons() {
        let mut disabled = loaded("disabled");
        disabled.enabled = false;
        let mut unmet = loaded("unmet");
        unmet.state = ModsLoadedModState::UnmetDependencies;
        unmet.missing_dependencies = vec!["lib".into()];
        let mut steam = loaded("steam");
        steam.has_steam_id = true;
        let context = ModsDialogContext {
            steam: true,
            graphics_width: 640.0,
            ..ModsDialogContext::default()
        };

        let cards = installed_cards(
            &[loaded("alpha"), disabled, unmet, steam],
            "",
            &context,
            "en",
        );
        assert_eq!(cards.len(), 4);
        assert!(cards[1].divider_before);
        assert!(cards[1].disabled_text_visible);
        assert_eq!(cards[2].state_text, Some("@mod.unmetdependencies"));
        assert!(cards[2].state_details.as_ref().unwrap().contains("lib"));
        assert_eq!(
            cards[3].delete_or_link_button.action,
            ModsDialogAction::ViewSteamListing("steam".into())
        );
        assert!(cards[0].publish_button.is_some());

        let filtered = installed_cards(&[loaded("alpha"), loaded("beta")], "alp", &context, "en");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].display_name, "alpha");
    }

    #[test]
    fn setup_model_handles_empty_search_and_platform_buttons() {
        let dialog = ModsDialog::new();
        let desktop = ModsDialogContext::default();
        let empty = dialog.setup_model(&[], true, "", &desktop, "en");
        assert_eq!(empty.title, "@mods");
        assert!(empty.reload_required_visible);
        assert_eq!(empty.empty_text, Some("@mods.none"));
        assert!(empty.open_folder_button.is_some());
        assert!(!empty.search_visible);

        let mobile_portrait = ModsDialogContext {
            mobile: true,
            portrait: true,
            ..ModsDialogContext::default()
        };
        let model = dialog.setup_model(&[loaded("alpha")], false, "", &mobile_portrait, "en");
        assert!(model.open_folder_button.is_none());
        assert!(model.search_visible);
    }

    #[test]
    fn show_mod_details_mirror_fields_repo_buttons_and_content_button() {
        let mut item = loaded("alpha");
        item.content_count = 2;
        item.state = ModsLoadedModState::Blacklisted;
        let model = show_mod_model(&item, &ModsDialogContext::default(), "en");
        assert_eq!(model.title, "alpha");
        assert!(model
            .buttons
            .iter()
            .any(|button| button.action == ModsDialogAction::OpenModFolder("mods/alpha".into())));
        assert!(model
            .buttons
            .iter()
            .any(|button| matches!(button.action, ModsDialogAction::ReinstallGithub { .. })));
        assert_eq!(model.fields[0].label, "@editor.name");
        assert_eq!(
            model.disabled_details,
            vec!["@mod.blacklisted.details".to_string()]
        );
        assert!(model.view_content_button.is_some());
    }

    #[test]
    fn browser_filters_sorts_and_marks_installed_like_java() {
        let context = ModsDialogContext {
            graphics_width: 960.0,
            ..ModsDialogContext::default()
        };
        let installed = vec![loaded("beta")];
        let listings = vec![
            listing("alpha", 5, "2024-01-01T00:00:00Z"),
            listing("beta", 50, "2025-01-01T00:00:00Z"),
            listing("gamma", 10, "2023-01-01T00:00:00Z"),
        ];
        let date = browser_model_from_listings(&listings, &installed, "", true, &context);
        assert_eq!(date.columns, 2);
        assert_eq!(date.cards[0].listing.name, "beta");
        assert!(date.cards[0].installed);
        assert_eq!(date.cards[0].icon_cache_key, "Anuken_beta");

        let stars = browser_model_from_listings(&listings, &installed, "", false, &context);
        assert_eq!(stars.cards[0].listing.name, "beta");
        assert_eq!(stars.sort_icon, "star");

        let filtered = browser_model_from_listings(&listings, &installed, "gam", true, &context);
        assert_eq!(filtered.cards.len(), 1);
        assert_eq!(filtered.cards[0].listing.name, "gamma");
    }

    #[test]
    fn browser_ios_and_version_compatibility_follow_upstream_branches() {
        let mut java_old = listing("java-old", 1, "2024-01-01T00:00:00Z");
        java_old.has_java = true;
        java_old.min_game_version = "100".into();
        let mut future = listing("future", 1, "2024-01-01T00:00:00Z");
        future.min_game_version = "999".into();
        let ios = ModsDialogContext {
            ios: true,
            ..ModsDialogContext::default()
        };
        assert!(!listing_visible(&java_old, "", &ios));

        let desktop = ModsDialogContext::default();
        assert_eq!(
            browser_compatibility(&java_old, &desktop),
            ModsBrowserCompatibility::IncompatibleJavaMod
        );
        assert_eq!(
            browser_compatibility(&future, &desktop),
            ModsBrowserCompatibility::RequiresVersion("999".into())
        );
    }

    #[test]
    fn github_import_release_branch_and_assets_are_planned_like_java() {
        assert_eq!(
            clean_github_repo_text(" https://github.com/Anuken/Example Mod "),
            "Anuken/ExampleMod"
        );
        assert_eq!(
            github_import_mod_plan("Anuken/java".into(), true, None),
            vec![
                ModsDialogAction::ShowLoading("@downloading"),
                ModsDialogAction::SetImportProgress(0.0),
                ModsDialogAction::SetImportCancelButton,
                ModsDialogAction::GithubFetchJavaRelease {
                    repo: "Anuken/java".into(),
                    release: None,
                },
            ]
        );
        assert_eq!(
            github_probe_result_plan("Anuken/kotlin", "main", "Kotlin", Some("1".into())),
            vec![ModsDialogAction::GithubFetchJavaRelease {
                repo: "Anuken/kotlin".into(),
                release: Some("1".into()),
            }]
        );
        assert_eq!(
            github_probe_result_plan("Anuken/json", "main", "HJSON", None),
            vec![ModsDialogAction::GithubFetchBranchZip {
                repo: "Anuken/json".into(),
                branch: "main".into(),
                release: None,
            }]
        );

        let assets = vec![
            ModsGithubAsset {
                name: "plain.jar".into(),
                browser_download_url: "plain-url".into(),
            },
            ModsGithubAsset {
                name: "dexed-release.jar".into(),
                browser_download_url: "dex-url".into(),
            },
        ];
        assert_eq!(
            java_release_assets_plan("Anuken/java", &assets).unwrap(),
            vec![ModsDialogAction::GithubDownload {
                repo: "Anuken/java".into(),
                url: "dex-url".into(),
            }]
        );
    }

    #[test]
    fn dependencies_handle_mod_and_errors_match_java_flow() {
        let listings = vec![listing("alpha-lib", 1, "2024-01-01T00:00:00Z")];
        let actions =
            import_dependencies_plan(vec!["alpha-lib".into(), "missing".into()], &listings);
        assert_eq!(
            actions,
            vec![
                ModsDialogAction::ImportDependency {
                    internal_name: "alpha-lib".into(),
                    repo: "Anuken/alpha-lib".into(),
                    is_java: false,
                },
                ModsDialogAction::DependenciesDone {
                    remaining: vec!["missing".into()],
                },
            ]
        );

        assert_eq!(
            handle_mod_plan("Anuken/example", 100, false),
            vec![
                ModsDialogAction::SetImportProgressAvailable(true),
                ModsDialogAction::HandleDownloadedMod {
                    repo: "Anuken/example".into(),
                    temp_file: "Anukenexample.zip".into(),
                },
                ModsDialogAction::ImportDownloadedMod {
                    temp_file: "Anukenexample.zip".into(),
                    repo: "Anuken/example".into(),
                },
                ModsDialogAction::DeleteTempFile("Anukenexample.zip".into()),
                ModsDialogAction::Setup,
                ModsDialogAction::HideLoading,
            ]
        );
        assert!(handle_mod_plan("Anuken/example", 100, true).is_empty());

        assert_eq!(
            mod_error_kind(ModsImportErrorInput::Message("SSL trust anchor".into())),
            ModsImportErrorKind::FeatureUnsupported
        );
        assert_eq!(
            mod_error_kind(ModsImportErrorInput::Message("writable dex".into())),
            ModsImportErrorKind::WritableDex
        );
    }

    #[test]
    fn releases_model_formats_dates_and_empty_lists_show_info() {
        assert!(release_dialog_model("Anuken/mod", &[]).is_none());
        let releases = vec![ModsRelease {
            name: "v1".into(),
            published_at: "2026-06-10T12:00:00Z".into(),
            html_url: "https://github.com/Anuken/mod/releases/tag/v1".into(),
            api_url: "https://api.github.com/repos/Anuken/mod/releases/123".into(),
        }];
        let model = release_dialog_model("Anuken/mod", &releases).unwrap();
        assert_eq!(model.entries[0].date, "2026/06/10");
        assert!(model.entries[0].latest);
        assert_eq!(model.entries[0].release_id, "123");
    }
}
