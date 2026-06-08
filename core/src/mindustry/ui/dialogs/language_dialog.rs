//! Language dialog model mirroring upstream `mindustry.ui.dialogs.LanguageDialog`.

use super::BaseDialog;
use crate::mindustry::graphics::RenderFontId;

pub const LANGUAGE_DIALOG_TITLE_KEY: &str = "@settings.language";
pub const LANGUAGE_DIALOG_RESTART_MESSAGE_KEY: &str = "@language.restart";
pub const LANGUAGE_DIALOG_ROW_WIDTH: f32 = 400.0;
pub const LANGUAGE_DIALOG_ROW_HEIGHT: f32 = 50.0;
pub const LANGUAGE_DIALOG_TABLE_MARGIN_HORIZONTAL: f32 = 24.0;
pub const LANGUAGE_DIALOG_ROW_TEXT_BUTTON_STYLE: &str = "flatTogglet";
pub const LANGUAGE_DIALOG_ROW_FONT: RenderFontId = RenderFontId::Default;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageDialogLocale {
    pub code: String,
    pub display_name: String,
}

impl LanguageDialogLocale {
    pub fn new(code: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            display_name: display_name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LanguageDialogRow {
    pub code: String,
    pub display_name: String,
    pub selected: bool,
    pub button_width: f32,
    pub button_height: f32,
    pub button_style: &'static str,
    pub font: RenderFontId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LanguageDialog {
    pub base: BaseDialog,
    pub locales: Vec<LanguageDialogLocale>,
    pub selected_locale: String,
    pub last_restart_message: Option<&'static str>,
}

impl LanguageDialog {
    pub fn new(locales: Vec<LanguageDialogLocale>, selected_locale: impl Into<String>) -> Self {
        Self::new_with_title_text(LANGUAGE_DIALOG_TITLE_KEY, locales, selected_locale)
    }

    /// Java-style render seam that allows the caller to supply a localized
    /// dialog title before the dialog shell is drawn.
    ///
    /// The default `new(...)` constructor still keeps the raw key for
    /// compatibility with the existing tests and upstream-like state model.
    pub fn new_with_title_text(
        title_text: impl Into<String>,
        locales: Vec<LanguageDialogLocale>,
        selected_locale: impl Into<String>,
    ) -> Self {
        let selected_locale = selected_locale.into();
        Self {
            base: BaseDialog::new(title_text),
            locales,
            selected_locale,
            last_restart_message: None,
        }
    }

    pub fn rows(&self) -> Vec<LanguageDialogRow> {
        self.rows_with_display_names(
            self.locales
                .iter()
                .map(|locale| locale.display_name.clone()),
        )
    }

    /// Java-style render seam for pre-localized button copy.
    ///
    /// The row metadata stays tied to the locale list and selected state, while
    /// the caller may swap in already-localized button labels at render time.
    pub fn rows_with_display_names<I, S>(&self, display_names: I) -> Vec<LanguageDialogRow>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let display_names = display_names
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        self.locales
            .iter()
            .enumerate()
            .map(|(index, locale)| LanguageDialogRow {
                code: locale.code.clone(),
                display_name: display_names
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| locale.display_name.clone()),
                selected: locale.code == self.selected_locale,
                button_width: LANGUAGE_DIALOG_ROW_WIDTH,
                button_height: LANGUAGE_DIALOG_ROW_HEIGHT,
                button_style: LANGUAGE_DIALOG_ROW_TEXT_BUTTON_STYLE,
                font: LANGUAGE_DIALOG_ROW_FONT,
            })
            .collect()
    }

    pub fn select_locale(&mut self, code: impl AsRef<str>) -> Option<&'static str> {
        let code = code.as_ref();
        if self.selected_locale == code {
            return None;
        }
        self.selected_locale = code.to_string();
        self.last_restart_message = Some(LANGUAGE_DIALOG_RESTART_MESSAGE_KEY);
        self.last_restart_message
    }

