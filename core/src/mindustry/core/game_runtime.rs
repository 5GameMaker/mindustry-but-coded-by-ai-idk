//! Minimal game-runtime facade that connects `GameState` frame advancement with
//! migrated building runtime slices.
//!
//! Upstream Java drives buildings from `Logic.update()` through `Groups.update()`
//! and finally `BuildingComp.update()/updateTile()`. The Rust port does not have
//! the full `Groups.build` owner yet, so this facade is the narrow runtime seam:
//! it owns game-wide sidecar stores and dispatches externally supplied building
//! slices from the real `GameState` frame source.

use std::collections::BTreeMap;

use crate::mindustry::{
    content::blocks::{
        BlockDef, DistributionBlockKind, EffectBlockKind, LiquidBlockKind, PayloadBlockKind,
        PowerBlockKind, SandboxBlockKind, StorageBlockKind,
    },
    core::content_loader::ContentLoader,
    core::game_state::GameState,
    ctype::ContentId,
    entities::{
        bullet::BulletType,
        comp::{BuildingComp, BulletComp, UnitComp},
    },
    io::LegacyShortChunkMap,
    vars::TILE_SIZE,
    world::blocks::defense::{
        effect_block_frame_input_from_game_update, effect_block_update_building_slice_with_stores,
        read_base_shield_state, read_force_projector_state, read_mend_projector_state,
        read_overdrive_projector_state, read_radar_state, EffectBlockFrameBatchReport,
        EffectBlockFrameBatchResources, EffectBlockRuntimeState, EffectBlockRuntimeStateStore,
        EffectBlockTimerStateStore, EffectProjectorRuntimeState,
    },
    world::blocks::distribution::{
        read_buffered_bridge_state, read_conveyor_state, read_directional_unloader_state,
        read_duct_junction_state, read_duct_router_state, read_duct_state, read_item_bridge_state,
        read_mass_driver_state, read_stack_conveyor_state, BufferedItemBridgeState, ConveyorState,
        DirectionalUnloaderState, DuctJunctionState, DuctRouterState, DuctState, ItemBridgeState,
        MassDriverState, StackConveyorState,
    },
    world::blocks::liquid::{read_liquid_bridge_state, LiquidBridgeState},
    world::blocks::payloads::{
        read_block_producer_progress, read_constructor_recipe, read_deconstructor_extra,
        read_empty_payload_block_build_common, read_empty_payload_conveyor_extra,
        read_empty_payload_ref, read_payload_loader_extra, read_payload_mass_driver_extra,
        read_payload_router_extra, read_payload_source_extra, BlockProducerState,
        PayloadBlockBuildState, PayloadConveyorState, PayloadDeconstructorState,
        PayloadLoaderState, PayloadMassDriverState, PayloadSortKey, PayloadSourceState,
    },
    world::blocks::power::{
        read_heater_generator_state, read_impact_reactor_state, read_light_block_state,
        read_nuclear_reactor_state, read_power_generator_state, read_variable_reactor_state,
        HeaterGeneratorState, ImpactReactorState, LightBlockState, NuclearReactorState,
        PowerGeneratorState, VariableReactorState,
    },
    world::blocks::storage::{read_core_state, CoreBuildState},
    world::blocks::units::{
        read_repair_turret_state, read_unit_cargo_loader_state, read_unit_cargo_unload_state,
        read_unit_factory_state, RepairTurretState, UnitCargoLoaderState,
        UnitCargoUnloadPointState, UnitFactoryState,
    },
    world::{footprint_tiles, get_edges, Tile},
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameRuntimeMapLoadReport {
    pub tiles: usize,
    pub building_records: usize,
    pub buildings_added: usize,
    pub block_states_added: usize,
    pub missing_block_defs: usize,
    pub skipped_non_building_blocks: usize,
    pub building_parse_errors: usize,
    pub block_state_parse_errors: usize,
    pub block_state_bytes_ignored: usize,
    pub disabled_buildings: usize,
    pub proximity_links: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameRuntimeBlockStateReadError {
    Parse,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimePayloadBlockState {
    MassDriver {
        common: PayloadBlockBuildState,
        driver: PayloadMassDriverState,
    },
    Loader {
        common: PayloadBlockBuildState,
        loader: PayloadLoaderState,
    },
    Source {
        common: PayloadBlockBuildState,
        source: PayloadSourceState,
    },
    Conveyor(PayloadConveyorState),
    Router {
        conveyor: PayloadConveyorState,
        sorted: Option<PayloadSortKey>,
        rec_dir: i32,
    },
    Deconstructor {
        common: PayloadBlockBuildState,
        deconstructor: PayloadDeconstructorState,
    },
    Constructor {
        common: PayloadBlockBuildState,
        producer: BlockProducerState,
        recipe: Option<i16>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimePowerBlockState {
    Generator(PowerGeneratorState),
    NuclearReactor(NuclearReactorState),
    ImpactReactor(ImpactReactorState),
    VariableReactor(VariableReactorState),
    HeaterGenerator(HeaterGeneratorState),
    Light(LightBlockState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimeDistributionBlockState {
    Conveyor(ConveyorState),
    StackConveyor(StackConveyorState),
    ItemBridge(ItemBridgeState),
    BufferedItemBridge(BufferedItemBridgeState),
    MassDriver(MassDriverState),
    DirectionalUnloader(DirectionalUnloaderState),
    Duct(DuctState),
    DuctRouter(DuctRouterState),
    DuctJunction(DuctJunctionState),
    UnitCargoLoader(UnitCargoLoaderState),
    UnitCargoUnload(UnitCargoUnloadPointState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimeStorageBlockState {
    Core(CoreBuildState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimeLiquidBlockState {
    Bridge(LiquidBridgeState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimeUnitBlockState {
    Factory(UnitFactoryState),
    RepairTower(RepairTurretState),
}

#[derive(Debug, Clone, PartialEq)]
enum GameRuntimeLoadedBlockState {
    Effect(EffectBlockRuntimeState),
    Payload(GameRuntimePayloadBlockState),
    Power(GameRuntimePowerBlockState),
    Distribution(GameRuntimeDistributionBlockState),
    Storage(GameRuntimeStorageBlockState),
    Liquid(GameRuntimeLiquidBlockState),
    Unit(GameRuntimeUnitBlockState),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameRuntime {
    pub state: GameState,
    pub buildings: Vec<BuildingComp>,
    pub effect_runtime_store: EffectBlockRuntimeStateStore,
    pub effect_timer_store: EffectBlockTimerStateStore,
    pub payload_runtime_states: BTreeMap<i32, GameRuntimePayloadBlockState>,
    pub power_runtime_states: BTreeMap<i32, GameRuntimePowerBlockState>,
    pub distribution_runtime_states: BTreeMap<i32, GameRuntimeDistributionBlockState>,
    pub storage_runtime_states: BTreeMap<i32, GameRuntimeStorageBlockState>,
    pub liquid_runtime_states: BTreeMap<i32, GameRuntimeLiquidBlockState>,
    pub unit_runtime_states: BTreeMap<i32, GameRuntimeUnitBlockState>,
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
            payload_runtime_states: BTreeMap::new(),
            power_runtime_states: BTreeMap::new(),
            distribution_runtime_states: BTreeMap::new(),
            storage_runtime_states: BTreeMap::new(),
            liquid_runtime_states: BTreeMap::new(),
            unit_runtime_states: BTreeMap::new(),
        }
    }

    pub fn buildings(&self) -> &[BuildingComp] {
        &self.buildings
    }

    pub fn buildings_mut(&mut self) -> &mut [BuildingComp] {
        &mut self.buildings
    }

    pub fn add_building(&mut self, building: BuildingComp) -> usize {
        for tile_pos in self.overlapping_building_positions(&building) {
            let _ = self.remove_building_by_tile_pos(tile_pos);
        }

        let index = self.buildings.len();
        self.buildings.push(building);
        self.sync_world_footprint_refs(index);
        self.refresh_owned_building_proximity();
        index
    }

    pub fn remove_building_by_tile_pos(&mut self, tile_pos: i32) -> Option<BuildingComp> {
        let index = self
            .buildings
            .iter()
            .position(|existing| existing.tile_pos == tile_pos)?;
        self.remove_building_at_index(index)
    }

    pub fn remove_building_at_index(&mut self, index: usize) -> Option<BuildingComp> {
        if index >= self.buildings.len() {
            return None;
        }

        let removed = self.buildings.remove(index);
        self.clear_world_refs_for_building(&removed);
        self.effect_runtime_store.remove(removed.tile_pos);
        self.effect_timer_store.remove(removed.tile_pos);
        self.payload_runtime_states.remove(&removed.tile_pos);
        self.power_runtime_states.remove(&removed.tile_pos);
        self.distribution_runtime_states.remove(&removed.tile_pos);
        self.storage_runtime_states.remove(&removed.tile_pos);
        self.liquid_runtime_states.remove(&removed.tile_pos);
        self.unit_runtime_states.remove(&removed.tile_pos);
        self.refresh_owned_building_proximity();
        Some(removed)
    }

    fn overlapping_building_positions(&self, building: &BuildingComp) -> Vec<i32> {
        let mut positions = vec![building.tile_pos];
        for (x, y) in footprint_tiles(building.tile_x(), building.tile_y(), building.block.size) {
            let Some(existing) = self.state.world.tile(x, y).and_then(|tile| tile.build) else {
                continue;
            };
            if !positions.contains(&existing.tile_pos) {
                positions.push(existing.tile_pos);
            }
        }
        positions
    }

    pub fn refresh_owned_building_proximity(&mut self) -> usize {
        let mut proximities = vec![Vec::new(); self.buildings.len()];

        for (index, building) in self.buildings.iter().enumerate() {
            let this_ref = building.pos_ref();
            let tile_x = building.tile_x();
            let tile_y = building.tile_y();
            let team = building.team;

            for point in get_edges(building.block.size.max(1)) {
                let Some(other_ref) = self.state.world.build(tile_x + point.x, tile_y + point.y)
                else {
                    continue;
                };
                if other_ref.tile_pos == building.tile_pos {
                    continue;
                }
                let Some(other_index) = self
                    .buildings
                    .iter()
                    .position(|other| other.tile_pos == other_ref.tile_pos)
                else {
                    continue;
                };
                if self.buildings[other_index].team != team {
                    continue;
                }

                let other_current_ref = self.buildings[other_index].pos_ref();
                if !proximities[index].contains(&other_current_ref) {
                    proximities[index].push(other_current_ref);
                }
                if !proximities[other_index].contains(&this_ref) {
                    proximities[other_index].push(this_ref);
                }
            }
        }

        let mut total = 0;
        for (building, proximity) in self.buildings.iter_mut().zip(proximities) {
            total += proximity.len();
            building.proximity = proximity;
        }
        total
    }

    pub fn clear_buildings(&mut self) {
        self.buildings.clear();
        self.state.world.clear_buildings();
        self.reset_effect_block_sidecars();
    }

    pub fn load_network_map_with_buildings(
        &mut self,
        content: &ContentLoader,
        map: &LegacyShortChunkMap,
    ) -> GameRuntimeMapLoadReport {
        self.buildings.clear();
        self.reset_effect_block_sidecars();
        self.state.world.load_network_map(map);

        let mut report = GameRuntimeMapLoadReport {
            tiles: map.tile_count(),
            ..GameRuntimeMapLoadReport::default()
        };

        let width = map.width as usize;
        if width == 0 {
            self.state.world.clear_load_events();
            return report;
        }
        for record in &map.blocks {
            if !record.has_entity || !record.is_center {
                continue;
            }
            report.building_records += 1;

            let Some(block) = content.block(record.block_id) else {
                report.missing_block_defs += 1;
                continue;
            };
            if !block.base().has_building() {
                report.skipped_non_building_blocks += 1;
                continue;
            }
            let Some(bytes) = &record.building else {
                report.building_parse_errors += 1;
                continue;
            };
            let Some((&revision, building_payload)) = bytes.split_first() else {
                report.building_parse_errors += 1;
                continue;
            };

            let x = (record.index % width) as i32;
            let y = (record.index / width) as i32;
            let mut building = BuildingComp::new(
                crate::mindustry::world::point2_pack(x, y),
                block.base().clone(),
                crate::mindustry::io::TeamId(0),
            );
            let mut building_bytes = building_payload;
            if building.read_base(&mut building_bytes).is_err() {
                report.building_parse_errors += 1;
                continue;
            }

            let block_state = match self.read_runtime_state_from_building_payload(
                block,
                &building,
                revision,
                &mut building_bytes,
            ) {
                Ok(state) => state,
                Err(GameRuntimeBlockStateReadError::Parse) => {
                    report.block_state_parse_errors += 1;
                    None
                }
                Err(GameRuntimeBlockStateReadError::Unsupported) => {
                    report.block_state_bytes_ignored += 1;
                    None
                }
            };

            let added_index = self.add_building(building);
            if let Some(block_state) = block_state {
                let tile_pos = self.buildings[added_index].tile_pos;
                match block_state {
                    GameRuntimeLoadedBlockState::Effect(block_state) => {
                        self.effect_runtime_store.ensure_for_building(
                            content,
                            &self.buildings[added_index],
                            0.0,
                        );
                        if let Some(slot) = self.effect_runtime_store.get_mut(tile_pos) {
                            *slot = block_state;
                            report.block_states_added += 1;
                        }
                    }
                    GameRuntimeLoadedBlockState::Payload(block_state) => {
                        self.payload_runtime_states.insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                    GameRuntimeLoadedBlockState::Power(block_state) => {
                        self.power_runtime_states.insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                    GameRuntimeLoadedBlockState::Distribution(block_state) => {
                        self.distribution_runtime_states
                            .insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                    GameRuntimeLoadedBlockState::Storage(block_state) => {
                        self.storage_runtime_states.insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                    GameRuntimeLoadedBlockState::Liquid(block_state) => {
                        self.liquid_runtime_states.insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                    GameRuntimeLoadedBlockState::Unit(block_state) => {
                        self.unit_runtime_states.insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                }
            }
            report.buildings_added += 1;
        }

        report.disabled_buildings = self.refresh_owned_building_update_permissions(content);
        report.proximity_links = self.refresh_owned_building_proximity();
        self.state.world.clear_load_events();
        report
    }

    fn read_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        building: &BuildingComp,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeLoadedBlockState>, GameRuntimeBlockStateReadError> {
        match self.read_effect_runtime_state_from_building_payload(
            block,
            revision,
            building_payload,
        ) {
            Ok(Some(state)) => Ok(Some(GameRuntimeLoadedBlockState::Effect(state))),
            Ok(None) => Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                Err(GameRuntimeBlockStateReadError::Parse)
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => self
                .read_payload_runtime_state_from_building_payload(block, revision, building_payload)
                .and_then(|state| {
                    if state.is_some() {
                        Ok(state.map(GameRuntimeLoadedBlockState::Payload))
                    } else {
                        Ok(None)
                    }
                })
                .or_else(|err| match err {
                    GameRuntimeBlockStateReadError::Unsupported => self
                        .read_power_runtime_state_from_building_payload(
                            block,
                            revision,
                            building_payload,
                        )
                        .and_then(|state| {
                            if state.is_some() {
                                Ok(state.map(GameRuntimeLoadedBlockState::Power))
                            } else {
                                Ok(None)
                            }
                        })
                        .or_else(|err| match err {
                            GameRuntimeBlockStateReadError::Unsupported => self
                                .read_distribution_runtime_state_from_building_payload(
                                    block,
                                    building,
                                    revision,
                                    building_payload,
                                )
                                .and_then(|state| {
                                    if state.is_some() {
                                        Ok(state.map(GameRuntimeLoadedBlockState::Distribution))
                                    } else {
                                        Ok(None)
                                    }
                                })
                                .or_else(|err| match err {
                                    GameRuntimeBlockStateReadError::Unsupported => self
                                        .read_storage_runtime_state_from_building_payload(
                                            block,
                                            revision,
                                            building_payload,
                                        )
                                        .and_then(|state| {
                                            if state.is_some() {
                                                Ok(state.map(GameRuntimeLoadedBlockState::Storage))
                                            } else {
                                                Ok(None)
                                            }
                                        })
                                        .or_else(|err| match err {
                                            GameRuntimeBlockStateReadError::Unsupported => self
                                                .read_liquid_runtime_state_from_building_payload(
                                                    block,
                                                    revision,
                                                    building_payload,
                                                )
                                                .and_then(|state| {
                                                    if state.is_some() {
                                                        Ok(state.map(
                                                            GameRuntimeLoadedBlockState::Liquid,
                                                        ))
                                                    } else {
                                                        Ok(None)
                                                    }
                                                })
                                                .or_else(|err| match err {
                                                    GameRuntimeBlockStateReadError::Unsupported => {
                                                        self.read_unit_runtime_state_from_building_payload(
                                                            block,
                                                            revision,
                                                            building_payload,
                                                        )
                                                        .map(|state| {
                                                            state.map(GameRuntimeLoadedBlockState::Unit)
                                                        })
                                                    }
                                                    GameRuntimeBlockStateReadError::Parse => {
                                                        Err(GameRuntimeBlockStateReadError::Parse)
                                                    }
                                                }),
                                            GameRuntimeBlockStateReadError::Parse => {
                                                Err(GameRuntimeBlockStateReadError::Parse)
                                            }
                                        }),
                                    GameRuntimeBlockStateReadError::Parse => {
                                        Err(GameRuntimeBlockStateReadError::Parse)
                                    }
                                }),
                            GameRuntimeBlockStateReadError::Parse => {
                                Err(GameRuntimeBlockStateReadError::Parse)
                            }
                        }),
                    GameRuntimeBlockStateReadError::Parse => {
                        Err(GameRuntimeBlockStateReadError::Parse)
                    }
                }),
        }
    }

    fn read_effect_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<EffectBlockRuntimeState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::Effect(effect) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };

        match effect.kind {
            EffectBlockKind::MendProjector => read_mend_projector_state(building_payload)
                .map(|state| {
                    Some(EffectBlockRuntimeState::Projector(
                        EffectProjectorRuntimeState::Mend(state),
                    ))
                })
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            EffectBlockKind::OverdriveProjector => read_overdrive_projector_state(building_payload)
                .map(|state| {
                    Some(EffectBlockRuntimeState::Projector(
                        EffectProjectorRuntimeState::Overdrive(state),
                    ))
                })
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            EffectBlockKind::ForceProjector => read_force_projector_state(building_payload)
                .map(|state| Some(EffectBlockRuntimeState::ForceProjector(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            EffectBlockKind::Radar => read_radar_state(building_payload)
                .map(|state| Some(EffectBlockRuntimeState::Radar(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            EffectBlockKind::BaseShield => read_base_shield_state(building_payload, revision)
                .map(|state| Some(EffectBlockRuntimeState::BaseShield(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            _ => Err(GameRuntimeBlockStateReadError::Unsupported),
        }
    }

    fn read_payload_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimePayloadBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        match block {
            BlockDef::Payload(payload) => {
                let (item_rotation, item) = read_empty_payload_conveyor_extra(building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let conveyor = PayloadConveyorState {
                    item,
                    item_rotation,
                    ..PayloadConveyorState::default()
                };
                match payload.kind {
                    PayloadBlockKind::PayloadConveyor => {
                        Ok(Some(GameRuntimePayloadBlockState::Conveyor(conveyor)))
                    }
                    PayloadBlockKind::PayloadRouter => {
                        let (sorted, rec_dir) =
                            read_payload_router_extra(building_payload, revision)
                                .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                        Ok(Some(GameRuntimePayloadBlockState::Router {
                            conveyor,
                            sorted,
                            rec_dir,
                        }))
                    }
                }
            }
            BlockDef::PayloadMassDriver(_) => {
                let common = read_empty_payload_block_build_common(building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let driver = read_payload_mass_driver_extra(building_payload, revision)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                Ok(Some(GameRuntimePayloadBlockState::MassDriver {
                    common,
                    driver,
                }))
            }
            BlockDef::PayloadLoader(_) => {
                let common = read_empty_payload_block_build_common(building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let exporting = read_payload_loader_extra(building_payload, revision)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let loader = PayloadLoaderState {
                    has_payload: common.payload.is_some(),
                    exporting,
                    ..PayloadLoaderState::default()
                };
                Ok(Some(GameRuntimePayloadBlockState::Loader {
                    common,
                    loader,
                }))
            }
            BlockDef::PayloadDeconstructor(_) => {
                let common = read_empty_payload_block_build_common(building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let (progress, accum) = read_deconstructor_extra(building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let deconstructing = read_empty_payload_ref(building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let deconstructor = PayloadDeconstructorState {
                    progress,
                    accum,
                    has_payload: common.payload.is_some(),
                    has_deconstructing: deconstructing.is_some(),
                };
                Ok(Some(GameRuntimePayloadBlockState::Deconstructor {
                    common,
                    deconstructor,
                }))
            }
            BlockDef::PayloadConstructor(_) => {
                let common = read_empty_payload_block_build_common(building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let progress = read_block_producer_progress(building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let recipe = read_constructor_recipe(building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let producer = BlockProducerState {
                    progress,
                    has_payload: common.payload.is_some(),
                    ..BlockProducerState::default()
                };
                Ok(Some(GameRuntimePayloadBlockState::Constructor {
                    common,
                    producer,
                    recipe,
                }))
            }
            BlockDef::Sandbox(sandbox) if sandbox.kind == SandboxBlockKind::PayloadSource => {
                let common = read_empty_payload_block_build_common(building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let (unit, config_block, command_pos) =
                    read_payload_source_extra(building_payload, revision)
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let source = PayloadSourceState {
                    unit,
                    config_block,
                    command_pos,
                    has_payload: common.payload.is_some(),
                    ..PayloadSourceState::default()
                };
                Ok(Some(GameRuntimePayloadBlockState::Source {
                    common,
                    source,
                }))
            }
            _ => Err(GameRuntimeBlockStateReadError::Unsupported),
        }
    }

    fn read_power_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimePowerBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let revision = revision as i32;

        match block {
            BlockDef::Power(power) => match power.kind {
                PowerBlockKind::ConsumeGenerator
                | PowerBlockKind::ThermalGenerator
                | PowerBlockKind::SolarGenerator => {
                    read_power_generator_state(building_payload, revision)
                        .map(|state| Some(GameRuntimePowerBlockState::Generator(state)))
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)
                }
                PowerBlockKind::NuclearReactor => {
                    read_nuclear_reactor_state(building_payload, revision)
                        .map(|state| Some(GameRuntimePowerBlockState::NuclearReactor(state)))
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)
                }
                PowerBlockKind::ImpactReactor => {
                    read_impact_reactor_state(building_payload, revision)
                        .map(|state| Some(GameRuntimePowerBlockState::ImpactReactor(state)))
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)
                }
                PowerBlockKind::VariableReactor => {
                    read_variable_reactor_state(building_payload, revision)
                        .map(|state| Some(GameRuntimePowerBlockState::VariableReactor(state)))
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)
                }
                PowerBlockKind::HeaterGenerator => {
                    read_heater_generator_state(building_payload, revision)
                        .map(|state| Some(GameRuntimePowerBlockState::HeaterGenerator(state)))
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)
                }
                _ => Err(GameRuntimeBlockStateReadError::Unsupported),
            },
            BlockDef::Light(_) => read_light_block_state(building_payload)
                .map(|state| Some(GameRuntimePowerBlockState::Light(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            _ => Err(GameRuntimeBlockStateReadError::Unsupported),
        }
    }

    fn read_distribution_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        building: &BuildingComp,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeDistributionBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::Distribution(distribution) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };
        let current_item = building
            .items
            .as_ref()
            .and_then(|items| items.each().next().map(|(item, _)| item));

        match distribution.kind {
            DistributionBlockKind::Conveyor | DistributionBlockKind::ArmoredConveyor => {
                read_conveyor_state(building_payload, revision)
                    .map(|state| Some(GameRuntimeDistributionBlockState::Conveyor(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            DistributionBlockKind::StackConveyor => {
                read_stack_conveyor_state(building_payload, current_item)
                    .map(|state| Some(GameRuntimeDistributionBlockState::StackConveyor(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            DistributionBlockKind::ItemBridge | DistributionBlockKind::DuctBridge => {
                read_item_bridge_state(building_payload, revision)
                    .map(|state| Some(GameRuntimeDistributionBlockState::ItemBridge(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            DistributionBlockKind::BufferedItemBridge => {
                read_buffered_bridge_state(building_payload, revision)
                    .map(|state| Some(GameRuntimeDistributionBlockState::BufferedItemBridge(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            DistributionBlockKind::MassDriver => read_mass_driver_state(building_payload)
                .map(|state| Some(GameRuntimeDistributionBlockState::MassDriver(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            DistributionBlockKind::DirectionalUnloader => {
                read_directional_unloader_state(building_payload)
                    .map(|state| {
                        Some(GameRuntimeDistributionBlockState::DirectionalUnloader(
                            state,
                        ))
                    })
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            DistributionBlockKind::Duct => {
                read_duct_state(building_payload, revision, current_item)
                    .map(|state| Some(GameRuntimeDistributionBlockState::Duct(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            DistributionBlockKind::DuctRouter | DistributionBlockKind::OverflowDuct => {
                read_duct_router_state(building_payload, revision, current_item)
                    .map(|state| Some(GameRuntimeDistributionBlockState::DuctRouter(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            DistributionBlockKind::Junction => read_duct_junction_state(building_payload)
                .map(|state| Some(GameRuntimeDistributionBlockState::DuctJunction(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            DistributionBlockKind::UnitCargoLoader => {
                read_unit_cargo_loader_state(building_payload)
                    .map(|state| Some(GameRuntimeDistributionBlockState::UnitCargoLoader(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            DistributionBlockKind::UnitCargoUnloadPoint => {
                read_unit_cargo_unload_state(building_payload)
                    .map(|state| Some(GameRuntimeDistributionBlockState::UnitCargoUnload(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            _ => Err(GameRuntimeBlockStateReadError::Unsupported),
        }
    }

    fn read_storage_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeStorageBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::Storage(storage) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };

        match storage.kind {
            StorageBlockKind::Core => read_core_state(building_payload, revision as i32)
                .map(|state| Some(GameRuntimeStorageBlockState::Core(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            StorageBlockKind::Storage => Err(GameRuntimeBlockStateReadError::Unsupported),
        }
    }

    fn read_liquid_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeLiquidBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::Liquid(liquid) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };

        match liquid.kind {
            LiquidBlockKind::LiquidBridge | LiquidBlockKind::DirectionLiquidBridge => {
                read_liquid_bridge_state(building_payload, revision)
                    .map(|state| Some(GameRuntimeLiquidBlockState::Bridge(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            _ => Err(GameRuntimeBlockStateReadError::Unsupported),
        }
    }

    fn read_unit_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeUnitBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        match block {
            BlockDef::UnitFactory(_) => read_unit_factory_state(building_payload, revision as i32)
                .map(|state| Some(GameRuntimeUnitBlockState::Factory(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            BlockDef::UnitRepairTower(_) => {
                read_repair_turret_state(building_payload, revision as i32)
                    .map(|state| Some(GameRuntimeUnitBlockState::RepairTower(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            _ => Err(GameRuntimeBlockStateReadError::Unsupported),
        }
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
        self.payload_runtime_states.clear();
        self.power_runtime_states.clear();
        self.distribution_runtime_states.clear();
        self.storage_runtime_states.clear();
        self.liquid_runtime_states.clear();
        self.unit_runtime_states.clear();
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
        io::{
            LegacyMapBlockRecord, LegacyMapFloorRecord, LegacyShortChunkMap, TeamId, Vec2 as IoVec2,
        },
        world::{
            blocks::defense::{
                write_base_shield_state, write_force_projector_state, write_radar_state,
                BaseShieldState, EffectBlockRuntimeState, ForceProjectorState, RadarState,
            },
            blocks::distribution::{
                write_conveyor_state, write_directional_unloader_state, write_duct_router_state,
                write_item_bridge_state, write_mass_driver_state, ConveyorItemState, ConveyorState,
                DirectionalUnloaderState, DuctRouterState, ItemBridgeState, MassDriverState,
                MassDriverStateKind,
            },
            blocks::liquid::{write_liquid_bridge_state, LiquidBridgeState},
            blocks::payloads::{
                write_block_producer_progress, write_constructor_recipe, write_deconstructor_extra,
                write_payload_block_build_common, write_payload_conveyor_extra,
                write_payload_loader_extra, write_payload_mass_driver_extra, write_payload_ref,
                write_payload_router_extra, write_payload_source_extra, BlockProducerState,
                PayloadBlockBuildState, PayloadConveyorState, PayloadDeconstructorState,
                PayloadDriverState, PayloadLoaderState, PayloadMassDriverState, PayloadSortKey,
                PayloadSourceState, Vec2,
            },
            blocks::power::{
                write_heater_generator_state, write_impact_reactor_state, write_light_block_state,
                write_nuclear_reactor_state, write_power_generator_state,
                write_variable_reactor_state, HeaterGeneratorState, ImpactReactorState,
                LightBlockState, NuclearReactorState, PowerGeneratorState, VariableReactorState,
            },
            blocks::storage::{write_core_state, CoreBuildState},
            blocks::units::{
                write_repair_turret_state, write_unit_cargo_loader_state,
                write_unit_cargo_unload_state, write_unit_factory_state, RepairTurretState,
                UnitCargoLoaderState, UnitCargoUnloadPointState, UnitFactoryState,
            },
            footprint_tiles, point2_pack, Block, Tile,
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

    fn single_building_network_map(
        width: u16,
        height: u16,
        index: usize,
        block_id: i16,
        building_bytes: Vec<u8>,
    ) -> LegacyShortChunkMap {
        let tile_count = width as usize * height as usize;
        assert!(index < tile_count);
        let mut blocks = Vec::new();

        if index > 0 {
            blocks.push(LegacyMapBlockRecord {
                index: 0,
                block_id: Tile::AIR,
                packed_flags: 0,
                has_entity: false,
                has_old_data: false,
                has_new_data: false,
                is_center: true,
                new_data: None,
                old_data: None,
                building: None,
                consecutives: (index - 1) as u8,
            });
        }

        blocks.push(LegacyMapBlockRecord {
            index,
            block_id,
            packed_flags: 1,
            has_entity: true,
            has_old_data: false,
            has_new_data: false,
            is_center: true,
            new_data: None,
            old_data: None,
            building: Some(building_bytes),
            consecutives: 0,
        });

        if index + 1 < tile_count {
            blocks.push(LegacyMapBlockRecord {
                index: index + 1,
                block_id: Tile::AIR,
                packed_flags: 0,
                has_entity: false,
                has_old_data: false,
                has_new_data: false,
                is_center: true,
                new_data: None,
                old_data: None,
                building: None,
                consecutives: (tile_count - index - 2) as u8,
            });
        }

        LegacyShortChunkMap {
            width,
            height,
            floors: vec![LegacyMapFloorRecord {
                index: 0,
                floor_id: 0,
                ore_id: 0,
                consecutives: (tile_count - 1) as u8,
            }],
            blocks,
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
    fn game_runtime_add_building_removes_overlapping_multiblock_and_sidecars() {
        let content = ContentLoader::create_base_content().unwrap();
        let mend_def = content.block_by_name("mend-projector").unwrap();
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(32, 32);

        let center = point2_pack(10, 10);
        runtime.add_building(BuildingComp::new(
            center,
            mend_def.base().clone(),
            TeamId(1),
        ));
        let mend_snapshot = runtime.buildings()[0].clone();
        runtime
            .effect_runtime_store
            .ensure_for_building(&content, &mend_snapshot, 0.0);
        runtime
            .effect_timer_store
            .ensure_for_building(&content, &mend_snapshot);
        assert!(runtime.effect_runtime_store.get(center).is_some());
        assert!(runtime.effect_timer_store.get(center).is_some());

        let mut large_block = Block::new(30_010, "test-large");
        large_block.size = 3;
        runtime.add_building(BuildingComp::new(center, large_block.clone(), TeamId(1)));
        assert_eq!(runtime.buildings().len(), 1);
        assert!(runtime.effect_runtime_store.get(center).is_none());
        assert!(runtime.effect_timer_store.get(center).is_none());
        assert_eq!(
            runtime
                .state
                .world
                .tile(9, 9)
                .unwrap()
                .build
                .unwrap()
                .tile_pos,
            center
        );

        let mut small_block = Block::new(30_011, "test-small");
        small_block.size = 1;
        let overlap = point2_pack(9, 9);
        runtime.add_building(BuildingComp::new(overlap, small_block.clone(), TeamId(2)));

        assert_eq!(runtime.buildings().len(), 1);
        assert_eq!(runtime.buildings()[0].tile_pos, overlap);
        assert_eq!(
            runtime.state.world.tile(9, 9).unwrap().block,
            small_block.id
        );
        assert_eq!(
            runtime
                .state
                .world
                .tile(9, 9)
                .unwrap()
                .build
                .unwrap()
                .tile_pos,
            overlap
        );
        let old_center = runtime.state.world.tile(10, 10).unwrap();
        assert_eq!(old_center.block, Tile::AIR);
        assert!(old_center.build.is_none());
    }

    #[test]
    fn game_runtime_refreshes_owned_building_proximity_like_java_edges() {
        let mut large_block = Block::new(30_020, "test-large");
        large_block.size = 3;
        let small_block = Block::new(30_021, "test-small");
        let enemy_block = Block::new(30_022, "test-enemy");
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(32, 32);

        let large_pos = point2_pack(10, 10);
        let same_team_pos = point2_pack(12, 10);
        let enemy_pos = point2_pack(10, 12);
        runtime.add_building(BuildingComp::new(large_pos, large_block.clone(), TeamId(1)));
        runtime.add_building(BuildingComp::new(
            same_team_pos,
            small_block.clone(),
            TeamId(1),
        ));
        runtime.add_building(BuildingComp::new(enemy_pos, enemy_block, TeamId(2)));

        let large = runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == large_pos)
            .unwrap();
        let same_team = runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == same_team_pos)
            .unwrap();
        let enemy = runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == enemy_pos)
            .unwrap();
        assert_eq!(large.proximity, vec![same_team.pos_ref()]);
        assert_eq!(same_team.proximity, vec![large.pos_ref()]);
        assert!(enemy.proximity.is_empty());

        runtime.remove_building_by_tile_pos(same_team_pos).unwrap();
        let large = runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == large_pos)
            .unwrap();
        assert!(large.proximity.is_empty());
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
    fn game_runtime_clear_buildings_resets_world_refs_and_sidecars() {
        let content = ContentLoader::create_base_content().unwrap();
        let mend_def = content.block_by_name("mend-projector").unwrap();
        let tile_pos = point2_pack(36, 9);
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

        runtime.clear_buildings();

        assert!(runtime.buildings().is_empty());
        assert!(runtime.state.world.build_pos(tile_pos).is_none());
        assert!(runtime.effect_runtime_store.is_empty());
        assert!(runtime.effect_timer_store.is_empty());
    }

    #[test]
    fn game_runtime_loads_network_map_center_buildings_into_owned_runtime() {
        let content = ContentLoader::create_base_content().unwrap();
        let mend_def = content.block_by_name("mend-projector").unwrap();
        let tile_pos = point2_pack(2, 1);
        let mut saved = BuildingComp::new(tile_pos, mend_def.base().clone(), TeamId(2));
        saved.set_rotation(3);
        saved.health = 42.0;
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();

        let map = LegacyShortChunkMap {
            width: 4,
            height: 4,
            floors: vec![LegacyMapFloorRecord {
                index: 0,
                floor_id: 0,
                ore_id: 0,
                consecutives: 15,
            }],
            blocks: vec![
                LegacyMapBlockRecord {
                    index: 0,
                    block_id: Tile::AIR,
                    packed_flags: 0,
                    has_entity: false,
                    has_old_data: false,
                    has_new_data: false,
                    is_center: true,
                    new_data: None,
                    old_data: None,
                    building: None,
                    consecutives: 5,
                },
                LegacyMapBlockRecord {
                    index: 6,
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
                    index: 7,
                    block_id: Tile::AIR,
                    packed_flags: 0,
                    has_entity: false,
                    has_old_data: false,
                    has_new_data: false,
                    is_center: true,
                    new_data: None,
                    old_data: None,
                    building: None,
                    consecutives: 8,
                },
            ],
        };

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(&content, &map);

        assert_eq!(report.tiles, 16);
        assert_eq!(report.building_records, 1);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.building_parse_errors, 0);
        assert!(runtime.state.world.load_events().is_empty());
        assert_eq!(runtime.state.world.width(), 4);
        assert_eq!(runtime.state.world.height(), 4);
        assert_eq!(
            runtime.state.world.build_pos(tile_pos).unwrap().tile_pos,
            tile_pos
        );
        assert_eq!(runtime.buildings().len(), 1);
        let building = &runtime.buildings()[0];
        assert_eq!(building.tile_pos, tile_pos);
        assert_eq!(building.team, TeamId(2));
        assert_eq!(building.rotation, 3);
        assert_eq!(building.health, 42.0);
        assert_eq!(building.block.id, mend_def.base().id);
    }

    #[test]
    fn game_runtime_loads_effect_block_specific_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let force_def = content.block_by_name("force-projector").unwrap();
        let tile_pos = point2_pack(2, 2);
        let mut saved = BuildingComp::new(tile_pos, force_def.base().clone(), TeamId(4));
        saved.set_rotation(1);
        let force_state = ForceProjectorState {
            broken: false,
            buildup: 12.5,
            radscl: 0.75,
            hit: 0.0,
            warmup: 0.25,
            phase_heat: 0.5,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_force_projector_state(&mut building_bytes, &force_state).unwrap();

        let map = LegacyShortChunkMap {
            width: 5,
            height: 5,
            floors: vec![LegacyMapFloorRecord {
                index: 0,
                floor_id: 0,
                ore_id: 0,
                consecutives: 24,
            }],
            blocks: vec![
                LegacyMapBlockRecord {
                    index: 0,
                    block_id: Tile::AIR,
                    packed_flags: 0,
                    has_entity: false,
                    has_old_data: false,
                    has_new_data: false,
                    is_center: true,
                    new_data: None,
                    old_data: None,
                    building: None,
                    consecutives: 11,
                },
                LegacyMapBlockRecord {
                    index: 12,
                    block_id: force_def.base().id,
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
                    index: 13,
                    block_id: Tile::AIR,
                    packed_flags: 0,
                    has_entity: false,
                    has_old_data: false,
                    has_new_data: false,
                    is_center: true,
                    new_data: None,
                    old_data: None,
                    building: None,
                    consecutives: 11,
                },
            ],
        };

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(&content, &map);

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.effect_runtime_store.get(tile_pos),
            Some(&EffectBlockRuntimeState::ForceProjector(force_state))
        );
    }

    #[test]
    fn game_runtime_loads_radar_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let radar_def = content.block_by_name("radar").unwrap();
        let tile_pos = point2_pack(1, 2);
        let mut saved = BuildingComp::new(tile_pos, radar_def.base().clone(), TeamId(3));
        saved.set_rotation(2);
        let radar_state = RadarState {
            progress: 0.625,
            ..RadarState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_radar_state(&mut building_bytes, &radar_state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(4, 4, 9, radar_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.effect_runtime_store.get(tile_pos),
            Some(&EffectBlockRuntimeState::Radar(radar_state))
        );
    }

    #[test]
    fn game_runtime_loads_base_shield_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let shield_def = content.block_by_name("shield-projector").unwrap();
        let tile_pos = point2_pack(2, 2);
        let saved = BuildingComp::new(tile_pos, shield_def.base().clone(), TeamId(5));
        let shield_state = BaseShieldState {
            broken: true,
            hit: 0.0,
            smooth_radius: 18.25,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_base_shield_state(&mut building_bytes, &shield_state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(5, 5, 12, shield_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.effect_runtime_store.get(tile_pos),
            Some(&EffectBlockRuntimeState::BaseShield(shield_state))
        );
    }

    #[test]
    fn game_runtime_loads_payload_mass_driver_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let driver_def = content.block_by_name("payload-mass-driver").unwrap();
        let tile_pos = point2_pack(2, 3);
        let mut saved = BuildingComp::new(tile_pos, driver_def.base().clone(), TeamId(6));
        saved.set_rotation(1);
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: 1.5, y: -2.25 },
            pay_rotation: 45.0,
            carried: false,
        };
        let driver = PayloadMassDriverState {
            link: point2_pack(4, 3),
            turret_rotation: 135.0,
            state: PayloadDriverState::Shooting,
            reload_counter: 0.5,
            charge: 12.0,
            loaded: true,
            charging: true,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
        write_payload_mass_driver_extra(&mut building_bytes, &driver).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 20, driver_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.payload_runtime_states.get(&tile_pos),
            Some(&GameRuntimePayloadBlockState::MassDriver { common, driver })
        );
    }

    #[test]
    fn game_runtime_loads_payload_loader_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let loader_def = content.block_by_name("payload-loader").unwrap();
        let tile_pos = point2_pack(3, 2);
        let mut saved = BuildingComp::new(tile_pos, loader_def.base().clone(), TeamId(6));
        saved.set_rotation(3);
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: -1.0, y: 2.0 },
            pay_rotation: 270.0,
            carried: false,
        };
        let loader = PayloadLoaderState {
            exporting: true,
            ..PayloadLoaderState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
        write_payload_loader_extra(&mut building_bytes, loader.exporting).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 15, loader_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.payload_runtime_states.get(&tile_pos),
            Some(&GameRuntimePayloadBlockState::Loader { common, loader })
        );
    }

    #[test]
    fn game_runtime_loads_payload_source_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let source_def = content.block_by_name("payload-source").unwrap();
        let tile_pos = point2_pack(4, 2);
        let saved = BuildingComp::new(tile_pos, source_def.base().clone(), TeamId(6));
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: 0.25, y: 0.5 },
            pay_rotation: 90.0,
            carried: false,
        };
        let source = PayloadSourceState {
            unit: Some(0),
            config_block: None,
            command_pos: Some(Vec2 { x: 8.0, y: 16.0 }),
            has_payload: false,
            ..PayloadSourceState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
        write_payload_source_extra(
            &mut building_bytes,
            source.unit,
            source.config_block,
            source.command_pos,
        )
        .unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 16, source_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.payload_runtime_states.get(&tile_pos),
            Some(&GameRuntimePayloadBlockState::Source { common, source })
        );
    }

    #[test]
    fn game_runtime_loads_payload_conveyor_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let conveyor_def = content.block_by_name("payload-conveyor").unwrap();
        let tile_pos = point2_pack(1, 4);
        let mut saved = BuildingComp::new(tile_pos, conveyor_def.base().clone(), TeamId(6));
        saved.set_rotation(1);
        let conveyor = PayloadConveyorState {
            item: None,
            item_rotation: 33.0,
            ..PayloadConveyorState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_conveyor_extra(&mut building_bytes, 12.0, conveyor.item_rotation, None)
            .unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 25, conveyor_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.payload_runtime_states.get(&tile_pos),
            Some(&GameRuntimePayloadBlockState::Conveyor(conveyor))
        );
    }

    #[test]
    fn game_runtime_loads_payload_router_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let router_def = content.block_by_name("payload-router").unwrap();
        let tile_pos = point2_pack(2, 4);
        let mut saved = BuildingComp::new(tile_pos, router_def.base().clone(), TeamId(6));
        saved.set_rotation(2);
        let conveyor = PayloadConveyorState {
            item: None,
            item_rotation: 180.0,
            ..PayloadConveyorState::default()
        };
        let sorted = Some(PayloadSortKey {
            content_type: 0,
            id: router_def.base().id,
        });
        let rec_dir = 3;
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_conveyor_extra(&mut building_bytes, 6.0, conveyor.item_rotation, None)
            .unwrap();
        write_payload_router_extra(&mut building_bytes, sorted, rec_dir).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 26, router_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.payload_runtime_states.get(&tile_pos),
            Some(&GameRuntimePayloadBlockState::Router {
                conveyor,
                sorted,
                rec_dir
            })
        );
    }

    #[test]
    fn game_runtime_loads_payload_deconstructor_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let deconstructor_def = content.block_by_name("small-deconstructor").unwrap();
        let tile_pos = point2_pack(3, 4);
        let saved = BuildingComp::new(tile_pos, deconstructor_def.base().clone(), TeamId(6));
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: -0.5, y: 0.75 },
            pay_rotation: 15.0,
            carried: false,
        };
        let deconstructor = PayloadDeconstructorState {
            progress: 0.4,
            accum: Some(vec![1.0, 2.5, 0.25]),
            has_payload: false,
            has_deconstructing: false,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
        write_deconstructor_extra(
            &mut building_bytes,
            deconstructor.progress,
            deconstructor.accum.as_deref(),
        )
        .unwrap();
        write_payload_ref(&mut building_bytes, None).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 27, deconstructor_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.payload_runtime_states.get(&tile_pos),
            Some(&GameRuntimePayloadBlockState::Deconstructor {
                common,
                deconstructor
            })
        );
    }

    #[test]
    fn game_runtime_loads_payload_constructor_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let constructor_def = content.block_by_name("constructor").unwrap();
        let recipe = content.block_by_name("router").map(|block| block.base().id);
        let tile_pos = point2_pack(4, 4);
        let saved = BuildingComp::new(tile_pos, constructor_def.base().clone(), TeamId(6));
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: 0.0, y: -1.0 },
            pay_rotation: 180.0,
            carried: false,
        };
        let producer = BlockProducerState {
            progress: 3.5,
            ..BlockProducerState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
        write_block_producer_progress(&mut building_bytes, producer.progress).unwrap();
        write_constructor_recipe(&mut building_bytes, recipe).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 28, constructor_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.payload_runtime_states.get(&tile_pos),
            Some(&GameRuntimePayloadBlockState::Constructor {
                common,
                producer,
                recipe
            })
        );
    }

    #[test]
    fn game_runtime_loads_power_generator_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let generator_def = content.block_by_name("thermal-generator").unwrap();
        let tile_pos = point2_pack(1, 5);
        let saved = BuildingComp::new(tile_pos, generator_def.base().clone(), TeamId(2));
        let state = PowerGeneratorState {
            production_efficiency: 0.75,
            generate_time: 4.0,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_power_generator_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 31, generator_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.power_runtime_states.get(&tile_pos),
            Some(&GameRuntimePowerBlockState::Generator(state))
        );
    }

    #[test]
    fn game_runtime_loads_nuclear_reactor_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let reactor_def = content.block_by_name("thorium-reactor").unwrap();
        let tile_pos = point2_pack(2, 5);
        let saved = BuildingComp::new(tile_pos, reactor_def.base().clone(), TeamId(2));
        let state = NuclearReactorState {
            generator: PowerGeneratorState {
                production_efficiency: 0.5,
                generate_time: 2.0,
            },
            heat: 0.8,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_nuclear_reactor_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 32, reactor_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.power_runtime_states.get(&tile_pos),
            Some(&GameRuntimePowerBlockState::NuclearReactor(state))
        );
    }

    #[test]
    fn game_runtime_loads_impact_reactor_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let reactor_def = content.block_by_name("impact-reactor").unwrap();
        let tile_pos = point2_pack(3, 5);
        let saved = BuildingComp::new(tile_pos, reactor_def.base().clone(), TeamId(2));
        let state = ImpactReactorState {
            generator: PowerGeneratorState {
                production_efficiency: 0.9,
                generate_time: 1.5,
            },
            warmup: 0.6,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_impact_reactor_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 33, reactor_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.power_runtime_states.get(&tile_pos),
            Some(&GameRuntimePowerBlockState::ImpactReactor(state))
        );
    }

    #[test]
    fn game_runtime_loads_variable_reactor_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let reactor_def = content.block_by_name("flux-reactor").unwrap();
        let tile_pos = point2_pack(4, 5);
        let saved = BuildingComp::new(tile_pos, reactor_def.base().clone(), TeamId(2));
        let state = VariableReactorState {
            generator: PowerGeneratorState {
                production_efficiency: 0.4,
                generate_time: 3.0,
            },
            heat: 7.5,
            instability: 0.25,
            warmup: 0.5,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_variable_reactor_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 34, reactor_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.power_runtime_states.get(&tile_pos),
            Some(&GameRuntimePowerBlockState::VariableReactor(state))
        );
    }

    #[test]
    fn game_runtime_loads_heater_generator_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let generator_def = content.block_by_name("neoplasia-reactor").unwrap();
        let tile_pos = point2_pack(5, 5);
        let saved = BuildingComp::new(tile_pos, generator_def.base().clone(), TeamId(2));
        let state = HeaterGeneratorState {
            generator: PowerGeneratorState {
                production_efficiency: 0.3,
                generate_time: 2.25,
            },
            heat: 12.0,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_heater_generator_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 35, generator_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.power_runtime_states.get(&tile_pos),
            Some(&GameRuntimePowerBlockState::HeaterGenerator(state))
        );
    }

    #[test]
    fn game_runtime_loads_light_block_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let light_def = content.block_by_name("illuminator").unwrap();
        let tile_pos = point2_pack(0, 5);
        let saved = BuildingComp::new(tile_pos, light_def.base().clone(), TeamId(2));
        let state = LightBlockState { color: 0x12_34_56 };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_light_block_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 30, light_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.power_runtime_states.get(&tile_pos),
            Some(&GameRuntimePowerBlockState::Light(state))
        );
    }

    #[test]
    fn game_runtime_loads_conveyor_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let conveyor_def = content.block_by_name("conveyor").unwrap();
        let tile_pos = point2_pack(1, 1);
        let saved = BuildingComp::new(tile_pos, conveyor_def.base().clone(), TeamId(1));
        let state = ConveyorState {
            items: vec![ConveyorItemState {
                item: 0,
                x: 0.0,
                y: 128.0 / 255.0,
            }],
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_conveyor_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 7, conveyor_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.distribution_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDistributionBlockState::Conveyor(state))
        );
    }

    #[test]
    fn game_runtime_loads_item_bridge_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let bridge_def = content.block_by_name("phase-conveyor").unwrap();
        let tile_pos = point2_pack(2, 1);
        let saved = BuildingComp::new(tile_pos, bridge_def.base().clone(), TeamId(1));
        let state = ItemBridgeState {
            link: point2_pack(4, 1),
            warmup: 0.6,
            incoming: vec![point2_pack(1, 1), point2_pack(3, 1)],
            was_moved: true,
            moved: true,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_item_bridge_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 8, bridge_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.distribution_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDistributionBlockState::ItemBridge(state))
        );
    }

    #[test]
    fn game_runtime_loads_mass_driver_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let driver_def = content.block_by_name("mass-driver").unwrap();
        let tile_pos = point2_pack(3, 1);
        let saved = BuildingComp::new(tile_pos, driver_def.base().clone(), TeamId(1));
        let state = MassDriverState {
            link: point2_pack(5, 1),
            rotation: 135.0,
            state: MassDriverStateKind::Shooting,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_mass_driver_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 9, driver_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.distribution_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDistributionBlockState::MassDriver(state))
        );
    }

    #[test]
    fn game_runtime_loads_duct_router_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let router_def = content.block_by_name("duct-router").unwrap();
        let tile_pos = point2_pack(4, 1);
        let saved = BuildingComp::new(tile_pos, router_def.base().clone(), TeamId(1));
        let state = DuctRouterState {
            sort_item: Some(0),
            current: None,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_duct_router_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 10, router_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.distribution_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDistributionBlockState::DuctRouter(state))
        );
    }

    #[test]
    fn game_runtime_loads_directional_unloader_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let unloader_def = content.block_by_name("duct-unloader").unwrap();
        let tile_pos = point2_pack(5, 1);
        let saved = BuildingComp::new(tile_pos, unloader_def.base().clone(), TeamId(1));
        let state = DirectionalUnloaderState {
            unload_item: Some(0),
            offset: 17,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_directional_unloader_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 11, unloader_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.distribution_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDistributionBlockState::DirectionalUnloader(
                state
            ))
        );
    }

    #[test]
    fn game_runtime_loads_core_storage_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let core_def = content.block_by_name("core-shard").unwrap();
        let tile_pos = point2_pack(2, 0);
        let saved = BuildingComp::new(tile_pos, core_def.base().clone(), TeamId(1));
        let state = CoreBuildState {
            command_pos: Some(IoVec2 { x: 64.0, y: 128.0 }),
            ..CoreBuildState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_core_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 2, core_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.storage_runtime_states.get(&tile_pos),
            Some(&GameRuntimeStorageBlockState::Core(state))
        );
    }

    #[test]
    fn game_runtime_loads_liquid_bridge_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let bridge_def = content.block_by_name("bridge-conduit").unwrap();
        let tile_pos = point2_pack(3, 0);
        let saved = BuildingComp::new(tile_pos, bridge_def.base().clone(), TeamId(1));
        let state = LiquidBridgeState {
            link: point2_pack(5, 0),
            warmup: 0.8,
            incoming: vec![point2_pack(2, 0)],
            was_moved: true,
            moved: true,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_liquid_bridge_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 3, bridge_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.liquid_runtime_states.get(&tile_pos),
            Some(&GameRuntimeLiquidBlockState::Bridge(state))
        );
    }

    #[test]
    fn game_runtime_loads_unit_factory_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let factory_def = content.block_by_name("ground-factory").unwrap();
        let tile_pos = point2_pack(4, 0);
        let saved = BuildingComp::new(tile_pos, factory_def.base().clone(), TeamId(1));
        let state = UnitFactoryState {
            base: crate::mindustry::world::blocks::units::UnitBlockState {
                progress: 25.0,
                ..Default::default()
            },
            current_plan: 1,
            command_pos: Some(IoVec2 { x: 12.0, y: 34.0 }),
            command_id: Some(2),
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(3);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_unit_factory_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 4, factory_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.unit_runtime_states.get(&tile_pos),
            Some(&GameRuntimeUnitBlockState::Factory(state))
        );
    }

    #[test]
    fn game_runtime_loads_unit_repair_tower_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let tower_def = content.block_by_name("unit-repair-tower").unwrap();
        let tile_pos = point2_pack(5, 0);
        let saved = BuildingComp::new(tile_pos, tower_def.base().clone(), TeamId(1));
        let state = RepairTurretState {
            rotation: 45.0,
            ..RepairTurretState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_repair_turret_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 5, tower_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.unit_runtime_states.get(&tile_pos),
            Some(&GameRuntimeUnitBlockState::RepairTower(state))
        );
    }

    #[test]
    fn game_runtime_loads_unit_cargo_loader_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let loader_def = content.block_by_name("unit-cargo-loader").unwrap();
        let tile_pos = point2_pack(0, 1);
        let saved = BuildingComp::new(tile_pos, loader_def.base().clone(), TeamId(1));
        let state = UnitCargoLoaderState {
            read_unit_id: 77,
            ..UnitCargoLoaderState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_unit_cargo_loader_state(&mut building_bytes, Some(state.read_unit_id)).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 6, loader_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.distribution_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDistributionBlockState::UnitCargoLoader(state))
        );
    }

    #[test]
    fn game_runtime_loads_unit_cargo_unload_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let unload_def = content.block_by_name("unit-cargo-unload-point").unwrap();
        let tile_pos = point2_pack(1, 0);
        let saved = BuildingComp::new(tile_pos, unload_def.base().clone(), TeamId(1));
        let state = UnitCargoUnloadPointState {
            item_id: Some(0),
            stale_timer: 0.0,
            stale: true,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_unit_cargo_unload_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 1, unload_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.distribution_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDistributionBlockState::UnitCargoUnload(state))
        );
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
