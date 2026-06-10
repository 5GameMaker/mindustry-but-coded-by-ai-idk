//! Ban list dialog model mirroring upstream `mindustry.ui.dialogs.BansDialog`.

pub const BANS_DIALOG_TITLE: &str = "@server.bans";
pub const BANS_DIALOG_ROW_WIDTH: f32 = 400.0;
pub const BANS_DIALOG_ROW_HEIGHT: f32 = 80.0;
pub const BANS_DIALOG_ROW_MARGIN: f32 = 14.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannedPlayerInfo {
    pub id: String,
    pub last_ip: String,
    pub last_name: String,
}

impl BannedPlayerInfo {
    pub fn new(
        id: impl Into<String>,
        last_ip: impl Into<String>,
        last_name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            last_ip: last_ip.into(),
            last_name: last_name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BansDialogRow {
    pub id: String,
    pub label: String,
    pub confirm_title: &'static str,
    pub confirm_text: &'static str,
    pub button_size: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BansDialogModel {
    pub title: &'static str,
    pub empty_label: Option<&'static str>,
    pub row_width: f32,
    pub row_height: f32,
    pub row_margin: f32,
    pub rows: Vec<BansDialogRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BansDialog;

impl BansDialog {
    pub fn new() -> Self {
        Self
    }

    pub fn setup(&self, banned: &[BannedPlayerInfo]) -> BansDialogModel {
        let rows = banned
            .iter()
            .map(|info| BansDialogRow {
                id: info.id.clone(),
                label: format!(
                    "IP: [lightgray]{}\n[]Name: [lightgray]{}",
                    info.last_ip, info.last_name
                ),
                confirm_title: "@confirm",
                confirm_text: "@confirmunban",
                button_size: BANS_DIALOG_ROW_HEIGHT,
            })
            .collect::<Vec<_>>();

        BansDialogModel {
            title: BANS_DIALOG_TITLE,
            empty_label: banned.is_empty().then_some("@server.bans.none"),
            row_width: BANS_DIALOG_ROW_WIDTH,
            row_height: BANS_DIALOG_ROW_HEIGHT,
            row_margin: BANS_DIALOG_ROW_MARGIN,
            rows,
        }
    }

    pub fn unban_target(row: &BansDialogRow) -> &str {
        &row.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bans_dialog_uses_java_empty_label() {
        let model = BansDialog::new().setup(&[]);

        assert_eq!(model.title, "@server.bans");
        assert_eq!(model.empty_label, Some("@server.bans.none"));
    }

    #[test]
    fn banned_rows_include_ip_name_and_unban_target() {
        let model =
            BansDialog::new().setup(&[BannedPlayerInfo::new("uuid", "127.0.0.1", "Griefer")]);

        assert_eq!(
            model.rows[0].label,
            "IP: [lightgray]127.0.0.1\n[]Name: [lightgray]Griefer"
        );
        assert_eq!(model.rows[0].confirm_text, "@confirmunban");
        assert_eq!(BansDialog::unban_target(&model.rows[0]), "uuid");
    }
}
