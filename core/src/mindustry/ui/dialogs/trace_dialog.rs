//! Player trace dialog model mirroring upstream `mindustry.ui.dialogs.TraceDialog`.

use crate::mindustry::ui::upstream_menu_bundle_format_for_locale;

pub const TRACE_DIALOG_TITLE: &str = "@trace";
pub const TRACE_COPY_BUTTON_SIZE: f32 = 28.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceInfo {
    pub ip: String,
    pub locale: String,
    pub uuid: String,
    pub modded: bool,
    pub mobile: bool,
    pub times_joined: i32,
    pub times_kicked: i32,
    pub ips: Vec<String>,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCopyRow {
    pub copy_value: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceDialogModel {
    pub title: &'static str,
    pub copy_button_size: i32,
    pub copy_rows: Vec<TraceCopyRow>,
    pub summary_rows: Vec<String>,
    pub ips: Vec<String>,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TraceDialog;

impl TraceDialog {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&self, player_name: &str, info: &TraceInfo) -> TraceDialogModel {
        let copy_rows = vec![
            TraceCopyRow {
                copy_value: player_name.to_string(),
                label: bundle_format("trace.playername", &[player_name]),
            },
            TraceCopyRow {
                copy_value: info.ip.clone(),
                label: bundle_format("trace.ip", &[info.ip.as_str()]),
            },
            TraceCopyRow {
                copy_value: info.locale.clone(),
                label: bundle_format("trace.language", &[info.locale.as_str()]),
            },
            TraceCopyRow {
                copy_value: info.uuid.clone(),
                label: bundle_format("trace.id", &[info.uuid.as_str()]),
            },
        ];

        TraceDialogModel {
            title: TRACE_DIALOG_TITLE,
            copy_button_size: TRACE_COPY_BUTTON_SIZE as i32,
            copy_rows,
            summary_rows: vec![
                bundle_format("trace.modclient", &[&info.modded.to_string()]),
                bundle_format("trace.mobile", &[&info.mobile.to_string()]),
                bundle_format("trace.times.joined", &[&info.times_joined.to_string()]),
                bundle_format("trace.times.kicked", &[&info.times_kicked.to_string()]),
            ],
            ips: info
                .ips
                .iter()
                .map(|value| format!("[lightgray]{value}"))
                .collect(),
            names: info
                .names
                .iter()
                .map(|value| format!("[lightgray]{value}"))
                .collect(),
        }
    }

    pub fn copy_feedback_key() -> &'static str {
        "@copied"
    }
}

fn bundle_format(key: &str, args: &[&str]) -> String {
    upstream_menu_bundle_format_for_locale("en", key, args).unwrap_or_else(|| {
        let mut value = key.to_string();
        for (index, arg) in args.iter().enumerate() {
            value = value.replace(&format!("{{{index}}}"), arg);
        }
        value
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info() -> TraceInfo {
        TraceInfo {
            ip: "127.0.0.1".into(),
            locale: "en_US".into(),
            uuid: "uuid".into(),
            modded: true,
            mobile: false,
            times_joined: 3,
            times_kicked: 1,
            ips: vec!["127.0.0.1".into()],
            names: vec!["Player".into()],
        }
    }

    #[test]
    fn trace_dialog_builds_copy_rows_and_lists_like_java() {
        let model = TraceDialog::new().show("Player", &info());

        assert_eq!(model.title, "@trace");
        assert_eq!(model.copy_button_size, 28);
        assert_eq!(model.copy_rows[0].copy_value, "Player");
        assert_eq!(model.copy_rows[1].copy_value, "127.0.0.1");
        assert_eq!(model.ips, vec!["[lightgray]127.0.0.1"]);
        assert_eq!(model.names, vec!["[lightgray]Player"]);
        assert_eq!(TraceDialog::copy_feedback_key(), "@copied");
    }
}
