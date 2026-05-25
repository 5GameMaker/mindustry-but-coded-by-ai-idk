//! Minimal game-runtime facade that connects `GameState` frame advancement with
//! migrated building runtime slices.
//!
//! Upstream Java drives buildings from `Logic.update()` through `Groups.update()`
//! and finally `BuildingComp.update()/updateTile()`. The Rust port does not have
//! the full `Groups.build` owner yet, so this facade is the narrow runtime seam:
//! it owns game-wide sidecar stores and dispatches externally supplied building
//! slices from the real `GameState` frame source.

use crate::mindustry::{
    content::blocks::BlockDef,
    core::content_loader::ContentLoader,
    core::game_state::GameState,
    ctype::ContentId,
    entities::{
        bullet::BulletType,
        comp::{BuildingComp, BulletComp, UnitComp},
    },
    vars::TILE_SIZE,
    world::blocks::defense::{
        effect_block_frame_input_from_game_update, effect_block_update_building_slice_with_stores,
        EffectBlockFrameBatchReport, EffectBlockFrameBatchResources, EffectBlockRuntimeStateStore,
        EffectBlockTimerStateStore,
    },
    world::{footprint_tiles, Tile},
};

pub struct GameRuntimeEffectResources<'a, 'b> {
    pub buildings: &'a mut [BuildingComp],
    pub bullets: &'a mut [BulletComp],
    pub bullet_type: &'a mut dyn FnMut(ContentId) -> Option<&'b BulletType>,
    pub units: &'a mut [UnitComp],
    pub suppressed: &'a mut dyn FnMut(&BuildingComp) -> bool,
    pub force_coolant: &'a mut dyn FnMut(&BuildingComp) -> (f32, f32),
    pub spark_random: &'a mut dyn for<'u> FnMut(&'u UnitComp) -> f32,
}

pub struct GameRuntimeOwnedEffectResources<'a, 'b> {
    pub bullets: &'a mut [BulletComp],
    pub bullet_type: &'a mut dyn FnMut(ContentId) -> Option<&'b BulletType>,
    pub units: &'a mut [UnitComp],
    pub suppressed: &'a mut dyn FnMut(&BuildingComp) -> bool,
    pub force_coolant: &'a mut dyn FnMut(&BuildingComp) -> (f32, f32),
    pub spark_random: &'a mut dyn for<'u> FnMut(&'u UnitComp) -> f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameRuntime {
    pub state: GameState,
    pub buildings: Vec<BuildingComp>,
    pub effect_runtime_store: EffectBlockRuntimeStateStore,
    pub effect_timer_store: EffectBlockTimerStateStore,
}

impl Default for GameRuntime {
    fn default() -> Self {
        Self::new(GameState::new())
    }
}

impl GameRuntime {
    pub fn new(state: GameState) -> Self {
        Self {
            state,
            buildings: Vec::new(),
            effect_runtime_store: EffectBlockRuntimeStateStore::new(),
            effect_timer_store: EffectBlockTimerStateStore::new(),
        }
    }

    pub fn buildings(&self) -> &[BuildingComp] {
        &self.buildings
    }

    pub fn buildings_mut(&mut self) -> &mut [BuildingComp] {
        &mut self.buildings
    }

    pub fn add_building(&mut self, building: BuildingComp) -> usize {
        if let Some(index) = self
            .buildings
            .iter()
            .position(|existing| existing.tile_pos == building.tile_pos)
        {
            let previous = self.buildings[index].clone();
            self.clear_world_refs_for_building(&previous);
            self.buildings[index] = building;
            self.sync_world_footprint_refs(index);
            index
        } else {
            let index = self.buildings.len();
            self.buildings.push(building);
            self.sync_world_footprint_refs(index);
            index
        }
    }

    pub fn clear_buildings(&mut self) {
        self.buildings.clear();
        self.state.world.clear_buildings();
    }

    pub fn clear_world_refs_for_building(&mut self, building: &BuildingComp) -> usize {
        let tile_pos = building.tile_pos;
        let mut cleared = 0;
        for (x, y) in footprint_tiles(building.tile_x(), building.tile_y(), building.block.size) {
            let Some(tile) = self.state.world.tile_mut(x, y) else {
                continue;
            };
            if tile.build.is_some_and(|build| build.tile_pos == tile_pos) {
                tile.build = None;
                tile.block = Tile::AIR;
                cleared += 1;
            }
        }
        cleared
    }

