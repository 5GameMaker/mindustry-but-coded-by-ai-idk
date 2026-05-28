use std::collections::BTreeSet;

/// Backend-neutral migration of upstream `MinimapRenderer`.
///
/// The Java implementation owns a `Pixmap`/`Texture`, batches tile updates
/// every two frames, computes a cropped texture region from camera/zoom, and
/// draws entity/fog/spawn/indicator overlays directly.  The Rust core keeps the
/// same state and calculations as data plans so desktop/mobile render backends
/// can consume them without depending on GL objects.
pub const MINIMAP_BASE_SIZE: f32 = 16.0;
pub const MINIMAP_UPDATE_INTERVAL: f32 = 2.0;
pub const MINIMAP_DEFAULT_ZOOM: f32 = 4.0;
pub const MINIMAP_TILE_SIZE: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MinimapTilePos {
    pub x: i32,
    pub y: i32,
}

impl MinimapTilePos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn pack(self) -> i32 {
        ((self.x & 0xffff) << 16) | (self.y & 0xffff)
    }

    pub fn unpack(pos: i32) -> Self {
        let x = ((pos >> 16) & 0xffff) as i16 as i32;
        let y = (pos & 0xffff) as i16 as i32;
        Self::new(x, y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimapWorldSize {
    pub width: i32,
    pub height: i32,
}

impl MinimapWorldSize {
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    pub fn min_side(self) -> i32 {
        self.width.min(self.height)
    }

    pub fn unit_width(self, tile_size: f32) -> f32 {
        self.width as f32 * tile_size
    }

    pub fn unit_height(self, tile_size: f32) -> f32 {
        self.height as f32 * tile_size
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapCamera {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl MinimapCamera {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn tile_x(self, tile_size: f32) -> f32 {
        self.x / tile_size
    }

    pub fn tile_y(self, tile_size: f32) -> f32 {
        self.y / tile_size
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl MinimapRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapRegion {
    pub u: f32,
    pub v: f32,
    pub u2: f32,
    pub v2: f32,
    pub source_rect_tiles: MinimapRect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapTileSnapshot {
    pub pos: MinimapTilePos,
    pub real_block_color: u32,
    pub fallback_map_color: u32,
    pub floor_color: u32,
    pub block_is_air: bool,
    pub overlay_is_air: bool,
    pub above_real_block_solid: bool,
    pub floor_is_liquid: bool,
    pub above_floor_is_liquid: bool,
    pub darkness: f32,
}

impl MinimapTileSnapshot {
    pub const fn new(pos: MinimapTilePos, fallback_map_color: u32) -> Self {
        Self {
            pos,
            real_block_color: 0,
            fallback_map_color,
            floor_color: 0,
            block_is_air: false,
            overlay_is_air: false,
            above_real_block_solid: false,
            floor_is_liquid: false,
            above_floor_is_liquid: false,
            darkness: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimapPixelUpdate {
    pub pos: MinimapTilePos,
    pub pixmap_x: i32,
    pub pixmap_y: i32,
    pub rgba: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimapTileUpdatePlan {
    pub updates: Vec<MinimapPixelUpdate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimapResetPlan {
    pub width: i32,
    pub height: i32,
    pub recreate_texture: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimapFullUpdatePlan {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimapTextureSize {
    pub width: i32,
    pub height: i32,
}

impl MinimapTextureSize {
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

impl From<MinimapWorldSize> for MinimapTextureSize {
    fn from(value: MinimapWorldSize) -> Self {
        Self::new(value.width, value.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimapTexturePixelUpdate {
    pub pos: MinimapTilePos,
    pub texture_x: i32,
    pub texture_y: i32,
    pub rgba: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimapTextureFramePlan {
    pub texture_size: MinimapTextureSize,
    pub recreate_texture: bool,
    pub full_upload: Option<MinimapFullUpdatePlan>,
    pub dirty_pixels: Vec<MinimapTexturePixelUpdate>,
}

impl MinimapTextureFramePlan {
    pub fn new(texture_size: MinimapTextureSize) -> Self {
        Self {
            texture_size,
            recreate_texture: false,
            full_upload: None,
            dirty_pixels: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinimapFogLayer {
    Dynamic,
    Static,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MinimapOverlayCommand {
    UnitIcon {
        entity_id: u64,
        x: f32,
        y: f32,
        rotation: f32,
        team_color: u32,
        region: String,
        scale: f32,
    },
    PlayerLabel {
        player_id: u64,
        x: f32,
        y: f32,
        text: String,
        color: u32,
        background: bool,
    },
    Ping {
        player_id: u64,
        x: f32,
        y: f32,
        text: Option<String>,
        color: u32,
    },
    FogTexture {
        layer: MinimapFogLayer,
        color: u32,
        alpha: f32,
    },
    Spawn {
        x: f32,
        y: f32,
        radius: f32,
        pulse: f32,
        team_color: u32,
    },
    CameraBounds(MinimapRect),
    Indicator {
        tile: MinimapTilePos,
        radius: f32,
        alpha: f32,
        color_from: u32,
        color_to: u32,
    },
    Marker {
        id: u64,
        x: f32,
        y: f32,
        label: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimapEntitySnapshot {
    pub entity_id: u64,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub team_color: u32,
    pub region: String,
    pub draw_minimap: bool,
    pub hidden_by_fog: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimapPlayerSnapshot {
    pub player_id: u64,
    pub x: f32,
    pub y: f32,
    pub name: String,
    pub color: u32,
    pub dead: bool,
    pub ping_time: f32,
    pub ping_x: f32,
    pub ping_y: f32,
    pub ping_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapSpawnSnapshot {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimapMarkerSnapshot {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub label: String,
    pub minimap: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapIndicatorSnapshot {
    pub tile: MinimapTilePos,
    pub time: f32,
    pub block_offset_tiles: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimapOverlayInput {
    pub screen_x: f32,
    pub screen_y: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub full_view: bool,
    pub mobile: bool,
    pub net_active: bool,
    pub show_pings: bool,
    pub fog: bool,
    pub static_fog: bool,
    pub dynamic_color: u32,
    pub dynamic_alpha: f32,
    pub show_spawns: bool,
    pub has_spawns: bool,
    pub waves: bool,
    pub wave_team_color: u32,
    pub drop_zone_radius: f32,
    pub time: f32,
    pub global_time: f32,
    pub units: Vec<MinimapEntitySnapshot>,
    pub players: Vec<MinimapPlayerSnapshot>,
    pub spawns: Vec<MinimapSpawnSnapshot>,
    pub indicators: Vec<MinimapIndicatorSnapshot>,
    pub markers: Vec<MinimapMarkerSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimapOverlayPlan {
    pub world_rect: MinimapRect,
    pub scale_factor: f32,
    pub commands: Vec<MinimapOverlayCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimapRendererState {
    pub world: MinimapWorldSize,
    pub zoom: f32,
    pub update_counter: f32,
    pub updates: BTreeSet<i32>,
    pub has_pixmap: bool,
    pub has_texture: bool,
    pub tile_size: f32,
}

impl MinimapRendererState {
    pub fn new(world: MinimapWorldSize) -> Self {
        let mut state = Self {
            world,
            zoom: MINIMAP_DEFAULT_ZOOM,
            update_counter: 0.0,
            updates: BTreeSet::new(),
            has_pixmap: true,
            has_texture: true,
            tile_size: MINIMAP_TILE_SIZE,
        };
        state.set_zoom(MINIMAP_DEFAULT_ZOOM);
        state
    }

    pub fn reset(&mut self, world: MinimapWorldSize) -> MinimapResetPlan {
        self.updates.clear();
        self.update_counter = 0.0;
        self.world = world;
        self.has_pixmap = true;
        self.has_texture = true;
        self.set_zoom(MINIMAP_DEFAULT_ZOOM);
        MinimapResetPlan {
            width: world.width,
            height: world.height,
            recreate_texture: true,
        }
    }

    pub fn update_all(&self) -> Option<MinimapFullUpdatePlan> {
        if !self.has_pixmap || !self.has_texture || self.world.width <= 0 || self.world.height <= 0
        {
            return None;
        }

        Some(MinimapFullUpdatePlan {
            width: self.world.width,
            height: self.world.height,
        })
    }

    pub fn texture_size(&self) -> MinimapTextureSize {
        self.world.into()
    }

    pub fn zoom_by(&mut self, amount: f32) {
        self.set_zoom(self.zoom + amount);
    }

    pub fn set_zoom(&mut self, amount: f32) {
        let max_zoom = (self.world.min_side() as f32 / MINIMAP_BASE_SIZE / 2.0).max(1.0);
        self.zoom = arc_clamp(amount, 1.0, max_zoom);
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }

    pub fn update_pixel(&mut self, pos: MinimapTilePos) {
        self.updates.insert(pos.pack());
    }

    pub fn update_tile(&mut self, center: MinimapTilePos, linked: &[MinimapTilePos], solid: bool) {
        for &other in linked {
            if other != center {
                self.update_pixel(other);
            }
            if solid && other.y > 0 {
                self.update_pixel(MinimapTilePos::new(other.x, other.y - 1));
            }
        }
        self.update_pixel(center);
    }

    pub fn update_tick(
        &mut self,
        delta: f32,
        snapshots: &[MinimapTileSnapshot],
    ) -> Option<MinimapTileUpdatePlan> {
        self.update_counter += delta;
        if self.update_counter < MINIMAP_UPDATE_INTERVAL {
            return None;
        }

        self.update_counter %= MINIMAP_UPDATE_INTERVAL;
        if !self.has_pixmap || !self.has_texture || self.world.width <= 0 || self.world.height <= 0
        {
            self.updates.clear();
            return None;
        }

        let mut updates = Vec::new();
        for packed in &self.updates {
            let pos = MinimapTilePos::unpack(*packed);
            if let Some(snapshot) = snapshots.iter().find(|snapshot| snapshot.pos == pos) {
                updates.push(MinimapPixelUpdate {
                    pos,
                    pixmap_x: pos.x,
                    pixmap_y: pixmap_y(self.world.height, pos.y),
                    rgba: color_for(*snapshot),
                });
            }
        }
        self.updates.clear();

        Some(MinimapTileUpdatePlan { updates })
    }

    pub fn texture_frame_plan(&self) -> Option<MinimapTextureFramePlan> {
        self.update_all()
            .map(MinimapFullUpdatePlan::texture_frame_plan)
    }

    pub fn region(&self, camera: MinimapCamera) -> Option<MinimapRegion> {
        if !self.has_texture || self.world.width <= 0 || self.world.height <= 0 {
            return None;
        }

        let sz = arc_clamp(
            MINIMAP_BASE_SIZE * self.zoom,
            MINIMAP_BASE_SIZE,
            self.world.min_side() as f32,
        );
        let mut dx = camera.tile_x(self.tile_size);
        let mut dy = camera.tile_y(self.tile_size);
        dx = arc_clamp(dx, sz, self.world.width as f32 - sz);
        dy = arc_clamp(dy, sz, self.world.height as f32 - sz);

        let x = dx - sz;
        let y = self.world.height as f32 - dy - sz;
        let width = sz * 2.0;
        let height = sz * 2.0;
        let inv_w = 1.0 / self.world.width as f32;
        let inv_h = 1.0 / self.world.height as f32;

        Some(MinimapRegion {
            u: x * inv_w,
            v: y * inv_h,
            u2: (x + width) * inv_w,
            v2: (y + height) * inv_h,
            source_rect_tiles: MinimapRect::new(x, y, width, height),
        })
    }

    pub fn visible_world_rect(&self, camera: MinimapCamera, full_view: bool) -> MinimapRect {
        if full_view {
            return MinimapRect::new(
                0.0,
                0.0,
                self.world.unit_width(self.tile_size),
                self.world.unit_height(self.tile_size),
            );
        }

        if self.world.width <= 0 || self.world.height <= 0 {
            return MinimapRect::new(0.0, 0.0, 0.0, 0.0);
        }

        let sz = MINIMAP_BASE_SIZE * self.zoom;
        let mut dx = camera.tile_x(self.tile_size);
        let mut dy = camera.tile_y(self.tile_size);
        dx = arc_clamp(dx, sz, self.world.width as f32 - sz);
        dy = arc_clamp(dy, sz, self.world.height as f32 - sz);
        MinimapRect::new(
            (dx - sz) * self.tile_size,
            (dy - sz) * self.tile_size,
            sz * 2.0 * self.tile_size,
            sz * 2.0 * self.tile_size,
        )
    }

    pub fn overlay_plan(
        &self,
        camera: MinimapCamera,
        input: MinimapOverlayInput,
    ) -> MinimapOverlayPlan {
        let world_rect = self.visible_world_rect(camera, input.full_view);
        let scale_factor = if input.full_view {
            let world_width = self.world.unit_width(self.tile_size);
            if world_width <= 0.0 {
                0.0
            } else {
                input.screen_width / world_width
            }
        } else if world_rect.width <= 0.0 {
            0.0
        } else {
            input.screen_width / world_rect.width
        };
        let inverse_scale = if scale_factor == 0.0 {
            0.0
        } else {
            1.0 / scale_factor
        };
        let mut commands = Vec::new();

        for unit in input
            .units
            .iter()
            .filter(|unit| unit.draw_minimap && !unit.hidden_by_fog)
        {
            commands.push(MinimapOverlayCommand::UnitIcon {
                entity_id: unit.entity_id,
                x: unit.x,
                y: unit.y,
                rotation: unit.rotation - 90.0,
                team_color: unit.team_color,
                region: unit.region.clone(),
                scale: self.tile_size * 3.0,
            });
        }

        if input.full_view {
            for player in input
                .players
                .iter()
                .filter(|player| !player.dead && input.net_active)
            {
                commands.push(MinimapOverlayCommand::PlayerLabel {
                    player_id: player.player_id,
                    x: player.x,
                    y: player.y,
                    text: player.name.clone(),
                    color: player.color,
                    background: true,
                });
                if player.ping_time > 0.0 && input.show_pings {
                    commands.push(MinimapOverlayCommand::Ping {
                        player_id: player.player_id,
                        x: player.ping_x,
                        y: player.ping_y,
                        text: player.ping_text.clone(),
                        color: player.color,
                    });
                }
            }
        }

        if input.fog {
            commands.push(MinimapOverlayCommand::FogTexture {
                layer: MinimapFogLayer::Dynamic,
                color: input.dynamic_color,
                alpha: dynamic_fog_alpha(input.dynamic_alpha),
            });
            if input.static_fog {
                commands.push(MinimapOverlayCommand::FogTexture {
                    layer: MinimapFogLayer::Static,
                    color: 0x000000ff,
                    alpha: 1.0,
                });
            }
        }

        if input.full_view && input.show_spawns && input.has_spawns && input.waves {
            let curve = curve(input.time % 240.0, 120.0, 240.0);
            let pulse = if curve > 0.0 { pow3_out(curve) } else { 0.0 };
            for spawn in input.spawns {
                commands.push(MinimapOverlayCommand::Spawn {
                    x: spawn.x,
                    y: spawn.y,
                    radius: input.drop_zone_radius,
                    pulse,
                    team_color: input.wave_team_color,
                });
            }
        }

        if input.full_view && !input.mobile {
            commands.push(MinimapOverlayCommand::CameraBounds(MinimapRect::new(
                camera.x - camera.width / 2.0,
                camera.y - camera.height / 2.0,
                camera.width,
                camera.height,
            )));
        }

        let fin = (input.global_time / 30.0) % 1.0;
        let rad = fin * 5.0 + self.tile_size - 2.0;
        for indicator in input.indicators {
            commands.push(MinimapOverlayCommand::Indicator {
                tile: indicator.tile,
                radius: rad * inverse_scale.max(1.0),
                alpha: (indicator.time / 70.0).clamp(0.0, 1.0),
                color_from: 0xffa500ff,
                color_to: 0xff2400ff,
            });
        }

        for marker in input.markers.into_iter().filter(|marker| marker.minimap) {
            commands.push(MinimapOverlayCommand::Marker {
                id: marker.id,
                x: marker.x,
                y: marker.y,
                label: marker.label,
            });
        }

        MinimapOverlayPlan {
            world_rect,
            scale_factor: inverse_scale,
            commands,
        }
    }
}

pub fn color_for(tile: MinimapTileSnapshot) -> u32 {
    let mut color = if tile.real_block_color == 0 && tile.block_is_air && tile.overlay_is_air {
        if tile.floor_color == 0 {
            tile.fallback_map_color
        } else {
            tile.floor_color
        }
    } else if tile.real_block_color == 0 {
        tile.fallback_map_color
    } else {
        tile.real_block_color
    };

    let darkness = 1.0 - (tile.darkness / 4.0).clamp(0.0, 1.0);
    color = mul_rgb(color, darkness, darkness, darkness);

    if tile.block_is_air && tile.above_real_block_solid {
        color = mul_rgb(color, 0.7, 0.7, 0.7);
    } else if tile.floor_is_liquid && !tile.above_floor_is_liquid {
        color = mul_rgb(color, 0.84, 0.84, 0.9);
    }

    color
}

pub fn dynamic_fog_alpha(alpha: f32) -> f32 {
    if alpha.is_nan() {
        0.5
    } else {
        alpha.max(0.5)
    }
}

fn pixmap_y(height: i32, tile_y: i32) -> i32 {
    height - 1 - tile_y
}

fn arc_clamp(value: f32, min: f32, max: f32) -> f32 {
    min.max(value.min(max))
}

impl MinimapResetPlan {
    pub fn texture_frame_plan(self) -> MinimapTextureFramePlan {
        let texture_size = MinimapTextureSize::new(self.width, self.height);
        MinimapTextureFramePlan {
            texture_size,
            recreate_texture: self.recreate_texture,
            full_upload: Some(MinimapFullUpdatePlan {
                width: self.width,
                height: self.height,
            }),
            dirty_pixels: Vec::new(),
        }
    }
}

impl MinimapFullUpdatePlan {
    pub const fn texture_size(self) -> MinimapTextureSize {
        MinimapTextureSize::new(self.width, self.height)
    }

    pub fn texture_frame_plan(self) -> MinimapTextureFramePlan {
        MinimapTextureFramePlan {
            texture_size: self.texture_size(),
            recreate_texture: false,
            full_upload: Some(self),
            dirty_pixels: Vec::new(),
        }
    }
}

impl MinimapPixelUpdate {
    pub const fn texture_update(
        self,
        texture_size: MinimapTextureSize,
    ) -> MinimapTexturePixelUpdate {
        MinimapTexturePixelUpdate {
            pos: self.pos,
            texture_x: self.pixmap_x,
            texture_y: texture_size.height - 1 - self.pos.y,
            rgba: self.rgba,
        }
    }
}

impl MinimapTileUpdatePlan {
    pub fn texture_frame_plan(&self, texture_size: MinimapTextureSize) -> MinimapTextureFramePlan {
        MinimapTextureFramePlan {
            texture_size,
            recreate_texture: false,
            full_upload: None,
            dirty_pixels: self
                .updates
                .iter()
                .copied()
                .map(|update| update.texture_update(texture_size))
                .collect(),
        }
    }
}

fn mul_rgb(color: u32, r_mul: f32, g_mul: f32, b_mul: f32) -> u32 {
    let r = (((color >> 24) & 0xff) as f32 * r_mul).clamp(0.0, 255.0) as u32;
    let g = (((color >> 16) & 0xff) as f32 * g_mul).clamp(0.0, 255.0) as u32;
    let b = (((color >> 8) & 0xff) as f32 * b_mul).clamp(0.0, 255.0) as u32;
    let a = color & 0xff;
    (r << 24) | (g << 16) | (b << 8) | a
}

fn curve(value: f32, start: f32, end: f32) -> f32 {
    if value <= start {
        0.0
    } else if value >= end {
        1.0
    } else {
        (value - start) / (end - start)
    }
}

fn pow3_out(value: f32) -> f32 {
    1.0 - (1.0 - value).powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimap_zoom_clamps_like_upstream() {
        let mut state = MinimapRendererState::new(MinimapWorldSize::new(200, 100));
        assert_eq!(state.get_zoom(), 3.125);

        state.set_zoom(0.25);
        assert_eq!(state.get_zoom(), 1.0);

        state.zoom_by(999.0);
        assert_eq!(state.get_zoom(), 3.125);
    }

    #[test]
    fn update_tick_batches_every_two_frames_and_flips_pixmap_y() {
        let mut state = MinimapRendererState::new(MinimapWorldSize::new(20, 10));
        let pos = MinimapTilePos::new(2, 3);
        state.update_pixel(pos);
        let snapshot = MinimapTileSnapshot {
            real_block_color: 0x204060ff,
            ..MinimapTileSnapshot::new(pos, 0)
        };

        assert!(state.update_tick(1.0, &[snapshot]).is_none());
        let plan = state.update_tick(1.0, &[snapshot]).unwrap();

        assert_eq!(
            plan.updates,
            vec![MinimapPixelUpdate {
                pos,
                pixmap_x: 2,
                pixmap_y: 6,
                rgba: 0x204060ff,
            }]
        );
        assert!(state.updates.is_empty());
    }

    #[test]
    fn update_all_returns_world_size_when_buffers_exist() {
        let state = MinimapRendererState::new(MinimapWorldSize::new(32, 18));

        assert_eq!(
            state.update_all(),
            Some(MinimapFullUpdatePlan {
                width: 32,
                height: 18,
            })
        );
    }

    #[test]
    fn texture_frame_plan_from_full_update_carries_texture_size_and_upload() {
        let state = MinimapRendererState::new(MinimapWorldSize::new(32, 18));
        assert_eq!(state.texture_size(), MinimapTextureSize::new(32, 18));

        assert_eq!(
            state.texture_frame_plan(),
            Some(MinimapTextureFramePlan {
                texture_size: MinimapTextureSize::new(32, 18),
                recreate_texture: false,
                full_upload: Some(MinimapFullUpdatePlan {
                    width: 32,
                    height: 18,
                }),
                dirty_pixels: Vec::new(),
            })
        );
    }

    #[test]
    fn reset_plan_texture_frame_plan_requests_recreate_and_full_upload() {
        let mut state = MinimapRendererState::new(MinimapWorldSize::new(8, 8));
        let plan = state.reset(MinimapWorldSize::new(64, 48));

        assert_eq!(state.texture_size(), MinimapTextureSize::new(64, 48));
        assert_eq!(
            plan.texture_frame_plan(),
            MinimapTextureFramePlan {
                texture_size: MinimapTextureSize::new(64, 48),
                recreate_texture: true,
                full_upload: Some(MinimapFullUpdatePlan {
                    width: 64,
                    height: 48,
                }),
                dirty_pixels: Vec::new(),
            }
        );
    }

    #[test]
    fn tile_update_plan_texture_frame_plan_keeps_y_flipped_texture_coords() {
        let pos = MinimapTilePos::new(4, 3);
        let update_plan = MinimapTileUpdatePlan {
            updates: vec![MinimapPixelUpdate {
                pos,
                pixmap_x: 4,
                pixmap_y: 6,
                rgba: 0x123456ff,
            }],
        };

        assert_eq!(
            update_plan.texture_frame_plan(MinimapTextureSize::new(20, 10)),
            MinimapTextureFramePlan {
                texture_size: MinimapTextureSize::new(20, 10),
                recreate_texture: false,
                full_upload: None,
                dirty_pixels: vec![MinimapTexturePixelUpdate {
                    pos,
                    texture_x: 4,
                    texture_y: 6,
                    rgba: 0x123456ff,
                }],
            }
        );
    }

    #[test]
    fn update_tile_batches_center_linked_and_below_tiles_like_java() {
        let mut state = MinimapRendererState::new(MinimapWorldSize::new(12, 8));
        let center = MinimapTilePos::new(4, 3);
        let linked = [
            MinimapTilePos::new(4, 3),
            MinimapTilePos::new(5, 3),
            MinimapTilePos::new(5, 0),
        ];

        state.update_tile(center, &linked, true);

        let packed: Vec<_> = state
            .updates
            .iter()
            .copied()
            .map(MinimapTilePos::unpack)
            .collect();
        assert_eq!(
            packed,
            vec![
                MinimapTilePos::new(4, 2),
                MinimapTilePos::new(4, 3),
                MinimapTilePos::new(5, 0),
                MinimapTilePos::new(5, 2),
                MinimapTilePos::new(5, 3),
            ]
        );
    }

    #[test]
    fn update_tick_honors_two_frame_batching_and_clears_after_flush() {
        let mut state = MinimapRendererState::new(MinimapWorldSize::new(10, 10));
        let pos = MinimapTilePos::new(1, 1);
        let snapshot = MinimapTileSnapshot {
            real_block_color: 0xabcdefff,
            ..MinimapTileSnapshot::new(pos, 0)
        };

        state.update_pixel(pos);
        assert!(state.update_tick(0.75, &[snapshot]).is_none());
        assert!(state.update_tick(0.75, &[snapshot]).is_none());
        let plan = state.update_tick(0.5, &[snapshot]).unwrap();

        assert_eq!(plan.updates.len(), 1);
        assert_eq!(plan.updates[0].pixmap_y, 8);
        assert!(state.updates.is_empty());
    }

    #[test]
    fn region_matches_java_uv_crop_math() {
        let mut state = MinimapRendererState::new(MinimapWorldSize::new(200, 100));
        state.set_zoom(2.0);
        let region = state
            .region(MinimapCamera::new(800.0, 400.0, 320.0, 200.0))
            .unwrap();

        assert_eq!(
            region.source_rect_tiles,
            MinimapRect::new(68.0, 18.0, 64.0, 64.0)
        );
        assert!((region.u - 0.34).abs() < 0.0001);
        assert!((region.v - 0.18).abs() < 0.0001);
        assert!((region.u2 - 0.66).abs() < 0.0001);
        assert!((region.v2 - 0.82).abs() < 0.0001);
    }

    #[test]
    fn region_clamps_like_java_even_when_zoom_is_overridden() {
        let mut state = MinimapRendererState::new(MinimapWorldSize::new(192, 192));
        state.zoom = 999.0;

        let region = state
            .region(MinimapCamera::new(768.0, 768.0, 64.0, 64.0))
            .unwrap();

        assert_eq!(
            region.source_rect_tiles,
            MinimapRect::new(0.0, -192.0, 384.0, 384.0)
        );
        assert_eq!(region.u, 0.0);
        assert_eq!(region.v, -1.0);
        assert_eq!(region.u2, 2.0);
        assert_eq!(region.v2, 1.0);
    }

    #[test]
    fn color_for_applies_darkness_wall_shadow_and_liquid_edge() {
        let dark = MinimapTileSnapshot {
            darkness: 2.0,
            real_block_color: 0x804020ff,
            ..MinimapTileSnapshot::new(MinimapTilePos::new(0, 0), 0)
        };
        assert_eq!(color_for(dark), 0x402010ff);

        let wall_shadow = MinimapTileSnapshot {
            block_is_air: true,
            overlay_is_air: true,
            floor_color: 0x808080ff,
            above_real_block_solid: true,
            ..MinimapTileSnapshot::new(MinimapTilePos::new(0, 0), 0)
        };
        assert_eq!(color_for(wall_shadow), 0x595959ff);

        let liquid_edge = MinimapTileSnapshot {
            block_is_air: true,
            overlay_is_air: true,
            floor_color: 0x6496c8ff,
            floor_is_liquid: true,
            above_floor_is_liquid: false,
            ..MinimapTileSnapshot::new(MinimapTilePos::new(0, 0), 0)
        };
        assert_eq!(color_for(liquid_edge), 0x547db4ff);
    }

    #[test]
    fn overlay_plan_emits_full_view_entities_fog_spawns_camera_and_markers() {
        let state = MinimapRendererState::new(MinimapWorldSize::new(100, 50));
        let plan = state.overlay_plan(
            MinimapCamera::new(400.0, 200.0, 160.0, 120.0),
            MinimapOverlayInput {
                screen_x: 0.0,
                screen_y: 0.0,
                screen_width: 400.0,
                screen_height: 200.0,
                full_view: true,
                mobile: false,
                net_active: true,
                show_pings: true,
                fog: true,
                static_fog: true,
                dynamic_color: 0x112233ff,
                dynamic_alpha: f32::NAN,
                show_spawns: true,
                has_spawns: true,
                waves: true,
                wave_team_color: 0xff00ffff,
                drop_zone_radius: 30.0,
                time: 180.0,
                global_time: 15.0,
                units: vec![MinimapEntitySnapshot {
                    entity_id: 7,
                    x: 10.0,
                    y: 20.0,
                    rotation: 180.0,
                    team_color: 0xff0000ff,
                    region: "flare".into(),
                    draw_minimap: true,
                    hidden_by_fog: false,
                }],
                players: vec![MinimapPlayerSnapshot {
                    player_id: 1,
                    x: 15.0,
                    y: 25.0,
                    name: "player".into(),
                    color: 0xffffffff,
                    dead: false,
                    ping_time: 1.0,
                    ping_x: 16.0,
                    ping_y: 26.0,
                    ping_text: Some("go".into()),
                }],
                spawns: vec![MinimapSpawnSnapshot { x: 2.0, y: 3.0 }],
                indicators: vec![MinimapIndicatorSnapshot {
                    tile: MinimapTilePos::new(4, 5),
                    time: 35.0,
                    block_offset_tiles: 0.0,
                }],
                markers: vec![MinimapMarkerSnapshot {
                    id: 9,
                    x: 30.0,
                    y: 40.0,
                    label: "target".into(),
                    minimap: true,
                }],
            },
        );

        assert_eq!(plan.world_rect, MinimapRect::new(0.0, 0.0, 800.0, 400.0));
        assert!(plan
            .commands
            .iter()
            .any(|cmd| matches!(cmd, MinimapOverlayCommand::UnitIcon { entity_id: 7, .. })));
        assert!(plan
            .commands
            .iter()
            .any(|cmd| matches!(cmd, MinimapOverlayCommand::PlayerLabel { player_id: 1, .. })));
        assert!(plan
            .commands
            .iter()
            .any(|cmd| matches!(cmd, MinimapOverlayCommand::Ping { player_id: 1, .. })));
        assert!(plan.commands.iter().any(|cmd| matches!(
            cmd,
            MinimapOverlayCommand::FogTexture {
                layer: MinimapFogLayer::Dynamic,
                alpha: 0.5,
                ..
            }
        )));
        assert!(plan
            .commands
            .iter()
            .any(|cmd| matches!(cmd, MinimapOverlayCommand::Spawn { .. })));
        assert!(plan
            .commands
            .iter()
            .any(|cmd| matches!(cmd, MinimapOverlayCommand::CameraBounds(_))));
        assert!(plan
            .commands
            .iter()
            .any(|cmd| matches!(cmd, MinimapOverlayCommand::Indicator { .. })));
        assert!(plan
            .commands
            .iter()
            .any(|cmd| matches!(cmd, MinimapOverlayCommand::Marker { id: 9, .. })));
    }
}
