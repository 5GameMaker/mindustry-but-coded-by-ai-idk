//! Core database dialog model mirroring upstream `mindustry.ui.dialogs.DatabaseDialog`.

use crate::mindustry::ctype::{ContentId, ContentType, UnlockableContentBase};

pub const DATABASE_DIALOG_TITLE: &str = "@database";
pub const DATABASE_SEARCH_ICON: &str = "zoom";
pub const DATABASE_SEARCH_MESSAGE: &str = "@players.search";
pub const DATABASE_SEARCH_PAD_RIGHT: f32 = 8.0;
pub const DATABASE_SEARCH_PAD_BOTTOM: f32 = 4.0;
pub const DATABASE_ALL_MARGIN: f32 = 20.0;
pub const DATABASE_ALL_MARGIN_TOP: f32 = 0.0;
pub const DATABASE_ALL_MARGIN_RIGHT: f32 = 30.0;
pub const DATABASE_ALL_TAB_NAME: &str = "sun";
pub const DATABASE_ALL_TAB_TOOLTIP: &str = "@all";
pub const DATABASE_ALL_TAB_ICON: &str = "eyeSmall";
pub const DATABASE_PLANET_FALLBACK_ICON: &str = "commandRally";
pub const DATABASE_TAB_STYLE: &str = "Styles.clearNoneTogglei";
pub const DATABASE_TAB_SIZE: f32 = 50.0;
pub const DATABASE_TAB_ICON_SIZE: f32 = 32.0;
pub const DATABASE_TAB_COLUMNS: usize = 10;
pub const DATABASE_CATEGORY_PREFIX: &str = "@database-category.";
pub const DATABASE_CATEGORY_COLOR: &str = "Pal.accent";
pub const DATABASE_CATEGORY_DIVIDER_PAD: f32 = 5.0;
pub const DATABASE_CATEGORY_DIVIDER_HEIGHT: f32 = 3.0;
pub const DATABASE_TAG_PREFIX: &str = "@database-tag.";
pub const DATABASE_TAG_COLOR: &str = "Pal.gray";
pub const DATABASE_TAG_PAD: (f32, f32, f32, f32) = (4.0, 8.0, 4.0, 8.0);
pub const DATABASE_TAG_DIVIDER_PAD: f32 = 5.0;
pub const DATABASE_TAG_DIVIDER_HEIGHT: f32 = 3.0;
pub const DATABASE_DEFAULT_TAG: &str = "default";
pub const DATABASE_CONTENT_CELL_SIZE: f32 = 32.0;
pub const DATABASE_CONTENT_CELL_PAD: f32 = 3.0;
pub const DATABASE_CONTENT_COLUMN_WIDTH: f32 = 44.0;
pub const DATABASE_CONTENT_COLUMN_WIDTH_PAD: f32 = 30.0;
pub const DATABASE_MIN_COLUMNS: usize = 1;
pub const DATABASE_MAX_COLUMNS: usize = 22;
pub const DATABASE_LOCK_ICON: &str = "lock";
pub const DATABASE_BANNED_ICON: &str = "cancel";
pub const DATABASE_BANNED_COLOR: &str = "Color.scarlet";
pub const DATABASE_PATCHED_ICON: &str = "fileSmall";
pub const DATABASE_PATCHED_COLOR: &str = "Color.white.a(0.5)";
pub const DATABASE_UNLOCKED_DESKTOP_COLOR: &str = "Color.lightGray";
pub const DATABASE_UNLOCKED_MOBILE_COLOR: &str = "Color.white";
pub const DATABASE_LOCKED_COLOR: &str = "Pal.gray";
pub const DATABASE_TOOLTIP_BACKGROUND: &str = "Tex.button";
pub const DATABASE_NONE_FOUND: &str = "@none.found";
pub const DATABASE_COPIED_MESSAGE: &str = "@copied";

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseContent {
    pub content_type: ContentType,
    pub id: ContentId,
    pub name: String,
    pub localized_name: String,
    pub icon: String,
    pub hidden: bool,
    pub hide_database: bool,
    pub all_database_tabs: bool,
    pub database_tabs: Vec<String>,
    pub database_category: String,
    pub database_tag: String,
    pub unlocked: bool,
    pub banned: bool,
    pub patched: bool,
    pub unicode: Option<char>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseTab {
    pub name: String,
    pub localized_name: String,
    pub icon: DatabaseTabIcon,
    pub icon_color_rgba: u32,
    pub sort_key: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseTabIcon {
    All,
    Planet { icon: String },
    Content { icon: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseDialogContext {
    pub state_is_campaign: bool,
    pub state_is_game: bool,
    pub state_is_menu: bool,
    pub campaign_planet: Option<String>,
    pub rules_planet: Option<String>,
    pub console: bool,
    pub mobile: bool,
    pub graphics_width: f32,
    pub scl: f32,
}

impl Default for DatabaseDialogContext {
    fn default() -> Self {
        Self {
            state_is_campaign: false,
            state_is_game: false,
            state_is_menu: true,
            campaign_planet: None,
            rules_planet: None,
            console: false,
            mobile: false,
            graphics_width: 960.0,
            scl: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseDialogModel {
    pub title: &'static str,
    pub should_pause: bool,
    pub close_button_added: bool,
    pub all_margin: f32,
    pub all_margin_top: f32,
    pub all_margin_right: f32,
    pub search: DatabaseSearchModel,
    pub selected_tab: String,
    pub tab_buttons: Vec<DatabaseTabButton>,
    pub categories: Vec<DatabaseCategorySection>,
    pub none_found: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseSearchModel {
    pub icon: &'static str,
    pub message_text: &'static str,
    pub text: String,
    pub pad_right: f32,
    pub pad_bottom: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseTabButton {
    pub name: String,
    pub icon: DatabaseTabIcon,
    pub icon_color_rgba: u32,
    pub style: &'static str,
    pub icon_size: f32,
    pub size: f32,
    pub checked: bool,
    pub tooltip: String,
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseCategorySection {
    pub name: String,
    pub title: String,
    pub color: &'static str,
    pub divider: DatabaseDivider,
    pub tags: Vec<DatabaseTagSection>,
    pub pad_bottom: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseDivider {
    pub color: &'static str,
    pub pad: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseTagSection {
    pub name: String,
    pub title: Option<String>,
    pub color: &'static str,
    pub divider: Option<DatabaseDivider>,
    pub pad: Option<(f32, f32, f32, f32)>,
    pub columns: usize,
    pub cards: Vec<DatabaseContentCard>,
    pub empty_fill_cells: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseContentCard {
    pub content_type: ContentType,
    pub id: ContentId,
    pub name: String,
    pub localized_name: String,
    pub icon: String,
    pub image_color: &'static str,
    pub badge: Option<DatabaseContentBadge>,
    pub tooltip: Option<String>,
    pub tooltip_background: &'static str,
    pub unlocked: bool,
    pub size: f32,
    pub pad: f32,
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseContentBadge {
    Banned,
    Patched,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseDialogAction {
    SetTab {
        tab: String,
    },
    Rebuild,
    ShowContent {
        content_type: ContentType,
        id: ContentId,
        name: String,
    },
    SetClipboardText {
        text: String,
    },
    ShowInfoFade {
        message: &'static str,
    },
    HideDialog,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseDialog {
    selected_tab: String,
    all_tabs: Option<Vec<DatabaseTab>>,
}

impl Default for DatabaseDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseContent {
    pub fn from_base(base: &UnlockableContentBase, hidden: bool) -> Self {
        let content_type = base.mappable.base.content_type;
        let name = base.mappable.name.clone();
        Self {
            content_type,
            id: base.mappable.base.id,
            localized_name: base.localized_name.clone().unwrap_or_else(|| name.clone()),
            icon: base
                .icon_candidates(None)
                .ui_candidates
                .first()
                .cloned()
                .unwrap_or_else(|| name.clone()),
            name,
            hidden,
            hide_database: base.hide_database,
            all_database_tabs: base.all_database_tabs,
            database_tabs: base.database_tabs.clone(),
            database_category: base.database_category_key().to_string(),
            database_tag: base.database_tag_key().to_string(),
            unlocked: base.unlocked(),
            banned: false,
            patched: false,
            unicode: None,
        }
    }

    pub fn visible_on_tab(&self, tab: &str) -> bool {
        tab == DATABASE_ALL_TAB_NAME
            || self.all_database_tabs
            || self.database_tabs.iter().any(|entry| entry == tab)
    }
}

impl DatabaseTab {
    pub fn all() -> Self {
        Self {
            name: DATABASE_ALL_TAB_NAME.to_string(),
            localized_name: "Sun".to_string(),
            icon: DatabaseTabIcon::All,
            icon_color_rgba: 0xffffffff,
            sort_key: i32::MIN,
        }
    }

    pub fn planet(
        name: impl Into<String>,
        localized_name: impl Into<String>,
        icon: impl Into<String>,
        icon_color_rgba: u32,
        sort_key: i32,
    ) -> Self {
        Self {
            name: name.into(),
            localized_name: localized_name.into(),
            icon: DatabaseTabIcon::Planet { icon: icon.into() },
            icon_color_rgba,
            sort_key,
        }
    }

    pub fn content(
        name: impl Into<String>,
        localized_name: impl Into<String>,
        icon: impl Into<String>,
        sort_key: i32,
    ) -> Self {
        Self {
            name: name.into(),
            localized_name: localized_name.into(),
            icon: DatabaseTabIcon::Content { icon: icon.into() },
            icon_color_rgba: 0xffffffff,
            sort_key,
        }
    }
}

impl DatabaseDialog {
    pub fn new() -> Self {
        Self {
            selected_tab: DATABASE_ALL_TAB_NAME.to_string(),
            all_tabs: None,
        }
    }

    pub fn show(
        &mut self,
        contents: &[DatabaseContent],
        known_tabs: &[DatabaseTab],
        context: &DatabaseDialogContext,
        search_text: &str,
    ) -> DatabaseDialogModel {
        self.check_tab_list(contents, known_tabs);
        if context.state_is_campaign {
            if let Some(planet) = context
                .campaign_planet
                .as_deref()
                .filter(|planet| self.has_tab(planet))
            {
                self.selected_tab = planet.to_string();
            }
        } else if context.state_is_game {
            if let Some(planet) = context
                .rules_planet
                .as_deref()
                .filter(|planet| self.has_tab(planet))
            {
                self.selected_tab = planet.to_string();
            }
        }
        self.rebuild(contents, context, search_text)
    }

    pub fn rebuild(
        &mut self,
        contents: &[DatabaseContent],
        context: &DatabaseDialogContext,
        search_text: &str,
    ) -> DatabaseDialogModel {
        let tabs = self.all_tabs.clone().unwrap_or_else(|| {
            let tabs = check_tab_list(contents, &[]);
            self.all_tabs = Some(tabs.clone());
            tabs
        });
        let lower_text = search_text.to_lowercase();
        let columns = database_columns(context.graphics_width, context.scl);
        let mut categories = Vec::new();

        for mut category in sort_contents(contents) {
            let mut tags = Vec::new();

            for mut tag in category.tags.drain(..) {
                tag.contents.retain(|content| {
                    !content.hidden
                        && !content.hide_database
                        && content.visible_on_tab(&self.selected_tab)
                        && (lower_text.is_empty()
                            || content.localized_name.to_lowercase().contains(&lower_text))
                });
                if tag.contents.is_empty() {
                    continue;
                }

                if context.state_is_game {
                    tag.contents
                        .sort_by_key(|content| (content.banned, content.id, content.name.clone()));
                }

                let count = tag.contents.len();
                let cards = tag
                    .contents
                    .into_iter()
                    .enumerate()
                    .map(|(index, content)| content_card(content, context, index, columns))
                    .collect::<Vec<_>>();
                let tag_title = (tag.name != DATABASE_DEFAULT_TAG)
                    .then(|| format!("{DATABASE_TAG_PREFIX}{}", tag.name));
                tags.push(DatabaseTagSection {
                    name: tag.name,
                    title: tag_title.clone(),
                    color: DATABASE_TAG_COLOR,
                    divider: tag_title.as_ref().map(|_| DatabaseDivider {
                        color: DATABASE_TAG_COLOR,
                        pad: DATABASE_TAG_DIVIDER_PAD,
                        height: DATABASE_TAG_DIVIDER_HEIGHT,
                    }),
                    pad: tag_title.as_ref().map(|_| DATABASE_TAG_PAD),
                    columns,
                    cards,
                    empty_fill_cells: columns.saturating_sub(count),
                });
            }

            if tags.is_empty() {
                continue;
            }

            categories.push(DatabaseCategorySection {
                title: format!("{DATABASE_CATEGORY_PREFIX}{}", category.name),
                name: category.name,
                color: DATABASE_CATEGORY_COLOR,
                divider: DatabaseDivider {
                    color: DATABASE_CATEGORY_COLOR,
                    pad: DATABASE_CATEGORY_DIVIDER_PAD,
                    height: DATABASE_CATEGORY_DIVIDER_HEIGHT,
                },
                tags,
                pad_bottom: 10.0,
            });
        }

        DatabaseDialogModel {
            title: DATABASE_DIALOG_TITLE,
            should_pause: true,
            close_button_added: true,
            all_margin: DATABASE_ALL_MARGIN,
            all_margin_top: DATABASE_ALL_MARGIN_TOP,
            all_margin_right: DATABASE_ALL_MARGIN_RIGHT,
            search: DatabaseSearchModel {
                icon: DATABASE_SEARCH_ICON,
                message_text: DATABASE_SEARCH_MESSAGE,
                text: search_text.to_string(),
                pad_right: DATABASE_SEARCH_PAD_RIGHT,
                pad_bottom: DATABASE_SEARCH_PAD_BOTTOM,
            },
            selected_tab: self.selected_tab.clone(),
            tab_buttons: tabs
                .iter()
                .enumerate()
                .map(|(index, tab)| DatabaseTabButton {
                    name: tab.name.clone(),
                    icon: normalized_tab_icon(tab),
                    icon_color_rgba: tab.icon_color_rgba,
                    style: DATABASE_TAB_STYLE,
                    icon_size: DATABASE_TAB_ICON_SIZE,
                    size: DATABASE_TAB_SIZE,
                    checked: tab.name == self.selected_tab,
                    tooltip: if tab.name == DATABASE_ALL_TAB_NAME {
                        DATABASE_ALL_TAB_TOOLTIP.to_string()
                    } else {
                        tab.localized_name.clone()
                    },
                    row: index / DATABASE_TAB_COLUMNS,
                    column: index % DATABASE_TAB_COLUMNS,
                })
                .collect(),
            none_found: categories.is_empty().then_some(DATABASE_NONE_FOUND),
            categories,
        }
    }

    pub fn select_tab_plan(&mut self, tab: &str) -> Vec<DatabaseDialogAction> {
        self.selected_tab = tab.to_string();
        vec![
            DatabaseDialogAction::SetTab {
                tab: tab.to_string(),
            },
            DatabaseDialogAction::Rebuild,
        ]
    }

    pub fn content_click_plan(
        content: &DatabaseContent,
        context: &DatabaseDialogContext,
        shift_down: bool,
    ) -> Vec<DatabaseDialogAction> {
        if !database_content_unlocked(content, context) {
            return Vec::new();
        }

        if shift_down {
            if let Some(unicode) = content.unicode.filter(|unicode| *unicode != '\0') {
                return vec![
                    DatabaseDialogAction::SetClipboardText {
                        text: unicode.to_string(),
                    },
                    DatabaseDialogAction::ShowInfoFade {
                        message: DATABASE_COPIED_MESSAGE,
                    },
                ];
            }
        }

        vec![DatabaseDialogAction::ShowContent {
            content_type: content.content_type,
            id: content.id,
            name: content.name.clone(),
        }]
    }

    fn check_tab_list(&mut self, contents: &[DatabaseContent], known_tabs: &[DatabaseTab]) {
        if self.all_tabs.is_none() {
            self.all_tabs = Some(check_tab_list(contents, known_tabs));
        }
    }

    fn has_tab(&self, tab: &str) -> bool {
        self.all_tabs
            .as_ref()
            .is_some_and(|tabs| tabs.iter().any(|entry| entry.name == tab))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SortedCategory {
    name: String,
    tags: Vec<SortedTag>,
}

#[derive(Debug, Clone, PartialEq)]
struct SortedTag {
    name: String,
    contents: Vec<DatabaseContent>,
}

fn check_tab_list(contents: &[DatabaseContent], known_tabs: &[DatabaseTab]) -> Vec<DatabaseTab> {
    let mut tabs = Vec::new();
    for tab in known_tabs {
        if tab.name != DATABASE_ALL_TAB_NAME
            && !tabs
                .iter()
                .any(|entry: &DatabaseTab| entry.name == tab.name)
        {
            tabs.push(tab.clone());
        }
    }
    for content in contents {
        for tab_name in &content.database_tabs {
            if tab_name != DATABASE_ALL_TAB_NAME
                && !tabs.iter().any(|entry| entry.name == *tab_name)
            {
                tabs.push(DatabaseTab::content(
                    tab_name.clone(),
                    tab_name.clone(),
                    tab_name.clone(),
                    0,
                ));
            }
        }
    }
    tabs.sort_by(|left, right| {
        left.sort_key
            .cmp(&right.sort_key)
            .then_with(|| left.name.cmp(&right.name))
    });
    tabs.insert(0, DatabaseTab::all());
    tabs
}

fn sort_contents(contents: &[DatabaseContent]) -> Vec<SortedCategory> {
    let mut categories = Vec::<SortedCategory>::new();

    for content in contents {
        let category_index = match categories
            .iter()
            .position(|cat| cat.name == content.database_category)
        {
            Some(index) => index,
            None => {
                categories.push(SortedCategory {
                    name: content.database_category.clone(),
                    tags: Vec::new(),
                });
                categories.len() - 1
            }
        };
        let category = &mut categories[category_index];
        let tag_index = match category
            .tags
            .iter()
            .position(|tag| tag.name == content.database_tag)
        {
            Some(index) => index,
            None => {
                category.tags.push(SortedTag {
                    name: content.database_tag.clone(),
                    contents: Vec::new(),
                });
                category.tags.len() - 1
            }
        };
        category.tags[tag_index].contents.push(content.clone());
    }

    categories
}

fn content_card(
    content: DatabaseContent,
    context: &DatabaseDialogContext,
    index: usize,
    columns: usize,
) -> DatabaseContentCard {
    let unlocked = database_content_unlocked(&content, context);
    let badge = if context.state_is_game && content.banned {
        Some(DatabaseContentBadge::Banned)
    } else if context.state_is_game && content.patched {
        Some(DatabaseContentBadge::Patched)
    } else {
        None
    };
    let tooltip = unlocked.then(|| {
        if context.console {
            format!("{}\n[gray]{}", content.localized_name, content.name)
        } else {
            content.localized_name.clone()
        }
    });
    DatabaseContentCard {
        content_type: content.content_type,
        id: content.id,
        name: content.name,
        localized_name: content.localized_name,
        icon: if unlocked {
            content.icon
        } else {
            DATABASE_LOCK_ICON.to_string()
        },
        image_color: if unlocked {
            if context.mobile {
                DATABASE_UNLOCKED_MOBILE_COLOR
            } else {
                DATABASE_UNLOCKED_DESKTOP_COLOR
            }
        } else {
            DATABASE_LOCKED_COLOR
        },
        badge,
        tooltip,
        tooltip_background: DATABASE_TOOLTIP_BACKGROUND,
        unlocked,
        size: DATABASE_CONTENT_CELL_SIZE,
        pad: DATABASE_CONTENT_CELL_PAD,
        row: index / columns,
        column: index % columns,
    }
}

fn normalized_tab_icon(tab: &DatabaseTab) -> DatabaseTabIcon {
    match &tab.icon {
        DatabaseTabIcon::All => DatabaseTabIcon::All,
        DatabaseTabIcon::Planet { icon } => DatabaseTabIcon::Planet {
            icon: if icon.is_empty() {
                DATABASE_PLANET_FALLBACK_ICON.to_string()
            } else {
                icon.clone()
            },
        },
        DatabaseTabIcon::Content { icon } => DatabaseTabIcon::Content { icon: icon.clone() },
    }
}

pub fn database_columns(graphics_width: f32, scl: f32) -> usize {
    let scaled_pad = DATABASE_CONTENT_COLUMN_WIDTH_PAD * scl;
    let scaled_width = DATABASE_CONTENT_COLUMN_WIDTH * scl;
    let raw = ((graphics_width - scaled_pad) / scaled_width) as i32;
    raw.clamp(DATABASE_MIN_COLUMNS as i32, DATABASE_MAX_COLUMNS as i32) as usize
}

pub fn database_content_unlocked(
    content: &DatabaseContent,
    context: &DatabaseDialogContext,
) -> bool {
    (!context.state_is_campaign && !context.state_is_menu) || content.unlocked
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content(
        content_type: ContentType,
        id: ContentId,
        name: &str,
        localized_name: &str,
        category: &str,
        tag: &str,
        tabs: &[&str],
    ) -> DatabaseContent {
        DatabaseContent {
            content_type,
            id,
            name: name.into(),
            localized_name: localized_name.into(),
            icon: format!("{}-{name}-ui", content_type.wire_name()),
            hidden: false,
            hide_database: false,
            all_database_tabs: false,
            database_tabs: tabs.iter().map(|tab| (*tab).to_string()).collect(),
            database_category: category.into(),
            database_tag: tag.into(),
            unlocked: true,
            banned: false,
            patched: false,
            unicode: None,
        }
    }

    #[test]
    fn show_builds_search_tabs_and_chooses_campaign_or_rules_planet_like_java() {
        let contents = vec![content(
            ContentType::Item,
            0,
            "copper",
            "Copper",
            "item",
            "default",
            &["serpulo"],
        )];
        let tabs = vec![
            DatabaseTab::planet("erekir", "Erekir", "erekir", 0xff9266ff, 1),
            DatabaseTab::planet("serpulo", "Serpulo", "database", 0x7d4dffff, 5),
        ];
        let mut dialog = DatabaseDialog::new();
        let model = dialog.show(
            &contents,
            &tabs,
            &DatabaseDialogContext {
                state_is_campaign: true,
                campaign_planet: Some("serpulo".into()),
                state_is_menu: false,
                graphics_width: 960.0,
                ..Default::default()
            },
            "cop",
        );

        assert_eq!(model.title, "@database");
        assert!(model.should_pause);
        assert!(model.close_button_added);
        assert_eq!(model.search.icon, "zoom");
        assert_eq!(model.search.message_text, "@players.search");
        assert_eq!(model.search.text, "cop");
        assert_eq!(model.selected_tab, "serpulo");
        assert_eq!(
            model
                .tab_buttons
                .iter()
                .map(|button| (
                    button.name.as_str(),
                    button.tooltip.as_str(),
                    button.checked
                ))
                .collect::<Vec<_>>(),
            vec![
                ("sun", "@all", false),
                ("erekir", "Erekir", false),
                ("serpulo", "Serpulo", true)
            ]
        );
        assert_eq!(model.tab_buttons[0].icon, DatabaseTabIcon::All);
        assert_eq!(
            model.tab_buttons[1].icon,
            DatabaseTabIcon::Planet {
                icon: "erekir".into()
            }
        );

        let mut rules_dialog = DatabaseDialog::new();
        let rules_model = rules_dialog.show(
            &contents,
            &tabs,
            &DatabaseDialogContext {
                state_is_game: true,
                state_is_menu: false,
                rules_planet: Some("erekir".into()),
                ..Default::default()
            },
            "",
        );
        assert_eq!(rules_model.selected_tab, "erekir");
    }

    #[test]
    fn rebuild_filters_groups_tags_cards_and_game_badges_like_java() {
        let mut copper = content(
            ContentType::Item,
            0,
            "copper",
            "Copper",
            "item",
            "default",
            &["serpulo"],
        );
        copper.unlocked = false;
        copper.unicode = Some('\u{f838}');
        let mut lead = content(
            ContentType::Item,
            1,
            "lead",
            "Lead",
            "item",
            "default",
            &["serpulo"],
        );
        lead.hide_database = true;
        let mut router = content(
            ContentType::Block,
            2,
            "router",
            "Router",
            "block",
            "distribution",
            &["serpulo"],
        );
        router.banned = true;
        let mut junction = content(
            ContentType::Block,
            1,
            "junction",
            "Junction",
            "block",
            "distribution",
            &["serpulo"],
        );
        junction.patched = true;
        let mut flare = content(
            ContentType::Unit,
            5,
            "flare",
            "Flare",
            "unit",
            "unit-air",
            &["erekir"],
        );
        flare.all_database_tabs = true;
        let hidden = DatabaseContent {
            hidden: true,
            ..content(
                ContentType::Item,
                3,
                "hidden",
                "Hidden",
                "item",
                "default",
                &["serpulo"],
            )
        };
        let contents = vec![copper, lead, router, junction, flare, hidden];
        let mut dialog = DatabaseDialog::new();
        let _model = dialog.show(
            &contents,
            &[DatabaseTab::planet(
                "serpulo", "Serpulo", "database", 0x7d4dffff, 0,
            )],
            &DatabaseDialogContext {
                state_is_game: true,
                state_is_menu: false,
                console: true,
                graphics_width: 118.0,
                ..Default::default()
            },
            "",
        );
        dialog.select_tab_plan("serpulo");
        let model = dialog.rebuild(
            &contents,
            &DatabaseDialogContext {
                state_is_game: true,
                state_is_menu: false,
                console: true,
                graphics_width: 118.0,
                ..Default::default()
            },
            "",
        );

        assert_eq!(database_columns(118.0, 1.0), 2);
        assert_eq!(
            model
                .categories
                .iter()
                .map(|category| category.title.as_str())
                .collect::<Vec<_>>(),
            vec![
                "@database-category.item",
                "@database-category.block",
                "@database-category.unit"
            ]
        );
        let item_cards = &model.categories[0].tags[0].cards;
        assert_eq!(item_cards.len(), 1);
        assert_eq!(item_cards[0].name, "copper");
        assert_eq!(item_cards[0].icon, "item-copper-ui");
        assert_eq!(item_cards[0].image_color, "Color.lightGray");
        assert_eq!(
            item_cards[0].tooltip.as_deref(),
            Some("Copper\n[gray]copper")
        );
        assert_eq!(model.categories[0].tags[0].empty_fill_cells, 1);

        let block_tag = &model.categories[1].tags[0];
        assert_eq!(
            block_tag.title.as_deref(),
            Some("@database-tag.distribution")
        );
        assert_eq!(block_tag.columns, 2);
        assert_eq!(
            block_tag
                .cards
                .iter()
                .map(|card| (card.name.as_str(), card.badge))
                .collect::<Vec<_>>(),
            vec![
                ("junction", Some(DatabaseContentBadge::Patched)),
                ("router", Some(DatabaseContentBadge::Banned))
            ],
            "game rebuild sorts unbanned content before banned content, then by id"
        );
        assert_eq!(
            block_tag.cards[0].tooltip.as_deref(),
            Some("Junction\n[gray]junction")
        );
        assert_eq!(block_tag.cards[1].row, 0);
        assert_eq!(block_tag.cards[1].column, 1);

        let unit_tag = &model.categories[2].tags[0];
        assert_eq!(unit_tag.title.as_deref(), Some("@database-tag.unit-air"));
        assert_eq!(unit_tag.cards[0].name, "flare");
    }

    #[test]
    fn search_and_tab_filtering_report_none_found() {
        let contents = vec![content(
            ContentType::Item,
            0,
            "copper",
            "Copper",
            "item",
            "default",
            &["serpulo"],
        )];
        let mut dialog = DatabaseDialog::new();
        let model = dialog.show(
            &contents,
            &[DatabaseTab::planet(
                "serpulo", "Serpulo", "database", 0x7d4dffff, 0,
            )],
            &DatabaseDialogContext::default(),
            "lead",
        );

        assert_eq!(model.none_found, Some("@none.found"));
        assert!(model.categories.is_empty());
        assert_eq!(
            dialog.select_tab_plan("serpulo"),
            vec![
                DatabaseDialogAction::SetTab {
                    tab: "serpulo".into()
                },
                DatabaseDialogAction::Rebuild
            ]
        );
    }

    #[test]
    fn content_click_copies_unicode_with_shift_or_opens_content_when_unlocked() {
        let mut copper = content(
            ContentType::Item,
            0,
            "copper",
            "Copper",
            "item",
            "default",
            &["serpulo"],
        );
        copper.unicode = Some('');
        let context = DatabaseDialogContext {
            state_is_game: true,
            state_is_menu: false,
            ..Default::default()
        };

        assert_eq!(
            DatabaseDialog::content_click_plan(&copper, &context, true),
            vec![
                DatabaseDialogAction::SetClipboardText { text: "".into() },
                DatabaseDialogAction::ShowInfoFade { message: "@copied" },
            ]
        );
        copper.unicode = None;
        assert_eq!(
            DatabaseDialog::content_click_plan(&copper, &context, true),
            vec![DatabaseDialogAction::ShowContent {
                content_type: ContentType::Item,
                id: 0,
                name: "copper".into(),
            }]
        );

        copper.unlocked = false;
        let menu_campaign = DatabaseDialogContext {
            state_is_campaign: true,
            state_is_menu: true,
            ..Default::default()
        };
        assert_eq!(
            DatabaseDialog::content_click_plan(&copper, &menu_campaign, false),
            Vec::<DatabaseDialogAction>::new()
        );
    }
}
