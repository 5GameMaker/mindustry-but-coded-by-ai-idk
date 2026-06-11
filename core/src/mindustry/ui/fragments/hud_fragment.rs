//! In-game HUD state model mirroring upstream `mindustry.ui.fragments.HudFragment`.

use crate::mindustry::ui::{
    core_items_display::CoreItemsDisplay, fonts::format_icon_tokens_like_java,
    upstream_menu_bundle_format_for_locale, upstream_menu_bundle_value_for_locale,
};

use super::placement_fragment::PlacementFragment;

pub const HUD_DSIZE: f32 = 65.0;
pub const HUD_PAUSE_HEIGHT: f32 = 36.0;
pub const HUD_STATUS_TABLE_WIDTH: f32 = HUD_DSIZE * 5.0 + 4.0;
pub const HUD_TOAST_INTERVAL_MILLIS: i64 = 3_500;
pub const HUD_TOAST_TEXT_WIDTH: f32 = 280.0;
pub const HUD_TOAST_MARGIN: f32 = 12.0;
pub const HUD_TOAST_TRANSLATE_DURATION: f32 = 1.0;
pub const HUD_TOAST_HOLD_DURATION: f32 = 2.5;
pub const HUD_PAUSE_DISABLED_DURATION: f32 = 60.0;
pub const HUD_UNLOCK_COLUMNS: usize = 3;
pub const HUD_UNLOCK_ICON_CAP: usize = HUD_UNLOCK_COLUMNS * HUD_UNLOCK_COLUMNS - 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudObjective {
    pub text: String,
    pub qualified: bool,
    pub hidden: bool,
}

