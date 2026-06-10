//! Discord dialog model mirroring upstream `mindustry.ui.dialogs.DiscordDialog`.

pub const DISCORD_DIALOG_TITLE: &str = "";
pub const DISCORD_URL: &str = "https://discord.gg/mindustry";
pub const DISCORD_DIALOG_MARGIN: f32 = 12.0;
pub const DISCORD_CARD_HEIGHT: f32 = 70.0;
pub const DISCORD_CARD_WIDTH: f32 = 520.0;
pub const DISCORD_CARD_PAD: f32 = 10.0;
pub const DISCORD_CARD_BACKGROUND: &str = "Tex.button";
pub const DISCORD_COLOR_RGBA: u32 = 0x7289daff;
pub const DISCORD_DARK_COLOR_RGBA: u32 = 0x5b6eaeff;
pub const DISCORD_STRIPE_WIDTH: f32 = 40.0;
pub const DISCORD_STRIPE_DARK_HEIGHT: f32 = 5.0;
pub const DISCORD_ICON: &str = "discord";
pub const DISCORD_TEXT: &str = "@discord";
pub const DISCORD_TEXT_COLOR: &str = "Pal.accent";
pub const DISCORD_TEXT_PAD_LEFT: f32 = 10.0;
pub const DISCORD_BUTTON_SIZE: (f32, f32) = (170.0, 50.0);
pub const DISCORD_BACK_TEXT: &str = "@back";
pub const DISCORD_BACK_ICON: &str = "left";
pub const DISCORD_COPY_TEXT: &str = "@copylink";
pub const DISCORD_COPY_ICON: &str = "copy";
pub const DISCORD_OPEN_TEXT: &str = "@openlink";
pub const DISCORD_OPEN_ICON: &str = "discord";
pub const DISCORD_COPIED_INFO: &str = "@copied";
pub const DISCORD_LINK_FAIL_ERROR: &str = "@linkfail";