    pub fn sync_world_center_ref(&mut self, index: usize) -> bool {
        self.sync_world_footprint_refs(index) > 0
    }

    pub fn sync_world_footprint_refs(&mut self, index: usize) -> usize {
        let Some(building) = self.buildings.get(index) else {
            return 0;
        };
        let block_id = building.block.id;
        let block_size = building.block.size;
        let build_ref = building.pos_ref();
        let center_x = building.tile_x();
        let center_y = building.tile_y();

        let mut synced = 0;
        for (x, y) in footprint_tiles(center_x, center_y, block_size) {
            let Some(tile) = self.state.world.tile_mut(x, y) else {
                continue;
            };
            tile.block = block_id;
            tile.build = Some(build_ref);
            synced += 1;
        }
        synced
    }

    pub fn reset_effect_block_sidecars(&mut self) {
        self.effect_runtime_store.clear();
        self.effect_timer_store.clear();
    }

    pub fn refresh_owned_building_update_permissions(&mut self, content: &ContentLoader) -> usize {
        let env = self.state.rules.env;
        let mut disabled = 0;
        for building in &mut self.buildings {
            let was_enabled = building.enabled;
            let supports_env = content
                .block(building.block.id)
                .is_some_and(|block| block.supports_env(env));
            let in_bounds = self.state.world.tile_pos(building.tile_pos).is_some();
            if !building.check_allow_update(supports_env, in_bounds) {
                building.enabled = false;
                if was_enabled {
                    disabled += 1;
                }
            }
        }
        disabled
    }

    /// Consumes pending world-load lifecycle markers and resets tile-position keyed
    /// sidecars once. This mirrors the Java requirement that a fresh world load
    /// cannot reuse stale `Building` runtime state from a previous map.
    pub fn consume_world_load_events_and_reset_sidecars(&mut self) -> bool {
        let should_reset = !self.state.world.load_events().is_empty();
        if should_reset {
            self.reset_effect_block_sidecars();
            self.buildings.clear();
            self.state.world.clear_load_events();
        }
        should_reset
    }

    pub fn advance_and_dispatch_effect_blocks<'a, 'b>(
        &'a mut self,
        content: &ContentLoader,
        delta_seconds: f32,
        resources: GameRuntimeEffectResources<'a, 'b>,
    ) -> Option<EffectBlockFrameBatchReport> {
        self.consume_world_load_events_and_reset_sidecars();

        let advanced = self.state.advance_game_update_frame(delta_seconds);
        let frame = effect_block_frame_input_from_game_update(
            advanced,
            TILE_SIZE as f32,
            self.state.rules.fog,
            self.state.rules.static_fog,
        )?;

        for building in resources.buildings.iter_mut() {
            let can_overdrive = content
                .block(building.block.id)
                .map(BlockDef::can_overdrive)
                .unwrap_or(false);
            building.advance_update_timing(frame.delta, can_overdrive);
        }

        let mut batch_resources = EffectBlockFrameBatchResources {
            fog_control: Some(&mut self.state.fog_control),
            bullets: resources.bullets,
            bullet_type: resources.bullet_type,
            units: resources.units,
            suppressed: resources.suppressed,
            force_coolant: resources.force_coolant,
            spark_random: resources.spark_random,
        };

        Some(effect_block_update_building_slice_with_stores(
            &mut self.effect_runtime_store,
            &mut self.effect_timer_store,
            content,
            resources.buildings,
            frame,
            &mut batch_resources,
        ))
    }

