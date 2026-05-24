//! Side-effect-free state planner for upstream `mindustry.input.MobileInput`.
//!
//! The Java mobile input class mixes durable input state with `Core`, `Vars`,
//! UI widgets, live world objects and generated network calls.  This module
//! ports the deterministic state transitions and gesture math first, so later
//! renderer/world/network wiring can call into a tested Rust core instead of
//! re-implementing the same branch logic in platform code.

use super::PlaceMode;

pub const MAX_PAN_SPEED: f32 = 1.3;
pub const DEFAULT_EDGE_PAN: f32 = 60.0;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MobileVec2 {
    pub x: f32,
    pub y: f32,
}

impl MobileVec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
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

    pub fn is_zero(self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobileInputAction {
    ResetMenuState,
    ClearCommandSelections,
    RequestAutoPan,
    UpdateLine {
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    },
    ClearLinePlans,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MobileInputUpdate {
    pub command_mode: bool,
    pub mode: PlaceMode,
    pub line_mode: bool,
    pub schematic_mode: bool,
    pub rebuild_mode: bool,
    pub queue_command_mode: bool,
    pub line_scale: f32,
    pub manual_shooting: bool,
    pub payload_target_present: bool,
    pub actions: Vec<MobileInputAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MobileInputFrame {
    pub locked: bool,
    pub command_mode: bool,
    pub scene_has_field: bool,
    pub scene_has_keyboard: bool,
    pub scene_has_scroll: bool,
    pub state_menu: bool,
    pub player_dead: bool,
    pub player_unit_building: bool,
    pub player_builder: bool,
    pub console_shown: bool,
    pub selected_block: Option<String>,
    pub selected_block_rotates: bool,
    pub select_plans_empty: bool,
    pub last_schematic_present: bool,
    pub rotate_placed_key_down: bool,
    pub diagonal_placement_key_down: bool,
    pub zoom_axis_tap: f32,
    pub keyboard: bool,
    pub touched_primary: bool,
    pub touched_secondary: bool,
    pub raw_tile_x: i32,
    pub raw_tile_y: i32,
    pub cursor_tile_x: i32,
    pub cursor_tile_y: i32,
    pub delta: f32,
}

impl Default for MobileInputFrame {
    fn default() -> Self {
        Self {
            locked: false,
            command_mode: false,
            scene_has_field: false,
            scene_has_keyboard: false,
            scene_has_scroll: false,
            state_menu: false,
            player_dead: false,
            player_unit_building: false,
            player_builder: true,
            console_shown: false,
            selected_block: None,
            selected_block_rotates: false,
            select_plans_empty: true,
            last_schematic_present: false,
            rotate_placed_key_down: false,
            diagonal_placement_key_down: false,
            zoom_axis_tap: 0.0,
            keyboard: false,
            touched_primary: false,
            touched_secondary: false,
            raw_tile_x: 0,
            raw_tile_y: 0,
            cursor_tile_x: 0,
            cursor_tile_y: 0,
            delta: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MobilePanFrame {
    pub scene_available: bool,
    pub scene_has_dialog: bool,
    pub keyboard: bool,
    pub locked: bool,
    pub command_rect: bool,
    pub touched_secondary: bool,
    pub dropping_item: bool,
    pub camera_width: f32,
    pub graphics_width: f32,
    pub tile_size: f32,
}

impl Default for MobilePanFrame {
    fn default() -> Self {
        Self {
            scene_available: true,
            scene_has_dialog: false,
            keyboard: false,
            locked: false,
            command_rect: false,
            touched_secondary: false,
            dropping_item: false,
            camera_width: 1.0,
            graphics_width: 1.0,
            tile_size: 8.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MobilePanPlan {
    pub accepted: bool,
    pub camera_delta: MobileVec2,
    pub shifted_tiles_x: i32,
    pub shifted_tiles_y: i32,
    pub residual_shift: MobileVec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MobileZoomPlan {
    pub accepted: bool,
    pub scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MobileInput {
    pub edge_pan: f32,
    pub vector: MobileVec2,
    pub movement: MobileVec2,
    pub target_pos: MobileVec2,
    pub last_zoom: f32,
    pub line_start_x: i32,
    pub line_start_y: i32,
    pub last_line_x: i32,
    pub last_line_y: i32,
    pub line_scale: f32,
    pub crosshair_scale: f32,
    pub shift_delta_x: f32,
    pub shift_delta_y: f32,
    pub selecting: bool,
    pub line_mode: bool,
    pub schematic_mode: bool,
    pub rebuild_mode: bool,
    pub queue_command_mode: bool,
    pub command_mode: bool,
    pub mode: PlaceMode,
    pub last_block: Option<String>,
    pub selected_block: Option<String>,
    pub last_placed_present: bool,
    pub down: bool,
    pub manual_shooting: bool,
    pub target_present: bool,
    pub payload_target_present: bool,
    pub unit_tapped_present: bool,
    pub building_tapped_present: bool,
    pub select_plans_empty: bool,
    pub removals_empty: bool,
    pub last_schematic_present: bool,
}

impl Default for MobileInput {
    fn default() -> Self {
        Self {
            edge_pan: DEFAULT_EDGE_PAN,
            vector: MobileVec2::ZERO,
            movement: MobileVec2::ZERO,
            target_pos: MobileVec2::ZERO,
            last_zoom: -1.0,
            line_start_x: 0,
            line_start_y: 0,
            last_line_x: 0,
            last_line_y: 0,
            line_scale: 0.0,
            crosshair_scale: 0.0,
            shift_delta_x: 0.0,
            shift_delta_y: 0.0,
            selecting: false,
            line_mode: false,
            schematic_mode: false,
            rebuild_mode: false,
            queue_command_mode: false,
            command_mode: false,
            mode: PlaceMode::None,
            last_block: None,
            selected_block: None,
            last_placed_present: false,
            down: false,
            manual_shooting: false,
            target_present: false,
            payload_target_present: false,
            unit_tapped_present: false,
            building_tapped_present: false,
            select_plans_empty: true,
            removals_empty: true,
            last_schematic_present: false,
        }
    }
}

impl MobileInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show_cancel(&self, frame: &MobileInputFrame) -> bool {
        !frame.player_dead
            && (frame.player_unit_building
                || self.selected_block.is_some()
                || self.mode == PlaceMode::Breaking
                || !self.select_plans_empty)
            && !self.has_schematic()
            && !frame.console_shown
    }

    pub fn has_schematic(&self) -> bool {
        self.last_schematic_present && !self.select_plans_empty
    }

    pub fn is_rebuild_selecting(&self) -> bool {
        self.rebuild_mode
    }

    pub fn is_placing(&self) -> bool {
        self.selected_block.is_some() && self.mode == PlaceMode::Placing
    }

    pub fn is_breaking(&self) -> bool {
        self.mode == PlaceMode::Breaking
    }

    pub fn reset(&mut self) {
        self.manual_shooting = false;
        self.down = false;
    }

    pub fn update_state(&mut self, state_menu: bool) -> Vec<MobileInputAction> {
        if state_menu {
            self.select_plans_empty = true;
            self.removals_empty = true;
            self.mode = PlaceMode::None;
            self.manual_shooting = false;
            self.payload_target_present = false;
            vec![MobileInputAction::ResetMenuState]
        } else {
            Vec::new()
        }
    }

    pub fn update(&mut self, frame: &MobileInputFrame) -> MobileInputUpdate {
        self.command_mode = frame.command_mode;
        self.selected_block = frame.selected_block.clone();
        self.select_plans_empty = frame.select_plans_empty;
        self.last_schematic_present = frame.last_schematic_present;

        let mut actions = self.update_state(frame.state_menu);

        if !self.command_mode {
            self.queue_command_mode = false;
        } else {
            self.mode = PlaceMode::None;
            self.schematic_mode = false;
        }

        if self.selected_block.is_some() {
            self.rebuild_mode = false;
        }

        if frame.player_dead {
            self.mode = PlaceMode::None;
            self.manual_shooting = false;
            self.payload_target_present = false;
        }

        if frame.locked
            || self.selected_block.is_some()
            || frame.scene_has_field
            || self.has_schematic()
        {
            self.command_mode = false;
        }

        if !self.command_mode {
            actions.push(MobileInputAction::ClearCommandSelections);
        }

        if self.mode == PlaceMode::None {
            self.line_mode = false;
        }

        if self.line_mode && self.mode == PlaceMode::Placing && self.selected_block.is_none() {
            self.line_mode = false;
        }

        if self.selected_block.is_some() && self.mode == PlaceMode::None {
            self.mode = PlaceMode::Placing;
        }

        if self.selected_block.is_none() && self.mode == PlaceMode::Placing {
            self.mode = PlaceMode::None;
        }

        if self.selected_block.is_some() {
            self.schematic_mode = false;
        }

        if !self.schematic_mode
            && (self.mode == PlaceMode::SchematicSelect || self.mode == PlaceMode::RebuildSelect)
        {
            self.mode = PlaceMode::None;
        }

        if !self.rebuild_mode && self.mode == PlaceMode::RebuildSelect {
            self.mode = PlaceMode::None;
        }

        if self.mode == PlaceMode::SchematicSelect || self.mode == PlaceMode::RebuildSelect {
            self.last_line_x = frame.raw_tile_x;
            self.last_line_y = frame.raw_tile_y;
            actions.push(MobileInputAction::RequestAutoPan);
        }

        if self.last_block != self.selected_block
            && self.mode == PlaceMode::Breaking
            && self.selected_block.is_some()
        {
            self.mode = PlaceMode::Placing;
            self.last_block = self.selected_block.clone();
        }

        if self.line_mode {
            self.line_scale = lerp_delta(self.line_scale, 1.0, 0.1, frame.delta);

            if frame.touched_primary {
                actions.push(MobileInputAction::RequestAutoPan);
            }

            if (self.last_line_x != frame.cursor_tile_x || self.last_line_y != frame.cursor_tile_y)
                && self.is_placing()
            {
                self.last_line_x = frame.cursor_tile_x;
                self.last_line_y = frame.cursor_tile_y;
                actions.push(MobileInputAction::UpdateLine {
                    start_x: self.line_start_x,
                    start_y: self.line_start_y,
                    end_x: frame.cursor_tile_x,
                    end_y: frame.cursor_tile_y,
                });
            }
        } else {
            self.line_scale = 0.0;
            actions.push(MobileInputAction::ClearLinePlans);
        }

        MobileInputUpdate {
            command_mode: self.command_mode,
            mode: self.mode,
            line_mode: self.line_mode,
            schematic_mode: self.schematic_mode,
            rebuild_mode: self.rebuild_mode,
            queue_command_mode: self.queue_command_mode,
            line_scale: self.line_scale,
            manual_shooting: self.manual_shooting,
            payload_target_present: self.payload_target_present,
            actions,
        }
    }

    pub fn auto_pan(
        &mut self,
        screen_x: f32,
        screen_y: f32,
        graphics_width: f32,
        graphics_height: f32,
        camera_width: f32,
    ) -> MobileVec2 {
        let pan_x = if screen_x <= self.edge_pan {
            -(self.edge_pan - screen_x)
        } else if screen_x >= graphics_width - self.edge_pan {
            (screen_x - graphics_width) + self.edge_pan
        } else {
            0.0
        };

        let pan_y = if screen_y <= self.edge_pan {
            -(self.edge_pan - screen_y)
        } else if screen_y >= graphics_height - self.edge_pan {
            (screen_y - graphics_height) + self.edge_pan
        } else {
            0.0
        };

        let scale = if graphics_width == 0.0 {
            0.0
        } else {
            camera_width / graphics_width
        };
        self.vector = MobileVec2::new(pan_x, pan_y)
            .scaled(scale)
            .limited(MAX_PAN_SPEED);
        self.vector
    }

    pub fn pan(
        &mut self,
        _x: f32,
        _y: f32,
        delta_x: f32,
        delta_y: f32,
        frame: &MobilePanFrame,
    ) -> MobilePanPlan {
        if !frame.scene_available
            || frame.scene_has_dialog
            || frame.keyboard
            || frame.locked
            || frame.command_rect
        {
            return MobilePanPlan::default();
        }

        let scale = if frame.graphics_width == 0.0 {
            0.0
        } else {
            frame.camera_width / frame.graphics_width
        };
        let delta_x = delta_x * scale;
        let delta_y = delta_y * scale;

        if (self.line_mode && !frame.touched_secondary)
            || frame.dropping_item
            || self.schematic_mode
        {
            return MobilePanPlan::default();
        }

        if !self.down || self.manual_shooting {
            return MobilePanPlan::default();
        }

        if self.selecting {
            self.shift_delta_x += delta_x;
            self.shift_delta_y += delta_y;

            let tile_size = if frame.tile_size == 0.0 {
                1.0
            } else {
                frame.tile_size
            };
            let shifted_tiles_x = (self.shift_delta_x / tile_size) as i32;
            let shifted_tiles_y = (self.shift_delta_y / tile_size) as i32;

            if shifted_tiles_x != 0 || shifted_tiles_y != 0 {
                self.shift_delta_x %= tile_size;
                self.shift_delta_y %= tile_size;
            }

            MobilePanPlan {
                accepted: true,
                camera_delta: MobileVec2::ZERO,
                shifted_tiles_x,
                shifted_tiles_y,
                residual_shift: MobileVec2::new(self.shift_delta_x, self.shift_delta_y),
            }
        } else {
            MobilePanPlan {
                accepted: true,
                camera_delta: MobileVec2::new(-delta_x, -delta_y),
                shifted_tiles_x: 0,
                shifted_tiles_y: 0,
                residual_shift: MobileVec2::new(self.shift_delta_x, self.shift_delta_y),
            }
        }
    }

    pub fn pan_stop(&mut self) {
        self.shift_delta_x = 0.0;
        self.shift_delta_y = 0.0;
    }

    pub fn zoom(
        &mut self,
        initial_distance: f32,
        distance: f32,
        keyboard: bool,
        current_renderer_scale: f32,
    ) -> MobileZoomPlan {
        if keyboard || initial_distance == 0.0 {
            return MobileZoomPlan {
                accepted: false,
                scale: current_renderer_scale,
            };
        }

        if self.last_zoom < 0.0 {
            self.last_zoom = current_renderer_scale;
        }

        MobileZoomPlan {
            accepted: true,
            scale: distance / initial_distance * self.last_zoom,
        }
    }
}

fn lerp_delta(from: f32, to: f32, alpha: f32, delta: f32) -> f32 {
    let t = (alpha * delta.max(0.0)).clamp(0.0, 1.0);
    from + (to - from) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_matches_java_field_defaults() {
        let input = MobileInput::new();

        assert_eq!(input.edge_pan, DEFAULT_EDGE_PAN);
        assert_eq!(input.vector, MobileVec2::ZERO);
        assert_eq!(input.movement, MobileVec2::ZERO);
        assert_eq!(input.target_pos, MobileVec2::ZERO);
        assert_eq!(input.last_zoom, -1.0);
        assert_eq!(input.mode, PlaceMode::None);
        assert!(!input.down);
        assert!(!input.manual_shooting);
        assert!(!input.line_mode);
        assert!(!input.schematic_mode);
        assert!(!input.rebuild_mode);
        assert!(!input.queue_command_mode);
    }

    #[test]
    fn show_cancel_matches_player_block_plan_and_schematic_guards() {
        let mut input = MobileInput::new();
        let mut frame = MobileInputFrame::default();

        assert!(!input.show_cancel(&frame));

        frame.player_unit_building = true;
        assert!(input.show_cancel(&frame));

        frame.console_shown = true;
        assert!(!input.show_cancel(&frame));

        frame.console_shown = false;
        input.last_schematic_present = true;
        input.select_plans_empty = false;
        assert!(!input.show_cancel(&frame));

        input.last_schematic_present = false;
        input.mode = PlaceMode::Breaking;
        frame.player_unit_building = false;
        assert!(input.show_cancel(&frame));
    }

    #[test]
    fn reset_only_clears_down_and_manual_shooting_like_mobile_override() {
        let mut input = MobileInput::new();
        input.down = true;
        input.manual_shooting = true;
        input.mode = PlaceMode::Breaking;

        input.reset();

        assert!(!input.down);
        assert!(!input.manual_shooting);
        assert_eq!(input.mode, PlaceMode::Breaking);
    }

    #[test]
    fn update_state_resets_menu_transients() {
        let mut input = MobileInput::new();
        input.select_plans_empty = false;
        input.removals_empty = false;
        input.mode = PlaceMode::Placing;
        input.manual_shooting = true;
        input.payload_target_present = true;

        assert_eq!(
            input.update_state(true),
            vec![MobileInputAction::ResetMenuState]
        );
        assert!(input.select_plans_empty);
        assert!(input.removals_empty);
        assert_eq!(input.mode, PlaceMode::None);
        assert!(!input.manual_shooting);
        assert!(!input.payload_target_present);
    }

    #[test]
    fn update_converges_command_block_schematic_and_rebuild_modes() {
        let mut input = MobileInput::new();
        input.command_mode = true;
        input.queue_command_mode = true;
        input.schematic_mode = true;
        input.rebuild_mode = true;
        input.mode = PlaceMode::SchematicSelect;

        let update = input.update(&MobileInputFrame {
            command_mode: true,
            selected_block: Some("duo".to_string()),
            ..MobileInputFrame::default()
        });

        assert!(!update.command_mode);
        assert_eq!(update.mode, PlaceMode::Placing);
        assert!(!update.schematic_mode);
        assert!(!update.rebuild_mode);
        assert!(update
            .actions
            .contains(&MobileInputAction::ClearCommandSelections));
    }

    #[test]
    fn player_death_clears_mode_manual_shooting_and_payload_target() {
        let mut input = MobileInput::new();
        input.mode = PlaceMode::Breaking;
        input.manual_shooting = true;
        input.payload_target_present = true;

        let update = input.update(&MobileInputFrame {
            player_dead: true,
            ..MobileInputFrame::default()
        });

        assert_eq!(update.mode, PlaceMode::None);
        assert!(!update.manual_shooting);
        assert!(!update.payload_target_present);
    }

    #[test]
    fn line_mode_updates_scale_and_requests_line_rebuild() {
        let mut input = MobileInput::new();
        input.selected_block = Some("conveyor".to_string());
        input.mode = PlaceMode::Placing;
        input.line_mode = true;
        input.line_start_x = 1;
        input.line_start_y = 2;
        input.last_line_x = 1;
        input.last_line_y = 2;

        let update = input.update(&MobileInputFrame {
            selected_block: Some("conveyor".to_string()),
            cursor_tile_x: 5,
            cursor_tile_y: 2,
            touched_primary: true,
            ..MobileInputFrame::default()
        });

        assert_eq!(update.line_scale, 0.1);
        assert!(update.actions.contains(&MobileInputAction::RequestAutoPan));
        assert!(update.actions.contains(&MobileInputAction::UpdateLine {
            start_x: 1,
            start_y: 2,
            end_x: 5,
            end_y: 2,
        }));
    }

    #[test]
    fn auto_pan_uses_edge_distance_camera_scale_and_speed_limit() {
        let mut input = MobileInput::new();

        assert_eq!(
            input.auto_pan(250.0, 250.0, 500.0, 500.0, 1000.0),
            MobileVec2::ZERO
        );

        let left = input.auto_pan(0.0, 250.0, 500.0, 500.0, 1000.0);
        assert_eq!(left, MobileVec2::new(-MAX_PAN_SPEED, 0.0));

        let corner = input.auto_pan(500.0, 500.0, 500.0, 500.0, 500.0);
        assert!((corner.len() - MAX_PAN_SPEED).abs() < 0.0001);
        assert!(corner.x > 0.0 && corner.y > 0.0);
    }

    #[test]
    fn pan_respects_gates_and_returns_camera_delta_when_not_selecting() {
        let mut input = MobileInput::new();
        input.down = true;

        assert!(
            !input
                .pan(
                    0.0,
                    0.0,
                    10.0,
                    -5.0,
                    &MobilePanFrame {
                        locked: true,
                        ..MobilePanFrame::default()
                    },
                )
                .accepted
        );

        let plan = input.pan(
            0.0,
            0.0,
            10.0,
            -5.0,
            &MobilePanFrame {
                camera_width: 100.0,
                graphics_width: 50.0,
                ..MobilePanFrame::default()
            },
        );

        assert!(plan.accepted);
        assert_eq!(plan.camera_delta, MobileVec2::new(-20.0, 10.0));
    }

    #[test]
    fn pan_selecting_accumulates_tile_shifts_and_residual() {
        let mut input = MobileInput::new();
        input.down = true;
        input.selecting = true;

        let plan = input.pan(
            0.0,
            0.0,
            18.0,
            -9.0,
            &MobilePanFrame {
                camera_width: 1.0,
                graphics_width: 1.0,
                tile_size: 8.0,
                ..MobilePanFrame::default()
            },
        );

        assert!(plan.accepted);
        assert_eq!(plan.shifted_tiles_x, 2);
        assert_eq!(plan.shifted_tiles_y, -1);
        assert_eq!(plan.residual_shift, MobileVec2::new(2.0, -1.0));

        input.pan_stop();
        assert_eq!(input.shift_delta_x, 0.0);
        assert_eq!(input.shift_delta_y, 0.0);
    }

    #[test]
    fn zoom_initializes_last_zoom_and_reuses_it_until_gesture_reset() {
        let mut input = MobileInput::new();

        let first = input.zoom(100.0, 150.0, false, 2.0);
        assert_eq!(
            first,
            MobileZoomPlan {
                accepted: true,
                scale: 3.0,
            }
        );
        assert_eq!(input.last_zoom, 2.0);

        let second = input.zoom(100.0, 50.0, false, 99.0);
        assert_eq!(second.scale, 1.0);

        let keyboard = input.zoom(100.0, 200.0, true, 4.0);
        assert_eq!(
            keyboard,
            MobileZoomPlan {
                accepted: false,
                scale: 4.0,
            }
        );
    }
}
