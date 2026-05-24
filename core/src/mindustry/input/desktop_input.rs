//! Side-effect-free state planner for upstream `mindustry.input.DesktopInput`.
//!
//! The Java implementation is tightly coupled to `Core`, `Vars`, UI fragments,
//! generated `Call` helpers and live world entities.  This module starts the
//! Rust port with the durable state and deterministic input decisions that can
//! be unit-tested before wiring renderer/network side effects.

use super::PlaceMode;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesktopVec2 {
    pub x: f32,
    pub y: f32,
}

impl DesktopVec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalized(self) -> Self {
        let len = self.len();
        if len == 0.0 {
            Self::ZERO
        } else {
            Self::new(self.x / len, self.y / len)
        }
    }

    pub fn scaled(self, scale: f32) -> Self {
        Self::new(self.x * scale, self.y * scale)
    }

    pub fn limited(self, max: f32) -> Self {
        let len = self.len();
        if len > max && len > 0.0 {
            self.scaled(max / len)
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopCursor {
    Arrow,
    Hand,
    Drill,
    Repair,
    Target,
    Unload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopCameraPlan {
    None,
    FollowTarget,
    KeyboardMove,
    EdgePan,
    Detached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopPayloadAction {
    None,
    Pickup,
    Drop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopInputAction {
    TogglePlayerList,
    ToggleDebugHitboxes,
    ToggleDetachedCamera { detached: bool },
    ClearSpectating,
    ResetMenuState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesktopInputSettings {
    pub hints: bool,
    pub build_autopause: bool,
    pub command_mode_hold: bool,
    pub detach_camera: bool,
    pub smooth_camera: bool,
}

impl Default for DesktopInputSettings {
    fn default() -> Self {
        Self {
            hints: true,
            build_autopause: false,
            command_mode_hold: false,
            detach_camera: false,
            smooth_camera: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesktopInputFrame {
    pub locked: bool,
    pub scene_has_mouse: bool,
    pub scene_has_field: bool,
    pub scene_has_dialog: bool,
    pub scene_has_keyboard: bool,
    pub hud_shown: bool,
    pub chat_shown: bool,
    pub minimap_shown: bool,
    pub net_active: bool,
    pub state_editor: bool,
    pub state_paused: bool,
    pub state_menu: bool,
    pub state_game: bool,
    pub possession_allowed: bool,
    pub player_dead: bool,
    pub player_builder: bool,
    pub player_unit_building: bool,
    pub player_spawned_by_core: bool,
    pub player_can_boost: bool,
    pub player_has_weapon: bool,
    pub block_selected: bool,
    pub select_plans_empty: bool,
    pub command_mode_key_down: bool,
    pub command_mode_key_tap: bool,
    pub boost_key_down: bool,
    pub pan_key_down: bool,
    pub mouse_move_key_down: bool,
    pub move_axis_x: f32,
    pub move_axis_y: f32,
    pub delta: f32,
    pub select_key_down: bool,
    pub select_key_release: bool,
    pub player_list_key_tap: bool,
    pub debug_hitboxes_key_tap: bool,
    pub detach_camera_key_tap: bool,
}

impl Default for DesktopInputFrame {
    fn default() -> Self {
        Self {
            locked: false,
            scene_has_mouse: false,
            scene_has_field: false,
            scene_has_dialog: false,
            scene_has_keyboard: false,
            hud_shown: true,
            chat_shown: false,
            minimap_shown: false,
            net_active: false,
            state_editor: false,
            state_paused: false,
            state_menu: false,
            state_game: true,
            possession_allowed: true,
            player_dead: false,
            player_builder: true,
            player_unit_building: false,
            player_spawned_by_core: false,
            player_can_boost: false,
            player_has_weapon: true,
            block_selected: false,
            select_plans_empty: true,
            command_mode_key_down: false,
            command_mode_key_tap: false,
            boost_key_down: false,
            pan_key_down: false,
            mouse_move_key_down: false,
            move_axis_x: 0.0,
            move_axis_y: 0.0,
            delta: 1.0,
            select_key_down: false,
            select_key_release: false,
            player_list_key_tap: false,
            debug_hitboxes_key_tap: false,
            detach_camera_key_tap: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopInputUpdate {
    pub should_shoot: bool,
    pub command_mode: bool,
    pub panning: bool,
    pub camera_plan: DesktopCameraPlan,
    pub camera_keyboard_delta: DesktopVec2,
    pub actions: Vec<DesktopInputAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopInput {
    pub movement: DesktopVec2,
    pub cursor_type: DesktopCursor,
    pub select_x: i32,
    pub select_y: i32,
    pub schem_x: i32,
    pub schem_y: i32,
    pub last_line_x: i32,
    pub last_line_y: i32,
    pub schematic_x: i32,
    pub schematic_y: i32,
    pub mode: PlaceMode,
    pub select_scale: f32,
    pub deleting: bool,
    pub should_shoot: bool,
    pub panning: bool,
    pub moved_plan: bool,
    pub pan_scale: f32,
    pub pan_speed: f32,
    pub pan_boost_speed: f32,
    pub select_millis: i64,
    pub last_ctrl_group: i32,
    pub last_ctrl_group_select_millis: i64,
    pub last_payload_key_tap_millis: i64,
    pub last_payload_key_hold_millis: i64,
    pub command_mode: bool,
    pub is_building: bool,
    pub build_was_auto_paused: bool,
    pub dropping_item: bool,
    pub block_selected: bool,
    pub select_plans_empty: bool,
}

impl Default for DesktopInput {
    fn default() -> Self {
        Self {
            movement: DesktopVec2::ZERO,
            cursor_type: DesktopCursor::Arrow,
            select_x: -1,
            select_y: -1,
            schem_x: -1,
            schem_y: -1,
            last_line_x: 0,
            last_line_y: 0,
            schematic_x: 0,
            schematic_y: 0,
            mode: PlaceMode::None,
            select_scale: 0.0,
            deleting: false,
            should_shoot: false,
            panning: false,
            moved_plan: false,
            pan_scale: 0.005,
            pan_speed: 4.5,
            pan_boost_speed: 15.0,
            select_millis: 0,
            last_ctrl_group: 0,
            last_ctrl_group_select_millis: 0,
            last_payload_key_tap_millis: 0,
            last_payload_key_hold_millis: 0,
            command_mode: false,
            is_building: false,
            build_was_auto_paused: false,
            dropping_item: false,
            block_selected: false,
            select_plans_empty: true,
        }
    }
}

impl DesktopInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.should_shoot = false;
        self.deleting = false;
    }

    pub fn show_hint(&self, frame: &DesktopInputFrame, settings: &DesktopInputSettings) -> bool {
        frame.hud_shown
            && settings.hints
            && self.select_plans_empty
            && !frame.player_dead
            && ((!self.is_building && !settings.build_autopause)
                || frame.player_unit_building
                || (!frame.player_dead && !frame.player_spawned_by_core))
    }

    pub fn selected_block(&self) -> bool {
        self.is_placing() && self.mode != PlaceMode::Breaking
    }

    pub fn is_breaking(&self) -> bool {
        self.mode == PlaceMode::Breaking
    }

    pub fn is_placing(&self) -> bool {
        self.block_selected
    }

    pub fn update_state(&mut self, state_menu: bool) -> Vec<DesktopInputAction> {
        if state_menu {
            self.dropping_item = false;
            self.mode = PlaceMode::None;
            self.block_selected = false;
            self.select_plans_empty = true;
            vec![DesktopInputAction::ResetMenuState]
        } else {
            Vec::new()
        }
    }

    pub fn update(
        &mut self,
        frame: &DesktopInputFrame,
        settings: &mut DesktopInputSettings,
    ) -> DesktopInputUpdate {
        self.block_selected = frame.block_selected;
        self.select_plans_empty = frame.select_plans_empty;
        let mut actions = Vec::new();

        if frame.net_active && frame.player_list_key_tap && !frame.scene_has_dialog {
            actions.push(DesktopInputAction::TogglePlayerList);
        }

        if !frame.scene_has_field && !frame.scene_has_dialog {
            if frame.debug_hitboxes_key_tap {
                actions.push(DesktopInputAction::ToggleDebugHitboxes);
            }

            if frame.detach_camera_key_tap {
                settings.detach_camera = !settings.detach_camera;
                if !settings.detach_camera {
                    self.panning = false;
                }
                actions.push(DesktopInputAction::ToggleDetachedCamera {
                    detached: settings.detach_camera,
                });
                actions.push(DesktopInputAction::ClearSpectating);
            }

            if frame.pan_key_down {
                self.panning = true;
                actions.push(DesktopInputAction::ClearSpectating);
            }

            if frame.move_axis_x.abs() > 0.0
                || frame.move_axis_y.abs() > 0.0
                || frame.mouse_move_key_down
            {
                self.panning = false;
                actions.push(DesktopInputAction::ClearSpectating);
            }
        }

        self.panning |= settings.detach_camera;

        let camera_plan = self.camera_plan(frame, settings);
        let camera_keyboard_delta = self.keyboard_camera_delta(frame);

        self.should_shoot = !frame.scene_has_mouse && !frame.locked && !frame.state_editor;

        self.command_mode = self.next_command_mode(frame, settings);

        if frame.state_menu {
            actions.extend(self.update_state(true));
        }

        DesktopInputUpdate {
            should_shoot: self.should_shoot,
            command_mode: self.command_mode,
            panning: self.panning,
            camera_plan,
            camera_keyboard_delta,
            actions,
        }
    }

    pub fn payload_action(
        &mut self,
        pickup_tap: bool,
        pickup_down: bool,
        drop_tap: bool,
        drop_down: bool,
        now_millis: i64,
    ) -> DesktopPayloadAction {
        if pickup_tap {
            self.last_payload_key_tap_millis = now_millis;
            return DesktopPayloadAction::Pickup;
        }

        if pickup_down
            && now_millis - self.last_payload_key_hold_millis > 20
            && now_millis - self.last_payload_key_tap_millis > 200
        {
            self.last_payload_key_hold_millis = now_millis;
            return DesktopPayloadAction::Pickup;
        }

        if drop_tap {
            self.last_payload_key_tap_millis = now_millis;
            return DesktopPayloadAction::Drop;
        }

        if drop_down
            && now_millis - self.last_payload_key_hold_millis > 20
            && now_millis - self.last_payload_key_tap_millis > 200
        {
            self.last_payload_key_hold_millis = now_millis;
            return DesktopPayloadAction::Drop;
        }

        DesktopPayloadAction::None
    }

    fn next_command_mode(
        &self,
        frame: &DesktopInputFrame,
        settings: &DesktopInputSettings,
    ) -> bool {
        let can_command = !frame.locked
            && !frame.block_selected
            && !frame.scene_has_field
            && !frame.scene_has_dialog
            && !(frame.player_can_boost && frame.boost_key_down);

        if can_command {
            if settings.command_mode_hold {
                frame.command_mode_key_down
            } else if frame.command_mode_key_tap {
                !self.command_mode
            } else {
                self.command_mode
            }
        } else {
            false
        }
    }

    fn camera_plan(
        &self,
        frame: &DesktopInputFrame,
        settings: &DesktopInputSettings,
    ) -> DesktopCameraPlan {
        if frame.locked {
            return DesktopCameraPlan::None;
        }

        if settings.detach_camera {
            return DesktopCameraPlan::Detached;
        }

        if frame.pan_key_down || frame.mouse_move_key_down {
            return DesktopCameraPlan::EdgePan;
        }

        if (frame.player_dead || frame.state_paused)
            && !frame.chat_shown
            && !frame.scene_has_field
            && !frame.scene_has_dialog
        {
            DesktopCameraPlan::KeyboardMove
        } else if (!frame.player_dead || self.panning) && !self.panning {
            DesktopCameraPlan::FollowTarget
        } else {
            DesktopCameraPlan::None
        }
    }

    fn keyboard_camera_delta(&self, frame: &DesktopInputFrame) -> DesktopVec2 {
        let speed = if frame.boost_key_down {
            self.pan_boost_speed
        } else {
            self.pan_speed
        } * frame.delta;
        DesktopVec2::new(frame.move_axis_x, frame.move_axis_y)
            .normalized()
            .scaled(speed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_matches_java_field_defaults() {
        let input = DesktopInput::new();

        assert_eq!(input.movement, DesktopVec2::ZERO);
        assert_eq!(input.cursor_type, DesktopCursor::Arrow);
        assert_eq!(input.select_x, -1);
        assert_eq!(input.select_y, -1);
        assert_eq!(input.schem_x, -1);
        assert_eq!(input.schem_y, -1);
        assert_eq!(input.mode, PlaceMode::None);
        assert_eq!(input.pan_scale, 0.005);
        assert_eq!(input.pan_speed, 4.5);
        assert_eq!(input.pan_boost_speed, 15.0);
    }

    #[test]
    fn reset_clears_shooting_and_deleting_like_java() {
        let mut input = DesktopInput::new();
        input.should_shoot = true;
        input.deleting = true;
        input.mode = PlaceMode::Breaking;

        input.reset();

        assert!(!input.should_shoot);
        assert!(!input.deleting);
        assert_eq!(input.mode, PlaceMode::Breaking);
    }

    #[test]
    fn show_hint_follows_hud_settings_player_and_building_guards() {
        let mut input = DesktopInput::new();
        let frame = DesktopInputFrame::default();
        let settings = DesktopInputSettings::default();

        assert!(input.show_hint(&frame, &settings));

        input.select_plans_empty = false;
        assert!(!input.show_hint(&frame, &settings));

        input.select_plans_empty = true;
        input.is_building = true;
        let mut no_reason = frame;
        no_reason.player_spawned_by_core = true;
        assert!(!input.show_hint(&no_reason, &settings));

        no_reason.player_unit_building = true;
        assert!(input.show_hint(&no_reason, &settings));
    }

    #[test]
    fn selected_block_is_placing_except_breaking_mode() {
        let mut input = DesktopInput::new();
        input.block_selected = true;
        input.mode = PlaceMode::Placing;
        assert!(input.selected_block());

        input.mode = PlaceMode::Breaking;
        assert!(!input.selected_block());

        input.block_selected = false;
        input.mode = PlaceMode::Placing;
        assert!(!input.selected_block());
    }

    #[test]
    fn update_state_resets_menu_transient_fields() {
        let mut input = DesktopInput::new();
        input.dropping_item = true;
        input.mode = PlaceMode::Placing;
        input.block_selected = true;
        input.select_plans_empty = false;

        assert_eq!(
            input.update_state(true),
            vec![DesktopInputAction::ResetMenuState]
        );
        assert!(!input.dropping_item);
        assert_eq!(input.mode, PlaceMode::None);
        assert!(!input.block_selected);
        assert!(input.select_plans_empty);
    }

    #[test]
    fn command_mode_toggle_and_hold_modes_match_java() {
        let mut input = DesktopInput::new();
        let mut settings = DesktopInputSettings::default();
        let mut frame = DesktopInputFrame {
            command_mode_key_tap: true,
            ..DesktopInputFrame::default()
        };

        assert!(input.update(&frame, &mut settings).command_mode);
        frame.command_mode_key_tap = false;
        assert!(input.update(&frame, &mut settings).command_mode);
        frame.command_mode_key_tap = true;
        assert!(!input.update(&frame, &mut settings).command_mode);

        settings.command_mode_hold = true;
        frame.command_mode_key_tap = false;
        frame.command_mode_key_down = true;
        assert!(input.update(&frame, &mut settings).command_mode);
        frame.command_mode_key_down = false;
        assert!(!input.update(&frame, &mut settings).command_mode);
    }

    #[test]
    fn command_mode_disabled_when_locked_blocking_or_boost_conflict() {
        let mut input = DesktopInput::new();
        input.command_mode = true;
        let mut settings = DesktopInputSettings::default();

        for frame in [
            DesktopInputFrame {
                locked: true,
                ..DesktopInputFrame::default()
            },
            DesktopInputFrame {
                block_selected: true,
                ..DesktopInputFrame::default()
            },
            DesktopInputFrame {
                scene_has_dialog: true,
                ..DesktopInputFrame::default()
            },
            DesktopInputFrame {
                player_can_boost: true,
                boost_key_down: true,
                ..DesktopInputFrame::default()
            },
        ] {
            assert!(!input.update(&frame, &mut settings).command_mode);
        }
    }

    #[test]
    fn update_emits_ui_debug_detach_and_spectating_actions() {
        let mut input = DesktopInput::new();
        let mut settings = DesktopInputSettings::default();
        let frame = DesktopInputFrame {
            net_active: true,
            player_list_key_tap: true,
            debug_hitboxes_key_tap: true,
            detach_camera_key_tap: true,
            ..DesktopInputFrame::default()
        };

        let update = input.update(&frame, &mut settings);

        assert!(settings.detach_camera);
        assert_eq!(
            update.actions,
            vec![
                DesktopInputAction::TogglePlayerList,
                DesktopInputAction::ToggleDebugHitboxes,
                DesktopInputAction::ToggleDetachedCamera { detached: true },
                DesktopInputAction::ClearSpectating,
            ]
        );
        assert_eq!(update.camera_plan, DesktopCameraPlan::Detached);
    }

    #[test]
    fn camera_keyboard_delta_uses_boosted_pan_speed() {
        let input = DesktopInput::new();
        let frame = DesktopInputFrame {
            move_axis_x: 3.0,
            move_axis_y: 4.0,
            boost_key_down: true,
            delta: 2.0,
            ..DesktopInputFrame::default()
        };

        assert_eq!(
            input.keyboard_camera_delta(&frame),
            DesktopVec2::new(18.0, 24.0)
        );
    }

    #[test]
    fn payload_repeat_actions_follow_tap_and_hold_timing() {
        let mut input = DesktopInput::new();

        assert_eq!(
            input.payload_action(true, false, false, false, 1000),
            DesktopPayloadAction::Pickup
        );
        assert_eq!(
            input.payload_action(false, true, false, false, 1100),
            DesktopPayloadAction::None
        );
        assert_eq!(
            input.payload_action(false, true, false, false, 1210),
            DesktopPayloadAction::Pickup
        );
        assert_eq!(
            input.payload_action(false, false, true, false, 1300),
            DesktopPayloadAction::Drop
        );
        assert_eq!(
            input.payload_action(false, false, false, true, 1510),
            DesktopPayloadAction::Drop
        );
    }

    #[test]
    fn should_shoot_requires_unlocked_non_mouse_non_editor_frame() {
        let mut input = DesktopInput::new();
        let mut settings = DesktopInputSettings::default();

        assert!(
            input
                .update(&DesktopInputFrame::default(), &mut settings)
                .should_shoot
        );

        for frame in [
            DesktopInputFrame {
                scene_has_mouse: true,
                ..DesktopInputFrame::default()
            },
            DesktopInputFrame {
                locked: true,
                ..DesktopInputFrame::default()
            },
            DesktopInputFrame {
                state_editor: true,
                ..DesktopInputFrame::default()
            },
        ] {
            assert!(!input.update(&frame, &mut settings).should_shoot);
        }
    }
}
