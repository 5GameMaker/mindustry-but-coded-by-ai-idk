//! Campaign completion dialog model mirroring upstream `CampaignCompleteDialog`.

use crate::mindustry::ui::upstream_menu_bundle_format_for_locale;

pub const CAMPAIGN_COMPLETE_MENU_BUTTON_SIZE: (f32, f32) = (210.0, 64.0);
pub const CAMPAIGN_COMPLETE_FADE_DURATION: f32 = 1.1;
pub const CAMPAIGN_COMPLETE_TRANSLATE_DURATION: f32 = 6.0;

#[derive(Debug, Clone, PartialEq)]
pub struct CampaignCompletePlanet {
    pub localized_name: String,
    pub icon_color: String,
    pub sector_time_played_millis: Vec<f32>,
}

impl CampaignCompletePlanet {
    pub fn new(
        localized_name: impl Into<String>,
        icon_color: impl Into<String>,
        sector_time_played_millis: Vec<f32>,
    ) -> Self {
        Self {
            localized_name: localized_name.into(),
            icon_color: icon_color.into(),
            sector_time_played_millis,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CampaignCompleteDialogModel {
    pub should_pause: bool,
    pub planet_line: String,
    pub playtime_line: String,
    pub initial_translation_y: f32,
    pub initial_alpha: f32,
    pub fade_duration: f32,
    pub translate_duration: f32,
    pub button_size: (f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CampaignCompleteDialog;

impl CampaignCompleteDialog {
    pub fn new() -> Self {
        Self
    }

    pub fn show(
        &self,
        planet: &CampaignCompletePlanet,
        graphics_height: f32,
    ) -> CampaignCompleteDialogModel {
        let planet_name = format!("[#{}]{}[]", planet.icon_color, planet.localized_name);
        let planet_line = upstream_menu_bundle_format_for_locale(
            "en",
            "campaign.complete",
            &[planet_name.as_str()],
        )
        .unwrap_or(planet_name);
        let playtime = planet.sector_time_played_millis.iter().sum::<f32>() / 1000.0;
        let formatted_time = format_time(playtime);
        let playtime_line = upstream_menu_bundle_format_for_locale(
            "en",
            "campaign.playtime",
            &[formatted_time.as_str()],
        )
        .unwrap_or(formatted_time);

        CampaignCompleteDialogModel {
            should_pause: true,
            planet_line,
            playtime_line,
            initial_translation_y: -graphics_height,
            initial_alpha: 0.0,
            fade_duration: CAMPAIGN_COMPLETE_FADE_DURATION,
            translate_duration: CAMPAIGN_COMPLETE_TRANSLATE_DURATION,
            button_size: CAMPAIGN_COMPLETE_MENU_BUTTON_SIZE,
        }
    }
}

pub fn format_time(ticks: f32) -> String {
    let seconds = (ticks / 60.0) as i32;
    if seconds < 60 {
        return format!("0:{:02}", seconds);
    }

    let minutes = seconds / 60;
    let mod_sec = seconds % 60;
    if minutes < 60 {
        return format!("{minutes}:{mod_sec:02}");
    }

    let hours = minutes / 60;
    let mod_minute = minutes % 60;
    format!("{hours}:{mod_minute:02}:{mod_sec:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campaign_complete_model_uses_java_translation_and_button_metrics() {
        let planet = CampaignCompletePlanet::new("Serpulo", "89f5ff", vec![120_000.0]);
        let model = CampaignCompleteDialog::new().show(&planet, 720.0);

        assert!(model.should_pause);
        assert_eq!(model.initial_translation_y, -720.0);
        assert_eq!(model.initial_alpha, 0.0);
        assert_eq!(model.fade_duration, 1.1);
        assert_eq!(model.translate_duration, 6.0);
        assert_eq!(model.button_size, (210.0, 64.0));
        assert!(model.planet_line.contains("[#89f5ff]Serpulo[]"));
    }

    #[test]
    fn format_time_matches_ui_format_time() {
        assert_eq!(format_time(59.0 * 60.0), "0:59");
        assert_eq!(format_time(61.0 * 60.0), "1:01");
        assert_eq!(format_time(3661.0 * 60.0), "1:01:01");
    }
}
