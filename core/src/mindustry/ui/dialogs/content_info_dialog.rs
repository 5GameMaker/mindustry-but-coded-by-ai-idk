//! Content info dialog model mirroring upstream `mindustry.ui.dialogs.ContentInfoDialog`.

use crate::mindustry::{
    ctype::{ContentType, UnlockableContentBase},
    world::meta::{Stat, StatValue, Stats},
};

pub const CONTENT_INFO_TITLE: &str = "@info.title";
pub const CONTENT_INFO_TABLE_MARGIN: f32 = 10.0;
pub const CONTENT_INFO_TABLE_MARGIN_RIGHT: f32 = 30.0;
pub const CONTENT_INFO_ICON_SIZE: f32 = 48.0;
pub const CONTENT_INFO_ICON_SCALING: &str = "Scaling.fit";
pub const CONTENT_INFO_HEADER_NAME_PAD_LEFT: f32 = 5.0;
pub const CONTENT_INFO_HEADER_ACCENT_PREFIX: &str = "[accent]";
pub const CONTENT_INFO_CONSOLE_NAME_PREFIX: &str = "\n[gray]";
pub const CONTENT_INFO_PATCHED_ICON: &str = "info";
pub const CONTENT_INFO_PATCHED_TEXT: &str = "@database.patched";
pub const CONTENT_INFO_PATCHED_COLOR: &str = "Pal.lightishGray";
pub const CONTENT_INFO_PATCHED_PAD: f32 = 4.0;
pub const CONTENT_INFO_PATCHED_TEXT_PAD_LEFT: f32 = 4.0;
pub const CONTENT_INFO_CATEGORY_PURPOSE: &str = "@category.purpose";
pub const CONTENT_INFO_CATEGORY_GENERAL: &str = "@category.general";
pub const CONTENT_INFO_CATEGORY_COLOR: &str = "Pal.accent";
pub const CONTENT_INFO_DESCRIPTION_COLOR_PREFIX: &str = "[lightgray]";
pub const CONTENT_INFO_DESCRIPTION_WIDTH: f32 = 500.0;
pub const CONTENT_INFO_DESCRIPTION_PAD_LEFT_WITH_STATS: f32 = 10.0;
pub const CONTENT_INFO_DESCRIPTION_PAD_LEFT_NO_STATS: f32 = 0.0;
pub const CONTENT_INFO_DESCRIPTION_PAD_TOP_WITH_STATS: f32 = 0.0;
pub const CONTENT_INFO_DESCRIPTION_PAD_TOP_NO_STATS: f32 = 10.0;
pub const CONTENT_INFO_STAT_PAD_LEFT: f32 = 10.0;
pub const CONTENT_INFO_STAT_LABEL_PREFIX: &str = "[lightgray]";
pub const CONTENT_INFO_STAT_LABEL_SUFFIX: &str = ":[] ";
pub const CONTENT_INFO_STAT_VALUE_SPACER: f32 = 10.0;
pub const CONTENT_INFO_DETAILS_COLOR_PREFIX: &str = "[gray]";
pub const CONTENT_INFO_DETAILS_PAD: f32 = 6.0;
pub const CONTENT_INFO_DETAILS_PAD_TOP: f32 = 20.0;
pub const CONTENT_INFO_DETAILS_WIDTH: f32 = 400.0;
pub const CONTENT_INFO_LOCK_ICON: &str = "Iconc.lock";
pub const CONTENT_INFO_UNLOCK_IN_CAMPAIGN: &str = "@unlock.incampaign";
pub const CONTENT_INFO_CREDIT_KEY: &str = "content.createdby";
pub const CONTENT_INFO_CREDIT_COLOR: &str = "Color.gray";
pub const CONTENT_INFO_CREDIT_PAD_TOP: f32 = 40.0;
pub const CONTENT_INFO_VIEW_FIELDS_TEXT: &str = "@viewfields";
pub const CONTENT_INFO_VIEW_FIELDS_ICON: &str = "link";
pub const CONTENT_INFO_VIEW_FIELDS_STYLE: &str = "Styles.grayt";
pub const CONTENT_INFO_VIEW_FIELDS_MARGIN: f32 = 8.0;
pub const CONTENT_INFO_VIEW_FIELDS_PAD: f32 = 4.0;
pub const CONTENT_INFO_VIEW_FIELDS_PAD_TOP: f32 = 16.0;
pub const CONTENT_INFO_VIEW_FIELDS_SIZE: (f32, f32) = (300.0, 50.0);
pub const CONTENT_INFO_FIELDS_URI_PREFIX: &str =
    "https://mindustrygame.github.io/wiki/Modding%20Classes/";
