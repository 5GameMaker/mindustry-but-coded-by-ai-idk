//! Side-effect-free state planner for upstream `mindustry.input.MobileInput`.
//!
//! The Java mobile input class mixes durable input state with `Core`, `Vars`,
//! UI widgets, live world objects and generated network calls.  This module
//! ports the deterministic state transitions and gesture math first, so later
//! renderer/world/network wiring can call into a tested Rust core instead of
//! re-implementing the same branch logic in platform code.

use super::PlaceMode;
use crate::mindustry::{
    entities::{units::BuildPlan, Rect},
    vars::TILE_SIZE,
};

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

#[derive(Debug, Clone, PartialEq)]
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
    ToggleDiagonalPlacement,
    ToggleBreakMode {
        mode: PlaceMode,
    },
    RotateBlock {
        rotation: i32,
    },
    ToggleSchematicMode {
        enabled: bool,
    },
    ToggleRebuildMode {
        enabled: bool,
    },
    ConfirmSelectedPlans,
    CancelPlacement,
    ClearBuilding,
    BeginSchematicSelection {
        rebuild: bool,
        tile_x: i32,
        tile_y: i32,
    },
    StartKeyboardShooting,
    ConfirmLinePlacement,
    ConfirmAreaBreak {
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    },
    CreateSchematicSelection {
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    },
    RebuildArea {
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    },
    TryDropItems {
        tile_present: bool,
        world_x: f32,
        world_y: f32,
    },
    SelectUnitsRect,
    BeginCommandRect {
        world_x: f32,
        world_y: f32,
    },
    SetPayloadTargetUnit,
    SetPayloadTargetBuilding,
    SetPayloadDropPosition {
        world_x: f32,
        world_y: f32,
    },
    BeginManualShooting,
    PlaySelectEffect {
        world_x: f32,
        world_y: f32,
    },
    PlayTapBlockEffect {
        world_x: f32,
        world_y: f32,
        size: f32,
    },
    TileTap {
        tile_x: i32,
        tile_y: i32,
    },
    CheckTargets {
        world_x: f32,
        world_y: f32,
    },
    RemovePlanAtCursor,
    AddPlacePlan {
        tile_x: i32,
        tile_y: i32,
        rotation: i32,
        block: String,
    },
    AddBreakPlan {
        tile_x: i32,
        tile_y: i32,
    },
    CommandTap {
        queue: bool,
    },
    TapCommandUnit,
    PingLocation {
        world_x: f32,
        world_y: f32,
    },
    ResetPayloadTarget,
    UnitControlTapped,
    BuildingControlTapped,
    FallbackDoubleTap,
    StoreTapCandidates {
        unit: bool,
        building: bool,
    },
    TryBeginMine,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobilePlacementButton {
    BreakMode,
    Diagonal,
    RotateOrSchematic,
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MobileGesturePlan {
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MobileActionPlan {
    pub accepted: bool,
    pub actions: Vec<MobileInputAction>,
}

impl MobileActionPlan {
    pub fn rejected() -> Self {
        Self {
            accepted: false,
            actions: Vec::new(),
        }
    }

    pub fn accepted(actions: Vec<MobileInputAction>) -> Self {
        Self {
            accepted: true,
            actions,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobileTouchDownFrame {
    pub state_menu: bool,
    pub locked: bool,
    pub player_dead: bool,
    pub cursor: Option<(i32, i32)>,
    pub scene_has_mouse: bool,
    pub pointer: i32,
    pub keyboard: bool,
    pub state_editor: bool,
    pub try_tap_player: bool,
    pub has_plan_at_cursor: bool,
}

impl Default for MobileTouchDownFrame {
    fn default() -> Self {
        Self {
            state_menu: false,
            locked: false,
            player_dead: false,
            cursor: Some((0, 0)),
            scene_has_mouse: false,
            pointer: 0,
            keyboard: false,
            state_editor: false,
            try_tap_player: false,
            has_plan_at_cursor: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MobileTouchUpFrame {
    pub any_touched: bool,
    pub renderer_scale: f32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub player_dead: bool,
    pub tile_present: bool,
    pub world_x: f32,
    pub world_y: f32,
}

impl Default for MobileTouchUpFrame {
    fn default() -> Self {
        Self {
            any_touched: false,
            renderer_scale: 1.0,
            tile_x: 0,
            tile_y: 0,
            player_dead: false,
            tile_present: false,
            world_x: 0.0,
            world_y: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MobileLongPressFrame {
    pub state_menu: bool,
    pub locked: bool,
    pub player_dead: bool,
    pub scene_has_mouse: bool,
    pub state_paused: bool,
    pub cursor: Option<(i32, i32)>,
    pub cursor_world_x: f32,
    pub cursor_world_y: f32,
    pub command_mode: bool,
    pub payload_unit_available: bool,
    pub payload_building_available: bool,
    pub payload_has_payload: bool,
    pub selected_block_size: f32,
    pub selected_block_offset: f32,
}

impl Default for MobileLongPressFrame {
    fn default() -> Self {
        Self {
            state_menu: false,
            locked: false,
            player_dead: false,
            scene_has_mouse: false,
            state_paused: false,
            cursor: Some((0, 0)),
            cursor_world_x: 0.0,
            cursor_world_y: 0.0,
            command_mode: false,
            payload_unit_available: false,
            payload_building_available: false,
            payload_has_payload: false,
            selected_block_size: 1.0,
            selected_block_offset: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MobileTapFrame {
    pub state_menu: bool,
    pub locked: bool,
    pub line_mode: bool,
    pub cursor: Option<(i32, i32)>,
    pub linked_cursor: (i32, i32),
    pub scene_has_mouse: bool,
    pub world_x: f32,
    pub world_y: f32,
    pub count: i32,
    pub player_dead: bool,
    pub plan_at_cursor: bool,
    pub plan_at_linked: bool,
    pub valid_place: bool,
    pub overlap_place: bool,
    pub valid_break: bool,
    pub command_selection_available: bool,
    pub command_building_available: bool,
    pub net_active: bool,
    pub possession_allowed: bool,
    pub unit_tapped_controllable: bool,
    pub building_tapped_present: bool,
    pub selected_unit_present: bool,
    pub selected_control_building_present: bool,
    pub try_repair_derelict: bool,
    pub try_stop_mine: bool,
    pub can_tap_player: bool,
    pub config_tap_handled: bool,
    pub tile_tapped_handled: bool,
    pub double_tap_mine: bool,
}

impl Default for MobileTapFrame {
    fn default() -> Self {
        Self {
            state_menu: false,
            locked: false,
            line_mode: false,
            cursor: Some((0, 0)),
            linked_cursor: (0, 0),
            scene_has_mouse: false,
            world_x: 0.0,
            world_y: 0.0,
            count: 1,
            player_dead: false,
            plan_at_cursor: false,
            plan_at_linked: false,
            valid_place: false,
            overlap_place: false,
            valid_break: false,
            command_selection_available: false,
            command_building_available: false,
            net_active: false,
            possession_allowed: false,
            unit_tapped_controllable: false,
            building_tapped_present: false,
            selected_unit_present: false,
            selected_control_building_present: false,
            try_repair_derelict: false,
            try_stop_mine: false,
            can_tap_player: false,
            config_tap_handled: false,
            tile_tapped_handled: false,
            double_tap_mine: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MobileBlockFootprint {
    pub size: i32,
    pub offset: f32,
}

impl MobileBlockFootprint {
    pub const fn new(size: i32, offset: f32) -> Self {
        Self { size, offset }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MobilePlanSnapshot {
    pub plan: BuildPlan,
    pub block: Option<MobileBlockFootprint>,
    pub tile_block: Option<MobileBlockFootprint>,
    pub tile_world_x: f32,
    pub tile_world_y: f32,
    pub tile_present: bool,
}

impl MobilePlanSnapshot {
    pub fn from_plan(plan: BuildPlan, block: Option<MobileBlockFootprint>) -> Self {
        Self {
            tile_world_x: plan.x as f32 * TILE_SIZE as f32,
            tile_world_y: plan.y as f32 * TILE_SIZE as f32,
            tile_present: true,
            tile_block: block,
            block,
            plan,
        }
    }

    pub fn missing_tile(plan: BuildPlan, block: Option<MobileBlockFootprint>) -> Self {
        Self {
            tile_present: false,
            ..Self::from_plan(plan, block)
        }
    }

    pub fn with_tile_block(mut self, tile_block: MobileBlockFootprint) -> Self {
        self.tile_block = Some(tile_block);
        self
    }

    pub fn with_world(mut self, tile_world_x: f32, tile_world_y: f32) -> Self {
        self.tile_world_x = tile_world_x;
        self.tile_world_y = tile_world_y;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MobileRemovePlanResult {
    pub removed: Option<BuildPlan>,
    pub remaining: Vec<BuildPlan>,
    pub removals: Vec<BuildPlan>,
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

    pub fn placement_button(
        &mut self,
        button: MobilePlacementButton,
        rotation: i32,
        selected_block_rotates: bool,
        player_dead: bool,
    ) -> MobileActionPlan {
        let mut actions = Vec::new();

        match button {
            MobilePlacementButton::BreakMode => {
                self.mode = if self.mode == PlaceMode::Breaking {
                    if self.selected_block.is_none() {
                        PlaceMode::None
                    } else {
                        PlaceMode::Placing
                    }
                } else {
                    PlaceMode::Breaking
                };
                self.last_block = self.selected_block.clone();
                actions.push(MobileInputAction::ToggleBreakMode { mode: self.mode });
            }
            MobilePlacementButton::Diagonal => {
                actions.push(MobileInputAction::ToggleDiagonalPlacement);
            }
            MobilePlacementButton::RotateOrSchematic => {
                if self.selected_block.is_some() && selected_block_rotates {
                    actions.push(MobileInputAction::RotateBlock {
                        rotation: (rotation + 1).rem_euclid(4),
                    });
                } else {
                    self.schematic_mode = !self.schematic_mode;
                    if self.schematic_mode {
                        self.selected_block = None;
                        self.mode = PlaceMode::None;
                    } else {
                        self.rebuild_mode = false;
                    }
                    actions.push(MobileInputAction::ToggleSchematicMode {
                        enabled: self.schematic_mode,
                    });
                }
            }
            MobilePlacementButton::Confirm => {
                if self.schematic_mode {
                    self.rebuild_mode = !self.rebuild_mode;
                    actions.push(MobileInputAction::ToggleRebuildMode {
                        enabled: self.rebuild_mode,
                    });
                } else if !player_dead {
                    self.selecting = false;
                    self.select_plans_empty = true;
                    actions.push(MobileInputAction::ConfirmSelectedPlans);
                }
            }
            MobilePlacementButton::Cancel => {
                if !player_dead {
                    actions.push(MobileInputAction::ClearBuilding);
                }
                self.select_plans_empty = true;
                self.mode = PlaceMode::None;
                self.selected_block = None;
                actions.push(MobileInputAction::CancelPlacement);
            }
        }

        MobileActionPlan::accepted(actions)
    }

    pub fn touch_down(&mut self, frame: &MobileTouchDownFrame) -> MobileActionPlan {
        if frame.state_menu || frame.locked {
            return MobileActionPlan::rejected();
        }

        self.down = true;

        if frame.player_dead || frame.cursor.is_none() || frame.scene_has_mouse {
            return MobileActionPlan::accepted(Vec::new());
        }

        let (tile_x, tile_y) = frame.cursor.expect("cursor checked above");
        self.selecting = frame.has_plan_at_cursor && !self.command_mode;

        let mut actions = Vec::new();
        if frame.pointer == 0 && !self.selecting {
            if self.schematic_mode && self.selected_block.is_none() {
                self.mode = if self.rebuild_mode {
                    PlaceMode::RebuildSelect
                } else {
                    PlaceMode::SchematicSelect
                };
                self.line_start_x = tile_x;
                self.line_start_y = tile_y;
                self.last_line_x = tile_x;
                self.last_line_y = tile_y;
                actions.push(MobileInputAction::BeginSchematicSelection {
                    rebuild: self.rebuild_mode,
                    tile_x,
                    tile_y,
                });
            } else if !frame.try_tap_player && frame.keyboard && !frame.state_editor {
                actions.push(MobileInputAction::StartKeyboardShooting);
            }
        }

        MobileActionPlan::accepted(actions)
    }

    pub fn touch_up(&mut self, frame: &MobileTouchUpFrame) -> MobileActionPlan {
        self.last_zoom = frame.renderer_scale;
        if !frame.any_touched {
            self.down = false;
        }
        self.manual_shooting = false;
        self.selecting = false;

        let mut actions = Vec::new();
        if self.line_mode {
            if self.mode == PlaceMode::Placing && self.is_placing() {
                actions.push(MobileInputAction::ConfirmLinePlacement);
            } else if self.mode == PlaceMode::Breaking {
                actions.push(MobileInputAction::ConfirmAreaBreak {
                    start_x: self.line_start_x,
                    start_y: self.line_start_y,
                    end_x: frame.tile_x,
                    end_y: frame.tile_y,
                });
            }
            self.line_mode = false;
        } else if self.mode == PlaceMode::SchematicSelect {
            actions.push(MobileInputAction::CreateSchematicSelection {
                start_x: self.line_start_x,
                start_y: self.line_start_y,
                end_x: self.last_line_x,
                end_y: self.last_line_y,
            });
            self.last_schematic_present = true;
            self.schematic_mode = false;
            self.mode = PlaceMode::None;
        } else if self.mode == PlaceMode::RebuildSelect {
            actions.push(MobileInputAction::RebuildArea {
                start_x: self.line_start_x,
                start_y: self.line_start_y,
                end_x: self.last_line_x,
                end_y: self.last_line_y,
            });
            self.mode = PlaceMode::None;
        } else if !frame.player_dead {
            actions.push(MobileInputAction::TryDropItems {
                tile_present: frame.tile_present,
                world_x: frame.world_x,
                world_y: frame.world_y,
            });
        }

        actions.push(MobileInputAction::SelectUnitsRect);
        MobileActionPlan::accepted(actions)
    }

    pub fn long_press(&mut self, frame: &MobileLongPressFrame) -> MobileActionPlan {
        if frame.state_menu
            || frame.player_dead
            || frame.locked
            || frame.scene_has_mouse
            || self.schematic_mode
        {
            return MobileActionPlan::rejected();
        }

        let mut actions = Vec::new();
        if self.mode == PlaceMode::None {
            if frame.command_mode || self.command_mode {
                actions.push(MobileInputAction::BeginCommandRect {
                    world_x: frame.cursor_world_x,
                    world_y: frame.cursor_world_y,
                });
            } else if frame.payload_unit_available {
                self.payload_target_present = true;
                actions.push(MobileInputAction::SetPayloadTargetUnit);
            } else if frame.payload_building_available {
                self.payload_target_present = true;
                actions.push(MobileInputAction::SetPayloadTargetBuilding);
            } else if frame.payload_has_payload {
                self.payload_target_present = true;
                actions.push(MobileInputAction::SetPayloadDropPosition {
                    world_x: frame.cursor_world_x,
                    world_y: frame.cursor_world_y,
                });
            } else {
                self.manual_shooting = true;
                self.target_present = false;
                actions.push(MobileInputAction::BeginManualShooting);
            }

            if !frame.state_paused {
                actions.push(MobileInputAction::PlaySelectEffect {
                    world_x: frame.cursor_world_x,
                    world_y: frame.cursor_world_y,
                });
            }
        } else {
            let Some((tile_x, tile_y)) = frame.cursor else {
                return MobileActionPlan::rejected();
            };

            self.line_start_x = tile_x;
            self.line_start_y = tile_y;
            self.last_line_x = tile_x;
            self.last_line_y = tile_y;
            self.line_mode = true;

            if self.mode == PlaceMode::Breaking {
                if !frame.state_paused {
                    actions.push(MobileInputAction::PlayTapBlockEffect {
                        world_x: frame.cursor_world_x,
                        world_y: frame.cursor_world_y,
                        size: 1.0,
                    });
                }
            } else if self.selected_block.is_some() {
                actions.push(MobileInputAction::UpdateLine {
                    start_x: tile_x,
                    start_y: tile_y,
                    end_x: tile_x,
                    end_y: tile_y,
                });
                if !frame.state_paused {
                    actions.push(MobileInputAction::PlayTapBlockEffect {
                        world_x: frame.cursor_world_x + frame.selected_block_offset,
                        world_y: frame.cursor_world_y + frame.selected_block_offset,
                        size: frame.selected_block_size,
                    });
                }
            }
        }

        MobileActionPlan::accepted(actions)
    }

    pub fn tap(&mut self, frame: &MobileTapFrame, rotation: i32) -> MobileActionPlan {
        if frame.state_menu || frame.line_mode || self.line_mode || frame.locked {
            return MobileActionPlan::rejected();
        }

        let Some((tile_x, tile_y)) = frame.cursor else {
            return MobileActionPlan::rejected();
        };

        if frame.scene_has_mouse {
            return MobileActionPlan::rejected();
        }

        let mut actions = vec![MobileInputAction::TileTap { tile_x, tile_y }];

        if !frame.player_dead {
            actions.push(MobileInputAction::CheckTargets {
                world_x: frame.world_x,
                world_y: frame.world_y,
            });
        }

        if frame.plan_at_cursor && !self.command_mode {
            actions.push(MobileInputAction::RemovePlanAtCursor);
        } else if self.mode == PlaceMode::Placing
            && self.is_placing()
            && frame.valid_place
            && !frame.overlap_place
        {
            let block = self.selected_block.clone().unwrap_or_default();
            self.last_placed_present = true;
            self.select_plans_empty = false;
            actions.push(MobileInputAction::AddPlacePlan {
                tile_x,
                tile_y,
                rotation,
                block,
            });
        } else if self.mode == PlaceMode::Breaking && frame.valid_break && !frame.plan_at_linked {
            let (linked_x, linked_y) = frame.linked_cursor;
            self.select_plans_empty = false;
            actions.push(MobileInputAction::AddBreakPlan {
                tile_x: linked_x,
                tile_y: linked_y,
            });
        } else if (self.command_mode && frame.command_selection_available)
            || frame.command_building_available
        {
            actions.push(MobileInputAction::CommandTap {
                queue: self.queue_command_mode,
            });
        } else if self.command_mode {
            actions.push(MobileInputAction::TapCommandUnit);
        } else if frame.count == 3 && frame.net_active {
            actions.push(MobileInputAction::PingLocation {
                world_x: frame.world_x,
                world_y: frame.world_y,
            });
        } else if frame.count == 2 {
            self.payload_target_present = false;
            actions.push(MobileInputAction::ResetPayloadTarget);

            if frame.possession_allowed && frame.unit_tapped_controllable {
                actions.push(MobileInputAction::UnitControlTapped);
            } else if frame.possession_allowed && frame.building_tapped_present {
                actions.push(MobileInputAction::BuildingControlTapped);
            } else if !frame.config_tap_handled {
                actions.push(MobileInputAction::FallbackDoubleTap);
            }
        } else {
            self.unit_tapped_present = frame.selected_unit_present;
            self.building_tapped_present = frame.selected_control_building_present;
            actions.push(MobileInputAction::StoreTapCandidates {
                unit: frame.selected_unit_present,
                building: frame.selected_control_building_present,
            });

            if !frame.try_repair_derelict
                && !frame.try_stop_mine
                && !frame.can_tap_player
                && !frame.config_tap_handled
                && !frame.tile_tapped_handled
                && self.mode == PlaceMode::None
                && !frame.double_tap_mine
            {
                actions.push(MobileInputAction::TryBeginMine);
            }
        }

        MobileActionPlan::accepted(actions)
    }
}

pub fn is_line_placing(
    mode: PlaceMode,
    line_mode: bool,
    line_start_x: i32,
    line_start_y: i32,
    mouse_world_x: f32,
    mouse_world_y: f32,
    tile_size: f32,
) -> bool {
    mode == PlaceMode::Placing
        && line_mode
        && distance(
            line_start_x as f32 * tile_size,
            line_start_y as f32 * tile_size,
            mouse_world_x,
            mouse_world_y,
        ) >= 3.0 * tile_size
}

pub fn is_area_breaking(
    mode: PlaceMode,
    line_mode: bool,
    line_start_x: i32,
    line_start_y: i32,
    mouse_world_x: f32,
    mouse_world_y: f32,
    tile_size: f32,
) -> bool {
    mode == PlaceMode::Breaking
        && line_mode
        && distance(
            line_start_x as f32 * tile_size,
            line_start_y as f32 * tile_size,
            mouse_world_x,
            mouse_world_y,
        ) >= 2.0 * tile_size
}

pub fn synced_mobile_plans(plans: &[BuildPlan]) -> Vec<BuildPlan> {
    plans
        .iter()
        .filter(|plan| !plan.breaking)
        .cloned()
        .collect()
}

pub fn remove_mobile_plan(plans: &[BuildPlan], target: &BuildPlan) -> MobileRemovePlanResult {
    let mut removed = None;
    let mut remaining = Vec::with_capacity(plans.len());
    let mut removals = Vec::new();

    for plan in plans {
        if removed.is_none() && plan == target {
            removed = Some(plan.clone());
            if !plan.breaking {
                removals.push(plan.clone());
            }
        } else {
            remaining.push(plan.clone());
        }
    }

    MobileRemovePlanResult {
        removed,
        remaining,
        removals,
    }
}

pub fn plan_rect(
    x: i32,
    y: i32,
    block: MobileBlockFootprint,
    tile_size: f32,
    tile_world_x: Option<f32>,
    tile_world_y: Option<f32>,
) -> Rect {
    let size = block.size as f32 * tile_size;
    rect_centered(
        tile_world_x.unwrap_or(x as f32 * tile_size) + block.offset,
        tile_world_y.unwrap_or(y as f32 * tile_size) + block.offset,
        size,
        size,
    )
}

pub fn get_mobile_plan<'a>(
    plans: &'a [MobilePlanSnapshot],
    tile_world_x: f32,
    tile_world_y: f32,
    tile_size: f32,
) -> Option<&'a BuildPlan> {
    let tile_rect = rect_centered(tile_world_x, tile_world_y, tile_size, tile_size);

    plans.iter().find_map(|snapshot| {
        if !snapshot.tile_present {
            return None;
        }

        let footprint = if snapshot.plan.breaking {
            snapshot.tile_block
        } else {
            snapshot.block
        }?;

        let plan_rect = plan_rect(
            snapshot.plan.x,
            snapshot.plan.y,
            footprint,
            tile_size,
            Some(snapshot.tile_world_x),
            Some(snapshot.tile_world_y),
        );

        tile_rect.overlaps(plan_rect).then_some(&snapshot.plan)
    })
}

pub fn has_mobile_plan(
    plans: &[MobilePlanSnapshot],
    tile_world_x: f32,
    tile_world_y: f32,
    tile_size: f32,
) -> bool {
    get_mobile_plan(plans, tile_world_x, tile_world_y, tile_size).is_some()
}

pub fn check_mobile_overlap_placement(
    x: i32,
    y: i32,
    block: MobileBlockFootprint,
    selected_plans: &[MobilePlanSnapshot],
    unit_plans: &[MobilePlanSnapshot],
    player_dead: bool,
    tile_size: f32,
) -> bool {
    let candidate = plan_rect(x, y, block, tile_size, None, None);
    selected_plans
        .iter()
        .filter(|snapshot| snapshot.tile_present && !snapshot.plan.breaking)
        .chain(
            unit_plans.iter().filter(|snapshot| {
                !player_dead && snapshot.tile_present && !snapshot.plan.breaking
            }),
        )
        .any(|snapshot| {
            snapshot.block.is_some_and(|footprint| {
                candidate.overlaps(plan_rect(
                    snapshot.plan.x,
                    snapshot.plan.y,
                    footprint,
                    tile_size,
                    Some(snapshot.tile_world_x),
                    Some(snapshot.tile_world_y),
                ))
            })
        })
}

pub fn mobile_schematic_origin(plans: &[MobilePlanSnapshot], tile_size: f32) -> Option<(i32, i32)> {
    if plans.is_empty() {
        return None;
    }

    let (sum_x, sum_y) = plans.iter().fold((0.0, 0.0), |(sum_x, sum_y), snapshot| {
        (sum_x + snapshot.tile_world_x, sum_y + snapshot.tile_world_y)
    });
    let inv = 1.0 / plans.len() as f32;
    Some((
        world_to_tile(sum_x * inv, tile_size),
        world_to_tile(sum_y * inv, tile_size),
    ))
}

fn lerp_delta(from: f32, to: f32, alpha: f32, delta: f32) -> f32 {
    let t = (alpha * delta.max(0.0)).clamp(0.0, 1.0);
    from + (to - from) * t
}

fn distance(x: f32, y: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x;
    let dy = y2 - y;
    (dx * dx + dy * dy).sqrt()
}

fn rect_centered(center_x: f32, center_y: f32, width: f32, height: f32) -> Rect {
    Rect::new(
        center_x - width / 2.0,
        center_y - height / 2.0,
        width,
        height,
    )
}

fn world_to_tile(value: f32, tile_size: f32) -> i32 {
    (value / tile_size).floor() as i32
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

    #[test]
    fn line_and_area_thresholds_match_mobile_tile_distances() {
        assert!(!is_line_placing(
            PlaceMode::Placing,
            true,
            0,
            0,
            23.9,
            0.0,
            8.0
        ));
        assert!(is_line_placing(
            PlaceMode::Placing,
            true,
            0,
            0,
            24.0,
            0.0,
            8.0
        ));
        assert!(is_area_breaking(
            PlaceMode::Breaking,
            true,
            1,
            1,
            8.0,
            24.0,
            8.0
        ));
        assert!(!is_area_breaking(
            PlaceMode::Placing,
            true,
            1,
            1,
            8.0,
            24.0,
            8.0
        ));
    }

    #[test]
    fn synced_mobile_plans_filter_breaking_entries() {
        let place = BuildPlan::new_place(1, 2, 0, "router");
        let breaking = BuildPlan::new_break(3, 4);

        assert_eq!(synced_mobile_plans(&[place.clone(), breaking]), vec![place]);
    }

    #[test]
    fn remove_mobile_plan_moves_only_non_breaking_to_removals() {
        let place = BuildPlan::new_place(1, 2, 0, "router");
        let breaking = BuildPlan::new_break(3, 4);
        let keep = BuildPlan::new_place(5, 6, 0, "duo");

        let result = remove_mobile_plan(&[place.clone(), breaking.clone(), keep.clone()], &place);
        assert_eq!(result.removed, Some(place.clone()));
        assert_eq!(result.remaining, vec![breaking.clone(), keep.clone()]);
        assert_eq!(result.removals, vec![place]);

        let break_result = remove_mobile_plan(&[breaking.clone(), keep.clone()], &breaking);
        assert_eq!(break_result.removed, Some(breaking));
        assert_eq!(break_result.remaining, vec![keep]);
        assert!(break_result.removals.is_empty());
    }

    #[test]
    fn get_mobile_plan_uses_plan_or_tile_block_footprint() {
        let duo = MobileBlockFootprint::new(1, 4.0);
        let large = MobileBlockFootprint::new(2, 8.0);
        let plan = MobilePlanSnapshot::from_plan(BuildPlan::new_place(2, 2, 0, "duo"), Some(duo));
        let breaking = MobilePlanSnapshot::from_plan(BuildPlan::new_break(10, 10), None)
            .with_tile_block(large);

        assert_eq!(
            get_mobile_plan(&[plan.clone()], 16.0, 16.0, 8.0),
            Some(&plan.plan)
        );
        assert!(has_mobile_plan(&[breaking.clone()], 80.0, 80.0, 8.0));
        assert_eq!(
            get_mobile_plan(
                &[MobilePlanSnapshot::missing_tile(
                    plan.plan.clone(),
                    Some(duo)
                )],
                16.0,
                16.0,
                8.0,
            ),
            None
        );
    }

    #[test]
    fn check_mobile_overlap_placement_compares_selected_and_unit_plans() {
        let small = MobileBlockFootprint::new(1, 4.0);
        let large = MobileBlockFootprint::new(2, 8.0);
        let selected =
            MobilePlanSnapshot::from_plan(BuildPlan::new_place(3, 3, 0, "duo"), Some(small));
        let unit =
            MobilePlanSnapshot::from_plan(BuildPlan::new_place(8, 8, 0, "large"), Some(large));

        assert!(check_mobile_overlap_placement(
            3,
            3,
            small,
            &[selected.clone()],
            &[],
            false,
            8.0
        ));
        assert!(check_mobile_overlap_placement(
            9,
            8,
            small,
            &[],
            &[unit.clone()],
            false,
            8.0
        ));
        assert!(!check_mobile_overlap_placement(
            9,
            8,
            small,
            &[],
            &[unit],
            true,
            8.0
        ));
    }

    #[test]
    fn mobile_schematic_origin_averages_plan_draw_positions_as_tiles() {
        let small = MobileBlockFootprint::new(1, 4.0);
        let plans = vec![
            MobilePlanSnapshot::from_plan(BuildPlan::new_place(0, 0, 0, "router"), Some(small))
                .with_world(0.0, 8.0),
            MobilePlanSnapshot::from_plan(BuildPlan::new_place(2, 4, 0, "router"), Some(small))
                .with_world(16.0, 24.0),
        ];

        assert_eq!(mobile_schematic_origin(&plans, 8.0), Some((1, 2)));
        assert_eq!(mobile_schematic_origin(&[], 8.0), None);
    }

    #[test]
    fn placement_buttons_toggle_break_rotate_schematic_confirm_and_cancel() {
        let mut input = MobileInput::new();
        input.selected_block = Some("duo".into());

        let break_plan = input.placement_button(MobilePlacementButton::BreakMode, 0, false, false);
        assert_eq!(input.mode, PlaceMode::Breaking);
        assert_eq!(input.last_block, Some("duo".into()));
        assert_eq!(
            break_plan.actions,
            vec![MobileInputAction::ToggleBreakMode {
                mode: PlaceMode::Breaking
            }]
        );

        let rotate =
            input.placement_button(MobilePlacementButton::RotateOrSchematic, 3, true, false);
        assert_eq!(
            rotate.actions,
            vec![MobileInputAction::RotateBlock { rotation: 0 }]
        );

        input.selected_block = None;
        let schematic =
            input.placement_button(MobilePlacementButton::RotateOrSchematic, 0, false, false);
        assert!(input.schematic_mode);
        assert_eq!(input.mode, PlaceMode::None);
        assert_eq!(
            schematic.actions,
            vec![MobileInputAction::ToggleSchematicMode { enabled: true }]
        );

        let confirm = input.placement_button(MobilePlacementButton::Confirm, 0, false, false);
        assert!(input.rebuild_mode);
        assert_eq!(
            confirm.actions,
            vec![MobileInputAction::ToggleRebuildMode { enabled: true }]
        );

        let cancel = input.placement_button(MobilePlacementButton::Cancel, 0, false, false);
        assert_eq!(input.mode, PlaceMode::None);
        assert!(input.select_plans_empty);
        assert_eq!(
            cancel.actions,
            vec![
                MobileInputAction::ClearBuilding,
                MobileInputAction::CancelPlacement
            ]
        );
    }

    #[test]
    fn touch_down_begins_schematic_selection_or_keyboard_shooting() {
        let mut input = MobileInput::new();
        input.schematic_mode = true;
        input.rebuild_mode = true;

        let plan = input.touch_down(&MobileTouchDownFrame {
            cursor: Some((7, 9)),
            ..MobileTouchDownFrame::default()
        });

        assert!(plan.accepted);
        assert!(input.down);
        assert_eq!(input.mode, PlaceMode::RebuildSelect);
        assert_eq!((input.line_start_x, input.line_start_y), (7, 9));
        assert_eq!(
            plan.actions,
            vec![MobileInputAction::BeginSchematicSelection {
                rebuild: true,
                tile_x: 7,
                tile_y: 9,
            }]
        );

        let mut keyboard = MobileInput::new();
        let shoot = keyboard.touch_down(&MobileTouchDownFrame {
            keyboard: true,
            try_tap_player: false,
            ..MobileTouchDownFrame::default()
        });
        assert_eq!(
            shoot.actions,
            vec![MobileInputAction::StartKeyboardShooting]
        );
    }

    #[test]
    fn touch_up_finishes_line_schematic_rebuild_or_drop_paths() {
        let mut line = MobileInput::new();
        line.line_mode = true;
        line.mode = PlaceMode::Placing;
        line.selected_block = Some("conveyor".into());

        let line_plan = line.touch_up(&MobileTouchUpFrame {
            renderer_scale: 2.0,
            ..MobileTouchUpFrame::default()
        });

        assert_eq!(line.last_zoom, 2.0);
        assert!(!line.down);
        assert!(!line.line_mode);
        assert!(line_plan
            .actions
            .contains(&MobileInputAction::ConfirmLinePlacement));
        assert!(line_plan
            .actions
            .contains(&MobileInputAction::SelectUnitsRect));

        let mut schematic = MobileInput::new();
        schematic.mode = PlaceMode::SchematicSelect;
        schematic.schematic_mode = true;
        schematic.line_start_x = 1;
        schematic.line_start_y = 2;
        schematic.last_line_x = 3;
        schematic.last_line_y = 4;
        let schematic_plan = schematic.touch_up(&MobileTouchUpFrame::default());
        assert_eq!(schematic.mode, PlaceMode::None);
        assert!(!schematic.schematic_mode);
        assert_eq!(
            schematic_plan.actions[0],
            MobileInputAction::CreateSchematicSelection {
                start_x: 1,
                start_y: 2,
                end_x: 3,
                end_y: 4,
            }
        );

        let mut dropping = MobileInput::new();
        let drop_plan = dropping.touch_up(&MobileTouchUpFrame {
            tile_present: true,
            world_x: 12.0,
            world_y: 34.0,
            ..MobileTouchUpFrame::default()
        });
        assert!(drop_plan
            .actions
            .contains(&MobileInputAction::TryDropItems {
                tile_present: true,
                world_x: 12.0,
                world_y: 34.0,
            }));
    }

    #[test]
    fn long_press_starts_command_rect_payload_manual_or_line_mode() {
        let mut command = MobileInput::new();
        command.command_mode = true;
        let command_plan = command.long_press(&MobileLongPressFrame {
            cursor_world_x: 5.0,
            cursor_world_y: 6.0,
            ..MobileLongPressFrame::default()
        });
        assert!(command_plan
            .actions
            .contains(&MobileInputAction::BeginCommandRect {
                world_x: 5.0,
                world_y: 6.0,
            }));

        let mut manual = MobileInput::new();
        manual.target_present = true;
        let manual_plan = manual.long_press(&MobileLongPressFrame::default());
        assert!(manual.manual_shooting);
        assert!(!manual.target_present);
        assert!(manual_plan
            .actions
            .contains(&MobileInputAction::BeginManualShooting));

        let mut line = MobileInput::new();
        line.mode = PlaceMode::Breaking;
        let line_plan = line.long_press(&MobileLongPressFrame {
            cursor: Some((4, 5)),
            cursor_world_x: 32.0,
            cursor_world_y: 40.0,
            ..MobileLongPressFrame::default()
        });
        assert!(line.line_mode);
        assert_eq!((line.line_start_x, line.line_start_y), (4, 5));
        assert!(line_plan
            .actions
            .contains(&MobileInputAction::PlayTapBlockEffect {
                world_x: 32.0,
                world_y: 40.0,
                size: 1.0,
            }));
    }

    #[test]
    fn tap_emits_place_break_command_ping_and_double_tap_intents() {
        let mut place = MobileInput::new();
        place.mode = PlaceMode::Placing;
        place.selected_block = Some("router".into());
        let place_plan = place.tap(
            &MobileTapFrame {
                cursor: Some((2, 3)),
                valid_place: true,
                ..MobileTapFrame::default()
            },
            1,
        );
        assert_eq!(
            place_plan.actions,
            vec![
                MobileInputAction::TileTap {
                    tile_x: 2,
                    tile_y: 3
                },
                MobileInputAction::CheckTargets {
                    world_x: 0.0,
                    world_y: 0.0,
                },
                MobileInputAction::AddPlacePlan {
                    tile_x: 2,
                    tile_y: 3,
                    rotation: 1,
                    block: "router".into(),
                },
            ]
        );

        let mut breaking = MobileInput::new();
        breaking.mode = PlaceMode::Breaking;
        let break_plan = breaking.tap(
            &MobileTapFrame {
                cursor: Some((2, 3)),
                linked_cursor: (9, 10),
                valid_break: true,
                ..MobileTapFrame::default()
            },
            0,
        );
        assert!(break_plan
            .actions
            .contains(&MobileInputAction::AddBreakPlan {
                tile_x: 9,
                tile_y: 10,
            }));

        let mut command = MobileInput::new();
        command.command_mode = true;
        command.queue_command_mode = true;
        let command_plan = command.tap(
            &MobileTapFrame {
                command_selection_available: true,
                ..MobileTapFrame::default()
            },
            0,
        );
        assert!(command_plan
            .actions
            .contains(&MobileInputAction::CommandTap { queue: true }));

        let mut ping = MobileInput::new();
        let ping_plan = ping.tap(
            &MobileTapFrame {
                count: 3,
                net_active: true,
                world_x: 11.0,
                world_y: 22.0,
                ..MobileTapFrame::default()
            },
            0,
        );
        assert!(ping_plan
            .actions
            .contains(&MobileInputAction::PingLocation {
                world_x: 11.0,
                world_y: 22.0,
            }));

        let mut double = MobileInput::new();
        double.payload_target_present = true;
        let double_plan = double.tap(
            &MobileTapFrame {
                count: 2,
                possession_allowed: true,
                unit_tapped_controllable: true,
                ..MobileTapFrame::default()
            },
            0,
        );
        assert!(!double.payload_target_present);
        assert!(double_plan
            .actions
            .contains(&MobileInputAction::UnitControlTapped));
    }
}
