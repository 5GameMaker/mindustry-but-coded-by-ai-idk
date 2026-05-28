//! 后端无关的图形帧桥接层。
//!
//! 这个模块只负责把现有的各类 render plan 聚合成一个 bundle，并同步计算
//! 统计信息；它不触碰任何 desktop/backend 相关对象。

#[cfg(not(test))]
use super::{
    BlockRendererPlan, FloorRenderPlan, FogFramePlan, MinimapOverlayPlan, OverlayRendererPlan,
    PixelatorFramePlan, RenderFramePlan,
};

#[cfg(test)]
mod test_support {
    #[derive(Debug, Clone, PartialEq)]
    pub struct RenderPass {
        pub commands: Vec<()>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct RenderFramePlan {
        pub frame_index: u64,
        pub passes: Vec<RenderPass>,
    }

    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct DarknessPlan {
        pub tiles: Vec<()>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct BlockRendererPlan {
        pub tile_passes: Vec<()>,
        pub building_passes: Vec<()>,
        pub cracks: Vec<()>,
        pub build_previews: Vec<()>,
        pub darkness: DarknessPlan,
        pub overlays: Vec<()>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct FloorRenderPlan {
        pub visible_chunks: Vec<()>,
        pub stage_plans: Vec<()>,
        pub cache_dirty_chunks: Vec<()>,
        pub pending_invalidations: Vec<()>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct FogFramePlan {
        pub stages: Vec<()>,
        pub consumed_events: Vec<()>,
        pub team_changed: bool,
        pub static_fog_enabled: bool,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct OverlayRendererPlan {
        pub core_edges: Vec<()>,
        pub build_placements: Vec<()>,
        pub commands: Vec<()>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct MinimapOverlayPlan {
        pub commands: Vec<()>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct PixelatorFramePlan {
        pub buffer_width: i32,
        pub buffer_height: i32,
    }
}

#[cfg(test)]
use test_support::{
    BlockRendererPlan, FloorRenderPlan, FogFramePlan, MinimapOverlayPlan, OverlayRendererPlan,
    PixelatorFramePlan, RenderFramePlan,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphicsFrameStats {
    pub present_plans: usize,
    pub render_passes: usize,
    pub render_commands: usize,
    pub block_tile_passes: usize,
    pub block_building_passes: usize,
    pub block_cracks: usize,
    pub block_build_previews: usize,
    pub block_darkness_tiles: usize,
    pub block_overlays: usize,
    pub floor_visible_chunks: usize,
    pub floor_stage_plans: usize,
    pub floor_cache_dirty_chunks: usize,
    pub floor_pending_invalidations: usize,
    pub fog_stages: usize,
    pub fog_consumed_events: usize,
    pub fog_team_changed_frames: usize,
    pub fog_static_fog_enabled_frames: usize,
    pub overlay_core_edges: usize,
    pub overlay_build_placements: usize,
    pub overlay_commands: usize,
    pub minimap_commands: usize,
    pub pixelator_frames: usize,
    pub pixelator_buffer_pixels: usize,
    pub total_units: usize,
}

impl GraphicsFrameStats {
    pub fn recalculate_total(&mut self) {
        self.total_units = self.present_plans
            + self.render_passes
            + self.render_commands
            + self.block_tile_passes
            + self.block_building_passes
            + self.block_cracks
            + self.block_build_previews
            + self.block_darkness_tiles
            + self.block_overlays
            + self.floor_visible_chunks
            + self.floor_stage_plans
            + self.floor_cache_dirty_chunks
            + self.floor_pending_invalidations
            + self.fog_stages
            + self.fog_consumed_events
            + self.fog_team_changed_frames
            + self.fog_static_fog_enabled_frames
            + self.overlay_core_edges
            + self.overlay_build_placements
            + self.overlay_commands
            + self.minimap_commands
            + self.pixelator_frames
            + self.pixelator_buffer_pixels;
    }

    pub fn is_empty(&self) -> bool {
        self.total_units == 0
    }
}

impl Default for GraphicsFrameStats {
    fn default() -> Self {
        Self {
            present_plans: 0,
            render_passes: 0,
            render_commands: 0,
            block_tile_passes: 0,
            block_building_passes: 0,
            block_cracks: 0,
            block_build_previews: 0,
            block_darkness_tiles: 0,
            block_overlays: 0,
            floor_visible_chunks: 0,
            floor_stage_plans: 0,
            floor_cache_dirty_chunks: 0,
            floor_pending_invalidations: 0,
            fog_stages: 0,
            fog_consumed_events: 0,
            fog_team_changed_frames: 0,
            fog_static_fog_enabled_frames: 0,
            overlay_core_edges: 0,
            overlay_build_placements: 0,
            overlay_commands: 0,
            minimap_commands: 0,
            pixelator_frames: 0,
            pixelator_buffer_pixels: 0,
            total_units: 0,
        }
    }
}

pub trait GraphicsFrameStatsSource {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats);
}

#[cfg(not(test))]
impl GraphicsFrameStatsSource for RenderFramePlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.render_passes += self.passes.len();
        stats.render_commands += self.command_count();
    }
}

#[cfg(test)]
impl GraphicsFrameStatsSource for RenderFramePlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.render_passes += self.passes.len();
        stats.render_commands += self
            .passes
            .iter()
            .map(|pass| pass.commands.len())
            .sum::<usize>();
    }
}

#[cfg(not(test))]
impl GraphicsFrameStatsSource for BlockRendererPlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.block_tile_passes += self.tile_passes.len();
        stats.block_building_passes += self.building_passes.len();
        stats.block_cracks += self.cracks.len();
        stats.block_build_previews += self.build_previews.len();
        stats.block_darkness_tiles += self.darkness.tiles.len();
        stats.block_overlays += self.overlays.len();
    }
}

#[cfg(test)]
impl GraphicsFrameStatsSource for BlockRendererPlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.block_tile_passes += self.tile_passes.len();
        stats.block_building_passes += self.building_passes.len();
        stats.block_cracks += self.cracks.len();
        stats.block_build_previews += self.build_previews.len();
        stats.block_darkness_tiles += self.darkness.tiles.len();
        stats.block_overlays += self.overlays.len();
    }
}

