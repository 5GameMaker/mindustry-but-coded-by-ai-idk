//! In-game hints state model mirroring upstream `mindustry.ui.fragments.HintsFragment`.

use std::collections::HashSet;

use crate::mindustry::ui::upstream_menu_bundle_value_for_locale;

pub const HINT_VISIBLE_DESKTOP: u8 = 1;
pub const HINT_VISIBLE_MOBILE: u8 = 2;
pub const HINT_VISIBLE_ALL: u8 = HINT_VISIBLE_DESKTOP | HINT_VISIBLE_MOBILE;
pub const HINT_FADE_OUT_TIME: f32 = 0.6;
pub const HINT_DESKTOP_WIDTH: f32 = 400.0;
pub const HINT_MOBILE_WIDTH: f32 = 270.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefaultHint {
    DesktopMove,
    Zoom,
    Breaking,
    DesktopShoot,
    DepositItems,
    DesktopPause,
    UnitControl,
    UnitSelectControl,
    Respawn,
    Launch,
    SchematicSelect,
    ConveyorPathfind,
    Boost,
    BlockInfo,
    Derelict,
    PayloadPickup,
    PayloadDrop,
    WaveFire,
    RebuildSelect,
    Guardian,
    CannotUpgrade,
    FactoryControl,
    CoreUpgrade,
    SerpuloCoreZone,
    PresetLaunch,
    PresetDifficulty,
    CoreIncinerate,
}

