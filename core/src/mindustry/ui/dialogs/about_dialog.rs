//! About dialog model mirroring upstream `mindustry.ui.dialogs.AboutDialog`.

use crate::mindustry::ui::links::{get_links_for_locale, LinkEntry};

pub const ABOUT_DIALOG_TITLE: &str = "@about.button";
pub const ABOUT_CONTRIBUTORS_FILE: &str = "contributors";
pub const ABOUT_CONTRIBUTORS_ENCODING: &str = "UTF-8";
pub const ABOUT_LINK_PORTRAIT_HEIGHT: f32 = 90.0;
pub const ABOUT_LINK_LANDSCAPE_HEIGHT: f32 = 80.0;
pub const ABOUT_LINK_PORTRAIT_WIDTH: f32 = 400.0;
pub const ABOUT_LINK_LANDSCAPE_WIDTH: f32 = 600.0;
pub const ABOUT_LINK_PANEL_STYLE: &str = "Styles.grayPanel";
pub const ABOUT_LINK_PANEL_MARGIN: f32 = 0.0;
pub const ABOUT_LINK_STRIPE_WIDTH: f32 = 40.0;
pub const ABOUT_LINK_STRIPE_DARK_HEIGHT: f32 = 5.0;
pub const ABOUT_LINK_ICON_PAD_LEFT: f32 = 8.0;
pub const ABOUT_LINK_BUTTON_ICON: &str = "link";
pub const ABOUT_LINK_BUTTON_STYLE: &str = "Styles.clearNonei";
pub const ABOUT_LINK_PAD_TOP: f32 = 5.0;
pub const ABOUT_CREDITS_BUTTON_TEXT: &str = "@credits";
pub const ABOUT_CREDITS_BUTTON_SIZE: (f32, f32) = (200.0, 64.0);
pub const ABOUT_CREDITS_TITLE: &str = "@credits";
pub const ABOUT_CREDITS_TEXT: &str = "@credits.text";
pub const ABOUT_CREDITS_ALIGNMENT: &str = "Align.center";
pub const ABOUT_CREDITS_DIVIDER_COLOR: &str = "Pal.accent";
pub const ABOUT_CREDITS_DIVIDER_HEIGHT: f32 = 3.0;
pub const ABOUT_CREDITS_DIVIDER_PAD: f32 = 3.0;
pub const ABOUT_CONTRIBUTORS_LABEL: &str = "@contributors";
pub const ABOUT_CONTRIBUTOR_PREFIX: &str = "[lightgray]";
pub const ABOUT_CONTRIBUTOR_COLUMNS: usize = 3;
pub const ABOUT_CONTRIBUTOR_PAD: f32 = 3.0;
pub const ABOUT_CONTRIBUTOR_PAD_LEFT: f32 = 6.0;
pub const ABOUT_CONTRIBUTOR_PAD_RIGHT: f32 = 6.0;
pub const ABOUT_LINK_FAIL_ERROR: &str = "@linkfail";
pub const ABOUT_OPEN_WIKI_TRIGGER: &str = "Trigger.openWiki";