#[cfg(not(test))]
impl GraphicsFrameStatsSource for FloorRenderPlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.floor_visible_chunks += self.visible_chunks.len();
        stats.floor_stage_plans += self.stage_plans.len();
        stats.floor_cache_dirty_chunks += self.cache_dirty_chunks.len();
        stats.floor_pending_invalidations += self.pending_invalidations.len();
    }
}

#[cfg(test)]
impl GraphicsFrameStatsSource for FloorRenderPlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.floor_visible_chunks += self.visible_chunks.len();
        stats.floor_stage_plans += self.stage_plans.len();
        stats.floor_cache_dirty_chunks += self.cache_dirty_chunks.len();
        stats.floor_pending_invalidations += self.pending_invalidations.len();
    }
}

#[cfg(not(test))]
impl GraphicsFrameStatsSource for FogFramePlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.fog_stages += self.stages.len();
        stats.fog_consumed_events += self.consumed_events.len();
        if self.team_changed {
            stats.fog_team_changed_frames += 1;
        }
        if self.static_fog_enabled {
            stats.fog_static_fog_enabled_frames += 1;
        }
    }
}

#[cfg(test)]
impl GraphicsFrameStatsSource for FogFramePlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.fog_stages += self.stages.len();
        stats.fog_consumed_events += self.consumed_events.len();
        if self.team_changed {
            stats.fog_team_changed_frames += 1;
        }
        if self.static_fog_enabled {
            stats.fog_static_fog_enabled_frames += 1;
        }
    }
}

#[cfg(not(test))]
impl GraphicsFrameStatsSource for OverlayRendererPlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.overlay_core_edges += self.core_edges.len();
        stats.overlay_build_placements += self.build_placements.len();
        stats.overlay_commands += self.commands.len();
    }
}

#[cfg(test)]
impl GraphicsFrameStatsSource for OverlayRendererPlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.overlay_core_edges += self.core_edges.len();
        stats.overlay_build_placements += self.build_placements.len();
        stats.overlay_commands += self.commands.len();
    }
}

#[cfg(not(test))]
impl GraphicsFrameStatsSource for MinimapOverlayPlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.minimap_commands += self.commands.len();
    }
}

#[cfg(test)]
impl GraphicsFrameStatsSource for MinimapOverlayPlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.minimap_commands += self.commands.len();
    }
}

#[cfg(not(test))]
impl GraphicsFrameStatsSource for PixelatorFramePlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.pixelator_frames += 1;
        stats.pixelator_buffer_pixels +=
            (self.buffer_width.max(0) as usize) * (self.buffer_height.max(0) as usize);
    }
}