impl DefaultHint {
    pub const ALL: [DefaultHint; 27] = [
        DefaultHint::DesktopMove,
        DefaultHint::Zoom,
        DefaultHint::Breaking,
        DefaultHint::DesktopShoot,
        DefaultHint::DepositItems,
        DefaultHint::DesktopPause,
        DefaultHint::UnitControl,
        DefaultHint::UnitSelectControl,
        DefaultHint::Respawn,
        DefaultHint::Launch,
        DefaultHint::SchematicSelect,
        DefaultHint::ConveyorPathfind,
        DefaultHint::Boost,
        DefaultHint::BlockInfo,
        DefaultHint::Derelict,
        DefaultHint::PayloadPickup,
        DefaultHint::PayloadDrop,
        DefaultHint::WaveFire,
        DefaultHint::RebuildSelect,
        DefaultHint::Guardian,
        DefaultHint::CannotUpgrade,
        DefaultHint::FactoryControl,
        DefaultHint::CoreUpgrade,
        DefaultHint::SerpuloCoreZone,
        DefaultHint::PresetLaunch,
        DefaultHint::PresetDifficulty,
        DefaultHint::CoreIncinerate,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            DefaultHint::DesktopMove => "desktopMove",
            DefaultHint::Zoom => "zoom",
            DefaultHint::Breaking => "breaking",
            DefaultHint::DesktopShoot => "desktopShoot",
            DefaultHint::DepositItems => "depositItems",
            DefaultHint::DesktopPause => "desktopPause",
            DefaultHint::UnitControl => "unitControl",
            DefaultHint::UnitSelectControl => "unitSelectControl",
            DefaultHint::Respawn => "respawn",
            DefaultHint::Launch => "launch",
            DefaultHint::SchematicSelect => "schematicSelect",
            DefaultHint::ConveyorPathfind => "conveyorPathfind",
            DefaultHint::Boost => "boost",
            DefaultHint::BlockInfo => "blockInfo",
            DefaultHint::Derelict => "derelict",
            DefaultHint::PayloadPickup => "payloadPickup",
            DefaultHint::PayloadDrop => "payloadDrop",
            DefaultHint::WaveFire => "waveFire",
            DefaultHint::RebuildSelect => "rebuildSelect",
            DefaultHint::Guardian => "guardian",
            DefaultHint::CannotUpgrade => "cannotUpgrade",
            DefaultHint::FactoryControl => "factoryControl",
            DefaultHint::CoreUpgrade => "coreUpgrade",
            DefaultHint::SerpuloCoreZone => "serpuloCoreZone",
            DefaultHint::PresetLaunch => "presetLaunch",
            DefaultHint::PresetDifficulty => "presetDifficulty",
            DefaultHint::CoreIncinerate => "coreIncinerate",
        }
    }

    pub const fn order(self) -> i32 {
        self as i32
    }

    pub const fn visibility(self) -> u8 {
        match self {
            DefaultHint::DesktopMove
            | DefaultHint::Zoom
            | DefaultHint::DesktopShoot
            | DefaultHint::DesktopPause
            | DefaultHint::SchematicSelect
            | DefaultHint::Boost => HINT_VISIBLE_DESKTOP,
            DefaultHint::Respawn => HINT_VISIBLE_MOBILE,
            _ => HINT_VISIBLE_ALL,
        }
    }

    pub fn valid(self, mobile: bool) -> bool {
        (mobile && (self.visibility() & HINT_VISIBLE_MOBILE) != 0)
            || (!mobile && (self.visibility() & HINT_VISIBLE_DESKTOP) != 0)
    }

    pub fn text(self, mobile: bool, locale: &str) -> String {
        let mobile_key = format!("hint.{}.mobile", self.name());
        let key = format!("hint.{}", self.name());
        let mut text = if mobile {
            upstream_menu_bundle_value_for_locale(locale, &mobile_key)
                .or_else(|| upstream_menu_bundle_value_for_locale(locale, &key))
        } else {
            upstream_menu_bundle_value_for_locale(locale, &key)
        }
        .unwrap_or(&key)
        .to_string();
        if !mobile {
            text = text.replace("tap", "click").replace("Tap", "Click");
        }
        text
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HintState {
    pub name: String,
    pub text: String,
    pub order: i32,
    pub visibility: u8,
    pub dependencies: Vec<String>,
    pub finished: bool,
    pub show: bool,
    pub complete: bool,
}

impl HintState {
    pub fn from_default(hint: DefaultHint, mobile: bool, locale: &str) -> Self {
        Self {
            name: hint.name().into(),
            text: hint.text(mobile, locale),
            order: hint.order(),
            visibility: hint.visibility(),
            dependencies: Vec::new(),
            finished: false,
            show: false,
            complete: false,
        }
    }

    pub fn valid(&self, mobile: bool) -> bool {
        (mobile && (self.visibility & HINT_VISIBLE_MOBILE) != 0)
            || (!mobile && (self.visibility & HINT_VISIBLE_DESKTOP) != 0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HintsUpdateContext {
    pub mobile: bool,
    pub hints_enabled: bool,
    pub hud_shown: bool,
    pub renderer_cutscene: bool,
    pub state_is_game: bool,
    pub total_playtime: i64,
    pub derelict_under_mouse: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HintsAction {
    Display {
        name: String,
        text: String,
        width: f32,
    },
    Complete(String),
    FinishSetting(String),
    Hide {
        fade_out: f32,
        translate_y: f32,
    },
    RemoveHint(String),
    EventAdded(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HintsFragment {
    hints: Vec<HintState>,
    current: Option<String>,
    events: HashSet<String>,
    placed_blocks: HashSet<String>,
    last_present: bool,
}

impl HintsFragment {
    pub fn new(mobile: bool, locale: &str) -> Self {
        Self {
            hints: DefaultHint::ALL
                .into_iter()
                .map(|hint| HintState::from_default(hint, mobile, locale))
                .collect(),
            current: None,
            events: HashSet::new(),
            placed_blocks: HashSet::new(),
            last_present: false,
        }
    }

    pub fn from_hints(hints: Vec<HintState>) -> Self {
        Self {
            hints,
            current: None,
            events: HashSet::new(),
            placed_blocks: HashSet::new(),
            last_present: false,
        }
    }

    pub fn shown(&self) -> bool {
        self.current.is_some()
    }

    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    pub fn hints(&self) -> &[HintState] {
        &self.hints
    }

    pub fn events(&self) -> &HashSet<String> {
        &self.events
    }

    pub fn placed_blocks(&self) -> &HashSet<String> {
        &self.placed_blocks
    }

    pub fn set_hint_show(&mut self, name: &str, show: bool) {
        if let Some(hint) = self.hints.iter_mut().find(|hint| hint.name == name) {
            hint.show = show;
        }
    }

    pub fn set_hint_complete(&mut self, name: &str, complete: bool) {
        if let Some(hint) = self.hints.iter_mut().find(|hint| hint.name == name) {
            hint.complete = complete;
        }
    }

    pub fn block_build_end(
        &mut self,
        breaking: bool,
        own_unit: bool,
        block_name: impl Into<String>,
    ) -> Vec<HintsAction> {
        let mut actions = Vec::new();
        if !breaking && own_unit {
            self.placed_blocks.insert(block_name.into());
        }
        if breaking && self.events.insert("break".into()) {
            actions.push(HintsAction::EventAdded("break".into()));
        }
        actions
    }

    pub fn cannot_upgrade(&mut self) -> Option<HintsAction> {
        self.add_event("cannotupgrade")
    }

    pub fn factory_control(&mut self) -> Option<HintsAction> {
        self.add_event("factorycontrol")
    }

    pub fn derelict_break(&mut self) -> Option<HintsAction> {
        self.add_event("derelictbreak")
    }

    pub fn reset_event(&mut self) {
        self.placed_blocks.clear();
        self.events.clear();
    }

    pub fn update(&mut self, context: HintsUpdateContext) -> Vec<HintsAction> {
        if !context.hints_enabled || !context.hud_shown {
            return Vec::new();
        }

        if let Some(current) = self.current.clone() {
            if self
                .hints
                .iter()
                .find(|hint| hint.name == current)
                .is_some_and(|hint| hint.complete)
            {
                return self.complete_current(context.mobile);
            }
            if self
                .hints
                .iter()
                .find(|hint| hint.name == current)
                .is_some_and(|hint| !hint.show)
            {
                return self.hide(context.mobile);
            }
            return Vec::new();
        }

        if context.derelict_under_mouse {
            self.add_event("derelictmouse");
        }

        if context.renderer_cutscene || !context.state_is_game || context.total_playtime <= 8000 {
            return Vec::new();
        }

        self.check_next(context.mobile, false)
    }

    pub fn check_next(&mut self, mobile: bool, ignore_playtime_gate: bool) -> Vec<HintsAction> {
        if self.current.is_some() {
            return Vec::new();
        }

        let finished_names = self
            .hints
            .iter()
            .filter(|hint| hint.finished)
            .map(|hint| hint.name.clone())
            .collect::<HashSet<_>>();
        self.hints.retain(|hint| {
            hint.valid(mobile)
                && !hint.finished
                && !(hint.show && hint.complete)
                && hint
                    .dependencies
                    .iter()
                    .all(|dep| finished_names.contains(dep))
        });
        self.hints.sort_by_key(|hint| hint.order);

        if ignore_playtime_gate {
            if let Some(index) = self.hints.iter().position(|hint| hint.show) {
                let hint = self.hints.remove(index);
                return self.display_hint(hint, mobile);
            }
        } else if let Some(index) = self.hints.iter().position(|hint| hint.show) {
            let hint = self.hints.remove(index);
            return self.display_hint(hint, mobile);
        }

        Vec::new()
    }

    pub fn complete_current(&mut self, mobile: bool) -> Vec<HintsAction> {
        let Some(current) = self.current.clone() else {
            return Vec::new();
        };

        let mut actions = vec![
            HintsAction::Complete(current.clone()),
            HintsAction::FinishSetting(format!("{current}-hint-done")),
            HintsAction::RemoveHint(current),
        ];
        actions.extend(self.hide(mobile));
        actions
    }

    pub fn skip_current(&mut self, mobile: bool) -> Vec<HintsAction> {
        self.complete_current(mobile)
    }

    pub fn hide(&mut self, mobile: bool) -> Vec<HintsAction> {
        let mut actions = Vec::new();
        if self.last_present {
            actions.push(HintsAction::Hide {
                fade_out: HINT_FADE_OUT_TIME,
                translate_y: -200.0,
            });
        }
        self.current = None;
        self.last_present = false;
        actions.extend(self.check_next(mobile, true));
        actions
    }

    fn display_hint(&mut self, hint: HintState, mobile: bool) -> Vec<HintsAction> {
        self.current = Some(hint.name.clone());
        self.last_present = true;
        vec![HintsAction::Display {
            name: hint.name,
            text: hint.text,
            width: if mobile {
                HINT_MOBILE_WIDTH
            } else {
                HINT_DESKTOP_WIDTH
            },
        }]
    }

    fn add_event(&mut self, event: &str) -> Option<HintsAction> {
        self.events
            .insert(event.to_string())
            .then(|| HintsAction::EventAdded(event.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hint(name: &str, order: i32) -> HintState {
        HintState {
            name: name.into(),
            text: format!("hint {name}"),
            order,
            visibility: HINT_VISIBLE_ALL,
            dependencies: Vec::new(),
            finished: false,
            show: false,
            complete: false,
        }
    }

    #[test]
    fn default_hint_order_visibility_and_text_follow_java_enum_shape() {
        assert_eq!(DefaultHint::ALL.len(), 27);
        assert_eq!(DefaultHint::DesktopMove.name(), "desktopMove");
        assert_eq!(DefaultHint::CoreIncinerate.order(), 26);
        assert_eq!(DefaultHint::Respawn.visibility(), HINT_VISIBLE_MOBILE);
        assert!(!DefaultHint::Respawn.valid(false));
        assert!(DefaultHint::Respawn.valid(true));

        let text = DefaultHint::Zoom.text(false, "en");
        assert!(!text.contains("Tap"));
    }

    #[test]
    fn check_next_removes_invalid_finished_and_completed_visible_hints_then_displays_first_by_order(
    ) {
        let mut a = hint("a", 2);
        a.show = true;
        let mut b = hint("b", 1);
        b.show = true;
        b.complete = true;
        let mut c = hint("c", 0);
        c.visibility = HINT_VISIBLE_MOBILE;
        c.show = true;
        let mut fragment = HintsFragment::from_hints(vec![a, b, c]);

        let actions = fragment.check_next(false, true);

        assert_eq!(
            actions,
            vec![HintsAction::Display {
                name: "a".into(),
                text: "hint a".into(),
                width: HINT_DESKTOP_WIDTH,
            }]
        );
        assert_eq!(fragment.current(), Some("a"));
        assert_eq!(fragment.hints().len(), 0);
    }

    #[test]
    fn update_waits_for_game_playtime_and_cutscene_before_displaying_hint() {
        let mut a = hint("a", 0);
        a.show = true;
        let mut fragment = HintsFragment::from_hints(vec![a]);

        assert!(fragment
            .update(HintsUpdateContext {
                mobile: false,
                hints_enabled: true,
                hud_shown: true,
                renderer_cutscene: false,
                state_is_game: true,
                total_playtime: 8000,
                derelict_under_mouse: false,
            })
            .is_empty());

        let actions = fragment.update(HintsUpdateContext {
            mobile: false,
            hints_enabled: true,
            hud_shown: true,
            renderer_cutscene: false,
            state_is_game: true,
            total_playtime: 8001,
            derelict_under_mouse: false,
        });
        assert!(matches!(actions[0], HintsAction::Display { .. }));
    }

    #[test]
    fn completing_current_finishes_setting_hides_and_immediately_checks_next() {
        let mut a = hint("a", 0);
        a.show = true;
        let mut b = hint("b", 1);
        b.show = true;
        let mut fragment = HintsFragment::from_hints(vec![a, b]);
        fragment.check_next(false, true);
        fragment.set_hint_complete("a", true);

        let actions = fragment.complete_current(false);

        assert!(actions.contains(&HintsAction::Complete("a".into())));
        assert!(actions.contains(&HintsAction::FinishSetting("a-hint-done".into())));
        assert!(actions.contains(&HintsAction::Hide {
            fade_out: HINT_FADE_OUT_TIME,
            translate_y: -200.0,
        }));
        assert_eq!(fragment.current(), Some("b"));
    }

    #[test]
    fn event_hooks_track_break_factory_derelict_and_reset_like_java() {
        let mut fragment = HintsFragment::new(false, "en");

        fragment.block_build_end(false, true, "router");
        assert!(fragment.placed_blocks().contains("router"));
        assert_eq!(
            fragment.block_build_end(true, false, "router"),
            vec![HintsAction::EventAdded("break".into())]
        );
        assert_eq!(
            fragment.cannot_upgrade(),
            Some(HintsAction::EventAdded("cannotupgrade".into()))
        );
        assert_eq!(
            fragment.factory_control(),
            Some(HintsAction::EventAdded("factorycontrol".into()))
        );
        assert_eq!(
            fragment.derelict_break(),
            Some(HintsAction::EventAdded("derelictbreak".into()))
        );

        fragment.reset_event();
        assert!(fragment.events().is_empty());
        assert!(fragment.placed_blocks().is_empty());
    }
}