#[derive(Debug, Clone, PartialEq)]
pub struct DiscordDialogModel {
    pub title: &'static str,
    pub margin: f32,
    pub card: DiscordCard,
    pub buttons: Vec<DiscordButton>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscordCard {
    pub background: &'static str,
    pub width: f32,
    pub height: f32,
    pub pad: f32,
    pub color_rgba: u32,
    pub dark_color_rgba: u32,
    pub stripe_width: f32,
    pub stripe_light_height: f32,
    pub stripe_dark_height: f32,
    pub icon: &'static str,
    pub icon_size: f32,
    pub text: &'static str,
    pub text_color: &'static str,
    pub text_pad_left: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscordButtonKind {
    Back,
    CopyLink,
    OpenLink,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscordButton {
    pub kind: DiscordButtonKind,
    pub text: &'static str,
    pub icon: &'static str,
    pub size: (f32, f32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscordDialogAction {
    HideDialog,
    SetClipboardText { text: &'static str },
    ShowInfoFade { message: &'static str },
    OpenUri { uri: &'static str },
    ShowErrorMessage { message: &'static str },
}

pub struct DiscordDialog;

impl DiscordDialog {
    pub fn model() -> DiscordDialogModel {
        DiscordDialogModel {
            title: DISCORD_DIALOG_TITLE,
            margin: DISCORD_DIALOG_MARGIN,
            card: DiscordCard {
                background: DISCORD_CARD_BACKGROUND,
                width: DISCORD_CARD_WIDTH,
                height: DISCORD_CARD_HEIGHT,
                pad: DISCORD_CARD_PAD,
                color_rgba: DISCORD_COLOR_RGBA,
                dark_color_rgba: DISCORD_DARK_COLOR_RGBA,
                stripe_width: DISCORD_STRIPE_WIDTH,
                stripe_light_height: DISCORD_CARD_HEIGHT - DISCORD_STRIPE_DARK_HEIGHT,
                stripe_dark_height: DISCORD_STRIPE_DARK_HEIGHT,
                icon: DISCORD_ICON,
                icon_size: DISCORD_CARD_HEIGHT,
                text: DISCORD_TEXT,
                text_color: DISCORD_TEXT_COLOR,
                text_pad_left: DISCORD_TEXT_PAD_LEFT,
            },
            buttons: vec![
                DiscordButton {
                    kind: DiscordButtonKind::Back,
                    text: DISCORD_BACK_TEXT,
                    icon: DISCORD_BACK_ICON,
                    size: DISCORD_BUTTON_SIZE,
                },
                DiscordButton {
                    kind: DiscordButtonKind::CopyLink,
                    text: DISCORD_COPY_TEXT,
                    icon: DISCORD_COPY_ICON,
                    size: DISCORD_BUTTON_SIZE,
                },
                DiscordButton {
                    kind: DiscordButtonKind::OpenLink,
                    text: DISCORD_OPEN_TEXT,
                    icon: DISCORD_OPEN_ICON,
                    size: DISCORD_BUTTON_SIZE,
                },
            ],
        }
    }

    pub fn button_plan(
        kind: DiscordButtonKind,
        open_uri_succeeds: bool,
    ) -> Vec<DiscordDialogAction> {
        match kind {
            DiscordButtonKind::Back => vec![DiscordDialogAction::HideDialog],
            DiscordButtonKind::CopyLink => vec![
                DiscordDialogAction::SetClipboardText { text: DISCORD_URL },
                DiscordDialogAction::ShowInfoFade {
                    message: DISCORD_COPIED_INFO,
                },
            ],
            DiscordButtonKind::OpenLink => {
                let mut actions = vec![DiscordDialogAction::OpenUri { uri: DISCORD_URL }];
                if !open_uri_succeeds {
                    actions.extend([
                        DiscordDialogAction::ShowErrorMessage {
                            message: DISCORD_LINK_FAIL_ERROR,
                        },
                        DiscordDialogAction::SetClipboardText { text: DISCORD_URL },
                    ]);
                }
                actions
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_matches_java_constructor_card_and_buttons() {
        let model = DiscordDialog::model();

        assert_eq!(model.title, "");
        assert_eq!(model.margin, 12.0);
        assert_eq!(model.card.background, "Tex.button");
        assert_eq!(model.card.width, 520.0);
        assert_eq!(model.card.height, 70.0);
        assert_eq!(model.card.color_rgba, 0x7289daff);
        assert_eq!(model.card.dark_color_rgba, 0x5b6eaeff);
        assert_eq!(model.card.stripe_light_height, 65.0);
        assert_eq!(model.card.stripe_dark_height, 5.0);
        assert_eq!(model.card.icon, "discord");
        assert_eq!(model.card.text, "@discord");
        assert_eq!(
            model
                .buttons
                .iter()
                .map(|button| (button.kind, button.text, button.icon, button.size))
                .collect::<Vec<_>>(),
            vec![
                (DiscordButtonKind::Back, "@back", "left", (170.0, 50.0)),
                (
                    DiscordButtonKind::CopyLink,
                    "@copylink",
                    "copy",
                    (170.0, 50.0)
                ),
                (
                    DiscordButtonKind::OpenLink,
                    "@openlink",
                    "discord",
                    (170.0, 50.0),
                ),
            ]
        );
    }

    #[test]
    fn button_plans_hide_copy_or_open_with_clipboard_fallback() {
        assert_eq!(
            DiscordDialog::button_plan(DiscordButtonKind::Back, true),
            vec![DiscordDialogAction::HideDialog]
        );
        assert_eq!(
            DiscordDialog::button_plan(DiscordButtonKind::CopyLink, true),
            vec![
                DiscordDialogAction::SetClipboardText {
                    text: "https://discord.gg/mindustry",
                },
                DiscordDialogAction::ShowInfoFade { message: "@copied" },
            ]
        );
        assert_eq!(
            DiscordDialog::button_plan(DiscordButtonKind::OpenLink, false),
            vec![
                DiscordDialogAction::OpenUri {
                    uri: "https://discord.gg/mindustry",
                },
                DiscordDialogAction::ShowErrorMessage {
                    message: "@linkfail",
                },
                DiscordDialogAction::SetClipboardText {
                    text: "https://discord.gg/mindustry",
                },
            ]
        );
    }
}
