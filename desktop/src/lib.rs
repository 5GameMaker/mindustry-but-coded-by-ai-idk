use mindustry_core::mindustry::client_launcher::ClientLauncher;
use mindustry_core::mindustry::content::blocks::{BlockDef, DistributionBlockKind};
use mindustry_core::mindustry::core::game_runtime::{
    GameRuntimeBlockVisualRuntimeSnapshot, GameRuntimeClientCameraShakeEvent,
    GameRuntimeClientSnapshotApplyReport, GameRuntimeClientUnitEnteredPayloadApplyReport,
    GameRuntimeCommandBuildingReport, GameRuntimeReconstructorConfigureResult,
    GameRuntimeUnitCargoUnloadConfigureResult, GameRuntimeUnitFactoryConfigureResult,
};
use mindustry_core::mindustry::core::net_client::{
    ClientBlockSnapshotMirror, ClientHiddenSnapshotMirror, ClientTileStorageMirror,
    ClientUnitItemMirror, ClientUnitPayloadMirror,
};
use mindustry_core::mindustry::core::{
    content_loader::ContentLoader, ClientConnectConfig, GameRuntime, GameRuntimeMapLoadReport,
    GameRuntimeNetworkContext, GameState, GameStateState, NetClient,
};
use mindustry_core::mindustry::ctype::{ContentId, ContentType, UnlockableContentBase};
use mindustry_core::mindustry::entities::comp::BuildingComp;
use mindustry_core::mindustry::entities::{
    entity_class_kind, shake_intensity, standard_effect,
    standard_effect_draw_plans_with_data_value_and_resolved_context,
    standard_effect_render_lifetime, EffectRenderInput, EntityClassKind, PlayerComp,
    PlayerUnitSwitchContext, ShieldArcAbility, StandardEffectCircleRenderPrimitive,
    StandardEffectDrawPlan, StandardEffectLightRenderPrimitive, StandardEffectLineRenderPrimitive,
    StandardEffectRectRenderPrimitive, StandardEffectShieldArcBreak,
    StandardEffectSquareRenderPrimitive, StandardEffectTriangleRenderPrimitive, PLAYER_CLASS_ID,
};
use mindustry_core::mindustry::graphics::floor_renderer::FloorChunkDrawBatch;
use mindustry_core::mindustry::graphics::{
    BlockRendererBlockParticleWorldSample, BlockRendererBuildingSnapshot,
    BlockRendererBuildingVisualRuntimeLiquidSnapshot,
    BlockRendererBuildingVisualRuntimePowerSnapshot, BlockRendererBuildingVisualRuntimeSnapshot,
    BlockRendererBuildingVisualRuntimeTurretSnapshot, BlockRendererPlan, BlockRendererState,
    BlockRendererTileSnapshot, BlockRendererWorldSnapshot, CacheLayer as GraphicsCacheLayer,
    FloorRenderPlan, FloorRendererState, FogColor, FogFrameInput, FogFramePlan, FogRendererState,
    FogViewport, GraphicsFrameBundle, GraphicsFrameStats, LightRendererPlan, LightRendererState,
    LoadFrameInput, LoadFramePlan, LoadRendererState, MenuFrameInput, MenuFramePlan,
    MenuRendererConfig, MenuRendererState, MinimapCamera, MinimapOverlayInput, MinimapOverlayPlan,
    MinimapRect, MinimapRendererState, MinimapTextureFramePlan, MinimapWorldSize,
    OverlayRendererPlan, OverlayRendererState, PageType, PixelatorCamera, PixelatorFramePlan,
    PixelatorInput, PixelatorState, RenderBlendMode, RenderBridge, RenderCamera, RenderCommand,
    RenderEngineState, RenderFramePlan, RenderPassKind, RenderPoint, RenderRect, RenderResolveKind,
    RenderSize, RenderTarget, RenderViewport, ShaderApplyContext, ShaderCamera, ShaderCatalog,
    ShaderDispatchFrame, ShaderId, ShaderViewport, TextureAtlasPlan,
    TextureAtlasSpriteSourceDescriptor, TileBounds, TileCoord, Viewport as FloorViewport,
};
use mindustry_core::mindustry::input::input_handler::{
    other_player_preview_overlay_plan, OtherPlayerPreviewBlock, OtherPlayerPreviewOverlayFrame,
    OtherPlayerPreviewOverlayPlan,
};
use mindustry_core::mindustry::io::{
    read_bullet_sync, read_decal_sync, read_effect_state_sync, read_fire_sync, read_puddle_sync,
    read_unit_sync, read_weather_state_sync, read_world_label_sync, ContentHeaderSnapshot,
    LegacyTeamBlocks, TeamId, TypeValue, Vec2,
};
use mindustry_core::mindustry::modsys::{ModResourceContainerPlan, ModResourcePlan};
use mindustry_core::mindustry::net::{
    ArcNetProvider, EffectCallPacket2, Net, NetworkPlayerData, NetworkPlayerSyncData,
    NetworkWorldData, PacketKind, SoundAtCallPacket, StateSnapshotCallPacket,
};
use mindustry_core::mindustry::service::{
    AchievementContext, GameServiceApplySummary, GameServiceTriggerSnapshot,
};
use mindustry_core::mindustry::vars::{AppContext, MAX_PLAYER_PREVIEW_PLANS};
use mindustry_core::mindustry::world::draw::{
    DrawBlockParticleBlendMode, DrawBlockParticleRenderKind,
};
use mindustry_core::mindustry::world::{BuildingRef, CacheLayer as WorldCacheLayer, Tile};
use mindustry_core::mindustry::UPSTREAM_BASELINE;
use std::collections::BTreeMap;
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopConnectTarget {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopStandardEffectRenderFrame {
    pub draw_plans: Vec<StandardEffectDrawPlan>,
    pub circle_primitives: Vec<StandardEffectCircleRenderPrimitive>,
    pub square_primitives: Vec<StandardEffectSquareRenderPrimitive>,
    pub rect_primitives: Vec<StandardEffectRectRenderPrimitive>,
    pub line_primitives: Vec<StandardEffectLineRenderPrimitive>,
    pub triangle_primitives: Vec<StandardEffectTriangleRenderPrimitive>,
    pub light_primitives: Vec<StandardEffectLightRenderPrimitive>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopSoundAtAudioFrame {
    pub sound_at_events: Vec<SoundAtCallPacket>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DesktopCameraShakeFrame {
    pub max_offset: f32,
    pub remaining_intensity: f32,
    pub remaining_time: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DesktopCameraShakeRenderStats {
    pub max_offset: f32,
    pub remaining_intensity: f32,
    pub remaining_time: f32,
}

impl DesktopCameraShakeRenderStats {
    pub fn from_camera_shake_frame(frame: &DesktopCameraShakeFrame) -> Self {
        Self {
            max_offset: frame.max_offset,
            remaining_intensity: frame.remaining_intensity,
            remaining_time: frame.remaining_time,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DesktopCameraShakeState {
    pub intensity: f32,
    pub time: f32,
    pub reduction: f32,
}

impl DesktopCameraShakeState {
    pub fn apply(&mut self, intensity: f32, duration: f32) {
        if !intensity.is_finite() || !duration.is_finite() {
            return;
        }
        let intensity = intensity.clamp(0.0, 100.0);
        let duration = duration.max(0.0);
        if intensity <= 0.0 || duration <= 0.0 {
            return;
        }

        self.intensity = self.intensity.max(intensity);
        self.time = self.time.max(duration);
        self.reduction = if self.time > 0.0 {
            self.intensity / self.time
        } else {
            0.0
        };
    }

    pub fn tick(&mut self, delta: f32, screen_shake_setting: i32) -> DesktopCameraShakeFrame {
        let max_offset = if self.time > 0.0 {
            let setting = screen_shake_setting.clamp(0, 4) as f32 / 4.0;
            self.intensity * setting * 0.75
        } else {
            0.0
        };

        if self.time > 0.0 {
            let delta = if delta.is_finite() {
                delta.max(0.0)
            } else {
                0.0
            };
            self.intensity = (self.intensity - self.reduction * delta).clamp(0.0, 100.0);
            self.time = (self.time - delta).max(0.0);
            if self.time <= 0.0 {
                self.intensity = 0.0;
                self.reduction = 0.0;
            }
        } else {
            self.intensity = 0.0;
            self.reduction = 0.0;
        }

        DesktopCameraShakeFrame {
            max_offset,
            remaining_intensity: self.intensity,
            remaining_time: self.time,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DesktopEffectRenderStats {
    pub draw_plans: usize,
    pub circle_primitives: usize,
    pub square_primitives: usize,
    pub rect_primitives: usize,
    pub line_primitives: usize,
    pub triangle_primitives: usize,
    pub light_primitives: usize,
}

impl DesktopEffectRenderStats {
    pub fn from_standard_effect_frame(frame: &DesktopStandardEffectRenderFrame) -> Self {
        Self {
            draw_plans: frame.draw_plans.len(),
            circle_primitives: frame.circle_primitives.len(),
            square_primitives: frame.square_primitives.len(),
            rect_primitives: frame.rect_primitives.len(),
            line_primitives: frame.line_primitives.len(),
            triangle_primitives: frame.triangle_primitives.len(),
            light_primitives: frame.light_primitives.len(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DesktopSoundAudioStats {
    pub sound_at_events: usize,
}

impl DesktopSoundAudioStats {
    pub fn from_sound_at_audio_frame(frame: &DesktopSoundAtAudioFrame) -> Self {
        Self {
            sound_at_events: frame.sound_at_events.len(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DesktopPlayableSmokeStatus {
    pub client_ready: bool,
    pub connected: bool,
    pub world_loaded: bool,
    pub confirmed: bool,
    pub game_playing: bool,
    pub runtime_client: bool,
    pub world_width: usize,
    pub world_height: usize,
    pub buildings: usize,
    pub player_on_default_team: bool,
}

impl DesktopPlayableSmokeStatus {
    pub fn ready(self) -> bool {
        self.client_ready
            && self.connected
            && self.world_loaded
            && self.confirmed
            && self.game_playing
            && self.runtime_client
            && self.world_width > 0
            && self.world_height > 0
            && self.buildings > 0
            && self.player_on_default_team
    }
}

pub trait DesktopEffectRenderer {
    fn render_standard_effect_frame(
        &mut self,
        frame: &DesktopStandardEffectRenderFrame,
    ) -> DesktopEffectRenderStats;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeadlessDesktopEffectRenderer {
    pub frames_rendered: usize,
    pub last_stats: DesktopEffectRenderStats,
}

impl DesktopEffectRenderer for HeadlessDesktopEffectRenderer {
    fn render_standard_effect_frame(
        &mut self,
        frame: &DesktopStandardEffectRenderFrame,
    ) -> DesktopEffectRenderStats {
        let stats = DesktopEffectRenderStats::from_standard_effect_frame(frame);
        self.frames_rendered += 1;
        self.last_stats = stats;
        stats
    }
}

pub trait DesktopAudioRenderer {
    fn play_sound_at_audio_frame(
        &mut self,
        frame: &DesktopSoundAtAudioFrame,
    ) -> DesktopSoundAudioStats;
}

pub trait DesktopCameraShakeRenderer {
    fn apply_camera_shake_frame(
        &mut self,
        frame: &DesktopCameraShakeFrame,
    ) -> DesktopCameraShakeRenderStats;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeadlessDesktopAudioRenderer {
    pub frames_played: usize,
    pub last_stats: DesktopSoundAudioStats,
}

impl DesktopAudioRenderer for HeadlessDesktopAudioRenderer {
    fn play_sound_at_audio_frame(
        &mut self,
        frame: &DesktopSoundAtAudioFrame,
    ) -> DesktopSoundAudioStats {
        let stats = DesktopSoundAudioStats::from_sound_at_audio_frame(frame);
        self.frames_played += 1;
        self.last_stats = stats;
        stats
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HeadlessDesktopCameraShakeRenderer {
    pub frames_applied: usize,
    pub last_stats: DesktopCameraShakeRenderStats,
}

impl DesktopCameraShakeRenderer for HeadlessDesktopCameraShakeRenderer {
    fn apply_camera_shake_frame(
        &mut self,
        frame: &DesktopCameraShakeFrame,
    ) -> DesktopCameraShakeRenderStats {
        let stats = DesktopCameraShakeRenderStats::from_camera_shake_frame(frame);
        self.frames_applied += 1;
        self.last_stats = stats;
        stats
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopGraphicsFrame {
    pub bundle: GraphicsFrameBundle,
    pub floor_chunk_batches: Vec<FloorChunkDrawBatch>,
    pub minimap_texture_frame: Option<MinimapTextureFramePlan>,
    pub texture_atlas: TextureAtlasPlan<bool>,
}

impl DesktopGraphicsFrame {
    pub fn stats(&self) -> &GraphicsFrameStats {
        &self.bundle.stats
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopGraphicsShaderApplyExecutionTrace {
    pub shader: ShaderId,
    pub operation_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DesktopGraphicsShaderDispatchExecutionTrace {
    pub applies: Vec<DesktopGraphicsShaderApplyExecutionTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopGraphicsExecutionStepTrace {
    ShaderDispatch {
        apply_count: usize,
    },
    BlockParticles {
        plan_count: usize,
    },
    RenderPass {
        kind: RenderPassKind,
        order: i32,
        target: RenderTarget,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopGraphicsCommandExecutionTrace {
    DrawSprite { symbol: String },
    DrawText { text: String },
    DrawPolygon { sides: usize },
    NoOp { kind: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopGraphicsPassExecutionTrace {
    pub kind: RenderPassKind,
    pub order: i32,
    pub target: RenderTarget,
    pub resolve_target: Option<RenderTarget>,
    pub resolve_kind: Option<RenderResolveKind>,
    pub command_count: usize,
    pub commands: Vec<RenderCommand>,
    pub command_trace: Vec<DesktopGraphicsCommandExecutionTrace>,
    pub draw_sprite_symbols: Vec<String>,
    pub resolved_sprites: Vec<DesktopGraphicsResolvedSpriteTrace>,
    pub draw_texts: Vec<String>,
    pub draw_polygon_sides: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopGraphicsTextureSamplerTrace {
    Nearest,
    Linear,
}

impl DesktopGraphicsTextureSamplerTrace {
    pub const fn from_linear_filter(linear_filter: bool) -> Self {
        if linear_filter {
            Self::Linear
        } else {
            Self::Nearest
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopGraphicsResolvedSpriteTrace {
    pub symbol: String,
    pub page_type: Option<PageType>,
    pub page_source_path: Option<String>,
    pub page_width: Option<u32>,
    pub page_height: Option<u32>,
    pub linear_filter: bool,
    pub sampler: DesktopGraphicsTextureSamplerTrace,
    pub region_source_path: Option<String>,
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub u: Option<f32>,
    pub v: Option<f32>,
    pub u2: Option<f32>,
    pub v2: Option<f32>,
    pub region_width: Option<u32>,
    pub region_height: Option<u32>,
    pub missing: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopGraphicsLiveBackendDrawSpriteTrace {
    pub pass_index: usize,
    pub command_index: usize,
    pub pass_kind: RenderPassKind,
    pub pass_order: i32,
    pub target: RenderTarget,
    pub symbol: String,
    pub resolved_sprite: Option<DesktopGraphicsResolvedSpriteTrace>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DesktopGraphicsLiveBackendRenderCommandSource {
    RenderPass {
        pass_index: usize,
        command_index: usize,
        pass_kind: RenderPassKind,
        pass_order: i32,
        target: RenderTarget,
    },
    BlockParticles {
        command_index: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopGraphicsLiveBackendRenderCommandTrace {
    pub source: DesktopGraphicsLiveBackendRenderCommandSource,
    pub kind: &'static str,
    pub command: RenderCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopGraphicsLiveBackendRenderTargetEventKind {
    Begin,
    End,
    Resolve,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopGraphicsLiveBackendRenderTargetTrace {
    pub pass_index: usize,
    pub pass_kind: RenderPassKind,
    pub pass_order: i32,
    pub target: RenderTarget,
    pub resolve_target: Option<RenderTarget>,
    pub resolve_kind: Option<RenderResolveKind>,
    pub event: DesktopGraphicsLiveBackendRenderTargetEventKind,
    pub command_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopGraphicsBlockParticleTrace {
    pub plan_index: usize,
    pub sample_index: usize,
    pub coord: TileCoord,
    pub block: String,
    pub sample: BlockRendererBlockParticleWorldSample,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DesktopGraphicsBlockParticleDrawCallKind {
    Circle,
    Polygon { sides: usize, rotation: f32 },
    SoftSprite { region: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopGraphicsBlockParticleDrawCall {
    pub plan_index: usize,
    pub sample_index: usize,
    pub coord: TileCoord,
    pub block: String,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub alpha: f32,
    pub layer: f32,
    pub color: [f32; 4],
    pub secondary_color: Option<[f32; 4]>,
    pub color_t: Option<f32>,
    pub blend_mode: DrawBlockParticleBlendMode,
    pub kind: DesktopGraphicsBlockParticleDrawCallKind,
}

impl DesktopGraphicsBlockParticleDrawCall {
    pub fn from_trace(trace: &DesktopGraphicsBlockParticleTrace) -> Self {
        let sample = &trace.sample;
        let kind = match sample.render_kind {
            DrawBlockParticleRenderKind::Circle => DesktopGraphicsBlockParticleDrawCallKind::Circle,
            DrawBlockParticleRenderKind::Polygon => {
                DesktopGraphicsBlockParticleDrawCallKind::Polygon {
                    sides: sample.sides,
                    rotation: sample.rotation,
                }
            }
            DrawBlockParticleRenderKind::SoftSprite => {
                DesktopGraphicsBlockParticleDrawCallKind::SoftSprite {
                    region: sample.region.map(str::to_string),
                }
            }
        };

        Self {
            plan_index: trace.plan_index,
            sample_index: trace.sample_index,
            coord: trace.coord,
            block: trace.block.clone(),
            x: sample.x,
            y: sample.y,
            size: sample.size,
            alpha: sample.alpha,
            layer: sample.layer,
            color: [
                sample.color.r,
                sample.color.g,
                sample.color.b,
                sample.color.a,
            ],
            secondary_color: sample
                .secondary_color
                .map(|color| [color.r, color.g, color.b, color.a]),
            color_t: sample.color_t,
            blend_mode: sample.blend_mode,
            kind,
        }
    }

    pub fn render_blend_mode(&self) -> RenderBlendMode {
        match self.blend_mode {
            DrawBlockParticleBlendMode::Normal => RenderBlendMode::Normal,
            DrawBlockParticleBlendMode::Additive => RenderBlendMode::Additive,
        }
    }

    pub fn tint(&self) -> [f32; 4] {
        let color = if let (Some(secondary), Some(t)) = (self.secondary_color, self.color_t) {
            let t = t.clamp(0.0, 1.0);
            [
                self.color[0] + (secondary[0] - self.color[0]) * t,
                self.color[1] + (secondary[1] - self.color[1]) * t,
                self.color[2] + (secondary[2] - self.color[2]) * t,
                self.color[3] + (secondary[3] - self.color[3]) * t,
            ]
        } else {
            self.color
        };

        [color[0], color[1], color[2], color[3] * self.alpha]
    }

    pub fn render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = vec![RenderCommand::set_blend(self.render_blend_mode())];
        let center = RenderPoint::new(self.x, self.y);

        match &self.kind {
            DesktopGraphicsBlockParticleDrawCallKind::Circle => {
                commands.push(RenderCommand::draw_circle(
                    center,
                    self.size.max(0.0),
                    self.tint(),
                    true,
                    self.layer,
                ));
            }
            DesktopGraphicsBlockParticleDrawCallKind::SoftSprite { region } => {
                let symbol = region
                    .clone()
                    .unwrap_or_else(|| String::from("circle-shadow"));
                commands.push(RenderCommand::draw_sprite(
                    symbol,
                    RenderRect::from_center(center, self.size, self.size),
                    self.tint(),
                    0.0,
                    self.layer,
                ));
            }
            DesktopGraphicsBlockParticleDrawCallKind::Polygon { sides, rotation } => {
                commands.push(RenderCommand::draw_polygon(
                    center,
                    self.size.max(0.0),
                    *sides,
                    *rotation,
                    self.tint(),
                    true,
                    self.layer,
                ));
            }
        }

        commands
    }
}

pub trait DesktopGraphicsLiveBackendBlockParticleSink {
    fn consume_block_particle_trace(&mut self, trace: DesktopGraphicsBlockParticleTrace);
}

pub trait DesktopGraphicsLiveBackendBlockParticleDrawCallSink {
    fn consume_block_particle_draw_call(&mut self, draw_call: DesktopGraphicsBlockParticleDrawCall);
}

pub trait DesktopGraphicsLiveBackendRenderCommandSink {
    fn consume_render_command_trace(&mut self, trace: DesktopGraphicsLiveBackendRenderCommandTrace);
}

pub trait DesktopGraphicsLiveBackendRenderTargetSink {
    fn consume_render_target_trace(&mut self, trace: DesktopGraphicsLiveBackendRenderTargetTrace);
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct DesktopGraphicsNullLiveBackendBlockParticleSink;

impl DesktopGraphicsLiveBackendBlockParticleSink
    for DesktopGraphicsNullLiveBackendBlockParticleSink
{
    fn consume_block_particle_trace(&mut self, _trace: DesktopGraphicsBlockParticleTrace) {}
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct DesktopGraphicsNullLiveBackendBlockParticleDrawCallSink;

impl DesktopGraphicsLiveBackendBlockParticleDrawCallSink
    for DesktopGraphicsNullLiveBackendBlockParticleDrawCallSink
{
    fn consume_block_particle_draw_call(
        &mut self,
        _draw_call: DesktopGraphicsBlockParticleDrawCall,
    ) {
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct DesktopGraphicsNullLiveBackendRenderCommandSink;

impl DesktopGraphicsLiveBackendRenderCommandSink
    for DesktopGraphicsNullLiveBackendRenderCommandSink
{
    fn consume_render_command_trace(
        &mut self,
        _trace: DesktopGraphicsLiveBackendRenderCommandTrace,
    ) {
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct DesktopGraphicsNullLiveBackendRenderTargetSink;

impl DesktopGraphicsLiveBackendRenderTargetSink for DesktopGraphicsNullLiveBackendRenderTargetSink {
    fn consume_render_target_trace(&mut self, _trace: DesktopGraphicsLiveBackendRenderTargetTrace) {
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DesktopGraphicsLiveBackendExecutionState {
    pub render_passes_visited: usize,
    pub render_commands_visited: usize,
    pub backend_render_commands_emitted: usize,
    pub last_backend_render_command: Option<DesktopGraphicsLiveBackendRenderCommandTrace>,
    pub backend_target_events_emitted: usize,
    pub resolve_target_events_emitted: usize,
    pub screen_target_events_emitted: usize,
    pub texture_target_events_emitted: usize,
    pub buffer_target_events_emitted: usize,
    pub last_backend_target_event: Option<DesktopGraphicsLiveBackendRenderTargetTrace>,
    pub draw_sprite_traces_emitted: usize,
    pub last_draw_sprite_trace: Option<DesktopGraphicsLiveBackendDrawSpriteTrace>,
    pub block_particle_traces_emitted: usize,
    pub last_block_particle_trace: Option<DesktopGraphicsBlockParticleTrace>,
    pub block_particle_draw_calls_emitted: usize,
    pub last_block_particle_draw_call: Option<DesktopGraphicsBlockParticleDrawCall>,
}

pub trait DesktopGraphicsLiveBackendDrawSpriteSink {
    fn consume_draw_sprite_trace(&mut self, trace: DesktopGraphicsLiveBackendDrawSpriteTrace);
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct DesktopGraphicsNullLiveBackendDrawSpriteSink;

impl DesktopGraphicsLiveBackendDrawSpriteSink for DesktopGraphicsNullLiveBackendDrawSpriteSink {
    fn consume_draw_sprite_trace(&mut self, _trace: DesktopGraphicsLiveBackendDrawSpriteTrace) {}
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DesktopGraphicsExecutionTrace {
    pub shader_dispatch: DesktopGraphicsShaderDispatchExecutionTrace,
    pub block_particle_plans: usize,
    pub block_particle_world_samples: usize,
    pub block_particle_traces: Vec<DesktopGraphicsBlockParticleTrace>,
    pub block_particle_draw_calls: Vec<DesktopGraphicsBlockParticleDrawCall>,
    pub block_particle_render_commands: Vec<RenderCommand>,
    pub execution_steps: Vec<DesktopGraphicsExecutionStepTrace>,
    pub render_target_traces: Vec<DesktopGraphicsLiveBackendRenderTargetTrace>,
    pub render_passes: Vec<DesktopGraphicsPassExecutionTrace>,
}

impl DesktopGraphicsExecutionTrace {
    pub fn from_frame(frame: &DesktopGraphicsFrame) -> Self {
        Self::from_frame_and_atlas(frame, Some(&frame.texture_atlas))
    }

    pub fn from_frame_with_atlas<T>(
        frame: &DesktopGraphicsFrame,
        atlas: &TextureAtlasPlan<T>,
    ) -> Self {
        Self::from_frame_and_atlas(frame, Some(atlas))
    }

    pub fn drive_draw_sprite_sink<S: DesktopGraphicsLiveBackendDrawSpriteSink>(
        &self,
        sink: &mut S,
    ) -> DesktopGraphicsLiveBackendExecutionState {
        let mut state = DesktopGraphicsLiveBackendExecutionState::default();

        for (pass_index, pass) in self.render_passes.iter().enumerate() {
            state.render_passes_visited += 1;
            let mut draw_sprite_index = 0usize;

            for (command_index, command) in pass.command_trace.iter().enumerate() {
                state.render_commands_visited += 1;

                if let DesktopGraphicsCommandExecutionTrace::DrawSprite { symbol } = command {
                    let draw_sprite_trace = DesktopGraphicsLiveBackendDrawSpriteTrace {
                        pass_index,
                        command_index,
                        pass_kind: pass.kind.clone(),
                        pass_order: pass.order,
                        target: pass.target.clone(),
                        symbol: symbol.clone(),
                        resolved_sprite: pass.resolved_sprites.get(draw_sprite_index).cloned(),
                    };
                    sink.consume_draw_sprite_trace(draw_sprite_trace.clone());
                    state.draw_sprite_traces_emitted += 1;
                    state.last_draw_sprite_trace = Some(draw_sprite_trace);
                    draw_sprite_index += 1;
                }
            }
        }

        state
    }

    pub fn drive_render_command_sink<S: DesktopGraphicsLiveBackendRenderCommandSink>(
        &self,
        sink: &mut S,
    ) -> DesktopGraphicsLiveBackendExecutionState {
        let mut state = DesktopGraphicsLiveBackendExecutionState::default();

        let block_particle_commands_in_render_pass =
            !self.block_particle_render_commands.is_empty()
                && self
                    .render_passes
                    .iter()
                    .any(|pass| pass.commands == self.block_particle_render_commands);

        if !block_particle_commands_in_render_pass {
            for (command_index, command) in self.block_particle_render_commands.iter().enumerate() {
                let trace = DesktopGraphicsLiveBackendRenderCommandTrace {
                    source: DesktopGraphicsLiveBackendRenderCommandSource::BlockParticles {
                        command_index,
                    },
                    kind: render_command_trace_kind(command),
                    command: command.clone(),
                };
                sink.consume_render_command_trace(trace.clone());
                state.backend_render_commands_emitted += 1;
                state.last_backend_render_command = Some(trace);
            }
        }

        for (pass_index, pass) in self.render_passes.iter().enumerate() {
            state.render_passes_visited += 1;
            for (command_index, command) in pass.commands.iter().enumerate() {
                let trace = DesktopGraphicsLiveBackendRenderCommandTrace {
                    source: DesktopGraphicsLiveBackendRenderCommandSource::RenderPass {
                        pass_index,
                        command_index,
                        pass_kind: pass.kind.clone(),
                        pass_order: pass.order,
                        target: pass.target.clone(),
                    },
                    kind: render_command_trace_kind(command),
                    command: command.clone(),
                };
                sink.consume_render_command_trace(trace.clone());
                state.backend_render_commands_emitted += 1;
                state.last_backend_render_command = Some(trace);
            }
        }

        state
    }

    pub fn drive_render_target_sink<S: DesktopGraphicsLiveBackendRenderTargetSink>(
        &self,
        sink: &mut S,
    ) -> DesktopGraphicsLiveBackendExecutionState {
        let mut state = DesktopGraphicsLiveBackendExecutionState::default();
        state.render_passes_visited = self.render_passes.len();

        for trace in &self.render_target_traces {
            sink.consume_render_target_trace(trace.clone());
            state.backend_target_events_emitted += 1;
            if trace.event == DesktopGraphicsLiveBackendRenderTargetEventKind::Resolve {
                state.resolve_target_events_emitted += 1;
            }
            match &trace.target {
                RenderTarget::Screen => state.screen_target_events_emitted += 1,
                RenderTarget::Texture(_) => state.texture_target_events_emitted += 1,
                RenderTarget::Buffer(_) => state.buffer_target_events_emitted += 1,
            }
            state.last_backend_target_event = Some(trace.clone());
        }

        state
    }

    pub fn drive_live_backend_sinks<
        S: DesktopGraphicsLiveBackendDrawSpriteSink,
        P: DesktopGraphicsLiveBackendBlockParticleSink,
        D: DesktopGraphicsLiveBackendBlockParticleDrawCallSink,
        R: DesktopGraphicsLiveBackendRenderCommandSink,
        T: DesktopGraphicsLiveBackendRenderTargetSink,
    >(
        &self,
        draw_sprite_sink: &mut S,
        block_particle_sink: &mut P,
        block_particle_draw_call_sink: &mut D,
        render_command_sink: &mut R,
        render_target_sink: &mut T,
    ) -> DesktopGraphicsLiveBackendExecutionState {
        let mut state = self.drive_draw_sprite_sink(draw_sprite_sink);

        for trace in &self.block_particle_traces {
            block_particle_sink.consume_block_particle_trace(trace.clone());
            state.block_particle_traces_emitted += 1;
            state.last_block_particle_trace = Some(trace.clone());
        }

        for draw_call in &self.block_particle_draw_calls {
            block_particle_draw_call_sink.consume_block_particle_draw_call(draw_call.clone());
            state.block_particle_draw_calls_emitted += 1;
            state.last_block_particle_draw_call = Some(draw_call.clone());
        }

        let render_command_state = self.drive_render_command_sink(render_command_sink);
        state.backend_render_commands_emitted =
            render_command_state.backend_render_commands_emitted;
        state.last_backend_render_command = render_command_state.last_backend_render_command;

        let render_target_state = self.drive_render_target_sink(render_target_sink);
        state.backend_target_events_emitted = render_target_state.backend_target_events_emitted;
        state.resolve_target_events_emitted = render_target_state.resolve_target_events_emitted;
        state.screen_target_events_emitted = render_target_state.screen_target_events_emitted;
        state.texture_target_events_emitted = render_target_state.texture_target_events_emitted;
        state.buffer_target_events_emitted = render_target_state.buffer_target_events_emitted;
        state.last_backend_target_event = render_target_state.last_backend_target_event;

        state
    }

    fn from_frame_and_atlas<T>(
        frame: &DesktopGraphicsFrame,
        atlas: Option<&TextureAtlasPlan<T>>,
    ) -> Self {
        let shader_dispatch = frame.bundle.shader_dispatch.as_ref().map_or_else(
            DesktopGraphicsShaderDispatchExecutionTrace::default,
            |dispatch| DesktopGraphicsShaderDispatchExecutionTrace {
                applies: dispatch
                    .applies
                    .iter()
                    .map(|apply| DesktopGraphicsShaderApplyExecutionTrace {
                        shader: apply.shader,
                        operation_count: apply.operations.len(),
                        error_count: apply.errors.len(),
                    })
                    .collect(),
            },
        );

        let mut execution_steps = Vec::new();
        if frame.bundle.shader_dispatch.is_some() {
            execution_steps.push(DesktopGraphicsExecutionStepTrace::ShaderDispatch {
                apply_count: shader_dispatch.applies.len(),
            });
        }
        let block_particle_plans = frame
            .bundle
            .block_renderer
            .as_ref()
            .map_or(0, |block_renderer| block_renderer.block_particles.len());
        let block_particle_traces =
            frame
                .bundle
                .block_renderer
                .as_ref()
                .map_or_else(Vec::new, |block_renderer| {
                    block_renderer
                        .block_particles
                        .iter()
                        .enumerate()
                        .flat_map(|(plan_index, particle)| {
                            particle.world_samples(8.0).into_iter().map(move |sample| {
                                DesktopGraphicsBlockParticleTrace {
                                    plan_index,
                                    sample_index: sample.index,
                                    coord: particle.coord,
                                    block: particle.block.clone(),
                                    sample,
                                }
                            })
                        })
                        .collect()
                });
        let block_particle_world_samples = block_particle_traces.len();
        let block_particle_draw_calls: Vec<DesktopGraphicsBlockParticleDrawCall> =
            block_particle_traces
                .iter()
                .map(DesktopGraphicsBlockParticleDrawCall::from_trace)
                .collect();
        let block_particle_render_commands = frame
            .bundle
            .block_renderer
            .as_ref()
            .map_or_else(Vec::new, |block_renderer| {
                block_renderer.to_block_particle_render_commands(8.0)
            });
        if block_particle_plans > 0 {
            execution_steps.push(DesktopGraphicsExecutionStepTrace::BlockParticles {
                plan_count: block_particle_plans,
            });
        }

        let render_passes =
            frame
                .bundle
                .render_frame
                .as_ref()
                .map_or_else(Vec::new, |render_frame| {
                    render_frame
                        .passes
                        .iter()
                        .map(|pass| {
                            execution_steps.push(DesktopGraphicsExecutionStepTrace::RenderPass {
                                kind: pass.kind.clone(),
                                order: pass.order,
                                target: pass.target.clone(),
                            });

                            let mut command_trace = Vec::with_capacity(pass.commands.len());
                            let mut draw_sprite_symbols = Vec::new();
                            let mut resolved_sprites = Vec::new();
                            let mut draw_texts = Vec::new();
                            let mut draw_polygon_sides = Vec::new();
                            for command in &pass.commands {
                                match command {
                                    RenderCommand::DrawSprite { symbol, .. } => {
                                        command_trace.push(
                                            DesktopGraphicsCommandExecutionTrace::DrawSprite {
                                                symbol: symbol.clone(),
                                            },
                                        );
                                        draw_sprite_symbols.push(symbol.clone());
                                        if let Some(atlas) = atlas {
                                            resolved_sprites
                                                .push(resolve_sprite_symbol(atlas, symbol));
                                        }
                                    }
                                    RenderCommand::DrawText { text, .. } => {
                                        command_trace.push(
                                            DesktopGraphicsCommandExecutionTrace::DrawText {
                                                text: text.clone(),
                                            },
                                        );
                                        draw_texts.push(text.clone());
                                    }
                                    RenderCommand::DrawPolygon { sides, .. } => {
                                        command_trace.push(
                                            DesktopGraphicsCommandExecutionTrace::DrawPolygon {
                                                sides: *sides,
                                            },
                                        );
                                        draw_polygon_sides.push(*sides);
                                    }
                                    other => command_trace.push(
                                        DesktopGraphicsCommandExecutionTrace::NoOp {
                                            kind: render_command_trace_kind(other).to_string(),
                                        },
                                    ),
                                }
                            }

                            DesktopGraphicsPassExecutionTrace {
                                kind: pass.kind.clone(),
                                order: pass.order,
                                target: pass.target.clone(),
                                resolve_target: pass.resolve_target.clone(),
                                resolve_kind: pass.resolve_kind,
                                command_count: pass.commands.len(),
                                commands: pass.commands.clone(),
                                command_trace,
                                draw_sprite_symbols,
                                resolved_sprites,
                                draw_texts,
                                draw_polygon_sides,
                            }
                        })
                        .collect()
                });

        let render_target_traces = render_passes
            .iter()
            .enumerate()
            .flat_map(|(pass_index, pass)| {
                let mut events = vec![
                    DesktopGraphicsLiveBackendRenderTargetEventKind::Begin,
                    DesktopGraphicsLiveBackendRenderTargetEventKind::End,
                ];
                if pass.resolve_target.is_some() {
                    events.push(DesktopGraphicsLiveBackendRenderTargetEventKind::Resolve);
                }
                events
                    .into_iter()
                    .map(move |event| DesktopGraphicsLiveBackendRenderTargetTrace {
                        pass_index,
                        pass_kind: pass.kind.clone(),
                        pass_order: pass.order,
                        target: pass.target.clone(),
                        resolve_target: pass.resolve_target.clone(),
                        resolve_kind: pass.resolve_kind,
                        event,
                        command_count: pass.command_count,
                    })
            })
            .collect();

        Self {
            shader_dispatch,
            block_particle_plans,
            block_particle_world_samples,
            block_particle_traces,
            block_particle_draw_calls,
            block_particle_render_commands,
            execution_steps,
            render_target_traces,
            render_passes,
        }
    }
}

fn resolve_sprite_symbol<T>(
    atlas: &TextureAtlasPlan<T>,
    symbol: &str,
) -> DesktopGraphicsResolvedSpriteTrace {
    let linear_filter = atlas.linear_filter();
    let sampler = DesktopGraphicsTextureSamplerTrace::from_linear_filter(linear_filter);
    match atlas.lookup(symbol) {
        Ok(located) => {
            let page = atlas.page(located.page_type);
            DesktopGraphicsResolvedSpriteTrace {
                symbol: symbol.to_string(),
                page_type: Some(located.page_type),
                page_source_path: Some(located.page_source_path.to_string()),
                page_width: Some(page.spec.width),
                page_height: Some(page.spec.height),
                linear_filter,
                sampler,
                region_source_path: Some(located.region.source_path.clone()),
                x: Some(located.region.x),
                y: Some(located.region.y),
                u: Some(located.region.u),
                v: Some(located.region.v),
                u2: Some(located.region.u2),
                v2: Some(located.region.v2),
                region_width: Some(located.region.width),
                region_height: Some(located.region.height),
                missing: false,
            }
        }
        Err(miss) => DesktopGraphicsResolvedSpriteTrace {
            symbol: symbol.to_string(),
            page_type: miss.page_type,
            page_source_path: miss.page_source_path,
            page_width: miss
                .page_type
                .map(|page_type| atlas.page(page_type).spec.width),
            page_height: miss
                .page_type
                .map(|page_type| atlas.page(page_type).spec.height),
            linear_filter,
            sampler,
            region_source_path: None,
            x: None,
            y: None,
            u: None,
            v: None,
            u2: None,
            v2: None,
            region_width: None,
            region_height: None,
            missing: true,
        },
    }
}

fn render_command_trace_kind(command: &RenderCommand) -> &'static str {
    match command {
        RenderCommand::Clear { .. } => "Clear",
        RenderCommand::SetBlend { .. } => "SetBlend",
        RenderCommand::SetClip { .. } => "SetClip",
        RenderCommand::ClearClip => "ClearClip",
        RenderCommand::FillRect { .. } => "FillRect",
        RenderCommand::StrokeRect { .. } => "StrokeRect",
        RenderCommand::DrawSprite { .. } => "DrawSprite",
        RenderCommand::DrawLine { .. } => "DrawLine",
        RenderCommand::DrawCircle { .. } => "DrawCircle",
        RenderCommand::DrawPolygon { .. } => "DrawPolygon",
        RenderCommand::DrawPixel { .. } => "DrawPixel",
        RenderCommand::DrawText { .. } => "DrawText",
        RenderCommand::Custom { .. } => "Custom",
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DesktopGraphicsExecutionSummary {
    pub render_passes_visited: usize,
    pub render_commands_visited: usize,
    pub draw_sprite_commands: usize,
    pub draw_text_commands: usize,
    pub draw_polygon_commands: usize,
    pub shader_dispatch_applies: usize,
    pub shader_dispatch_operations: usize,
    pub shader_dispatch_errors: usize,
    pub screen_target_passes: usize,
    pub texture_target_passes: usize,
    pub buffer_target_passes: usize,
    pub atlas_resolved_sprites: usize,
    pub atlas_missing_sprites: usize,
    pub block_renderer_slots: usize,
    pub block_particle_plans: usize,
    pub block_particle_world_samples: usize,
    pub block_particle_draw_calls: usize,
    pub block_particle_render_commands: usize,
    pub floor_renderer_slots: usize,
    pub fog_frame_slots: usize,
    pub overlay_renderer_slots: usize,
    pub minimap_overlay_slots: usize,
    pub pixelator_slots: usize,
    pub floor_chunk_batches: usize,
    pub minimap_texture_frames: usize,
    pub minimap_full_uploads: usize,
    pub minimap_dirty_pixels: usize,
}

impl DesktopGraphicsExecutionSummary {
    pub fn from_frame(frame: &DesktopGraphicsFrame) -> Self {
        let trace = DesktopGraphicsExecutionTrace::from_frame(frame);
        Self::from_trace(frame, &trace)
    }

    fn from_trace(frame: &DesktopGraphicsFrame, trace: &DesktopGraphicsExecutionTrace) -> Self {
        let mut summary = Self::default();

        summary.render_passes_visited = trace.render_passes.len();
        for pass in &trace.render_passes {
            summary.render_commands_visited += pass.command_count;
            summary.draw_sprite_commands += pass.draw_sprite_symbols.len();
            summary.atlas_resolved_sprites += pass
                .resolved_sprites
                .iter()
                .filter(|sprite| !sprite.missing)
                .count();
            summary.atlas_missing_sprites += pass
                .resolved_sprites
                .iter()
                .filter(|sprite| sprite.missing)
                .count();
            summary.draw_text_commands += pass.draw_texts.len();
            summary.draw_polygon_commands += pass.draw_polygon_sides.len();
            match &pass.target {
                RenderTarget::Screen => summary.screen_target_passes += 1,
                RenderTarget::Texture(_) => summary.texture_target_passes += 1,
                RenderTarget::Buffer(_) => summary.buffer_target_passes += 1,
            }
        }

        summary.shader_dispatch_applies = trace.shader_dispatch.applies.len();
        summary.shader_dispatch_operations = trace
            .shader_dispatch
            .applies
            .iter()
            .map(|apply| apply.operation_count)
            .sum();
        summary.shader_dispatch_errors = trace
            .shader_dispatch
            .applies
            .iter()
            .map(|apply| apply.error_count)
            .sum();
        summary.block_renderer_slots = usize::from(frame.bundle.block_renderer.is_some());
        summary.block_particle_plans = trace.block_particle_plans;
        summary.block_particle_world_samples = trace.block_particle_world_samples;
        summary.block_particle_draw_calls = trace.block_particle_draw_calls.len();
        summary.block_particle_render_commands = trace.block_particle_render_commands.len();
        summary.floor_renderer_slots = usize::from(frame.bundle.floor_renderer.is_some());
        summary.fog_frame_slots = usize::from(frame.bundle.fog_frame.is_some());
        summary.overlay_renderer_slots = usize::from(frame.bundle.overlay_renderer.is_some());
        summary.minimap_overlay_slots = usize::from(frame.bundle.minimap_overlay.is_some());
        summary.pixelator_slots = usize::from(frame.bundle.pixelator.is_some());
        summary.floor_chunk_batches = frame.floor_chunk_batches.len();
        if let Some(minimap_texture) = &frame.minimap_texture_frame {
            summary.minimap_texture_frames = 1;
            summary.minimap_full_uploads = usize::from(minimap_texture.full_upload.is_some());
            summary.minimap_dirty_pixels = minimap_texture.dirty_pixels.len();
        }
        summary
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopFrameKind {
    World,
    Menu,
    Load,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DesktopFramePayload {
    World(DesktopGraphicsFrame),
    Menu(MenuFramePlan),
    Load(LoadFramePlan),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopFrame {
    pub kind: DesktopFrameKind,
    pub payload: DesktopFramePayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopSurfaceSize {
    pub width: u32,
    pub height: u32,
}

impl DesktopSurfaceSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Default for DesktopSurfaceSize {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopSurfaceConfig {
    pub title: String,
    pub size: DesktopSurfaceSize,
    pub scale_factor: f32,
    pub resizable: bool,
    pub visible: bool,
}

impl Default for DesktopSurfaceConfig {
    fn default() -> Self {
        Self {
            title: "Mindustry".into(),
            size: DesktopSurfaceSize::default(),
            scale_factor: 1.0,
            resizable: true,
            visible: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopFramePacing {
    pub target_frame_time: Duration,
}

impl DesktopFramePacing {
    pub const fn new(target_frame_time: Duration) -> Self {
        Self { target_frame_time }
    }

    pub const fn uncapped() -> Self {
        Self {
            target_frame_time: Duration::ZERO,
        }
    }

    pub fn is_paced(self) -> bool {
        !self.target_frame_time.is_zero()
    }
}

impl Default for DesktopFramePacing {
    fn default() -> Self {
        Self {
            target_frame_time: Duration::from_millis(16),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DesktopInputTickEvent {
    Key { key_code: String, pressed: bool },
    MouseButton { button: String, pressed: bool },
    CursorMoved { x: f32, y: f32 },
    Text(String),
    Scroll { delta_x: f32, delta_y: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DesktopFrameLoopEvent {
    Tick,
    Resize(DesktopSurfaceSize),
    Input(DesktopInputTickEvent),
    CloseRequested,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopFrameLoopState {
    pub surface: DesktopSurfaceConfig,
    pub pacing: DesktopFramePacing,
    pub next_frame_index: u64,
    pub closed: bool,
    pub input_events_seen: u64,
}

impl DesktopFrameLoopState {
    pub fn new(surface: DesktopSurfaceConfig, pacing: DesktopFramePacing) -> Self {
        Self {
            surface,
            pacing,
            next_frame_index: 0,
            closed: false,
            input_events_seen: 0,
        }
    }

    pub fn request_close(&mut self) {
        self.closed = true;
    }
}

impl Default for DesktopFrameLoopState {
    fn default() -> Self {
        Self::new(
            DesktopSurfaceConfig::default(),
            DesktopFramePacing::default(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopFrameSkipReason {
    AlreadyClosed,
    CloseRequested,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopPresentResult {
    pub frame_index: u64,
    pub surface: DesktopSurfaceConfig,
    pub presented: bool,
    pub skip_reason: Option<DesktopFrameSkipReason>,
    pub close_requested: bool,
    pub resized_to: Option<DesktopSurfaceSize>,
    pub input_events: Vec<DesktopInputTickEvent>,
    pub graphics_stats: Option<GraphicsFrameStats>,
    pub effect_stats: Option<DesktopEffectRenderStats>,
}

impl DesktopPresentResult {
    pub fn should_stop(&self) -> bool {
        self.close_requested
            || matches!(
                self.skip_reason,
                Some(DesktopFrameSkipReason::AlreadyClosed)
            )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopFrameLoopExitReason {
    FrameLimit,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopFrameLoopRunSummary {
    pub steps: u64,
    pub frames_presented: u64,
    pub last_frame_index: Option<u64>,
    pub exit_reason: DesktopFrameLoopExitReason,
}

pub trait DesktopGraphicsRenderer {
    fn render_graphics_frame(&mut self, frame: &DesktopGraphicsFrame) -> GraphicsFrameStats;
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HeadlessDesktopGraphicsRenderer {
    pub frames_rendered: usize,
    pub last_stats: GraphicsFrameStats,
    pub last_execution: DesktopGraphicsExecutionSummary,
    pub last_trace: DesktopGraphicsExecutionTrace,
    pub last_live_backend_state: DesktopGraphicsLiveBackendExecutionState,
}

impl DesktopGraphicsRenderer for HeadlessDesktopGraphicsRenderer {
    fn render_graphics_frame(&mut self, frame: &DesktopGraphicsFrame) -> GraphicsFrameStats {
        let stats = frame.stats().clone();
        let trace = DesktopGraphicsExecutionTrace::from_frame(frame);
        let execution = DesktopGraphicsExecutionSummary::from_trace(frame, &trace);
        let mut live_backend_sink = DesktopGraphicsNullLiveBackendDrawSpriteSink;
        let mut block_particle_sink = DesktopGraphicsNullLiveBackendBlockParticleSink;
        let mut block_particle_draw_call_sink =
            DesktopGraphicsNullLiveBackendBlockParticleDrawCallSink;
        let mut render_command_sink = DesktopGraphicsNullLiveBackendRenderCommandSink;
        let mut render_target_sink = DesktopGraphicsNullLiveBackendRenderTargetSink;
        let live_backend_state = trace.drive_live_backend_sinks(
            &mut live_backend_sink,
            &mut block_particle_sink,
            &mut block_particle_draw_call_sink,
            &mut render_command_sink,
            &mut render_target_sink,
        );
        self.frames_rendered += 1;
        self.last_stats = stats.clone();
        self.last_execution = execution;
        self.last_trace = trace;
        self.last_live_backend_state = live_backend_state;
        stats
    }
}

#[derive(Debug, Clone)]
pub struct DesktopLauncher {
    pub client: ClientLauncher,
    pub net_client: NetClient,
    pub game_state: GameState,
    pub runtime: GameRuntime,
    pub player: PlayerComp,
    pub remote_players: BTreeMap<i32, PlayerComp>,
    pub other_player_preview_overlays: Vec<OtherPlayerPreviewOverlayPlan>,
    pub standard_local_effect_draw_plans: Vec<StandardEffectDrawPlan>,
    pub standard_local_effect_circle_primitives: Vec<StandardEffectCircleRenderPrimitive>,
    pub standard_local_effect_square_primitives: Vec<StandardEffectSquareRenderPrimitive>,
    pub standard_local_effect_rect_primitives: Vec<StandardEffectRectRenderPrimitive>,
    pub standard_local_effect_line_primitives: Vec<StandardEffectLineRenderPrimitive>,
    pub standard_local_effect_triangle_primitives: Vec<StandardEffectTriangleRenderPrimitive>,
    pub standard_local_effect_light_primitives: Vec<StandardEffectLightRenderPrimitive>,
    pub pending_sound_at_events: Vec<SoundAtCallPacket>,
    pub pending_camera_shake_events: Vec<GameRuntimeClientCameraShakeEvent>,
    pub camera_shake_state: DesktopCameraShakeState,
    pub last_camera_shake_frame: DesktopCameraShakeFrame,
    pub overlay_renderer_state: OverlayRendererState,
    pub block_renderer_state: BlockRendererState,
    pub light_renderer_state: LightRendererState,
    pub floor_renderer_state: FloorRendererState,
    pub fog_renderer_state: FogRendererState,
    pub minimap_renderer_state: MinimapRendererState,
    pub menu_renderer_state: MenuRendererState,
    pub load_renderer_state: LoadRendererState,
    pub pixelator_state: PixelatorState,
    pub pixelate: bool,
    pub renderer_scale: f32,
    pub land_scale: f32,
    pub cutscene: bool,
    pub connect_target: Option<DesktopConnectTarget>,
    pub connect_error: Option<String>,
    pub mods_directory_arg: Option<String>,
    pub mods_directory_error: Option<String>,
    pub last_mods_directory_merge_count: Option<usize>,
    pub args: Vec<String>,
    pub texture_atlas: TextureAtlasPlan<bool>,
    content_loader: ContentLoader,
    last_applied_world_data: Option<mindustry_core::mindustry::net::NetworkWorldData>,
    last_applied_state_snapshot: Option<StateSnapshotCallPacket>,
    last_applied_block_snapshot_mirror: Option<ClientBlockSnapshotMirror>,
    last_applied_entity_snapshot_mirror_count: usize,
    last_applied_hidden_snapshot_mirror: Option<ClientHiddenSnapshotMirror>,
    last_applied_building_storage_mirrors: BTreeMap<i32, ClientTileStorageMirror>,
    last_applied_unit_item_mirrors: BTreeMap<i32, ClientUnitItemMirror>,
    last_applied_unit_payload_mirrors: BTreeMap<i32, ClientUnitPayloadMirror>,
    last_applied_unit_entered_payload_packets_seen: u64,
    last_applied_unit_tether_block_spawned_packets_seen: u64,
    last_applied_unit_lifecycle_packets_seen: u64,
    last_applied_unit_spawn_packet_count: usize,
    last_applied_world_update_packets_seen: u64,
    last_applied_tile_config_packets_seen: u64,
    last_applied_command_building_packets_seen: u64,
    last_applied_effect_packets_seen: u64,
    last_applied_effect_with_data_packets_seen: u64,
    last_applied_reliable_effect_packets_seen: u64,
    last_unit_entered_payload_apply_report: Option<GameRuntimeClientUnitEnteredPayloadApplyReport>,
    last_tile_config_apply_result: Option<GameRuntimeUnitCargoUnloadConfigureResult>,
    last_unit_factory_tile_config_apply_result: Option<GameRuntimeUnitFactoryConfigureResult>,
    last_reconstructor_tile_config_apply_result: Option<GameRuntimeReconstructorConfigureResult>,
    last_command_building_apply_report: Option<GameRuntimeCommandBuildingReport>,
    last_runtime_map_load_report: Option<GameRuntimeMapLoadReport>,
    last_client_snapshot_apply_report: Option<GameRuntimeClientSnapshotApplyReport>,
    last_service_trigger_apply_summary: Option<GameServiceApplySummary>,
    last_applied_client_plan_snapshot_received_count: usize,
    puddle_particle_rand_state: u64,
}

fn visible_block_tiles(
    mut camera: RenderCamera,
    viewport: RenderViewport,
    world: MinimapWorldSize,
    tile_size_world: f32,
) -> Vec<TileCoord> {
    if world.width <= 0 || world.height <= 0 || tile_size_world <= 0.0 {
        return Vec::new();
    }

    camera.viewport = viewport;
    let rect = camera.world_rect();
    let min_x = (rect.x / tile_size_world).floor() as i32;
    let min_y = (rect.y / tile_size_world).floor() as i32;
    let max_x = ((rect.x + rect.width) / tile_size_world).ceil() as i32;
    let max_y = ((rect.y + rect.height) / tile_size_world).ceil() as i32;

    let min_x = min_x.clamp(0, world.width);
    let min_y = min_y.clamp(0, world.height);
    let max_x = max_x.clamp(0, world.width);
    let max_y = max_y.clamp(0, world.height);

    if min_x >= max_x || min_y >= max_y {
        return Vec::new();
    }

    let mut tiles = Vec::with_capacity(((max_x - min_x) * (max_y - min_y)) as usize);
    for y in min_y..max_y {
        for x in min_x..max_x {
            tiles.push(TileCoord::new(x, y));
        }
    }
    tiles
}

fn graphics_cache_layer_from_world(layer: WorldCacheLayer) -> GraphicsCacheLayer {
    match layer {
        WorldCacheLayer::None => GraphicsCacheLayer::None,
        WorldCacheLayer::Water => GraphicsCacheLayer::Water,
        WorldCacheLayer::Mud => GraphicsCacheLayer::Mud,
        WorldCacheLayer::Tar => GraphicsCacheLayer::Tar,
        WorldCacheLayer::Slag => GraphicsCacheLayer::Slag,
        WorldCacheLayer::Arkycite => GraphicsCacheLayer::Arkycite,
        WorldCacheLayer::Cryofluid => GraphicsCacheLayer::Cryofluid,
        WorldCacheLayer::Space => GraphicsCacheLayer::Space,
        WorldCacheLayer::Normal => GraphicsCacheLayer::Normal,
        WorldCacheLayer::Walls => GraphicsCacheLayer::Walls,
    }
}

fn block_renderer_visual_runtime_snapshot_from_game_runtime(
    snapshot: GameRuntimeBlockVisualRuntimeSnapshot,
) -> BlockRendererBuildingVisualRuntimeSnapshot {
    BlockRendererBuildingVisualRuntimeSnapshot {
        liquid: snapshot
            .liquid
            .map(|liquid| BlockRendererBuildingVisualRuntimeLiquidSnapshot {
                current: liquid.current,
                amount: liquid.amount,
                capacity: liquid.capacity,
            }),
        progress: snapshot.progress,
        heat: snapshot.heat,
        warmup: snapshot.warmup,
        total_progress: snapshot.total_progress,
        charge: snapshot.charge,
        power: snapshot
            .power
            .map(|power| BlockRendererBuildingVisualRuntimePowerSnapshot {
                status: power.status,
                production_efficiency: power.production_efficiency,
            }),
        turret: snapshot
            .turret
            .map(|turret| BlockRendererBuildingVisualRuntimeTurretSnapshot {
                rotation: turret.rotation,
                recoil: turret.recoil,
                heat: turret.heat,
                charge: turret.charge,
                side_heat: turret.side_heat,
            }),
    }
}

fn block_drawer_from_content_block(block: &BlockDef) -> Option<&str> {
    match block {
        BlockDef::Turret(turret) => Some(&turret.drawer),
        BlockDef::Crafting(crafting) => Some(&crafting.drawer),
        BlockDef::Effect(effect) => Some(&effect.drawer),
        BlockDef::Liquid(liquid) => Some(&liquid.drawer),
        BlockDef::Power(power) => Some(&power.drawer),
        _ => None,
    }
    .filter(|drawer| !drawer.is_empty())
    .map(String::as_str)
}

fn block_renderer_building_snapshot_from_world(
    coord: TileCoord,
    tile_build: Option<BuildingRef>,
    runtime_building: Option<&BuildingComp>,
    visual_runtime: Option<GameRuntimeBlockVisualRuntimeSnapshot>,
    content_loader: &ContentLoader,
) -> Option<BlockRendererBuildingSnapshot> {
    let block_id = tile_build
        .map(|build| build.block)
        .or_else(|| runtime_building.map(|building| building.block.id))?;
    let content_block = content_loader.block(block_id);
    let block = runtime_building
        .map(|building| &building.block)
        .or_else(|| content_block.map(BlockDef::base));

    let mut snapshot = BlockRendererBuildingSnapshot::new(
        coord,
        block.map(|block| block.name.clone()).unwrap_or_default(),
    );

    if let Some(block) = block {
        snapshot.cache_layer = graphics_cache_layer_from_world(block.cache_layer);
        snapshot.size = block.size.max(1).min(u8::MAX as i32) as u8;
        snapshot.emits_light = block.emit_light;
        snapshot.draw_cracks = block.draw_cracks;
    }

    if let Some(def) = content_block {
        if let Some(drawer) = block_drawer_from_content_block(def) {
            snapshot.drawer = drawer.to_string();
        }
        snapshot.draw_team_overlay = matches!(def, BlockDef::DefenseWall(_));
        snapshot.draw_status =
            matches!(def, BlockDef::Sandbox(sandbox) if sandbox.enable_draw_status);
    }

    if let Some(building) = runtime_building {
        snapshot.build_id_seed = building.tile_pos;
        snapshot.rotation = building.rotation as i16;
        snapshot.team = building.team.0;
        snapshot.visible = true;
        snapshot.was_visible = building.was_visible;
        snapshot.damaged =
            building.health + f32::EPSILON < building.max_health || building.was_damaged;
        snapshot.health_fraction = building_health_fraction(building.health, building.max_health);
    } else if let Some(build_ref) = tile_build {
        snapshot.build_id_seed = build_ref.tile_pos;
        snapshot.rotation = build_ref.rotation as i16;
        snapshot.team = build_ref.team.clamp(0, u8::MAX as i32) as u8;
        snapshot.visible = true;
    }

    if let Some(visual_runtime) = visual_runtime {
        snapshot.visual_runtime = Some(block_renderer_visual_runtime_snapshot_from_game_runtime(
            visual_runtime,
        ));
    }

    Some(snapshot)
}

fn building_health_fraction(health: f32, max_health: f32) -> f32 {
    if max_health > 0.0 && health.is_finite() && max_health.is_finite() {
        (health / max_health).clamp(0.0, 1.0)
    } else {
        1.0
    }
}

fn default_desktop_texture_atlas(
    block_renderer_state: &BlockRendererState,
    content_loader: &ContentLoader,
) -> TextureAtlasPlan<bool> {
    TextureAtlasPlan::from_virtual_source_paths(
        content_icon_candidate_virtual_source_paths(content_loader)
            .into_iter()
            .chain(
                block_renderer_state
                    .crack_atlas
                    .virtual_source_paths()
                    .into_iter(),
            )
            .chain(content_loader.blocks().map(|block| {
                let name = &block.base().name;
                format!("sprites/blocks/{}.png", name)
            })),
    )
}

fn content_icon_candidate_virtual_source_paths(content_loader: &ContentLoader) -> Vec<String> {
    let mut paths = Vec::new();

    for block in content_loader.blocks() {
        let block = block.base();
        let icon_content = UnlockableContentBase::new(block.id, ContentType::Block, &block.name);
        push_icon_candidate_virtual_source_paths(&mut paths, &icon_content);
    }

    for item in content_loader.items() {
        push_icon_candidate_virtual_source_paths(&mut paths, &item.base);
    }

    for liquid in content_loader.liquids() {
        push_icon_candidate_virtual_source_paths(&mut paths, &liquid.base);
    }

    for unit in content_loader.units() {
        push_icon_candidate_virtual_source_paths(&mut paths, &unit.base);
    }

    for status in content_loader.status_effects() {
        push_icon_candidate_virtual_source_paths(&mut paths, &status.base);
    }

    paths
}

fn push_icon_candidate_virtual_source_paths(
    paths: &mut Vec<String>,
    content: &UnlockableContentBase,
) {
    let candidates = content.icon_candidates(None);
    if !candidates.generate_icons {
        return;
    }

    paths.extend(
        candidates
            .full_candidates
            .into_iter()
            .map(|name| format!("sprites/{}.png", name)),
    );
    paths.extend(
        candidates
            .ui_candidates
            .into_iter()
            .map(|name| format!("sprites/ui/{}.png", name)),
    );
}

fn block_renderer_tile_snapshot_from_world(
    tile: &Tile,
    runtime_building: Option<&BuildingComp>,
    visual_runtime: Option<GameRuntimeBlockVisualRuntimeSnapshot>,
    content_loader: &ContentLoader,
) -> BlockRendererTileSnapshot {
    let coord = TileCoord::new(tile.x as i32, tile.y as i32);
    let tile_build = tile.build;
    let block_def = content_loader.block(tile.block);
    let block = runtime_building
        .map(|building| &building.block)
        .or_else(|| block_def.map(BlockDef::base));

    let mut snapshot = BlockRendererTileSnapshot::new(
        coord,
        block.map(|block| block.name.clone()).unwrap_or_default(),
    );

    if let Some(block) = block {
        snapshot.cache_layer = graphics_cache_layer_from_world(block.cache_layer);
        snapshot.draw_custom_shadow = block.custom_shadow;
        snapshot.emits_light = block.emit_light;
        snapshot.obstructs_light = block.obstructs_light;
        snapshot.darkness = Some(tile.static_darkness(block) as f32);
    }

    snapshot.building = block_renderer_building_snapshot_from_world(
        coord,
        tile_build,
        runtime_building,
        visual_runtime,
        content_loader,
    );
    snapshot
}

impl DesktopLauncher {
    pub fn new(args: Vec<String>) -> Self {
        let connect_target = parse_connect_target(&args);
        let mods_directory_arg = parse_mods_directory_arg(&args);
        let block_renderer_state = BlockRendererState::default();
        let content_loader = ContentLoader::create_base_content_or_panic();
        let texture_atlas = default_desktop_texture_atlas(&block_renderer_state, &content_loader);
        Self {
            client: ClientLauncher::new(AppContext::new("data")),
            net_client: NetClient::with_net(Net::new(Box::new(ArcNetProvider::new()))),
            game_state: GameState::new(),
            runtime: GameRuntime::default(),
            player: PlayerComp::default(),
            remote_players: BTreeMap::new(),
            other_player_preview_overlays: Vec::new(),
            standard_local_effect_draw_plans: Vec::new(),
            standard_local_effect_circle_primitives: Vec::new(),
            standard_local_effect_square_primitives: Vec::new(),
            standard_local_effect_rect_primitives: Vec::new(),
            standard_local_effect_line_primitives: Vec::new(),
            standard_local_effect_triangle_primitives: Vec::new(),
            standard_local_effect_light_primitives: Vec::new(),
            pending_sound_at_events: Vec::new(),
            pending_camera_shake_events: Vec::new(),
            camera_shake_state: DesktopCameraShakeState::default(),
            last_camera_shake_frame: DesktopCameraShakeFrame::default(),
            overlay_renderer_state: OverlayRendererState::default(),
            block_renderer_state,
            light_renderer_state: LightRendererState::default(),
            floor_renderer_state: FloorRendererState::default(),
            fog_renderer_state: FogRendererState::default(),
            minimap_renderer_state: MinimapRendererState::new(MinimapWorldSize::new(0, 0)),
            menu_renderer_state: MenuRendererState::new(MenuRendererConfig::new(false, 7)),
            load_renderer_state: LoadRendererState::default(),
            pixelator_state: PixelatorState::default(),
            pixelate: false,
            renderer_scale: 1.0,
            land_scale: 1.0,
            cutscene: false,
            connect_target,
            connect_error: None,
            mods_directory_arg,
            mods_directory_error: None,
            last_mods_directory_merge_count: None,
            args,
            texture_atlas,
            content_loader,
            last_applied_world_data: None,
            last_applied_state_snapshot: None,
            last_applied_block_snapshot_mirror: None,
            last_applied_entity_snapshot_mirror_count: 0,
            last_applied_hidden_snapshot_mirror: None,
            last_applied_building_storage_mirrors: BTreeMap::new(),
            last_applied_unit_item_mirrors: BTreeMap::new(),
            last_applied_unit_payload_mirrors: BTreeMap::new(),
            last_applied_unit_entered_payload_packets_seen: 0,
            last_applied_unit_tether_block_spawned_packets_seen: 0,
            last_applied_unit_lifecycle_packets_seen: 0,
            last_applied_unit_spawn_packet_count: 0,
            last_applied_world_update_packets_seen: 0,
            last_applied_tile_config_packets_seen: 0,
            last_applied_command_building_packets_seen: 0,
            last_applied_effect_packets_seen: 0,
            last_applied_effect_with_data_packets_seen: 0,
            last_applied_reliable_effect_packets_seen: 0,
            last_unit_entered_payload_apply_report: None,
            last_tile_config_apply_result: None,
            last_unit_factory_tile_config_apply_result: None,
            last_reconstructor_tile_config_apply_result: None,
            last_command_building_apply_report: None,
            last_runtime_map_load_report: None,
            last_client_snapshot_apply_report: None,
            last_service_trigger_apply_summary: None,
            last_applied_client_plan_snapshot_received_count: 0,
            puddle_particle_rand_state: DESKTOP_PUDDLE_PARTICLE_RAND_DEFAULT,
        }
    }

    pub fn merge_mod_directory_into_texture_atlas(
        &mut self,
        mod_name: impl Into<String>,
        headless: bool,
        root: impl AsRef<Path>,
    ) -> io::Result<usize> {
        let plan = ModResourcePlan::from_directory(mod_name, headless, root)?;
        Ok(self.merge_mod_resource_plan_into_texture_atlas(&plan))
    }

    pub fn merge_mods_directory_into_texture_atlas(
        &mut self,
        mods_dir: impl AsRef<Path>,
        headless: bool,
    ) -> io::Result<usize> {
        let container = ModResourceContainerPlan::discover_from_mods_directory(mods_dir, headless)?;
        Ok(self.merge_mod_resource_container_plan_into_texture_atlas(&container))
    }

    pub fn merge_mod_resource_container_plan_into_texture_atlas(
        &mut self,
        container: &ModResourceContainerPlan,
    ) -> usize {
        container
            .mods
            .iter()
            .map(|mod_dir| self.merge_mod_resource_plan_into_texture_atlas(&mod_dir.resource_plan))
            .sum()
    }

    pub fn merge_mods_directory_arg_into_texture_atlas(&mut self) -> io::Result<usize> {
        let Some(mods_dir) = self.mods_directory_arg.clone() else {
            self.last_mods_directory_merge_count = Some(0);
            self.mods_directory_error = None;
            return Ok(0);
        };

        match self.merge_mods_directory_into_texture_atlas(&mods_dir, false) {
            Ok(count) => {
                self.last_mods_directory_merge_count = Some(count);
                self.mods_directory_error = None;
                Ok(count)
            }
            Err(error) => {
                self.last_mods_directory_merge_count = None;
                self.mods_directory_error = Some(error.to_string());
                Err(error)
            }
        }
    }

    pub fn merge_mod_resource_plan_into_texture_atlas(&mut self, plan: &ModResourcePlan) -> usize {
        let mod_atlas = TextureAtlasPlan::from_sprite_sources(
            plan.sprite_requests().into_iter().map(|request| {
                let texture_scale = request.texture_scale();
                TextureAtlasSpriteSourceDescriptor::new(request.source_path, request.atlas_name)
                    .with_page_hint(request.page_hint)
                    .with_override(request.r#override)
                    .with_texture_scale(texture_scale)
            }),
        );
        let mut merged = 0;

        for page in mod_atlas.pages {
            let page_type = page.page_type;
            for region in page.regions {
                let _ = self
                    .texture_atlas
                    .insert_or_replace_region(page_type, region);
                merged += 1;
            }
        }

        merged
    }

    pub fn update(&mut self) {
        self.client.update();
        self.net_client.update();
        self.sync_loaded_world_data();
        self.sync_client_loaded_state();
        self.sync_state_snapshot();
        self.sync_snapshot_mirrors();
        self.runtime
            .update_client_effect_snapshot_parent_transforms();
        self.sync_building_storage_mirrors_to_runtime();
        self.sync_unit_item_mirrors_to_runtime();
        self.sync_unit_payload_mirrors_to_runtime();
        self.sync_unit_entered_payload_to_runtime();
        self.sync_unit_tether_block_spawned_to_runtime();
        self.sync_unit_lifecycle_to_runtime();
        self.sync_unit_spawn_packets_to_runtime();
        self.sync_world_update_events_to_runtime();
        self.sync_runtime_trigger_events_to_service();
        self.sync_tile_config_to_runtime();
        self.sync_command_building_to_runtime();
        self.sync_effect_packets_to_runtime();
        let now_millis = current_millis();
        self.sync_remote_player_snapshots_from_runtime();
        self.sync_remote_preview_plan_packets(now_millis);
        self.rebuild_other_player_preview_overlays_at(now_millis, 1.0, None);
        self.runtime.tick_client_move_effect_abilities(1.0, false);
        let mut puddle_particle_rand_state = self.puddle_particle_rand_state;
        self.runtime
            .tick_client_puddle_snapshot_particle_effects(1.0, |particle| {
                (
                    next_puddle_particle_range(&mut puddle_particle_rand_state, particle.range),
                    next_puddle_particle_range(&mut puddle_particle_rand_state, particle.range),
                )
            });
        self.puddle_particle_rand_state = puddle_particle_rand_state;
        self.materialize_local_effect_events_for_render();
        self.sync_local_sound_at_events_for_audio();
        self.sync_local_camera_shake_events_for_render(self.player.x, self.player.y);
        self.tick_camera_shake_for_render(1.0, 4);
        self.tick_local_effect_states_for_render(1.0);
        let standard_local_effect_draw_plans =
            self.collect_standard_local_effect_draw_plans_for_render();
        self.standard_local_effect_circle_primitives = standard_local_effect_draw_plans
            .iter()
            .flat_map(StandardEffectDrawPlan::circle_render_primitives_from_seed)
            .collect();
        self.standard_local_effect_square_primitives = standard_local_effect_draw_plans
            .iter()
            .flat_map(StandardEffectDrawPlan::square_render_primitives_from_seed)
            .collect();
        self.standard_local_effect_rect_primitives = standard_local_effect_draw_plans
            .iter()
            .flat_map(StandardEffectDrawPlan::rect_render_primitives_from_seed)
            .collect();
        self.standard_local_effect_line_primitives = standard_local_effect_draw_plans
            .iter()
            .flat_map(StandardEffectDrawPlan::line_render_primitives_from_seed)
            .collect();
        self.standard_local_effect_triangle_primitives = standard_local_effect_draw_plans
            .iter()
            .flat_map(StandardEffectDrawPlan::triangle_render_primitives_from_seed)
            .collect();
        self.standard_local_effect_light_primitives = standard_local_effect_draw_plans
            .iter()
            .flat_map(StandardEffectDrawPlan::light_render_primitives)
            .collect();
        self.standard_local_effect_draw_plans = standard_local_effect_draw_plans;
    }

    pub fn materialize_local_effect_events_for_render(&mut self) -> usize {
        self.runtime
            .drain_client_local_effect_events_to_states(|effect_id| {
                standard_effect(effect_id as i32)
            })
    }

    pub fn tick_local_effect_states_for_render(&mut self, delta: f32) -> usize {
        self.runtime.tick_client_local_effect_entities(delta)
    }

    pub fn draw_local_effect_states_for_render<F>(&mut self, render: F) -> usize
    where
        F: FnMut(EffectRenderInput<'_>) -> f32,
    {
        self.runtime.draw_client_local_effect_entities(render)
    }

    pub fn draw_standard_local_effect_states_for_render(&mut self) -> usize {
        self.collect_standard_local_effect_draw_plans_for_render()
            .len()
    }

    pub fn collect_standard_local_effect_draw_plans_for_render(
        &mut self,
    ) -> Vec<StandardEffectDrawPlan> {
        let mut plans = Vec::new();
        let unit_hit_sizes: BTreeMap<i32, f32> = self
            .runtime
            .client_unit_snapshot_entities
            .iter()
            .map(|(id, unit)| (*id, unit.hitbox.hit_size))
            .collect();
        let unit_shield_arcs: BTreeMap<i32, StandardEffectShieldArcBreak> = self
            .runtime
            .client_unit_snapshot_entities
            .iter()
            .filter_map(|(id, unit)| {
                let ability = unit
                    .type_info
                    .abilities
                    .iter()
                    .find_map(|descriptor| ShieldArcAbility::from_descriptor(descriptor))?;
                Some((
                    *id,
                    StandardEffectShieldArcBreak {
                        unit_x: unit.x(),
                        unit_y: unit.y(),
                        unit_rotation: unit.rotation(),
                        ability_x: ability.x,
                        ability_y: ability.y,
                        radius: ability.radius,
                        width: ability.width,
                        angle: ability.angle,
                        angle_offset: ability.angle_offset,
                    },
                ))
            })
            .collect();
        let block_full_icon_sizes: BTreeMap<ContentId, i32> = self
            .content_loader
            .blocks()
            .map(|block| (block.base().id, block.base().size))
            .collect();
        self.draw_local_effect_states_for_render(|input| {
            let resolved_unit_hit_size = match input.data {
                TypeValue::Unit(id) => unit_hit_sizes.get(id).copied(),
                _ => None,
            };
            let resolved_block_full_icon_size = match input.data {
                TypeValue::Content(content) if content.content_type == ContentType::Block => {
                    block_full_icon_sizes.get(&content.id).copied()
                }
                _ => None,
            };
            let resolved_shield_arc_break = match input.data {
                TypeValue::Unit(id) => unit_shield_arcs.get(id).copied(),
                _ => None,
            };
            plans.extend(
                standard_effect_draw_plans_with_data_value_and_resolved_context(
                    input.effect_id,
                    input.id,
                    input.x,
                    input.y,
                    input.rotation,
                    input.time,
                    input.lifetime,
                    input.color,
                    Some(input.data),
                    resolved_unit_hit_size,
                    resolved_block_full_icon_size,
                    resolved_shield_arc_break,
                ),
            );
            standard_effect_render_lifetime(input.effect_id, input.rotation, input.lifetime)
        });
        plans
    }

    pub fn collect_standard_local_effect_circle_primitives_for_render(
        &self,
    ) -> Vec<StandardEffectCircleRenderPrimitive> {
        self.standard_local_effect_draw_plans
            .iter()
            .flat_map(StandardEffectDrawPlan::circle_render_primitives_from_seed)
            .collect()
    }

    pub fn collect_standard_local_effect_light_primitives_for_render(
        &self,
    ) -> Vec<StandardEffectLightRenderPrimitive> {
        self.standard_local_effect_draw_plans
            .iter()
            .flat_map(StandardEffectDrawPlan::light_render_primitives)
            .collect()
    }

    pub fn collect_standard_local_effect_square_primitives_for_render(
        &self,
    ) -> Vec<StandardEffectSquareRenderPrimitive> {
        self.standard_local_effect_draw_plans
            .iter()
            .flat_map(StandardEffectDrawPlan::square_render_primitives_from_seed)
            .collect()
    }

    pub fn collect_standard_local_effect_rect_primitives_for_render(
        &self,
    ) -> Vec<StandardEffectRectRenderPrimitive> {
        self.standard_local_effect_draw_plans
            .iter()
            .flat_map(StandardEffectDrawPlan::rect_render_primitives_from_seed)
            .collect()
    }

    pub fn collect_standard_local_effect_line_primitives_for_render(
        &self,
    ) -> Vec<StandardEffectLineRenderPrimitive> {
        self.standard_local_effect_draw_plans
            .iter()
            .flat_map(StandardEffectDrawPlan::line_render_primitives_from_seed)
            .collect()
    }

    pub fn collect_standard_local_effect_triangle_primitives_for_render(
        &self,
    ) -> Vec<StandardEffectTriangleRenderPrimitive> {
        self.standard_local_effect_draw_plans
            .iter()
            .flat_map(StandardEffectDrawPlan::triangle_render_primitives_from_seed)
            .collect()
    }

    pub fn standard_effect_render_frame(&self) -> DesktopStandardEffectRenderFrame {
        DesktopStandardEffectRenderFrame {
            draw_plans: self.standard_local_effect_draw_plans.clone(),
            circle_primitives: self.standard_local_effect_circle_primitives.clone(),
            square_primitives: self.standard_local_effect_square_primitives.clone(),
            rect_primitives: self.standard_local_effect_rect_primitives.clone(),
            line_primitives: self.standard_local_effect_line_primitives.clone(),
            triangle_primitives: self.standard_local_effect_triangle_primitives.clone(),
            light_primitives: self.standard_local_effect_light_primitives.clone(),
        }
    }

    pub fn render_standard_effect_frame_with<R>(&self, renderer: &mut R) -> DesktopEffectRenderStats
    where
        R: DesktopEffectRenderer,
    {
        let frame = self.standard_effect_render_frame();
        renderer.render_standard_effect_frame(&frame)
    }

    pub fn drain_local_effect_events_for_render(&mut self) -> Vec<EffectCallPacket2> {
        std::mem::take(&mut self.runtime.client_local_effect_events)
    }

    pub fn sync_local_sound_at_events_for_audio(&mut self) -> usize {
        let events = std::mem::take(&mut self.runtime.client_local_sound_at_events);
        let count = events.len();
        self.pending_sound_at_events.extend(events);
        count
    }

    pub fn sound_at_audio_frame(&self) -> DesktopSoundAtAudioFrame {
        DesktopSoundAtAudioFrame {
            sound_at_events: self.pending_sound_at_events.clone(),
        }
    }

    pub fn play_sound_at_audio_frame_with<R>(&self, renderer: &mut R) -> DesktopSoundAudioStats
    where
        R: DesktopAudioRenderer,
    {
        let frame = self.sound_at_audio_frame();
        renderer.play_sound_at_audio_frame(&frame)
    }

    pub fn drain_and_play_sound_at_audio_frame_with<R>(
        &mut self,
        renderer: &mut R,
    ) -> DesktopSoundAudioStats
    where
        R: DesktopAudioRenderer,
    {
        let frame = DesktopSoundAtAudioFrame {
            sound_at_events: self.drain_sound_at_events_for_audio(),
        };
        renderer.play_sound_at_audio_frame(&frame)
    }

    pub fn drain_sound_at_events_for_audio(&mut self) -> Vec<SoundAtCallPacket> {
        std::mem::take(&mut self.pending_sound_at_events)
    }

    pub fn sync_local_camera_shake_events_for_render(
        &mut self,
        camera_x: f32,
        camera_y: f32,
    ) -> usize {
        let events = std::mem::take(&mut self.runtime.client_local_camera_shake_events);
        let count = events.len();
        for event in events {
            let resolved_intensity =
                shake_intensity(event.intensity, camera_x, camera_y, event.x, event.y);
            self.camera_shake_state
                .apply(resolved_intensity, event.duration);
            self.pending_camera_shake_events.push(event);
        }
        count
    }

    pub fn tick_camera_shake_for_render(
        &mut self,
        delta: f32,
        screen_shake_setting: i32,
    ) -> DesktopCameraShakeFrame {
        let frame = self.camera_shake_state.tick(delta, screen_shake_setting);
        self.last_camera_shake_frame = frame;
        frame
    }

    pub fn apply_camera_shake_frame_with<R>(
        &self,
        renderer: &mut R,
    ) -> DesktopCameraShakeRenderStats
    where
        R: DesktopCameraShakeRenderer,
    {
        renderer.apply_camera_shake_frame(&self.last_camera_shake_frame)
    }

    pub fn drain_camera_shake_events_for_render(
        &mut self,
    ) -> Vec<GameRuntimeClientCameraShakeEvent> {
        std::mem::take(&mut self.pending_camera_shake_events)
    }

    pub fn current_render_world_size(&self) -> RenderSize {
        RenderSize::new(
            self.game_state.world.unit_width().max(0) as f32,
            self.game_state.world.unit_height().max(0) as f32,
        )
    }

    pub fn current_minimap_world_size(&self) -> MinimapWorldSize {
        MinimapWorldSize::new(
            self.game_state.world.width().min(i32::MAX as usize) as i32,
            self.game_state.world.height().min(i32::MAX as usize) as i32,
        )
    }

    pub fn default_render_viewport(&self) -> RenderViewport {
        let size = self.current_render_world_size();
        RenderViewport::new(0.0, 0.0, size.width, size.height)
    }

    pub fn default_render_camera(&self) -> RenderCamera {
        let viewport = self.default_render_viewport();
        RenderCamera::new(
            RenderPoint::new(viewport.width / 2.0, viewport.height / 2.0),
            viewport,
        )
    }

    pub fn default_minimap_camera(&self) -> MinimapCamera {
        let viewport = self.default_render_viewport();
        MinimapCamera::new(
            viewport.width / 2.0,
            viewport.height / 2.0,
            viewport.width,
            viewport.height,
        )
    }

    pub fn default_minimap_overlay_input(&self) -> MinimapOverlayInput {
        let viewport = self.default_render_viewport();
        MinimapOverlayInput {
            screen_x: viewport.x,
            screen_y: viewport.y,
            screen_width: viewport.width,
            screen_height: viewport.height,
            full_view: true,
            mobile: false,
            net_active: self.net_client.state().lock().unwrap().connected,
            show_pings: false,
            fog: false,
            static_fog: false,
            dynamic_color: 0x000000ff,
            dynamic_alpha: 0.0,
            show_spawns: false,
            has_spawns: false,
            waves: self.game_state.rules.waves,
            wave_team_color: 0xffffffff,
            drop_zone_radius: self.game_state.rules.drop_zone_radius,
            time: self.game_state.tick as f32,
            global_time: self.game_state.tick as f32,
            units: Vec::new(),
            players: Vec::new(),
            spawns: Vec::new(),
            indicators: Vec::new(),
            markers: Vec::new(),
        }
    }

    pub fn render_frame_plan(
        &self,
        frame_index: u64,
        mut camera: RenderCamera,
        viewport: RenderViewport,
    ) -> RenderFramePlan {
        camera.viewport = viewport;
        let mut state = RenderEngineState::new(self.current_render_world_size(), camera);
        state.set_viewport(viewport);
        state.begin_frame(frame_index);
        state.finish()
    }

    pub fn minimap_overlay_plan(
        &mut self,
        camera: MinimapCamera,
        input: MinimapOverlayInput,
    ) -> MinimapOverlayPlan {
        let world = self.current_minimap_world_size();
        if world.width <= 0 || world.height <= 0 {
            return MinimapOverlayPlan {
                world_rect: MinimapRect::new(0.0, 0.0, 0.0, 0.0),
                scale_factor: 0.0,
                commands: Vec::new(),
            };
        }

        if self.minimap_renderer_state.world != world {
            self.minimap_renderer_state.reset(world);
        }

        self.minimap_renderer_state.overlay_plan(camera, input)
    }

    pub fn minimap_texture_frame_plan(&mut self) -> Option<MinimapTextureFramePlan> {
        let world = self.current_minimap_world_size();
        if world.width <= 0 || world.height <= 0 {
            return None;
        }
        if self.minimap_renderer_state.world != world {
            return Some(
                self.minimap_renderer_state
                    .reset(world)
                    .texture_frame_plan(),
            );
        }

        self.minimap_renderer_state.texture_frame_plan()
    }

    pub fn drain_overlay_renderer_plan(&mut self) -> OverlayRendererPlan {
        self.overlay_renderer_state.drain_plan()
    }

    pub fn drain_light_renderer_plan(&mut self) -> LightRendererPlan {
        self.light_renderer_state.drain_plan()
    }

    pub fn block_render_plan(
        &mut self,
        mut camera: RenderCamera,
        viewport: RenderViewport,
    ) -> Option<BlockRendererPlan> {
        let world = self.current_minimap_world_size();
        if world.width <= 0 || world.height <= 0 {
            return None;
        }

        let bounds = TileBounds::new(0, 0, world.width, world.height);
        if self.block_renderer_state.cache.block_tree.bounds != bounds {
            self.block_renderer_state =
                BlockRendererState::reload(bounds, self.game_state.rules.limit_map_area);
        }

        camera.viewport = viewport;
        let visible_tiles = visible_block_tiles(camera, viewport, world, 8.0);
        let snapshot = self.block_renderer_world_snapshot(&visible_tiles);
        self.block_renderer_state.cache.tile_view = visible_tiles;

        let plan = self
            .block_renderer_state
            .build_plan_from_snapshot(&snapshot);
        if plan.is_empty() {
            None
        } else {
            Some(plan)
        }
    }

    pub fn floor_render_plan(
        &mut self,
        mut camera: RenderCamera,
        viewport: RenderViewport,
    ) -> Option<FloorRenderPlan> {
        let world = self.current_minimap_world_size();
        if world.width <= 0 || world.height <= 0 {
            return None;
        }

        self.floor_renderer_state
            .set_world_tiles(world.width, world.height);
        camera.viewport = viewport;
        let world_rect = camera.world_rect();
        Some(self.floor_renderer_state.build_plan(FloorViewport::new(
            world_rect.center().x,
            world_rect.center().y,
            world_rect.width,
            world_rect.height,
        )))
    }

    pub fn floor_chunk_draw_batches(
        &mut self,
        mut camera: RenderCamera,
        viewport: RenderViewport,
    ) -> Vec<FloorChunkDrawBatch> {
        let world = self.current_minimap_world_size();
        if world.width <= 0 || world.height <= 0 {
            return Vec::new();
        }

        self.floor_renderer_state
            .set_world_tiles(world.width, world.height);
        camera.viewport = viewport;
        let world_rect = camera.world_rect();
        self.floor_renderer_state
            .build_chunk_draw_batches(FloorViewport::new(
                world_rect.center().x,
                world_rect.center().y,
                world_rect.width,
                world_rect.height,
            ))
    }

    pub fn fog_frame_plan(
        &mut self,
        mut camera: RenderCamera,
        viewport: RenderViewport,
    ) -> Option<FogFramePlan> {
        let world = self.current_minimap_world_size();
        if world.width <= 0 || world.height <= 0 || !self.game_state.rules.fog {
            return None;
        }

        let team = self.player.team.0;
        let discovered = self.game_state.fog_control.get_discovered(team)?;
        let discovered_tiles = (0..discovered.len())
            .map(|index| discovered.get(index))
            .collect::<Vec<_>>();

        camera.viewport = viewport;
        let world_rect = camera.world_rect();
        let mut input = FogFrameInput::new(
            FogViewport::new(
                world.width,
                world.height,
                8,
                world_rect.x,
                world_rect.y,
                world_rect.width,
                world_rect.height,
            ),
            team as u32,
            true,
            self.game_state.rules.static_fog,
            FogColor::WHITE,
            FogColor::BLACK,
        );
        input.discovered_tiles = Some(discovered_tiles);

        self.fog_renderer_state.draw_fog_plan(input)
    }

    pub fn pixelator_frame_plan(
        &mut self,
        mut camera: RenderCamera,
        viewport: RenderViewport,
    ) -> Option<PixelatorFramePlan> {
        camera.viewport = viewport;
        let world_rect = camera.world_rect();
        self.pixelator_state.draw_pixelate_plan(PixelatorInput::new(
            self.pixelate,
            self.renderer_scale,
            self.land_scale,
            self.cutscene,
            viewport.width.max(0.0).round() as i32,
            viewport.height.max(0.0).round() as i32,
            PixelatorCamera::new(
                camera.center.x,
                camera.center.y,
                world_rect.width,
                world_rect.height,
            ),
        ))
    }

    pub fn shader_dispatch_frame_plan(
        &self,
        camera: RenderCamera,
        viewport: RenderViewport,
    ) -> ShaderDispatchFrame {
        let mut context = ShaderApplyContext::default();
        context.camera = Some(ShaderCamera::new(
            camera.center.x,
            camera.center.y,
            viewport.width,
            viewport.height,
        ));
        context.graphics = Some(ShaderViewport::new(viewport.width, viewport.height));
        context.time = self.game_state.tick as f32;
        context.global_time = self.game_state.update_id as f32;

        ShaderDispatchFrame::from_applies([
            ShaderCatalog::apply_plan(ShaderId::Light, &context),
            ShaderCatalog::apply_plan(ShaderId::Shockwave, &context),
        ])
    }

    pub fn menu_frame_for_render(&mut self, input: MenuFrameInput) -> DesktopFrame {
        let plan = self.menu_renderer_state.render_plan(input);
        DesktopFrame {
            kind: DesktopFrameKind::Menu,
            payload: DesktopFramePayload::Menu(plan),
        }
    }

    fn block_renderer_world_snapshot(
        &self,
        visible_tiles: &[TileCoord],
    ) -> BlockRendererWorldSnapshot {
        let runtime_buildings = self
            .runtime
            .buildings
            .iter()
            .map(|building| (building.tile_pos, building))
            .collect::<BTreeMap<_, _>>();
        let mut tiles = Vec::with_capacity(visible_tiles.len());

        for coord in visible_tiles {
            let tile_snapshot = self
                .game_state
                .world
                .tile(coord.x, coord.y)
                .map(|tile| {
                    let runtime_building = tile
                        .build
                        .and_then(|build| runtime_buildings.get(&build.tile_pos).copied())
                        .or_else(|| runtime_buildings.get(&tile.pos()).copied());
                    let visual_runtime = runtime_building.map(|building| {
                        self.runtime
                            .block_visual_runtime_snapshot_for_building(building)
                    });
                    block_renderer_tile_snapshot_from_world(
                        tile,
                        runtime_building,
                        visual_runtime,
                        &self.content_loader,
                    )
                })
                .unwrap_or_else(|| BlockRendererTileSnapshot::new(*coord, ""));
            tiles.push(tile_snapshot);
        }

        BlockRendererWorldSnapshot::new(tiles)
    }

    pub fn load_frame_for_render(&mut self, input: LoadFrameInput) -> DesktopFrame {
        let plan = self.load_renderer_state.build_plan(input);
        DesktopFrame {
            kind: DesktopFrameKind::Load,
            payload: DesktopFramePayload::Load(plan),
        }
    }

    pub fn graphics_frame_for_render(
        &mut self,
        frame_index: u64,
        camera: RenderCamera,
        viewport: RenderViewport,
        minimap_camera: MinimapCamera,
        minimap_input: MinimapOverlayInput,
    ) -> DesktopGraphicsFrame {
        let mut render_frame = self.render_frame_plan(frame_index, camera, viewport);
        if let Some(light_pass) = self.drain_light_renderer_plan().to_render_pass() {
            render_frame.push_pass(light_pass);
        }
        let block_renderer = self.block_render_plan(camera, viewport);
        if let Some(block_renderer) = &block_renderer {
            for pass in block_renderer.to_sprite_render_passes(8.0) {
                render_frame.push_pass(pass);
            }
            if let Some(pass) = block_renderer.to_block_particle_render_pass(8.0) {
                render_frame.push_pass(pass);
            }
            for pass in block_renderer.to_resolve_render_passes(8.0) {
                render_frame.push_pass(pass);
            }
        }
        let floor_renderer = self.floor_render_plan(camera, viewport);
        if let Some(floor_renderer) = &floor_renderer {
            for pass in floor_renderer.cache_layer_passes.iter().cloned() {
                render_frame.push_pass(pass);
            }
        }
        let fog_frame = self.fog_frame_plan(camera, viewport);
        if let Some(fog_frame) = &fog_frame {
            for pass in fog_frame.to_render_passes() {
                render_frame.push_pass(pass);
            }
        }
        render_frame.sort_passes_like_java_renderer_draw();
        let floor_chunk_batches = self.floor_chunk_draw_batches(camera, viewport);
        let pixelator = self.pixelator_frame_plan(camera, viewport);
        let shader_dispatch = self.shader_dispatch_frame_plan(camera, viewport);
        let overlay_renderer = self.drain_overlay_renderer_plan();
        let minimap_texture_frame = self.minimap_texture_frame_plan();
        let minimap_overlay = self.minimap_overlay_plan(minimap_camera, minimap_input);

        let mut bridge = RenderBridge::new();
        bridge
            .set_render_frame(render_frame)
            .set_shader_dispatch(shader_dispatch)
            .set_overlay_renderer(overlay_renderer)
            .set_minimap_overlay(minimap_overlay);
        if let Some(block_renderer) = block_renderer {
            bridge.set_block_renderer(block_renderer);
        }
        if let Some(floor_renderer) = floor_renderer {
            bridge.set_floor_renderer(floor_renderer);
        }
        if let Some(fog_frame) = fog_frame {
            bridge.set_fog_frame(fog_frame);
        }
        if let Some(pixelator) = pixelator {
            bridge.set_pixelator(pixelator);
        }

        DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches,
            minimap_texture_frame,
            texture_atlas: self.texture_atlas.clone(),
        }
    }

    pub fn render_graphics_frame_with<R>(
        &mut self,
        frame_index: u64,
        camera: RenderCamera,
        viewport: RenderViewport,
        minimap_camera: MinimapCamera,
        minimap_input: MinimapOverlayInput,
        renderer: &mut R,
    ) -> GraphicsFrameStats
    where
        R: DesktopGraphicsRenderer,
    {
        let frame = self.graphics_frame_for_render(
            frame_index,
            camera,
            viewport,
            minimap_camera,
            minimap_input,
        );
        renderer.render_graphics_frame(&frame)
    }

    pub fn render_default_graphics_frame_with<R>(
        &mut self,
        frame_index: u64,
        renderer: &mut R,
    ) -> GraphicsFrameStats
    where
        R: DesktopGraphicsRenderer,
    {
        let viewport = self.default_render_viewport();
        let camera = self.default_render_camera();
        let minimap_camera = self.default_minimap_camera();
        let minimap_input = self.default_minimap_overlay_input();
        self.render_graphics_frame_with(
            frame_index,
            camera,
            viewport,
            minimap_camera,
            minimap_input,
            renderer,
        )
    }

    pub fn step_desktop_frame_loop<R, E>(
        &mut self,
        loop_state: &mut DesktopFrameLoopState,
        events: &[DesktopFrameLoopEvent],
        graphics_renderer: &mut R,
        effect_renderer: &mut E,
    ) -> DesktopPresentResult
    where
        R: DesktopGraphicsRenderer,
        E: DesktopEffectRenderer,
    {
        let frame_index = loop_state.next_frame_index;
        if loop_state.closed {
            return DesktopPresentResult {
                frame_index,
                surface: loop_state.surface.clone(),
                presented: false,
                skip_reason: Some(DesktopFrameSkipReason::AlreadyClosed),
                close_requested: true,
                resized_to: None,
                input_events: Vec::new(),
                graphics_stats: None,
                effect_stats: None,
            };
        }

        let mut close_requested = false;
        let mut resized_to = None;
        let mut input_events = Vec::new();
        for event in events {
            match event {
                DesktopFrameLoopEvent::Tick => {}
                DesktopFrameLoopEvent::Resize(size) => {
                    loop_state.surface.size = *size;
                    resized_to = Some(*size);
                }
                DesktopFrameLoopEvent::Input(input) => input_events.push(input.clone()),
                DesktopFrameLoopEvent::CloseRequested => close_requested = true,
            }
        }
        loop_state.input_events_seen = loop_state
            .input_events_seen
            .saturating_add(input_events.len() as u64);

        if close_requested {
            loop_state.request_close();
            return DesktopPresentResult {
                frame_index,
                surface: loop_state.surface.clone(),
                presented: false,
                skip_reason: Some(DesktopFrameSkipReason::CloseRequested),
                close_requested: true,
                resized_to,
                input_events,
                graphics_stats: None,
                effect_stats: None,
            };
        }

        self.update();
        let graphics_stats =
            self.render_default_graphics_frame_with(frame_index, graphics_renderer);
        let effect_stats = self.render_standard_effect_frame_with(effect_renderer);
        loop_state.next_frame_index = loop_state.next_frame_index.wrapping_add(1);

        DesktopPresentResult {
            frame_index,
            surface: loop_state.surface.clone(),
            presented: true,
            skip_reason: None,
            close_requested: false,
            resized_to,
            input_events,
            graphics_stats: Some(graphics_stats),
            effect_stats: Some(effect_stats),
        }
    }

    pub fn run_with_desktop_frame_loop<R, E, P, A, S>(
        &mut self,
        loop_state: &mut DesktopFrameLoopState,
        graphics_renderer: &mut R,
        effect_renderer: &mut E,
        max_presented_frames: Option<u64>,
        mut poll_events: P,
        mut after_present: A,
        mut sleep_frame: S,
    ) -> DesktopFrameLoopRunSummary
    where
        R: DesktopGraphicsRenderer,
        E: DesktopEffectRenderer,
        P: FnMut(&DesktopFrameLoopState) -> Vec<DesktopFrameLoopEvent>,
        A: FnMut(&DesktopPresentResult),
        S: FnMut(Duration),
    {
        let mut steps = 0;
        let mut frames_presented = 0;
        let mut last_frame_index = None;

        loop {
            if loop_state.closed {
                return DesktopFrameLoopRunSummary {
                    steps,
                    frames_presented,
                    last_frame_index,
                    exit_reason: DesktopFrameLoopExitReason::Closed,
                };
            }
            if max_presented_frames.is_some_and(|limit| frames_presented >= limit) {
                return DesktopFrameLoopRunSummary {
                    steps,
                    frames_presented,
                    last_frame_index,
                    exit_reason: DesktopFrameLoopExitReason::FrameLimit,
                };
            }

            let events = poll_events(loop_state);
            let result = self.step_desktop_frame_loop(
                loop_state,
                &events,
                graphics_renderer,
                effect_renderer,
            );
            steps += 1;
            if result.presented {
                frames_presented += 1;
                last_frame_index = Some(result.frame_index);
            }
            let should_stop = result.should_stop();
            after_present(&result);

            if should_stop || loop_state.closed {
                return DesktopFrameLoopRunSummary {
                    steps,
                    frames_presented,
                    last_frame_index,
                    exit_reason: DesktopFrameLoopExitReason::Closed,
                };
            }
            if max_presented_frames.is_some_and(|limit| frames_presented >= limit) {
                return DesktopFrameLoopRunSummary {
                    steps,
                    frames_presented,
                    last_frame_index,
                    exit_reason: DesktopFrameLoopExitReason::FrameLimit,
                };
            }
            if loop_state.pacing.is_paced() {
                sleep_frame(loop_state.pacing.target_frame_time);
            }
        }
    }

    pub fn playable_smoke_status(&self) -> DesktopPlayableSmokeStatus {
        let (connected, confirmed) = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            (state.connected, state.connect_confirm_sent)
        };
        let default_team = TeamId(self.game_state.rules.default_team as u8);

        DesktopPlayableSmokeStatus {
            client_ready: self.client.is_ready_for_play(),
            connected,
            world_loaded: self.last_applied_world_data.is_some(),
            confirmed,
            game_playing: self.game_state.is_playing(),
            runtime_client: self.runtime.network_context == GameRuntimeNetworkContext::client(),
            world_width: self.runtime.state.world.width(),
            world_height: self.runtime.state.world.height(),
            buildings: self.runtime.buildings().len(),
            player_on_default_team: self.player.team == default_team,
        }
    }

    pub fn playable_smoke_ready(&self) -> bool {
        self.playable_smoke_status().ready()
    }

    pub fn connect_from_args(&mut self) {
        let Some(target) = self.connect_target.clone() else {
            return;
        };

        self.net_client
            .set_connect_config(Some(ClientConnectConfig::default()));
        self.net_client.begin_connecting();
        let result = {
            let mut net = self.net_client.net_mut();
            net.connect(&target.host, target.port, Box::new(|| {}))
        };
        match result {
            Ok(()) => self.connect_error = None,
            Err(error) => self.connect_error = Some(error.to_string()),
        }
    }

    fn sync_loaded_world_data(&mut self) {
        let loaded_world_data = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            if state.last_world_data_error.is_some() {
                None
            } else {
                state.last_loaded_world_data.clone()
            }
        };

        match loaded_world_data.as_ref() {
            Some(world_data) => {
                if loaded_world_data.as_ref() == self.last_applied_world_data.as_ref() {
                    return;
                }
                self.apply_network_content_header(world_data.content_header_snapshot.as_ref());
                self.game_state.apply_network_world_data(world_data);
                self.apply_network_player_data(world_data.player_id, world_data.player.as_ref());
                self.apply_network_team_blocks(world_data.team_blocks_snapshot.as_ref());
                self.sync_runtime_state_from_world_data(world_data);
                self.last_applied_state_snapshot = None;
                self.reset_snapshot_apply_cursors_to_current_net_state();
            }
            None => {
                if self.last_applied_world_data.is_some() {
                    self.game_state = GameState::new();
                    self.runtime = GameRuntime::default();
                    self.runtime
                        .set_network_context(GameRuntimeNetworkContext::offline());
                    self.player = PlayerComp::default();
                    self.remote_players.clear();
                    self.other_player_preview_overlays.clear();
                    self.standard_local_effect_draw_plans.clear();
                    self.standard_local_effect_circle_primitives.clear();
                    self.standard_local_effect_square_primitives.clear();
                    self.standard_local_effect_rect_primitives.clear();
                    self.standard_local_effect_line_primitives.clear();
                    self.standard_local_effect_triangle_primitives.clear();
                    self.standard_local_effect_light_primitives.clear();
                    self.content_loader.clear_temporary_mapper();
                    self.last_applied_state_snapshot = None;
                    self.last_runtime_map_load_report = None;
                    self.clear_snapshot_apply_cursors();
                }
            }
        }
        self.last_applied_world_data = loaded_world_data;
    }

    fn sync_snapshot_mirrors(&mut self) {
        if self.last_applied_world_data.is_none() {
            return;
        }

        let (block_mirror, entity_mirrors, hidden_mirror) = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            (
                state.last_block_snapshot_mirror.clone(),
                state.entity_snapshot_mirrors.clone(),
                state.last_hidden_snapshot_mirror.clone(),
            )
        };

        let mut report = GameRuntimeClientSnapshotApplyReport::default();

        if block_mirror != self.last_applied_block_snapshot_mirror {
            if let Some(mirror) = block_mirror.as_ref() {
                if mirror.parse_error.is_some() {
                    report.merge(self.runtime.note_client_block_snapshot_parse_error());
                }
                for record in &mirror.records {
                    report.merge(
                        self.runtime
                            .apply_client_block_snapshot_record_with_content(
                                &self.content_loader,
                                record.tile_pos,
                                record.block_id,
                                record.sync_bytes.clone(),
                            ),
                    );
                }
            }
            self.last_applied_block_snapshot_mirror = block_mirror;
        }

        if entity_mirrors.len() < self.last_applied_entity_snapshot_mirror_count {
            self.last_applied_entity_snapshot_mirror_count = 0;
        }
        for mirror in entity_mirrors
            .iter()
            .skip(self.last_applied_entity_snapshot_mirror_count)
        {
            if mirror.parse_error.is_some() {
                let mixed_fallback =
                    self.apply_client_entity_snapshot_packet_mixed(mirror.amount, &mirror.data);
                if mixed_fallback.entity_records_applied > 0
                    || mixed_fallback.entity_typed_records_applied > 0
                {
                    report.merge(mixed_fallback);
                } else {
                    let fallback = self
                        .runtime
                        .apply_client_entity_snapshot_packet_with_content(
                            &self.content_loader,
                            mirror.amount,
                            &mirror.data,
                        );
                    if fallback.entity_records_applied > 0
                        || fallback.entity_typed_records_applied > 0
                    {
                        report.merge(fallback);
                    } else {
                        report.merge(self.runtime.note_client_entity_snapshot_parse_error());
                    }
                }
                continue;
            }
            for record in &mirror.records {
                let record_report = self
                    .runtime
                    .apply_client_entity_snapshot_record_with_content(
                        &self.content_loader,
                        record.entity_id,
                        record.type_id,
                        record.sync_bytes.clone(),
                    );
                let runtime_typed_applied = record_report.entity_typed_records_applied > 0;
                report.merge(record_report);
                if record.type_id == PLAYER_CLASS_ID
                    && self
                        .apply_client_player_entity_snapshot(record.entity_id, &record.sync_bytes)
                    && !runtime_typed_applied
                {
                    report.entity_typed_records_applied += 1;
                }
                if !runtime_typed_applied
                    && entity_class_kind(record.type_id) == Some(EntityClassKind::Fire)
                    && self.apply_client_fire_entity_snapshot(record.entity_id, &record.sync_bytes)
                {
                    report.entity_typed_records_applied += 1;
                }
            }
        }
        self.last_applied_entity_snapshot_mirror_count = entity_mirrors.len();

        if hidden_mirror != self.last_applied_hidden_snapshot_mirror {
            if let Some(mirror) = hidden_mirror.as_ref() {
                report.merge(self.runtime.apply_client_hidden_snapshot_ids(&mirror.ids));
            }
            self.last_applied_hidden_snapshot_mirror = hidden_mirror;
        }

        if report.has_activity() {
            self.last_client_snapshot_apply_report = Some(report);
        }
    }

    fn sync_building_storage_mirrors_to_runtime(&mut self) -> usize {
        if self.last_applied_world_data.is_none() {
            return 0;
        }

        let mirrors = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            state.building_storage_mirrors.clone()
        };
        self.last_applied_building_storage_mirrors
            .retain(|tile_pos, _| mirrors.contains_key(tile_pos));

        let mut applied = 0;
        for (tile_pos, mirror) in mirrors {
            if self.last_applied_building_storage_mirrors.get(&tile_pos) == Some(&mirror) {
                continue;
            }
            if self.runtime.apply_client_building_item_storage_mirror(
                &self.content_loader,
                tile_pos,
                &mirror.items,
            ) {
                self.last_applied_building_storage_mirrors
                    .insert(tile_pos, mirror);
                applied += 1;
            }
        }
        applied
    }

    fn sync_unit_item_mirrors_to_runtime(&mut self) -> usize {
        if self.last_applied_world_data.is_none() {
            return 0;
        }

        let mirrors = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            state.unit_item_mirrors.clone()
        };
        self.last_applied_unit_item_mirrors
            .retain(|unit_id, _| mirrors.contains_key(unit_id));

        let mut applied = 0;
        for (unit_id, mirror) in mirrors {
            if self.last_applied_unit_item_mirrors.get(&unit_id) == Some(&mirror) {
                continue;
            }
            if self.runtime.apply_client_unit_item_mirror(
                unit_id,
                mirror.item.as_deref(),
                mirror.amount,
            ) {
                self.last_applied_unit_item_mirrors.insert(unit_id, mirror);
                applied += 1;
            }
        }
        applied
    }

    fn sync_unit_payload_mirrors_to_runtime(&mut self) -> usize {
        if self.last_applied_world_data.is_none() {
            return 0;
        }

        let mirrors = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            state.unit_payload_mirrors.clone()
        };
        self.last_applied_unit_payload_mirrors
            .retain(|unit_id, _| mirrors.contains_key(unit_id));

        let mut applied = 0;
        for (unit_id, mirror) in mirrors {
            if self.last_applied_unit_payload_mirrors.get(&unit_id) == Some(&mirror) {
                continue;
            }
            if self.runtime.apply_client_unit_payload_mirror(
                unit_id,
                mirror.payload_count,
                mirror.picked_build_payloads_seen,
                mirror.picked_unit_payloads_seen,
            ) {
                self.last_applied_unit_payload_mirrors
                    .insert(unit_id, mirror);
                applied += 1;
            }
        }
        applied
    }

    fn sync_unit_entered_payload_to_runtime(&mut self) -> bool {
        if self.last_applied_world_data.is_none() {
            return false;
        }

        let (seen, packet) = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            (
                state.unit_entered_payload_packets_seen,
                state.last_unit_entered_payload.clone(),
            )
        };
        if seen == self.last_applied_unit_entered_payload_packets_seen {
            return false;
        }
        self.last_applied_unit_entered_payload_packets_seen = seen;

        let Some(packet) = packet else {
            self.last_unit_entered_payload_apply_report = None;
            return false;
        };
        let report = self
            .runtime
            .apply_client_unit_entered_payload_packet(&self.content_loader, &packet);
        let applied = report.payload_attached;
        self.last_unit_entered_payload_apply_report = Some(report);
        applied
    }

    fn sync_unit_tether_block_spawned_to_runtime(&mut self) -> bool {
        if self.last_applied_world_data.is_none() {
            return false;
        }

        let (seen, packet) = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            (
                state.unit_tether_block_spawned_packets_seen,
                state.last_unit_tether_block_spawned.clone(),
            )
        };
        if seen == self.last_applied_unit_tether_block_spawned_packets_seen {
            return false;
        }
        self.last_applied_unit_tether_block_spawned_packets_seen = seen;

        let Some(packet) = packet else {
            return false;
        };
        self.runtime
            .apply_client_unit_tether_block_spawned_packet(&self.content_loader, &packet)
    }

    fn sync_unit_lifecycle_to_runtime(&mut self) -> bool {
        if self.last_applied_world_data.is_none() {
            return false;
        }

        let (seen, packets) = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            (
                state.unit_lifecycle_packets_seen,
                state.unit_lifecycle_packets.clone(),
            )
        };
        if seen == self.last_applied_unit_lifecycle_packets_seen {
            return false;
        }
        let start = self
            .last_applied_unit_lifecycle_packets_seen
            .min(packets.len() as u64) as usize;
        self.last_applied_unit_lifecycle_packets_seen = seen;

        let mut applied = false;
        for packet in packets.into_iter().skip(start) {
            applied |= match packet {
                PacketKind::UnitBlockSpawnCallPacket(packet) => self
                    .runtime
                    .apply_client_unit_block_spawn_packet(&self.content_loader, &packet),
                PacketKind::AssemblerUnitSpawnedCallPacket(packet) => self
                    .runtime
                    .apply_client_assembler_unit_spawned_packet(&self.content_loader, &packet),
                PacketKind::AssemblerDroneSpawnedCallPacket(packet) => self
                    .runtime
                    .apply_client_assembler_drone_spawned_packet(&self.content_loader, &packet),
                PacketKind::UnitDespawnCallPacket(packet) => {
                    self.runtime.apply_client_unit_despawn_packet(&packet)
                }
                PacketKind::UnitDestroyCallPacket(packet) => {
                    self.runtime.apply_client_unit_destroy_packet_with_content(
                        &self.content_loader,
                        Some(self.player.id),
                        &packet,
                    )
                }
                PacketKind::UnitDeathCallPacket(packet) => {
                    self.runtime.apply_client_unit_death_packet(&packet)
                }
                PacketKind::UnitSafeDeathCallPacket(packet) => {
                    self.runtime.apply_client_unit_safe_death_packet(&packet)
                }
                PacketKind::UnitCapDeathCallPacket(packet) => {
                    self.runtime.apply_client_unit_cap_death_packet(&packet)
                }
                PacketKind::UnitEnvDeathCallPacket(packet) => {
                    self.runtime.apply_client_unit_env_death_packet(&packet)
                }
                _ => false,
            };
        }
        applied
    }

    fn sync_unit_spawn_packets_to_runtime(&mut self) -> bool {
        if self.last_applied_world_data.is_none() {
            return false;
        }

        let packets = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            state.unit_spawn_packets.clone()
        };
        if packets.len() == self.last_applied_unit_spawn_packet_count {
            return false;
        }

        let start = self.last_applied_unit_spawn_packet_count.min(packets.len());
        self.last_applied_unit_spawn_packet_count = packets.len();
        let mut applied = false;
        for packet in packets.iter().skip(start) {
            applied |= self
                .runtime
                .apply_client_unit_spawn_packet(&self.content_loader, packet);
        }
        applied
    }

    fn sync_world_update_events_to_runtime(&mut self) -> bool {
        if self.last_applied_world_data.is_none() {
            return false;
        }

        let (seen, packet) = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            (
                state.world_update_packets_seen,
                state.last_world_update_packet.clone(),
            )
        };
        if seen == self.last_applied_world_update_packets_seen {
            return false;
        }
        self.last_applied_world_update_packets_seen = seen;

        let Some(packet) = packet else {
            return false;
        };
        match packet {
            PacketKind::LandingPadLandedCallPacket(packet) => self
                .runtime
                .apply_client_landing_pad_landed_packet(&self.content_loader, &packet),
            _ => false,
        }
    }

    fn sync_runtime_trigger_events_to_service(&mut self) -> bool {
        let events = self.runtime.drain_trigger_events();
        if events.is_empty() {
            return false;
        }

        let mut total = GameServiceApplySummary::default();
        for event in events {
            let plan = self
                .client
                .service
                .state()
                .trigger_plan(GameServiceTriggerSnapshot {
                    trigger: event.trigger,
                    campaign: event.campaign,
                });
            let summary = plan.apply_to(
                &mut self.client.service,
                &mut self.client.achievement_state,
                AchievementContext::normal(),
            );
            total.stat_additions += summary.stat_additions;
            total.stat_amount_additions += summary.stat_amount_additions;
            total.stat_sets += summary.stat_sets;
            total.stat_max_updates += summary.stat_max_updates;
            total.achievements_completed += summary.achievements_completed;
        }

        let changed = total.stat_additions > 0
            || total.stat_amount_additions > 0
            || total.stat_sets > 0
            || total.stat_max_updates > 0
            || total.achievements_completed > 0;
        self.last_service_trigger_apply_summary = Some(total);
        changed
    }

    fn sync_tile_config_to_runtime(&mut self) -> bool {
        if self.last_applied_world_data.is_none() {
            return false;
        }

        let (seen, packet) = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            (
                state.tile_config_packets_seen,
                state.last_tile_config.clone(),
            )
        };
        if seen == self.last_applied_tile_config_packets_seen {
            return false;
        }
        self.last_applied_tile_config_packets_seen = seen;

        let Some(packet) = packet else {
            self.last_tile_config_apply_result = None;
            self.last_unit_factory_tile_config_apply_result = None;
            self.last_reconstructor_tile_config_apply_result = None;
            return false;
        };
        let Some(tile_pos) = packet.build.tile_pos else {
            self.last_tile_config_apply_result =
                Some(GameRuntimeUnitCargoUnloadConfigureResult::MissingBuilding);
            self.last_unit_factory_tile_config_apply_result = None;
            self.last_reconstructor_tile_config_apply_result = None;
            return false;
        };

        let target_block = self
            .runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == tile_pos)
            .and_then(|building| self.content_loader.block(building.block.id));
        let is_unit_cargo_unload = target_block.is_some_and(|block| {
            matches!(
                block,
                BlockDef::Distribution(distribution)
                    if distribution.kind == DistributionBlockKind::UnitCargoUnloadPoint
            )
        });
        if is_unit_cargo_unload {
            let result = self.runtime.configure_owned_unit_cargo_unload_value(
                &self.content_loader,
                tile_pos,
                &packet.value,
            );
            let changed = result.changed();
            self.last_tile_config_apply_result = Some(result);
            self.last_unit_factory_tile_config_apply_result = None;
            self.last_reconstructor_tile_config_apply_result = None;
            return changed;
        }

        let is_unit_factory =
            target_block.is_some_and(|block| matches!(block, BlockDef::UnitFactory(_)));
        if is_unit_factory {
            let result = self.runtime.configure_owned_unit_factory_value(
                &self.content_loader,
                tile_pos,
                &packet.value,
            );
            let changed = result.changed();
            self.last_tile_config_apply_result = None;
            self.last_unit_factory_tile_config_apply_result = Some(result);
            self.last_reconstructor_tile_config_apply_result = None;
            return changed;
        }

        let is_reconstructor =
            target_block.is_some_and(|block| matches!(block, BlockDef::UnitReconstructor(_)));
        if is_reconstructor {
            let result = self.runtime.configure_owned_reconstructor_value(
                &self.content_loader,
                tile_pos,
                &packet.value,
            );
            let changed = result.changed();
            self.last_tile_config_apply_result = None;
            self.last_unit_factory_tile_config_apply_result = None;
            self.last_reconstructor_tile_config_apply_result = Some(result);
            return changed;
        }

        self.last_tile_config_apply_result = None;
        self.last_unit_factory_tile_config_apply_result = None;
        self.last_reconstructor_tile_config_apply_result = None;
        false
    }

    fn sync_command_building_to_runtime(&mut self) -> bool {
        if self.last_applied_world_data.is_none() {
            return false;
        }

        let (seen, packet) = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            (
                state.command_building_packets_seen,
                state.last_command_building.clone(),
            )
        };
        if seen == self.last_applied_command_building_packets_seen {
            return false;
        }
        self.last_applied_command_building_packets_seen = seen;

        let Some(packet) = packet else {
            self.last_command_building_apply_report = None;
            return false;
        };

        let player = packet.player.id.and_then(|id| {
            if id == self.player.id {
                Some(self.player.clone())
            } else {
                self.remote_players.get(&id).cloned()
            }
        });
        let team = player.as_ref().map(|player| player.team).or_else(|| {
            packet.buildings.iter().find_map(|tile_pos| {
                self.runtime
                    .buildings()
                    .iter()
                    .find(|building| building.tile_pos == *tile_pos)
                    .map(|building| building.team)
            })
        });
        let Some(team) = team else {
            self.last_command_building_apply_report = None;
            return false;
        };
        let last_accessed = player.as_ref().map(|player| player.colored_name());
        let report = self.runtime.command_owned_building_positions(
            &self.content_loader,
            team,
            &packet.buildings,
            packet.target,
            last_accessed,
        );
        let changed = report.changed();
        self.last_command_building_apply_report = Some(report);
        changed
    }

    fn sync_effect_packets_to_runtime(&mut self) -> usize {
        let (
            effect_seen,
            effect,
            effect_with_data_seen,
            effect_with_data,
            reliable_effect_seen,
            reliable_effect,
        ) = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            (
                state.effect_packets_seen,
                state.last_effect,
                state.effect_with_data_packets_seen,
                state.last_effect_with_data.clone(),
                state.reliable_effect_packets_seen,
                state.last_reliable_effect,
            )
        };

        let mut queued = 0;
        if effect_seen != self.last_applied_effect_packets_seen {
            self.last_applied_effect_packets_seen = effect_seen;
            if let Some(effect) = effect {
                self.runtime
                    .client_local_effect_events
                    .push(EffectCallPacket2 {
                        effect,
                        data: TypeValue::Null,
                    });
                queued += 1;
            }
        }
        if effect_with_data_seen != self.last_applied_effect_with_data_packets_seen {
            self.last_applied_effect_with_data_packets_seen = effect_with_data_seen;
            if let Some(effect) = effect_with_data {
                self.runtime.client_local_effect_events.push(effect);
                queued += 1;
            }
        }
        if reliable_effect_seen != self.last_applied_reliable_effect_packets_seen {
            self.last_applied_reliable_effect_packets_seen = reliable_effect_seen;
            if let Some(effect) = reliable_effect {
                self.runtime
                    .client_local_effect_events
                    .push(EffectCallPacket2 {
                        effect: effect.0,
                        data: TypeValue::Null,
                    });
                queued += 1;
            }
        }

        queued
    }

    fn reset_snapshot_apply_cursors_to_current_net_state(&mut self) {
        let (
            block_mirror,
            entity_count,
            hidden_mirror,
            building_storage_mirrors,
            preview_plan_count,
            unit_item_mirrors,
            unit_payload_mirrors,
            unit_entered_payload_packets_seen,
            unit_tether_block_spawned_packets_seen,
            unit_lifecycle_packets_seen,
            unit_spawn_packet_count,
            world_update_packets_seen,
            tile_config_packets_seen,
            command_building_packets_seen,
            effect_packets_seen,
            effect_with_data_packets_seen,
            reliable_effect_packets_seen,
        ) = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            (
                state.last_block_snapshot_mirror.clone(),
                state.entity_snapshot_mirrors.len(),
                state.last_hidden_snapshot_mirror.clone(),
                state.building_storage_mirrors.clone(),
                state.client_plan_snapshot_received_packets.len(),
                state.unit_item_mirrors.clone(),
                state.unit_payload_mirrors.clone(),
                state.unit_entered_payload_packets_seen,
                state.unit_tether_block_spawned_packets_seen,
                state.unit_lifecycle_packets_seen,
                state.unit_spawn_packets.len(),
                state.world_update_packets_seen,
                state.tile_config_packets_seen,
                state.command_building_packets_seen,
                state.effect_packets_seen,
                state.effect_with_data_packets_seen,
                state.reliable_effect_packets_seen,
            )
        };
        self.last_applied_block_snapshot_mirror = block_mirror;
        self.last_applied_entity_snapshot_mirror_count = entity_count;
        self.last_applied_hidden_snapshot_mirror = hidden_mirror;
        self.last_client_snapshot_apply_report = None;
        self.last_applied_client_plan_snapshot_received_count = preview_plan_count;
        self.last_applied_building_storage_mirrors = building_storage_mirrors;
        self.last_applied_unit_item_mirrors = unit_item_mirrors;
        self.last_applied_unit_payload_mirrors = unit_payload_mirrors;
        self.last_applied_unit_entered_payload_packets_seen = unit_entered_payload_packets_seen;
        self.last_applied_unit_tether_block_spawned_packets_seen =
            unit_tether_block_spawned_packets_seen;
        self.last_applied_unit_lifecycle_packets_seen = unit_lifecycle_packets_seen;
        self.last_applied_unit_spawn_packet_count = unit_spawn_packet_count;
        self.last_applied_world_update_packets_seen = world_update_packets_seen;
        self.last_applied_tile_config_packets_seen = tile_config_packets_seen;
        self.last_applied_command_building_packets_seen = command_building_packets_seen;
        self.last_applied_effect_packets_seen = effect_packets_seen;
        self.last_applied_effect_with_data_packets_seen = effect_with_data_packets_seen;
        self.last_applied_reliable_effect_packets_seen = reliable_effect_packets_seen;
        self.last_unit_entered_payload_apply_report = None;
        self.last_tile_config_apply_result = None;
        self.last_unit_factory_tile_config_apply_result = None;
        self.last_reconstructor_tile_config_apply_result = None;
        self.last_command_building_apply_report = None;
    }

    fn clear_snapshot_apply_cursors(&mut self) {
        self.last_applied_block_snapshot_mirror = None;
        self.last_applied_entity_snapshot_mirror_count = 0;
        self.last_applied_hidden_snapshot_mirror = None;
        self.last_client_snapshot_apply_report = None;
        self.last_applied_client_plan_snapshot_received_count = 0;
        self.last_applied_building_storage_mirrors.clear();
        self.last_applied_unit_item_mirrors.clear();
        self.last_applied_unit_payload_mirrors.clear();
        self.last_applied_unit_entered_payload_packets_seen = 0;
        self.last_applied_unit_tether_block_spawned_packets_seen = 0;
        self.last_applied_unit_lifecycle_packets_seen = 0;
        self.last_applied_world_update_packets_seen = 0;
        self.last_applied_tile_config_packets_seen = 0;
        self.last_applied_command_building_packets_seen = 0;
        self.last_applied_effect_packets_seen = 0;
        self.last_applied_effect_with_data_packets_seen = 0;
        self.last_applied_reliable_effect_packets_seen = 0;
        self.last_unit_entered_payload_apply_report = None;
        self.last_tile_config_apply_result = None;
        self.last_unit_factory_tile_config_apply_result = None;
        self.last_reconstructor_tile_config_apply_result = None;
        self.last_command_building_apply_report = None;
        self.remote_players.clear();
        self.other_player_preview_overlays.clear();
        self.standard_local_effect_draw_plans.clear();
        self.standard_local_effect_circle_primitives.clear();
        self.standard_local_effect_square_primitives.clear();
        self.standard_local_effect_rect_primitives.clear();
        self.standard_local_effect_line_primitives.clear();
        self.standard_local_effect_triangle_primitives.clear();
        self.standard_local_effect_light_primitives.clear();
        self.pending_sound_at_events.clear();
        self.pending_camera_shake_events.clear();
        self.camera_shake_state = DesktopCameraShakeState::default();
        self.last_camera_shake_frame = DesktopCameraShakeFrame::default();
        self.overlay_renderer_state = OverlayRendererState::default();
        self.block_renderer_state = BlockRendererState::default();
        self.light_renderer_state = LightRendererState::default();
        self.floor_renderer_state = FloorRendererState::default();
        self.fog_renderer_state = FogRendererState::default();
        self.minimap_renderer_state = MinimapRendererState::new(MinimapWorldSize::new(0, 0));
        self.menu_renderer_state = MenuRendererState::new(MenuRendererConfig::new(false, 7));
        self.load_renderer_state = LoadRendererState::default();
        self.pixelator_state = PixelatorState::default();
    }

    fn apply_client_player_entity_snapshot(&mut self, entity_id: i32, sync_bytes: &[u8]) -> bool {
        if entity_id != self.player.id {
            return false;
        }

        let Ok(player_sync) = NetworkPlayerSyncData::read_exact_from(sync_bytes) else {
            return false;
        };

        self.apply_client_player_sync_record(entity_id, player_sync)
    }

    fn apply_client_player_sync_record(
        &mut self,
        entity_id: i32,
        player_sync: NetworkPlayerSyncData,
    ) -> bool {
        if entity_id != self.player.id {
            return false;
        }

        self.player
            .apply_network_player_sync_data(&player_sync, true);
        self.player.after_sync_unit_state(PlayerUnitSwitchContext {
            is_local: true,
            headless: false,
            net_client: true,
        });
        self.runtime
            .apply_client_player_snapshot_record(entity_id, player_sync);
        true
    }

    fn sync_remote_player_snapshots_from_runtime(&mut self) -> usize {
        let hidden_ids: Vec<i32> = self
            .runtime
            .client_hidden_entity_ids
            .iter()
            .copied()
            .collect();
        for id in hidden_ids {
            self.remote_players.remove(&id);
        }

        let snapshots: Vec<_> = self
            .runtime
            .client_player_snapshot_entities
            .iter()
            .map(|(id, sync)| (*id, sync.clone()))
            .collect();
        let mut synced = 0;
        for (entity_id, sync) in snapshots {
            if entity_id == self.player.id {
                self.remote_players.remove(&entity_id);
                continue;
            }
            if self.runtime.client_hidden_entity_ids.contains(&entity_id) {
                self.remote_players.remove(&entity_id);
                continue;
            }

            let player = self.remote_players.entry(entity_id).or_insert_with(|| {
                let mut player = PlayerComp::new(sync.team);
                player.id = entity_id;
                player
            });
            player.id = entity_id;
            player.apply_network_player_sync_data(&sync, false);
            player.after_sync_unit_state(PlayerUnitSwitchContext {
                is_local: false,
                headless: false,
                net_client: true,
            });
            synced += 1;
        }
        synced
    }

    fn sync_remote_preview_plan_packets(&mut self, now_millis: i64) -> usize {
        let packets = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            state.client_plan_snapshot_received_packets.clone()
        };
        if packets.len() < self.last_applied_client_plan_snapshot_received_count {
            self.last_applied_client_plan_snapshot_received_count = 0;
        }

        let mut applied = 0;
        for packet in packets
            .iter()
            .skip(self.last_applied_client_plan_snapshot_received_count)
        {
            if packet.player_id == self.player.id {
                continue;
            }

            let player = self
                .remote_players
                .entry(packet.player_id)
                .or_insert_with(|| {
                    let mut player = PlayerComp::new(self.player.team);
                    player.id = packet.player_id;
                    player.name = format!("player-{}", packet.player_id);
                    player
                });
            if NetClient::apply_received_preview_plans_to_player(
                player,
                packet,
                now_millis,
                MAX_PLAYER_PREVIEW_PLANS,
            )
            .is_ok()
            {
                applied += 1;
            }
        }
        self.last_applied_client_plan_snapshot_received_count = packets.len();
        applied
    }

    pub fn rebuild_other_player_preview_overlays_at(
        &mut self,
        now_millis: i64,
        delta: f32,
        mouse_world: Option<Vec2>,
    ) -> usize {
        let frame = OtherPlayerPreviewOverlayFrame {
            local_player_id: self.player.id,
            local_team: self.player.team,
            now_millis,
            delta,
            mouse_world,
        };
        let content_loader = self.content_loader.clone();
        let mut overlays = Vec::new();
        for player in self.remote_players.values_mut() {
            let overlay = other_player_preview_overlay_plan(player, frame, |name| {
                content_loader
                    .block_by_name(name)
                    .map(|block| OtherPlayerPreviewBlock {
                        size: block.base().size,
                        offset: block.base().offset,
                    })
            });
            if !overlay.entries.is_empty() || overlay.overlap.is_some() {
                overlays.push(overlay);
            }
        }
        self.other_player_preview_overlays = overlays;
        self.other_player_preview_overlays.len()
    }

    fn apply_client_fire_entity_snapshot(&mut self, entity_id: i32, sync_bytes: &[u8]) -> bool {
        let mut read = sync_bytes;
        let Ok(fire_sync) = read_fire_sync(&mut read) else {
            return false;
        };
        if !read.is_empty() {
            return false;
        }
        self.runtime
            .apply_client_fire_sync_wire(entity_id, &fire_sync)
    }

    fn apply_client_entity_snapshot_packet_mixed(
        &mut self,
        amount: i16,
        data: &[u8],
    ) -> GameRuntimeClientSnapshotApplyReport {
        let Ok(amount) = usize::try_from(amount) else {
            return self.runtime.note_client_entity_snapshot_parse_error();
        };

        let mut report = GameRuntimeClientSnapshotApplyReport::default();
        let mut read = data;
        for _ in 0..amount {
            if read.len() < 5 {
                report.entity_parse_errors += 1;
                return report;
            }
            let entity_id = i32::from_be_bytes(read[0..4].try_into().unwrap());
            let type_id = read[4];
            read = &read[5..];

            let sync_start = read;
            let before_len = sync_start.len();
            if entity_id == self.player.id && type_id == PLAYER_CLASS_ID {
                let Ok(player_sync) = NetworkPlayerSyncData::read_from(&mut read) else {
                    report.entity_parse_errors += 1;
                    return report;
                };
                let consumed = before_len - read.len();
                let sync_bytes = sync_start[..consumed].to_vec();
                report.merge(
                    self.runtime
                        .apply_client_entity_snapshot_record(entity_id, type_id, sync_bytes),
                );
                if self.apply_client_player_sync_record(entity_id, player_sync) {
                    report.entity_typed_records_applied += 1;
                }
                continue;
            }

            match entity_class_kind(type_id) {
                Some(EntityClassKind::Player) => {
                    let Ok(player_sync) = NetworkPlayerSyncData::read_from(&mut read) else {
                        report.entity_parse_errors += 1;
                        return report;
                    };
                    let consumed = before_len - read.len();
                    let sync_bytes = sync_start[..consumed].to_vec();
                    report.merge(
                        self.runtime
                            .apply_client_entity_snapshot_record(entity_id, type_id, sync_bytes),
                    );
                    self.runtime
                        .apply_client_player_snapshot_record(entity_id, player_sync);
                    report.entity_typed_records_applied += 1;
                }
                Some(EntityClassKind::Bullet) => {
                    let Ok(bullet_sync) = read_bullet_sync(&mut read) else {
                        report.entity_parse_errors += 1;
                        return report;
                    };
                    let consumed = before_len - read.len();
                    let sync_bytes = sync_start[..consumed].to_vec();
                    report.merge(
                        self.runtime
                            .apply_client_entity_snapshot_record(entity_id, type_id, sync_bytes),
                    );
                    if self.runtime.apply_client_bullet_sync_wire(
                        &self.content_loader,
                        entity_id,
                        &bullet_sync,
                    ) {
                        report.entity_typed_records_applied += 1;
                    }
                }
                Some(EntityClassKind::Decal) => {
                    let Ok(decal_sync) = read_decal_sync(&mut read) else {
                        report.entity_parse_errors += 1;
                        return report;
                    };
                    let consumed = before_len - read.len();
                    let sync_bytes = sync_start[..consumed].to_vec();
                    report.merge(
                        self.runtime
                            .apply_client_entity_snapshot_record(entity_id, type_id, sync_bytes),
                    );
                    if self
                        .runtime
                        .apply_client_decal_sync_wire(entity_id, &decal_sync)
                    {
                        report.entity_typed_records_applied += 1;
                    }
                }
                Some(EntityClassKind::Effect) => {
                    let Ok(effect_sync) = read_effect_state_sync(&mut read) else {
                        report.entity_parse_errors += 1;
                        return report;
                    };
                    let consumed = before_len - read.len();
                    let sync_bytes = sync_start[..consumed].to_vec();
                    report.merge(
                        self.runtime
                            .apply_client_entity_snapshot_record(entity_id, type_id, sync_bytes),
                    );
                    if self
                        .runtime
                        .apply_client_effect_state_sync_wire(entity_id, &effect_sync)
                    {
                        report.entity_typed_records_applied += 1;
                    }
                }
                Some(EntityClassKind::Unit) => {
                    let Ok(_unit_sync) = read_unit_sync(&mut read, &self.content_loader) else {
                        report.entity_parse_errors += 1;
                        return report;
                    };
                    let consumed = before_len - read.len();
                    let sync_bytes = sync_start[..consumed].to_vec();
                    report.merge(
                        self.runtime
                            .apply_client_entity_snapshot_record_with_content(
                                &self.content_loader,
                                entity_id,
                                type_id,
                                sync_bytes,
                            ),
                    );
                }
                Some(EntityClassKind::Fire) => {
                    let Ok(fire_sync) = read_fire_sync(&mut read) else {
                        report.entity_parse_errors += 1;
                        return report;
                    };
                    let consumed = before_len - read.len();
                    let sync_bytes = sync_start[..consumed].to_vec();
                    report.merge(
                        self.runtime
                            .apply_client_entity_snapshot_record(entity_id, type_id, sync_bytes),
                    );
                    if self
                        .runtime
                        .apply_client_fire_sync_wire(entity_id, &fire_sync)
                    {
                        report.entity_typed_records_applied += 1;
                    }
                }
                Some(EntityClassKind::Puddle) => {
                    let Ok(puddle_sync) = read_puddle_sync(&mut read) else {
                        report.entity_parse_errors += 1;
                        return report;
                    };
                    let consumed = before_len - read.len();
                    let sync_bytes = sync_start[..consumed].to_vec();
                    report.merge(
                        self.runtime
                            .apply_client_entity_snapshot_record(entity_id, type_id, sync_bytes),
                    );
                    if self.runtime.apply_client_puddle_sync_wire(
                        &self.content_loader,
                        entity_id,
                        &puddle_sync,
                    ) {
                        report.entity_typed_records_applied += 1;
                    }
                }
                Some(EntityClassKind::Weather) => {
                    let Ok(weather_sync) = read_weather_state_sync(&mut read) else {
                        report.entity_parse_errors += 1;
                        return report;
                    };
                    let consumed = before_len - read.len();
                    let sync_bytes = sync_start[..consumed].to_vec();
                    report.merge(
                        self.runtime
                            .apply_client_entity_snapshot_record(entity_id, type_id, sync_bytes),
                    );
                    if self.runtime.apply_client_weather_state_sync_wire(
                        &self.content_loader,
                        entity_id,
                        &weather_sync,
                    ) {
                        report.entity_typed_records_applied += 1;
                    }
                }
                Some(EntityClassKind::WorldLabel) => {
                    let Ok(label_sync) = read_world_label_sync(&mut read) else {
                        report.entity_parse_errors += 1;
                        return report;
                    };
                    let consumed = before_len - read.len();
                    let sync_bytes = sync_start[..consumed].to_vec();
                    report.merge(
                        self.runtime
                            .apply_client_entity_snapshot_record(entity_id, type_id, sync_bytes),
                    );
                    if self
                        .runtime
                        .apply_client_world_label_sync_wire(entity_id, &label_sync)
                    {
                        report.entity_typed_records_applied += 1;
                    }
                }
                _ => {
                    report.entity_parse_errors += 1;
                    return report;
                }
            }
        }

        if !read.is_empty() {
            report.entity_parse_errors += 1;
        }
        report
    }

    fn sync_state_snapshot(&mut self) {
        if self.last_applied_world_data.is_none() {
            return;
        }

        let state_snapshot = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            state.last_state_snapshot.clone()
        };

        let Some(snapshot) = state_snapshot else {
            return;
        };

        if self.last_applied_state_snapshot.as_ref() == Some(&snapshot) {
            return;
        }

        self.game_state.apply_state_snapshot(&snapshot);
        self.sync_runtime_state_from_game_state();
        self.last_applied_state_snapshot = Some(snapshot);
    }

    fn sync_client_loaded_state(&mut self) {
        if self.last_applied_world_data.is_none() || !self.game_state.is_menu() {
            return;
        }

        let connect_confirm_sent = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            state.connect_confirm_sent
        };

        if connect_confirm_sent {
            self.game_state.set(GameStateState::Playing);
            self.sync_runtime_state_from_game_state();
        }
    }

    fn sync_runtime_state_from_game_state(&mut self) {
        self.runtime.state = self.game_state.clone();
        let building_count = self.runtime.buildings().len();
        for index in 0..building_count {
            self.runtime.sync_world_footprint_refs(index);
        }
        let network_context = if self.last_applied_world_data.is_some() {
            GameRuntimeNetworkContext::client()
        } else {
            GameRuntimeNetworkContext::offline()
        };
        self.runtime.set_network_context(network_context);
    }

    fn sync_runtime_state_from_world_data(&mut self, world_data: &NetworkWorldData) {
        self.sync_runtime_state_from_game_state();
        self.runtime
            .set_network_context(GameRuntimeNetworkContext::client());
        self.puddle_particle_rand_state =
            mix_puddle_particle_seed(world_data.rand_seed0, world_data.rand_seed1);
        self.last_runtime_map_load_report = world_data.map_snapshot.as_ref().map(|map| {
            self.runtime
                .load_network_map_with_buildings(&self.content_loader, map)
        });
        if self.last_runtime_map_load_report.is_none() {
            self.runtime.clear_buildings();
        }
    }

    fn apply_network_content_header(&mut self, snapshot: Option<&ContentHeaderSnapshot>) {
        let Some(snapshot) = snapshot else {
            self.content_loader.clear_temporary_mapper();
            return;
        };

        let block_name_fallback = BTreeMap::new();
        if self
            .content_loader
            .read_content_header(snapshot, &block_name_fallback)
            .is_err()
        {
            self.content_loader.clear_temporary_mapper();
        }
    }

    fn apply_network_player_data(
        &mut self,
        player_id: i32,
        player_data: Option<&NetworkPlayerData>,
    ) {
        let Some(player_data) = player_data else {
            self.player = PlayerComp::default();
            return;
        };

        self.player
            .reset(TeamId(self.game_state.rules.default_team as u8));
        self.player.apply_network_player_data(player_data);
        self.player.id = player_id;
        let selected_block = player_data.selected_block_id.and_then(|block_id| {
            self.mapped_block_name(block_id).and_then(|name| {
                self.content_loader
                    .block_by_name(name)
                    .map(|block| block.base().clone())
            })
        });
        let last_command = player_data.last_command_id.and_then(|command_id| {
            self.mapped_unit_command_name(command_id)
                .and_then(|name| self.content_loader.unit_command_by_name(name).cloned())
        });
        self.player.selected_block = selected_block;
        self.player.last_command = last_command;
    }

    fn apply_network_team_blocks(&mut self, team_blocks: Option<&LegacyTeamBlocks>) {
        let content_loader = self.content_loader.clone();
        self.game_state.apply_legacy_team_blocks(
            team_blocks,
            |block_id| {
                content_loader
                    .get_by_id(ContentType::Block, block_id)
                    .and_then(|content| content.name().map(str::to_string))
            },
            |content_type, content_id| {
                content_loader
                    .get_by_id(content_type, content_id)
                    .and_then(|content| content.name().map(str::to_string))
            },
        );
    }

    fn mapped_block_name(&self, block_id: ContentId) -> Option<&str> {
        self.content_loader
            .get_by_id(ContentType::Block, block_id)
            .and_then(|content| content.name())
    }

    fn mapped_unit_command_name(&self, command_id: ContentId) -> Option<&str> {
        self.content_loader
            .get_by_id(ContentType::UnitCommand, command_id)
            .and_then(|content| content.name())
    }
}

pub fn run(args: Vec<String>) -> DesktopLauncher {
    let mut launcher = DesktopLauncher::new(args);
    launcher.client.setup();
    let _ = launcher.merge_mods_directory_arg_into_texture_atlas();
    launcher.connect_from_args();
    launcher
}

pub fn banner() -> String {
    format!("mindustry desktop bootstrap ({UPSTREAM_BASELINE})")
}

fn current_millis() -> i64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    millis.min(i64::MAX as u128) as i64
}

const DESKTOP_PUDDLE_PARTICLE_RAND_DEFAULT: u64 = 0x9e37_79b9_7f4a_7c15;

fn mix_puddle_particle_seed(seed0: i64, seed1: i64) -> u64 {
    let mixed = (seed0 as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (seed1 as u64).rotate_left(32)
        ^ 0xd1b5_4a32_d192_ed03;
    if mixed == 0 {
        DESKTOP_PUDDLE_PARTICLE_RAND_DEFAULT
    } else {
        mixed
    }
}

fn next_puddle_particle_unit(rand_state: &mut u64) -> f32 {
    *rand_state = (*rand_state)
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*rand_state >> 40) as u32 as f32) / ((1u32 << 24) as f32)
}

fn next_puddle_particle_range(rand_state: &mut u64, range: f32) -> f32 {
    let range = range.max(0.0);
    (next_puddle_particle_unit(rand_state) * 2.0 - 1.0) * range
}

fn parse_connect_target(args: &[String]) -> Option<DesktopConnectTarget> {
    for (index, arg) in args.iter().enumerate() {
        if arg == "--connect" {
            if let Some(next) = args.get(index + 1) {
                if let Some(target) = parse_host_port(next) {
                    return Some(target);
                }
            }
        } else if let Some(value) = arg.strip_prefix("--connect=") {
            if let Some(target) = parse_host_port(value) {
                return Some(target);
            }
        }
    }
    None
}

fn parse_mods_directory_arg(args: &[String]) -> Option<String> {
    for (index, arg) in args.iter().enumerate() {
        if arg == "--mods" || arg == "--mods-dir" {
            if let Some(next) = args.get(index + 1) {
                if !next.is_empty() {
                    return Some(next.clone());
                }
            }
        } else if let Some(value) = arg
            .strip_prefix("--mods=")
            .or_else(|| arg.strip_prefix("--mods-dir="))
        {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn parse_host_port(value: &str) -> Option<DesktopConnectTarget> {
    let (host, port) = value.rsplit_once(':')?;
    let port = port.parse().ok()?;
    (!host.is_empty()).then(|| DesktopConnectTarget {
        host: host.to_string(),
        port,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        run, DesktopCameraShakeFrame, DesktopEffectRenderStats, DesktopFrameKind,
        DesktopFrameLoopEvent, DesktopFrameLoopExitReason, DesktopFrameLoopState,
        DesktopFramePacing, DesktopFramePayload, DesktopFrameSkipReason,
        DesktopGraphicsBlockParticleDrawCallKind, DesktopGraphicsCommandExecutionTrace,
        DesktopGraphicsExecutionStepTrace, DesktopGraphicsExecutionSummary,
        DesktopGraphicsExecutionTrace, DesktopGraphicsFrame,
        DesktopGraphicsLiveBackendDrawSpriteSink, DesktopGraphicsLiveBackendDrawSpriteTrace,
        DesktopGraphicsLiveBackendRenderCommandSink, DesktopGraphicsLiveBackendRenderCommandSource,
        DesktopGraphicsLiveBackendRenderCommandTrace,
        DesktopGraphicsLiveBackendRenderTargetEventKind,
        DesktopGraphicsLiveBackendRenderTargetSink, DesktopGraphicsLiveBackendRenderTargetTrace,
        DesktopGraphicsRenderer, DesktopGraphicsResolvedSpriteTrace,
        DesktopGraphicsShaderApplyExecutionTrace, DesktopGraphicsTextureSamplerTrace,
        DesktopInputTickEvent, DesktopLauncher, DesktopSurfaceConfig, DesktopSurfaceSize,
        HeadlessDesktopAudioRenderer, HeadlessDesktopCameraShakeRenderer,
        HeadlessDesktopEffectRenderer, HeadlessDesktopGraphicsRenderer,
    };
    use mindustry_core::mindustry::core::game_runtime::{
        GameRuntimeCampaignBlockState, GameRuntimeClientCameraShakeEvent,
        GameRuntimeDistributionBlockState, GameRuntimePayloadBlockState,
        GameRuntimeReconstructorConfigureResult, GameRuntimeUnitBlockState,
        GameRuntimeUnitCargoUnloadConfigureResult, GameRuntimeUnitFactoryConfigureResult,
    };
    use mindustry_core::mindustry::core::net_client::{
        ClientBlockSnapshotMirror, ClientBlockSnapshotRecordMirror, ClientEntitySnapshotMirror,
        ClientEntitySnapshotRecordMirror, ClientHiddenSnapshotMirror, ClientTileStorageMirror,
        ClientUnitItemMirror, ClientUnitPayloadMirror,
    };
    use mindustry_core::mindustry::core::{
        GameRuntime, GameRuntimeNetworkContext, WorldLoadEventKind,
    };
    use mindustry_core::mindustry::ctype::ContentType;
    use mindustry_core::mindustry::ctype::{Content, ContentId};
    use mindustry_core::mindustry::entities::comp::DecalColor;
    use mindustry_core::mindustry::graphics::{
        BlockDrawStage, BlockRendererBlockParticlePlan, BlockRendererPlan, CacheLayer, Layer,
        LightPrimitive, LoadFrameInput, LoadStage, MenuFrameInput, MinimapCamera,
        MinimapOverlayInput, PageType, ParticleRendererState, RenderBlendMode, RenderBridge,
        RenderCamera, RenderCommand, RenderFramePlan, RenderPass, RenderPassKind, RenderPoint,
        RenderProperty, RenderRect, RenderResolveKind, RenderSize, RenderTarget, RenderTextAlign,
        RenderViewport, ShaderApplyContext, ShaderApplyPlan, ShaderCatalog, ShaderDispatchFrame,
        ShaderId, TextureAtlasPlan, TileCoord,
    };
    use mindustry_core::mindustry::io::{
        ContentHeaderEntry, ContentHeaderSnapshot, LegacyMapBlockRecord, LegacyMapFloorRecord,
        LegacyShortChunkMap,
    };
    use mindustry_core::mindustry::modsys::{ModResourcePlan, ModSpritePackSource};
    use mindustry_core::mindustry::net::{
        packet_ids, ConnectPacket, PacketEnvelope, PacketKind, PacketSerializer,
    };
    use mindustry_core::mindustry::net::{ArcNetProvider, NetProvider};
    use mindustry_core::mindustry::net::{
        AssemblerDroneSpawnedCallPacket, AssemblerUnitSpawnedCallPacket,
        ClientPlanSnapshotReceivedCallPacket, CommandBuildingCallPacket, EffectCallPacket,
        EffectCallPacket2, LandingPadLandedCallPacket, NetworkPlayerData, NetworkPlayerSyncData,
        NetworkWorldData, SoundAtCallPacket, StateSnapshotCallPacket, TileConfigCallPacket,
        UnitBlockSpawnCallPacket, UnitCapDeathCallPacket, UnitDeathCallPacket,
        UnitDespawnCallPacket, UnitDestroyCallPacket, UnitEnteredPayloadCallPacket,
        UnitEnvDeathCallPacket, UnitSafeDeathCallPacket, UnitSpawnCallPacket,
        UnitTetherBlockSpawnedCallPacket,
    };
    use mindustry_core::mindustry::{
        entities::{
            comp::{
                BuildingComp, BuildingTetherAction, BuildingTetherRef, PayloadKind, PuddleComp,
                UnitComp, UnitControllerState,
            },
            entity_class_id, standard_effect_id, LegDestroyData, PlayerComp, PuddleLiquidInfo,
            StandardEffectDrawKind, TextureRegionRef, BULLET_CLASS_ID, DECAL_CLASS_ID,
            EFFECT_STATE_CLASS_ID, FIRE_CLASS_ID, PLAYER_CLASS_ID, PUDDLE_CLASS_ID,
            WEATHER_STATE_CLASS_ID, WORLD_LABEL_CLASS_ID,
        },
        game::{BlockPlan, Trigger, TEAM_CRUX, TEAM_SHARDED},
        io::type_io::ControllerWire,
        io::{
            type_io, BuildingRef, LegacyTeamBlockGroup, LegacyTeamBlockPlan, LegacyTeamBlocks,
            TeamId, TypeValue, UnitRef, Vec2 as IoVec2,
        },
        r#type::{ItemStack, PayloadKey, PayloadSeq, Sector, UnitType, Weapon},
        world::blocks::campaign::LandingPadState,
        world::blocks::payloads::{PayloadBlockBuildState, PayloadLoaderState, PayloadRef},
        world::blocks::units::{
            ReconstructorState, UnitAssemblerState, UnitBlockState, UnitCargoLoaderState,
            UnitCargoUnloadPointState, UnitFactoryState,
        },
    };
    use std::collections::BTreeMap;
    use std::net::{TcpListener, UdpSocket};

    fn free_local_port() -> u16 {
        for _ in 0..32 {
            let tcp = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = tcp.local_addr().unwrap().port();
            if UdpSocket::bind(("127.0.0.1", port)).is_ok() {
                return port;
            }
        }
        panic!("could not reserve a local TCP/UDP port pair");
    }

    fn sample_network_player_data(
        selected_block_id: Option<ContentId>,
        last_command_id: Option<ContentId>,
    ) -> NetworkPlayerData {
        NetworkPlayerData {
            revision: 2,
            admin: true,
            boosting: true,
            color: 0x11_22_33_44,
            last_command_id,
            mouse_x: 12.5,
            mouse_y: -6.25,
            name: Some("pilot".into()),
            selected_block_id,
            selected_rotation: 3,
            shooting: true,
            team: TeamId(6),
            typing: true,
            unit: UnitRef::Block { tile_pos: 42 },
            x: 100.0,
            y: 200.0,
        }
    }

    fn sample_network_player_sync_data(
        selected_block_id: Option<ContentId>,
    ) -> NetworkPlayerSyncData {
        NetworkPlayerSyncData {
            admin: false,
            boosting: true,
            color: 0x55_66_77_88,
            mouse_x: 320.0,
            mouse_y: 640.0,
            name: Some("snapshot-pilot".into()),
            selected_block_id,
            selected_rotation: 2,
            shooting: true,
            team: TeamId(3),
            typing: true,
            unit: UnitRef::Unit { id: 7701 },
            x: 900.0,
            y: 901.0,
        }
    }

    fn sample_network_world_data(player: Option<NetworkPlayerData>) -> NetworkWorldData {
        let mut map_tags = BTreeMap::new();
        map_tags.insert("name".into(), "Network Map".into());
        map_tags.insert("build".into(), "157".into());
        map_tags.insert("version".into(), "11".into());

        NetworkWorldData {
            map_locales_json: r#"{"en":{"name":"Network Map"}}"#.into(),
            map_tags,
            wave: 12,
            wave_time: 30.5,
            tick: 99.25,
            rand_seed0: 123,
            rand_seed1: 456,
            player_id: 91,
            player,
            map_snapshot: Some(LegacyShortChunkMap {
                width: 3,
                height: 2,
                floors: vec![LegacyMapFloorRecord {
                    index: 0,
                    floor_id: 1,
                    ore_id: 0,
                    consecutives: 5,
                }],
                blocks: vec![LegacyMapBlockRecord {
                    index: 0,
                    block_id: 0,
                    packed_flags: 0,
                    has_entity: false,
                    has_old_data: false,
                    has_new_data: false,
                    is_center: true,
                    new_data: None,
                    old_data: None,
                    building: None,
                    consecutives: 5,
                }],
            }),
            ..NetworkWorldData::default()
        }
    }

    #[test]
    fn desktop_frame_loop_presents_limited_frames_and_increments_frame_index() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("fire").unwrap() as u16,
                    x: 32.0,
                    y: 48.0,
                    rotation: 0.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });
        let mut frame_loop =
            DesktopFrameLoopState::new(Default::default(), DesktopFramePacing::uncapped());
        let mut graphics_renderer = HeadlessDesktopGraphicsRenderer::default();
        let mut effect_renderer = HeadlessDesktopEffectRenderer::default();
        let mut results = Vec::new();

        let summary = launcher.run_with_desktop_frame_loop(
            &mut frame_loop,
            &mut graphics_renderer,
            &mut effect_renderer,
            Some(2),
            |_| vec![DesktopFrameLoopEvent::Tick],
            |result| results.push(result.clone()),
            |_| panic!("uncapped test loop should not sleep"),
        );

        assert_eq!(summary.exit_reason, DesktopFrameLoopExitReason::FrameLimit);
        assert_eq!(summary.steps, 2);
        assert_eq!(summary.frames_presented, 2);
        assert_eq!(summary.last_frame_index, Some(1));
        assert_eq!(frame_loop.next_frame_index, 2);
        assert_eq!(graphics_renderer.frames_rendered, 2);
        assert_eq!(effect_renderer.frames_rendered, 2);
        assert_eq!(
            results
                .iter()
                .map(|result| result.frame_index)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert!(results.iter().all(|result| result.presented));
        assert!(results[0].graphics_stats.is_some());
        assert_eq!(results[0].effect_stats.unwrap().draw_plans, 1);
    }

    #[test]
    fn desktop_frame_loop_close_event_stops_without_presenting() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let mut frame_loop =
            DesktopFrameLoopState::new(Default::default(), DesktopFramePacing::uncapped());
        let mut graphics_renderer = HeadlessDesktopGraphicsRenderer::default();
        let mut effect_renderer = HeadlessDesktopEffectRenderer::default();
        let mut results = Vec::new();

        let summary = launcher.run_with_desktop_frame_loop(
            &mut frame_loop,
            &mut graphics_renderer,
            &mut effect_renderer,
            None,
            |_| vec![DesktopFrameLoopEvent::CloseRequested],
            |result| results.push(result.clone()),
            |_| panic!("close-before-present should not sleep"),
        );

        assert_eq!(summary.exit_reason, DesktopFrameLoopExitReason::Closed);
        assert_eq!(summary.steps, 1);
        assert_eq!(summary.frames_presented, 0);
        assert_eq!(summary.last_frame_index, None);
        assert!(frame_loop.closed);
        assert_eq!(frame_loop.next_frame_index, 0);
        assert_eq!(graphics_renderer.frames_rendered, 0);
        assert_eq!(effect_renderer.frames_rendered, 0);
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].skip_reason,
            Some(DesktopFrameSkipReason::CloseRequested)
        );
        assert!(results[0].close_requested);
        assert!(!results[0].presented);
    }

    #[test]
    fn desktop_frame_loop_applies_resize_and_input_tick_events() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let mut frame_loop =
            DesktopFrameLoopState::new(Default::default(), DesktopFramePacing::uncapped());
        let mut graphics_renderer = HeadlessDesktopGraphicsRenderer::default();
        let mut effect_renderer = HeadlessDesktopEffectRenderer::default();
        let resized = DesktopSurfaceSize::new(800, 600);
        let input = DesktopInputTickEvent::CursorMoved { x: 12.5, y: 25.0 };

        let result = launcher.step_desktop_frame_loop(
            &mut frame_loop,
            &[
                DesktopFrameLoopEvent::Resize(resized),
                DesktopFrameLoopEvent::Input(input.clone()),
            ],
            &mut graphics_renderer,
            &mut effect_renderer,
        );

        assert!(result.presented);
        assert_eq!(result.frame_index, 0);
        assert_eq!(result.resized_to, Some(resized));
        assert_eq!(result.surface.size, resized);
        assert_eq!(result.input_events, vec![input]);
        assert_eq!(frame_loop.surface.size, resized);
        assert_eq!(frame_loop.input_events_seen, 1);
        assert_eq!(frame_loop.next_frame_index, 1);
    }

    #[test]
    fn desktop_frame_loop_paced_sleep_only_after_successful_present() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let mut frame_loop = DesktopFrameLoopState::new(
            Default::default(),
            DesktopFramePacing::new(std::time::Duration::from_millis(16)),
        );
        let mut graphics_renderer = HeadlessDesktopGraphicsRenderer::default();
        let mut effect_renderer = HeadlessDesktopEffectRenderer::default();
        let mut sleep_durations = Vec::new();
        let mut poll_count = 0u32;

        let summary = launcher.run_with_desktop_frame_loop(
            &mut frame_loop,
            &mut graphics_renderer,
            &mut effect_renderer,
            None,
            |_| {
                poll_count += 1;
                match poll_count {
                    1 => vec![DesktopFrameLoopEvent::Tick],
                    2 => vec![DesktopFrameLoopEvent::CloseRequested],
                    _ => panic!("paced loop should stop after the close request"),
                }
            },
            |_| {},
            |duration| sleep_durations.push(duration),
        );

        assert_eq!(summary.exit_reason, DesktopFrameLoopExitReason::Closed);
        assert_eq!(summary.steps, 2);
        assert_eq!(summary.frames_presented, 1);
        assert_eq!(summary.last_frame_index, Some(0));
        assert_eq!(poll_count, 2);
        assert_eq!(graphics_renderer.frames_rendered, 1);
        assert_eq!(effect_renderer.frames_rendered, 1);
        assert_eq!(sleep_durations, vec![std::time::Duration::from_millis(16)]);
    }

    #[test]
    fn desktop_frame_loop_closed_state_short_circuits_without_poll_render_or_sleep() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let mut frame_loop = DesktopFrameLoopState::new(
            Default::default(),
            DesktopFramePacing::new(std::time::Duration::from_millis(16)),
        );
        frame_loop.closed = true;
        let mut graphics_renderer = HeadlessDesktopGraphicsRenderer::default();
        let mut effect_renderer = HeadlessDesktopEffectRenderer::default();

        let summary = launcher.run_with_desktop_frame_loop(
            &mut frame_loop,
            &mut graphics_renderer,
            &mut effect_renderer,
            None,
            |_| panic!("closed loop state should short-circuit before polling"),
            |_| panic!("closed loop state should not reach after-present"),
            |_| panic!("closed loop state should not sleep"),
        );

        assert_eq!(summary.exit_reason, DesktopFrameLoopExitReason::Closed);
        assert_eq!(summary.steps, 0);
        assert_eq!(summary.frames_presented, 0);
        assert_eq!(summary.last_frame_index, None);
        assert!(frame_loop.closed);
        assert_eq!(graphics_renderer.frames_rendered, 0);
        assert_eq!(effect_renderer.frames_rendered, 0);
    }

    #[test]
    fn desktop_surface_config_default_uses_fixed_desktop_contract() {
        let config = DesktopSurfaceConfig::default();

        assert_eq!(config.title, "Mindustry");
        assert_eq!(config.size, DesktopSurfaceSize::new(1280, 720));
        assert_eq!(config.scale_factor, 1.0);
        assert!(config.resizable);
        assert!(config.visible);
    }

    #[test]
    fn desktop_default_run_keeps_headless_data_path_without_mod_scan_flags() {
        let launcher = run(vec!["mindustry-desktop".into()]);

        assert_eq!(launcher.client.context.paths.data_dir, "data");
        assert_eq!(launcher.args, vec!["mindustry-desktop".to_string()]);
        assert_eq!(launcher.connect_target, None);
        assert_eq!(launcher.connect_error, None);
        assert_eq!(launcher.mods_directory_arg, None);
        assert_eq!(launcher.mods_directory_error, None);
        assert_eq!(launcher.last_mods_directory_merge_count, Some(0));
    }

    #[test]
    fn desktop_run_merges_explicit_mods_directory_without_default_scan() {
        let root = create_mods_container_sprite_fixture_root();
        let launcher = run(vec![
            "mindustry-desktop".into(),
            "--mods-dir".into(),
            root.display().to_string(),
        ]);

        assert_eq!(
            launcher.mods_directory_arg.as_deref(),
            Some(root.to_string_lossy().as_ref())
        );
        assert_eq!(launcher.mods_directory_error, None);
        assert_eq!(launcher.last_mods_directory_merge_count, Some(2));
        assert!(launcher.texture_atlas.lookup("alpha-alpha-router").is_ok());
        assert_eq!(
            launcher
                .texture_atlas
                .lookup("router")
                .unwrap()
                .region
                .source_path,
            "sprites-override/router.png"
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn desktop_playable_smoke_ready_after_world_stream_and_confirm() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let content = launcher.content_loader.clone();
        let mut server_runtime = GameRuntime::default();
        let smoke = server_runtime.seed_playable_smoke_world(&content);
        let default_team = TeamId(server_runtime.state.rules.default_team as u8);
        let (spawn_x, spawn_y) = server_runtime
            .state
            .teams
            .get_or_null(default_team.0)
            .and_then(|team| team.core())
            .map(|core| (core.x, core.y))
            .expect("smoke core spawn should exist");
        let mut player = NetworkPlayerData::bootstrap();
        player.team = default_team;
        player.name = Some("smoke-client".into());
        player.x = spawn_x;
        player.y = spawn_y;

        let world_data = NetworkWorldData {
            map_tags: server_runtime.state.map.tags.clone(),
            player_id: 77,
            player: Some(player),
            map_snapshot: Some(server_runtime.export_network_map_snapshot(&content)),
            ..NetworkWorldData::default()
        };

        launcher.client.setup();
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.connected = true;
            state.connect_confirm_sent = true;
            state.last_loaded_world_data = Some(world_data);
        }

        launcher.update();

        let status = launcher.playable_smoke_status();
        assert!(status.ready(), "{status:?}, smoke={smoke:?}");
        assert!(launcher.playable_smoke_ready());
        assert_eq!(status.world_width, 16);
        assert_eq!(status.world_height, 16);
        assert_eq!(status.buildings, 1);
    }

    fn sample_state_snapshot() -> StateSnapshotCallPacket {
        let mut core_data = Vec::new();
        core_data.push(2);
        core_data.push(TEAM_SHARDED);
        core_data.extend_from_slice(&2i16.to_be_bytes());
        core_data.extend_from_slice(&0i16.to_be_bytes());
        core_data.extend_from_slice(&75i32.to_be_bytes());
        core_data.extend_from_slice(&3i16.to_be_bytes());
        core_data.extend_from_slice(&12i32.to_be_bytes());
        core_data.push(TEAM_CRUX);
        core_data.extend_from_slice(&1i16.to_be_bytes());
        core_data.extend_from_slice(&1i16.to_be_bytes());
        core_data.extend_from_slice(&5i32.to_be_bytes());

        StateSnapshotCallPacket {
            wave_time: 12.5,
            wave: 9,
            enemies: 17,
            paused: true,
            game_over: true,
            time_data: 456,
            tps: 255,
            rand0: 11,
            rand1: 22,
            core_data,
        }
    }

    fn sample_team_blocks(block_id: ContentId) -> LegacyTeamBlocks {
        LegacyTeamBlocks {
            groups: vec![LegacyTeamBlockGroup {
                team_id: 7,
                plans: vec![
                    LegacyTeamBlockPlan {
                        x: 5,
                        y: 6,
                        rotation: 1,
                        block_id,
                        config: TypeValue::String("cfg".into()),
                    },
                    LegacyTeamBlockPlan {
                        x: 7,
                        y: 8,
                        rotation: 2,
                        block_id,
                        config: TypeValue::Int(9),
                    },
                ],
            }],
        }
    }

    fn sample_content_header_snapshot(
        block_name: &str,
        unit_command_name: &str,
    ) -> ContentHeaderSnapshot {
        ContentHeaderSnapshot {
            entries: vec![
                ContentHeaderEntry {
                    content_type: ContentType::Block.ordinal(),
                    names: vec![block_name.into()],
                },
                ContentHeaderEntry {
                    content_type: ContentType::UnitCommand.ordinal(),
                    names: vec![unit_command_name.into()],
                },
            ],
        }
    }

    fn sample_minimap_overlay_input(full_view: bool) -> MinimapOverlayInput {
        MinimapOverlayInput {
            screen_x: 0.0,
            screen_y: 0.0,
            screen_width: 128.0,
            screen_height: 128.0,
            full_view,
            mobile: false,
            net_active: false,
            show_pings: false,
            fog: false,
            static_fog: false,
            dynamic_color: 0x000000ff,
            dynamic_alpha: 0.0,
            show_spawns: false,
            has_spawns: false,
            waves: false,
            wave_team_color: 0xffffffff,
            drop_zone_radius: 0.0,
            time: 0.0,
            global_time: 0.0,
            units: Vec::new(),
            players: Vec::new(),
            spawns: Vec::new(),
            indicators: Vec::new(),
            markers: Vec::new(),
        }
    }

    #[test]
    fn desktop_launcher_updates_client_service_and_real_net_client() {
        let port = free_local_port();
        let mut server = ArcNetProvider::new();
        server.host_server(port).unwrap();
        let mut launcher = DesktopLauncher::new(Vec::new());

        launcher.client.setup();
        assert!(launcher.client.service_waiting_for_client_load());

        {
            let mut net = launcher.net_client.net_mut();
            net.connect("127.0.0.1", port, Box::new(|| {})).unwrap();
        }

        launcher.update();

        assert!(launcher.client.loaded);
        assert!(launcher.client.service.events_registered());
        let state = launcher.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.connect_events, 1);
        assert_eq!(state.update_count, 1);
        drop(state);

        launcher.net_client.net_mut().disconnect();
        server.close_server();
    }

    #[test]
    fn desktop_launcher_drains_runtime_trigger_events_into_game_service() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher.runtime.state.set_sector(Some(Sector::new(7)));
        launcher.runtime.note_trigger_event(Trigger::NeoplasmReact);

        assert!(launcher.sync_runtime_trigger_events_to_service());
        assert!(launcher.runtime.trigger_events.is_empty());
        assert_eq!(
            launcher
                .last_service_trigger_apply_summary
                .map(|summary| summary.achievements_completed),
            Some(1)
        );
        assert!(launcher
            .client
            .service
            .achievements()
            .contains("neoplasmWater"));
        assert!(!launcher.sync_runtime_trigger_events_to_service());
    }

    #[test]
    fn desktop_launcher_applies_loaded_world_data_to_game_state_world_and_player() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let selected_block_id = launcher
            .content_loader
            .block(0)
            .expect("base content should include block 0")
            .base()
            .id;
        let last_command_id = launcher
            .content_loader
            .unit_command(0)
            .expect("base content should include command 0")
            .base
            .base
            .id;

        let world_data = sample_network_world_data(Some(sample_network_player_data(
            Some(selected_block_id),
            Some(last_command_id),
        )));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }

        launcher.update();

        assert_eq!(launcher.game_state.map.name(), "Network Map");
        assert_eq!(launcher.game_state.rand_seed0, 123);
        assert_eq!(launcher.game_state.rand_seed1, 456);
        assert_eq!(launcher.game_state.world.width(), 3);
        assert_eq!(launcher.game_state.world.height(), 2);
        assert_eq!(launcher.runtime.state.world.width(), 3);
        assert_eq!(launcher.runtime.state.world.height(), 2);
        assert_eq!(
            launcher.game_state.world.load_events(),
            &[
                WorldLoadEventKind::Begin,
                WorldLoadEventKind::End,
                WorldLoadEventKind::Loaded,
            ]
        );
        assert!(launcher.player.admin);
        assert_eq!(launcher.player.id, 91);
        assert!(launcher.player.boosting);
        assert_eq!(launcher.player.color, 0x11_22_33_44);
        assert_eq!(launcher.player.mouse_x, 12.5);
        assert_eq!(launcher.player.mouse_y, -6.25);
        assert_eq!(launcher.player.name, "pilot");
        assert_eq!(launcher.player.selected_rotation, 3);
        assert!(launcher.player.shooting);
        assert_eq!(launcher.player.team, TeamId(6));
        assert!(launcher.player.typing);
        assert_eq!(
            launcher.player.unit_ref(),
            Some(UnitRef::Block { tile_pos: 42 })
        );
        assert_eq!(
            launcher.player.selected_block,
            launcher
                .content_loader
                .block(selected_block_id)
                .map(|block| block.base().clone())
        );
        assert_eq!(
            launcher.player.last_command,
            launcher
                .content_loader
                .unit_command(last_command_id)
                .cloned()
        );
    }

    #[test]
    fn desktop_launcher_applies_unit_item_mirror_to_runtime_unit_snapshot() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let flare = launcher
            .content_loader
            .unit_by_name("flare")
            .unwrap()
            .clone();
        launcher
            .runtime
            .client_unit_snapshot_entities
            .insert(8801, UnitComp::new(8801, flare, TeamId(4)));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.unit_item_mirrors.insert(
                8801,
                ClientUnitItemMirror {
                    item: Some("copper".into()),
                    amount: 3,
                    take_items_packets_seen: 1,
                    ..ClientUnitItemMirror::default()
                },
            );
        }
        launcher.update();
        let unit = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&8801)
            .unwrap();
        assert_eq!(unit.items.stack.item.as_deref(), Some("copper"));
        assert_eq!(unit.items.stack.amount, 3);

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            let mirror = state.unit_item_mirrors.get_mut(&8801).unwrap();
            mirror.item = Some("lead".into());
            mirror.amount = 5;
            mirror.take_items_packets_seen = 2;
        }
        launcher.update();
        let unit = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&8801)
            .unwrap();
        assert_eq!(unit.items.stack.item.as_deref(), Some("lead"));
        assert_eq!(unit.items.stack.amount, 5);

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            let mirror = state.unit_item_mirrors.get_mut(&8801).unwrap();
            mirror.item = None;
            mirror.amount = 99;
            mirror.take_items_packets_seen = 3;
        }
        launcher.update();
        let unit = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&8801)
            .unwrap();
        assert_eq!(unit.items.stack.item, None);
        assert_eq!(unit.items.stack.amount, 0);
    }

    #[test]
    fn desktop_launcher_applies_building_storage_mirror_to_runtime_building() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let storage_block = launcher
            .content_loader
            .block_by_name("unit-cargo-unload-point")
            .unwrap();
        let copper = launcher
            .content_loader
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let tile_pos = mindustry_core::mindustry::world::point2_pack(9, 9);
        launcher.runtime.add_building(BuildingComp::new(
            tile_pos,
            storage_block.base().clone(),
            TeamId(4),
        ));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            let mut items = BTreeMap::new();
            items.insert("copper".into(), 4);
            state.building_storage_mirrors.insert(
                tile_pos,
                ClientTileStorageMirror {
                    items,
                    ..ClientTileStorageMirror::default()
                },
            );
        }
        launcher.update();
        assert_eq!(
            launcher
                .runtime
                .buildings()
                .iter()
                .find(|building| building.tile_pos == tile_pos)
                .unwrap()
                .items
                .as_ref()
                .unwrap()
                .get(copper),
            4
        );

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state
                .building_storage_mirrors
                .get_mut(&tile_pos)
                .unwrap()
                .items
                .insert("copper".into(), 7);
        }
        launcher.update();
        assert_eq!(
            launcher
                .runtime
                .buildings()
                .iter()
                .find(|building| building.tile_pos == tile_pos)
                .unwrap()
                .items
                .as_ref()
                .unwrap()
                .get(copper),
            7
        );
    }

    #[test]
    fn desktop_launcher_applies_unit_payload_mirror_to_runtime_unit_snapshot() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let mega = launcher
            .content_loader
            .unit_by_name("mega")
            .unwrap()
            .clone();
        launcher
            .runtime
            .client_unit_snapshot_entities
            .insert(8802, UnitComp::new(8802, mega, TeamId(4)));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.unit_payload_mirrors.insert(
                8802,
                ClientUnitPayloadMirror {
                    payload_count: 2,
                    picked_build_payloads_seen: 1,
                    picked_unit_payloads_seen: 1,
                    payload_drops_seen: 0,
                },
            );
        }
        launcher.update();
        let unit = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&8802)
            .unwrap();
        let payload = unit.payload.as_ref().unwrap();
        assert_eq!(payload.payloads.len(), 2);
        assert_eq!(
            payload
                .payloads
                .iter()
                .filter(|payload| payload.kind == PayloadKind::Unit)
                .count(),
            1
        );
        assert_eq!(
            payload
                .payloads
                .iter()
                .filter(|payload| payload.kind == PayloadKind::Build)
                .count(),
            1
        );

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            let mirror = state.unit_payload_mirrors.get_mut(&8802).unwrap();
            mirror.payload_count = 1;
            mirror.payload_drops_seen = 1;
        }
        launcher.update();
        let unit = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&8802)
            .unwrap();
        assert_eq!(unit.payload.as_ref().unwrap().payloads.len(), 1);

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            let mirror = state.unit_payload_mirrors.get_mut(&8802).unwrap();
            mirror.payload_count = 0;
            mirror.payload_drops_seen = 2;
        }
        launcher.update();
        let unit = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&8802)
            .unwrap();
        assert_eq!(unit.payload.as_ref().unwrap().payloads.len(), 0);
    }

    #[test]
    fn desktop_launcher_applies_unit_entered_payload_packet_to_runtime_payload_building() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let tile_pos = mindustry_core::mindustry::world::point2_pack(6, 7);
        let loader = launcher
            .content_loader
            .block_by_name("payload-mass-driver")
            .unwrap()
            .base()
            .clone();
        launcher
            .runtime
            .buildings
            .push(BuildingComp::new(tile_pos, loader, TeamId(4)));
        let flare = launcher
            .content_loader
            .unit_by_name("flare")
            .unwrap()
            .clone();
        let mut unit = UnitComp::new(8803, flare, TeamId(4));
        unit.set_rotation(90.0);
        launcher
            .runtime
            .client_unit_snapshot_entities
            .insert(8803, unit);

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::UnitEnteredPayloadCallPacket(
                UnitEnteredPayloadCallPacket {
                    unit: UnitRef::Unit { id: 8803 },
                    build: BuildingRef::new(tile_pos),
                },
            ));
        }
        launcher.update();

        let report = launcher
            .last_unit_entered_payload_apply_report
            .expect("unit-entered-payload packet should be applied");
        assert_eq!(report.unit_id, Some(8803));
        assert!(report.payload_attached);
        assert!(!launcher
            .runtime
            .client_unit_snapshot_entities
            .contains_key(&8803));
        assert!(launcher.runtime.client_hidden_entity_ids.contains(&8803));
        let Some(GameRuntimePayloadBlockState::MassDriver { common, driver }) =
            launcher.runtime.payload_runtime_states.get(&tile_pos)
        else {
            panic!("payload-mass-driver should receive entered unit payload");
        };
        assert!(driver.loaded);
        assert!(matches!(
            common.payload,
            Some(PayloadRef::Unit {
                class_id: 3,
                ref unit_bytes
            }) if !unit_bytes.is_empty()
        ));
    }

    #[test]
    fn desktop_launcher_syncs_unit_tether_block_spawned_packet_to_runtime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let tile_pos = mindustry_core::mindustry::world::point2_pack(7, 7);
        let loader_def = launcher
            .content_loader
            .block_by_name("unit-cargo-loader")
            .unwrap();
        launcher.runtime.add_building(BuildingComp::new(
            tile_pos,
            loader_def.base().clone(),
            TeamId(4),
        ));
        launcher.runtime.distribution_runtime_states.insert(
            tile_pos,
            GameRuntimeDistributionBlockState::UnitCargoLoader(UnitCargoLoaderState {
                build_progress: 0.5,
                has_unit: false,
                ..UnitCargoLoaderState::default()
            }),
        );

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::UnitTetherBlockSpawnedCallPacket(
                UnitTetherBlockSpawnedCallPacket {
                    tile: Some(tile_pos),
                    id: 9901,
                },
            ));
        }
        launcher.update();

        let Some(GameRuntimeDistributionBlockState::UnitCargoLoader(state)) =
            launcher.runtime.distribution_runtime_states.get(&tile_pos)
        else {
            panic!("unit cargo loader state should remain present");
        };
        assert_eq!(state.read_unit_id, 9901);
        assert_eq!(state.build_progress, 0.0);
        assert!(!state.has_unit);
        let spawned = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&9901)
            .expect("desktop runtime should materialize cargo unit from tether packet");
        assert_eq!(spawned.type_info.name(), "manifold");
        assert_eq!(spawned.team_id(), TeamId(4));
        assert_eq!(spawned.x(), launcher.runtime.buildings()[0].x);
        assert_eq!(spawned.y(), launcher.runtime.buildings()[0].y);
        assert!(spawned.controller.is_cargo());
        assert_eq!(
            spawned
                .cargo_ai
                .as_ref()
                .and_then(|cargo| cargo.tether_tile_pos),
            Some(tile_pos)
        );
        assert_eq!(
            launcher
                .runtime
                .client_unit_tether_block_spawned_packets_applied,
            1
        );
    }

    #[test]
    fn desktop_launcher_syncs_unit_block_spawn_packet_to_runtime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let tile_pos = mindustry_core::mindustry::world::point2_pack(8, 7);
        let factory_def = launcher
            .content_loader
            .block_by_name("air-factory")
            .unwrap();
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        launcher.runtime.add_building(BuildingComp::new(
            tile_pos,
            factory_def.base().clone(),
            TeamId(4),
        ));
        launcher.runtime.unit_runtime_states.insert(
            tile_pos,
            GameRuntimeUnitBlockState::Factory {
                common: PayloadBlockBuildState {
                    payload: Some(PayloadRef::Block {
                        block: router_def.base().id,
                        version: 0,
                        build_bytes: Vec::new(),
                    }),
                    ..PayloadBlockBuildState::default()
                },
                factory: UnitFactoryState {
                    base: UnitBlockState {
                        progress: 9.0,
                        has_payload: true,
                        ..UnitBlockState::default()
                    },
                    current_plan: 0,
                    ..UnitFactoryState::default()
                },
            },
        );

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::UnitBlockSpawnCallPacket(
                UnitBlockSpawnCallPacket {
                    tile: Some(tile_pos),
                },
            ));
        }
        launcher.update();

        let Some(GameRuntimeUnitBlockState::Factory { common, factory }) =
            launcher.runtime.unit_runtime_states.get(&tile_pos)
        else {
            panic!("unit factory state should remain present");
        };
        assert!(common.payload.is_none());
        assert_eq!(factory.base.progress, 0.0);
        assert!(!factory.base.has_payload);
        assert_eq!(factory.current_plan, 0);
        assert_eq!(launcher.last_applied_unit_lifecycle_packets_seen, 1);
    }

    #[test]
    fn desktop_launcher_syncs_effect_call_packet2_to_local_effect_queue() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let packet = EffectCallPacket2 {
            effect: EffectCallPacket {
                effect_id: standard_effect_id("neoplasmHeal").unwrap() as u16,
                x: 80.0,
                y: 96.0,
                rotation: 15.0,
                color: type_io::RgbaColor::new(-1),
            },
            data: TypeValue::Unit(37),
        };

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::EffectCallPacket2(packet.clone()));
        }
        launcher.update();

        assert!(
            launcher.runtime.client_local_effect_events.is_empty(),
            "desktop update should materialize packet ingress into EffectStateComp before render"
        );
        assert_eq!(launcher.runtime.client_local_effect_entities.len(), 1);
        let state = launcher
            .runtime
            .client_local_effect_entities
            .get(&-1)
            .unwrap();
        assert_eq!(state.effect_id, Some(packet.effect.effect_id));
        assert_eq!((state.x, state.y, state.rotation), (80.0, 96.0, 15.0));
        assert_eq!(state.lifetime, 120.0);
        assert_eq!(state.effect_clip, 50.0);
        assert!(state.rot_with_parent);
        assert_eq!(state.data, TypeValue::Unit(37));
        assert_eq!(state.time, 1.0);
        launcher.update();
        assert_eq!(
            launcher.runtime.client_local_effect_entities.len(),
            1,
            "same last effect packet should not create another local state without a new counter"
        );
    }

    #[test]
    fn desktop_launcher_drains_local_effect_events_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let packet = EffectCallPacket2 {
            effect: EffectCallPacket {
                effect_id: standard_effect_id("ripple").unwrap() as u16,
                x: 8.0,
                y: 16.0,
                rotation: 0.0,
                color: type_io::RgbaColor::new(-1),
            },
            data: TypeValue::Null,
        };
        launcher
            .runtime
            .client_local_effect_events
            .push(packet.clone());

        let drained = launcher.drain_local_effect_events_for_render();

        assert_eq!(drained, vec![packet]);
        assert!(launcher.runtime.client_local_effect_events.is_empty());
        assert!(launcher.drain_local_effect_events_for_render().is_empty());
    }

    #[test]
    fn desktop_launcher_syncs_and_drains_local_sound_at_events_for_audio() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let packet = SoundAtCallPacket {
            sound_id: mindustry_core::mindustry::audio::standard_sound_id("unitExplode1").unwrap(),
            x: 8.0,
            y: 16.0,
            volume: 0.7,
            pitch: 1.0,
        };
        launcher
            .runtime
            .client_local_sound_at_events
            .push(packet.clone());

        assert_eq!(launcher.sync_local_sound_at_events_for_audio(), 1);
        assert!(launcher.runtime.client_local_sound_at_events.is_empty());
        assert_eq!(launcher.pending_sound_at_events, vec![packet.clone()]);

        let drained = launcher.drain_sound_at_events_for_audio();
        assert_eq!(drained, vec![packet]);
        assert!(launcher.pending_sound_at_events.is_empty());
    }

    #[test]
    fn desktop_launcher_plays_sound_at_audio_frame_with_headless_renderer() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let packet = SoundAtCallPacket {
            sound_id: mindustry_core::mindustry::audio::standard_sound_id("wreckFall").unwrap(),
            x: 24.0,
            y: 48.0,
            volume: 0.8,
            pitch: 1.0,
        };
        launcher.pending_sound_at_events.push(packet.clone());

        let frame = launcher.sound_at_audio_frame();
        assert_eq!(frame.sound_at_events, vec![packet.clone()]);

        let mut renderer = HeadlessDesktopAudioRenderer::default();
        let stats = launcher.play_sound_at_audio_frame_with(&mut renderer);
        assert_eq!(stats.sound_at_events, 1);
        assert_eq!(renderer.frames_played, 1);
        assert_eq!(renderer.last_stats.sound_at_events, 1);
        assert_eq!(
            launcher.pending_sound_at_events.len(),
            1,
            "non-draining frame play should keep events pending until backend consumes them"
        );

        let stats = launcher.drain_and_play_sound_at_audio_frame_with(&mut renderer);
        assert_eq!(stats.sound_at_events, 1);
        assert_eq!(renderer.frames_played, 2);
        assert!(launcher.pending_sound_at_events.is_empty());
    }

    #[test]
    fn desktop_launcher_builds_graphics_frame_without_effect_cache_coupling() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let viewport = RenderViewport::new(0.0, 0.0, 64.0, 64.0);
        let camera = RenderCamera::new(RenderPoint::new(12.0, 8.0), viewport);
        let minimap_camera = MinimapCamera::new(12.0, 8.0, 64.0, 64.0);
        launcher.overlay_renderer_state.set_build_fade(0.5);

        let frame = launcher.graphics_frame_for_render(
            9,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
        );

        assert_eq!(frame.bundle.stats.present_plans, 5);
        assert!(frame.bundle.stats.render_passes > 0);
        assert!(frame.bundle.stats.render_commands > 0);
        assert!(frame.bundle.block_renderer.is_some());
        assert!(frame.bundle.stats.block_tile_passes > 0);
        assert!(frame.bundle.floor_renderer.is_some());
        assert_eq!(frame.bundle.stats.floor_visible_chunks, 1);
        assert!(frame.bundle.stats.floor_stage_plans > 0);
        assert_eq!(frame.bundle.stats.minimap_commands, 1);
        assert!(launcher.standard_local_effect_draw_plans.is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let render_frame = frame.bundle.render_frame.as_ref().unwrap();
        assert_eq!(render_frame.frame_index, 9);
        assert_eq!(render_frame.world_size, RenderSize::new(24.0, 16.0));
        assert_eq!(render_frame.viewport, viewport);

        let overlay_plan = frame.bundle.overlay_renderer.as_ref().unwrap();
        assert_eq!(overlay_plan.build_fade, 0.5);
        assert!(overlay_plan.updated_cores);

        let minimap_plan = frame.bundle.minimap_overlay.as_ref().unwrap();
        assert_eq!(minimap_plan.world_rect.width, 24.0);
        assert_eq!(minimap_plan.world_rect.height, 16.0);

        let floor_plan = frame.bundle.floor_renderer.as_ref().unwrap();
        assert_eq!(floor_plan.visible_chunks.len(), 1);
        assert_eq!(
            floor_plan.stage_plans.len(),
            frame.bundle.stats.floor_stage_plans
        );

        let block_plan = frame.bundle.block_renderer.as_ref().unwrap();
        assert!(!block_plan.is_empty());
        assert_eq!(block_plan.tile_passes.len(), 1);
        assert!(!block_plan.tile_passes[0].tiles.is_empty());
    }

    #[test]
    fn desktop_launcher_graphics_frame_feeds_block_renderer_plan_when_world_and_camera_exist() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();
        let router_id = launcher
            .content_loader
            .block_by_name("router")
            .expect("router should be registered")
            .base()
            .id;
        launcher
            .game_state
            .world
            .tile_mut(1, 1)
            .expect("sample world should contain tile 1,1")
            .block = router_id;

        let viewport = RenderViewport::new(0.0, 0.0, 32.0, 16.0);
        let camera = RenderCamera::new(RenderPoint::new(12.0, 8.0), viewport);
        let minimap_camera = MinimapCamera::new(12.0, 8.0, 32.0, 16.0);

        let frame = launcher.graphics_frame_for_render(
            10,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
        );

        let block_plan = frame.bundle.block_renderer.as_ref().unwrap();
        assert_eq!(block_plan.tile_passes.len(), 1);
        assert_eq!(block_plan.tile_passes[0].stage, BlockDrawStage::TileBase);
        assert_eq!(frame.bundle.stats.block_tile_passes, 1);
        assert_eq!(block_plan.tile_passes[0].tiles.len(), 6);

        let render_frame = frame.bundle.render_frame.as_ref().unwrap();
        let sprite_commands = render_frame
            .passes
            .iter()
            .flat_map(|pass| pass.commands.iter())
            .filter(|command| matches!(command, RenderCommand::DrawSprite { .. }))
            .count();
        assert_eq!(frame.bundle.stats.render_passes, 10);
        assert_eq!(frame.bundle.stats.render_commands, 6);
        assert_eq!(sprite_commands, 6);
        assert_eq!(
            render_frame
                .passes
                .iter()
                .filter(|pass| pass.target.clone()
                    == RenderTarget::Buffer("cache-layer:water:effect".into()))
                .count(),
            1
        );
        let walls_index = render_frame
            .passes
            .iter()
            .position(|pass| {
                pass.kind == RenderPassKind::BlockWalls
                    && pass.target == RenderTarget::Buffer("cache-layer:walls:floor".into())
            })
            .expect("walls cache layer should render in BlockWalls stage");
        let block_index = render_frame
            .passes
            .iter()
            .position(|pass| pass.kind == RenderPassKind::Block)
            .expect("block sprite pass should be present");
        assert!(walls_index < block_index);
        assert_eq!(
            render_frame
                .passes
                .iter()
                .filter(|pass| pass.resolve_kind == Some(RenderResolveKind::ShaderBlit))
                .count(),
            7
        );
        assert!(!frame.floor_chunk_batches.is_empty());
        assert!(frame.minimap_texture_frame.is_some());
        let execution = DesktopGraphicsExecutionSummary::from_frame(&frame);
        assert_eq!(
            execution.floor_chunk_batches,
            frame.floor_chunk_batches.len()
        );
        assert_eq!(execution.minimap_texture_frames, 1);
        assert_eq!(execution.minimap_full_uploads, 1);
        assert_eq!(execution.shader_dispatch_applies, 2);
        assert_eq!(
            frame
                .bundle
                .shader_dispatch
                .as_ref()
                .map(|dispatch| dispatch.applies.len()),
            Some(2)
        );
    }

    #[test]
    fn desktop_launcher_graphics_frame_includes_block_shadow_and_darkness_resolve_passes() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let viewport = RenderViewport::new(0.0, 0.0, 32.0, 16.0);
        let camera = RenderCamera::new(RenderPoint::new(12.0, 8.0), viewport);
        let minimap_camera = MinimapCamera::new(12.0, 8.0, 32.0, 16.0);
        let _ = launcher.graphics_frame_for_render(
            10,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
        );
        launcher
            .block_renderer_state
            .cache
            .shadow_events
            .insert(TileCoord::new(1, 1));
        launcher
            .block_renderer_state
            .cache
            .dark_events
            .insert(TileCoord::new(1, 1));

        let frame = launcher.graphics_frame_for_render(
            11,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
        );
        let render_frame = frame.bundle.render_frame.as_ref().unwrap();
        let shadow = render_frame
            .passes
            .iter()
            .find(|pass| pass.kind == RenderPassKind::BlockShadows)
            .expect("shadow resolve pass should be present");
        let shadow_index = render_frame
            .passes
            .iter()
            .position(|pass| pass.kind == RenderPassKind::BlockShadows)
            .expect("shadow resolve pass should be indexed");
        assert_eq!(shadow.target, RenderTarget::Buffer("block-shadows".into()));
        assert_eq!(shadow.resolve_target, Some(RenderTarget::Screen));
        assert_eq!(shadow.resolve_kind, Some(RenderResolveKind::DrawRectSample));
        assert!(shadow.commands.iter().any(|command| matches!(
            command,
            RenderCommand::DrawSprite { symbol, .. } if symbol == "block-shadow"
        )));

        let darkness = render_frame
            .passes
            .iter()
            .find(|pass| pass.kind == RenderPassKind::Darkness)
            .expect("darkness resolve pass should be present");
        let block_index = render_frame
            .passes
            .iter()
            .position(|pass| pass.kind == RenderPassKind::Block)
            .expect("block sprite pass should be present");
        let walls_index = render_frame
            .passes
            .iter()
            .position(|pass| pass.kind == RenderPassKind::BlockWalls)
            .expect("walls cache layer pass should be present");
        let darkness_index = render_frame
            .passes
            .iter()
            .position(|pass| pass.kind == RenderPassKind::Darkness)
            .expect("darkness resolve pass should be indexed");
        assert!(shadow_index < block_index);
        assert!(shadow_index < walls_index);
        assert!(walls_index < block_index);
        assert!(block_index < darkness_index);
        assert_eq!(
            darkness.target,
            RenderTarget::Buffer("block-darkness".into())
        );
        assert_eq!(darkness.resolve_target, Some(RenderTarget::Screen));
        assert_eq!(
            darkness.resolve_kind,
            Some(RenderResolveKind::DrawFboSample)
        );
        assert!(darkness.commands.iter().any(|command| matches!(
            command,
            RenderCommand::FillRect { rect, color, layer }
                if *rect == RenderRect::new(8.0, 8.0, 8.0, 8.0)
                    && *color == [1.0, 1.0, 1.0, 1.0]
                    && *layer == Layer::DARKNESS
        )));
        assert!(!darkness.commands.iter().any(|command| matches!(
            command,
            RenderCommand::Custom { name, .. } if name == "darkness-dirty-tile"
        )));
    }

    #[test]
    fn desktop_launcher_block_render_plan_uses_world_tile_and_runtime_building_snapshots() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let wall_large = launcher
            .content_loader
            .block_by_name("copper-wall-large")
            .unwrap()
            .base()
            .clone();

        let tile_pos = {
            let world = &mut launcher.game_state.world;
            world.resize(3, 3);
            let tile = world.tile_mut(1, 1).unwrap();
            tile.block = wall_large.id;
            let tile_pos = tile.pos();
            tile.build = Some(mindustry_core::mindustry::world::BuildingRef {
                tile_pos,
                block: wall_large.id,
                team: 7,
                rotation: 2,
            });
            tile_pos
        };

        let mut building = BuildingComp::new(tile_pos, wall_large.clone(), TeamId(7));
        building.rotation = 2;
        building.health = building.max_health * 0.5;
        building.was_visible = true;
        building.was_damaged = true;
        building.visible_flags = 1;
        launcher.runtime.buildings.push(building);

        let viewport = RenderViewport::new(8.0, 8.0, 8.0, 8.0);
        let camera = RenderCamera::new(RenderPoint::new(12.0, 12.0), viewport);
        let plan = launcher.block_render_plan(camera, viewport).unwrap();

        assert_eq!(plan.tile_passes.len(), 1);
        assert_eq!(plan.tile_passes[0].stage, BlockDrawStage::TileBase);
        assert_eq!(plan.tile_passes[0].tiles.len(), 1);

        let tile = &plan.tile_passes[0].tiles[0];
        assert_eq!(tile.coord, TileCoord::new(1, 1));
        assert_eq!(tile.block, "copper-wall-large");
        assert_eq!(tile.cache_layer, CacheLayer::Normal);
        assert!(!tile.draw_custom_shadow);
        assert!(!tile.emits_light);
        assert!(tile.obstructs_light);

        assert_eq!(plan.building_passes[0].stage, BlockDrawStage::BuildingBase);
        let building = &plan.building_passes[0].buildings[0];
        assert_eq!(building.block, "copper-wall-large");
        assert_eq!(building.cache_layer, CacheLayer::Normal);
        assert_eq!(building.size, 2);
        assert_eq!(building.rotation, 2);
        assert_eq!(building.team, 7);
        assert!(building.visible);
        assert!(building.was_visible);
        assert!(building.damaged);
        assert!((building.health_fraction - 0.5).abs() < f32::EPSILON);
        assert_eq!(plan.cracks.len(), 1);
        assert_eq!(plan.cracks[0].region_symbol(), "cracks-2-4");
        assert!(plan
            .building_passes
            .iter()
            .any(|pass| pass.stage == BlockDrawStage::BuildingCracks));
    }

    #[test]
    fn desktop_launcher_block_render_plan_carries_runtime_visual_snapshot_into_building_pass() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let smelter = launcher
            .content_loader
            .block_by_name("silicon-smelter")
            .unwrap()
            .base()
            .clone();

        let tile_pos = {
            let world = &mut launcher.game_state.world;
            world.resize(4, 4);
            let tile = world.tile_mut(1, 1).unwrap();
            tile.block = smelter.id;
            let tile_pos = tile.pos();
            tile.build = Some(mindustry_core::mindustry::world::BuildingRef {
                tile_pos,
                block: smelter.id,
                team: 3,
                rotation: 1,
            });
            tile_pos
        };

        launcher
            .runtime
            .add_building(BuildingComp::new(tile_pos, smelter, TeamId(3)));
        launcher.runtime.crafting_runtime_states.insert(
            tile_pos,
            mindustry_core::mindustry::core::game_runtime::GameRuntimeCraftingBlockState::GenericCrafter(
                mindustry_core::mindustry::world::blocks::production::GenericCrafterState {
                    progress: 0.25,
                    total_progress: 13.0,
                    warmup: 0.5,
                },
            ),
        );

        let viewport = RenderViewport::new(8.0, 8.0, 8.0, 8.0);
        let camera = RenderCamera::new(RenderPoint::new(12.0, 12.0), viewport);
        let plan = launcher.block_render_plan(camera, viewport).unwrap();
        let building = plan
            .building_passes
            .iter()
            .flat_map(|pass| pass.buildings.iter())
            .find(|building| building.coord == TileCoord::new(1, 1))
            .expect("runtime building should enter block renderer plan");
        let visual_runtime = building
            .visual_runtime
            .as_ref()
            .expect("runtime visual snapshot should be attached to building plan");

        assert_eq!(visual_runtime.progress, Some(0.25));
        assert_eq!(visual_runtime.total_progress, Some(13.0));
        assert_eq!(visual_runtime.warmup, Some(0.5));
        assert!(visual_runtime.liquid.is_none());
        let power = visual_runtime
            .power
            .as_ref()
            .expect("smelter power module status should be mirrored into visual runtime");
        assert_eq!(power.status, Some(0.0));
        assert_eq!(power.production_efficiency, None);
        assert!(visual_runtime.turret.is_none());
    }

    #[test]
    fn desktop_launcher_block_render_plan_collects_content_draw_particles() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let concentrator = launcher
            .content_loader
            .block_by_name("atmospheric-concentrator")
            .unwrap()
            .base()
            .clone();

        let tile_pos = {
            let world = &mut launcher.game_state.world;
            world.resize(4, 4);
            let tile = world.tile_mut(1, 1).unwrap();
            tile.block = concentrator.id;
            let tile_pos = tile.pos();
            tile.build = Some(mindustry_core::mindustry::world::BuildingRef {
                tile_pos,
                block: concentrator.id,
                team: 1,
                rotation: 0,
            });
            tile_pos
        };

        launcher
            .runtime
            .add_building(BuildingComp::new(tile_pos, concentrator.clone(), TeamId(1)));
        launcher.runtime.crafting_runtime_states.insert(
            tile_pos,
            mindustry_core::mindustry::core::game_runtime::GameRuntimeCraftingBlockState::GenericCrafter(
                mindustry_core::mindustry::world::blocks::production::GenericCrafterState {
                    progress: 0.1,
                    total_progress: 33.0,
                    warmup: 0.8,
                },
            ),
        );

        let viewport = RenderViewport::new(8.0, 8.0, 8.0, 8.0);
        let camera = RenderCamera::new(RenderPoint::new(12.0, 12.0), viewport);
        let plan = launcher.block_render_plan(camera, viewport).unwrap();

        assert_eq!(plan.block_particles.len(), 1);
        let particle = &plan.block_particles[0];
        assert_eq!(particle.coord, TileCoord::new(1, 1));
        assert_eq!(particle.block, "atmospheric-concentrator");
        assert_eq!(particle.plan.build_id_seed, tile_pos);
        assert_eq!(particle.plan.warmup, 0.8);
        assert_eq!(particle.plan.time, 33.0);
    }

    #[test]
    fn desktop_graphics_trace_reports_block_particle_plans_for_live_backend() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let concentrator = launcher
            .content_loader
            .block_by_name("atmospheric-concentrator")
            .unwrap()
            .base()
            .clone();

        let tile_pos = {
            let world = &mut launcher.game_state.world;
            world.resize(4, 4);
            let tile = world.tile_mut(1, 1).unwrap();
            tile.block = concentrator.id;
            let tile_pos = tile.pos();
            tile.build = Some(mindustry_core::mindustry::world::BuildingRef {
                tile_pos,
                block: concentrator.id,
                team: 1,
                rotation: 0,
            });
            tile_pos
        };

        launcher
            .runtime
            .add_building(BuildingComp::new(tile_pos, concentrator, TeamId(1)));
        launcher.runtime.crafting_runtime_states.insert(
            tile_pos,
            mindustry_core::mindustry::core::game_runtime::GameRuntimeCraftingBlockState::GenericCrafter(
                mindustry_core::mindustry::world::blocks::production::GenericCrafterState {
                    progress: 0.1,
                    total_progress: 33.0,
                    warmup: 0.8,
                },
            ),
        );

        let viewport = RenderViewport::new(8.0, 8.0, 8.0, 8.0);
        let camera = RenderCamera::new(RenderPoint::new(12.0, 12.0), viewport);
        let minimap_camera = MinimapCamera::new(12.0, 12.0, 8.0, 8.0);
        let frame = launcher.graphics_frame_for_render(
            12,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(false),
        );

        assert_eq!(frame.bundle.stats.block_particle_plans, 1);
        let mut renderer = HeadlessDesktopGraphicsRenderer::default();
        let stats = renderer.render_graphics_frame(&frame);

        assert_eq!(stats.block_particle_plans, 1);
        assert_eq!(renderer.last_trace.block_particle_plans, 1);
        assert_eq!(renderer.last_execution.block_particle_plans, 1);
        assert!(renderer.last_trace.block_particle_world_samples > 0);
        assert_eq!(
            renderer.last_trace.block_particle_traces.len(),
            renderer.last_trace.block_particle_world_samples
        );
        assert_eq!(
            renderer.last_trace.block_particle_draw_calls.len(),
            renderer.last_trace.block_particle_world_samples
        );
        assert_eq!(
            renderer
                .last_trace
                .block_particle_traces
                .first()
                .map(|trace| (trace.plan_index, trace.coord, trace.block.as_str())),
            Some((0, TileCoord::new(1, 1), "atmospheric-concentrator"))
        );
        assert_eq!(
            renderer.last_execution.block_particle_world_samples,
            renderer.last_trace.block_particle_world_samples
        );
        assert_eq!(
            renderer.last_execution.block_particle_draw_calls,
            renderer.last_trace.block_particle_draw_calls.len()
        );
        assert_eq!(
            renderer.last_execution.block_particle_render_commands,
            renderer.last_trace.block_particle_render_commands.len()
        );
        let particle_pass = renderer
            .last_trace
            .render_passes
            .iter()
            .find(|pass| pass.commands == renderer.last_trace.block_particle_render_commands)
            .expect("graphics frame should carry block particles as a render pass");
        assert_eq!(particle_pass.kind, RenderPassKind::Block);
        assert_eq!(particle_pass.target, RenderTarget::Screen);
        assert_eq!(
            renderer
                .last_live_backend_state
                .block_particle_traces_emitted,
            renderer.last_trace.block_particle_world_samples
        );
        assert_eq!(
            renderer
                .last_live_backend_state
                .block_particle_draw_calls_emitted,
            renderer.last_trace.block_particle_draw_calls.len()
        );
        assert_eq!(
            renderer
                .last_live_backend_state
                .last_block_particle_trace
                .as_ref()
                .map(|trace| trace.block.as_str()),
            Some("atmospheric-concentrator")
        );
        assert_eq!(
            renderer
                .last_live_backend_state
                .last_block_particle_draw_call
                .as_ref()
                .map(|draw_call| draw_call.block.as_str()),
            Some("atmospheric-concentrator")
        );
        assert!(renderer
            .last_trace
            .execution_steps
            .iter()
            .any(|step| matches!(
                step,
                DesktopGraphicsExecutionStepTrace::BlockParticles { plan_count: 1 }
            )));
    }

    #[test]
    fn desktop_graphics_trace_omits_empty_block_particle_steps() {
        let mut bridge = RenderBridge::new();
        bridge.set_block_renderer(BlockRendererPlan::default());
        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: TextureAtlasPlan::new(),
        };

        let trace = DesktopGraphicsExecutionTrace::from_frame(&frame);

        assert_eq!(trace.block_particle_plans, 0);
        assert_eq!(trace.block_particle_world_samples, 0);
        assert!(trace.block_particle_traces.is_empty());
        assert!(trace.block_particle_draw_calls.is_empty());
        assert!(trace.block_particle_render_commands.is_empty());
        assert!(!trace.execution_steps.iter().any(|step| matches!(
            step,
            DesktopGraphicsExecutionStepTrace::BlockParticles { .. }
        )));
    }

    #[test]
    fn desktop_graphics_trace_preserves_block_particle_order_and_soft_region() {
        let mut regular = mindustry_core::mindustry::world::draw::draw_particles_block_config();
        regular.particle_count = 1;
        regular.particle_radius = 0.0;
        let mut soft = mindustry_core::mindustry::world::draw::draw_soft_particles_block_config();
        soft.particle_count = 1;
        soft.particle_radius = 0.0;
        let mut polygon = mindustry_core::mindustry::world::draw::draw_particles_block_config();
        polygon.particle_count = 1;
        polygon.particle_radius = 0.0;
        polygon.sides = 5;
        polygon.particle_rotation = 15.0;
        polygon.render_kind =
            mindustry_core::mindustry::world::draw::DrawBlockParticleRenderKind::Polygon;

        let mut block_renderer = BlockRendererPlan::default();
        block_renderer.block_particles = vec![
            BlockRendererBlockParticlePlan {
                coord: TileCoord::new(1, 1),
                block: "regular-emitter".into(),
                size: 2,
                plan: ParticleRendererState::block_drawer_particle_plan_from_draw_config(
                    regular,
                    10,
                    1.0,
                    0.0,
                    Layer::BLOCK,
                ),
            },
            BlockRendererBlockParticlePlan {
                coord: TileCoord::new(2, 2),
                block: "soft-emitter".into(),
                size: 2,
                plan: ParticleRendererState::block_drawer_particle_plan_from_draw_config(
                    soft,
                    11,
                    1.0,
                    0.0,
                    Layer::BLOCK,
                ),
            },
            BlockRendererBlockParticlePlan {
                coord: TileCoord::new(3, 3),
                block: "polygon-emitter".into(),
                size: 2,
                plan: ParticleRendererState::block_drawer_particle_plan_from_draw_config(
                    polygon,
                    12,
                    1.0,
                    0.0,
                    Layer::BLOCK,
                ),
            },
        ];

        let mut bridge = RenderBridge::new();
        bridge.set_block_renderer(block_renderer);
        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: TextureAtlasPlan::new(),
        };

        let trace = DesktopGraphicsExecutionTrace::from_frame(&frame);

        assert_eq!(trace.block_particle_plans, 3);
        assert_eq!(trace.block_particle_world_samples, 3);
        assert_eq!(trace.block_particle_traces.len(), 3);
        assert_eq!(trace.block_particle_draw_calls.len(), 3);
        assert_eq!(trace.block_particle_render_commands.len(), 6);
        assert_eq!(
            trace
                .block_particle_traces
                .iter()
                .map(|trace| (trace.plan_index, trace.block.as_str(), trace.sample.region))
                .collect::<Vec<_>>(),
            vec![
                (0, "regular-emitter", None),
                (1, "soft-emitter", Some("circle-shadow")),
                (2, "polygon-emitter", None)
            ]
        );
        assert!(matches!(
            trace.block_particle_draw_calls[0].kind,
            DesktopGraphicsBlockParticleDrawCallKind::Circle
        ));
        assert_eq!(
            trace.block_particle_draw_calls[1].kind,
            DesktopGraphicsBlockParticleDrawCallKind::SoftSprite {
                region: Some("circle-shadow".into())
            }
        );
        assert!(trace.block_particle_draw_calls[1].secondary_color.is_some());
        assert!(trace.block_particle_draw_calls[1].color_t.is_some());
        let soft_tint = trace.block_particle_draw_calls[1].tint();
        assert_eq!(
            trace.block_particle_draw_calls[2].kind,
            DesktopGraphicsBlockParticleDrawCallKind::Polygon {
                sides: 5,
                rotation: 15.0
            }
        );
        assert!(matches!(
            trace.block_particle_render_commands.get(0),
            Some(RenderCommand::SetBlend { mode }) if *mode == RenderBlendMode::Normal
        ));
        assert!(matches!(
            trace.block_particle_render_commands.get(1),
            Some(RenderCommand::DrawCircle { filled: true, .. })
        ));
        assert!(matches!(
            trace.block_particle_render_commands.get(2),
            Some(RenderCommand::SetBlend { mode }) if *mode == RenderBlendMode::Additive
        ));
        assert!(matches!(
            trace.block_particle_render_commands.get(3),
            Some(RenderCommand::DrawSprite { symbol, tint, .. }) if symbol == "circle-shadow" && *tint == soft_tint
        ));
        assert!(matches!(
            trace.block_particle_render_commands.get(4),
            Some(RenderCommand::SetBlend { mode }) if *mode == RenderBlendMode::Normal
        ));
        let polygon_call = &trace.block_particle_draw_calls[2];
        match trace.block_particle_render_commands.get(5) {
            Some(RenderCommand::DrawPolygon {
                center,
                radius,
                sides,
                rotation,
                color,
                filled,
                layer,
            }) => {
                assert_eq!(*center, RenderPoint::new(polygon_call.x, polygon_call.y));
                assert_eq!(*radius, polygon_call.size.max(0.0));
                assert_eq!(*sides, 5);
                assert_eq!(*rotation, 15.0);
                assert_eq!(*color, polygon_call.tint());
                assert!(*filled);
                assert_eq!(*layer, Layer::BLOCK);
            }
            other => panic!("expected DrawPolygon block particle command, got {other:?}"),
        }
        assert!(trace.execution_steps.iter().any(|step| matches!(
            step,
            DesktopGraphicsExecutionStepTrace::BlockParticles { plan_count: 3 }
        )));
    }

    #[test]
    fn desktop_graphics_render_command_sink_includes_block_particle_commands() {
        #[derive(Default)]
        struct RecordingLiveBackendRenderCommandSink {
            traces: Vec<DesktopGraphicsLiveBackendRenderCommandTrace>,
        }

        impl DesktopGraphicsLiveBackendRenderCommandSink for RecordingLiveBackendRenderCommandSink {
            fn consume_render_command_trace(
                &mut self,
                trace: DesktopGraphicsLiveBackendRenderCommandTrace,
            ) {
                self.traces.push(trace);
            }
        }

        let mut regular = mindustry_core::mindustry::world::draw::draw_particles_block_config();
        regular.particle_count = 1;
        regular.particle_radius = 0.0;

        let mut block_renderer = BlockRendererPlan::default();
        block_renderer.block_particles = vec![BlockRendererBlockParticlePlan {
            coord: TileCoord::new(4, 5),
            block: "backend-particle-emitter".into(),
            size: 2,
            plan: ParticleRendererState::block_drawer_particle_plan_from_draw_config(
                regular,
                21,
                1.0,
                0.0,
                Layer::BLOCK,
            ),
        }];

        let mut bridge = RenderBridge::new();
        bridge.set_block_renderer(block_renderer);
        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: TextureAtlasPlan::new(),
        };

        let trace = DesktopGraphicsExecutionTrace::from_frame(&frame);
        let mut sink = RecordingLiveBackendRenderCommandSink::default();
        let state = trace.drive_render_command_sink(&mut sink);

        assert_eq!(trace.block_particle_render_commands.len(), 2);
        assert_eq!(state.render_passes_visited, 0);
        assert_eq!(
            state.backend_render_commands_emitted,
            trace.block_particle_render_commands.len()
        );
        assert_eq!(
            sink.traces.len(),
            trace.block_particle_render_commands.len()
        );
        assert_eq!(
            sink.traces
                .iter()
                .map(|trace| trace.command.clone())
                .collect::<Vec<_>>(),
            trace.block_particle_render_commands
        );
        assert_eq!(
            sink.traces[0].source,
            DesktopGraphicsLiveBackendRenderCommandSource::BlockParticles { command_index: 0 }
        );
        assert_eq!(
            sink.traces[1].source,
            DesktopGraphicsLiveBackendRenderCommandSource::BlockParticles { command_index: 1 }
        );
        assert!(matches!(
            &sink.traces[0].command,
            RenderCommand::SetBlend { mode } if *mode == RenderBlendMode::Normal
        ));
        assert!(matches!(
            &sink.traces[1].command,
            RenderCommand::DrawCircle { filled: true, .. }
        ));
        assert_eq!(
            state.last_backend_render_command.as_ref(),
            sink.traces.last()
        );
    }

    #[test]
    fn desktop_graphics_render_command_sink_matches_execution_step_order_for_particles_and_passes()
    {
        #[derive(Default)]
        struct RecordingLiveBackendRenderCommandSink {
            traces: Vec<DesktopGraphicsLiveBackendRenderCommandTrace>,
        }

        impl DesktopGraphicsLiveBackendRenderCommandSink for RecordingLiveBackendRenderCommandSink {
            fn consume_render_command_trace(
                &mut self,
                trace: DesktopGraphicsLiveBackendRenderCommandTrace,
            ) {
                self.traces.push(trace);
            }
        }

        let mut regular = mindustry_core::mindustry::world::draw::draw_particles_block_config();
        regular.particle_count = 1;
        regular.particle_radius = 0.0;
        let mut block_renderer = BlockRendererPlan::default();
        block_renderer.block_particles = vec![BlockRendererBlockParticlePlan {
            coord: TileCoord::new(6, 7),
            block: "mixed-order-emitter".into(),
            size: 2,
            plan: ParticleRendererState::block_drawer_particle_plan_from_draw_config(
                regular,
                29,
                1.0,
                0.0,
                Layer::BLOCK,
            ),
        }];

        let viewport = RenderViewport::new(0.0, 0.0, 32.0, 32.0);
        let camera = RenderCamera::new(RenderPoint::new(16.0, 16.0), viewport);
        let mut render_frame =
            RenderFramePlan::new(31, RenderSize::new(32.0, 32.0), camera, viewport);
        let mut pass = RenderPass::new(RenderPassKind::Block).with_order(12);
        pass.push(RenderCommand::clear([0.0, 0.0, 0.0, 1.0]));
        pass.push(RenderCommand::draw_text(
            "after-particles",
            RenderPoint::new(1.0, 2.0),
            [1.0, 1.0, 1.0, 1.0],
            8.0,
            0.0,
            RenderTextAlign::Start,
            13.0,
        ));
        render_frame.push_pass(pass);

        let mut bridge = RenderBridge::new();
        bridge.set_block_renderer(block_renderer);
        bridge.set_render_frame(render_frame);
        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: TextureAtlasPlan::new(),
        };

        let trace = DesktopGraphicsExecutionTrace::from_frame(&frame);
        assert!(matches!(
            trace.execution_steps.as_slice(),
            [
                DesktopGraphicsExecutionStepTrace::BlockParticles { .. },
                DesktopGraphicsExecutionStepTrace::RenderPass { .. }
            ]
        ));

        let mut sink = RecordingLiveBackendRenderCommandSink::default();
        let state = trace.drive_render_command_sink(&mut sink);

        assert_eq!(state.render_passes_visited, 1);
        assert_eq!(
            state.backend_render_commands_emitted,
            trace.block_particle_render_commands.len() + trace.render_passes[0].commands.len()
        );
        assert_eq!(
            sink.traces
                .iter()
                .map(|trace| trace.source.clone())
                .collect::<Vec<_>>(),
            vec![
                DesktopGraphicsLiveBackendRenderCommandSource::BlockParticles { command_index: 0 },
                DesktopGraphicsLiveBackendRenderCommandSource::BlockParticles { command_index: 1 },
                DesktopGraphicsLiveBackendRenderCommandSource::RenderPass {
                    pass_index: 0,
                    command_index: 0,
                    pass_kind: RenderPassKind::Block,
                    pass_order: 12,
                    target: RenderTarget::Screen,
                },
                DesktopGraphicsLiveBackendRenderCommandSource::RenderPass {
                    pass_index: 0,
                    command_index: 1,
                    pass_kind: RenderPassKind::Block,
                    pass_order: 12,
                    target: RenderTarget::Screen,
                },
            ]
        );
        assert!(matches!(
            state.last_backend_render_command.as_ref().map(|trace| &trace.command),
            Some(RenderCommand::DrawText { text, .. }) if text == "after-particles"
        ));

        let mut renderer = HeadlessDesktopGraphicsRenderer::default();
        renderer.render_graphics_frame(&frame);
        assert_eq!(
            renderer
                .last_live_backend_state
                .backend_render_commands_emitted,
            state.backend_render_commands_emitted
        );
        assert_eq!(
            renderer
                .last_live_backend_state
                .last_backend_render_command
                .as_ref()
                .map(|trace| trace.source.clone()),
            state
                .last_backend_render_command
                .as_ref()
                .map(|trace| trace.source.clone())
        );
    }

    #[test]
    fn desktop_launcher_default_texture_atlas_contains_block_crack_regions() {
        let launcher = DesktopLauncher::new(Vec::new());

        let router = launcher
            .texture_atlas
            .lookup("router")
            .expect("default desktop atlas should expose base content block sprites");
        assert_eq!(router.page_type, PageType::Main);
        assert_eq!(router.region.source_path, "sprites/blocks/router.png");
        assert_eq!(router.region.width, 1);
        assert_eq!(router.region.height, 1);

        let block_ui = launcher
            .texture_atlas
            .lookup("block-router-ui")
            .expect("default desktop atlas should expose block UI icon candidates");
        assert_eq!(block_ui.page_type, PageType::Ui);
        assert_eq!(
            block_ui.region.source_path,
            "sprites/ui/block-router-ui.png"
        );

        let item_full = launcher
            .texture_atlas
            .lookup("item-copper-full")
            .expect("default desktop atlas should expose item full icon candidates");
        assert_eq!(item_full.page_type, PageType::Main);
        assert_eq!(item_full.region.source_path, "sprites/item-copper-full.png");

        let liquid_ui = launcher
            .texture_atlas
            .lookup("liquid-water-ui")
            .expect("default desktop atlas should expose liquid UI icon candidates");
        assert_eq!(liquid_ui.page_type, PageType::Ui);
        assert_eq!(
            liquid_ui.region.source_path,
            "sprites/ui/liquid-water-ui.png"
        );

        let first = launcher
            .texture_atlas
            .lookup("cracks-1-0")
            .expect("default desktop atlas should expose Java crack region symbols");
        assert_eq!(first.page_type, PageType::Rubble);
        assert_eq!(first.region.source_path, "sprites/rubble/cracks-1-0.png");
        assert_eq!(first.region.width, 1);
        assert_eq!(first.region.height, 1);

        let last = launcher
            .texture_atlas
            .lookup("cracks-7-7")
            .expect("default desktop atlas should include all 7x8 crack regions");
        assert_eq!(last.page_type, PageType::Rubble);
        assert_eq!(last.region.source_path, "sprites/rubble/cracks-7-7.png");

        let crack_count = launcher
            .texture_atlas
            .page(PageType::Rubble)
            .regions
            .iter()
            .filter(|region| region.name.starts_with("cracks-"))
            .count();
        assert_eq!(crack_count, 7 * 8);
    }

    #[test]
    fn desktop_graphics_frame_resolves_default_block_crack_sprite_from_atlas() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let wall_large = launcher
            .content_loader
            .block_by_name("copper-wall-large")
            .unwrap()
            .base()
            .clone();

        let tile_pos = {
            let world = &mut launcher.game_state.world;
            world.resize(3, 3);
            let tile = world.tile_mut(1, 1).unwrap();
            tile.block = wall_large.id;
            tile.build = Some(mindustry_core::mindustry::world::BuildingRef {
                tile_pos: tile.pos(),
                block: wall_large.id,
                team: 7,
                rotation: 0,
            });
            tile.pos()
        };

        let mut building = BuildingComp::new(tile_pos, wall_large, TeamId(7));
        building.health = building.max_health * 0.5;
        building.was_damaged = true;
        building.visible_flags = 1;
        launcher.runtime.buildings.push(building);

        let viewport = RenderViewport::new(8.0, 8.0, 8.0, 8.0);
        let camera = RenderCamera::new(RenderPoint::new(12.0, 12.0), viewport);
        let minimap_camera = MinimapCamera::new(12.0, 12.0, 8.0, 8.0);

        let frame = launcher.graphics_frame_for_render(
            1,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
        );
        let trace = DesktopGraphicsExecutionTrace::from_frame(&frame);
        let sprites = trace
            .render_passes
            .iter()
            .flat_map(|pass| pass.resolved_sprites.iter())
            .collect::<Vec<_>>();
        let base = sprites
            .iter()
            .find(|sprite| sprite.symbol == "copper-wall-large")
            .expect("damaged size-2 wall should emit its base block sprite");
        assert_eq!(base.page_type, Some(PageType::Main));
        assert_eq!(base.page_source_path.as_deref(), Some("sprites.png"));
        assert_eq!(base.region_width, Some(1));
        assert_eq!(base.region_height, Some(1));
        assert!(!base.missing);

        let crack = sprites
            .iter()
            .find(|sprite| sprite.symbol == "cracks-2-4")
            .expect("damaged size-2 wall should emit cracks-2-4 sprite");

        assert_eq!(crack.page_type, Some(PageType::Rubble));
        assert_eq!(crack.page_source_path.as_deref(), Some("sprites4.png"));
        assert_eq!(crack.region_width, Some(1));
        assert_eq!(crack.region_height, Some(1));
        assert!(!crack.missing);
    }

    #[test]
    fn desktop_launcher_merges_mod_resource_plan_into_texture_atlas_without_clobbering_vanilla() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let vanilla_router = launcher.texture_atlas.lookup("router").unwrap();
        assert_eq!(
            vanilla_router.region.source_path,
            "sprites/blocks/router.png"
        );

        let plan = ModResourcePlan::new(false).with_sprite_sources([
            ModSpritePackSource::sprite("example", "mods/example/sprites/router-plus.png"),
            ModSpritePackSource::override_sprite(
                "example",
                "mods/example/sprites-override/router.png",
            ),
            ModSpritePackSource::override_sprite(
                "example",
                "mods/example/sprites-override/ui/block-router-ui.png",
            ),
        ]);

        assert_eq!(
            launcher.merge_mod_resource_plan_into_texture_atlas(&plan),
            3
        );

        let mod_sprite = launcher
            .texture_atlas
            .lookup("example-router-plus")
            .unwrap();
        assert_eq!(mod_sprite.page_type, PageType::Main);
        assert_eq!(
            mod_sprite.region.source_path,
            "mods/example/sprites/router-plus.png"
        );
        assert!(!mod_sprite.region.payload);

        let overridden_router = launcher.texture_atlas.lookup("router").unwrap();
        assert_eq!(overridden_router.page_type, PageType::Main);
        assert_eq!(
            overridden_router.region.source_path,
            "mods/example/sprites-override/router.png"
        );
        assert!(overridden_router.region.payload);

        let overridden_ui = launcher.texture_atlas.lookup("block-router-ui").unwrap();
        assert_eq!(overridden_ui.page_type, PageType::Ui);
        assert_eq!(
            overridden_ui.region.source_path,
            "mods/example/sprites-override/ui/block-router-ui.png"
        );
        assert!(overridden_ui.region.payload);

        assert!(launcher.texture_atlas.lookup("item-copper-full").is_ok());
        assert!(launcher.texture_atlas.lookup("liquid-water-ui").is_ok());
        assert!(launcher.texture_atlas.lookup("cracks-1-0").is_ok());
        assert!(launcher.texture_atlas.lookup("cracks-7-7").is_ok());
        let crack_count = launcher
            .texture_atlas
            .page(PageType::Rubble)
            .regions
            .iter()
            .filter(|region| region.name.starts_with("cracks-"))
            .count();
        assert_eq!(crack_count, 7 * 8);
    }

    fn write_test_png(path: &std::path::Path) {
        const PNG_1X1_TRANSPARENT: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x60, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xE5, 0x27, 0xD4, 0xA2, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        std::fs::write(path, PNG_1X1_TRANSPARENT).expect("test png should be writable");
    }

    fn create_mod_sprite_fixture_root() -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "mindustry-desktop-mod-atlas-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sprites"))
            .expect("mod fixture directories should be creatable");
        std::fs::create_dir_all(root.join("sprites-override/ui"))
            .expect("mod fixture directories should be creatable");

        write_test_png(&root.join("sprites").join("router-plus.png"));
        write_test_png(&root.join("sprites-override").join("router.png"));
        write_test_png(&root.join("sprites-override/ui").join("block-router-ui.png"));

        root
    }

    fn create_mods_container_sprite_fixture_root() -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "mindustry-desktop-mods-container-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let alpha = root.join("alpha");
        let beta = root.join("beta");

        std::fs::create_dir_all(alpha.join("sprites"))
            .expect("alpha mod sprite directory should be creatable");
        std::fs::create_dir_all(beta.join("sprites-override"))
            .expect("beta override sprite directory should be creatable");
        std::fs::create_dir_all(root.join(".git"))
            .expect("hidden container entry should be creatable");
        std::fs::create_dir_all(root.join("sprites"))
            .expect("top-level sprites folder should be creatable");

        write_test_png(&alpha.join("sprites/alpha-router.png"));
        write_test_png(&beta.join("sprites-override/router.png"));
        std::fs::write(alpha.join("mod.hjson"), b"name: alpha")
            .expect("alpha metadata should be writable");
        std::fs::write(beta.join("mod.hjson"), b"name: beta")
            .expect("beta metadata should be writable");
        write_test_png(&root.join("sprites/ignored-root.png"));
        std::fs::write(root.join(".git/HEAD"), b"ref: refs/heads/main")
            .expect("hidden marker should be writable");

        root
    }

    #[test]
    fn desktop_launcher_merges_mod_directory_into_texture_atlas_without_clobbering_vanilla() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let vanilla_router = launcher.texture_atlas.lookup("router").unwrap();
        assert_eq!(
            vanilla_router.region.source_path,
            "sprites/blocks/router.png"
        );

        let root = create_mod_sprite_fixture_root();
        let merge_count = launcher
            .merge_mod_directory_into_texture_atlas("example", false, &root)
            .expect("mod directory should scan and merge");

        assert_eq!(merge_count, 3);

        let mod_sprite = launcher
            .texture_atlas
            .lookup("example-router-plus")
            .unwrap();
        assert_eq!(mod_sprite.page_type, PageType::Main);
        assert_eq!(mod_sprite.region.source_path, "sprites/router-plus.png");
        assert!(!mod_sprite.region.payload);

        let overridden_router = launcher.texture_atlas.lookup("router").unwrap();
        assert_eq!(overridden_router.page_type, PageType::Main);
        assert_eq!(
            overridden_router.region.source_path,
            "sprites-override/router.png"
        );
        assert!(overridden_router.region.payload);

        let overridden_ui = launcher.texture_atlas.lookup("block-router-ui").unwrap();
        assert_eq!(overridden_ui.page_type, PageType::Ui);
        assert_eq!(
            overridden_ui.region.source_path,
            "sprites-override/ui/block-router-ui.png"
        );
        assert!(overridden_ui.region.payload);

        assert!(launcher.texture_atlas.lookup("item-copper-full").is_ok());
        assert!(launcher.texture_atlas.lookup("liquid-water-ui").is_ok());
        assert!(launcher.texture_atlas.lookup("cracks-1-0").is_ok());
        assert!(launcher.texture_atlas.lookup("cracks-7-7").is_ok());
        let crack_count = launcher
            .texture_atlas
            .page(PageType::Rubble)
            .regions
            .iter()
            .filter(|region| region.name.starts_with("cracks-"))
            .count();
        assert_eq!(crack_count, 7 * 8);

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn desktop_launcher_merges_mods_container_into_texture_atlas_explicitly() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let vanilla_router = launcher.texture_atlas.lookup("router").unwrap();
        assert_eq!(
            vanilla_router.region.source_path,
            "sprites/blocks/router.png"
        );

        let root = create_mods_container_sprite_fixture_root();
        let merge_count = launcher
            .merge_mods_directory_into_texture_atlas(&root, false)
            .expect("mods container should scan and merge explicitly");

        assert_eq!(merge_count, 2);
        let alpha = launcher.texture_atlas.lookup("alpha-alpha-router").unwrap();
        assert_eq!(alpha.page_type, PageType::Main);
        assert_eq!(alpha.region.source_path, "sprites/alpha-router.png");
        assert!(!alpha.region.payload);

        let overridden_router = launcher.texture_atlas.lookup("router").unwrap();
        assert_eq!(overridden_router.page_type, PageType::Main);
        assert_eq!(
            overridden_router.region.source_path,
            "sprites-override/router.png"
        );
        assert!(overridden_router.region.payload);
        assert!(launcher.texture_atlas.lookup("ignored-root").is_err());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn desktop_launcher_graphics_frame_carries_pixelator_wrapper_when_enabled() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher.pixelate = true;
        launcher.renderer_scale = 2.75;
        launcher.land_scale = 1.0;

        let viewport = RenderViewport::new(0.0, 0.0, 64.0, 48.0);
        let camera = RenderCamera::new(RenderPoint::new(32.25, 24.75), viewport);
        let minimap_camera = MinimapCamera::new(32.25, 24.75, 64.0, 48.0);

        let frame = launcher.graphics_frame_for_render(
            14,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
        );

        let pixelator = frame.bundle.pixelator.as_ref().unwrap();
        assert_eq!(frame.bundle.stats.pixelator_frames, 1);
        assert_eq!(pixelator.pixel_scale, 2.0);
        assert_eq!(pixelator.buffer_width, 64);
        assert_eq!(pixelator.buffer_height, 48);
        assert_eq!(pixelator.restore.renderer_scale, 2.75);
        assert_eq!(pixelator.restore.camera_x, 32.25);
        assert_eq!(pixelator.restore.camera_y, 24.75);
    }

    #[test]
    fn desktop_launcher_graphics_frame_drains_light_renderer_into_render_pass() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let viewport = RenderViewport::new(0.0, 0.0, 48.0, 48.0);
        let camera = RenderCamera::new(RenderPoint::new(24.0, 24.0), viewport);
        let minimap_camera = MinimapCamera::new(24.0, 24.0, 48.0, 48.0);

        assert!(launcher.light_renderer_state.add_circle(
            12.0,
            16.0,
            8.0,
            LightPrimitive {
                center: (0.0, 0.0),
                radius: 0.0,
                color: DecalColor::from_rgba(0xffcc66ff),
                opacity: 0.5,
            }
        ));

        let frame = launcher.graphics_frame_for_render(
            11,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
        );

        assert_eq!(frame.bundle.stats.present_plans, 3);
        assert_eq!(frame.bundle.stats.render_passes, 1);
        assert_eq!(frame.bundle.stats.render_commands, 1);
        assert!(frame.bundle.floor_renderer.is_none());
        assert!(launcher.light_renderer_state.circle_lights.is_empty());
        assert!(launcher.light_renderer_state.commands.is_empty());

        let render_frame = frame.bundle.render_frame.as_ref().unwrap();
        assert_eq!(render_frame.frame_index, 11);
        assert_eq!(render_frame.passes.len(), 1);
        assert!(render_frame.matches_java_renderer_draw_order());
        assert_eq!(render_frame.passes[0].kind, RenderPassKind::Lighting);

        match &render_frame.passes[0].commands[0] {
            RenderCommand::DrawCircle {
                center,
                radius,
                filled,
                ..
            } => {
                assert_eq!(*center, RenderPoint::new(12.0, 16.0));
                assert_eq!(*radius, 8.0);
                assert!(*filled);
            }
            other => panic!("expected drained circle light command, got {other:?}"),
        }

        let empty_frame = launcher.graphics_frame_for_render(
            12,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
        );
        assert_eq!(empty_frame.bundle.stats.render_passes, 0);
        assert_eq!(empty_frame.bundle.stats.render_commands, 0);
    }

    #[test]
    fn desktop_launcher_graphics_frame_feeds_fog_renderer_when_rules_and_data_exist() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();
        launcher.game_state.rules.fog = true;
        launcher.game_state.rules.static_fog = true;

        let width = launcher.game_state.world.width();
        let height = launcher.game_state.world.height();
        let team = launcher.player.team.0;
        launcher.game_state.fog_control.reset_world(width, height);
        launcher
            .game_state
            .fog_control
            .ensure_data(team)
            .static_data
            .set_range(0, width * height);

        let viewport = RenderViewport::new(0.0, 0.0, 64.0, 64.0);
        let camera = RenderCamera::new(RenderPoint::new(12.0, 8.0), viewport);
        let minimap_camera = MinimapCamera::new(12.0, 8.0, 64.0, 64.0);
        let frame = launcher.graphics_frame_for_render(
            13,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
        );

        assert_eq!(frame.bundle.stats.present_plans, 6);
        assert_eq!(frame.bundle.stats.fog_team_changed_frames, 1);
        assert_eq!(frame.bundle.stats.fog_static_fog_enabled_frames, 1);
        assert!(frame.bundle.stats.fog_stages >= 4);

        let fog_plan = frame.bundle.fog_frame.as_ref().unwrap();
        assert_eq!(fog_plan.viewport.world_width, width as i32);
        assert_eq!(fog_plan.viewport.world_height, height as i32);
        assert_eq!(fog_plan.viewport.tile_size, 8);
        assert!(fog_plan.team_changed);
        assert!(fog_plan.static_fog_enabled);
        let render_frame = frame.bundle.render_frame.as_ref().unwrap();
        let fog_passes = render_frame
            .passes
            .iter()
            .filter(|pass| pass.kind == RenderPassKind::Fog)
            .collect::<Vec<_>>();
        assert_eq!(fog_passes.len(), fog_plan.stages.len());
        assert!(fog_passes.iter().any(|pass| {
            pass.target == RenderTarget::Buffer("fog:dynamic".into())
                && pass.resolve_kind == Some(RenderResolveKind::DrawFboSample)
        }));
        assert!(fog_passes.iter().any(|pass| {
            pass.target == RenderTarget::Buffer("fog:static".into())
                && pass.resolve_kind == Some(RenderResolveKind::DrawFboSample)
        }));
    }

    #[test]
    fn desktop_launcher_renders_graphics_frame_with_headless_renderer_and_resets_overlay_state() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let viewport = RenderViewport::new(0.0, 0.0, 32.0, 32.0);
        let camera = RenderCamera::new(RenderPoint::new(16.0, 16.0), viewport);
        let minimap_camera = MinimapCamera::new(16.0, 16.0, 32.0, 32.0);

        let mut renderer = HeadlessDesktopGraphicsRenderer::default();
        let stats = launcher.render_graphics_frame_with(
            1,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
            &mut renderer,
        );

        assert_eq!(stats.present_plans, 3);
        assert_eq!(stats.floor_stage_plans, 0);
        assert_eq!(renderer.frames_rendered, 1);
        assert_eq!(renderer.last_stats, stats);
        assert_eq!(renderer.last_execution.render_passes_visited, 0);
        assert_eq!(renderer.last_execution.render_commands_visited, 0);
        assert_eq!(renderer.last_execution.overlay_renderer_slots, 1);
        assert_eq!(renderer.last_execution.minimap_overlay_slots, 1);

        launcher.overlay_renderer_state.set_build_fade(0.75);
        launcher.clear_snapshot_apply_cursors();
        let plan = launcher.drain_overlay_renderer_plan();
        assert_eq!(plan.build_fade, 0.0);
        assert!(plan.updated_cores);
    }

    #[test]
    fn desktop_launcher_default_graphics_frame_routes_to_renderer() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let mut renderer = HeadlessDesktopGraphicsRenderer::default();

        let stats = launcher.render_default_graphics_frame_with(7, &mut renderer);

        assert_eq!(renderer.frames_rendered, 1);
        assert_eq!(renderer.last_stats, stats);
        assert_eq!(renderer.last_stats.present_plans, 3);
        assert_eq!(renderer.last_execution.overlay_renderer_slots, 1);
        assert_eq!(renderer.last_execution.minimap_overlay_slots, 1);
        assert_eq!(renderer.last_trace.render_passes.len(), 0);
    }

    #[test]
    fn headless_graphics_renderer_records_execution_summary_without_polluting_stats() {
        let viewport = RenderViewport::new(0.0, 0.0, 64.0, 64.0);
        let camera = RenderCamera::new(RenderPoint::new(32.0, 32.0), viewport);
        let mut render_frame =
            RenderFramePlan::new(77, RenderSize::new(64.0, 64.0), camera, viewport);
        let mut block_pass = RenderPass::new(RenderPassKind::Block)
            .with_target(RenderTarget::Buffer("backend-buffer".into()));
        block_pass.push(RenderCommand::draw_sprite(
            "router",
            RenderRect::new(8.0, 8.0, 8.0, 8.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            30.0,
        ));
        block_pass.push(RenderCommand::draw_text(
            "backend-ready",
            RenderPoint::new(9.0, 10.0),
            [1.0, 1.0, 1.0, 1.0],
            12.0,
            0.0,
            RenderTextAlign::Center,
            31.0,
        ));
        block_pass.push(RenderCommand::fill_rect(
            RenderRect::new(0.0, 0.0, 4.0, 4.0),
            [0.0, 0.0, 0.0, 1.0],
            1.0,
        ));
        block_pass.push(RenderCommand::draw_polygon(
            RenderPoint::new(20.0, 21.0),
            5.0,
            6,
            30.0,
            [0.25, 0.5, 0.75, 1.0],
            true,
            32.0,
        ));
        render_frame.push_pass(block_pass);

        let mut ui_pass = RenderPass::new(RenderPassKind::Ui)
            .with_target(RenderTarget::Texture("ui-layer".into()));
        ui_pass.push(RenderCommand::draw_text(
            "status",
            RenderPoint::new(6.0, 7.0),
            [1.0, 1.0, 1.0, 1.0],
            10.0,
            0.0,
            RenderTextAlign::Start,
            60.0,
        ));
        ui_pass.push(RenderCommand::draw_sprite(
            "cursor",
            RenderRect::new(16.0, 16.0, 6.0, 6.0),
            [0.5, 0.5, 1.0, 1.0],
            0.0,
            61.0,
        ));
        render_frame.push_pass(ui_pass);

        let mut lighting_pass = RenderPass::new(RenderPassKind::Lighting);
        lighting_pass.push(RenderCommand::draw_sprite(
            "lighting-glow",
            RenderRect::new(1.0, 1.0, 2.0, 2.0),
            [1.0, 1.0, 0.5, 1.0],
            15.0,
            50.0,
        ));
        render_frame.push_pass(lighting_pass);

        let mut bridge = RenderBridge::new();
        bridge.set_render_frame(render_frame).set_shader_dispatch(
            ShaderDispatchFrame::from_applies([
                ShaderCatalog::apply_plan(ShaderId::Light, &ShaderApplyContext::default()),
                ShaderApplyPlan::new(ShaderId::Shield),
            ]),
        );
        let atlas = TextureAtlasPlan::from_virtual_source_paths([
            "sprites/router.png",
            "sprites/lighting-glow.png",
        ]);
        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: atlas.clone(),
        };

        let summary = DesktopGraphicsExecutionSummary::from_frame(&frame);
        assert_eq!(summary.render_passes_visited, 3);
        assert_eq!(summary.render_commands_visited, 7);
        assert_eq!(summary.draw_sprite_commands, 3);
        assert_eq!(summary.draw_text_commands, 2);
        assert_eq!(summary.draw_polygon_commands, 1);
        assert_eq!(summary.shader_dispatch_applies, 2);
        assert_eq!(summary.shader_dispatch_operations, 1);
        assert_eq!(summary.shader_dispatch_errors, 0);
        assert_eq!(summary.screen_target_passes, 1);
        assert_eq!(summary.texture_target_passes, 1);
        assert_eq!(summary.buffer_target_passes, 1);
        assert_eq!(frame.bundle.stats.render_passes, 3);
        assert_eq!(frame.bundle.stats.render_commands, 7);

        let mut renderer = HeadlessDesktopGraphicsRenderer::default();
        let stats = renderer.render_graphics_frame(&frame);
        assert_eq!(stats.render_passes, 3);
        assert_eq!(renderer.last_execution, summary);
        assert_eq!(renderer.last_stats.render_commands, 7);
        assert_eq!(
            renderer.last_trace,
            DesktopGraphicsExecutionTrace::from_frame(&frame)
        );
        assert_eq!(renderer.last_trace.shader_dispatch.applies.len(), 2);
        assert_eq!(
            renderer.last_trace.shader_dispatch.applies[0],
            DesktopGraphicsShaderApplyExecutionTrace {
                shader: ShaderId::Light,
                operation_count: 1,
                error_count: 0,
            }
        );
        assert_eq!(
            renderer.last_trace.render_passes[0].target,
            RenderTarget::Buffer("backend-buffer".into())
        );
        assert_eq!(
            renderer.last_trace.render_passes[0].draw_sprite_symbols,
            vec!["router".to_string()]
        );
        assert_eq!(
            renderer.last_trace.render_passes[0].resolved_sprites[0],
            DesktopGraphicsResolvedSpriteTrace {
                symbol: "router".to_string(),
                page_type: Some(mindustry_core::mindustry::graphics::PageType::Main),
                page_source_path: Some("sprites.png".to_string()),
                page_width: Some(4096),
                page_height: Some(4096),
                linear_filter: true,
                sampler: DesktopGraphicsTextureSamplerTrace::Linear,
                region_source_path: Some("sprites/router.png".to_string()),
                x: Some(0),
                y: Some(0),
                u: Some(0.0),
                v: Some(0.0),
                u2: Some(1.0 / 4096.0),
                v2: Some(1.0 / 4096.0),
                region_width: Some(1),
                region_height: Some(1),
                missing: false,
            }
        );
        assert_eq!(
            renderer.last_trace.render_passes[0].draw_texts,
            vec!["backend-ready".to_string()]
        );
        assert_eq!(
            renderer.last_trace.render_passes[0].draw_polygon_sides,
            vec![6]
        );
        assert_eq!(
            renderer.last_trace.render_passes[1].target,
            RenderTarget::Texture("ui-layer".into())
        );
        assert_eq!(
            renderer.last_trace.render_passes[1].draw_texts,
            vec!["status".to_string()]
        );
        assert_eq!(
            renderer.last_trace.render_passes[2].target,
            RenderTarget::Screen
        );
        assert_eq!(renderer.last_live_backend_state.render_passes_visited, 3);
        assert_eq!(renderer.last_live_backend_state.render_commands_visited, 7);
        assert_eq!(
            renderer
                .last_live_backend_state
                .backend_target_events_emitted,
            6
        );
        assert_eq!(
            renderer
                .last_live_backend_state
                .resolve_target_events_emitted,
            0
        );
        assert_eq!(
            renderer
                .last_live_backend_state
                .buffer_target_events_emitted,
            2
        );
        assert_eq!(
            renderer
                .last_live_backend_state
                .texture_target_events_emitted,
            2
        );
        assert_eq!(
            renderer
                .last_live_backend_state
                .screen_target_events_emitted,
            2
        );
        assert_eq!(
            renderer
                .last_live_backend_state
                .last_backend_target_event
                .as_ref()
                .map(|trace| (trace.target.clone(), trace.event, trace.command_count)),
            Some((
                RenderTarget::Screen,
                DesktopGraphicsLiveBackendRenderTargetEventKind::End,
                1,
            ))
        );
        assert_eq!(
            renderer.last_live_backend_state.draw_sprite_traces_emitted,
            3
        );
        let last_state_sprite = renderer
            .last_live_backend_state
            .last_draw_sprite_trace
            .as_ref()
            .and_then(|trace| trace.resolved_sprite.as_ref())
            .expect("live backend execution state should remember last sprite metadata");
        assert_eq!(last_state_sprite.symbol, "lighting-glow");
        assert_eq!(
            last_state_sprite.page_source_path.as_deref(),
            Some("sprites.png")
        );
        assert_eq!(last_state_sprite.page_width, Some(4096));
        assert_eq!(last_state_sprite.page_height, Some(4096));
        assert_eq!(
            last_state_sprite.sampler,
            DesktopGraphicsTextureSamplerTrace::Linear
        );

        let resolved_trace = DesktopGraphicsExecutionTrace::from_frame_with_atlas(&frame, &atlas);
        assert_eq!(
            resolved_trace,
            DesktopGraphicsExecutionTrace::from_frame(&frame)
        );
        let resolved = &resolved_trace.render_passes[0].resolved_sprites[0];
        assert_eq!(resolved.symbol, "router");
        assert_eq!(
            resolved.page_type,
            Some(mindustry_core::mindustry::graphics::PageType::Main)
        );
        assert_eq!(resolved.x, Some(0));
        assert_eq!(resolved.y, Some(0));
        assert_eq!(resolved.page_source_path.as_deref(), Some("sprites.png"));
        assert_eq!(resolved.page_width, Some(4096));
        assert_eq!(resolved.page_height, Some(4096));
        assert!(resolved.linear_filter);
        assert_eq!(resolved.sampler, DesktopGraphicsTextureSamplerTrace::Linear);
        assert_eq!(
            resolved.region_source_path.as_deref(),
            Some("sprites/router.png")
        );
        assert_eq!(resolved.u, Some(0.0));
        assert_eq!(resolved.v, Some(0.0));
        assert_eq!(resolved.u2, Some(1.0 / 4096.0));
        assert_eq!(resolved.v2, Some(1.0 / 4096.0));
        assert_eq!(resolved.region_width, Some(1));
        assert!(!resolved.missing);

        let missing = &resolved_trace.render_passes[1].resolved_sprites[0];
        assert_eq!(missing.symbol, "cursor");
        assert_eq!(missing.page_type, None);
        assert_eq!(missing.page_source_path, None);
        assert_eq!(missing.page_width, None);
        assert_eq!(missing.page_height, None);
        assert!(missing.linear_filter);
        assert_eq!(missing.sampler, DesktopGraphicsTextureSamplerTrace::Linear);
        assert_eq!(missing.region_source_path, None);
        assert_eq!(missing.x, None);
        assert_eq!(missing.y, None);
        assert_eq!(missing.u, None);
        assert_eq!(missing.v, None);
        assert_eq!(missing.u2, None);
        assert_eq!(missing.v2, None);
        assert!(missing.missing);

        let lighting = &resolved_trace.render_passes[2].resolved_sprites[0];
        assert_eq!(lighting.symbol, "lighting-glow");
        assert_eq!(lighting.page_source_path.as_deref(), Some("sprites.png"));
        assert_eq!(lighting.page_width, Some(4096));
        assert_eq!(lighting.page_height, Some(4096));
        assert!(lighting.linear_filter);
        assert_eq!(lighting.sampler, DesktopGraphicsTextureSamplerTrace::Linear);
        assert_eq!(
            lighting.region_source_path.as_deref(),
            Some("sprites/lighting-glow.png")
        );
        assert_eq!(lighting.x, Some(0));
        assert_eq!(lighting.y, Some(0));
        assert_eq!(lighting.u, Some(0.0));
        assert_eq!(lighting.v, Some(0.0));
        assert_eq!(lighting.u2, Some(1.0 / 4096.0));
        assert_eq!(lighting.v2, Some(1.0 / 4096.0));
        assert_eq!(lighting.region_width, Some(1));
        assert_eq!(lighting.region_height, Some(1));
        assert!(!lighting.missing);

        let resolved_summary = DesktopGraphicsExecutionSummary::from_trace(&frame, &resolved_trace);
        assert_eq!(resolved_summary.atlas_resolved_sprites, 2);
        assert_eq!(resolved_summary.atlas_missing_sprites, 1);
    }

    #[test]
    fn headless_graphics_renderer_resolves_draw_sprite_trace_coordinates_from_manual_atlas() {
        let viewport = RenderViewport::new(0.0, 0.0, 64.0, 64.0);
        let camera = RenderCamera::new(RenderPoint::new(32.0, 32.0), viewport);
        let mut render_frame =
            RenderFramePlan::new(88, RenderSize::new(64.0, 64.0), camera, viewport);
        let mut pass = RenderPass::new(RenderPassKind::Ui);
        pass.push(RenderCommand::draw_sprite(
            "router",
            RenderRect::new(8.0, 12.0, 16.0, 20.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            0.0,
        ));
        render_frame.push_pass(pass);

        let mut bridge = RenderBridge::new();
        bridge.set_render_frame(render_frame);

        let mut atlas = TextureAtlasPlan::new();
        let _ = atlas.insert_region(
            mindustry_core::mindustry::graphics::PageType::Main,
            mindustry_core::mindustry::graphics::TextureAtlasRegion::new(
                mindustry_core::mindustry::graphics::PageType::Main,
                "router",
                "sprites/router.png",
                12,
                34,
                8,
                16,
                false,
            ),
        );

        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: atlas,
        };

        let mut renderer = HeadlessDesktopGraphicsRenderer::default();
        renderer.render_graphics_frame(&frame);

        let resolved = &renderer.last_trace.render_passes[0].resolved_sprites[0];
        assert_eq!(resolved.symbol, "router");
        assert_eq!(
            resolved.page_type,
            Some(mindustry_core::mindustry::graphics::PageType::Main)
        );
        assert_eq!(resolved.page_source_path.as_deref(), Some("sprites.png"));
        assert_eq!(resolved.page_width, Some(4096));
        assert_eq!(resolved.page_height, Some(4096));
        assert!(resolved.linear_filter);
        assert_eq!(resolved.sampler, DesktopGraphicsTextureSamplerTrace::Linear);
        assert_eq!(
            resolved.region_source_path.as_deref(),
            Some("sprites/router.png")
        );
        assert_eq!(resolved.x, Some(12));
        assert_eq!(resolved.y, Some(34));
        assert_eq!(resolved.u, Some(12.0 / 4096.0));
        assert_eq!(resolved.v, Some(34.0 / 4096.0));
        assert_eq!(resolved.u2, Some(20.0 / 4096.0));
        assert_eq!(resolved.v2, Some(50.0 / 4096.0));
        assert_eq!(resolved.region_width, Some(8));
        assert_eq!(resolved.region_height, Some(16));
        assert!(!resolved.missing);
    }

    #[test]
    fn headless_graphics_renderer_records_shader_dispatch_before_render_passes() {
        let viewport = RenderViewport::new(0.0, 0.0, 48.0, 48.0);
        let camera = RenderCamera::new(RenderPoint::new(24.0, 24.0), viewport);
        let mut render_frame =
            RenderFramePlan::new(11, RenderSize::new(48.0, 48.0), camera, viewport);

        let mut ui_pass = RenderPass::new(RenderPassKind::Ui).with_order(200);
        ui_pass.push(RenderCommand::draw_sprite(
            "ui-sprite",
            RenderRect::new(16.0, 16.0, 6.0, 6.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            61.0,
        ));
        render_frame.push_pass(ui_pass);

        let mut block_pass = RenderPass::new(RenderPassKind::Block).with_order(100);
        block_pass.push(RenderCommand::draw_sprite(
            "block-sprite",
            RenderRect::new(4.0, 4.0, 8.0, 8.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            20.0,
        ));
        render_frame.push_pass(block_pass);

        let mut bridge = RenderBridge::new();
        bridge.set_render_frame(render_frame).set_shader_dispatch(
            ShaderDispatchFrame::from_applies([
                ShaderCatalog::apply_plan(ShaderId::Light, &ShaderApplyContext::default()),
                ShaderApplyPlan::new(ShaderId::Shield),
            ]),
        );
        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: TextureAtlasPlan::from_virtual_source_paths([
                "sprites/ui-sprite.png",
                "sprites/block-sprite.png",
            ]),
        };

        let mut renderer = HeadlessDesktopGraphicsRenderer::default();
        renderer.render_graphics_frame(&frame);

        assert_eq!(
            renderer.last_trace.execution_steps,
            vec![
                DesktopGraphicsExecutionStepTrace::ShaderDispatch { apply_count: 2 },
                DesktopGraphicsExecutionStepTrace::RenderPass {
                    kind: RenderPassKind::Ui,
                    order: 200,
                    target: RenderTarget::Screen,
                },
                DesktopGraphicsExecutionStepTrace::RenderPass {
                    kind: RenderPassKind::Block,
                    order: 100,
                    target: RenderTarget::Screen,
                },
            ]
        );
        assert_eq!(
            renderer
                .last_trace
                .render_passes
                .iter()
                .map(|pass| pass.draw_sprite_symbols.clone())
                .collect::<Vec<_>>(),
            vec![
                vec!["ui-sprite".to_string()],
                vec!["block-sprite".to_string()],
            ]
        );
        assert_eq!(renderer.last_trace.shader_dispatch.applies.len(), 2);
    }

    #[test]
    fn headless_graphics_renderer_keeps_draw_sprite_order_when_non_sprite_commands_are_interleaved()
    {
        let viewport = RenderViewport::new(0.0, 0.0, 32.0, 32.0);
        let camera = RenderCamera::new(RenderPoint::new(16.0, 16.0), viewport);
        let mut render_frame =
            RenderFramePlan::new(12, RenderSize::new(32.0, 32.0), camera, viewport);

        let mut pass = RenderPass::new(RenderPassKind::Overlay);
        pass.push(RenderCommand::clear([0.0, 0.0, 0.0, 1.0]));
        pass.push(RenderCommand::draw_sprite(
            "alpha",
            RenderRect::new(1.0, 1.0, 4.0, 4.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            10.0,
        ));
        pass.push(RenderCommand::draw_text(
            "status",
            RenderPoint::new(2.0, 3.0),
            [1.0, 1.0, 1.0, 1.0],
            8.0,
            0.0,
            RenderTextAlign::Start,
            11.0,
        ));
        pass.push(RenderCommand::custom(
            "noop",
            vec![mindustry_core::mindustry::graphics::RenderProperty::new(
                "kind", "debug",
            )],
        ));
        pass.push(RenderCommand::draw_polygon(
            RenderPoint::new(6.0, 7.0),
            3.0,
            5,
            45.0,
            [0.8, 0.7, 0.6, 1.0],
            true,
            11.5,
        ));
        pass.push(RenderCommand::draw_sprite(
            "beta",
            RenderRect::new(5.0, 5.0, 4.0, 4.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            12.0,
        ));
        pass.push(RenderCommand::fill_rect(
            RenderRect::new(0.0, 0.0, 2.0, 2.0),
            [0.0, 0.0, 0.0, 1.0],
            13.0,
        ));
        render_frame.push_pass(pass);

        let mut bridge = RenderBridge::new();
        bridge.set_render_frame(render_frame);
        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: TextureAtlasPlan::from_virtual_source_paths([
                "sprites/alpha.png",
                "sprites/beta.png",
            ]),
        };

        let mut renderer = HeadlessDesktopGraphicsRenderer::default();
        renderer.render_graphics_frame(&frame);

        let pass_trace = &renderer.last_trace.render_passes[0];
        assert_eq!(
            pass_trace.command_trace,
            vec![
                DesktopGraphicsCommandExecutionTrace::NoOp {
                    kind: "Clear".to_string(),
                },
                DesktopGraphicsCommandExecutionTrace::DrawSprite {
                    symbol: "alpha".to_string(),
                },
                DesktopGraphicsCommandExecutionTrace::DrawText {
                    text: "status".to_string(),
                },
                DesktopGraphicsCommandExecutionTrace::NoOp {
                    kind: "Custom".to_string(),
                },
                DesktopGraphicsCommandExecutionTrace::DrawPolygon { sides: 5 },
                DesktopGraphicsCommandExecutionTrace::DrawSprite {
                    symbol: "beta".to_string(),
                },
                DesktopGraphicsCommandExecutionTrace::NoOp {
                    kind: "FillRect".to_string(),
                },
            ]
        );
        assert_eq!(
            pass_trace.draw_sprite_symbols,
            vec!["alpha".to_string(), "beta".to_string()]
        );
        assert_eq!(pass_trace.draw_texts, vec!["status".to_string()]);
        assert_eq!(pass_trace.draw_polygon_sides, vec![5]);
    }

    #[test]
    fn desktop_graphics_execution_trace_drives_draw_sprite_sink_with_pass_and_command_order() {
        #[derive(Default)]
        struct RecordingLiveBackendDrawSpriteSink {
            traces: Vec<DesktopGraphicsLiveBackendDrawSpriteTrace>,
        }

        impl DesktopGraphicsLiveBackendDrawSpriteSink for RecordingLiveBackendDrawSpriteSink {
            fn consume_draw_sprite_trace(
                &mut self,
                trace: DesktopGraphicsLiveBackendDrawSpriteTrace,
            ) {
                self.traces.push(trace);
            }
        }

        let viewport = RenderViewport::new(0.0, 0.0, 40.0, 40.0);
        let camera = RenderCamera::new(RenderPoint::new(20.0, 20.0), viewport);
        let mut render_frame =
            RenderFramePlan::new(14, RenderSize::new(40.0, 40.0), camera, viewport);

        let mut first_pass = RenderPass::new(RenderPassKind::Block)
            .with_order(17)
            .with_target(RenderTarget::Buffer("backend-buffer".into()));
        first_pass.push(RenderCommand::clear([0.0, 0.0, 0.0, 1.0]));
        first_pass.push(RenderCommand::draw_sprite(
            "alpha",
            RenderRect::new(1.0, 1.0, 4.0, 4.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            10.0,
        ));
        first_pass.push(RenderCommand::draw_text(
            "status",
            RenderPoint::new(2.0, 3.0),
            [1.0, 1.0, 1.0, 1.0],
            8.0,
            0.0,
            RenderTextAlign::Start,
            11.0,
        ));
        first_pass.push(RenderCommand::custom(
            "noop",
            vec![mindustry_core::mindustry::graphics::RenderProperty::new(
                "kind", "debug",
            )],
        ));
        first_pass.push(RenderCommand::draw_sprite(
            "beta",
            RenderRect::new(5.0, 5.0, 4.0, 4.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            12.0,
        ));
        first_pass.push(RenderCommand::fill_rect(
            RenderRect::new(0.0, 0.0, 2.0, 2.0),
            [0.0, 0.0, 0.0, 1.0],
            13.0,
        ));
        render_frame.push_pass(first_pass);

        let mut second_pass = RenderPass::new(RenderPassKind::Ui).with_order(33);
        second_pass.push(RenderCommand::draw_text(
            "info",
            RenderPoint::new(6.0, 7.0),
            [1.0, 1.0, 1.0, 1.0],
            10.0,
            0.0,
            RenderTextAlign::Center,
            60.0,
        ));
        second_pass.push(RenderCommand::draw_sprite(
            "gamma",
            RenderRect::new(8.0, 8.0, 4.0, 4.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            61.0,
        ));
        second_pass.push(RenderCommand::fill_rect(
            RenderRect::new(1.0, 1.0, 2.0, 2.0),
            [0.0, 0.0, 0.0, 1.0],
            62.0,
        ));
        render_frame.push_pass(second_pass);

        let mut bridge = RenderBridge::new();
        bridge.set_render_frame(render_frame);
        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: TextureAtlasPlan::from_virtual_source_paths([
                "sprites/alpha.png",
                "sprites/beta.png",
                "sprites/gamma.png",
            ])
            .with_linear_filter(false),
        };
        let original_frame = frame.clone();

        let trace = DesktopGraphicsExecutionTrace::from_frame(&frame);
        let mut sink = RecordingLiveBackendDrawSpriteSink::default();
        let state = trace.drive_draw_sprite_sink(&mut sink);

        assert_eq!(state.render_passes_visited, 2);
        assert_eq!(state.render_commands_visited, 9);
        assert_eq!(state.draw_sprite_traces_emitted, 3);
        assert_eq!(sink.traces.len(), 3);
        assert_eq!(frame, original_frame);
        assert_eq!(
            sink.traces[0],
            DesktopGraphicsLiveBackendDrawSpriteTrace {
                pass_index: 0,
                command_index: 1,
                pass_kind: RenderPassKind::Block,
                pass_order: 17,
                target: RenderTarget::Buffer("backend-buffer".into()),
                symbol: "alpha".to_string(),
                resolved_sprite: Some(DesktopGraphicsResolvedSpriteTrace {
                    symbol: "alpha".to_string(),
                    page_type: Some(PageType::Main),
                    page_source_path: Some("sprites.png".to_string()),
                    page_width: Some(4096),
                    page_height: Some(4096),
                    linear_filter: false,
                    sampler: DesktopGraphicsTextureSamplerTrace::Nearest,
                    region_source_path: Some("sprites/alpha.png".to_string()),
                    x: Some(0),
                    y: Some(0),
                    u: Some(0.0),
                    v: Some(0.0),
                    u2: Some(1.0 / 4096.0),
                    v2: Some(1.0 / 4096.0),
                    region_width: Some(1),
                    region_height: Some(1),
                    missing: false,
                }),
            }
        );
        assert_eq!(sink.traces[1].pass_index, 0);
        assert_eq!(sink.traces[1].command_index, 4);
        assert_eq!(sink.traces[1].symbol, "beta".to_string());
        assert_eq!(sink.traces[1].pass_order, 17);
        assert_eq!(
            sink.traces[1].target,
            RenderTarget::Buffer("backend-buffer".into())
        );
        let beta_sprite = sink.traces[1]
            .resolved_sprite
            .as_ref()
            .expect("live backend trace should carry resolved beta sprite");
        assert_eq!(beta_sprite.page_source_path.as_deref(), Some("sprites.png"));
        assert_eq!(beta_sprite.page_width, Some(4096));
        assert_eq!(beta_sprite.page_height, Some(4096));
        assert_eq!(
            beta_sprite.sampler,
            DesktopGraphicsTextureSamplerTrace::Nearest
        );
        assert!(!beta_sprite.linear_filter);
        assert_eq!(state.last_draw_sprite_trace.as_ref(), sink.traces.last());
        assert_eq!(sink.traces[2].pass_index, 1);
        assert_eq!(sink.traces[2].command_index, 1);
        assert_eq!(sink.traces[2].symbol, "gamma".to_string());
        assert_eq!(sink.traces[2].pass_kind, RenderPassKind::Ui);
        assert_eq!(sink.traces[2].pass_order, 33);
        assert_eq!(sink.traces[2].target, RenderTarget::Screen);
    }

    #[test]
    fn desktop_graphics_execution_trace_drives_render_command_sink_with_full_payload() {
        #[derive(Default)]
        struct RecordingLiveBackendRenderCommandSink {
            traces: Vec<DesktopGraphicsLiveBackendRenderCommandTrace>,
        }

        impl DesktopGraphicsLiveBackendRenderCommandSink for RecordingLiveBackendRenderCommandSink {
            fn consume_render_command_trace(
                &mut self,
                trace: DesktopGraphicsLiveBackendRenderCommandTrace,
            ) {
                self.traces.push(trace);
            }
        }

        let viewport = RenderViewport::new(0.0, 0.0, 80.0, 60.0);
        let camera = RenderCamera::new(RenderPoint::new(40.0, 30.0), viewport);
        let mut render_frame =
            RenderFramePlan::new(25, RenderSize::new(80.0, 60.0), camera, viewport);

        let mut world_pass = RenderPass::new(RenderPassKind::Block)
            .with_order(20)
            .with_target(RenderTarget::Buffer("world-buffer".into()));
        let world_commands = vec![
            RenderCommand::clear([0.05, 0.06, 0.07, 1.0]),
            RenderCommand::set_clip(RenderRect::new(1.0, 2.0, 30.0, 20.0)),
            RenderCommand::stroke_rect(
                RenderRect::new(3.0, 4.0, 5.0, 6.0),
                [0.1, 0.2, 0.3, 0.4],
                2.5,
                7.0,
            ),
            RenderCommand::draw_line(
                RenderPoint::new(7.0, 8.0),
                RenderPoint::new(9.0, 10.0),
                1.5,
                [0.4, 0.3, 0.2, 0.1],
                11.0,
            ),
            RenderCommand::draw_polygon(
                RenderPoint::new(10.0, 11.0),
                3.5,
                5,
                22.5,
                [0.7, 0.6, 0.5, 0.4],
                true,
                12.0,
            ),
            RenderCommand::custom(
                "backend-marker",
                vec![RenderProperty::new("stage", "world")],
            ),
        ];
        for command in &world_commands {
            world_pass.push(command.clone());
        }
        render_frame.push_pass(world_pass);

        let mut ui_pass = RenderPass::new(RenderPassKind::Ui).with_order(200);
        let ui_commands = vec![
            RenderCommand::draw_sprite(
                "ui-button",
                RenderRect::new(12.0, 13.0, 14.0, 15.0),
                [1.0, 0.9, 0.8, 0.7],
                45.0,
                201.0,
            ),
            RenderCommand::draw_text(
                "ready",
                RenderPoint::new(16.0, 17.0),
                [0.2, 0.4, 0.6, 1.0],
                18.0,
                0.25,
                RenderTextAlign::Center,
                202.0,
            ),
            RenderCommand::clear_clip(),
        ];
        for command in &ui_commands {
            ui_pass.push(command.clone());
        }
        render_frame.push_pass(ui_pass);

        let mut bridge = RenderBridge::new();
        bridge.set_render_frame(render_frame);
        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: TextureAtlasPlan::from_virtual_source_paths(["sprites/ui-button.png"]),
        };

        let trace = DesktopGraphicsExecutionTrace::from_frame(&frame);
        let mut sink = RecordingLiveBackendRenderCommandSink::default();
        let state = trace.drive_render_command_sink(&mut sink);

        let expected_commands = world_commands
            .iter()
            .chain(ui_commands.iter())
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(state.render_passes_visited, 2);
        assert_eq!(
            state.backend_render_commands_emitted,
            expected_commands.len()
        );
        assert_eq!(sink.traces.len(), expected_commands.len());
        assert_eq!(
            sink.traces
                .iter()
                .map(|trace| trace.kind)
                .collect::<Vec<_>>(),
            vec![
                "Clear",
                "SetClip",
                "StrokeRect",
                "DrawLine",
                "DrawPolygon",
                "Custom",
                "DrawSprite",
                "DrawText",
                "ClearClip"
            ]
        );
        assert_eq!(
            sink.traces
                .iter()
                .map(|trace| trace.command.clone())
                .collect::<Vec<_>>(),
            expected_commands
        );
        assert_eq!(
            sink.traces[0].source,
            DesktopGraphicsLiveBackendRenderCommandSource::RenderPass {
                pass_index: 0,
                command_index: 0,
                pass_kind: RenderPassKind::Block,
                pass_order: 20,
                target: RenderTarget::Buffer("world-buffer".into()),
            }
        );
        assert_eq!(
            sink.traces[6].source,
            DesktopGraphicsLiveBackendRenderCommandSource::RenderPass {
                pass_index: 1,
                command_index: 0,
                pass_kind: RenderPassKind::Ui,
                pass_order: 200,
                target: RenderTarget::Screen,
            }
        );
        assert_eq!(
            state.last_backend_render_command.as_ref(),
            sink.traces.last()
        );
    }

    #[test]
    fn desktop_graphics_execution_trace_drives_render_target_sink_with_begin_end_per_pass() {
        #[derive(Default)]
        struct RecordingLiveBackendRenderTargetSink {
            traces: Vec<DesktopGraphicsLiveBackendRenderTargetTrace>,
        }

        impl DesktopGraphicsLiveBackendRenderTargetSink for RecordingLiveBackendRenderTargetSink {
            fn consume_render_target_trace(
                &mut self,
                trace: DesktopGraphicsLiveBackendRenderTargetTrace,
            ) {
                self.traces.push(trace);
            }
        }

        let viewport = RenderViewport::new(0.0, 0.0, 96.0, 64.0);
        let camera = RenderCamera::new(RenderPoint::new(48.0, 32.0), viewport);
        let mut render_frame =
            RenderFramePlan::new(35, RenderSize::new(96.0, 64.0), camera, viewport);

        let mut block_pass = RenderPass::new(RenderPassKind::Block)
            .with_order(10)
            .with_target(RenderTarget::Buffer("world-buffer".into()));
        block_pass.push(RenderCommand::clear([0.0, 0.0, 0.0, 1.0]));
        block_pass.push(RenderCommand::draw_sprite(
            "router",
            RenderRect::new(1.0, 2.0, 8.0, 8.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            11.0,
        ));
        render_frame.push_pass(block_pass);

        let mut overlay_pass = RenderPass::new(RenderPassKind::Overlay)
            .with_order(20)
            .with_target(RenderTarget::Texture("effect-buffer".into()));
        overlay_pass.push(RenderCommand::set_blend(RenderBlendMode::Additive));
        render_frame.push_pass(overlay_pass);

        let mut ui_pass = RenderPass::new(RenderPassKind::Ui).with_order(30);
        ui_pass.push(RenderCommand::draw_text(
            "ui",
            RenderPoint::new(2.0, 3.0),
            [1.0, 1.0, 1.0, 1.0],
            8.0,
            0.0,
            RenderTextAlign::Center,
            31.0,
        ));
        render_frame.push_pass(ui_pass);

        let mut bridge = RenderBridge::new();
        bridge.set_render_frame(render_frame);
        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: TextureAtlasPlan::from_virtual_source_paths(["sprites/router.png"]),
        };

        let trace = DesktopGraphicsExecutionTrace::from_frame(&frame);
        let mut sink = RecordingLiveBackendRenderTargetSink::default();
        let state = trace.drive_render_target_sink(&mut sink);

        assert_eq!(trace.render_target_traces, sink.traces);
        assert_eq!(state.render_passes_visited, 3);
        assert_eq!(state.backend_target_events_emitted, 6);
        assert_eq!(state.buffer_target_events_emitted, 2);
        assert_eq!(state.texture_target_events_emitted, 2);
        assert_eq!(state.screen_target_events_emitted, 2);
        assert_eq!(
            sink.traces
                .iter()
                .map(|trace| {
                    (
                        trace.pass_index,
                        trace.pass_kind.clone(),
                        trace.pass_order,
                        trace.target.clone(),
                        trace.event,
                        trace.command_count,
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                (
                    0,
                    RenderPassKind::Block,
                    10,
                    RenderTarget::Buffer("world-buffer".into()),
                    DesktopGraphicsLiveBackendRenderTargetEventKind::Begin,
                    2,
                ),
                (
                    0,
                    RenderPassKind::Block,
                    10,
                    RenderTarget::Buffer("world-buffer".into()),
                    DesktopGraphicsLiveBackendRenderTargetEventKind::End,
                    2,
                ),
                (
                    1,
                    RenderPassKind::Overlay,
                    20,
                    RenderTarget::Texture("effect-buffer".into()),
                    DesktopGraphicsLiveBackendRenderTargetEventKind::Begin,
                    1,
                ),
                (
                    1,
                    RenderPassKind::Overlay,
                    20,
                    RenderTarget::Texture("effect-buffer".into()),
                    DesktopGraphicsLiveBackendRenderTargetEventKind::End,
                    1,
                ),
                (
                    2,
                    RenderPassKind::Ui,
                    30,
                    RenderTarget::Screen,
                    DesktopGraphicsLiveBackendRenderTargetEventKind::Begin,
                    1,
                ),
                (
                    2,
                    RenderPassKind::Ui,
                    30,
                    RenderTarget::Screen,
                    DesktopGraphicsLiveBackendRenderTargetEventKind::End,
                    1,
                ),
            ]
        );
        assert_eq!(state.last_backend_target_event.as_ref(), sink.traces.last());
    }

    #[test]
    fn desktop_graphics_render_target_sink_resolves_only_explicit_targets() {
        #[derive(Default)]
        struct RecordingLiveBackendRenderTargetSink {
            traces: Vec<DesktopGraphicsLiveBackendRenderTargetTrace>,
        }

        impl DesktopGraphicsLiveBackendRenderTargetSink for RecordingLiveBackendRenderTargetSink {
            fn consume_render_target_trace(
                &mut self,
                trace: DesktopGraphicsLiveBackendRenderTargetTrace,
            ) {
                self.traces.push(trace);
            }
        }

        let viewport = RenderViewport::new(0.0, 0.0, 80.0, 80.0);
        let camera = RenderCamera::new(RenderPoint::new(40.0, 40.0), viewport);
        let mut render_frame =
            RenderFramePlan::new(36, RenderSize::new(80.0, 80.0), camera, viewport);

        let mut unresolved = RenderPass::new(RenderPassKind::Minimap)
            .with_order(7)
            .with_target(RenderTarget::Texture("minimap-buffer".into()));
        unresolved.push(RenderCommand::draw_sprite(
            "minimap",
            RenderRect::new(0.0, 0.0, 16.0, 16.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            7.0,
        ));
        render_frame.push_pass(unresolved);

        let mut resolved = RenderPass::new(RenderPassKind::Lighting)
            .with_order(8)
            .with_target(RenderTarget::Buffer("effect-buffer".into()))
            .with_resolve(RenderTarget::Screen, RenderResolveKind::ShaderBlit);
        resolved.push(RenderCommand::custom(
            "shader-blit-placeholder",
            vec![RenderProperty::new("shader", "shield")],
        ));
        render_frame.push_pass(resolved);

        let mut bridge = RenderBridge::new();
        bridge.set_render_frame(render_frame);
        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: TextureAtlasPlan::from_virtual_source_paths(["sprites/minimap.png"]),
        };

        let trace = DesktopGraphicsExecutionTrace::from_frame(&frame);
        let mut sink = RecordingLiveBackendRenderTargetSink::default();
        let state = trace.drive_render_target_sink(&mut sink);

        assert_eq!(state.render_passes_visited, 2);
        assert_eq!(state.backend_target_events_emitted, 5);
        assert_eq!(state.resolve_target_events_emitted, 1);
        assert_eq!(
            sink.traces
                .iter()
                .filter(|trace| {
                    trace.event == DesktopGraphicsLiveBackendRenderTargetEventKind::Resolve
                })
                .collect::<Vec<_>>(),
            vec![&DesktopGraphicsLiveBackendRenderTargetTrace {
                pass_index: 1,
                pass_kind: RenderPassKind::Lighting,
                pass_order: 8,
                target: RenderTarget::Buffer("effect-buffer".into()),
                resolve_target: Some(RenderTarget::Screen),
                resolve_kind: Some(RenderResolveKind::ShaderBlit),
                event: DesktopGraphicsLiveBackendRenderTargetEventKind::Resolve,
                command_count: 1,
            }]
        );
        assert_eq!(
            sink.traces
                .iter()
                .filter(|trace| trace.pass_index == 0)
                .map(|trace| trace.event)
                .collect::<Vec<_>>(),
            vec![
                DesktopGraphicsLiveBackendRenderTargetEventKind::Begin,
                DesktopGraphicsLiveBackendRenderTargetEventKind::End,
            ],
            "non-screen targets must not auto-resolve without explicit resolve_target"
        );
        assert_eq!(state.last_backend_target_event.as_ref(), sink.traces.last());
    }

    #[test]
    fn headless_graphics_renderer_does_not_write_back_frame() {
        let viewport = RenderViewport::new(0.0, 0.0, 32.0, 32.0);
        let camera = RenderCamera::new(RenderPoint::new(16.0, 16.0), viewport);
        let mut render_frame =
            RenderFramePlan::new(13, RenderSize::new(32.0, 32.0), camera, viewport);
        let mut pass = RenderPass::new(RenderPassKind::Block);
        pass.push(RenderCommand::draw_sprite(
            "immutable",
            RenderRect::new(8.0, 8.0, 4.0, 4.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            20.0,
        ));
        render_frame.push_pass(pass);

        let mut bridge = RenderBridge::new();
        bridge.set_render_frame(render_frame);
        let frame = DesktopGraphicsFrame {
            bundle: bridge.finish(),
            floor_chunk_batches: Vec::new(),
            minimap_texture_frame: None,
            texture_atlas: TextureAtlasPlan::from_virtual_source_paths(["sprites/immutable.png"]),
        };
        let original_frame = frame.clone();

        let mut renderer = HeadlessDesktopGraphicsRenderer::default();
        let stats = renderer.render_graphics_frame(&frame);

        assert_eq!(frame, original_frame);
        assert_eq!(stats.render_passes, 1);
        assert_eq!(
            renderer.last_trace.render_passes[0].draw_sprite_symbols,
            vec!["immutable".to_string()]
        );
    }

    #[test]
    fn desktop_launcher_copies_texture_atlas_into_graphics_frame() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let atlas = TextureAtlasPlan::from_virtual_source_paths([
            "sprites/router.png",
            "sprites/lighting-glow.png",
        ]);
        launcher.texture_atlas = atlas.clone();

        let viewport = RenderViewport::new(0.0, 0.0, 32.0, 32.0);
        let camera = RenderCamera::new(RenderPoint::new(16.0, 16.0), viewport);
        let minimap_camera = MinimapCamera::new(16.0, 16.0, 32.0, 32.0);

        let frame = launcher.graphics_frame_for_render(
            1,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
        );

        assert_eq!(frame.texture_atlas, atlas);
    }

    #[test]
    fn desktop_launcher_menu_frame_for_render_uses_menu_payload_without_world_bundle() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let frame = launcher.menu_frame_for_render(MenuFrameInput {
            graphics_width: 640.0,
            graphics_height: 360.0,
            scl4: 1.25,
            delta: 0.016,
        });

        assert_eq!(frame.kind, DesktopFrameKind::Menu);
        match frame.payload {
            DesktopFramePayload::Menu(plan) => {
                assert!(!plan.commands.is_empty());
            }
            DesktopFramePayload::World(_) => {
                panic!("menu frame must not contain a world bundle");
            }
            DesktopFramePayload::Load(_) => {
                panic!("menu frame must not use load payload");
            }
        }
    }

    #[test]
    fn desktop_launcher_load_frame_for_render_uses_load_payload_without_world_bundle() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let mut input = LoadFrameInput::new(800.0, 600.0, 1.0, 0.25, LoadStage::Initializing, 0.42);
        input.prompt = Some("initializing".to_string());
        input.completion = Some("done".to_string());

        let frame = launcher.load_frame_for_render(input);

        assert_eq!(frame.kind, DesktopFrameKind::Load);
        match frame.payload {
            DesktopFramePayload::Load(plan) => {
                assert_eq!(plan.stage, LoadStage::Initializing);
                assert!(!plan.commands.is_empty());
            }
            DesktopFramePayload::World(_) => {
                panic!("load frame must not contain a world bundle");
            }
            DesktopFramePayload::Menu(_) => {
                panic!("load frame must not use menu payload");
            }
        }
    }

    #[test]
    fn desktop_launcher_world_graphics_frame_is_not_affected_by_menu_or_load_renderer_state() {
        let viewport = RenderViewport::new(0.0, 0.0, 32.0, 32.0);
        let camera = RenderCamera::new(RenderPoint::new(16.0, 16.0), viewport);
        let minimap_camera = MinimapCamera::new(16.0, 16.0, 32.0, 32.0);

        let mut baseline = DesktopLauncher::new(Vec::new());
        let baseline_frame = baseline.graphics_frame_for_render(
            7,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
        );

        let mut polluted = DesktopLauncher::new(Vec::new());
        let menu_frame = polluted.menu_frame_for_render(MenuFrameInput {
            graphics_width: 640.0,
            graphics_height: 360.0,
            scl4: 1.25,
            delta: 0.5,
        });
        let mut load_input = LoadFrameInput::new(640.0, 360.0, 1.0, 0.5, LoadStage::Failed, 0.75);
        load_input.error = Some("failure".to_string());
        let load_frame = polluted.load_frame_for_render(load_input);

        assert!(matches!(menu_frame.payload, DesktopFramePayload::Menu(_)));
        assert!(matches!(load_frame.payload, DesktopFramePayload::Load(_)));

        let polluted_frame = polluted.graphics_frame_for_render(
            7,
            camera,
            viewport,
            minimap_camera,
            sample_minimap_overlay_input(true),
        );

        assert_eq!(baseline_frame, polluted_frame);
    }

    #[test]
    fn desktop_launcher_standard_effect_draw_updates_ripple_lifetime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("ripple").unwrap() as u16,
                    x: 8.0,
                    y: 16.0,
                    rotation: 2.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        assert_eq!(launcher.materialize_local_effect_events_for_render(), 1);
        assert_eq!(
            launcher
                .runtime
                .client_local_effect_entities
                .get(&-1)
                .unwrap()
                .lifetime,
            30.0
        );

        let plans = launcher.collect_standard_local_effect_draw_plans_for_render();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].kind, StandardEffectDrawKind::StrokedCircle);
        assert_eq!(plans[0].center, (8.0, 16.0));
        assert_eq!(plans[0].color_mul, 1.5);
        assert!((plans[0].radius - 4.0).abs() < 0.0001);
        assert_eq!(launcher.draw_standard_local_effect_states_for_render(), 1);
        assert_eq!(
            launcher
                .runtime
                .client_local_effect_entities
                .get(&-1)
                .unwrap()
                .lifetime,
            60.0
        );

        launcher.standard_local_effect_draw_plans = plans;
        launcher.standard_local_effect_circle_primitives =
            launcher.collect_standard_local_effect_circle_primitives_for_render();
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 1);
        launcher.standard_local_effect_square_primitives =
            launcher.collect_standard_local_effect_square_primitives_for_render();
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        launcher.standard_local_effect_line_primitives =
            launcher.collect_standard_local_effect_line_primitives_for_render();
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        launcher.standard_local_effect_light_primitives =
            launcher.collect_standard_local_effect_light_primitives_for_render();
        assert!(launcher.standard_local_effect_light_primitives.is_empty());
        launcher.clear_snapshot_apply_cursors();
        assert!(launcher.standard_local_effect_draw_plans.is_empty());
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());
    }

    #[test]
    fn desktop_launcher_caches_fire_light_primitives_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("fire").unwrap() as u16,
                    x: 32.0,
                    y: 48.0,
                    rotation: 0.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 1);
        assert_eq!(
            launcher.standard_local_effect_draw_plans[0].kind,
            StandardEffectDrawKind::SeededCircleParticles
        );
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 2);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_light_primitives.len(), 1);
        let frame = launcher.standard_effect_render_frame();
        assert_eq!(frame.draw_plans.len(), 1);
        assert_eq!(frame.circle_primitives.len(), 2);
        assert!(frame.square_primitives.is_empty());
        assert!(frame.line_primitives.is_empty());
        assert_eq!(frame.light_primitives.len(), 1);
        assert_eq!(
            frame.draw_plans[0],
            launcher.standard_local_effect_draw_plans[0]
        );
        assert_eq!(
            frame.circle_primitives,
            launcher.standard_local_effect_circle_primitives
        );
        assert_eq!(
            frame.square_primitives,
            launcher.standard_local_effect_square_primitives
        );
        assert_eq!(
            frame.line_primitives,
            launcher.standard_local_effect_line_primitives
        );
        let light = &launcher.standard_local_effect_light_primitives[0];
        assert_eq!(light.center, (32.0, 48.0));
        assert!((light.radius - 0.8).abs() < 0.0001);
        assert_eq!(light.color, "Pal.lightFlame");
        assert_eq!(light.opacity, 0.5);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(renderer.frames_rendered, 1);
        assert_eq!(
            stats,
            DesktopEffectRenderStats {
                draw_plans: 1,
                circle_primitives: 2,
                square_primitives: 0,
                rect_primitives: 0,
                line_primitives: 0,
                triangle_primitives: 0,
                light_primitives: 1
            }
        );
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_caches_square_and_line_primitives_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("healBlock").unwrap() as u16,
                    x: 16.0,
                    y: 24.0,
                    rotation: 2.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("hitBulletBig").unwrap() as u16,
                    x: 32.0,
                    y: 48.0,
                    rotation: 30.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 2);
        assert_eq!(launcher.standard_local_effect_square_primitives.len(), 1);
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 8);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let square = &launcher.standard_local_effect_square_primitives[0];
        assert_eq!(square.center, (16.0, 24.0));
        assert!(square.stroke > 0.0);

        let line = &launcher.standard_local_effect_line_primitives[0];
        assert!(line.length > 0.0);
        assert!(line.stroke > 0.0);

        let frame = launcher.standard_effect_render_frame();
        assert_eq!(
            frame.square_primitives,
            launcher.standard_local_effect_square_primitives
        );
        assert_eq!(
            frame.line_primitives,
            launcher.standard_local_effect_line_primitives
        );
    }

    #[test]
    fn desktop_launcher_flattens_multi_pass_standard_effects_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("pointShockwave").unwrap() as u16,
                    x: 40.0,
                    y: 64.0,
                    rotation: 32.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 2);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 1);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 8);
        assert!(launcher.standard_local_effect_light_primitives.is_empty());
        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 2);
        assert_eq!(stats.circle_primitives, 1);
        assert_eq!(stats.line_primitives, 8);
        assert_eq!(renderer.frames_rendered, 1);
    }

    #[test]
    fn desktop_launcher_flattens_hit_bullet_multi_pass_with_light_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("hitBulletColor").unwrap() as u16,
                    x: 56.0,
                    y: 72.0,
                    rotation: 0.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 2);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 1);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 5);
        assert_eq!(launcher.standard_local_effect_light_primitives.len(), 1);
        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 2);
        assert_eq!(stats.circle_primitives, 1);
        assert_eq!(stats.line_primitives, 5);
        assert_eq!(stats.light_primitives, 1);
    }

    #[test]
    fn desktop_launcher_flattens_hit_squares_multi_pass_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("hitSquaresColor").unwrap() as u16,
                    x: 56.0,
                    y: 72.0,
                    rotation: 0.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 2);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 1);
        assert_eq!(launcher.standard_local_effect_square_primitives.len(), 5);
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_light_primitives.len(), 1);
        let first_square = &launcher.standard_local_effect_square_primitives[0];
        assert!(first_square.rotation.is_finite());
        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 2);
        assert_eq!(stats.circle_primitives, 1);
        assert_eq!(stats.square_primitives, 5);
        assert_eq!(stats.light_primitives, 1);
    }

    #[test]
    fn desktop_launcher_flattens_square_wave_effect_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("squareWaveEffect").unwrap() as u16,
                    x: 56.0,
                    y: 72.0,
                    rotation: 30.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 1);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_square_primitives.len(), 1);
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_light_primitives.len(), 1);
        let square = &launcher.standard_local_effect_square_primitives[0];
        assert_eq!(square.center, (56.0, 72.0));
        assert!(square.radius > 4.0);
        assert!(square.stroke > 0.0);
        assert!(square.rotation.is_finite());

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(
            stats,
            DesktopEffectRenderStats {
                draw_plans: 1,
                circle_primitives: 0,
                square_primitives: 1,
                rect_primitives: 0,
                line_primitives: 0,
                triangle_primitives: 0,
                light_primitives: 1
            }
        );
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_shoot_triangle_pairs_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("shootSmall").unwrap() as u16,
                    x: 24.0,
                    y: 32.0,
                    rotation: 90.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("shootTitan").unwrap() as u16,
                    x: 40.0,
                    y: 48.0,
                    rotation: 30.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 2);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_triangle_primitives.len(), 4);
        assert!(launcher.standard_local_effect_light_primitives.is_empty());
        let first = launcher
            .standard_local_effect_triangle_primitives
            .iter()
            .find(|triangle| triangle.center == (24.0, 32.0) && triangle.rotation == 90.0)
            .expect("shootSmall front triangle should be cached");
        assert!(first.width > 1.0);
        assert!(first.length > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 2);
        assert_eq!(stats.triangle_primitives, 4);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_shoot_smoke_square_particles_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("shootSmokeSquare", 24.0_f32),
            ("shootSmokeSquareSparse", 40.0_f32),
            ("shootSmokeSquareBig", 56.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 45.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 3);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_square_primitives.len(), 21);
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let rotated_square = launcher
            .standard_local_effect_square_primitives
            .iter()
            .find(|square| square.center.0 != 24.0 && square.rotation != 0.0)
            .expect("shootSmokeSquare should cache offset rotated squares");
        assert!(rotated_square.radius > 0.0);
        assert_eq!(rotated_square.stroke, 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 3);
        assert_eq!(stats.square_primitives, 21);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.line_primitives, 0);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_shoot_smoke_titan_scaled_circles_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("shootSmokeTitan").unwrap() as u16,
                    x: 24.0,
                    y: 32.0,
                    rotation: 30.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 13);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 13);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let smoke_circle = launcher
            .standard_local_effect_circle_primitives
            .iter()
            .find(|circle| circle.center != (24.0, 32.0))
            .expect("shootSmokeTitan should cache offset scaled circles");
        assert_eq!(smoke_circle.kind, StandardEffectDrawKind::FilledCircle);
        assert!(smoke_circle.radius > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 13);
        assert_eq!(stats.circle_primitives, 13);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.line_primitives, 0);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_shoot_smoke_smite_scaled_lines_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("shootSmokeSmite").unwrap() as u16,
                    x: 24.0,
                    y: 32.0,
                    rotation: 30.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 13);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 13);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let smoke_line = launcher
            .standard_local_effect_line_primitives
            .iter()
            .find(|line| line.start != (24.0, 32.0))
            .expect("shootSmokeSmite should cache offset scaled lines");
        assert!(smoke_line.length > 0.0);
        assert!(smoke_line.stroke > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 13);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.line_primitives, 13);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_rand_life_and_payload_driver_lines_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("randLifeSpark", 24.0_f32),
            ("shootPayloadDriver", 40.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 30.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 35);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 35);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let line = launcher
            .standard_local_effect_line_primitives
            .iter()
            .find(|line| line.start != (24.0, 32.0) && line.length > 0.0)
            .expect("randLifeSpark/shootPayloadDriver should cache offset lines");
        assert!(line.stroke > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 35);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.line_primitives, 35);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_color_spark_lines_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("circleColorSpark", 24.0_f32),
            ("colorSpark", 40.0_f32),
            ("colorSparkBig", 56.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 45.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 22);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 22);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let spark_line = launcher
            .standard_local_effect_line_primitives
            .iter()
            .find(|line| line.start.0 != 24.0 && line.start.1 != 32.0)
            .expect("color spark effects should cache offset line primitives");
        assert!(spark_line.length > 0.0);
        assert!(spark_line.stroke > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 22);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.line_primitives, 22);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_spark_lightning_thorium_shoot_lines_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("sparkShoot", 24.0_f32),
            ("lightningShoot", 40.0_f32),
            ("thoriumShoot", 56.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 30.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 3);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 21);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let line = launcher
            .standard_local_effect_line_primitives
            .iter()
            .find(|line| line.start != (24.0, 32.0) && line.length > 0.0)
            .expect("spark/lightning/thorium shoot should cache offset lines");
        assert!(line.stroke > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 3);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.line_primitives, 21);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_rail_and_lancer_charge_primitives_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x, data) in [
            ("railShoot", 24.0_f32, TypeValue::Null),
            ("railTrail", 40.0_f32, TypeValue::Null),
            ("railHit", 56.0_f32, TypeValue::Null),
            ("lancerLaserShoot", 72.0_f32, TypeValue::Null),
            ("lancerLaserShootSmoke", 88.0_f32, TypeValue::Float(42.0)),
            ("lancerLaserCharge", 104.0_f32, TypeValue::Null),
            ("lancerLaserChargeBegin", 120.0_f32, TypeValue::Null),
            ("lightningCharge", 136.0_f32, TypeValue::Null),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 30.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 10);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 3);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 21);
        assert_eq!(launcher.standard_local_effect_triangle_primitives.len(), 10);
        assert_eq!(launcher.standard_local_effect_light_primitives.len(), 1);

        let smoke_plan = launcher
            .standard_local_effect_draw_plans
            .iter()
            .find(|plan| plan.effect_id == standard_effect_id("lancerLaserShootSmoke").unwrap())
            .expect("lancerLaserShootSmoke should preserve data Float length in draw plan");
        assert_eq!(smoke_plan.particles.unwrap().length, 42.0);

        let lightning_triangle = launcher
            .standard_local_effect_triangle_primitives
            .iter()
            .find(|triangle| triangle.center.0 != 136.0 && triangle.width > 1.0)
            .expect("lightningCharge should cache offset seeded triangle primitives");
        assert!(lightning_triangle.length > 1.0);

        let trail_light = launcher
            .standard_local_effect_light_primitives
            .iter()
            .find(|light| light.color == "Pal.orangeSpark")
            .expect("railTrail should cache its orange light");
        assert!(trail_light.radius > 0.0);
        assert_eq!(trail_light.opacity, 0.5);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 10);
        assert_eq!(stats.circle_primitives, 3);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.line_primitives, 21);
        assert_eq!(stats.triangle_primitives, 10);
        assert_eq!(stats.light_primitives, 1);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_casing_rect_primitives_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("casing1", 24.0_f32),
            ("casing2", 40.0_f32),
            ("casing3", 56.0_f32),
            ("casing4", 72.0_f32),
            ("casing2Double", 88.0_f32),
            ("casing3Double", 104.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 30.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 8);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_rect_primitives.len(), 8);
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let filled_rect = launcher
            .standard_local_effect_rect_primitives
            .iter()
            .find(|rect| rect.kind == StandardEffectDrawKind::FilledRect)
            .expect("casing1 should cache a filled rect primitive");
        assert_eq!(filled_rect.region, None);
        assert_eq!(filled_rect.width, 1.0);
        assert_eq!(filled_rect.height, 2.0);
        assert!(filled_rect.rotation.is_finite());

        let textured_rects: Vec<_> = launcher
            .standard_local_effect_rect_primitives
            .iter()
            .filter(|rect| rect.kind == StandardEffectDrawKind::TexturedRect)
            .collect();
        assert_eq!(textured_rects.len(), 7);
        assert!(textured_rects
            .iter()
            .all(|rect| rect.region.as_deref() == Some("casing")));
        assert!(textured_rects
            .iter()
            .any(|rect| rect.width == 3.0 && rect.height == 6.0));

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 8);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.rect_primitives, 8);
        assert_eq!(stats.line_primitives, 0);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_reactor_generation_particles_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("reactorsmoke", 24.0_f32),
            ("redgeneratespark", 40.0_f32),
            ("turbinegenerate", 56.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 0.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 6);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 9);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let generated_circle = launcher
            .standard_local_effect_circle_primitives
            .iter()
            .find(|circle| circle.center.0 != 24.0 && circle.radius > 0.0)
            .expect("generation effects should cache offset circle primitives");
        assert_eq!(generated_circle.kind, StandardEffectDrawKind::FilledCircle);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 6);
        assert_eq!(stats.circle_primitives, 9);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.line_primitives, 0);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_generate_burn_and_pulverize_particles_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("generatespark", 24.0_f32),
            ("fuelburn", 40.0_f32),
            ("incinerateSlag", 56.0_f32),
            ("coreBurn", 72.0_f32),
            ("plasticburn", 88.0_f32),
            ("conveyorPoof", 104.0_f32),
            ("pulverize", 120.0_f32),
            ("pulverizeRed", 136.0_f32),
            ("pulverizeSmall", 152.0_f32),
            ("pulverizeMedium", 168.0_f32),
            ("producesmoke", 184.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 0.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 11);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 28);
        assert_eq!(launcher.standard_local_effect_square_primitives.len(), 26);
        assert!(launcher.standard_local_effect_rect_primitives.is_empty());
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let burn_circle = launcher
            .standard_local_effect_circle_primitives
            .iter()
            .find(|circle| circle.center.0 != 24.0 && circle.radius > 0.0)
            .expect("generate/burn effects should cache offset circle particles");
        assert_eq!(burn_circle.kind, StandardEffectDrawKind::FilledCircle);

        let pulverize_square = launcher
            .standard_local_effect_square_primitives
            .iter()
            .find(|square| square.center.0 != 120.0 && square.rotation == 45.0)
            .expect("pulverize effects should cache rotated square particles");
        assert!(pulverize_square.radius > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 11);
        assert_eq!(stats.circle_primitives, 28);
        assert_eq!(stats.square_primitives, 26);
        assert_eq!(stats.rect_primitives, 0);
        assert_eq!(stats.line_primitives, 0);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_smoke_door_mine_and_teleport_primitives_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("artilleryTrailSmoke", 24.0_f32),
            ("smeltsmoke", 40.0_f32),
            ("coalSmeltsmoke", 56.0_f32),
            ("formsmoke", 72.0_f32),
            ("lava", 88.0_f32),
            ("dooropen", 104.0_f32),
            ("doorclose", 120.0_f32),
            ("dooropenlarge", 136.0_f32),
            ("doorcloselarge", 152.0_f32),
            ("generate", 168.0_f32),
            ("mineWallSmall", 184.0_f32),
            ("mineSmall", 200.0_f32),
            ("mine", 216.0_f32),
            ("mineBig", 232.0_f32),
            ("mineHuge", 248.0_f32),
            ("mineImpact", 264.0_f32),
            ("mineImpactWave", 280.0_f32),
            ("payloadReceive", 296.0_f32),
            ("teleportActivate", 312.0_f32),
            ("teleport", 328.0_f32),
            ("teleportOut", 344.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 32.0,
                        color: type_io::RgbaColor::new(0x336699ff_i32),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 44);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 26);
        assert_eq!(launcher.standard_local_effect_square_primitives.len(), 63);
        assert!(launcher.standard_local_effect_rect_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 90);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let artillery_circle = launcher
            .standard_local_effect_circle_primitives
            .iter()
            .find(|circle| {
                circle.kind == StandardEffectDrawKind::FilledCircle && circle.radius > 0.0
            })
            .expect("artilleryTrailSmoke should cache offset circle particles");
        assert_eq!(artillery_circle.kind, StandardEffectDrawKind::FilledCircle);

        let door_square = launcher
            .standard_local_effect_square_primitives
            .iter()
            .find(|square| square.center.0 == 104.0 && square.stroke > 0.0)
            .expect("door effects should cache stroked square primitives");
        assert_eq!(door_square.rotation, 0.0);

        let teleport_line = launcher
            .standard_local_effect_line_primitives
            .iter()
            .find(|line| line.length > 1.0)
            .expect("teleport/mine wave effects should cache radial line primitives");
        assert!(teleport_line.stroke > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 44);
        assert_eq!(stats.circle_primitives, 26);
        assert_eq!(stats.square_primitives, 63);
        assert_eq!(stats.rect_primitives, 0);
        assert_eq!(stats.line_primitives, 90);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_launch_pod_circle_and_lines_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("launchPod", 24.0_f32),
            ("coreLandDust", 40.0_f32),
            ("podLandDust", 56.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 30.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 4);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 3);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_rect_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 24);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let circle = launcher
            .standard_local_effect_circle_primitives
            .iter()
            .find(|circle| circle.kind == StandardEffectDrawKind::StrokedCircle)
            .expect("launchPod should cache a stroked scaled circle");
        assert_eq!(circle.kind, StandardEffectDrawKind::StrokedCircle);
        assert_eq!(
            circle.color,
            Some(mindustry_core::mindustry::entities::comp::DecalColor::from_rgba(0xffbb64ff))
        );

        let line = &launcher.standard_local_effect_line_primitives[0];
        assert!(line.length > 1.0);
        assert!(line.stroke > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 4);
        assert_eq!(stats.circle_primitives, 3);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.rect_primitives, 0);
        assert_eq!(stats.line_primitives, 24);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_shield_break_polygon_lines_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("shieldBreak").unwrap() as u16,
                    x: 24.0,
                    y: 32.0,
                    rotation: 12.0,
                    color: type_io::RgbaColor::new(0x336699ff_i32),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 6);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_rect_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 6);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let line = &launcher.standard_local_effect_line_primitives[0];
        assert!(line.length > 0.0);
        assert!(line.stroke > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 6);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.rect_primitives, 0);
        assert_eq!(stats.line_primitives, 6);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_chain_effect_vec2_data_lines_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, y) in [("chainLightning", 32.0_f32), ("chainEmp", 48.0_f32)] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x: 24.0,
                        y,
                        rotation: 0.0,
                        color: type_io::RgbaColor::new(0x336699ff_i32),
                    },
                    data: TypeValue::Vec2(IoVec2::new(54.0, y)),
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 10);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_rect_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 10);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let line = &launcher.standard_local_effect_line_primitives[0];
        assert!(line.length > 0.0);
        assert!(line.stroke > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 10);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.rect_primitives, 0);
        assert_eq!(stats.line_primitives, 10);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_debug_line_vec2_array_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("debugLine").unwrap() as u16,
                    x: 24.0,
                    y: 32.0,
                    rotation: 0.0,
                    color: type_io::RgbaColor::new(0x336699ff_i32),
                },
                data: TypeValue::Vec2Array(vec![
                    IoVec2::new(24.0, 32.0),
                    IoVec2::new(54.0, 32.0),
                    IoVec2::new(54.0, 72.0),
                ]),
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 2);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_rect_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 2);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let first = &launcher.standard_local_effect_line_primitives[0];
        assert_eq!(first.start, (24.0, 32.0));
        assert_eq!(first.length, 30.0);
        assert_eq!(first.stroke, 2.0);
        assert_eq!(
            first.color,
            Some(mindustry_core::mindustry::entities::comp::DecalColor::from_rgba(0x336699ff))
        );

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 2);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.rect_primitives, 0);
        assert_eq!(stats.line_primitives, 2);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_debug_rect_data_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("debugRect").unwrap() as u16,
                    x: 24.0,
                    y: 32.0,
                    rotation: 0.0,
                    color: type_io::RgbaColor::new(0x336699ff_i32),
                },
                data: TypeValue::Rect(mindustry_core::mindustry::entities::Rect::new(
                    24.0, 32.0, 30.0, 40.0,
                )),
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 4);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_rect_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 4);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let first = &launcher.standard_local_effect_line_primitives[0];
        assert_eq!(first.start, (24.0, 32.0));
        assert_eq!(first.length, 30.0);
        assert_eq!(first.stroke, 2.0);
        assert_eq!(
            first.color,
            Some(mindustry_core::mindustry::entities::comp::DecalColor::from_rgba(0x336699ff))
        );

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 4);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.rect_primitives, 0);
        assert_eq!(stats.line_primitives, 4);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_resolves_heal_block_full_icon_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let block = launcher
            .content_loader
            .block_by_name("duo")
            .expect("base content should include duo")
            .base()
            .clone();
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("healBlockFull").unwrap() as u16,
                    x: 24.0,
                    y: 32.0,
                    rotation: 0.0,
                    color: type_io::RgbaColor::new(0x33cc66ff_i32),
                },
                data: TypeValue::Content(type_io::ContentRef::new(ContentType::Block, block.id)),
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 1);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_rect_primitives.len(), 1);
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let rect = &launcher.standard_local_effect_rect_primitives[0];
        assert_eq!(rect.kind, StandardEffectDrawKind::TexturedRect);
        assert_eq!(rect.center, (24.0, 32.0));
        assert_eq!(
            rect.width,
            block.size as f32 * mindustry_core::mindustry::vars::TILE_SIZE as f32
        );
        assert_eq!(rect.height, rect.width);
        let expected_region = format!("block-fullIcon:{}", block.id);
        assert_eq!(rect.region.as_deref(), Some(expected_region.as_str()));
        let rect_color = rect.color.expect("heal block full should mix input color");
        let input_color =
            mindustry_core::mindustry::entities::comp::DecalColor::from_rgba(0x33cc66ff);
        assert_eq!(rect_color.r, input_color.r);
        assert_eq!(rect_color.g, input_color.g);
        assert_eq!(rect_color.b, input_color.b);
        assert!(rect_color.a > 0.0 && rect_color.a <= 1.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 1);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.rect_primitives, 1);
        assert_eq!(stats.line_primitives, 0);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_resolves_arc_shield_break_ability_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let tecta = launcher
            .content_loader
            .unit_by_name("tecta")
            .expect("base content should include tecta")
            .clone();
        let mut unit = UnitComp::new(778, tecta, TeamId(TEAM_SHARDED));
        unit.set_pos(100.0, 200.0);
        unit.set_rotation(90.0);
        launcher
            .runtime
            .client_unit_snapshot_entities
            .insert(778, unit);
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("arcShieldBreak").unwrap() as u16,
                    x: 100.0,
                    y: 200.0,
                    rotation: 90.0,
                    color: type_io::RgbaColor::new(0x66ccffff_i32),
                },
                data: TypeValue::Unit(778),
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 16);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_rect_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 16);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let line = &launcher.standard_local_effect_line_primitives[0];
        assert!(line.start.0.is_finite());
        assert!(line.start.1.is_finite());
        assert!(line.length > 0.0);
        assert!(line.stroke > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 16);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.rect_primitives, 0);
        assert_eq!(stats.line_primitives, 16);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_leg_destroy_textured_line_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("legDestroy").unwrap() as u16,
                    x: 24.0,
                    y: 32.0,
                    rotation: 0.0,
                    color: type_io::RgbaColor::new(0xffffffff_u32 as i32),
                },
                data: TypeValue::LegDestroyData(LegDestroyData::new(
                    IoVec2::new(24.0, 32.0),
                    IoVec2::new(54.0, 32.0),
                    TextureRegionRef::with_size("crawler-leg", 16, 8),
                )),
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 1);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_rect_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 1);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let line = &launcher.standard_local_effect_line_primitives[0];
        assert_eq!(line.region.as_deref(), Some("crawler-leg"));
        assert!((line.length - 30.0).abs() < 0.0001);
        assert_eq!(line.stroke, 8.0);
        assert!(line.alpha > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 1);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.rect_primitives, 0);
        assert_eq!(stats.line_primitives, 1);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_resolves_unit_shield_break_hit_size_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let flare = launcher
            .content_loader
            .unit_by_name("flare")
            .unwrap()
            .clone();
        let mut unit = UnitComp::new(777, flare, TeamId(TEAM_SHARDED));
        unit.hitbox.hit_size = 10.0;
        launcher
            .runtime
            .client_unit_snapshot_entities
            .insert(777, unit);
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("unitShieldBreak").unwrap() as u16,
                    x: 24.0,
                    y: 32.0,
                    rotation: 0.0,
                    color: type_io::RgbaColor::new(0x336699ff_i32),
                },
                data: TypeValue::Unit(777),
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 2);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 1);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_rect_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 15);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let circle = &launcher.standard_local_effect_circle_primitives[0];
        assert_eq!(circle.kind, StandardEffectDrawKind::StrokedCircle);
        assert_eq!(circle.radius, 13.0);
        let circle_color = circle
            .color
            .expect("unit shield circle should keep input color");
        let input_color =
            mindustry_core::mindustry::entities::comp::DecalColor::from_rgba(0x336699ff);
        assert_eq!(circle_color.r, input_color.r);
        assert_eq!(circle_color.g, input_color.g);
        assert_eq!(circle_color.b, input_color.b);
        assert!(circle_color.a > 0.0);
        assert!(circle_color.a <= 0.9);
        assert!(launcher.standard_local_effect_line_primitives[0].length > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 2);
        assert_eq!(stats.circle_primitives, 1);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.rect_primitives, 0);
        assert_eq!(stats.line_primitives, 15);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_shoot_flame_circle_particles_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("shootSmallFlame", 24.0_f32),
            ("shootPyraFlame", 40.0_f32),
            ("shootLiquid", 56.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 45.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 3);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 27);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let flame_circle = launcher
            .standard_local_effect_circle_primitives
            .iter()
            .find(|circle| circle.center.0 != 24.0 && circle.radius > 0.0)
            .expect("shoot flame effects should cache offset circle primitives");
        assert_eq!(flame_circle.kind, StandardEffectDrawKind::FilledCircle);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 3);
        assert_eq!(stats.circle_primitives, 27);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.line_primitives, 0);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_shoot_smoke_missile_scaled_circles_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("shootSmokeMissile", 24.0_f32),
            ("shootSmokeMissileColor", 40.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 30.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 70);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 70);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let smoke_circle = launcher
            .standard_local_effect_circle_primitives
            .iter()
            .find(|circle| circle.center.0 != 24.0 && circle.alpha == 0.5)
            .expect("shootSmokeMissile should cache offset half-alpha circles");
        assert_eq!(smoke_circle.kind, StandardEffectDrawKind::FilledCircle);
        assert!(smoke_circle.radius > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 70);
        assert_eq!(stats.circle_primitives, 70);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.line_primitives, 0);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_regen_particles_square_and_lines_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("regenParticle", 24.0_f32),
            ("regenSuppressParticle", 40.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 0.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 2);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_square_primitives.len(), 1);
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 4);
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let regen_square = &launcher.standard_local_effect_square_primitives[0];
        assert_eq!(regen_square.center, (24.0, 32.0));
        assert_eq!(regen_square.rotation, 45.0);
        assert!(regen_square.radius > 0.0);

        let suppress_line = &launcher.standard_local_effect_line_primitives[0];
        assert!(suppress_line.length > 0.0);
        assert!(suppress_line.stroke > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 2);
        assert_eq!(stats.circle_primitives, 0);
        assert_eq!(stats.square_primitives, 1);
        assert_eq!(stats.line_primitives, 4);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_reactor_and_neoplasia_smoke_circles_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        for (name, x) in [
            ("surgeCruciSmoke", 24.0_f32),
            ("neoplasiaSmoke", 40.0_f32),
            ("heatReactorSmoke", 56.0_f32),
        ] {
            launcher
                .runtime
                .client_local_effect_events
                .push(EffectCallPacket2 {
                    effect: EffectCallPacket {
                        effect_id: standard_effect_id(name).unwrap() as u16,
                        x,
                        y: 32.0,
                        rotation: 30.0,
                        color: type_io::RgbaColor::new(-1),
                    },
                    data: TypeValue::Null,
                });
        }

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 14);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 14);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert!(launcher
            .standard_local_effect_triangle_primitives
            .is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let smoke_circle = launcher
            .standard_local_effect_circle_primitives
            .iter()
            .find(|circle| circle.center.0 != 24.0 && circle.alpha <= 0.9)
            .expect("smoke effects should cache offset circles");
        assert_eq!(smoke_circle.kind, StandardEffectDrawKind::FilledCircle);
        assert!(smoke_circle.radius > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 14);
        assert_eq!(stats.circle_primitives, 14);
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.line_primitives, 0);
        assert_eq!(stats.triangle_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_inst_bomb_and_trail_triangles_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("instBomb").unwrap() as u16,
                    x: 20.0,
                    y: 28.0,
                    rotation: 0.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("instTrail").unwrap() as u16,
                    x: 40.0,
                    y: 48.0,
                    rotation: 30.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 5);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 1);
        assert_eq!(launcher.standard_local_effect_triangle_primitives.len(), 12);
        assert_eq!(launcher.standard_local_effect_light_primitives.len(), 2);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_line_primitives.is_empty());

        let inst_bomb_triangle = launcher
            .standard_local_effect_triangle_primitives
            .iter()
            .find(|triangle| triangle.center == (20.0, 28.0) && triangle.rotation == 45.0)
            .expect("instBomb fan triangle should be cached");
        assert_eq!(inst_bomb_triangle.width, 6.0);
        assert!(inst_bomb_triangle.length > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 5);
        assert_eq!(stats.circle_primitives, 1);
        assert_eq!(stats.triangle_primitives, 12);
        assert_eq!(stats.light_primitives, 2);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_inst_shoot_scaled_circle_and_triangles_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("instShoot").unwrap() as u16,
                    x: 32.0,
                    y: 40.0,
                    rotation: 30.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 3);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 1);
        assert_eq!(launcher.standard_local_effect_triangle_primitives.len(), 4);
        assert_eq!(launcher.standard_local_effect_light_primitives.len(), 1);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_line_primitives.is_empty());

        let side = launcher
            .standard_local_effect_triangle_primitives
            .iter()
            .find(|triangle| triangle.center == (32.0, 40.0) && triangle.rotation == -60.0)
            .expect("instShoot side triangle should be cached");
        assert_eq!(side.length, 85.0);
        assert!(side.width > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 3);
        assert_eq!(stats.circle_primitives, 1);
        assert_eq!(stats.triangle_primitives, 4);
        assert_eq!(stats.light_primitives, 1);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_shoot_scepter_secondary_triangles_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("shootScepterSecondary").unwrap() as u16,
                    x: 32.0,
                    y: 40.0,
                    rotation: 30.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 2);
        assert!(launcher.standard_local_effect_circle_primitives.is_empty());
        assert_eq!(launcher.standard_local_effect_triangle_primitives.len(), 4);
        assert!(launcher.standard_local_effect_light_primitives.is_empty());
        let side = launcher
            .standard_local_effect_triangle_primitives
            .iter()
            .find(|triangle| triangle.center == (32.0, 40.0) && triangle.rotation == -60.0)
            .expect("shootScepterSecondary side triangle should be cached");
        assert!(side.width > 0.0);
        assert!(side.length > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 2);
        assert_eq!(stats.triangle_primitives, 4);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_shoot_quell_pulse_circles_and_triangle_clusters_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("shootQuellPulse").unwrap() as u16,
                    x: 32.0,
                    y: 40.0,
                    rotation: 0.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert!((27..=32).contains(&launcher.standard_local_effect_draw_plans.len()));
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 10);
        assert!(launcher.standard_local_effect_square_primitives.is_empty());
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert!((34..=44).contains(&launcher.standard_local_effect_triangle_primitives.len()));
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let core_circle = launcher
            .standard_local_effect_circle_primitives
            .iter()
            .find(|circle| circle.center == (32.0, 40.0) && circle.stroke > 2.5)
            .expect("shootQuellPulse core circle should be cached");
        assert!(core_circle.radius > 0.0);
        assert!(core_circle.alpha > 0.8);

        let offset_triangle = launcher
            .standard_local_effect_triangle_primitives
            .iter()
            .find(|triangle| triangle.center != (32.0, 40.0))
            .expect("shootQuellPulse should cache offset triangle clusters");
        assert!(offset_triangle.width > 0.0);
        assert!(offset_triangle.length > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(
            stats.draw_plans,
            launcher.standard_local_effect_draw_plans.len()
        );
        assert_eq!(stats.circle_primitives, 10);
        assert_eq!(
            stats.triangle_primitives,
            launcher.standard_local_effect_triangle_primitives.len()
        );
        assert_eq!(stats.square_primitives, 0);
        assert_eq!(stats.line_primitives, 0);
        assert_eq!(stats.light_primitives, 0);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_flattens_inst_hit_triangles_circle_and_squares_for_render() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_effect_events
            .push(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: standard_effect_id("instHit").unwrap() as u16,
                    x: 32.0,
                    y: 40.0,
                    rotation: 30.0,
                    color: type_io::RgbaColor::new(-1),
                },
                data: TypeValue::Null,
            });

        launcher.update();

        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 12);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 1);
        assert_eq!(launcher.standard_local_effect_triangle_primitives.len(), 20);
        assert_eq!(launcher.standard_local_effect_square_primitives.len(), 25);
        assert!(launcher.standard_local_effect_line_primitives.is_empty());
        assert!(launcher.standard_local_effect_light_primitives.is_empty());

        let first_square = &launcher.standard_local_effect_square_primitives[0];
        assert_eq!(first_square.rotation, 45.0);
        assert!(first_square.radius > 0.0);

        let mut renderer = HeadlessDesktopEffectRenderer::default();
        let stats = launcher.render_standard_effect_frame_with(&mut renderer);
        assert_eq!(stats.draw_plans, 12);
        assert_eq!(stats.circle_primitives, 1);
        assert_eq!(stats.triangle_primitives, 20);
        assert_eq!(stats.square_primitives, 25);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_syncs_assembler_unit_spawned_packet_to_runtime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let tile_pos = mindustry_core::mindustry::world::point2_pack(9, 7);
        let assembler_def = launcher
            .content_loader
            .block_by_name("tank-assembler")
            .unwrap();
        let stell = launcher.content_loader.unit_by_name("stell").unwrap();
        let large_wall = launcher
            .content_loader
            .block_by_name("tungsten-wall-large")
            .unwrap();
        let router = launcher.content_loader.block_by_name("router").unwrap();
        launcher.runtime.add_building(BuildingComp::new(
            tile_pos,
            assembler_def.base().clone(),
            TeamId(4),
        ));
        let mut blocks = PayloadSeq::new();
        blocks.add(PayloadKey::new(ContentType::Unit, stell.id()), 4);
        blocks.add(
            PayloadKey::new(ContentType::Block, large_wall.base().id),
            10,
        );
        blocks.add(PayloadKey::new(ContentType::Block, router.base().id), 2);
        launcher.runtime.unit_runtime_states.insert(
            tile_pos,
            GameRuntimeUnitBlockState::Assembler {
                common: PayloadBlockBuildState::default(),
                assembler: UnitAssemblerState {
                    progress: 1.2,
                    blocks,
                    ..UnitAssemblerState::default()
                },
            },
        );

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::AssemblerUnitSpawnedCallPacket(
                AssemblerUnitSpawnedCallPacket {
                    tile: Some(tile_pos),
                },
            ));
        }
        launcher.update();

        let Some(GameRuntimeUnitBlockState::Assembler { assembler, .. }) =
            launcher.runtime.unit_runtime_states.get(&tile_pos)
        else {
            panic!("unit assembler state should remain present");
        };
        assert_eq!(assembler.progress, 0.0);
        assert_eq!(assembler.blocks.total(), 0);
        assert!(launcher.runtime.client_local_sound_at_events.is_empty());
        assert_eq!(launcher.pending_sound_at_events.len(), 1);
        assert!(launcher.runtime.client_local_effect_events.is_empty());
        assert_eq!(launcher.runtime.client_local_effect_entities.len(), 1);
        assert_eq!(launcher.last_applied_unit_lifecycle_packets_seen, 1);
    }

    #[test]
    fn desktop_launcher_ticks_elude_move_effect_to_local_effect_queue() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let elude = launcher
            .content_loader
            .unit_by_name("elude")
            .unwrap()
            .clone();
        let mut unit = UnitComp::new(4501, elude, TeamId(TEAM_SHARDED));
        unit.set_pos(100.0, 200.0);
        unit.set_rotation(0.0);
        unit.vel.vel.x = 1.0;
        unit.abilities[0].data = 3.0;
        launcher
            .runtime
            .client_unit_snapshot_entities
            .insert(unit.id(), unit);

        launcher.update();

        assert!(launcher.runtime.client_local_effect_events.is_empty());
        assert_eq!(launcher.runtime.client_local_effect_entities.len(), 1);
        let effect = launcher
            .runtime
            .client_local_effect_entities
            .get(&-1)
            .unwrap();
        assert_eq!(
            effect.effect_id,
            Some(standard_effect_id("missileTrailShort").unwrap() as u16)
        );
        assert!((effect.x - 93.0).abs() < 0.0001);
        assert!((effect.y - 200.0).abs() < 0.0001);
        assert_eq!(effect.lifetime, 22.0);
        assert_eq!(effect.effect_clip, 50.0);
        assert_eq!(effect.data, TypeValue::Null);
        assert_eq!(launcher.standard_local_effect_draw_plans.len(), 1);
        let plan = &launcher.standard_local_effect_draw_plans[0];
        assert_eq!(plan.kind, StandardEffectDrawKind::FilledCircle);
        assert_eq!(
            plan.effect_id,
            standard_effect_id("missileTrailShort").unwrap()
        );
        assert!((plan.center.0 - 93.0).abs() < 0.0001);
        assert!((plan.center.1 - 200.0).abs() < 0.0001);
        assert!((plan.radius - (3.0 * 21.0 / 22.0)).abs() < 0.0001);
        assert_eq!(launcher.standard_local_effect_circle_primitives.len(), 1);
        let primitive = &launcher.standard_local_effect_circle_primitives[0];
        assert_eq!(primitive.kind, StandardEffectDrawKind::FilledCircle);
        assert!((primitive.center.0 - 93.0).abs() < 0.0001);
        assert!((primitive.center.1 - 200.0).abs() < 0.0001);
        assert!((primitive.radius - plan.radius).abs() < 0.0001);
        assert_eq!(primitive.stroke, 0.0);
        assert_eq!(primitive.alpha, 1.0);
    }

    #[test]
    fn desktop_launcher_ticks_puddle_particle_snapshots_to_local_effect_queue() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let mut liquid = PuddleLiquidInfo::new("particle-liquid");
        liquid.has_particle_effect = true;
        liquid.particle_effect = "ripple".to_string();
        liquid.particle_spacing = 2.0;
        launcher
            .runtime
            .client_puddle_snapshot_liquids
            .insert(77, liquid);
        let mut puddle = PuddleComp::new(77, 8.0, 16.0);
        puddle.amount = PuddleComp::MAX_LIQUID / 2.0;
        puddle.effect_time = 1.0;
        launcher
            .runtime
            .client_puddle_snapshot_entities
            .insert(77, puddle);

        launcher.update();

        assert!(launcher.runtime.client_local_effect_events.is_empty());
        assert_eq!(launcher.runtime.client_local_effect_entities.len(), 1);
        let effect = launcher
            .runtime
            .client_local_effect_entities
            .get(&-1)
            .unwrap();
        assert_eq!(
            effect.effect_id,
            Some(standard_effect_id("ripple").unwrap() as u16)
        );
        let offset_x = effect.x - 8.0;
        let offset_y = effect.y - 16.0;
        assert_eq!(effect.lifetime, 30.0);
        assert_eq!(effect.effect_clip, 50.0);
        assert!(offset_x.abs() <= 3.0);
        assert!(offset_y.abs() <= 3.0);
        assert!(
            offset_x.abs() > 0.0001 || offset_y.abs() > 0.0001,
            "desktop should no longer collapse Java Mathf.range(size) to the puddle center"
        );
        assert_eq!(effect.data, TypeValue::Null);
    }

    #[test]
    fn desktop_launcher_syncs_unit_spawn_packet_without_losing_assembler_spawned() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let tile_pos = mindustry_core::mindustry::world::point2_pack(9, 7);
        let assembler_def = launcher
            .content_loader
            .block_by_name("tank-assembler")
            .unwrap();
        launcher.runtime.add_building(BuildingComp::new(
            tile_pos,
            assembler_def.base().clone(),
            TeamId(4),
        ));
        launcher.runtime.unit_runtime_states.insert(
            tile_pos,
            GameRuntimeUnitBlockState::Assembler {
                common: PayloadBlockBuildState::default(),
                assembler: UnitAssemblerState {
                    progress: 1.2,
                    ..UnitAssemblerState::default()
                },
            },
        );

        let stell = launcher
            .content_loader
            .unit_by_name("stell")
            .unwrap()
            .clone();
        let mut unit = UnitComp::new(2027, stell, TeamId(4));
        unit.set_pos(144.0, 96.0);
        unit.set_rotation(0.0);
        let mut sync = Vec::new();
        type_io::write_unit_sync(&mut sync, &launcher.content_loader, &unit.to_sync_wire())
            .unwrap();
        let unit_spawn = UnitSpawnCallPacket {
            container: type_io::UnitSyncContainer::new(
                unit.id(),
                entity_class_id(unit.type_info.name()).unwrap(),
                sync,
            ),
        };

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::AssemblerUnitSpawnedCallPacket(
                AssemblerUnitSpawnedCallPacket {
                    tile: Some(tile_pos),
                },
            ));
            net.handle_client_received(PacketKind::UnitSpawnCallPacket(unit_spawn));
        }
        launcher.update();

        let Some(GameRuntimeUnitBlockState::Assembler { assembler, .. }) =
            launcher.runtime.unit_runtime_states.get(&tile_pos)
        else {
            panic!("unit assembler state should remain present");
        };
        assert_eq!(assembler.progress, 0.0);
        assert!(launcher.runtime.client_local_sound_at_events.is_empty());
        assert_eq!(launcher.pending_sound_at_events.len(), 1);
        assert!(launcher.runtime.client_local_effect_events.is_empty());
        assert_eq!(launcher.runtime.client_local_effect_entities.len(), 1);
        let spawned = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&2027)
            .expect("desktop should apply unit spawn sync container");
        assert_eq!(spawned.type_info.name(), "stell");
        assert_eq!(spawned.x(), 144.0);
        assert_eq!(spawned.y(), 96.0);
        assert_eq!(launcher.last_applied_unit_lifecycle_packets_seen, 1);
        assert_eq!(launcher.last_applied_unit_spawn_packet_count, 1);
    }

    #[test]
    fn desktop_launcher_syncs_assembler_drone_spawned_packet_to_runtime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let tile_pos = mindustry_core::mindustry::world::point2_pack(10, 7);
        let assembler_def = launcher
            .content_loader
            .block_by_name("tank-assembler")
            .unwrap();
        launcher.runtime.add_building(BuildingComp::new(
            tile_pos,
            assembler_def.base().clone(),
            TeamId(4),
        ));
        launcher.runtime.unit_runtime_states.insert(
            tile_pos,
            GameRuntimeUnitBlockState::Assembler {
                common: PayloadBlockBuildState::default(),
                assembler: UnitAssemblerState {
                    drone_progress: 0.8,
                    read_unit_ids: vec![33],
                    ..UnitAssemblerState::default()
                },
            },
        );

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::AssemblerDroneSpawnedCallPacket(
                AssemblerDroneSpawnedCallPacket {
                    tile: Some(tile_pos),
                    id: 88,
                },
            ));
        }
        launcher.update();

        let Some(GameRuntimeUnitBlockState::Assembler { assembler, .. }) =
            launcher.runtime.unit_runtime_states.get(&tile_pos)
        else {
            panic!("unit assembler state should remain present");
        };
        assert_eq!(assembler.drone_progress, 0.0);
        assert_eq!(assembler.read_unit_ids, vec![33, 88]);
        let spawned = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&88)
            .expect("desktop should materialize assembler drone snapshot");
        assert_eq!(spawned.type_info.name(), "assembly-drone");
        assert_eq!(spawned.team_id(), TeamId(4));
        assert_eq!(spawned.x(), launcher.runtime.buildings()[0].x);
        assert_eq!(spawned.y(), launcher.runtime.buildings()[0].y);
        assert_eq!(spawned.rotation(), 90.0);
        assert!(matches!(spawned.controller, UnitControllerState::Assembler));
        let tether = spawned
            .building_tether
            .as_ref()
            .expect("desktop materialized assembler drone should keep a tether");
        assert_eq!(
            tether.building,
            Some(BuildingTetherRef {
                tile_pos,
                team: TeamId(4),
                valid: true,
            })
        );
        assert_eq!(tether.update(), BuildingTetherAction::Keep);
        assert_eq!(launcher.last_applied_unit_lifecycle_packets_seen, 1);
    }

    #[test]
    fn desktop_launcher_syncs_landing_pad_landed_packet_to_runtime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let tile_pos = mindustry_core::mindustry::world::point2_pack(11, 7);
        let landing_def = launcher
            .content_loader
            .block_by_name("landing-pad")
            .unwrap();
        let copper = launcher
            .content_loader
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        launcher.runtime.add_building(BuildingComp::new(
            tile_pos,
            landing_def.base().clone(),
            TeamId(4),
        ));
        launcher.runtime.campaign_runtime_states.insert(
            tile_pos,
            GameRuntimeCampaignBlockState::LandingPad(LandingPadState {
                config: Some(copper),
                ..LandingPadState::default()
            }),
        );

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::LandingPadLandedCallPacket(
                LandingPadLandedCallPacket {
                    tile: Some(tile_pos),
                },
            ));
        }
        launcher.update();

        let Some(GameRuntimeCampaignBlockState::LandingPad(state)) =
            launcher.runtime.campaign_runtime_states.get(&tile_pos)
        else {
            panic!("landing pad state should remain present");
        };
        assert_eq!(state.cooldown, 1.0);
        assert_eq!(state.arriving, Some(copper));
        assert_eq!(launcher.last_applied_world_update_packets_seen, 1);
    }

    #[test]
    fn desktop_launcher_syncs_unit_despawn_packet_to_runtime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let flare = launcher
            .content_loader
            .unit_by_name("flare")
            .unwrap()
            .clone();
        launcher
            .runtime
            .client_unit_snapshot_entities
            .insert(9902, UnitComp::new(9902, flare, TeamId(4)));

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::UnitDespawnCallPacket(UnitDespawnCallPacket {
                unit: UnitRef::Unit { id: 9902 },
            }));
        }
        launcher.update();

        assert!(!launcher
            .runtime
            .client_unit_snapshot_entities
            .contains_key(&9902));
        assert_eq!(launcher.last_applied_unit_lifecycle_packets_seen, 1);
    }

    #[test]
    fn desktop_launcher_syncs_unit_destroy_packet_to_leg_destroy_effects() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();
        launcher.runtime.state.set_sector(Some(Sector::new(7)));

        let mut unit_type = UnitType::new(9903, "crawler");
        unit_type.allow_leg_step = true;
        unit_type.leg_count = 2;
        unit_type.leg_length = 10.0;
        unit_type.leg_extension = 3.0;
        unit_type.leg_region = TextureRegionRef::with_size("crawler-leg", 16, 8);
        unit_type.leg_base_region = TextureRegionRef::with_size("crawler-leg-base", 12, 6);
        let mut death_weapon = Weapon::new("death-cannon");
        death_weapon.shoot_on_death = true;
        death_weapon.shoot_on_death_effect = Some("smoke".into());
        death_weapon.bullet = "death-blast".into();
        unit_type.weapons.push(death_weapon);
        unit_type
            .abilities
            .push("SpawnDeathAbility:flare,2,8".into());
        let mut unit = UnitComp::new(9903, unit_type, TeamId(4));
        unit.add();
        unit.set_pos(10.0, 20.0);
        unit.set_controller(UnitControllerState::Player {
            player_id: launcher.player.id,
        });
        unit.items.stack.item = Some("blast-compound".into());
        unit.items.stack.amount = 3;
        launcher
            .runtime
            .client_unit_snapshot_entities
            .insert(9903, unit);

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::UnitDestroyCallPacket(UnitDestroyCallPacket {
                uid: 9903,
            }));
        }
        launcher.update();

        assert!(!launcher
            .runtime
            .client_unit_snapshot_entities
            .contains_key(&9903));
        assert_eq!(launcher.last_applied_unit_lifecycle_packets_seen, 1);
        assert_eq!(launcher.runtime.client_local_effect_entities.len(), 6);
        assert_eq!(launcher.pending_sound_at_events.len(), 1);
        assert_eq!(
            launcher.pending_sound_at_events[0].sound_id,
            mindustry_core::mindustry::audio::standard_sound_id("unitExplode1").unwrap()
        );
        assert_eq!(launcher.pending_camera_shake_events.len(), 1);
        assert!((launcher.pending_camera_shake_events[0].intensity - 5.0).abs() < 0.0001);
        assert_eq!(launcher.runtime.client_decal_snapshot_entities.len(), 1);
        let scorch = launcher
            .runtime
            .client_decal_snapshot_entities
            .get(&-1)
            .unwrap();
        assert_eq!(scorch.region.name, "scorch-1-1");
        assert_eq!(scorch.x, 10.0);
        assert_eq!(scorch.y, 20.0);
        assert_eq!(launcher.runtime.unit_destroy_events.len(), 1);
        assert_eq!(launcher.runtime.unit_destroy_events[0].unit_id, 9903);
        assert_eq!(launcher.runtime.unit_shoot_on_death_events.len(), 1);
        assert_eq!(
            launcher.runtime.unit_shoot_on_death_events[0].weapon_name,
            "death-cannon"
        );
        assert_eq!(launcher.runtime.unit_ability_death_events.len(), 1);
        assert_eq!(
            launcher.runtime.unit_ability_death_events[0].ability_kind,
            "SpawnDeathAbility"
        );
        assert_eq!(launcher.runtime.unit_type_killed_events.len(), 1);
        assert_eq!(
            launcher.runtime.unit_type_killed_events[0].unit_type_name,
            "crawler"
        );
        assert!(
            launcher.runtime.trigger_events.is_empty(),
            "suicideBomb should be drained into GameService during the same desktop update"
        );
        assert_eq!(
            launcher
                .last_service_trigger_apply_summary
                .map(|summary| summary.achievements_completed),
            Some(1)
        );
        assert!(launcher
            .client
            .service
            .achievements()
            .contains("suicideBomb"));
        let leg_primitives = launcher
            .standard_local_effect_line_primitives
            .iter()
            .filter(|primitive| primitive.region.as_deref() == Some("crawler-leg"))
            .count();
        let leg_base_primitives = launcher
            .standard_local_effect_line_primitives
            .iter()
            .filter(|primitive| primitive.region.as_deref() == Some("crawler-leg-base"))
            .count();
        assert_eq!(leg_primitives, 2);
        assert_eq!(leg_base_primitives, 2);
        assert!(
            launcher.standard_local_effect_line_primitives.len()
                >= leg_primitives + leg_base_primitives
        );
    }

    #[test]
    fn desktop_launcher_syncs_multiple_unit_lifecycle_packets_in_one_update() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        for unit_id in [9911_i32, 9912_i32] {
            let mut unit_type = UnitType::new(unit_id as i16, "crawler");
            unit_type.allow_leg_step = true;
            unit_type.leg_count = 2;
            unit_type.leg_length = 10.0;
            unit_type.leg_extension = 3.0;
            unit_type.leg_region = TextureRegionRef::with_size("crawler-leg", 16, 8);
            unit_type.leg_base_region = TextureRegionRef::with_size("crawler-leg-base", 12, 6);
            let mut unit = UnitComp::new(unit_id, unit_type, TeamId(4));
            unit.add();
            unit.set_pos(10.0 + unit_id as f32, 20.0);
            launcher
                .runtime
                .client_unit_snapshot_entities
                .insert(unit_id, unit);
        }

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::UnitDeathCallPacket(UnitDeathCallPacket {
                uid: 9911,
            }));
            net.handle_client_received(PacketKind::UnitDestroyCallPacket(UnitDestroyCallPacket {
                uid: 9912,
            }));
        }
        launcher.update();

        assert!(!launcher
            .runtime
            .client_unit_snapshot_entities
            .contains_key(&9911));
        assert!(!launcher
            .runtime
            .client_unit_snapshot_entities
            .contains_key(&9912));
        assert_eq!(launcher.last_applied_unit_lifecycle_packets_seen, 2);
        assert_eq!(launcher.runtime.client_local_effect_entities.len(), 10);
        assert_eq!(launcher.pending_sound_at_events.len(), 2);
        assert_eq!(launcher.pending_camera_shake_events.len(), 2);
        assert_eq!(launcher.runtime.client_decal_snapshot_entities.len(), 2);
        assert_eq!(launcher.runtime.unit_destroy_events.len(), 2);
        assert!(launcher.standard_local_effect_line_primitives.len() >= 8);
        let leg_primitives = launcher
            .standard_local_effect_line_primitives
            .iter()
            .filter(|primitive| primitive.region.as_deref() == Some("crawler-leg"))
            .count();
        let leg_base_primitives = launcher
            .standard_local_effect_line_primitives
            .iter()
            .filter(|primitive| primitive.region.as_deref() == Some("crawler-leg-base"))
            .count();
        assert_eq!(leg_primitives, 4);
        assert_eq!(leg_base_primitives, 4);
    }

    #[test]
    fn desktop_launcher_syncs_flying_unit_death_to_wreck_sound_without_remove() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let mut unit_type = UnitType::new(9913, "flare");
        unit_type.flying = true;
        unit_type.create_wreck = true;
        unit_type.hit_size = 24.0;
        unit_type.wreck_sound_volume = 0.8;
        let mut unit = UnitComp::new(9913, unit_type, TeamId(4));
        unit.add();
        unit.set_pos(50.0, 60.0);
        launcher
            .runtime
            .client_unit_snapshot_entities
            .insert(9913, unit);

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::UnitDeathCallPacket(UnitDeathCallPacket {
                uid: 9913,
            }));
        }
        launcher.update();

        let unit = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&9913)
            .unwrap();
        assert!(unit.health.dead);
        assert!(unit.entity.is_added());
        assert_eq!(launcher.last_applied_unit_lifecycle_packets_seen, 1);
        assert!(launcher.runtime.client_local_effect_entities.is_empty());
        assert!(launcher.runtime.client_local_sound_at_events.is_empty());
        assert_eq!(launcher.pending_sound_at_events.len(), 1);
        let sound = &launcher.pending_sound_at_events[0];
        assert_eq!(
            sound.sound_id,
            mindustry_core::mindustry::audio::standard_sound_id("wreckFallBig").unwrap()
        );
        assert_eq!(sound.x, 50.0);
        assert_eq!(sound.y, 60.0);
        assert_eq!(sound.volume, 0.8);
        assert_eq!(sound.pitch, 1.0);
    }

    #[test]
    fn desktop_launcher_syncs_unit_safe_death_packet_to_runtime_remove_effect() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let mut unit_type = UnitType::new(9921, "crawler");
        unit_type.hit_size = 16.0;
        unit_type.death_sound = "unitExplode1".into();
        unit_type.death_sound_volume = 0.7;
        let mut unit = UnitComp::new(9921, unit_type, TeamId(4));
        unit.add();
        unit.set_pos(30.0, 40.0);
        launcher
            .runtime
            .client_unit_snapshot_entities
            .insert(9921, unit);

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::UnitSafeDeathCallPacket(
                UnitSafeDeathCallPacket {
                    unit: UnitRef::Unit { id: 9921 },
                },
            ));
        }
        launcher.update();

        assert!(!launcher
            .runtime
            .client_unit_snapshot_entities
            .contains_key(&9921));
        assert_eq!(launcher.last_applied_unit_lifecycle_packets_seen, 1);
        assert_eq!(launcher.runtime.client_local_effect_entities.len(), 1);
        assert!(launcher.runtime.client_local_sound_at_events.is_empty());
        assert_eq!(launcher.pending_sound_at_events.len(), 1);
        assert_eq!(
            launcher.pending_sound_at_events[0].sound_id,
            mindustry_core::mindustry::audio::standard_sound_id("unitExplode1").unwrap()
        );
        assert!(launcher.runtime.client_local_camera_shake_events.is_empty());
        assert_eq!(launcher.pending_camera_shake_events.len(), 1);
        assert_eq!(launcher.pending_camera_shake_events[0].x, 30.0);
        assert_eq!(launcher.pending_camera_shake_events[0].y, 40.0);
        assert!((launcher.pending_camera_shake_events[0].intensity - 16.0 / 3.0).abs() < 0.0001);
        assert!((launcher.last_camera_shake_frame.max_offset - 4.0).abs() < 0.0001);
        assert!((launcher.camera_shake_state.intensity - (16.0 / 3.0 - 1.0)).abs() < 0.0001);
    }

    #[test]
    fn desktop_launcher_resolves_camera_shake_events_for_render_like_java_effect_shake() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher
            .runtime
            .client_local_camera_shake_events
            .push(GameRuntimeClientCameraShakeEvent {
                x: 200.0,
                y: 0.0,
                intensity: 8.0,
                duration: 8.0,
            });

        assert_eq!(
            launcher.sync_local_camera_shake_events_for_render(0.0, 0.0),
            1
        );
        assert!(launcher.runtime.client_local_camera_shake_events.is_empty());
        assert_eq!(launcher.pending_camera_shake_events.len(), 1);
        assert_eq!(launcher.camera_shake_state.intensity, 2.0);
        assert_eq!(launcher.camera_shake_state.time, 8.0);
        assert_eq!(launcher.camera_shake_state.reduction, 0.25);

        let frame = launcher.tick_camera_shake_for_render(1.0, 4);
        assert_eq!(frame.max_offset, 1.5);
        assert_eq!(frame.remaining_intensity, 1.75);
        assert_eq!(frame.remaining_time, 7.0);

        let drained = launcher.drain_camera_shake_events_for_render();
        assert_eq!(drained.len(), 1);
        assert!(launcher.pending_camera_shake_events.is_empty());
    }

    #[test]
    fn desktop_launcher_applies_camera_shake_frame_with_headless_renderer() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher.last_camera_shake_frame = DesktopCameraShakeFrame {
            max_offset: 2.5,
            remaining_intensity: 1.25,
            remaining_time: 3.0,
        };

        let mut renderer = HeadlessDesktopCameraShakeRenderer::default();
        let stats = launcher.apply_camera_shake_frame_with(&mut renderer);

        assert_eq!(stats.max_offset, 2.5);
        assert_eq!(stats.remaining_intensity, 1.25);
        assert_eq!(stats.remaining_time, 3.0);
        assert_eq!(renderer.frames_applied, 1);
        assert_eq!(renderer.last_stats, stats);
    }

    #[test]
    fn desktop_launcher_syncs_unit_cap_and_env_death_packets_to_runtime_mark_dead() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        for (unit_id, x, y) in [(9931_i32, 11.0, 12.0), (9932_i32, 21.0, 22.0)] {
            let mut unit =
                UnitComp::new(unit_id, UnitType::new(unit_id as i16, "crawler"), TeamId(4));
            unit.add();
            unit.set_pos(x, y);
            launcher
                .runtime
                .client_unit_snapshot_entities
                .insert(unit_id, unit);
        }

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::UnitCapDeathCallPacket(
                UnitCapDeathCallPacket {
                    unit: UnitRef::Unit { id: 9931 },
                },
            ));
            net.handle_client_received(PacketKind::UnitEnvDeathCallPacket(
                UnitEnvDeathCallPacket {
                    unit: UnitRef::Unit { id: 9932 },
                },
            ));
        }
        launcher.update();

        let cap_unit = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&9931)
            .unwrap();
        assert!(cap_unit.health.dead);
        let env_unit = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&9932)
            .unwrap();
        assert!(env_unit.health.dead);
        assert_eq!(launcher.last_applied_unit_lifecycle_packets_seen, 2);
        assert_eq!(launcher.runtime.client_local_effect_entities.len(), 2);
    }

    #[test]
    fn desktop_launcher_syncs_unit_cargo_unload_tile_config_packet_to_runtime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let tile_pos = mindustry_core::mindustry::world::point2_pack(7, 7);
        let unload_def = launcher
            .content_loader
            .block_by_name("unit-cargo-unload-point")
            .unwrap();
        let copper = launcher
            .content_loader
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        launcher.runtime.add_building(BuildingComp::new(
            tile_pos,
            unload_def.base().clone(),
            TeamId(4),
        ));
        launcher.runtime.distribution_runtime_states.insert(
            tile_pos,
            GameRuntimeDistributionBlockState::UnitCargoUnload(UnitCargoUnloadPointState {
                item_id: None,
                stale_timer: 0.0,
                stale: false,
            }),
        );

        let value = TypeValue::Content(type_io::ContentRef::new(ContentType::Item, copper));
        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::TileConfigCallPacket(
                TileConfigCallPacket::server(
                    mindustry_core::mindustry::io::EntityRef::new(42),
                    BuildingRef::new(tile_pos),
                    value.clone(),
                ),
            ));
        }
        launcher.update();

        let Some(GameRuntimeDistributionBlockState::UnitCargoUnload(state)) =
            launcher.runtime.distribution_runtime_states.get(&tile_pos)
        else {
            panic!("unit cargo unload state should remain present");
        };
        assert_eq!(state.item_id, Some(copper as i32));
        assert_eq!(launcher.runtime.buildings()[0].config, Some(value));
        assert_eq!(
            launcher.last_tile_config_apply_result,
            Some(GameRuntimeUnitCargoUnloadConfigureResult::Configured)
        );
    }

    #[test]
    fn desktop_launcher_syncs_unit_factory_command_tile_config_packet_to_runtime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let tile_pos = mindustry_core::mindustry::world::point2_pack(9, 7);
        let factory_def = launcher
            .content_loader
            .block_by_name("air-factory")
            .unwrap();
        let rebuild_id = launcher
            .content_loader
            .unit_command_by_name("rebuild")
            .unwrap()
            .id();
        let mut factory_building =
            BuildingComp::new(tile_pos, factory_def.base().clone(), TeamId(4));
        factory_building.config = Some(TypeValue::Int(0));
        launcher.runtime.add_building(factory_building);
        launcher.runtime.unit_runtime_states.insert(
            tile_pos,
            GameRuntimeUnitBlockState::Factory {
                common: PayloadBlockBuildState::default(),
                factory: UnitFactoryState {
                    current_plan: 0,
                    base: UnitBlockState {
                        progress: 13.0,
                        ..UnitBlockState::default()
                    },
                    ..UnitFactoryState::default()
                },
            },
        );

        let value = TypeValue::Content(type_io::ContentRef::new(
            ContentType::UnitCommand,
            rebuild_id,
        ));
        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::TileConfigCallPacket(
                TileConfigCallPacket::server(
                    mindustry_core::mindustry::io::EntityRef::new(42),
                    BuildingRef::new(tile_pos),
                    value.clone(),
                ),
            ));
        }
        launcher.update();

        let Some(GameRuntimeUnitBlockState::Factory { factory, .. }) =
            launcher.runtime.unit_runtime_states.get(&tile_pos)
        else {
            panic!("unit factory state should remain present after command config");
        };
        assert_eq!(factory.command_id, Some(rebuild_id as u8));
        assert_eq!(factory.current_plan, 0);
        assert_eq!(factory.base.progress, 13.0);
        assert_eq!(
            launcher.runtime.buildings()[0].config,
            Some(TypeValue::Int(0))
        );
        assert_eq!(
            launcher.last_unit_factory_tile_config_apply_result,
            Some(GameRuntimeUnitFactoryConfigureResult::Configured)
        );

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::TileConfigCallPacket(
                TileConfigCallPacket::server(
                    mindustry_core::mindustry::io::EntityRef::new(42),
                    BuildingRef::new(tile_pos),
                    TypeValue::Null,
                ),
            ));
        }
        launcher.update();

        let Some(GameRuntimeUnitBlockState::Factory { factory, .. }) =
            launcher.runtime.unit_runtime_states.get(&tile_pos)
        else {
            panic!("unit factory state should remain present after command clear");
        };
        assert_eq!(factory.command_id, None);
        assert_eq!(factory.current_plan, 0);
        assert_eq!(
            launcher.runtime.buildings()[0].config,
            Some(TypeValue::Int(0))
        );
        assert_eq!(
            launcher.last_unit_factory_tile_config_apply_result,
            Some(GameRuntimeUnitFactoryConfigureResult::Cleared)
        );
    }

    #[test]
    fn desktop_launcher_syncs_reconstructor_command_tile_config_packet_to_runtime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let tile_pos = mindustry_core::mindustry::world::point2_pack(11, 7);
        let reconstructor_def = launcher
            .content_loader
            .block_by_name("additive-reconstructor")
            .unwrap();
        let rebuild_id = launcher
            .content_loader
            .unit_command_by_name("rebuild")
            .unwrap()
            .id();
        launcher.runtime.add_building(BuildingComp::new(
            tile_pos,
            reconstructor_def.base().clone(),
            TeamId(4),
        ));
        launcher.runtime.unit_runtime_states.insert(
            tile_pos,
            GameRuntimeUnitBlockState::Reconstructor {
                common: PayloadBlockBuildState::default(),
                reconstructor: ReconstructorState {
                    base: UnitBlockState {
                        progress: 19.0,
                        ..UnitBlockState::default()
                    },
                    constructing: true,
                    ..ReconstructorState::default()
                },
            },
        );

        let value = TypeValue::Content(type_io::ContentRef::new(
            ContentType::UnitCommand,
            rebuild_id,
        ));
        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::TileConfigCallPacket(
                TileConfigCallPacket::server(
                    mindustry_core::mindustry::io::EntityRef::new(42),
                    BuildingRef::new(tile_pos),
                    value.clone(),
                ),
            ));
        }
        launcher.update();

        let Some(GameRuntimeUnitBlockState::Reconstructor { reconstructor, .. }) =
            launcher.runtime.unit_runtime_states.get(&tile_pos)
        else {
            panic!("reconstructor state should remain present after command config");
        };
        assert_eq!(reconstructor.command_id, Some(rebuild_id as u8));
        assert_eq!(reconstructor.base.progress, 19.0);
        assert!(reconstructor.constructing);
        assert_eq!(launcher.runtime.buildings()[0].config, None);
        assert_eq!(
            launcher.last_reconstructor_tile_config_apply_result,
            Some(GameRuntimeReconstructorConfigureResult::Configured)
        );

        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::TileConfigCallPacket(
                TileConfigCallPacket::server(
                    mindustry_core::mindustry::io::EntityRef::new(42),
                    BuildingRef::new(tile_pos),
                    TypeValue::Null,
                ),
            ));
        }
        launcher.update();

        let Some(GameRuntimeUnitBlockState::Reconstructor { reconstructor, .. }) =
            launcher.runtime.unit_runtime_states.get(&tile_pos)
        else {
            panic!("reconstructor state should remain present after command clear");
        };
        assert_eq!(reconstructor.command_id, None);
        assert_eq!(reconstructor.base.progress, 19.0);
        assert_eq!(launcher.runtime.buildings()[0].config, None);
        assert_eq!(
            launcher.last_reconstructor_tile_config_apply_result,
            Some(GameRuntimeReconstructorConfigureResult::Cleared)
        );
    }

    #[test]
    fn desktop_launcher_syncs_command_building_packet_to_unit_factory_runtime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }
        launcher.update();

        let tile_pos = mindustry_core::mindustry::world::point2_pack(10, 7);
        let factory_def = launcher
            .content_loader
            .block_by_name("air-factory")
            .unwrap();
        let mut factory_building =
            BuildingComp::new(tile_pos, factory_def.base().clone(), TeamId(4));
        factory_building.config = Some(TypeValue::Int(0));
        launcher.runtime.add_building(factory_building);
        launcher.runtime.unit_runtime_states.insert(
            tile_pos,
            GameRuntimeUnitBlockState::Factory {
                common: PayloadBlockBuildState::default(),
                factory: UnitFactoryState {
                    current_plan: 0,
                    ..UnitFactoryState::default()
                },
            },
        );
        let mut remote = PlayerComp::new(TeamId(4));
        remote.id = 42;
        remote.name = "remote".into();
        remote.color = 0xAABB_CCDD;
        launcher.remote_players.insert(42, remote);

        let target = IoVec2::new(88.0, 104.0);
        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::CommandBuildingCallPacket(
                CommandBuildingCallPacket {
                    player: mindustry_core::mindustry::io::EntityRef::new(42),
                    buildings: vec![tile_pos],
                    target,
                },
            ));
        }
        launcher.update();

        let Some(GameRuntimeUnitBlockState::Factory { factory, .. }) =
            launcher.runtime.unit_runtime_states.get(&tile_pos)
        else {
            panic!("unit factory state should remain present after command building");
        };
        assert_eq!(factory.command_pos, Some(target));
        assert_eq!(
            launcher.runtime.buildings()[0].last_accessed,
            "[#AABBCCDD]remote"
        );
        assert!(launcher
            .last_command_building_apply_report
            .as_ref()
            .is_some_and(|report| report.commanded_positions == vec![tile_pos]));
    }

    #[test]
    fn desktop_launcher_applies_state_snapshot_to_runtime_game_state() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(None);
        let snapshot = sample_state_snapshot();

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
            state.connect_confirm_sent = true;
            state.last_state_snapshot = Some(snapshot.clone());
        }

        launcher.update();

        assert_eq!(launcher.game_state.wavetime, snapshot.wave_time);
        assert_eq!(launcher.game_state.wave, snapshot.wave);
        assert_eq!(launcher.game_state.enemies, snapshot.enemies);
        assert_eq!(launcher.game_state.game_over, snapshot.game_over);
        assert_eq!(launcher.game_state.server_tps, snapshot.tps as i32);
        assert_eq!(launcher.runtime.state.server_tps, snapshot.tps as i32);
        assert_eq!(launcher.game_state.rand_seed0, snapshot.rand0);
        assert_eq!(launcher.game_state.rand_seed1, snapshot.rand1);
        assert_eq!(
            launcher.game_state.universe.seconds(true),
            snapshot.time_data
        );
        assert!(launcher.game_state.is_paused());
        assert_eq!(
            launcher
                .game_state
                .teams
                .get_or_null(TEAM_SHARDED)
                .unwrap()
                .core_items,
            BTreeMap::from([(0, 75), (3, 12)])
        );
        assert_eq!(
            launcher
                .game_state
                .teams
                .get_or_null(TEAM_CRUX)
                .unwrap()
                .core_items,
            BTreeMap::from([(1, 5)])
        );
    }

    #[test]
    fn desktop_launcher_applies_team_block_snapshot_to_runtime_teams() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let router_block_id = launcher
            .content_loader
            .block_by_name("router")
            .expect("base content should include router")
            .base()
            .id;

        let mut world_data = sample_network_world_data(None);
        world_data.team_blocks_snapshot = Some(sample_team_blocks(router_block_id));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }

        launcher.update();

        assert_eq!(
            launcher.game_state.teams.get_or_null(7).unwrap().plans,
            vec![
                BlockPlan::new(5, 6, 1, "router", Some("cfg".into())),
                BlockPlan::with_config_value(
                    7,
                    8,
                    2,
                    "router",
                    Some("9".into()),
                    TypeValue::Int(9),
                ),
            ]
        );
    }

    #[test]
    fn desktop_launcher_uses_content_header_snapshot_to_map_remote_ids() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let router_block = launcher
            .content_loader
            .block_by_name("router")
            .expect("base content should include router")
            .base()
            .clone();
        let mapped_command = launcher
            .content_loader
            .unit_commands()
            .iter()
            .find(|command| command.base.base.id != 0)
            .expect("base content should include a non-zero unit command")
            .clone();

        let mut world_data =
            sample_network_world_data(Some(sample_network_player_data(Some(0), Some(0))));
        world_data.content_header_snapshot = Some(sample_content_header_snapshot(
            router_block.name.as_str(),
            mapped_command.name(),
        ));
        world_data.team_blocks_snapshot = Some(sample_team_blocks(0));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }

        launcher.update();

        assert_eq!(launcher.player.selected_block, Some(router_block));
        assert_eq!(launcher.player.last_command, Some(mapped_command));
        assert_eq!(
            launcher.game_state.teams.get_or_null(7).unwrap().plans,
            vec![
                BlockPlan::new(5, 6, 1, "router", Some("cfg".into())),
                BlockPlan::with_config_value(
                    7,
                    8,
                    2,
                    "router",
                    Some("9".into()),
                    TypeValue::Int(9),
                ),
            ]
        );
        assert!(launcher.content_loader.temporary_mapper().is_some());
    }

    #[test]
    fn desktop_launcher_clears_runtime_team_plans_when_snapshot_has_no_team_blocks() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let router_block_id = launcher
            .content_loader
            .block_by_name("router")
            .expect("base content should include router")
            .base()
            .id;

        let mut first = sample_network_world_data(None);
        first.team_blocks_snapshot = Some(sample_team_blocks(router_block_id));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(first);
        }

        launcher.update();
        assert_eq!(
            launcher
                .game_state
                .teams
                .get_or_null(7)
                .unwrap()
                .plans
                .len(),
            2
        );

        let mut second = sample_network_world_data(None);
        second.tick = 100.25;

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(second);
        }

        launcher.update();

        assert!(launcher
            .game_state
            .teams
            .get_or_null(7)
            .unwrap()
            .plans
            .is_empty());
    }

    #[test]
    fn desktop_launcher_clears_temporary_content_mapper_when_header_snapshot_disappears() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let router_block = launcher
            .content_loader
            .block_by_name("router")
            .expect("base content should include router")
            .base()
            .clone();
        let mapped_command = launcher
            .content_loader
            .unit_commands()
            .iter()
            .find(|command| command.base.base.id != 0)
            .expect("base content should include a non-zero unit command")
            .clone();

        let mut first =
            sample_network_world_data(Some(sample_network_player_data(Some(0), Some(0))));
        first.content_header_snapshot = Some(sample_content_header_snapshot(
            router_block.name.as_str(),
            mapped_command.name(),
        ));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(first);
        }

        launcher.update();
        assert!(launcher.content_loader.temporary_mapper().is_some());

        let mut second =
            sample_network_world_data(Some(sample_network_player_data(Some(0), Some(0))));
        second.tick = 100.25;

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(second);
        }

        launcher.update();

        assert!(launcher.content_loader.temporary_mapper().is_none());
        assert_eq!(
            launcher.player.selected_block,
            launcher
                .content_loader
                .block(0)
                .map(|block| block.base().clone())
        );
        assert_eq!(
            launcher.player.last_command,
            launcher.content_loader.unit_command(0).cloned()
        );
    }

    #[test]
    fn desktop_launcher_resets_game_state_and_player_when_world_data_clears() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let selected_block_id = launcher
            .content_loader
            .block(0)
            .expect("base content should include block 0")
            .base()
            .id;
        let last_command_id = launcher
            .content_loader
            .unit_command(0)
            .expect("base content should include command 0")
            .base
            .base
            .id;

        let world_data = sample_network_world_data(Some(sample_network_player_data(
            Some(selected_block_id),
            Some(last_command_id),
        )));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }

        launcher.update();
        assert_eq!(launcher.game_state.world.width(), 3);
        assert_eq!(
            launcher.player.selected_block,
            launcher
                .content_loader
                .block(selected_block_id)
                .map(|block| block.base().clone())
        );
        assert_eq!(
            launcher.player.last_command,
            launcher
                .content_loader
                .unit_command(last_command_id)
                .cloned()
        );

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = None;
        }

        launcher.update();

        assert_eq!(launcher.game_state.map.name(), "empty");
        assert_eq!(launcher.game_state.world.width(), 0);
        assert_eq!(launcher.game_state.world.height(), 0);
        assert_eq!(launcher.runtime.state.world.width(), 0);
        assert_eq!(launcher.runtime.state.world.height(), 0);
        assert_eq!(
            launcher.runtime.network_context,
            GameRuntimeNetworkContext::offline()
        );
        assert!(launcher.game_state.world.load_events().is_empty());
        assert_eq!(launcher.player, PlayerComp::default());
    }

    #[test]
    fn desktop_launcher_materializes_network_map_buildings_into_runtime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let mend_def = launcher
            .content_loader
            .block_by_name("mend-projector")
            .unwrap();
        let tile_pos = mindustry_core::mindustry::world::point2_pack(1, 1);
        let mut saved = BuildingComp::new(tile_pos, mend_def.base().clone(), TeamId(3));
        saved.set_rotation(2);
        saved.health = 55.0;
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();

        let mut world_data = sample_network_world_data(None);
        world_data.map_snapshot = Some(LegacyShortChunkMap {
            width: 3,
            height: 2,
            floors: vec![LegacyMapFloorRecord {
                index: 0,
                floor_id: 1,
                ore_id: 0,
                consecutives: 5,
            }],
            blocks: vec![
                LegacyMapBlockRecord {
                    index: 0,
                    block_id: 0,
                    packed_flags: 0,
                    has_entity: false,
                    has_old_data: false,
                    has_new_data: false,
                    is_center: true,
                    new_data: None,
                    old_data: None,
                    building: None,
                    consecutives: 3,
                },
                LegacyMapBlockRecord {
                    index: 4,
                    block_id: mend_def.base().id,
                    packed_flags: 1,
                    has_entity: true,
                    has_old_data: false,
                    has_new_data: false,
                    is_center: true,
                    new_data: None,
                    old_data: None,
                    building: Some(building_bytes),
                    consecutives: 0,
                },
                LegacyMapBlockRecord {
                    index: 5,
                    block_id: 0,
                    packed_flags: 0,
                    has_entity: false,
                    has_old_data: false,
                    has_new_data: false,
                    is_center: true,
                    new_data: None,
                    old_data: None,
                    building: None,
                    consecutives: 0,
                },
            ],
        });

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }

        launcher.update();

        assert_eq!(
            launcher.runtime.network_context,
            GameRuntimeNetworkContext::client()
        );
        let report = launcher
            .last_runtime_map_load_report
            .as_ref()
            .expect("network map snapshot should be materialized into runtime");
        assert_eq!(report.building_records, 1);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.building_parse_errors, 0);
        assert_eq!(launcher.runtime.buildings().len(), 1);
        let building = &launcher.runtime.buildings()[0];
        assert_eq!(building.tile_pos, tile_pos);
        assert_eq!(building.team, TeamId(3));
        assert_eq!(building.rotation, 2);
        assert_eq!(building.health, 55.0);
        assert_eq!(
            launcher
                .runtime
                .state
                .world
                .build_pos(tile_pos)
                .unwrap()
                .tile_pos,
            tile_pos
        );
    }

    #[test]
    fn desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let router_base = launcher
            .content_loader
            .block_by_name("router")
            .expect("base content should include router")
            .base()
            .clone();
        let tile_pos = mindustry_core::mindustry::world::point2_pack(2, 2);

        let mut source_runtime = GameRuntime::default();
        source_runtime.state.world.resize(6, 6);
        source_runtime.add_building(BuildingComp::new(tile_pos, router_base.clone(), TeamId(6)));

        let mut world_data = sample_network_world_data(None);
        world_data.map_snapshot =
            Some(source_runtime.export_network_map_snapshot(&launcher.content_loader));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }

        launcher.update();
        assert!(launcher.last_client_snapshot_apply_report.is_none());

        let mut synced_router = BuildingComp::new(tile_pos, router_base.clone(), TeamId(6));
        synced_router.health = 27.0;
        synced_router.set_rotation(3);
        let mut block_sync_bytes = Vec::new();
        synced_router
            .write_base(&mut block_sync_bytes, false)
            .unwrap();

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_block_snapshot_mirror = Some(ClientBlockSnapshotMirror {
                amount: 1,
                data: Vec::new(),
                records: vec![ClientBlockSnapshotRecordMirror {
                    tile_pos,
                    block_id: router_base.id,
                    sync_bytes: block_sync_bytes.clone(),
                }],
                parse_error: None,
            });
            state
                .entity_snapshot_mirrors
                .push(ClientEntitySnapshotMirror {
                    amount: 1,
                    data: Vec::new(),
                    records: vec![ClientEntitySnapshotRecordMirror {
                        entity_id: 1001,
                        type_id: 2,
                        sync_bytes: vec![4, 5],
                    }],
                    parse_error: None,
                });
            state.last_hidden_snapshot_mirror =
                Some(ClientHiddenSnapshotMirror { ids: vec![1001] });
        }

        launcher.update();

        let report = launcher
            .last_client_snapshot_apply_report
            .expect("snapshot mirrors should apply to runtime sidecars");
        assert_eq!(report.block_records_applied, 1);
        assert_eq!(report.block_base_records_applied, 1);
        assert_eq!(report.entity_records_applied, 1);
        assert_eq!(report.hidden_existing_entities, 1);

        let block_record = launcher
            .runtime
            .client_block_snapshot_records
            .get(&tile_pos)
            .expect("block snapshot should land on runtime sidecar");
        assert_eq!(block_record.block_id, router_base.id);
        assert_eq!(block_record.sync_bytes, block_sync_bytes);
        let building = launcher
            .runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == tile_pos)
            .unwrap();
        assert_eq!(building.health, 27.0);
        assert_eq!(building.rotation, 3);

        let entity_record = launcher
            .runtime
            .client_entity_snapshot_records
            .get(&1001)
            .expect("entity snapshot should land on runtime sidecar");
        assert_eq!(entity_record.type_id, 2);
        assert_eq!(entity_record.sync_bytes, vec![4, 5]);
        assert!(entity_record.hidden);
        assert!(launcher.runtime.client_hidden_entity_ids.contains(&1001));
    }

    #[test]
    fn desktop_launcher_applies_local_player_entity_snapshot_to_typed_player_runtime() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let selected_block_id = launcher
            .content_loader
            .block(0)
            .expect("base content should include block 0")
            .base()
            .id;
        let world_data = sample_network_world_data(Some(sample_network_player_data(
            Some(selected_block_id),
            None,
        )));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }

        launcher.update();
        assert_eq!(launcher.player.id, 91);

        launcher.player.boosting = false;
        launcher.player.mouse_x = 1.25;
        launcher.player.mouse_y = -1.5;
        launcher.player.selected_rotation = 1;
        launcher.player.shooting = false;
        launcher.player.typing = false;
        launcher.player.x = 10.0;
        launcher.player.y = 20.0;

        let sync = sample_network_player_sync_data(Some(selected_block_id));
        let mut sync_bytes = Vec::new();
        sync.write_to(&mut sync_bytes).unwrap();
        let mut packet_data = Vec::new();
        packet_data.extend_from_slice(&launcher.player.id.to_be_bytes());
        packet_data.push(PLAYER_CLASS_ID);
        packet_data.extend_from_slice(&sync_bytes);

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state
                .entity_snapshot_mirrors
                .push(ClientEntitySnapshotMirror {
                    amount: 1,
                    data: packet_data,
                    records: vec![ClientEntitySnapshotRecordMirror {
                        entity_id: launcher.player.id,
                        type_id: PLAYER_CLASS_ID,
                        sync_bytes: sync_bytes.clone(),
                    }],
                    parse_error: None,
                });
        }

        launcher.update();

        let report = launcher
            .last_client_snapshot_apply_report
            .expect("player entity snapshot should apply to typed player runtime");
        assert_eq!(report.entity_records_applied, 1);
        assert_eq!(report.entity_typed_records_applied, 1);

        let raw = launcher
            .runtime
            .client_entity_snapshot_records
            .get(&91)
            .expect("player entity snapshot should preserve raw sidecar");
        assert_eq!(raw.type_id, PLAYER_CLASS_ID);
        assert_eq!(raw.sync_bytes, sync_bytes);
        assert_eq!(
            launcher.runtime.client_player_snapshot_entities.get(&91),
            Some(&sync)
        );

        assert!(!launcher.player.admin);
        assert_eq!(launcher.player.color, 0x55_66_77_88);
        assert_eq!(launcher.player.name, "snapshot-pilot");
        assert_eq!(launcher.player.team, TeamId(3));
        assert_eq!(launcher.player.unit_ref(), Some(UnitRef::Unit { id: 7701 }));

        // Java Player.readSync consumes @SyncLocal fields for the local player
        // but does not overwrite local input/position state with them.
        assert!(!launcher.player.boosting);
        assert_eq!(launcher.player.mouse_x, 1.25);
        assert_eq!(launcher.player.mouse_y, -1.5);
        assert_eq!(launcher.player.selected_rotation, 1);
        assert!(!launcher.player.shooting);
        assert!(!launcher.player.typing);
        assert_eq!(launcher.player.x, 10.0);
        assert_eq!(launcher.player.y, 20.0);
    }

    #[test]
    fn desktop_launcher_builds_remote_player_preview_overlay_from_snapshot_and_plan_packets() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(Some(sample_network_player_data(None, None)));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }

        launcher.update();
        assert_eq!(launcher.player.id, 91);
        assert_eq!(launcher.player.team, TeamId(6));

        let remote_id = 1234;
        let remote_sync = NetworkPlayerSyncData {
            name: Some("ally-builder".into()),
            team: launcher.player.team,
            x: 64.0,
            y: 72.0,
            ..sample_network_player_sync_data(None)
        };
        let mut sync_bytes = Vec::new();
        remote_sync.write_to(&mut sync_bytes).unwrap();

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state
                .entity_snapshot_mirrors
                .push(ClientEntitySnapshotMirror {
                    amount: 1,
                    data: Vec::new(),
                    records: vec![ClientEntitySnapshotRecordMirror {
                        entity_id: remote_id,
                        type_id: PLAYER_CLASS_ID,
                        sync_bytes,
                    }],
                    parse_error: None,
                });
        }
        {
            let mut net = launcher.net_client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::ClientPlanSnapshotReceivedCallPacket(
                ClientPlanSnapshotReceivedCallPacket {
                    player_id: remote_id,
                    group_id: 1,
                    plans: Some(vec![type_io::BuildPlanWire::new_place(4, 5, 1, "router")]),
                },
            ));
        }

        launcher.update();
        assert!(launcher.remote_players.contains_key(&remote_id));
        assert!(launcher.other_player_preview_overlays.is_empty());

        let overlay_count = launcher.rebuild_other_player_preview_overlays_at(
            i64::MAX / 4,
            1.0,
            Some(IoVec2::new(32.0, 40.0)),
        );
        assert_eq!(overlay_count, 1);
        let overlay = &launcher.other_player_preview_overlays[0];
        assert_eq!(overlay.player_id, remote_id);
        assert_eq!(overlay.player_name, "ally-builder");
        assert_eq!(overlay.player_pos, IoVec2::new(64.0, 72.0));
        assert_eq!(overlay.entries.len(), 1);
        assert_eq!(overlay.entries[0].block, "router");
        assert_eq!(overlay.entries[0].world_pos, IoVec2::new(32.0, 40.0));
        assert!(overlay.entries[0].overlapping_mouse);
        assert_eq!(
            overlay.overlap.as_ref().unwrap().player_name,
            "ally-builder"
        );
    }

    #[test]
    fn desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let world_data = sample_network_world_data(Some(sample_network_player_data(None, None)));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }

        launcher.update();
        assert_eq!(launcher.player.id, 91);

        let player_sync = sample_network_player_sync_data(None);
        let mut player_bytes = Vec::new();
        player_sync.write_to(&mut player_bytes).unwrap();

        let dagger_id = launcher
            .content_loader
            .unit_by_name("dagger")
            .expect("base content should include dagger")
            .id();
        let unit_sync = type_io::UnitSyncWire {
            abilities: Vec::new(),
            ammo: 4.0,
            controller: ControllerWire::Ground,
            elevation: 0.25,
            flag: 12.0,
            health: 77.0,
            is_shooting: true,
            mine_tile: None,
            mounts: Vec::new(),
            plans: None,
            rotation: 180.0,
            shield: 3.0,
            spawned_by_core: false,
            stack: ItemStack::new("", 0),
            statuses: Vec::new(),
            team: TeamId(4),
            type_id: dagger_id,
            update_building: false,
            vel: IoVec2 { x: 0.5, y: -0.25 },
            x: 30.0,
            y: 45.0,
        };
        let mut unit_bytes = Vec::new();
        type_io::write_unit_sync(&mut unit_bytes, &launcher.content_loader, &unit_sync).unwrap();
        let bullet_sync = type_io::BulletSyncWire {
            collided: vec![7, 9],
            damage: 33.0,
            data: TypeValue::String("spark-bullet".into()),
            fdata: 2.5,
            lifetime: 120.0,
            owner: type_io::EntityRef::new(8801),
            rotation: 180.0,
            team: TeamId(4),
            time: 10.0,
            bullet_type_id: 1,
            vel: IoVec2 { x: -0.25, y: 1.5 },
            x: 20.0,
            y: 40.0,
        };
        let mut bullet_bytes = Vec::new();
        type_io::write_bullet_sync(&mut bullet_bytes, &bullet_sync).unwrap();
        let effect_sync = type_io::EffectStateSyncWire {
            color: type_io::RgbaColor::new(0x336699cc),
            data: TypeValue::String("spark".into()),
            effect_id: 7,
            lifetime: 50.0,
            offset_pos: 1.25,
            offset_rot: -2.5,
            offset_x: 3.0,
            offset_y: 4.0,
            parent_id: Some(1234),
            rot_with_parent: true,
            rotation: 90.0,
            time: 12.0,
            x: 100.0,
            y: 200.0,
        };
        let mut effect_bytes = Vec::new();
        type_io::write_effect_state_sync(&mut effect_bytes, &effect_sync).unwrap();
        let decal_sync = type_io::DecalSyncWire {
            color: type_io::RgbaColor::new(0x11223344),
            lifetime: 30.0,
            rotation: 15.0,
            time: 2.0,
            x: 12.0,
            y: 24.0,
        };
        let mut decal_bytes = Vec::new();
        type_io::write_decal_sync(&mut decal_bytes, &decal_sync).unwrap();
        let fire_sync = type_io::FireSyncWire {
            lifetime: 120.0,
            tile_pos: Some(mindustry_core::mindustry::world::point2_pack(2, 3)),
            time: 30.0,
            x: 16.0,
            y: 24.0,
        };
        let mut fire_bytes = Vec::new();
        type_io::write_fire_sync(&mut fire_bytes, &fire_sync).unwrap();
        let oil_id = launcher
            .content_loader
            .liquid_by_name("oil")
            .expect("base content should include oil")
            .base
            .mappable
            .base
            .id;
        let puddle_sync = type_io::PuddleSyncWire {
            amount: 36.5,
            liquid_id: Some(oil_id),
            tile_pos: Some(mindustry_core::mindustry::world::point2_pack(4, 5)),
            x: 32.0,
            y: 40.0,
        };
        let mut puddle_bytes = Vec::new();
        type_io::write_puddle_sync(&mut puddle_bytes, &puddle_sync).unwrap();
        let rain_id = launcher
            .content_loader
            .weather_by_name("rain")
            .expect("base content should include rain")
            .id();
        let weather_sync = type_io::WeatherStateSyncWire {
            effect_timer: 12.0,
            intensity: 0.75,
            life: 600.0,
            opacity: 0.5,
            weather_id: Some(rain_id),
            wind_vector: IoVec2 { x: -0.25, y: 0.75 },
            x: 10.0,
            y: 20.0,
        };
        let mut weather_bytes = Vec::new();
        type_io::write_weather_state_sync(&mut weather_bytes, &weather_sync).unwrap();
        let label_sync = type_io::WorldLabelSyncWire {
            flags: 1 | 8,
            font_size: 1.5,
            parent_id: Some(8801),
            text: Some("rally".into()),
            x: 72.0,
            y: 96.0,
            z: 155.0,
        };
        let mut label_bytes = Vec::new();
        type_io::write_world_label_sync(&mut label_bytes, &label_sync).unwrap();

        let mut data = Vec::new();
        data.extend_from_slice(&launcher.player.id.to_be_bytes());
        data.push(PLAYER_CLASS_ID);
        data.extend_from_slice(&player_bytes);
        data.extend_from_slice(&8801i32.to_be_bytes());
        data.push(2);
        data.extend_from_slice(&unit_bytes);
        data.extend_from_slice(&9800i32.to_be_bytes());
        data.push(BULLET_CLASS_ID);
        data.extend_from_slice(&bullet_bytes);
        data.extend_from_slice(&9801i32.to_be_bytes());
        data.push(EFFECT_STATE_CLASS_ID);
        data.extend_from_slice(&effect_bytes);
        data.extend_from_slice(&9802i32.to_be_bytes());
        data.push(DECAL_CLASS_ID);
        data.extend_from_slice(&decal_bytes);
        data.extend_from_slice(&9901i32.to_be_bytes());
        data.push(FIRE_CLASS_ID);
        data.extend_from_slice(&fire_bytes);
        data.extend_from_slice(&9902i32.to_be_bytes());
        data.push(PUDDLE_CLASS_ID);
        data.extend_from_slice(&puddle_bytes);
        data.extend_from_slice(&9903i32.to_be_bytes());
        data.push(WEATHER_STATE_CLASS_ID);
        data.extend_from_slice(&weather_bytes);
        data.extend_from_slice(&9904i32.to_be_bytes());
        data.push(WORLD_LABEL_CLASS_ID);
        data.extend_from_slice(&label_bytes);

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state
                .entity_snapshot_mirrors
                .push(ClientEntitySnapshotMirror {
                    amount: 9,
                    data,
                    records: Vec::new(),
                    parse_error: Some(
                        "multi-record entity snapshot with opaque sync bytes is not splittable yet"
                            .into(),
                    ),
                });
        }

        launcher.update();

        let report = launcher.last_client_snapshot_apply_report.expect(
            "mixed fallback should apply player, unit, bullet, decal, effect, fire, puddle, weather and world-label records",
        );
        assert_eq!(report.entity_records_applied, 9);
        assert_eq!(report.entity_typed_records_applied, 9);
        assert_eq!(report.entity_parse_errors, 0);

        assert_eq!(
            launcher.runtime.client_player_snapshot_entities.get(&91),
            Some(&player_sync)
        );
        assert_eq!(launcher.player.name, "snapshot-pilot");
        assert_eq!(
            launcher
                .runtime
                .client_entity_snapshot_records
                .get(&91)
                .map(|record| record.sync_bytes.as_slice()),
            Some(player_bytes.as_slice())
        );
        assert_eq!(
            launcher
                .runtime
                .client_entity_snapshot_records
                .get(&8801)
                .map(|record| record.sync_bytes.as_slice()),
            Some(unit_bytes.as_slice())
        );
        assert_eq!(
            launcher
                .runtime
                .client_entity_snapshot_records
                .get(&9801)
                .map(|record| record.sync_bytes.as_slice()),
            Some(effect_bytes.as_slice())
        );
        assert_eq!(
            launcher
                .runtime
                .client_entity_snapshot_records
                .get(&9901)
                .map(|record| record.sync_bytes.as_slice()),
            Some(fire_bytes.as_slice())
        );
        assert_eq!(
            launcher
                .runtime
                .client_entity_snapshot_records
                .get(&9902)
                .map(|record| record.sync_bytes.as_slice()),
            Some(puddle_bytes.as_slice())
        );
        assert_eq!(
            launcher
                .runtime
                .client_entity_snapshot_records
                .get(&9903)
                .map(|record| record.sync_bytes.as_slice()),
            Some(weather_bytes.as_slice())
        );

        let unit = launcher
            .runtime
            .client_unit_snapshot_entities
            .get(&8801)
            .expect("mixed fallback should materialize unit record");
        assert_eq!(unit.type_info.id(), dagger_id);
        assert_eq!(unit.team_id(), TeamId(4));
        assert_eq!(unit.x(), 30.0);
        assert_eq!(unit.y(), 45.0);
        assert_eq!(unit.rotation(), 180.0);
        assert!(unit.weapons.is_shooting);

        let bullet = launcher
            .runtime
            .client_bullet_snapshot_entities
            .get(&9800)
            .expect("mixed fallback should materialize bullet record");
        assert_eq!(bullet.bullet_type_id, 1);
        assert_eq!(bullet.team, TeamId(4));
        assert_eq!(bullet.owner, type_io::EntityRef::new(8801));
        assert_eq!(bullet.collided_ids, vec![7, 9]);
        assert_eq!(bullet.damage, 33.0);
        assert_eq!(bullet.data, TypeValue::String("spark-bullet".into()));
        assert_eq!(bullet.fdata, 2.5);
        assert_eq!(bullet.lifetime, 120.0);
        assert_eq!(bullet.rotation, 180.0);
        assert_eq!(bullet.time, 10.0);
        assert_eq!(bullet.velocity, IoVec2 { x: -0.25, y: 1.5 });
        assert_eq!((bullet.x, bullet.y), (20.0, 40.0));

        let effect = launcher
            .runtime
            .client_effect_snapshot_entities
            .get(&9801)
            .expect("mixed fallback should materialize effect record");
        assert_eq!(effect.effect_id, Some(7));
        assert_eq!(effect.data, TypeValue::String("spark".into()));
        assert_eq!(effect.lifetime, 50.0);
        assert_eq!(effect.parent_id, Some(1234));
        assert!(effect.rot_with_parent);
        assert_eq!(effect.rotation, 90.0);
        assert_eq!(effect.time, 12.0);
        assert_eq!(effect.x, 100.0);
        assert_eq!(effect.y, 200.0);

        let decal = launcher
            .runtime
            .client_decal_snapshot_entities
            .get(&9802)
            .expect("mixed fallback should materialize decal record");
        assert_eq!(decal.lifetime, 30.0);
        assert_eq!(decal.rotation, 15.0);
        assert_eq!(decal.time, 2.0);
        assert_eq!(decal.x, 12.0);
        assert_eq!(decal.y, 24.0);

        let fire = launcher
            .runtime
            .client_fire_snapshot_entities
            .get(&9901)
            .expect("mixed fallback should materialize fire record");
        assert_eq!(fire.lifetime, 120.0);
        assert_eq!(fire.time, 30.0);
        assert_eq!(fire.x, 16.0);
        assert_eq!(fire.y, 24.0);
        assert_eq!(fire.tile.unwrap().x, 2);
        assert_eq!(fire.tile.unwrap().y, 3);
        assert!(fire.registered);

        let puddle = launcher
            .runtime
            .client_puddle_snapshot_entities
            .get(&9902)
            .expect("mixed fallback should materialize puddle record");
        assert_eq!(puddle.amount, 36.5);
        assert_eq!(puddle.x, 32.0);
        assert_eq!(puddle.y, 40.0);
        assert_eq!(puddle.tile.unwrap().x, 4);
        assert_eq!(puddle.tile.unwrap().y, 5);
        assert_eq!(puddle.liquid.unwrap().flammability, 1.2);
        assert!(puddle.registered);

        let weather = launcher
            .runtime
            .client_weather_snapshot_entities
            .get(&9903)
            .expect("mixed fallback should materialize weather record");
        assert_eq!(weather.weather_name, "rain");
        assert_eq!(weather.effect_timer, 12.0);
        assert_eq!(weather.intensity, 0.75);
        assert_eq!(weather.life, 600.0);
        assert_eq!(weather.opacity, 0.5);
        assert_eq!(weather.wind_vector, (-0.25, 0.75));
        assert_eq!(weather.x, 10.0);
        assert_eq!(weather.y, 20.0);
        assert!(weather.added);

        let label = launcher
            .runtime
            .client_world_label_snapshot_entities
            .get(&9904)
            .expect("mixed fallback should materialize world label record");
        assert_eq!(label.flags, 1 | 8);
        assert_eq!(label.font_size, 1.5);
        assert_eq!(label.parent_id, Some(8801));
        assert_eq!(label.text, "rally");
        assert_eq!((label.x, label.y, label.z), (72.0, 96.0, 155.0));
    }

    #[test]
    fn desktop_launcher_materializes_payload_state_from_network_world_data() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        let loader_def = launcher
            .content_loader
            .block_by_name("payload-loader")
            .unwrap();
        let container_def = launcher.content_loader.block_by_name("container").unwrap();
        let loader_tile = mindustry_core::mindustry::world::point2_pack(4, 4);
        let container_id = container_def.base().id;
        let mut payload_bytes = Vec::new();
        BuildingComp::new(
            mindustry_core::mindustry::world::point2_pack(0, 0),
            container_def.base().clone(),
            TeamId(6),
        )
        .write_base(&mut payload_bytes, false)
        .unwrap();
        let mut loader_building =
            BuildingComp::new(loader_tile, loader_def.base().clone(), TeamId(6));
        loader_building.set_rotation(2);

        let mut source_runtime = GameRuntime::default();
        source_runtime.state.world.resize(12, 12);
        source_runtime.add_building(loader_building);
        source_runtime.payload_runtime_states.insert(
            loader_tile,
            GameRuntimePayloadBlockState::Loader {
                common: PayloadBlockBuildState {
                    payload: Some(PayloadRef::Block {
                        block: container_id,
                        version: 0,
                        build_bytes: payload_bytes,
                    }),
                    ..PayloadBlockBuildState::default()
                },
                loader: PayloadLoaderState {
                    exporting: true,
                    ..PayloadLoaderState::default()
                },
            },
        );

        let mut world_data = sample_network_world_data(None);
        world_data.map_snapshot =
            Some(source_runtime.export_network_map_snapshot(&launcher.content_loader));

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }

        launcher.update();

        assert_eq!(
            launcher.runtime.network_context,
            GameRuntimeNetworkContext::client()
        );
        let report = launcher
            .last_runtime_map_load_report
            .as_ref()
            .expect("network map snapshot should be materialized into runtime");
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.building_parse_errors, 0);
        let Some(GameRuntimePayloadBlockState::Loader { common, loader }) =
            launcher.runtime.payload_runtime_states.get(&loader_tile)
        else {
            panic!("payload loader sidecar should be materialized into desktop runtime");
        };
        assert!(loader.exporting);
        assert!(matches!(
            common.payload.as_ref(),
            Some(PayloadRef::Block { block, .. }) if *block == container_id
        ));
    }

    #[test]
    fn desktop_launcher_resets_player_when_world_data_has_no_player_snapshot() {
        let mut launcher = DesktopLauncher::new(Vec::new());
        launcher.player.selected_block = Some(
            launcher
                .content_loader
                .block(0)
                .expect("base content should include block 0")
                .base()
                .clone(),
        );
        launcher.player.last_command = launcher.content_loader.unit_command(0).cloned();

        let mut world_data = sample_network_world_data(None);
        world_data.tick = 123.0;

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state.last_world_data_error = None;
            state.last_loaded_world_data = Some(world_data);
        }

        launcher.update();

        assert_eq!(launcher.player, PlayerComp::default());
    }

    #[test]
    fn desktop_run_connect_arg_starts_real_client_handshake() {
        let port = free_local_port();
        let mut server = ArcNetProvider::new();
        server.host_server(port).unwrap();

        let mut launcher = run(vec![
            "mindustry-desktop".into(),
            "--connect".into(),
            format!("127.0.0.1:{port}"),
        ]);

        assert_eq!(
            launcher.connect_target,
            Some(super::DesktopConnectTarget {
                host: "127.0.0.1".into(),
                port,
            })
        );
        assert_eq!(launcher.connect_error, None);

        launcher.update();

        let state = launcher.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.connection_attempts, 1);
        assert_eq!(state.connect_events, 1);
        assert!(state.connect_packet_sent);
        assert_eq!(
            state.last_sent_connect_packet.as_ref().unwrap().name,
            "player"
        );
        drop(state);

        launcher.net_client.net_mut().disconnect();
        server.close_server();
    }

    #[test]
    fn desktop_client_connect_packet_uses_java_registered_packet_id() {
        let packet = ConnectPacket {
            version: 157,
            version_type: "official".into(),
            mods: vec!["example-mod".into()],
            name: "desktop-test".into(),
            locale: "en_US".into(),
            uuid: "AQIDBAUGBwg=".into(),
            usid: "usid".into(),
            mobile: false,
            color: 0x445566,
            uuid_crc32: None,
        };

        let bytes = PacketSerializer::write_packet_kind(&PacketKind::ConnectPacket(packet))
            .expect("desktop connect packet should encode");
        assert_eq!(bytes[0], packet_ids::CONNECT_PACKET);
        let declared_len = u16::from_be_bytes([bytes[1], bytes[2]]) as usize;
        assert!(declared_len >= PacketSerializer::COMPRESS_THRESHOLD);
        assert_eq!(bytes[3], PacketSerializer::COMPRESSION_LZ4);

        match PacketSerializer::read_envelope(&bytes)
            .expect("desktop connect envelope should decode")
        {
            PacketEnvelope::Packet {
                id,
                length,
                compression,
                payload,
            } => {
                assert_eq!(id, packet_ids::CONNECT_PACKET);
                assert_eq!(length as usize, declared_len);
                assert_eq!(compression, PacketSerializer::COMPRESSION_LZ4);
                assert_eq!(payload.len(), declared_len);
            }
            other => panic!("unexpected envelope: {other:?}"),
        }
    }
}