impl HudObjective {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            qualified: true,
            hidden: false,
        }
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub fn unqualified(mut self) -> Self {
        self.qualified = false;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudRulesSnapshot {
    pub waves: bool,
    pub wave_sending: bool,
    pub wave_timer: bool,
    pub attack_mode: bool,
    pub win_wave: i32,
    pub mission: String,
    pub objectives: Vec<HudObjective>,
}

impl Default for HudRulesSnapshot {
    fn default() -> Self {
        Self {
            waves: true,
            wave_sending: true,
            wave_timer: true,
            attack_mode: false,
            win_wave: 0,
            mission: String::new(),
            objectives: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudContext {
    pub locale: String,
    pub mobile: bool,
    pub portrait: bool,
    pub state_is_menu: bool,
    pub state_is_editor: bool,
    pub state_is_campaign: bool,
    pub state_paused: bool,
    pub state_after_game_over: bool,
    pub state_game_over: bool,
    pub net_active: bool,
    pub net_server: bool,
    pub net_client: bool,
    pub net_waiting_for_players: bool,
    pub console_enabled: bool,
    pub player_admin: bool,
    pub player_dead: bool,
    pub rules_pause_disabled: bool,
    pub enemies: i32,
    pub enemy_core_count: i32,
    pub wave: i32,
    pub wavetime_ticks: i32,
    pub logic_waiting_wave: bool,
    pub spawner_spawning: bool,
    pub rules: HudRulesSnapshot,
}

impl Default for HudContext {
    fn default() -> Self {
        Self {
            locale: "en".into(),
            mobile: false,
            portrait: false,
            state_is_menu: false,
            state_is_editor: false,
            state_is_campaign: false,
            state_paused: false,
            state_after_game_over: false,
            state_game_over: false,
            net_active: false,
            net_server: false,
            net_client: false,
            net_waiting_for_players: false,
            console_enabled: false,
            player_admin: false,
            player_dead: false,
            rules_pause_disabled: false,
            enemies: 0,
            enemy_core_count: 0,
            wave: 1,
            wavetime_ticks: 0,
            logic_waiting_wave: false,
            spawner_spawning: false,
            rules: HudRulesSnapshot::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HudStatusModel {
    pub visible: bool,
    pub text: String,
    pub can_skip_wave: bool,
    pub skip_button_disabled: bool,
    pub label_pad_right: f32,
    pub table_width: f32,
    pub background: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HudOverlayModel {
    pub paused_visible: bool,
    pub paused_text: &'static str,
    pub mobile_paused_visible: bool,
    pub pause_disabled_visible: bool,
    pub pause_disabled_text: &'static str,
    pub waiting_visible: bool,
    pub waiting_text: &'static str,
    pub core_info_pause_spacer_visible: bool,
    pub pause_height: f32,
    pub side_pad: f32,
    pub hud_text: String,
    pub hud_text_requested_visible: bool,
    pub hud_text_alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudMobileButtonKind {
    Menu,
    Flip,
    Schematics,
    Pause,
    Chat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudMobileButtonAction {
    OpenPauseDialog,
    ToggleMenus,
    ToggleConsoleMobile,
    OpenSchematics,
    TogglePlayerList,
    TogglePause,
    ToggleChat,
    OpenResearch,
    OpenDatabase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudMobileButtonModel {
    pub kind: HudMobileButtonKind,
    pub icon: &'static str,
    pub action: Option<HudMobileButtonAction>,
    pub disabled: bool,
    pub force_hud_shown: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HudMobileButtonsModel {
    pub visible: bool,
    pub background: &'static str,
    pub button_size: f32,
    pub buttons: Vec<HudMobileButtonModel>,
    pub divider_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudToastAction {
    pub icon_name: String,
    pub icon_size: Option<i32>,
    pub text: String,
    pub delay_millis: i64,
    pub sound: &'static str,
    pub text_width: i32,
    pub margin: i32,
    pub translate_in_millis: i32,
    pub hold_millis: i32,
    pub translate_out_millis: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HudUnlockAction {
    NewToast {
        icon_name: String,
        delay_millis: i64,
        icons_after: Vec<String>,
        sound: &'static str,
    },
    AppendIcon {
        icon_name: String,
        icons_after: Vec<String>,
    },
    AppendMore {
        icons_after: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HudWaveSkipAction {
    LocalSkipWave,
    AdminRequestWave,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HudFragmentAction {
    ToggleMenus { shown: bool },
    Reset,
    ShowPauseDisabled { duration: f32 },
    Toast(HudToastAction),
    Unlock(HudUnlockAction),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HudFragment {
    pub blockfrag: PlacementFragment,
    pub core_items: CoreItemsDisplay,
    pub shown: bool,
    hud_text: String,
    show_hud_text: bool,
    hud_text_alpha: f32,
    last_toast_millis: i64,
    pause_disable_dur: f32,
    pause_disabled_alpha: f32,
    unlock_icons: Vec<String>,
    unlock_notification_active: bool,
}

impl Default for HudFragment {
    fn default() -> Self {
        Self::new()
    }
}

impl HudFragment {
    pub fn new() -> Self {
        Self {
            blockfrag: PlacementFragment::new(),
            core_items: CoreItemsDisplay::new(),
            shown: true,
            hud_text: String::new(),
            show_hud_text: false,
            hud_text_alpha: 0.0,
            last_toast_millis: 0,
            pause_disable_dur: 0.0,
            pause_disabled_alpha: 1.0,
            unlock_icons: Vec::new(),
            unlock_notification_active: false,
        }
    }

    pub fn hud_text(&self) -> &str {
        &self.hud_text
    }

    pub fn show_hud_text(&self) -> bool {
        self.show_hud_text
    }

    pub fn hud_text_alpha(&self) -> f32 {
        self.hud_text_alpha
    }

    pub fn last_toast_millis(&self) -> i64 {
        self.last_toast_millis
    }

    pub fn pause_disable_dur(&self) -> f32 {
        self.pause_disable_dur
    }

    pub fn pause_disabled_alpha(&self) -> f32 {
        self.pause_disabled_alpha
    }

    pub fn unlock_icons(&self) -> &[String] {
        &self.unlock_icons
    }

    pub fn set_hud_text(&mut self, text: impl Into<String>) {
        self.show_hud_text = true;
        self.hud_text = text.into();
    }

    pub fn toggle_hud_text(&mut self, shown: bool) {
        self.show_hud_text = shown;
    }

    pub fn update_hud_text_alpha(&mut self) -> f32 {
        let target = if self.show_hud_text { 1.0 } else { 0.0 };
        self.hud_text_alpha += (target - self.hud_text_alpha) * 0.2;
        self.hud_text_alpha
    }

    pub fn toggle_menus(&mut self) -> HudFragmentAction {
        self.shown = !self.shown;
        HudFragmentAction::ToggleMenus { shown: self.shown }
    }

    pub fn reset_event(&mut self) -> HudFragmentAction {
        self.core_items.reset_used();
        self.hud_text_alpha = 0.0;
        self.show_hud_text = false;
        HudFragmentAction::Reset
    }

    pub fn show_pause_disabled(&mut self) -> HudFragmentAction {
        self.pause_disable_dur = HUD_PAUSE_DISABLED_DURATION;
        self.pause_disabled_alpha = 1.0;
        HudFragmentAction::ShowPauseDisabled {
            duration: HUD_PAUSE_DISABLED_DURATION,
        }
    }

    pub fn update_pause_disabled(&mut self, delta: f32) -> f32 {
        if self.pause_disabled_alpha > 0.0 && self.pause_disable_dur > 0.0 {
            self.pause_disabled_alpha -= delta / self.pause_disable_dur;
        } else {
            self.pause_disabled_alpha = 1.0;
        }

        if self.pause_disabled_alpha <= 0.0 {
            self.pause_disabled_alpha = 0.0;
            self.pause_disable_dur = 0.0;
        }

        self.pause_disabled_alpha
    }

    pub fn has_toast(&self, now_millis: i64) -> bool {
        now_millis - self.last_toast_millis < HUD_TOAST_INTERVAL_MILLIS
    }

    pub fn show_toast_at(
        &mut self,
        now_millis: i64,
        state_is_menu: bool,
        text: impl Into<String>,
    ) -> Option<HudFragmentAction> {
        self.show_toast_with_icon_at(now_millis, state_is_menu, "ok", None, text)
    }

    pub fn show_toast_with_icon_at(
        &mut self,
        now_millis: i64,
        state_is_menu: bool,
        icon_name: impl Into<String>,
        icon_size: Option<i32>,
        text: impl Into<String>,
    ) -> Option<HudFragmentAction> {
        if state_is_menu {
            return None;
        }

        let delay_millis = self.schedule_toast(now_millis);
        Some(HudFragmentAction::Toast(HudToastAction {
            icon_name: icon_name.into(),
            icon_size,
            text: text.into(),
            delay_millis,
            sound: "uiNotify",
            text_width: HUD_TOAST_TEXT_WIDTH as i32,
            margin: HUD_TOAST_MARGIN as i32,
            translate_in_millis: (HUD_TOAST_TRANSLATE_DURATION * 1000.0) as i32,
            hold_millis: (HUD_TOAST_HOLD_DURATION * 1000.0) as i32,
            translate_out_millis: (HUD_TOAST_TRANSLATE_DURATION * 1000.0) as i32,
        }))
    }

    pub fn show_unlock_at(
        &mut self,
        now_millis: i64,
        state_is_menu: bool,
        icon_name: impl Into<String>,
    ) -> Option<HudFragmentAction> {
        if state_is_menu {
            return None;
        }

        let icon_name = icon_name.into();
        if !self.unlock_notification_active {
            let delay_millis = self.schedule_toast(now_millis);
            self.unlock_notification_active = true;
            self.unlock_icons.clear();
            self.unlock_icons.push(icon_name.clone());
            return Some(HudFragmentAction::Unlock(HudUnlockAction::NewToast {
                icon_name,
                delay_millis,
                icons_after: self.unlock_icons.clone(),
                sound: "uiNotify",
            }));
        }

        let esize = self.unlock_icons.len();
        if esize > HUD_UNLOCK_ICON_CAP {
            return None;
        }

        if esize < HUD_UNLOCK_ICON_CAP {
            self.unlock_icons.push(icon_name.clone());
            Some(HudFragmentAction::Unlock(HudUnlockAction::AppendIcon {
                icon_name,
                icons_after: self.unlock_icons.clone(),
            }))
        } else {
            self.unlock_icons.push("+".into());
            Some(HudFragmentAction::Unlock(HudUnlockAction::AppendMore {
                icons_after: self.unlock_icons.clone(),
            }))
        }
    }

    pub fn clear_unlock_notification(&mut self) {
        self.unlock_notification_active = false;
        self.unlock_icons.clear();
    }

    pub fn status_model(&self, context: &HudContext) -> HudStatusModel {
        let can_skip_wave = can_skip_wave(context);
        HudStatusModel {
            visible: self.shown && !context.state_is_editor,
            text: status_text(context),
            can_skip_wave,
            skip_button_disabled: !can_skip_wave,
            label_pad_right: if can_skip_wave { 8.0 } else { -42.0 },
            table_width: HUD_STATUS_TABLE_WIDTH,
            background: "wavepane",
        }
    }

    pub fn overlay_model(&self, context: &HudContext) -> HudOverlayModel {
        HudOverlayModel {
            paused_visible: context.state_paused
                && self.shown
                && !context.net_waiting_for_players
                && !(context.mobile && context.portrait),
            paused_text: if context.state_game_over && context.state_is_campaign {
                "@sector.curlost"
            } else {
                "@paused"
            },
            mobile_paused_visible: context.state_paused
                && self.shown
                && !context.net_waiting_for_players
                && context.portrait,
            pause_disabled_visible: self.pause_disable_dur > 0.0
                && self.shown
                && !context.mobile
                && !context.net_waiting_for_players
                && !context.state_paused
                && !(context.state_game_over && context.state_is_campaign),
            pause_disabled_text: "@pause.disabled",
            waiting_visible: context.net_waiting_for_players && context.state_paused && self.shown,
            waiting_text: "@waiting.players",
            core_info_pause_spacer_visible: !context.net_waiting_for_players
                && (context.state_paused || self.pause_disable_dur > 0.0),
            pause_height: HUD_PAUSE_HEIGHT,
            side_pad: HUD_DSIZE * 5.0 + 4.0,
            hud_text: self.hud_text.clone(),
            hud_text_requested_visible: self.show_hud_text,
            hud_text_alpha: self.hud_text_alpha,
        }
    }

    pub fn mobile_buttons_model(&self, context: &HudContext) -> HudMobileButtonsModel {
        let flip = if context.console_enabled {
            HudMobileButtonModel {
                kind: HudMobileButtonKind::Flip,
                icon: "terminal",
                action: Some(HudMobileButtonAction::ToggleConsoleMobile),
                disabled: false,
                force_hud_shown: true,
            }
        } else {
            HudMobileButtonModel {
                kind: HudMobileButtonKind::Flip,
                icon: if self.shown { "downOpen" } else { "upOpen" },
                action: Some(HudMobileButtonAction::ToggleMenus),
                disabled: false,
                force_hud_shown: false,
            }
        };

        let pause_disabled = !context.net_active
            && (context.rules_pause_disabled
                || (context.state_is_campaign && context.state_after_game_over));
        let pause = if context.net_active {
            HudMobileButtonModel {
                kind: HudMobileButtonKind::Pause,
                icon: "players",
                action: Some(HudMobileButtonAction::TogglePlayerList),
                disabled: false,
                force_hud_shown: false,
            }
        } else {
            HudMobileButtonModel {
                kind: HudMobileButtonKind::Pause,
                icon: if context.state_paused {
                    "play"
                } else {
                    "pause"
                },
                action: (!pause_disabled).then_some(HudMobileButtonAction::TogglePause),
                disabled: pause_disabled,
                force_hud_shown: false,
            }
        };

        let chat = if context.net_active && context.mobile {
            HudMobileButtonModel {
                kind: HudMobileButtonKind::Chat,
                icon: "chat",
                action: Some(HudMobileButtonAction::ToggleChat),
                disabled: false,
                force_hud_shown: false,
            }
        } else if context.state_is_campaign {
            HudMobileButtonModel {
                kind: HudMobileButtonKind::Chat,
                icon: "tree",
                action: Some(HudMobileButtonAction::OpenResearch),
                disabled: false,
                force_hud_shown: false,
            }
        } else {
            HudMobileButtonModel {
                kind: HudMobileButtonKind::Chat,
                icon: "book",
                action: Some(HudMobileButtonAction::OpenDatabase),
                disabled: false,
                force_hud_shown: false,
            }
        };

        HudMobileButtonsModel {
            visible: context.mobile,
            background: "black6",
            button_size: HUD_DSIZE,
            buttons: vec![
                HudMobileButtonModel {
                    kind: HudMobileButtonKind::Menu,
                    icon: "menu",
                    action: Some(HudMobileButtonAction::OpenPauseDialog),
                    disabled: false,
                    force_hud_shown: false,
                },
                flip,
                HudMobileButtonModel {
                    kind: HudMobileButtonKind::Schematics,
                    icon: "paste",
                    action: Some(HudMobileButtonAction::OpenSchematics),
                    disabled: false,
                    force_hud_shown: false,
                },
                pause,
                chat,
            ],
            divider_visible: true,
        }
    }

    pub fn skip_wave_action(
        &self,
        context: &HudContext,
        scene_has_dialog: bool,
        scene_has_field: bool,
    ) -> Option<HudWaveSkipAction> {
        if !can_skip_wave(context) || scene_has_dialog || scene_has_field || context.player_dead {
            return None;
        }

        if context.net_client && context.player_admin {
            Some(HudWaveSkipAction::AdminRequestWave)
        } else {
            Some(HudWaveSkipAction::LocalSkipWave)
        }
    }

    fn schedule_toast(&mut self, now_millis: i64) -> i64 {
        let since = now_millis - self.last_toast_millis;
        if since > HUD_TOAST_INTERVAL_MILLIS {
            self.last_toast_millis = now_millis;
            0
        } else {
            self.last_toast_millis += HUD_TOAST_INTERVAL_MILLIS;
            HUD_TOAST_INTERVAL_MILLIS - since
        }
    }
}

pub fn can_skip_wave(context: &HudContext) -> bool {
    context.rules.waves
        && context.rules.wave_sending
        && ((context.net_server || context.player_admin) || !context.net_active)
        && context.enemies == 0
        && !context.spawner_spawning
}

pub fn status_text(context: &HudContext) -> String {
    if !context.rules.mission.is_empty() {
        return context.rules.mission.clone();
    }

    if !context.rules.objectives.is_empty() {
        let mut builder = String::new();
        let mut first = true;
        for objective in &context.rules.objectives {
            if !objective.qualified || objective.hidden || objective.text.is_empty() {
                continue;
            }

            if !first {
                builder.push_str("\n[white]");
            }
            builder.push_str(&format_icon_tokens_like_java(&objective.text));
            first = false;
        }

        if !builder.is_empty() {
            return builder;
        }
    }

    if !context.rules.waves && context.rules.attack_mode {
        let sum = context.enemy_core_count.max(1);
        let key = if sum > 1 {
            "wave.enemycores"
        } else {
            "wave.enemycore"
        };
        return bundle_format(&context.locale, key, &[&sum.to_string()]);
    }

    if context.state_after_game_over && context.state_is_campaign {
        return String::new();
    }

    let mut builder = String::new();
    if !context.rules.waves && context.state_is_campaign {
        builder.push_str("[lightgray]");
        builder.push_str(&bundle_value(&context.locale, "sector.curcapture"));
    }

    if !context.rules.waves {
        return builder;
    }

    if context.rules.win_wave > 1 && context.rules.win_wave >= context.wave {
        builder.push_str(&bundle_format(
            &context.locale,
            "wave.cap",
            &[
                &context.wave.to_string(),
                &context.rules.win_wave.to_string(),
            ],
        ));
    } else {
        builder.push_str(&bundle_format(
            &context.locale,
            "wave",
            &[&context.wave.to_string()],
        ));
    }
    builder.push('\n');

    if context.enemies > 0 {
        let key = if context.enemies == 1 {
            "wave.enemy"
        } else {
            "wave.enemies"
        };
        builder.push_str(&bundle_format(
            &context.locale,
            key,
            &[&context.enemies.to_string()],
        ));
        builder.push('\n');
    }

    if context.rules.wave_timer {
        if context.logic_waiting_wave {
            builder.push_str(&bundle_value(&context.locale, "wave.waveInProgress"));
        } else {
            let seconds = context.wavetime_ticks / 60;
            builder.push_str(&bundle_format(
                &context.locale,
                "wave.waiting",
                &[&format_waiting_seconds(seconds)],
            ));
        }
    } else if context.enemies == 0 {
        builder.push_str(&bundle_value(&context.locale, "waiting"));
    }

    builder
}

pub fn format_waiting_seconds(seconds: i32) -> String {
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    if minutes > 0 {
        format!("{minutes}:{seconds:02}")
    } else {
        seconds.to_string()
    }
}

fn bundle_value(locale: &str, key: &str) -> String {
    upstream_menu_bundle_value_for_locale(locale, key)
        .unwrap_or(key)
        .to_string()
}

fn bundle_format(locale: &str, key: &str, args: &[&str]) -> String {
    upstream_menu_bundle_format_for_locale(locale, key, args)
        .unwrap_or_else(|| replace_placeholders(key, args))
}

fn replace_placeholders(text: &str, args: &[&str]) -> String {
    let mut value = text.to_string();
    for (index, arg) in args.iter().enumerate() {
        value = value.replace(&format!("{{{index}}}"), arg);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::{content::items, r#type::Item, r#type::ItemSeq};

    fn context() -> HudContext {
        HudContext::default()
    }

    fn mobile_button(
        model: &HudMobileButtonsModel,
        kind: HudMobileButtonKind,
    ) -> &HudMobileButtonModel {
        model
            .buttons
            .iter()
            .find(|button| button.kind == kind)
            .expect("mobile button should exist")
    }

    #[test]
    fn can_skip_wave_matches_java_gates() {
        let mut ctx = context();
        assert!(can_skip_wave(&ctx));

        ctx.enemies = 1;
        assert!(!can_skip_wave(&ctx));
        ctx.enemies = 0;

        ctx.spawner_spawning = true;
        assert!(!can_skip_wave(&ctx));
        ctx.spawner_spawning = false;

        ctx.rules.wave_sending = false;
        assert!(!can_skip_wave(&ctx));
        ctx.rules.wave_sending = true;

        ctx.net_active = true;
        assert!(!can_skip_wave(&ctx));
        ctx.player_admin = true;
        assert!(can_skip_wave(&ctx));
        ctx.player_admin = false;
        ctx.net_server = true;
        assert!(can_skip_wave(&ctx));
    }

    #[test]
    fn status_text_prioritizes_mission_then_visible_objectives() {
        let mut ctx = context();
        ctx.rules.mission = "Destroy the enemy core".into();
        ctx.rules.objectives = vec![HudObjective::new("ignored")];
        assert_eq!(status_text(&ctx), "Destroy the enemy core");

        ctx.rules.mission.clear();
        ctx.rules.objectives = vec![
            HudObjective::new("hidden").hidden(),
            HudObjective::new("late").unqualified(),
            HudObjective::new("Read :warning:"),
            HudObjective::new("Launch"),
        ];

        assert_eq!(status_text(&ctx), "Read \u{26a0}\n[white]Launch");
    }

    #[test]
    fn status_text_matches_attack_campaign_and_wave_timer_branches() {
        let mut ctx = context();
        ctx.rules.waves = false;
        ctx.rules.attack_mode = true;
        ctx.enemy_core_count = 3;
        assert_eq!(
            status_text(&ctx),
            "[accent]3[lightgray] Enemy Cores".to_string()
        );

        ctx.rules.attack_mode = false;
        ctx.state_is_campaign = true;
        assert_eq!(status_text(&ctx), "[lightgray]Sector Captured");

        ctx.rules.waves = true;
        ctx.state_is_campaign = false;
        ctx.rules.win_wave = 10;
        ctx.wave = 3;
        ctx.enemies = 1;
        ctx.wavetime_ticks = 65 * 60;

        assert_eq!(
            status_text(&ctx),
            "[accent]Wave 3/10\n[lightgray]1 Enemy Remaining\n[lightgray]Wave in 1:05"
        );

        ctx.logic_waiting_wave = true;
        ctx.enemies = 0;
        assert_eq!(
            status_text(&ctx),
            "[accent]Wave 3/10\n[lightgray]Wave in progress"
        );
    }

    #[test]
    fn toast_schedule_uses_java_3500ms_spacing_and_menu_suppression() {
        let mut hud = HudFragment::new();
        let first = hud.show_toast_at(10_000, false, "first");
        assert_eq!(
            first,
            Some(HudFragmentAction::Toast(HudToastAction {
                icon_name: "ok".into(),
                icon_size: None,
                text: "first".into(),
                delay_millis: 0,
                sound: "uiNotify",
                text_width: 280,
                margin: 12,
                translate_in_millis: 1000,
                hold_millis: 2500,
                translate_out_millis: 1000,
            }))
        );
        assert!(hud.has_toast(12_000));

        let second = hud.show_toast_at(11_000, false, "second");
        assert!(matches!(
            second,
            Some(HudFragmentAction::Toast(HudToastAction {
                delay_millis: 2_500,
                ..
            }))
        ));
        assert_eq!(hud.last_toast_millis(), 13_500);

        assert_eq!(hud.show_toast_at(12_000, true, "menu"), None);
    }

    #[test]
    fn pause_disabled_banner_decays_from_sixty_frames() {
        let mut hud = HudFragment::new();
        let mut ctx = context();
        hud.show_pause_disabled();
        let overlay = hud.overlay_model(&ctx);
        assert!(overlay.pause_disabled_visible);
        assert_eq!(overlay.pause_disabled_text, "@pause.disabled");
        assert_eq!(overlay.pause_height, 36.0);

        assert_eq!(hud.update_pause_disabled(30.0), 0.5);
        assert_eq!(hud.update_pause_disabled(30.0), 0.0);
        assert_eq!(hud.pause_disable_dur(), 0.0);

        hud.show_pause_disabled();
        ctx.state_paused = true;
        assert!(!hud.overlay_model(&ctx).pause_disabled_visible);
        assert!(hud.overlay_model(&ctx).paused_visible);
    }

    #[test]
    fn mobile_buttons_pause_opens_player_list_when_network_active_like_java() {
        let hud = HudFragment::new();
        let mut ctx = context();
        ctx.mobile = true;
        ctx.net_active = true;
        ctx.rules_pause_disabled = true;
        ctx.state_is_campaign = true;
        ctx.state_after_game_over = true;

        let model = hud.mobile_buttons_model(&ctx);
        assert!(model.visible);
        assert_eq!(model.background, "black6");
        assert_eq!(model.button_size, HUD_DSIZE);
        assert!(model.divider_visible);
        assert_eq!(
            model
                .buttons
                .iter()
                .map(|button| button.kind)
                .collect::<Vec<_>>(),
            vec![
                HudMobileButtonKind::Menu,
                HudMobileButtonKind::Flip,
                HudMobileButtonKind::Schematics,
                HudMobileButtonKind::Pause,
                HudMobileButtonKind::Chat,
            ]
        );

        let pause = mobile_button(&model, HudMobileButtonKind::Pause);
        assert_eq!(pause.icon, "players");
        assert_eq!(pause.action, Some(HudMobileButtonAction::TogglePlayerList));
        assert!(!pause.disabled);
    }

    #[test]
    fn mobile_buttons_pause_toggles_game_pause_when_offline_like_java() {
        let hud = HudFragment::new();
        let mut ctx = context();
        ctx.mobile = true;
        ctx.state_paused = false;

        let model = hud.mobile_buttons_model(&ctx);
        let pause = mobile_button(&model, HudMobileButtonKind::Pause);
        assert_eq!(pause.icon, "pause");
        assert_eq!(pause.action, Some(HudMobileButtonAction::TogglePause));
        assert!(!pause.disabled);

        ctx.state_paused = true;
        let model = hud.mobile_buttons_model(&ctx);
        let pause = mobile_button(&model, HudMobileButtonKind::Pause);
        assert_eq!(pause.icon, "play");
        assert_eq!(pause.action, Some(HudMobileButtonAction::TogglePause));

        ctx.state_paused = false;
        ctx.rules_pause_disabled = true;
        let model = hud.mobile_buttons_model(&ctx);
        let pause = mobile_button(&model, HudMobileButtonKind::Pause);
        assert_eq!(pause.icon, "pause");
        assert_eq!(pause.action, None);
        assert!(pause.disabled);
    }

    #[test]
    fn mobile_buttons_flip_and_chat_icons_follow_java_update_branches() {
        let mut hud = HudFragment::new();
        let mut ctx = context();
        ctx.mobile = true;

        let model = hud.mobile_buttons_model(&ctx);
        let flip = mobile_button(&model, HudMobileButtonKind::Flip);
        assert_eq!(flip.icon, "downOpen");
        assert_eq!(flip.action, Some(HudMobileButtonAction::ToggleMenus));
        assert!(!flip.force_hud_shown);
        let chat = mobile_button(&model, HudMobileButtonKind::Chat);
        assert_eq!(chat.icon, "book");
        assert_eq!(chat.action, Some(HudMobileButtonAction::OpenDatabase));

        hud.toggle_menus();
        let model = hud.mobile_buttons_model(&ctx);
        assert_eq!(
            mobile_button(&model, HudMobileButtonKind::Flip).icon,
            "upOpen"
        );

        ctx.console_enabled = true;
        let model = hud.mobile_buttons_model(&ctx);
        let flip = mobile_button(&model, HudMobileButtonKind::Flip);
        assert_eq!(flip.icon, "terminal");
        assert_eq!(
            flip.action,
            Some(HudMobileButtonAction::ToggleConsoleMobile)
        );
        assert!(flip.force_hud_shown);

        ctx.console_enabled = false;
        ctx.state_is_campaign = true;
        let model = hud.mobile_buttons_model(&ctx);
        let chat = mobile_button(&model, HudMobileButtonKind::Chat);
        assert_eq!(chat.icon, "tree");
        assert_eq!(chat.action, Some(HudMobileButtonAction::OpenResearch));

        ctx.net_active = true;
        let model = hud.mobile_buttons_model(&ctx);
        let chat = mobile_button(&model, HudMobileButtonKind::Chat);
        assert_eq!(chat.icon, "chat");
        assert_eq!(chat.action, Some(HudMobileButtonAction::ToggleChat));
    }

    #[test]
    fn reset_event_clears_hud_text_and_core_items_like_reset_event() {
        let content_items = items::load();
        let names = content_items.iter().map(Item::name).collect::<Vec<_>>();
        let mut core_items = ItemSeq::new(names);
        core_items.set(1, 1);

        let mut hud = HudFragment::new();
        hud.core_items
            .update_from_core_items(&content_items, &core_items);
        hud.set_hud_text("hello");
        hud.update_hud_text_alpha();

        assert!(hud.show_hud_text());
        assert!(hud.hud_text_alpha() > 0.0);
        assert!(!hud.core_items.used_items().is_empty());

        assert_eq!(hud.reset_event(), HudFragmentAction::Reset);
        assert!(!hud.show_hud_text());
        assert_eq!(hud.hud_text_alpha(), 0.0);
        assert!(hud.core_items.used_items().is_empty());
    }

    #[test]
    fn unlock_notification_stacks_icons_until_java_plus_cap() {
        let mut hud = HudFragment::new();
        assert_eq!(
            hud.show_unlock_at(10_000, false, "duo"),
            Some(HudFragmentAction::Unlock(HudUnlockAction::NewToast {
                icon_name: "duo".into(),
                delay_millis: 0,
                icons_after: vec!["duo".into()],
                sound: "uiNotify",
            }))
        );

        for index in 0..7 {
            assert!(matches!(
                hud.show_unlock_at(10_010 + index, false, format!("block-{index}")),
                Some(HudFragmentAction::Unlock(
                    HudUnlockAction::AppendIcon { .. }
                ))
            ));
        }
        assert_eq!(hud.unlock_icons().len(), HUD_UNLOCK_ICON_CAP);

        assert_eq!(
            hud.show_unlock_at(10_100, false, "overflow"),
            Some(HudFragmentAction::Unlock(HudUnlockAction::AppendMore {
                icons_after: vec![
                    "duo".into(),
                    "block-0".into(),
                    "block-1".into(),
                    "block-2".into(),
                    "block-3".into(),
                    "block-4".into(),
                    "block-5".into(),
                    "block-6".into(),
                    "+".into(),
                ],
            }))
        );
        assert_eq!(hud.show_unlock_at(10_200, false, "ignored"), None);
    }

    #[test]
    fn skip_wave_action_uses_admin_request_only_for_admin_client() {
        let hud = HudFragment::new();
        let mut ctx = context();
        assert_eq!(
            hud.skip_wave_action(&ctx, false, false),
            Some(HudWaveSkipAction::LocalSkipWave)
        );

        ctx.net_active = true;
        ctx.net_client = true;
        ctx.player_admin = true;
        assert_eq!(
            hud.skip_wave_action(&ctx, false, false),
            Some(HudWaveSkipAction::AdminRequestWave)
        );

        assert_eq!(hud.skip_wave_action(&ctx, true, false), None);
        ctx.player_dead = true;
        assert_eq!(hud.skip_wave_action(&ctx, false, false), None);
    }
}
