//! Admin list dialog model mirroring upstream `mindustry.ui.dialogs.AdminsDialog`.

use crate::mindustry::ui::upstream_menu_bundle_format_for_locale;

pub const ADMINS_DIALOG_TITLE: &str = "@server.admins";
pub const ADMINS_DIALOG_ROW_WIDTH: f32 = 400.0;
pub const ADMINS_DIALOG_ROW_HEIGHT: f32 = 80.0;
pub const ADMINS_DIALOG_ROW_MARGIN: f32 = 14.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminPlayerInfo {
    pub id: String,
    pub last_name: String,
}

impl AdminPlayerInfo {
    pub fn new(id: impl Into<String>, last_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            last_name: last_name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdminsDialogRow {
    pub id: String,
    pub label: String,
    pub confirm_title: &'static str,
    pub confirm_text: String,
    pub button_size: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdminsDialogModel {
    pub title: &'static str,
    pub empty_label: Option<&'static str>,
    pub row_width: f32,
    pub row_height: f32,
    pub row_margin: f32,
    pub rows: Vec<AdminsDialogRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AdminsDialog;

impl AdminsDialog {
    pub fn new() -> Self {
        Self
    }

    pub fn setup(&self, admins: &[AdminPlayerInfo]) -> AdminsDialogModel {
        let rows = admins
            .iter()
            .map(|info| AdminsDialogRow {
                id: info.id.clone(),
                label: format!("[lightgray]{}", info.last_name),
                confirm_title: "@confirm",
                confirm_text: upstream_menu_bundle_format_for_locale(
                    "en",
                    "confirmunadmin",
                    &[info.last_name.as_str()],
                )
                .unwrap_or_else(|| format!("@confirmunadmin: {}", info.last_name)),
                button_size: ADMINS_DIALOG_ROW_HEIGHT,
            })
            .collect::<Vec<_>>();

        AdminsDialogModel {
            title: ADMINS_DIALOG_TITLE,
            empty_label: admins.is_empty().then_some("@server.admins.none"),
            row_width: ADMINS_DIALOG_ROW_WIDTH,
            row_height: ADMINS_DIALOG_ROW_HEIGHT,
            row_margin: ADMINS_DIALOG_ROW_MARGIN,
            rows,
        }
    }

    pub fn unadmin_target(row: &AdminsDialogRow) -> &str {
        &row.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_admins_dialog_uses_java_empty_label() {
        let model = AdminsDialog::new().setup(&[]);

        assert_eq!(model.title, "@server.admins");
        assert_eq!(model.empty_label, Some("@server.admins.none"));
        assert!(model.rows.is_empty());
    }

    #[test]
    fn admins_dialog_rows_match_java_sizes_label_and_unadmin_target() {
        let admins = [AdminPlayerInfo::new("uuid-1", "Player")];
        let model = AdminsDialog::new().setup(&admins);

        assert_eq!(model.empty_label, None);
        assert_eq!(model.row_width, 400.0);
        assert_eq!(model.row_height, 80.0);
        assert_eq!(model.rows[0].label, "[lightgray]Player");
        assert_eq!(AdminsDialog::unadmin_target(&model.rows[0]), "uuid-1");
    }
}