    pub fn advance_owned_effect_blocks<'a, 'b>(
        &'a mut self,
        content: &ContentLoader,
        delta_seconds: f32,
        resources: GameRuntimeOwnedEffectResources<'a, 'b>,
    ) -> Option<EffectBlockFrameBatchReport> {
        self.consume_world_load_events_and_reset_sidecars();

        let advanced = self.state.advance_game_update_frame(delta_seconds);
        let frame = effect_block_frame_input_from_game_update(
            advanced,
            TILE_SIZE as f32,
            self.state.rules.fog,
            self.state.rules.static_fog,
        )?;

        self.refresh_owned_building_update_permissions(content);

        for building in self.buildings.iter_mut() {
            let can_overdrive = content
                .block(building.block.id)
                .map(BlockDef::can_overdrive)
                .unwrap_or(false);
            building.advance_update_timing(frame.delta, can_overdrive);
        }

        let mut batch_resources = EffectBlockFrameBatchResources {
            fog_control: Some(&mut self.state.fog_control),
            bullets: resources.bullets,
            bullet_type: resources.bullet_type,
            units: resources.units,
            suppressed: resources.suppressed,
            force_coolant: resources.force_coolant,
            spark_random: resources.spark_random,
        };

        Some(effect_block_update_building_slice_with_stores(
            &mut self.effect_runtime_store,
            &mut self.effect_timer_store,
            content,
            self.buildings.as_mut_slice(),
            frame,
            &mut batch_resources,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::{
        core::GameStateState,
        io::TeamId,
        world::{
            blocks::defense::EffectBlockRuntimeState, footprint_tiles, point2_pack, Block, Tile,
        },
    };

    fn noop_resources<'a, 'b>(
        buildings: &'a mut [BuildingComp],
        bullets: &'a mut [BulletComp],
        units: &'a mut [UnitComp],
        bullet_type: &'a mut dyn FnMut(ContentId) -> Option<&'b BulletType>,
        suppressed: &'a mut dyn FnMut(&BuildingComp) -> bool,
        force_coolant: &'a mut dyn FnMut(&BuildingComp) -> (f32, f32),
        spark_random: &'a mut dyn for<'u> FnMut(&'u UnitComp) -> f32,
    ) -> GameRuntimeEffectResources<'a, 'b> {
        GameRuntimeEffectResources {
            buildings,
            bullets,
            bullet_type,
            units,
            suppressed,
            force_coolant,
            spark_random,
        }
    }

    fn owned_noop_resources<'a, 'b>(
        bullets: &'a mut [BulletComp],
        units: &'a mut [UnitComp],
        bullet_type: &'a mut dyn FnMut(ContentId) -> Option<&'b BulletType>,
        suppressed: &'a mut dyn FnMut(&BuildingComp) -> bool,
        force_coolant: &'a mut dyn FnMut(&BuildingComp) -> (f32, f32),
        spark_random: &'a mut dyn for<'u> FnMut(&'u UnitComp) -> f32,
    ) -> GameRuntimeOwnedEffectResources<'a, 'b> {
        GameRuntimeOwnedEffectResources {
            bullets,
            bullet_type,
            units,
            suppressed,
            force_coolant,
            spark_random,
        }
    }

    #[test]
    fn game_runtime_advance_frame_drives_effect_block_batch_dispatch() {
        let content = ContentLoader::create_base_content().unwrap();
        let mend_def = content.block_by_name("mend-projector").unwrap();
        let mend_block = match mend_def {
            BlockDef::Effect(effect) => effect,
            _ => unreachable!(),
        };
        let silicon = mend_block.boost_items[0].item;
        let mut mend = BuildingComp::new(point2_pack(31, 9), mend_def.base().clone(), TeamId(1));
        mend.efficiency = 1.0;
        mend.optional_efficiency = 1.0;
        mend.items.as_mut().unwrap().set(silicon, 1);
        let mut buildings = vec![mend];

        let mut runtime = GameRuntime::default();
        runtime.state.set(GameStateState::Playing);
        runtime.state.tick = mend_block.use_time as f64 - 30.0;

        let mut bullets = Vec::new();
        let mut units = Vec::new();
        let mut bullet_type = |_: ContentId| -> Option<&BulletType> { None };
        let mut suppressed = |_: &BuildingComp| false;
        let mut force_coolant = |_: &BuildingComp| (0.0, 0.0);
        let mut spark_random = |_: &UnitComp| 1.0;

        let batch = runtime
            .advance_and_dispatch_effect_blocks(
                &content,
                0.5,
                noop_resources(
                    &mut buildings,
                    &mut bullets,
                    &mut units,
                    &mut bullet_type,
                    &mut suppressed,
                    &mut force_coolant,
                    &mut spark_random,
                ),
            )
            .unwrap();

        assert_eq!(runtime.state.update_id, 1);
        assert_eq!(runtime.state.tick, mend_block.use_time as f64);
        assert_eq!(batch.visited_buildings, 1);
        assert_eq!(batch.effect_candidates, 1);
        assert_eq!(batch.reports.len(), 1);
        assert_eq!(buildings[0].items.as_ref().unwrap().get(silicon), 0);
        assert!(matches!(
            runtime.effect_runtime_store.get(buildings[0].tile_pos),
            Some(EffectBlockRuntimeState::Projector(_))
        ));
        assert!(runtime
            .effect_timer_store
            .get(buildings[0].tile_pos)
            .is_some());
    }

    #[test]
    fn game_runtime_skips_effect_dispatch_when_state_does_not_advance() {
        let content = ContentLoader::create_base_content().unwrap();
        let router = content.block_by_name("router").unwrap();
        let mut building = BuildingComp::new(point2_pack(32, 9), router.base().clone(), TeamId(1));
        building.apply_boost(2.0, 60.0);
        let mut buildings = vec![building];

        let mut runtime = GameRuntime::default();
        let mut bullets = Vec::new();
        let mut units = Vec::new();
        let mut bullet_type = |_: ContentId| -> Option<&BulletType> { None };
        let mut suppressed = |_: &BuildingComp| false;
        let mut force_coolant = |_: &BuildingComp| (0.0, 0.0);
        let mut spark_random = |_: &UnitComp| 1.0;

        let batch = runtime.advance_and_dispatch_effect_blocks(
            &content,
            0.5,
            noop_resources(
                &mut buildings,
                &mut bullets,
                &mut units,
                &mut bullet_type,
                &mut suppressed,
                &mut force_coolant,
                &mut spark_random,
            ),
        );

        assert!(batch.is_none());
        assert_eq!(runtime.state.update_id, 0);
        assert_eq!(buildings[0].time_scale, 2.0);
        assert_eq!(buildings[0].time_scale_duration, 60.0);
    }

    #[test]
    fn game_runtime_resets_effect_sidecars_after_world_load_events() {
        let content = ContentLoader::create_base_content().unwrap();
        let mend_def = content.block_by_name("mend-projector").unwrap();
        let mend_block = match mend_def {
            BlockDef::Effect(effect) => effect,
            _ => unreachable!(),
        };
        let silicon = mend_block.boost_items[0].item;
        let mut mend = BuildingComp::new(point2_pack(33, 9), mend_def.base().clone(), TeamId(1));
        mend.efficiency = 1.0;
        mend.optional_efficiency = 1.0;
        mend.items.as_mut().unwrap().set(silicon, 1);
        let mut buildings = vec![mend];

        let mut runtime = GameRuntime::default();
        runtime.state.set(GameStateState::Playing);
        runtime.state.tick = mend_block.use_time as f64 - 30.0;

        let mut bullets = Vec::new();
        let mut units = Vec::new();
        let mut bullet_type = |_: ContentId| -> Option<&BulletType> { None };
        let mut suppressed = |_: &BuildingComp| false;
        let mut force_coolant = |_: &BuildingComp| (0.0, 0.0);
        let mut spark_random = |_: &UnitComp| 1.0;

        runtime
            .advance_and_dispatch_effect_blocks(
                &content,
                0.5,
                noop_resources(
                    &mut buildings,
                    &mut bullets,
                    &mut units,
                    &mut bullet_type,
                    &mut suppressed,
                    &mut force_coolant,
                    &mut spark_random,
                ),
            )
            .unwrap();
        assert!(runtime
            .effect_runtime_store
            .get(buildings[0].tile_pos)
            .is_some());

        runtime.state.world.load_generator(1, 1, |_| {});
        assert!(runtime.consume_world_load_events_and_reset_sidecars());
        assert!(runtime
            .effect_runtime_store
            .get(buildings[0].tile_pos)
            .is_none());
        assert!(runtime
            .effect_timer_store
            .get(buildings[0].tile_pos)
            .is_none());
        assert!(runtime.state.world.load_events().is_empty());
    }

    #[test]
    fn game_runtime_owned_buildings_sync_world_refs_and_dispatch_effects() {
        let content = ContentLoader::create_base_content().unwrap();
        let mend_def = content.block_by_name("mend-projector").unwrap();
        let mend_block = match mend_def {
            BlockDef::Effect(effect) => effect,
            _ => unreachable!(),
        };
        let silicon = mend_block.boost_items[0].item;
        let tile_pos = point2_pack(34, 9);
        let mut mend = BuildingComp::new(tile_pos, mend_def.base().clone(), TeamId(1));
        mend.efficiency = 1.0;
        mend.optional_efficiency = 1.0;
        mend.items.as_mut().unwrap().set(silicon, 1);
        mend.apply_boost(2.0, 60.0);

        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(64, 64);
        let index = runtime.add_building(mend);
        assert_eq!(index, 0);
        let tile = runtime.state.world.tile(34, 9).unwrap();
        assert_eq!(tile.block, mend_def.base().id);
        assert_eq!(tile.build.unwrap().tile_pos, tile_pos);

        runtime.state.set(GameStateState::Playing);
        runtime.state.tick = mend_block.use_time as f64 - 30.0;

        let mut bullets = Vec::new();
        let mut units = Vec::new();
        let mut bullet_type = |_: ContentId| -> Option<&BulletType> { None };
        let mut suppressed = |_: &BuildingComp| false;
        let mut force_coolant = |_: &BuildingComp| (0.0, 0.0);
        let mut spark_random = |_: &UnitComp| 1.0;

        let batch = runtime
            .advance_owned_effect_blocks(
                &content,
                0.5,
                owned_noop_resources(
                    &mut bullets,
                    &mut units,
                    &mut bullet_type,
                    &mut suppressed,
                    &mut force_coolant,
                    &mut spark_random,
                ),
            )
            .unwrap();

        assert_eq!(batch.visited_buildings, 1);
        assert_eq!(batch.effect_candidates, 1);
        assert_eq!(batch.reports.len(), 1);
        assert_eq!(
            runtime.buildings()[0].items.as_ref().unwrap().get(silicon),
            0
        );
        assert_eq!(runtime.buildings()[0].time_scale, 2.0);
        assert_eq!(runtime.buildings()[0].time_scale_duration, 30.0);
        assert!(runtime.effect_runtime_store.get(tile_pos).is_some());
        assert!(runtime.effect_timer_store.get(tile_pos).is_some());
    }

    #[test]
    fn game_runtime_owned_buildings_sync_multiblock_footprint_refs() {
        let mut large_block = Block::new(30_000, "test-large");
        large_block.size = 3;
        let mut small_block = Block::new(30_001, "test-small");
        small_block.size = 1;
        let tile_pos = point2_pack(10, 10);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(32, 32);

        runtime.add_building(BuildingComp::new(tile_pos, large_block.clone(), TeamId(1)));
        for (x, y) in footprint_tiles(10, 10, 3) {
            let tile = runtime.state.world.tile(x, y).unwrap();
            assert_eq!(tile.block, large_block.id);
            assert_eq!(tile.build.unwrap().tile_pos, tile_pos);
        }

        runtime.add_building(BuildingComp::new(tile_pos, small_block.clone(), TeamId(1)));
        let center = runtime.state.world.tile(10, 10).unwrap();
        assert_eq!(center.block, small_block.id);
        assert_eq!(center.build.unwrap().tile_pos, tile_pos);

        let old_edge = runtime.state.world.tile(9, 9).unwrap();
        assert_eq!(old_edge.block, Tile::AIR);
        assert!(old_edge.build.is_none());
    }

    #[test]
    fn game_runtime_world_load_events_clear_owned_buildings_and_sidecars() {
        let content = ContentLoader::create_base_content().unwrap();
        let mend_def = content.block_by_name("mend-projector").unwrap();
        let tile_pos = point2_pack(35, 9);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(64, 64);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            mend_def.base().clone(),
            TeamId(1),
        ));
        let building_snapshot = runtime.buildings()[0].clone();
        runtime
            .effect_runtime_store
            .ensure_for_building(&content, &building_snapshot, 0.0);
        runtime
            .effect_timer_store
            .ensure_for_building(&content, &building_snapshot);
        assert_eq!(runtime.buildings().len(), 1);
        assert!(runtime.effect_runtime_store.get(tile_pos).is_some());
        assert!(runtime.effect_timer_store.get(tile_pos).is_some());

        runtime.state.world.load_generator(1, 1, |_| {});
        assert!(runtime.consume_world_load_events_and_reset_sidecars());
        assert!(runtime.buildings().is_empty());
        assert!(runtime.effect_runtime_store.is_empty());
        assert!(runtime.effect_timer_store.is_empty());
    }

    #[test]
    fn game_runtime_refresh_owned_building_permissions_disables_out_of_bounds_buildings() {
        let content = ContentLoader::create_base_content().unwrap();
        let mend_def = content.block_by_name("mend-projector").unwrap();
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(16, 16);
        runtime.add_building(BuildingComp::new(
            point2_pack(4, 4),
            mend_def.base().clone(),
            TeamId(1),
        ));
        runtime.add_building(BuildingComp::new(
            point2_pack(40, 40),
            mend_def.base().clone(),
            TeamId(1),
        ));

        assert_eq!(
            runtime.refresh_owned_building_update_permissions(&content),
            1
        );
        assert!(runtime.buildings()[0].enabled);
        assert!(!runtime.buildings()[1].enabled);
    }
}