    pub fn take_restart_message(&mut self) -> Option<&'static str> {
        self.last_restart_message.take()
    }

    pub fn find_closest_locale_code<'a>(
        locales: impl IntoIterator<Item = &'a str>,
        default_locale: &str,
    ) -> String {
        let locale_codes = locales.into_iter().collect::<Vec<_>>();
        if locale_codes
            .iter()
            .any(|locale| locale.eq_ignore_ascii_case(default_locale))
        {
            return locale_codes
                .iter()
                .find(|locale| locale.eq_ignore_ascii_case(default_locale))
                .copied()
                .unwrap_or(default_locale)
                .to_string();
        }

        let default_language = default_locale
            .split(['_', '-'])
            .next()
            .unwrap_or(default_locale);
        if let Some(locale) = locale_codes.iter().find(|locale| {
            locale
                .split(['_', '-'])
                .next()
                .unwrap_or(locale)
                .eq_ignore_ascii_case(default_language)
        }) {
            return (*locale).to_string();
        }

        "en".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_dialog_rows_match_java_button_group_metrics() {
        let dialog = LanguageDialog::new(
            vec![
                LanguageDialogLocale::new("en", "English"),
                LanguageDialogLocale::new("zh_CN", "简体中文"),
            ],
            "zh_CN",
        );

        assert_eq!(dialog.base.title, LANGUAGE_DIALOG_TITLE_KEY);
        assert_eq!(LANGUAGE_DIALOG_TABLE_MARGIN_HORIZONTAL, 24.0);
        assert_eq!(
            dialog.rows(),
            vec![
                LanguageDialogRow {
                    code: "en".into(),
                    display_name: "English".into(),
                    selected: false,
                    button_width: 400.0,
                    button_height: 50.0,
                    button_style: LANGUAGE_DIALOG_ROW_TEXT_BUTTON_STYLE,
                    font: RenderFontId::Default,
                },
                LanguageDialogRow {
                    code: "zh_CN".into(),
                    display_name: "简体中文".into(),
                    selected: true,
                    button_width: 400.0,
                    button_height: 50.0,
                    button_style: LANGUAGE_DIALOG_ROW_TEXT_BUTTON_STYLE,
                    font: RenderFontId::Default,
                },
            ]
        );
    }

    #[test]
    fn language_dialog_accepts_prelocalized_title_and_button_texts_before_render() {
        let dialog = LanguageDialog::new_with_title_text(
            "语言",
            vec![
                LanguageDialogLocale::new("en", "English"),
                LanguageDialogLocale::new("zh_CN", "简体中文"),
            ],
            "zh_CN",
        );

        assert_eq!(dialog.base.title, "语言");
        assert_eq!(
            dialog.rows_with_display_names(["English", "简体中文"]),
            vec![
                LanguageDialogRow {
                    code: "en".into(),
                    display_name: "English".into(),
                    selected: false,
                    button_width: 400.0,
                    button_height: 50.0,
                    button_style: LANGUAGE_DIALOG_ROW_TEXT_BUTTON_STYLE,
                    font: RenderFontId::Default,
                },
                LanguageDialogRow {
                    code: "zh_CN".into(),
                    display_name: "简体中文".into(),
                    selected: true,
                    button_width: 400.0,
                    button_height: 50.0,
                    button_style: LANGUAGE_DIALOG_ROW_TEXT_BUTTON_STYLE,
                    font: RenderFontId::Default,
                },
            ]
        );
        assert_eq!(
            dialog.rows(),
            vec![
                LanguageDialogRow {
                    code: "en".into(),
                    display_name: "English".into(),
                    selected: false,
                    button_width: 400.0,
                    button_height: 50.0,
                    button_style: LANGUAGE_DIALOG_ROW_TEXT_BUTTON_STYLE,
                    font: RenderFontId::Default,
                },
                LanguageDialogRow {
                    code: "zh_CN".into(),
                    display_name: "简体中文".into(),
                    selected: true,
                    button_width: 400.0,
                    button_height: 50.0,
                    button_style: LANGUAGE_DIALOG_ROW_TEXT_BUTTON_STYLE,
                    font: RenderFontId::Default,
                },
            ],
            "the legacy rows() path must remain compatible for existing tests and callers"
        );
    }

    #[test]
    fn language_dialog_select_locale_emits_restart_notice_like_java() {
        let mut dialog = LanguageDialog::new(
            vec![
                LanguageDialogLocale::new("en", "English"),
                LanguageDialogLocale::new("ja", "日本語"),
            ],
            "en",
        );

        assert_eq!(dialog.select_locale("en"), None);
        assert_eq!(
            dialog.select_locale("ja"),
            Some(LANGUAGE_DIALOG_RESTART_MESSAGE_KEY)
        );
        assert_eq!(dialog.selected_locale, "ja");
        assert_eq!(
            dialog.take_restart_message(),
            Some(LANGUAGE_DIALOG_RESTART_MESSAGE_KEY)
        );
        assert_eq!(dialog.take_restart_message(), None);
    }

    #[test]
    fn language_dialog_find_closest_locale_matches_java_order() {
        let locales = ["en", "pt_BR", "pt_PT", "zh_CN", "zh_TW"];
        assert_eq!(
            LanguageDialog::find_closest_locale_code(locales, "pt_PT"),
            "pt_PT"
        );
        assert_eq!(
            LanguageDialog::find_closest_locale_code(locales, "pt_AO"),
            "pt_BR"
        );
        assert_eq!(
            LanguageDialog::find_closest_locale_code(locales, "fr_FR"),
            "en"
        );
    }
}