pub const ABOUT_BANNED_ITEMS_ON_IOS_OR_STEAM: [&str; 4] =
    ["google-play", "itch.io", "dev-builds", "f-droid"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AboutDialogContext {
    pub portrait: bool,
    pub ios: bool,
    pub steam: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AboutDialogModel {
    pub title: &'static str,
    pub link_rows: Vec<AboutLinkRow>,
    pub scroll_focus_delay_frames: f32,
    pub close_button_added: bool,
    pub credits_button: AboutDialogButton,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AboutLinkRow {
    pub name: String,
    pub title: String,
    pub description: String,
    pub link: String,
    pub icon: String,
    pub color_rgba: u32,
    pub dark_color_rgba: u32,
    pub style: &'static str,
    pub margin: f32,
    pub width: f32,
    pub height: f32,
    pub stripe_width: f32,
    pub stripe_light_height: f32,
    pub stripe_dark_height: f32,
    pub icon_size: (f32, f32),
    pub description_width: f32,
    pub inset_pad_left: f32,
    pub link_button: AboutDialogButton,
    pub pad_top: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AboutDialogButton {
    pub text: Option<&'static str>,
    pub icon: Option<&'static str>,
    pub style: Option<&'static str>,
    pub size: (f32, f32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutCreditsDialogModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub text: &'static str,
    pub alignment: &'static str,
    pub divider: Option<AboutCreditsDivider>,
    pub contributors_label: Option<&'static str>,
    pub contributors: Vec<AboutContributorCell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutCreditsDivider {
    pub color: &'static str,
    pub height: i32,
    pub pad: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutContributorCell {
    pub text: String,
    pub row: usize,
    pub column: usize,
    pub pad: i32,
    pub pad_left: i32,
    pub pad_right: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AboutDialogAction {
    ReadContributors {
        path: &'static str,
        encoding: &'static str,
    },
    PostSetup,
    Setup,
    FireTrigger {
        trigger: &'static str,
    },
    OpenUri {
        uri: String,
    },
    ShowErrorMessage {
        message: &'static str,
    },
    SetClipboardText {
        text: String,
    },
    ShowCredits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutDialog {
    pub contributors: Vec<String>,
}

impl Default for AboutDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl AboutDialog {
    pub fn new() -> Self {
        Self {
            contributors: Vec::new(),
        }
    }

    pub fn shown_plan() -> Vec<AboutDialogAction> {
        vec![
            AboutDialogAction::ReadContributors {
                path: ABOUT_CONTRIBUTORS_FILE,
                encoding: ABOUT_CONTRIBUTORS_ENCODING,
            },
            AboutDialogAction::PostSetup,
            AboutDialogAction::Setup,
            AboutDialogAction::Setup,
        ]
    }

    pub fn set_contributors_from_file(&mut self, text: &str) {
        let mut contributors = text.split('\n').map(str::to_string).collect::<Vec<_>>();
        while contributors.last().is_some_and(String::is_empty) {
            contributors.pop();
        }
        self.contributors = contributors;
    }

    pub fn setup_model(&self, context: AboutDialogContext, locale: &str) -> AboutDialogModel {
        let height = if context.portrait {
            ABOUT_LINK_PORTRAIT_HEIGHT
        } else {
            ABOUT_LINK_LANDSCAPE_HEIGHT
        };
        let width = if context.portrait {
            ABOUT_LINK_PORTRAIT_WIDTH
        } else {
            ABOUT_LINK_LANDSCAPE_WIDTH
        };
        AboutDialogModel {
            title: ABOUT_DIALOG_TITLE,
            link_rows: about_link_rows(context, locale, width, height),
            scroll_focus_delay_frames: 1.0,
            close_button_added: true,
            credits_button: AboutDialogButton {
                text: Some(ABOUT_CREDITS_BUTTON_TEXT),
                icon: None,
                style: None,
                size: ABOUT_CREDITS_BUTTON_SIZE,
            },
        }
    }

    pub fn link_button_plan(
        link: &AboutLinkRow,
        open_uri_succeeds: bool,
    ) -> Vec<AboutDialogAction> {
        let mut actions = Vec::new();
        if link.name == "wiki" {
            actions.push(AboutDialogAction::FireTrigger {
                trigger: ABOUT_OPEN_WIKI_TRIGGER,
            });
        }
        actions.push(AboutDialogAction::OpenUri {
            uri: link.link.clone(),
        });
        if !open_uri_succeeds {
            actions.extend([
                AboutDialogAction::ShowErrorMessage {
                    message: ABOUT_LINK_FAIL_ERROR,
                },
                AboutDialogAction::SetClipboardText {
                    text: link.link.clone(),
                },
            ]);
        }
        actions
    }

    pub fn show_credits_plan() -> Vec<AboutDialogAction> {
        vec![AboutDialogAction::ShowCredits]
    }

    pub fn credits_model(&self) -> AboutCreditsDialogModel {
        let has_contributors = !self.contributors.is_empty();
        AboutCreditsDialogModel {
            title: ABOUT_CREDITS_TITLE,
            close_button_added: true,
            text: ABOUT_CREDITS_TEXT,
            alignment: ABOUT_CREDITS_ALIGNMENT,
            divider: has_contributors.then_some(AboutCreditsDivider {
                color: ABOUT_CREDITS_DIVIDER_COLOR,
                height: ABOUT_CREDITS_DIVIDER_HEIGHT as i32,
                pad: ABOUT_CREDITS_DIVIDER_PAD as i32,
            }),
            contributors_label: has_contributors.then_some(ABOUT_CONTRIBUTORS_LABEL),
            contributors: self
                .contributors
                .iter()
                .enumerate()
                .map(|(index, contributor)| AboutContributorCell {
                    text: format!("{ABOUT_CONTRIBUTOR_PREFIX}{contributor}"),
                    row: index / ABOUT_CONTRIBUTOR_COLUMNS,
                    column: index % ABOUT_CONTRIBUTOR_COLUMNS,
                    pad: ABOUT_CONTRIBUTOR_PAD as i32,
                    pad_left: ABOUT_CONTRIBUTOR_PAD_LEFT as i32,
                    pad_right: ABOUT_CONTRIBUTOR_PAD_RIGHT as i32,
                })
                .collect(),
        }
    }
}

pub fn about_link_rows(
    context: AboutDialogContext,
    locale: &str,
    width: f32,
    height: f32,
) -> Vec<AboutLinkRow> {
    get_links_for_locale(locale)
        .into_iter()
        .filter(|link| !(context.ios || context.steam) || !banned_about_link(&link.name))
        .map(|link| about_link_row(link, width, height))
        .collect()
}

fn about_link_row(link: LinkEntry, width: f32, height: f32) -> AboutLinkRow {
    AboutLinkRow {
        name: link.name,
        title: format!("[accent]{}", link.title),
        description: link.description,
        link: link.link,
        icon: link.icon_name,
        color_rgba: link.color_rgba,
        dark_color_rgba: mul_rgb(link.color_rgba, 0.6, 0.6, 0.8),
        style: ABOUT_LINK_PANEL_STYLE,
        margin: ABOUT_LINK_PANEL_MARGIN,
        width,
        height,
        stripe_width: ABOUT_LINK_STRIPE_WIDTH,
        stripe_light_height: height - ABOUT_LINK_STRIPE_DARK_HEIGHT,
        stripe_dark_height: ABOUT_LINK_STRIPE_DARK_HEIGHT,
        icon_size: (height - ABOUT_LINK_STRIPE_DARK_HEIGHT, height),
        description_width: width - 100.0 - height,
        inset_pad_left: ABOUT_LINK_ICON_PAD_LEFT,
        link_button: AboutDialogButton {
            text: None,
            icon: Some(ABOUT_LINK_BUTTON_ICON),
            style: Some(ABOUT_LINK_BUTTON_STYLE),
            size: (height - ABOUT_LINK_STRIPE_DARK_HEIGHT, height),
        },
        pad_top: ABOUT_LINK_PAD_TOP,
    }
}

fn banned_about_link(name: &str) -> bool {
    ABOUT_BANNED_ITEMS_ON_IOS_OR_STEAM.contains(&name)
}

fn mul_rgb(rgba: u32, r: f32, g: f32, b: f32) -> u32 {
    let rr = (((rgba >> 24) & 0xff) as f32 * r).round().clamp(0.0, 255.0) as u32;
    let gg = (((rgba >> 16) & 0xff) as f32 * g).round().clamp(0.0, 255.0) as u32;
    let bb = (((rgba >> 8) & 0xff) as f32 * b).round().clamp(0.0, 255.0) as u32;
    let aa = rgba & 0xff;
    (rr << 24) | (gg << 16) | (bb << 8) | aa
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shown_plan_reads_contributors_then_runs_posted_and_direct_setup_like_java() {
        assert_eq!(
            AboutDialog::shown_plan(),
            vec![
                AboutDialogAction::ReadContributors {
                    path: "contributors",
                    encoding: "UTF-8",
                },
                AboutDialogAction::PostSetup,
                AboutDialogAction::Setup,
                AboutDialogAction::Setup,
            ]
        );
    }

    #[test]
    fn setup_model_builds_links_filters_banned_items_and_credits_button() {
        let dialog = AboutDialog::new();
        let desktop = dialog.setup_model(AboutDialogContext::default(), "en");

        assert_eq!(desktop.title, "@about.button");
        assert_eq!(desktop.link_rows.len(), 12);
        assert_eq!(desktop.link_rows[0].name, "discord");
        assert_eq!(desktop.link_rows[0].title, "[accent]Discord");
        assert_eq!(desktop.link_rows[0].style, "Styles.grayPanel");
        assert_eq!(desktop.link_rows[0].height, 80.0);
        assert_eq!(desktop.link_rows[0].width, 600.0);
        assert_eq!(desktop.link_rows[0].stripe_width, 40.0);
        assert_eq!(desktop.link_rows[0].stripe_light_height, 75.0);
        assert_eq!(desktop.link_rows[0].link_button.icon, Some("link"));
        assert_eq!(desktop.link_rows[0].description_width, 420.0);
        assert!(desktop.close_button_added);
        assert_eq!(desktop.credits_button.text, Some("@credits"));
        assert_eq!(desktop.credits_button.size, (200.0, 64.0));

        let mobile = dialog.setup_model(
            AboutDialogContext {
                portrait: true,
                ios: true,
                steam: false,
            },
            "en",
        );
        assert_eq!(mobile.link_rows.len(), 8);
        assert!(!mobile
            .link_rows
            .iter()
            .any(|row| row.name == "google-play" || row.name == "itch.io"));
        assert_eq!(mobile.link_rows[0].height, 90.0);
        assert_eq!(mobile.link_rows[0].width, 400.0);
        assert_eq!(mobile.link_rows[0].description_width, 210.0);
    }

    #[test]
    fn link_button_plan_fires_wiki_trigger_and_falls_back_to_clipboard_on_failure() {
        let dialog = AboutDialog::new();
        let model = dialog.setup_model(AboutDialogContext::default(), "en");
        let wiki = model
            .link_rows
            .iter()
            .find(|row| row.name == "wiki")
            .unwrap();

        assert_eq!(
            AboutDialog::link_button_plan(wiki, false),
            vec![
                AboutDialogAction::FireTrigger {
                    trigger: "Trigger.openWiki",
                },
                AboutDialogAction::OpenUri {
                    uri: "https://mindustrygame.github.io/wiki/".into(),
                },
                AboutDialogAction::ShowErrorMessage {
                    message: "@linkfail",
                },
                AboutDialogAction::SetClipboardText {
                    text: "https://mindustrygame.github.io/wiki/".into(),
                },
            ]
        );
        assert_eq!(
            AboutDialog::link_button_plan(&model.link_rows[0], true),
            vec![AboutDialogAction::OpenUri {
                uri: "https://discord.gg/mindustry".into(),
            }]
        );
    }

    #[test]
    fn credits_dialog_wraps_text_and_places_contributors_in_three_columns() {
        let mut dialog = AboutDialog::new();
        dialog.set_contributors_from_file("Alice\nBob\nCara\nDan\n");

        let model = dialog.credits_model();

        assert_eq!(model.title, "@credits");
        assert!(model.close_button_added);
        assert_eq!(model.text, "@credits.text");
        assert_eq!(model.alignment, "Align.center");
        assert_eq!(
            model.divider,
            Some(AboutCreditsDivider {
                color: "Pal.accent",
                height: 3,
                pad: 3,
            })
        );
        assert_eq!(model.contributors_label, Some("@contributors"));
        assert_eq!(
            model
                .contributors
                .iter()
                .map(|cell| (cell.text.as_str(), cell.row, cell.column))
                .collect::<Vec<_>>(),
            vec![
                ("[lightgray]Alice", 0, 0),
                ("[lightgray]Bob", 0, 1),
                ("[lightgray]Cara", 0, 2),
                ("[lightgray]Dan", 1, 0),
            ]
        );
        assert_eq!(model.contributors.len(), 4);
    }
}
