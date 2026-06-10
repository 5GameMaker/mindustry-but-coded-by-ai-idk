//! Minimap overlay state model mirroring upstream `mindustry.ui.fragments.MinimapFragment`.

use crate::mindustry::entities::entity_group::Rect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapTexture {
    pub width: f32,
    pub height: f32,
}

impl MinimapTexture {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn ratio(self) -> f32 {
        self.height / self.width
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapWorld {
    pub width_tiles: f32,
    pub height_tiles: f32,
    pub tile_size: f32,
}

impl MinimapWorld {
    pub const fn new(width_tiles: f32, height_tiles: f32, tile_size: f32) -> Self {
        Self {
            width_tiles,
            height_tiles,
            tile_size,
        }
    }

    pub fn unit_width(self) -> f32 {
        self.width_tiles * self.tile_size
    }

    pub fn unit_height(self) -> f32 {
        self.height_tiles * self.tile_size
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapGraphics {
    pub width: f32,
    pub height: f32,
}

impl MinimapGraphics {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapSceneMargins {
    pub left: f32,
    pub bottom: f32,
}

impl MinimapSceneMargins {
    pub const fn new(left: f32, bottom: f32) -> Self {
        Self { left, bottom }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapToggleFocus {
    pub player_dead: bool,
    pub player_x: f32,
    pub player_y: f32,
    pub camera_x: f32,
    pub camera_y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimapDrawPlan {
    pub visible: bool,
    pub black_rect: Rect,
    pub texture_rect: Option<Rect>,
    pub entity_bounds: Option<Rect>,
    pub title_key: &'static str,
    pub back_button_key: &'static str,
    pub back_button_size: (f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MinimapAction {
    RequestKeyboard,
    RequestScroll,
    Hide,
    PanCamera {
        x: f32,
        y: f32,
    },
    PingLocation {
        x: f32,
        y: f32,
        text: Option<String>,
    },
    ShowPingTextInput {
        x: f32,
        y: f32,
        max_len: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapUpdateContext {
    pub chat_shown: bool,
    pub keyboard_focus_text_field: bool,
    pub menu_key_tapped: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapFragment {
    shown: bool,
    panx: f32,
    pany: f32,
    zoom: f32,
    last_zoom: f32,
    base_size: f32,
}

impl MinimapFragment {
    pub fn new(scl: f32) -> Self {
        Self {
            shown: false,
            panx: 0.0,
            pany: 0.0,
            zoom: 1.0,
            last_zoom: -1.0,
            base_size: scl * 5.0,
        }
    }

    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn hide(&mut self) {
        self.shown = false;
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn pan(&self) -> (f32, f32) {
        (self.panx, self.pany)
    }

    pub fn rect_bounds(
        &self,
        graphics: MinimapGraphics,
        world: MinimapWorld,
        texture: Option<MinimapTexture>,
    ) -> Rect {
        let ratio = texture.map_or(1.0, MinimapTexture::ratio);
        let size = self.base_size * self.zoom * world.width_tiles;
        Rect::new(
            graphics.width / 2.0 + self.panx * self.zoom - size / 2.0,
            graphics.height / 2.0 + self.pany * self.zoom - size / 2.0 * ratio,
            size,
            size * ratio,
        )
    }

    pub fn draw_plan(
        &self,
        graphics: MinimapGraphics,
        world: MinimapWorld,
        texture: Option<MinimapTexture>,
    ) -> MinimapDrawPlan {
        let bounds = texture.map(|_| self.rect_bounds(graphics, world, texture));
        MinimapDrawPlan {
            visible: self.shown,
            black_rect: Rect::new(0.0, 0.0, graphics.width, graphics.height),
            texture_rect: bounds,
            entity_bounds: bounds,
            title_key: "@minimap",
            back_button_key: "@back",
            back_button_size: (220.0, 60.0),
        }
    }

    pub fn update(&mut self, context: MinimapUpdateContext) -> Vec<MinimapAction> {
        let mut actions = Vec::new();
        if !context.chat_shown && !context.keyboard_focus_text_field {
            actions.push(MinimapAction::RequestKeyboard);
            actions.push(MinimapAction::RequestScroll);
        }
        if context.menu_key_tapped {
            self.shown = false;
            actions.push(MinimapAction::Hide);
        }
        actions
    }

    pub fn gesture_zoom(&mut self, initial_distance: f32, distance: f32) {
        if self.last_zoom < 0.0 {
            self.last_zoom = self.zoom;
        }
        self.zoom = (distance / initial_distance * self.last_zoom).clamp(0.25, 10.0);
    }

    pub fn gesture_pan(
        &mut self,
        x: f32,
        y: f32,
        delta_x: f32,
        delta_y: f32,
        right_button: bool,
        env: MinimapConvertEnv,
    ) -> Option<MinimapAction> {
        if right_button {
            Some(self.pan_to(x, y, env))
        } else {
            self.panx += delta_x / self.zoom;
            self.pany += delta_y / self.zoom;
            None
        }
    }

    pub fn touch_down(
        &self,
        x: f32,
        y: f32,
        right_button: bool,
        env: MinimapConvertEnv,
    ) -> Option<MinimapAction> {
        right_button.then(|| self.pan_to_action(x, y, env))
    }

    pub fn tap(
        &self,
        x: f32,
        y: f32,
        count: i32,
        mobile: bool,
        env: MinimapConvertEnv,
    ) -> Option<MinimapAction> {
        if mobile && count == 2 {
            let pos = self.convert(x, y, env);
            Some(MinimapAction::PingLocation {
                x: pos.0,
                y: pos.1,
                text: None,
            })
        } else {
            None
        }
    }

    pub fn touch_up(&mut self) {
        self.last_zoom = self.zoom;
    }

    pub fn scrolled(&mut self, amount_y: f32) {
        self.zoom = (self.zoom - amount_y / 10.0 * self.zoom).clamp(0.25, 10.0);
    }

    pub fn key_down_ping(
        &self,
        stage_x: f32,
        stage_y: f32,
        ctrl: bool,
        max_ping_text_length: usize,
        env: MinimapConvertEnv,
    ) -> MinimapAction {
        let pos = self.convert(stage_x, stage_y, env);
        if ctrl {
            MinimapAction::ShowPingTextInput {
                x: pos.0,
                y: pos.1,
                max_len: max_ping_text_length,
            }
        } else {
            MinimapAction::PingLocation {
                x: pos.0,
                y: pos.1,
                text: None,
            }
        }
    }

    pub fn toggle(
        &mut self,
        graphics: MinimapGraphics,
        world: MinimapWorld,
        texture: Option<MinimapTexture>,
        focus: MinimapToggleFocus,
    ) {
        if let Some(texture) = texture {
            let size = self.base_size * self.zoom * world.width_tiles;
            let ratio = texture.ratio();
            let px = if focus.player_dead {
                focus.camera_x
            } else {
                focus.player_x
            };
            let py = if focus.player_dead {
                focus.camera_y
            } else {
                focus.player_y
            };
            self.panx =
                (size / 2.0 - px / (world.width_tiles * world.tile_size) * size) / self.zoom;
            self.pany = (size * ratio / 2.0
                - py / (world.height_tiles * world.tile_size) * size * ratio)
                / self.zoom;
            let _ = graphics;
        }

        self.shown = !self.shown;
    }

    pub fn pan_to(
        &self,
        relative_x: f32,
        relative_y: f32,
        env: MinimapConvertEnv,
    ) -> MinimapAction {
        self.pan_to_action(relative_x, relative_y, env)
    }

    fn pan_to_action(
        &self,
        relative_x: f32,
        relative_y: f32,
        env: MinimapConvertEnv,
    ) -> MinimapAction {
        let (x, y) = self.convert(relative_x, relative_y, env);
        let min = -env.world.tile_size / 2.0;
        let max_x = env.world.unit_width() + env.world.tile_size / 2.0;
        let max_y = env.world.unit_height() + env.world.tile_size / 2.0;
        MinimapAction::PanCamera {
            x: x.clamp(min, max_x),
            y: y.clamp(min, max_y),
        }
    }

    pub fn convert(&self, relative_x: f32, relative_y: f32, env: MinimapConvertEnv) -> (f32, f32) {
        let r = self.rect_bounds(env.graphics, env.world, env.texture);
        let x = (relative_x - (r.x - env.scene_margins.left))
            * (1.0 / r.width)
            * env.world.unit_width()
            - env.world.tile_size / 2.0;
        let y = (relative_y - (r.y - env.scene_margins.bottom))
            * (1.0 / r.height)
            * env.world.unit_height()
            - env.world.tile_size / 2.0;
        (x, y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapConvertEnv {
    pub graphics: MinimapGraphics,
    pub world: MinimapWorld,
    pub texture: Option<MinimapTexture>,
    pub scene_margins: MinimapSceneMargins,
}

impl MinimapConvertEnv {
    pub const fn new(
        graphics: MinimapGraphics,
        world: MinimapWorld,
        texture: Option<MinimapTexture>,
        scene_margins: MinimapSceneMargins,
    ) -> Self {
        Self {
            graphics,
            world,
            texture,
            scene_margins,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> MinimapConvertEnv {
        MinimapConvertEnv::new(
            MinimapGraphics::new(800.0, 600.0),
            MinimapWorld::new(100.0, 50.0, 8.0),
            Some(MinimapTexture::new(200.0, 100.0)),
            MinimapSceneMargins::new(0.0, 0.0),
        )
    }

    #[test]
    fn rect_bounds_match_java_formula() {
        let fragment = MinimapFragment::new(1.0);
        let rect = fragment.rect_bounds(
            MinimapGraphics::new(800.0, 600.0),
            MinimapWorld::new(100.0, 50.0, 8.0),
            Some(MinimapTexture::new(200.0, 100.0)),
        );

        assert_eq!(rect, Rect::new(150.0, 175.0, 500.0, 250.0));
    }

    #[test]
    fn convert_maps_stage_coordinates_into_world_units_like_java() {
        let fragment = MinimapFragment::new(1.0);
        let converted = fragment.convert(400.0, 300.0, env());

        assert_eq!(converted, (396.0, 196.0));
    }

    #[test]
    fn toggle_centers_minimap_on_player_when_texture_exists() {
        let mut fragment = MinimapFragment::new(1.0);
        fragment.toggle(
            MinimapGraphics::new(800.0, 600.0),
            MinimapWorld::new(100.0, 50.0, 8.0),
            Some(MinimapTexture::new(200.0, 100.0)),
            MinimapToggleFocus {
                player_dead: false,
                player_x: 400.0,
                player_y: 200.0,
                camera_x: 0.0,
                camera_y: 0.0,
            },
        );

        assert!(fragment.shown());
        assert_eq!(fragment.pan(), (0.0, 0.0));
    }

    #[test]
    fn scroll_and_gesture_zoom_clamp_to_java_bounds() {
        let mut fragment = MinimapFragment::new(1.0);
        fragment.scrolled(-100.0);
        assert_eq!(fragment.zoom(), 10.0);
        fragment.scrolled(100.0);
        assert_eq!(fragment.zoom(), 0.25);

        fragment.gesture_zoom(100.0, 1000.0);
        assert_eq!(fragment.zoom(), 2.5);
    }

    #[test]
    fn ping_key_uses_convert_and_ctrl_requests_text_input() {
        let fragment = MinimapFragment::new(1.0);

        assert_eq!(
            fragment.key_down_ping(400.0, 300.0, true, 64, env()),
            MinimapAction::ShowPingTextInput {
                x: 396.0,
                y: 196.0,
                max_len: 64
            }
        );
    }
}