#[cfg(test)]
impl GraphicsFrameStatsSource for PixelatorFramePlan {
    fn contribute_graphics_stats(&self, stats: &mut GraphicsFrameStats) {
        stats.pixelator_frames += 1;
        stats.pixelator_buffer_pixels +=
            (self.buffer_width.max(0) as usize) * (self.buffer_height.max(0) as usize);
    }
}

fn accumulate_plan<T: GraphicsFrameStatsSource>(slot: &Option<T>, stats: &mut GraphicsFrameStats) {
    if let Some(plan) = slot {
        stats.present_plans += 1;
        plan.contribute_graphics_stats(stats);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicsFrameBundle<
    R = RenderFramePlan,
    B = BlockRendererPlan,
    F = FloorRenderPlan,
    G = FogFramePlan,
    O = OverlayRendererPlan,
    M = MinimapOverlayPlan,
    P = PixelatorFramePlan,
> {
    pub render_frame: Option<R>,
    pub block_renderer: Option<B>,
    pub floor_renderer: Option<F>,
    pub fog_frame: Option<G>,
    pub overlay_renderer: Option<O>,
    pub minimap_overlay: Option<M>,
    pub pixelator: Option<P>,
    pub stats: GraphicsFrameStats,
}

impl<R, B, F, G, O, M, P> Default for GraphicsFrameBundle<R, B, F, G, O, M, P> {
    fn default() -> Self {
        Self {
            render_frame: None,
            block_renderer: None,
            floor_renderer: None,
            fog_frame: None,
            overlay_renderer: None,
            minimap_overlay: None,
            pixelator: None,
            stats: GraphicsFrameStats::default(),
        }
    }
}

impl<R, B, F, G, O, M, P> GraphicsFrameBundle<R, B, F, G, O, M, P> {
    pub fn into_stats(self) -> GraphicsFrameStats {
        self.stats
    }
}

impl<R, B, F, G, O, M, P> GraphicsFrameBundle<R, B, F, G, O, M, P>
where
    R: GraphicsFrameStatsSource,
    B: GraphicsFrameStatsSource,
    F: GraphicsFrameStatsSource,
    G: GraphicsFrameStatsSource,
    O: GraphicsFrameStatsSource,
    M: GraphicsFrameStatsSource,
    P: GraphicsFrameStatsSource,
{
    pub fn rebuild_stats(&mut self) -> &mut Self {
        let mut stats = GraphicsFrameStats::default();

        accumulate_plan(&self.render_frame, &mut stats);
        accumulate_plan(&self.block_renderer, &mut stats);
        accumulate_plan(&self.floor_renderer, &mut stats);
        accumulate_plan(&self.fog_frame, &mut stats);
        accumulate_plan(&self.overlay_renderer, &mut stats);
        accumulate_plan(&self.minimap_overlay, &mut stats);
        accumulate_plan(&self.pixelator, &mut stats);

        stats.recalculate_total();
        self.stats = stats;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.render_frame.is_none()
            && self.block_renderer.is_none()
            && self.floor_renderer.is_none()
            && self.fog_frame.is_none()
            && self.overlay_renderer.is_none()
            && self.minimap_overlay.is_none()
            && self.pixelator.is_none()
            && self.stats.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrameComposer<
    R = RenderFramePlan,
    B = BlockRendererPlan,
    F = FloorRenderPlan,
    G = FogFramePlan,
    O = OverlayRendererPlan,
    M = MinimapOverlayPlan,
    P = PixelatorFramePlan,
> {
    bundle: GraphicsFrameBundle<R, B, F, G, O, M, P>,
}

impl<R, B, F, G, O, M, P> Default for FrameComposer<R, B, F, G, O, M, P> {
    fn default() -> Self {
        Self {
            bundle: GraphicsFrameBundle::default(),
        }
    }
}

impl<R, B, F, G, O, M, P> FrameComposer<R, B, F, G, O, M, P> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bundle(&self) -> &GraphicsFrameBundle<R, B, F, G, O, M, P> {
        &self.bundle
    }

    pub fn bundle_mut(&mut self) -> &mut GraphicsFrameBundle<R, B, F, G, O, M, P> {
        &mut self.bundle
    }

    pub fn stats(&self) -> &GraphicsFrameStats {
        &self.bundle.stats
    }
}

impl<R, B, F, G, O, M, P> FrameComposer<R, B, F, G, O, M, P>
where
    R: GraphicsFrameStatsSource,
    B: GraphicsFrameStatsSource,
    F: GraphicsFrameStatsSource,
    G: GraphicsFrameStatsSource,
    O: GraphicsFrameStatsSource,
    M: GraphicsFrameStatsSource,
    P: GraphicsFrameStatsSource,
{
    pub fn rebuild_stats(&mut self) -> &mut Self {
        self.bundle.rebuild_stats();
        self
    }

    pub fn set_render_frame(&mut self, plan: R) -> &mut Self {
        self.bundle.render_frame = Some(plan);
        self.rebuild_stats()
    }

    pub fn set_block_renderer(&mut self, plan: B) -> &mut Self {
        self.bundle.block_renderer = Some(plan);
        self.rebuild_stats()
    }

    pub fn set_floor_renderer(&mut self, plan: F) -> &mut Self {
        self.bundle.floor_renderer = Some(plan);
        self.rebuild_stats()
    }

    pub fn set_fog_frame(&mut self, plan: G) -> &mut Self {
        self.bundle.fog_frame = Some(plan);
        self.rebuild_stats()
    }

    pub fn set_overlay_renderer(&mut self, plan: O) -> &mut Self {
        self.bundle.overlay_renderer = Some(plan);
        self.rebuild_stats()
    }

    pub fn set_minimap_overlay(&mut self, plan: M) -> &mut Self {
        self.bundle.minimap_overlay = Some(plan);
        self.rebuild_stats()
    }

    pub fn set_pixelator(&mut self, plan: P) -> &mut Self {
        self.bundle.pixelator = Some(plan);
        self.rebuild_stats()
    }

    pub fn finish(mut self) -> GraphicsFrameBundle<R, B, F, G, O, M, P> {
        self.bundle.rebuild_stats();
        self.bundle
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderBridge<
    R = RenderFramePlan,
    B = BlockRendererPlan,
    F = FloorRenderPlan,
    G = FogFramePlan,
    O = OverlayRendererPlan,
    M = MinimapOverlayPlan,
    P = PixelatorFramePlan,
> {
    composer: FrameComposer<R, B, F, G, O, M, P>,
}

impl<R, B, F, G, O, M, P> Default for RenderBridge<R, B, F, G, O, M, P> {
    fn default() -> Self {
        Self {
            composer: FrameComposer::default(),
        }
    }
}

impl<R, B, F, G, O, M, P> RenderBridge<R, B, F, G, O, M, P> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn composer(&self) -> &FrameComposer<R, B, F, G, O, M, P> {
        &self.composer
    }

    pub fn composer_mut(&mut self) -> &mut FrameComposer<R, B, F, G, O, M, P> {
        &mut self.composer
    }

    pub fn bundle(&self) -> &GraphicsFrameBundle<R, B, F, G, O, M, P> {
        self.composer.bundle()
    }

    pub fn stats(&self) -> &GraphicsFrameStats {
        self.composer.stats()
    }
}

impl<R, B, F, G, O, M, P> RenderBridge<R, B, F, G, O, M, P>
where
    R: GraphicsFrameStatsSource,
    B: GraphicsFrameStatsSource,
    F: GraphicsFrameStatsSource,
    G: GraphicsFrameStatsSource,
    O: GraphicsFrameStatsSource,
    M: GraphicsFrameStatsSource,
    P: GraphicsFrameStatsSource,
{
    pub fn rebuild_stats(&mut self) -> &mut Self {
        self.composer.rebuild_stats();
        self
    }

    pub fn set_render_frame(&mut self, plan: R) -> &mut Self {
        self.composer.set_render_frame(plan);
        self
    }

    pub fn set_block_renderer(&mut self, plan: B) -> &mut Self {
        self.composer.set_block_renderer(plan);
        self
    }

    pub fn set_floor_renderer(&mut self, plan: F) -> &mut Self {
        self.composer.set_floor_renderer(plan);
        self
    }

    pub fn set_fog_frame(&mut self, plan: G) -> &mut Self {
        self.composer.set_fog_frame(plan);
        self
    }

    pub fn set_overlay_renderer(&mut self, plan: O) -> &mut Self {
        self.composer.set_overlay_renderer(plan);
        self
    }

    pub fn set_minimap_overlay(&mut self, plan: M) -> &mut Self {
        self.composer.set_minimap_overlay(plan);
        self
    }

    pub fn set_pixelator(&mut self, plan: P) -> &mut Self {
        self.composer.set_pixelator(plan);
        self
    }

    pub fn finish(self) -> GraphicsFrameBundle<R, B, F, G, O, M, P> {
        self.composer.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_frame(pass_commands: &[usize]) -> RenderFramePlan {
        RenderFramePlan {
            frame_index: 7,
            passes: pass_commands
                .iter()
                .map(|&commands| super::test_support::RenderPass {
                    commands: vec![(); commands],
                })
                .collect(),
        }
    }

    fn block_plan(
        tile_passes: usize,
        building_passes: usize,
        cracks: usize,
        build_previews: usize,
        darkness_tiles: usize,
        overlays: usize,
    ) -> BlockRendererPlan {
        BlockRendererPlan {
            tile_passes: vec![(); tile_passes],
            building_passes: vec![(); building_passes],
            cracks: vec![(); cracks],
            build_previews: vec![(); build_previews],
            darkness: super::test_support::DarknessPlan {
                tiles: vec![(); darkness_tiles],
            },
            overlays: vec![(); overlays],
        }
    }

    fn floor_plan(
        visible_chunks: usize,
        stage_plans: usize,
        cache_dirty_chunks: usize,
        pending_invalidations: usize,
    ) -> FloorRenderPlan {
        FloorRenderPlan {
            visible_chunks: vec![(); visible_chunks],
            stage_plans: vec![(); stage_plans],
            cache_dirty_chunks: vec![(); cache_dirty_chunks],
            pending_invalidations: vec![(); pending_invalidations],
        }
    }

    fn fog_plan(
        stages: usize,
        events: usize,
        team_changed: bool,
        static_enabled: bool,
    ) -> FogFramePlan {
        FogFramePlan {
            stages: vec![(); stages],
            consumed_events: vec![(); events],
            team_changed,
            static_fog_enabled: static_enabled,
        }
    }

    fn overlay_plan(edges: usize, placements: usize, commands: usize) -> OverlayRendererPlan {
        OverlayRendererPlan {
            core_edges: vec![(); edges],
            build_placements: vec![(); placements],
            commands: vec![(); commands],
        }
    }

    fn minimap_plan(commands: usize) -> MinimapOverlayPlan {
        MinimapOverlayPlan {
            commands: vec![(); commands],
        }
    }

    fn pixelator_plan(width: i32, height: i32) -> PixelatorFramePlan {
        PixelatorFramePlan {
            buffer_width: width,
            buffer_height: height,
        }
    }

    fn total_units(stats: &GraphicsFrameStats) -> usize {
        stats.present_plans
            + stats.render_passes
            + stats.render_commands
            + stats.block_tile_passes
            + stats.block_building_passes
            + stats.block_cracks
            + stats.block_build_previews
            + stats.block_darkness_tiles
            + stats.block_overlays
            + stats.floor_visible_chunks
            + stats.floor_stage_plans
            + stats.floor_cache_dirty_chunks
            + stats.floor_pending_invalidations
            + stats.fog_stages
            + stats.fog_consumed_events
            + stats.fog_team_changed_frames
            + stats.fog_static_fog_enabled_frames
            + stats.overlay_core_edges
            + stats.overlay_build_placements
            + stats.overlay_commands
            + stats.minimap_commands
            + stats.pixelator_frames
            + stats.pixelator_buffer_pixels
    }

    #[test]
    fn empty_bridge_starts_with_zeroed_stats() {
        let bridge: RenderBridge = RenderBridge::new();
        let stats = bridge.finish().into_stats();

        assert!(stats.is_empty());
        assert_eq!(stats.present_plans, 0);
        assert_eq!(stats.total_units, 0);
    }

    #[test]
    fn composer_aggregates_all_plans_into_bundle_and_stats() {
        let mut composer: FrameComposer = FrameComposer::new();

        composer
            .set_render_frame(render_frame(&[1, 2]))
            .set_block_renderer(block_plan(2, 1, 3, 4, 5, 6))
            .set_floor_renderer(floor_plan(7, 8, 9, 10))
            .set_fog_frame(fog_plan(11, 12, true, false))
            .set_overlay_renderer(overlay_plan(13, 14, 15))
            .set_minimap_overlay(minimap_plan(16))
            .set_pixelator(pixelator_plan(17, 18));

        let bundle = composer.finish();

        assert!(bundle.render_frame.is_some());
        assert!(bundle.block_renderer.is_some());
        assert!(bundle.floor_renderer.is_some());
        assert!(bundle.fog_frame.is_some());
        assert!(bundle.overlay_renderer.is_some());
        assert!(bundle.minimap_overlay.is_some());
        assert!(bundle.pixelator.is_some());

        assert_eq!(bundle.stats.present_plans, 7);
        assert_eq!(bundle.stats.render_passes, 2);
        assert_eq!(bundle.stats.render_commands, 3);
        assert_eq!(bundle.stats.block_tile_passes, 2);
        assert_eq!(bundle.stats.block_building_passes, 1);
        assert_eq!(bundle.stats.block_cracks, 3);
        assert_eq!(bundle.stats.block_build_previews, 4);
        assert_eq!(bundle.stats.block_darkness_tiles, 5);
        assert_eq!(bundle.stats.block_overlays, 6);
        assert_eq!(bundle.stats.floor_visible_chunks, 7);
        assert_eq!(bundle.stats.floor_stage_plans, 8);
        assert_eq!(bundle.stats.floor_cache_dirty_chunks, 9);
        assert_eq!(bundle.stats.floor_pending_invalidations, 10);
        assert_eq!(bundle.stats.fog_stages, 11);
        assert_eq!(bundle.stats.fog_consumed_events, 12);
        assert_eq!(bundle.stats.fog_team_changed_frames, 1);
        assert_eq!(bundle.stats.fog_static_fog_enabled_frames, 0);
        assert_eq!(bundle.stats.overlay_core_edges, 13);
        assert_eq!(bundle.stats.overlay_build_placements, 14);
        assert_eq!(bundle.stats.overlay_commands, 15);
        assert_eq!(bundle.stats.minimap_commands, 16);
        assert_eq!(bundle.stats.pixelator_frames, 1);
        assert_eq!(bundle.stats.pixelator_buffer_pixels, 17 * 18);
        assert_eq!(bundle.stats.total_units, total_units(&bundle.stats));

        let stats = bundle.into_stats();
        assert_eq!(stats.present_plans, 7);
        assert_eq!(stats.render_passes, 2);
        assert_eq!(stats.render_commands, 3);
        assert_eq!(stats.block_tile_passes, 2);
        assert_eq!(stats.block_building_passes, 1);
        assert_eq!(stats.block_cracks, 3);
        assert_eq!(stats.block_build_previews, 4);
        assert_eq!(stats.block_darkness_tiles, 5);
        assert_eq!(stats.block_overlays, 6);
        assert_eq!(stats.floor_visible_chunks, 7);
        assert_eq!(stats.floor_stage_plans, 8);
        assert_eq!(stats.floor_cache_dirty_chunks, 9);
        assert_eq!(stats.floor_pending_invalidations, 10);
        assert_eq!(stats.fog_stages, 11);
        assert_eq!(stats.fog_consumed_events, 12);
        assert_eq!(stats.fog_team_changed_frames, 1);
        assert_eq!(stats.fog_static_fog_enabled_frames, 0);
        assert_eq!(stats.overlay_core_edges, 13);
        assert_eq!(stats.overlay_build_placements, 14);
        assert_eq!(stats.overlay_commands, 15);
        assert_eq!(stats.minimap_commands, 16);
        assert_eq!(stats.pixelator_frames, 1);
        assert_eq!(stats.pixelator_buffer_pixels, 17 * 18);
        assert_eq!(stats.total_units, total_units(&stats));
    }

    #[test]
    fn later_plans_replace_earlier_ones_before_finish() {
        let mut bridge: RenderBridge = RenderBridge::new();

        bridge
            .set_render_frame(render_frame(&[1]))
            .set_render_frame(render_frame(&[2, 1, 1]))
            .set_block_renderer(block_plan(1, 0, 0, 0, 0, 0));

        let bundle = bridge.finish();

        assert_eq!(bundle.stats.render_passes, 3);
        assert_eq!(bundle.stats.render_commands, 4);
        assert_eq!(bundle.stats.block_tile_passes, 1);
        assert_eq!(bundle.stats.present_plans, 2);
        assert_eq!(bundle.stats.total_units, total_units(&bundle.stats));
    }

    #[test]
    fn bridge_carries_pixelator_as_frame_wrapper_slot() {
        let mut bridge: RenderBridge = RenderBridge::new();

        bridge
            .set_render_frame(render_frame(&[1]))
            .set_pixelator(pixelator_plan(320, 240));

        let bundle = bridge.finish();

        assert!(bundle.render_frame.is_some());
        assert!(bundle.pixelator.is_some());
        assert_eq!(bundle.stats.present_plans, 2);
        assert_eq!(bundle.stats.pixelator_frames, 1);
        assert_eq!(bundle.stats.pixelator_buffer_pixels, 320 * 240);
        assert_eq!(bundle.stats.render_passes, 1);
        assert_eq!(bundle.stats.render_commands, 1);
    }
}