pub const CONTENT_INFO_RESHOW_FADE_DURATION: f32 = 0.0;

#[derive(Debug, Clone, PartialEq)]
pub struct ContentInfoContent {
    pub base: UnlockableContentBase,
    pub stats: Stats,
    pub mod_display_name: Option<String>,
    pub class_simple_name: String,
    pub display_extra: Vec<ContentInfoExtraBlock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentInfoDialogContext {
    pub console: bool,
    pub state_is_game: bool,
    pub content_patched: bool,
    pub dialog_already_shown: bool,
}

impl Default for ContentInfoDialogContext {
    fn default() -> Self {
        Self {
            console: false,
            state_is_game: false,
            content_patched: false,
            dialog_already_shown: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentInfoDialogModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub table_margin: f32,
    pub table_margin_right: f32,
    pub header: ContentInfoHeader,
    pub patched_banner: Option<ContentInfoPatchedBanner>,
    pub body: Vec<ContentInfoBodyBlock>,
    pub scroll_pane: ContentInfoScrollPane,
    pub show_action: ContentInfoShowAction,
    pub stats_check_ran: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentInfoHeader {
    pub icon: String,
    pub icon_size: f32,
    pub icon_scaling: &'static str,
    pub text: String,
    pub name_pad_left: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentInfoPatchedBanner {
    pub icon: &'static str,
    pub text: &'static str,
    pub color: &'static str,
    pub pad: f32,
    pub text_pad_left: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentInfoBodyBlock {
    Category(ContentInfoCategoryBlock),
    Description(ContentInfoDescriptionBlock),
    Stat(ContentInfoStatBlock),
    Details(ContentInfoDetailsBlock),
    Credit(ContentInfoCreditBlock),
    ViewFieldsButton(ContentInfoButton),
    Extra(ContentInfoExtraBlock),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentInfoCategoryBlock {
    pub text: String,
    pub color: &'static str,
    pub fill_x: bool,
    pub pad_top: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentInfoDescriptionBlock {
    pub text: String,
    pub width: f32,
    pub wrap: bool,
    pub fill_x: bool,
    pub pad_left: f32,
    pub pad_top: f32,
    pub left: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentInfoStatBlock {
    pub stat: Stat,
    pub label: String,
    pub info_key: Option<String>,
    pub values: Vec<Vec<String>>,
    pub value_spacer: f32,
    pub fill_x: bool,
    pub pad_left: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentInfoDetailsBlock {
    pub text: String,
    pub locked: bool,
    pub pad: f32,
    pub pad_top: f32,
    pub width: f32,
    pub wrap: bool,
    pub fill_x: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentInfoCreditBlock {
    pub bundle_key: &'static str,
    pub credit: String,
    pub color: &'static str,
    pub pad_top: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentInfoButton {
    pub text: &'static str,
    pub icon: &'static str,
    pub style: &'static str,
    pub margin: f32,
    pub pad: f32,
    pub pad_top: f32,
    pub size: (f32, f32),
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentInfoExtraBlock {
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentInfoScrollPane {
    pub contains_table: bool,
    pub vertical_scrolling: bool,
    pub horizontal_scrolling_disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentInfoShowAction {
    ShowDialog,
    ShowWithSceneFadeIn { duration: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentInfoKey {
    BlockInfo,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentInfoDialogAction {
    PostHide,
    OpenUri { uri: String },
}

pub struct ContentInfoDialog;

impl ContentInfoContent {
    pub fn new(base: UnlockableContentBase, stats: Stats) -> Self {
        let class_simple_name = default_content_class_simple_name(base.mappable.base.content_type);
        Self {
            base,
            stats,
            mod_display_name: None,
            class_simple_name: class_simple_name.to_string(),
            display_extra: Vec::new(),
        }
    }

    pub fn localized_name(&self) -> &str {
        self.base
            .localized_name
            .as_deref()
            .unwrap_or(&self.base.mappable.name)
    }

    pub fn display_description(&self) -> Option<String> {
        self.base.description.as_ref().map(|description| {
            if let Some(mod_display_name) = &self.mod_display_name {
                format!("{description}\n@mod.display({mod_display_name})")
            } else {
                description.clone()
            }
        })
    }

    fn check_stats(&mut self) -> bool {
        let ran = !self.stats.initialized;
        if ran {
            self.stats.initialized = true;
        }
        ran
    }
}

impl ContentInfoDialog {
    pub fn new() -> Self {
        Self
    }

    pub fn show(
        &self,
        content: &mut ContentInfoContent,
        context: ContentInfoDialogContext,
        stat_info: impl Fn(Stat) -> bool,
    ) -> ContentInfoDialogModel {
        let stats_check_ran = content.check_stats();
        let stats_map = content.stats.to_map();
        let any_stats = stats_map.iter().any(|(_, entries)| !entries.is_empty());
        let mut body = Vec::new();

        if let Some(description) = content.display_description() {
            if any_stats {
                body.push(ContentInfoBodyBlock::Category(ContentInfoCategoryBlock {
                    text: CONTENT_INFO_CATEGORY_PURPOSE.to_string(),
                    color: CONTENT_INFO_CATEGORY_COLOR,
                    fill_x: true,
                    pad_top: 10.0,
                }));
            }

            body.push(ContentInfoBodyBlock::Description(
                ContentInfoDescriptionBlock {
                    text: format!("{CONTENT_INFO_DESCRIPTION_COLOR_PREFIX}{description}"),
                    width: CONTENT_INFO_DESCRIPTION_WIDTH,
                    wrap: true,
                    fill_x: true,
                    pad_left: if any_stats {
                        CONTENT_INFO_DESCRIPTION_PAD_LEFT_WITH_STATS
                    } else {
                        CONTENT_INFO_DESCRIPTION_PAD_LEFT_NO_STATS
                    },
                    pad_top: if any_stats {
                        CONTENT_INFO_DESCRIPTION_PAD_TOP_WITH_STATS
                    } else {
                        CONTENT_INFO_DESCRIPTION_PAD_TOP_NO_STATS
                    },
                    left: true,
                },
            ));

            if !content.stats.use_categories && any_stats {
                body.push(ContentInfoBodyBlock::Category(ContentInfoCategoryBlock {
                    text: CONTENT_INFO_CATEGORY_GENERAL.to_string(),
                    color: CONTENT_INFO_CATEGORY_COLOR,
                    fill_x: true,
                    pad_top: 0.0,
                }));
            }
        }

        for (cat, entries) in stats_map {
            if entries.is_empty() {
                continue;
            }

            if content.stats.use_categories {
                body.push(ContentInfoBodyBlock::Category(ContentInfoCategoryBlock {
                    text: format!("@{}", cat.bundle_key()),
                    color: CONTENT_INFO_CATEGORY_COLOR,
                    fill_x: true,
                    pad_top: 0.0,
                }));
            }

            for (stat, values) in entries {
                body.push(ContentInfoBodyBlock::Stat(ContentInfoStatBlock {
                    stat,
                    label: format!(
                        "{CONTENT_INFO_STAT_LABEL_PREFIX}@{}{}",
                        stat.bundle_key(),
                        CONTENT_INFO_STAT_LABEL_SUFFIX
                    ),
                    info_key: Stats::stat_info_key(stat, stat_info(stat)),
                    values: values.iter().map(StatValue::display_tokens).collect(),
                    value_spacer: CONTENT_INFO_STAT_VALUE_SPACER,
                    fill_x: true,
                    pad_left: CONTENT_INFO_STAT_PAD_LEFT,
                }));
            }
        }

        if let Some(details) = content.base.details.as_ref() {
            let unlocked = content.base.unlocked();
            let visible_details = unlocked || !content.base.hide_details;
            let text = if visible_details {
                details.clone()
            } else {
                format!("{CONTENT_INFO_LOCK_ICON} {CONTENT_INFO_UNLOCK_IN_CAMPAIGN}")
            };
            body.push(ContentInfoBodyBlock::Details(ContentInfoDetailsBlock {
                text: format!("{CONTENT_INFO_DETAILS_COLOR_PREFIX}{text}"),
                locked: !visible_details,
                pad: CONTENT_INFO_DETAILS_PAD,
                pad_top: CONTENT_INFO_DETAILS_PAD_TOP,
                width: CONTENT_INFO_DETAILS_WIDTH,
                wrap: true,
                fill_x: true,
            }));
        }

        if let Some(credit) = content.base.credit.as_ref() {
            body.push(ContentInfoBodyBlock::Credit(ContentInfoCreditBlock {
                bundle_key: CONTENT_INFO_CREDIT_KEY,
                credit: credit.clone(),
                color: CONTENT_INFO_CREDIT_COLOR,
                pad_top: CONTENT_INFO_CREDIT_PAD_TOP,
            }));
        }

        if context.console {
            body.push(ContentInfoBodyBlock::ViewFieldsButton(
                Self::view_fields_button(content),
            ));
        }

        body.extend(
            content
                .display_extra
                .iter()
                .cloned()
                .map(ContentInfoBodyBlock::Extra),
        );

        ContentInfoDialogModel {
            title: CONTENT_INFO_TITLE,
            close_button_added: true,
            table_margin: CONTENT_INFO_TABLE_MARGIN,
            table_margin_right: CONTENT_INFO_TABLE_MARGIN_RIGHT,
            header: Self::header(content, context.console),
            patched_banner: (context.state_is_game && context.content_patched).then_some(
                ContentInfoPatchedBanner {
                    icon: CONTENT_INFO_PATCHED_ICON,
                    text: CONTENT_INFO_PATCHED_TEXT,
                    color: CONTENT_INFO_PATCHED_COLOR,
                    pad: CONTENT_INFO_PATCHED_PAD,
                    text_pad_left: CONTENT_INFO_PATCHED_TEXT_PAD_LEFT,
                },
            ),
            body,
            scroll_pane: ContentInfoScrollPane {
                contains_table: true,
                vertical_scrolling: true,
                horizontal_scrolling_disabled: false,
            },
            show_action: if context.dialog_already_shown {
                ContentInfoShowAction::ShowWithSceneFadeIn {
                    duration: CONTENT_INFO_RESHOW_FADE_DURATION,
                }
            } else {
                ContentInfoShowAction::ShowDialog
            },
            stats_check_ran,
        }
    }

    pub fn key_down_plan(key: ContentInfoKey) -> Vec<ContentInfoDialogAction> {
        match key {
            ContentInfoKey::BlockInfo => vec![ContentInfoDialogAction::PostHide],
            ContentInfoKey::Other => Vec::new(),
        }
    }

    pub fn view_fields_plan(content: &ContentInfoContent) -> Vec<ContentInfoDialogAction> {
        vec![ContentInfoDialogAction::OpenUri {
            uri: fields_uri(content),
        }]
    }

    fn header(content: &ContentInfoContent, console: bool) -> ContentInfoHeader {
        let name = content.localized_name();
        let mut text = format!("{CONTENT_INFO_HEADER_ACCENT_PREFIX}{name}");
        if console {
            text.push_str(CONTENT_INFO_CONSOLE_NAME_PREFIX);
            text.push_str(&content.base.mappable.name);
        }

        ContentInfoHeader {
            icon: content
                .base
                .icon_candidates(None)
                .ui_candidates
                .first()
                .cloned()
                .unwrap_or_else(|| content.base.mappable.name.clone()),
            icon_size: CONTENT_INFO_ICON_SIZE,
            icon_scaling: CONTENT_INFO_ICON_SCALING,
            text,
            name_pad_left: CONTENT_INFO_HEADER_NAME_PAD_LEFT,
        }
    }

    fn view_fields_button(content: &ContentInfoContent) -> ContentInfoButton {
        ContentInfoButton {
            text: CONTENT_INFO_VIEW_FIELDS_TEXT,
            icon: CONTENT_INFO_VIEW_FIELDS_ICON,
            style: CONTENT_INFO_VIEW_FIELDS_STYLE,
            margin: CONTENT_INFO_VIEW_FIELDS_MARGIN,
            pad: CONTENT_INFO_VIEW_FIELDS_PAD,
            pad_top: CONTENT_INFO_VIEW_FIELDS_PAD_TOP,
            size: CONTENT_INFO_VIEW_FIELDS_SIZE,
            uri: fields_uri(content),
        }
    }
}

pub fn fields_uri(content: &ContentInfoContent) -> String {
    format!(
        "{CONTENT_INFO_FIELDS_URI_PREFIX}{}",
        content.class_simple_name
    )
}

pub fn default_content_class_simple_name(content_type: ContentType) -> &'static str {
    match content_type {
        ContentType::Item => "Item",
        ContentType::Block => "Block",
        ContentType::Liquid => "Liquid",
        ContentType::Status => "StatusEffect",
        ContentType::Unit => "UnitType",
        ContentType::Weather => "Weather",
        ContentType::Sector => "SectorPreset",
        ContentType::Planet => "Planet",
        ContentType::Team => "Team",
        ContentType::UnitCommand => "UnitCommand",
        ContentType::UnitStance => "UnitStance",
        _ => "Content",
    }
}

impl Default for ContentInfoDialog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::{
        ctype::{ContentType, UnlockableContentBase},
        world::meta::{Stat, StatUnit},
    };

    fn copper_base() -> UnlockableContentBase {
        let mut base = UnlockableContentBase::new(0, ContentType::Item, "copper");
        base.localized_name = Some("Copper".into());
        base.description = Some("Used in all types of construction.".into());
        base.details = Some("Copper details are hidden until unlocked.".into());
        base.credit = Some("Test Author".into());
        base
    }

    #[test]
    fn show_builds_header_patched_banner_stats_locked_details_credit_and_viewfields() {
        let mut stats = Stats::new();
        stats.add_number(Stat::Health, 100.0, StatUnit::None);
        stats.add_bool(Stat::TargetsAir, true);
        let mut content = ContentInfoContent::new(copper_base(), stats);
        content.display_extra.push(ContentInfoExtraBlock {
            text: "@team.crux.log".into(),
        });

        let model = ContentInfoDialog::new().show(
            &mut content,
            ContentInfoDialogContext {
                console: true,
                state_is_game: true,
                content_patched: true,
                dialog_already_shown: true,
            },
            |stat| stat == Stat::Health,
        );

        assert!(model.stats_check_ran);
        assert!(content.stats.initialized);
        assert_eq!(model.title, "@info.title");
        assert!(model.close_button_added);
        assert_eq!(model.table_margin, 10.0);
        assert_eq!(model.table_margin_right, 30.0);
        assert_eq!(model.header.icon, "item-copper-ui");
        assert_eq!(model.header.icon_size, 48.0);
        assert_eq!(model.header.icon_scaling, "Scaling.fit");
        assert_eq!(model.header.text, "[accent]Copper\n[gray]copper");
        assert_eq!(
            model.patched_banner,
            Some(ContentInfoPatchedBanner {
                icon: "info",
                text: "@database.patched",
                color: "Pal.lightishGray",
                pad: 4.0,
                text_pad_left: 4.0,
            })
        );

        assert!(matches!(
            &model.body[0],
            ContentInfoBodyBlock::Category(block)
                if block.text == "@category.purpose" && block.pad_top == 10.0
        ));
        assert!(matches!(
            &model.body[1],
            ContentInfoBodyBlock::Description(block)
                if block.text == "[lightgray]Used in all types of construction."
                    && block.width == 500.0
                    && block.pad_left == 10.0
                    && block.pad_top == 0.0
        ));
        assert!(matches!(
            &model.body[2],
            ContentInfoBodyBlock::Category(block) if block.text == "@category.general"
        ));
        assert!(matches!(
            &model.body[3],
            ContentInfoBodyBlock::Stat(block)
                if block.stat == Stat::Health
                    && block.label == "[lightgray]@stat.health:[] "
                    && block.info_key.as_deref() == Some("@stat.health.info")
                    && block.values == vec![vec!["100".to_string(), "".to_string()]]
        ));
        assert!(matches!(
            &model.body[4],
            ContentInfoBodyBlock::Stat(block)
                if block.stat == Stat::TargetsAir
                    && block.info_key.is_none()
                    && block.values == vec![vec!["@yes".to_string()]]
        ));
        assert!(matches!(
            &model.body[5],
            ContentInfoBodyBlock::Details(block)
                if block.locked
                    && block.text == "[gray]Iconc.lock @unlock.incampaign"
                    && block.pad == 6.0
                    && block.pad_top == 20.0
                    && block.width == 400.0
        ));
        assert!(matches!(
            &model.body[6],
            ContentInfoBodyBlock::Credit(block)
                if block.bundle_key == "content.createdby"
                    && block.credit == "Test Author"
                    && block.color == "Color.gray"
                    && block.pad_top == 40.0
        ));
        assert!(matches!(
            &model.body[7],
            ContentInfoBodyBlock::ViewFieldsButton(button)
                if button.text == "@viewfields"
                    && button.icon == "link"
                    && button.style == "Styles.grayt"
                    && button.margin == 8.0
                    && button.pad == 4.0
                    && button.pad_top == 16.0
                    && button.size == (300.0, 50.0)
                    && button.uri == "https://mindustrygame.github.io/wiki/Modding%20Classes/Item"
        ));
        assert!(matches!(
            &model.body[8],
            ContentInfoBodyBlock::Extra(block) if block.text == "@team.crux.log"
        ));
        assert_eq!(
            model.show_action,
            ContentInfoShowAction::ShowWithSceneFadeIn { duration: 0.0 }
        );
    }

    #[test]
    fn show_omits_console_only_patched_and_viewfields_when_java_conditions_are_false() {
        let mut base = copper_base();
        base.hide_details = false;
        let mut content = ContentInfoContent::new(base, Stats::new());

        let model = ContentInfoDialog::new().show(
            &mut content,
            ContentInfoDialogContext::default(),
            |_| false,
        );

        assert_eq!(model.header.text, "[accent]Copper");
        assert_eq!(model.patched_banner, None);
        assert_eq!(model.show_action, ContentInfoShowAction::ShowDialog);
        assert!(model
            .body
            .iter()
            .all(|block| { !matches!(block, ContentInfoBodyBlock::ViewFieldsButton(_)) }));
        assert!(matches!(
            &model.body[0],
            ContentInfoBodyBlock::Description(block)
                if block.pad_left == 0.0 && block.pad_top == 10.0
        ));
        assert!(matches!(
            &model.body[1],
            ContentInfoBodyBlock::Details(block)
                if !block.locked && block.text == "[gray]Copper details are hidden until unlocked."
        ));
    }

    #[test]
    fn use_categories_displays_real_stat_categories_instead_of_general_header() {
        let mut stats = Stats::new();
        stats.use_categories = true;
        stats.add_number(Stat::PowerUse, 1.5, StatUnit::PowerSecond);
        stats.add_bool(Stat::TargetsGround, false);
        let mut content = ContentInfoContent::new(copper_base(), stats);

        let model = ContentInfoDialog::new().show(
            &mut content,
            ContentInfoDialogContext::default(),
            |_| false,
        );

        let categories = model
            .body
            .iter()
            .filter_map(|block| match block {
                ContentInfoBodyBlock::Category(category) => Some(category.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            categories,
            vec!["@category.purpose", "@category.power", "@category.function"]
        );
        assert!(!categories.contains(&"@category.general"));
    }

    #[test]
    fn key_down_and_viewfields_actions_match_java_callbacks() {
        let content = ContentInfoContent::new(copper_base(), Stats::new());

        assert_eq!(
            ContentInfoDialog::key_down_plan(ContentInfoKey::BlockInfo),
            vec![ContentInfoDialogAction::PostHide]
        );
        assert_eq!(
            ContentInfoDialog::key_down_plan(ContentInfoKey::Other),
            Vec::<ContentInfoDialogAction>::new()
        );
        assert_eq!(
            ContentInfoDialog::view_fields_plan(&content),
            vec![ContentInfoDialogAction::OpenUri {
                uri: "https://mindustrygame.github.io/wiki/Modding%20Classes/Item".into(),
            }]
        );
        assert_eq!(
            default_content_class_simple_name(ContentType::Block),
            "Block"
        );
        assert_eq!(
            default_content_class_simple_name(ContentType::Status),
            "StatusEffect"
        );
    }
}
