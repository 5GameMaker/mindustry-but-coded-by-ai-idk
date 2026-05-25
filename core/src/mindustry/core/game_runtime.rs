//! Minimal game-runtime facade that connects `GameState` frame advancement with
//! migrated building runtime slices.
//!
//! Upstream Java drives buildings from `Logic.update()` through `Groups.update()`
//! and finally `BuildingComp.update()/updateTile()`. The Rust port does not have
//! the full `Groups.build` owner yet, so this facade is the narrow runtime seam:
//! it owns game-wide sidecar stores and dispatches externally supplied building
//! slices from the real `GameState` frame source.

use std::{collections::BTreeMap, io};

use crate::mindustry::{
    content::blocks::{
        BlockDef, CampaignBlockKind, CraftingBlockKind, DefenseWallKind, DistributionBlockKind,
        EffectBlockKind, LegacyBlockKind, LiquidBlockKind, LogicBlockKind, PayloadBlockKind,
        PowerBlockKind, ProductionBlockKind, SandboxBlockKind, StorageBlockKind, TurretBlockKind,
    },
    core::content_loader::ContentLoader,
    core::game_state::GameState,
    ctype::ContentId,
    entities::{
        bullet::BulletType,
        comp::{BuildingComp, BulletComp, UnitComp},
    },
    io::{
        type_io, LegacyMapBlockRecord, LegacyMapFloorRecord, LegacyMapTileData, LegacyShortChunkMap,
    },
    r#type::PayloadSeq,
    vars::TILE_SIZE,
    world::blocks::campaign::{
        read_accelerator_state, read_landing_pad_state, read_launch_pad_state,
        write_accelerator_state, write_landing_pad_state, write_launch_pad_state, AcceleratorState,
        LandingPadState, LaunchPadState,
    },
    world::blocks::defense::turrets::{
        continuous_turret_read_child, continuous_turret_write_child, item_turret_read_ammo,
        item_turret_write_ammo, payload_ammo_turret_read_payloads,
        payload_ammo_turret_write_payloads, point_defense_read_child, point_defense_write_child,
        tractor_beam_read_child, tractor_beam_write_child, turret_read_child, turret_write_child,
        ContinuousTurretState, ItemAmmoEntry, PointDefenseState, TractorBeamState, TurretState,
    },
    world::blocks::defense::{
        build_turret_read_child_with_loader, build_turret_write_child_with_loader,
        effect_block_frame_input_from_game_update, effect_block_update_building_slice_with_stores,
        read_auto_door_state, read_base_shield_state, read_door_state, read_force_projector_state,
        read_mend_projector_state, read_overdrive_projector_state, read_radar_state,
        read_shield_wall_state, write_auto_door_state, write_base_shield_state, write_door_state,
        write_force_projector_state, write_mend_projector_state, write_overdrive_projector_state,
        write_radar_state, write_shield_wall_state, DoorState, EffectBlockFrameBatchReport,
        EffectBlockFrameBatchResources, EffectBlockRuntimeState, EffectBlockRuntimeStateStore,
        EffectBlockTimerStateStore, EffectProjectorRuntimeState, ShieldWallState,
    },
    world::blocks::distribution::{
        read_buffered_bridge_state, read_conveyor_state, read_directional_unloader_state,
        read_duct_junction_state, read_duct_router_state, read_duct_state, read_item_bridge_state,
        read_mass_driver_state, read_overflow_gate_legacy_payload, read_sorter_state,
        read_stack_conveyor_state, write_buffered_bridge_state, write_conveyor_state,
        write_directional_unloader_state, write_duct_junction_state, write_duct_router_state,
        write_duct_state, write_item_bridge_state, write_mass_driver_state, write_sorter_state,
        write_stack_conveyor_state, BufferedItemBridgeState, ConveyorState,
        DirectionalUnloaderState, DuctJunctionState, DuctRouterState, DuctState, ItemBridgeState,
        MassDriverState, SorterState, StackConveyorState,
    },
    world::blocks::heat::{read_heat_producer_state, write_heat_producer_state, HeatProducerState},
    world::blocks::legacy::{
        read_legacy_command_center_extra, read_legacy_mech_pad_extra,
        read_legacy_unit_factory_extra, write_legacy_command_center_extra,
        write_legacy_mech_pad_extra, write_legacy_unit_factory_extra, LegacyUnitFactoryExtra,
    },
    world::blocks::liquid::{
        read_liquid_bridge_state, write_liquid_bridge_state, LiquidBridgeState,
    },
    world::blocks::logic::{
        read_canvas_state, read_logic_display_state, read_logic_processor_state, read_memory_state,
        read_message_state, read_switch_enabled, write_canvas_state, write_logic_display_state,
        write_logic_processor_state, write_memory_state, write_message_state, write_switch_enabled,
        CanvasBlockState, LogicDisplayState, LogicProcessorState, MemoryBlockState,
        MessageBlockState,
    },
    world::blocks::payloads::{
        block_producer_update, read_block_producer_progress, read_constructor_recipe,
        read_deconstructor_extra, read_payload_loader_extra, read_payload_mass_driver_extra,
        read_payload_ref_to_end, read_payload_router_extra, read_payload_source_extra,
        read_terminal_payload_block_build_common, read_terminal_payload_conveyor_extra,
        write_block_producer_progress, write_constructor_recipe, write_deconstructor_extra,
        write_payload_block_build_common, write_payload_conveyor_extra, write_payload_loader_extra,
        write_payload_mass_driver_extra, write_payload_ref, write_payload_router_extra,
        write_payload_source_extra, BlockProducerState, PayloadBlockBuildState,
        PayloadConveyorState, PayloadDeconstructorState, PayloadLoaderState,
        PayloadMassDriverState, PayloadRef, PayloadSortKey, PayloadSourceState,
        Vec2 as PayloadVec2, PAYLOAD_BLOCK_TYPE, PAYLOAD_UNIT_TYPE,
    },
    world::blocks::power::{
        read_heater_generator_state, read_impact_reactor_state, read_light_block_state,
        read_nuclear_reactor_state, read_power_generator_state, read_variable_reactor_state,
        write_heater_generator_state, write_impact_reactor_state, write_light_block_state,
        write_nuclear_reactor_state, write_power_generator_state, write_variable_reactor_state,
        HeaterGeneratorState, ImpactReactorState, LightBlockState, NuclearReactorState,
        PowerGeneratorState, VariableReactorState,
    },
    world::blocks::production::{
        read_beam_drill_state, read_burst_drill_state, read_drill_state,
        read_generic_crafter_state, read_separator_state, write_beam_drill_state,
        write_burst_drill_state, write_drill_state, write_generic_crafter_state,
        write_separator_state, BeamDrillState, BurstDrillState, DrillState, GenericCrafterState,
        SeparatorState,
    },
    world::blocks::sandbox::{
        read_item_source_config, read_liquid_source_config, write_item_source_config,
        write_liquid_source_config, ItemSourceState, LiquidSourceState,
    },
    world::blocks::storage::{
        read_core_state, read_unloader_sort_item, write_core_state, write_unloader_sort_item,
        CoreBuildState,
    },
    world::blocks::units::{
        read_reconstructor_state, read_repair_turret_state, read_unit_assembler_state,
        read_unit_cargo_loader_state, read_unit_cargo_unload_state, read_unit_factory_state,
        write_reconstructor_state, write_repair_turret_state, write_unit_assembler_state,
        write_unit_cargo_loader_state, write_unit_cargo_unload_state, write_unit_factory_state,
        ReconstructorState, RepairTurretState, UnitAssemblerState, UnitCargoLoaderState,
        UnitCargoUnloadPointState, UnitFactoryState,
    },
    world::blocks::{is_construct_block_name, read_construct_block_state, ConstructBlockState},
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

#[derive(Debug, Clone)]
struct GameRuntimeMapEntityRecord {
    block_id: i16,
    is_center: bool,
    building: Option<Vec<u8>>,
}

fn write_network_map_block_state_tail<W: io::Write>(
    runtime: &GameRuntime,
    content: &ContentLoader,
    building: &BuildingComp,
    write: &mut W,
) -> io::Result<()> {
    let Some(block) = content.block(building.block.id) else {
        return Ok(());
    };

    if let (BlockDef::DefenseWall(wall), Some(state)) = (
        block,
        runtime.defense_wall_runtime_states.get(&building.tile_pos),
    ) {
        match (wall.kind, state) {
            (DefenseWallKind::Door, GameRuntimeDefenseWallState::Door(state)) => {
                write_door_state(write, *state)?;
            }
            (DefenseWallKind::AutoDoor, GameRuntimeDefenseWallState::Door(state)) => {
                write_auto_door_state(write, *state)?;
            }
            (DefenseWallKind::ShieldWall, GameRuntimeDefenseWallState::ShieldWall(state)) => {
                write_shield_wall_state(write, state)?;
            }
            _ => {}
        }
    }

    if let (BlockDef::Effect(effect), Some(state)) =
        (block, runtime.effect_runtime_store.get(building.tile_pos))
    {
        match (effect.kind, state) {
            (
                EffectBlockKind::MendProjector,
                EffectBlockRuntimeState::Projector(EffectProjectorRuntimeState::Mend(state)),
            ) => {
                write_mend_projector_state(write, state)?;
            }
            (
                EffectBlockKind::OverdriveProjector,
                EffectBlockRuntimeState::Projector(EffectProjectorRuntimeState::Overdrive(state)),
            ) => {
                write_overdrive_projector_state(write, state)?;
            }
            (EffectBlockKind::ForceProjector, EffectBlockRuntimeState::ForceProjector(state)) => {
                write_force_projector_state(write, state)?;
            }
            (EffectBlockKind::Radar, EffectBlockRuntimeState::Radar(state)) => {
                write_radar_state(write, state)?;
            }
            (EffectBlockKind::BaseShield, EffectBlockRuntimeState::BaseShield(state)) => {
                write_base_shield_state(write, state)?;
            }
            (EffectBlockKind::BuildTurret, EffectBlockRuntimeState::BuildTurret(state)) => {
                build_turret_write_child_with_loader(write, content, state)?;
            }
            _ => {}
        }
    }

    if let (BlockDef::Turret(turret), Some(state)) =
        (block, runtime.turret_runtime_states.get(&building.tile_pos))
    {
        match (turret.kind, state) {
            (TurretBlockKind::ItemTurret, GameRuntimeTurretBlockState::Item { turret, ammo }) => {
                turret_write_child(write, turret)?;
                item_turret_write_ammo(write, ammo)?;
            }
            (
                TurretBlockKind::PayloadAmmoTurret,
                GameRuntimeTurretBlockState::PayloadAmmo { turret, payloads },
            ) => {
                turret_write_child(write, turret)?;
                payload_ammo_turret_write_payloads(write, payloads)?;
            }
            (
                TurretBlockKind::ContinuousTurret | TurretBlockKind::ContinuousLiquidTurret,
                GameRuntimeTurretBlockState::Continuous { turret, continuous },
            ) => {
                turret_write_child(write, turret)?;
                continuous_turret_write_child(write, continuous)?;
            }
            (
                TurretBlockKind::PointDefenseTurret,
                GameRuntimeTurretBlockState::PointDefense(state),
            ) => {
                point_defense_write_child(write, state)?;
            }
            (
                TurretBlockKind::TractorBeamTurret,
                GameRuntimeTurretBlockState::TractorBeam(state),
            ) => {
                tractor_beam_write_child(write, state)?;
            }
            (
                TurretBlockKind::LiquidTurret
                | TurretBlockKind::PowerTurret
                | TurretBlockKind::LaserTurret,
                GameRuntimeTurretBlockState::Generic(state),
            ) => {
                turret_write_child(write, state)?;
            }
            _ => {}
        }
    }

    if let Some(state) = runtime.power_runtime_states.get(&building.tile_pos) {
        match (block, state) {
            (BlockDef::Power(power), GameRuntimePowerBlockState::Generator(state))
                if matches!(
                    power.kind,
                    PowerBlockKind::ConsumeGenerator
                        | PowerBlockKind::ThermalGenerator
                        | PowerBlockKind::SolarGenerator
                ) =>
            {
                write_power_generator_state(write, state)?;
            }
            (BlockDef::Power(power), GameRuntimePowerBlockState::NuclearReactor(state))
                if power.kind == PowerBlockKind::NuclearReactor =>
            {
                write_nuclear_reactor_state(write, state)?;
            }
            (BlockDef::Power(power), GameRuntimePowerBlockState::ImpactReactor(state))
                if power.kind == PowerBlockKind::ImpactReactor =>
            {
                write_impact_reactor_state(write, state)?;
            }
            (BlockDef::Power(power), GameRuntimePowerBlockState::VariableReactor(state))
                if power.kind == PowerBlockKind::VariableReactor =>
            {
                write_variable_reactor_state(write, state)?;
            }
            (BlockDef::Power(power), GameRuntimePowerBlockState::HeaterGenerator(state))
                if power.kind == PowerBlockKind::HeaterGenerator =>
            {
                write_heater_generator_state(write, state)?;
            }
            (BlockDef::Light(_), GameRuntimePowerBlockState::Light(state)) => {
                write_light_block_state(write, state)?;
            }
            _ => {}
        }
    }

    if let (BlockDef::Production(production), Some(state)) = (
        block,
        runtime.production_runtime_states.get(&building.tile_pos),
    ) {
        match (production.kind, state) {
            (ProductionBlockKind::Drill, GameRuntimeProductionBlockState::Drill(state)) => {
                write_drill_state(write, state)?;
            }
            (ProductionBlockKind::BeamDrill, GameRuntimeProductionBlockState::BeamDrill(state)) => {
                write_beam_drill_state(write, state)?;
            }
            (
                ProductionBlockKind::BurstDrill,
                GameRuntimeProductionBlockState::BurstDrill(state),
            ) => {
                write_burst_drill_state(write, state)?;
            }
            _ => {}
        }
    }

    if let (BlockDef::Crafting(crafting), Some(state)) = (
        block,
        runtime.crafting_runtime_states.get(&building.tile_pos),
    ) {
        match (crafting.kind, state) {
            (
                CraftingBlockKind::GenericCrafter
                | CraftingBlockKind::AttributeCrafter
                | CraftingBlockKind::HeatCrafter,
                GameRuntimeCraftingBlockState::GenericCrafter(state),
            ) => {
                write_generic_crafter_state(write, state, crafting.legacy_read_warmup)?;
            }
            (CraftingBlockKind::Separator, GameRuntimeCraftingBlockState::Separator(state)) => {
                write_separator_state(write, state)?;
            }
            (
                CraftingBlockKind::HeatProducer,
                GameRuntimeCraftingBlockState::HeatProducer { crafter, heat },
            ) => {
                write_generic_crafter_state(write, crafter, crafting.legacy_read_warmup)?;
                write_heat_producer_state(write, heat)?;
            }
            _ => {}
        }
    }

    if let (BlockDef::Distribution(distribution), Some(state)) = (
        block,
        runtime.distribution_runtime_states.get(&building.tile_pos),
    ) {
        match (distribution.kind, state) {
            (
                DistributionBlockKind::Conveyor | DistributionBlockKind::ArmoredConveyor,
                GameRuntimeDistributionBlockState::Conveyor(state),
            ) => {
                write_conveyor_state(write, state)?;
            }
            (
                DistributionBlockKind::StackConveyor,
                GameRuntimeDistributionBlockState::StackConveyor(state),
            ) => {
                write_stack_conveyor_state(write, state)?;
            }
            (
                DistributionBlockKind::ItemBridge | DistributionBlockKind::DuctBridge,
                GameRuntimeDistributionBlockState::ItemBridge(state),
            ) => {
                write_item_bridge_state(write, state)?;
            }
            (
                DistributionBlockKind::BufferedItemBridge,
                GameRuntimeDistributionBlockState::BufferedItemBridge(state),
            ) => {
                write_buffered_bridge_state(write, state)?;
            }
            (
                DistributionBlockKind::MassDriver,
                GameRuntimeDistributionBlockState::MassDriver(state),
            ) => {
                write_mass_driver_state(write, state)?;
            }
            (
                DistributionBlockKind::DirectionalUnloader,
                GameRuntimeDistributionBlockState::DirectionalUnloader(state),
            ) => {
                write_directional_unloader_state(write, state)?;
            }
            (DistributionBlockKind::Duct, GameRuntimeDistributionBlockState::Duct(state)) => {
                write_duct_state(write, state)?;
            }
            (
                DistributionBlockKind::DuctRouter
                | DistributionBlockKind::OverflowDuct
                | DistributionBlockKind::StackRouter,
                GameRuntimeDistributionBlockState::DuctRouter(state),
            ) => {
                write_duct_router_state(write, state)?;
            }
            (
                DistributionBlockKind::Junction,
                GameRuntimeDistributionBlockState::DuctJunction(state),
            ) => {
                write_duct_junction_state(write, state)?;
            }
            (DistributionBlockKind::Sorter, GameRuntimeDistributionBlockState::Sorter(state)) => {
                write_sorter_state(write, state)?;
            }
            (
                DistributionBlockKind::Unloader,
                GameRuntimeDistributionBlockState::Unloader(sort_item),
            ) => {
                write_unloader_sort_item(write, sort_item.map(i32::from))?;
            }
            (
                DistributionBlockKind::UnitCargoLoader,
                GameRuntimeDistributionBlockState::UnitCargoLoader(state),
            ) => {
                write_unit_cargo_loader_state(
                    write,
                    (state.read_unit_id >= 0).then_some(state.read_unit_id),
                )?;
            }
            (
                DistributionBlockKind::UnitCargoUnloadPoint,
                GameRuntimeDistributionBlockState::UnitCargoUnload(state),
            ) => {
                write_unit_cargo_unload_state(write, state)?;
            }
            _ => {}
        }
    }

    if let (BlockDef::Logic(logic), Some(state)) =
        (block, runtime.logic_runtime_states.get(&building.tile_pos))
    {
        match (logic.kind, state) {
            (LogicBlockKind::Message, GameRuntimeLogicBlockState::Message(state)) => {
                write_message_state(write, state)?;
            }
            (LogicBlockKind::Switch, GameRuntimeLogicBlockState::Switch { enabled }) => {
                write_switch_enabled(write, *enabled)?;
            }
            (
                LogicBlockKind::Display | LogicBlockKind::TileDisplay,
                GameRuntimeLogicBlockState::Display(state),
            ) => {
                write_logic_display_state(write, state)?;
            }
            (LogicBlockKind::Memory, GameRuntimeLogicBlockState::Memory(state)) => {
                write_memory_state(write, state)?;
            }
            (LogicBlockKind::Canvas, GameRuntimeLogicBlockState::Canvas(state)) => {
                write_canvas_state(write, state)?;
            }
            (LogicBlockKind::Processor, GameRuntimeLogicBlockState::Processor(state)) => {
                write_logic_processor_state(
                    write,
                    state,
                    4,
                    logic.privileged_only,
                    logic.max_instructions_per_tick.max(1) as i16,
                )?;
            }
            _ => {}
        }
    }

    if let (BlockDef::Campaign(campaign), Some(state)) = (
        block,
        runtime.campaign_runtime_states.get(&building.tile_pos),
    ) {
        match (campaign.kind, state) {
            (
                CampaignBlockKind::LaunchPad | CampaignBlockKind::AdvancedLaunchPad,
                GameRuntimeCampaignBlockState::LaunchPad(state),
            ) => {
                write_launch_pad_state(write, state)?;
            }
            (CampaignBlockKind::LandingPad, GameRuntimeCampaignBlockState::LandingPad(state)) => {
                write_landing_pad_state(write, state)?;
            }
            (CampaignBlockKind::Accelerator, GameRuntimeCampaignBlockState::Accelerator(state)) => {
                write_accelerator_state(write, state)?;
            }
            _ => {}
        }
    }

    if let Some(state) = runtime.payload_runtime_states.get(&building.tile_pos) {
        match (block, state) {
            (BlockDef::Payload(payload), GameRuntimePayloadBlockState::Conveyor(conveyor))
                if payload.kind == PayloadBlockKind::PayloadConveyor =>
            {
                write_payload_conveyor_extra(
                    write,
                    conveyor.progress,
                    conveyor.item_rotation,
                    conveyor.item.as_ref(),
                )?;
            }
            (
                BlockDef::Payload(payload),
                GameRuntimePayloadBlockState::Router {
                    conveyor,
                    sorted,
                    rec_dir,
                },
            ) if payload.kind == PayloadBlockKind::PayloadRouter => {
                write_payload_conveyor_extra(
                    write,
                    conveyor.progress,
                    conveyor.item_rotation,
                    conveyor.item.as_ref(),
                )?;
                write_payload_router_extra(write, *sorted, *rec_dir)?;
            }
            (
                BlockDef::PayloadMassDriver(_),
                GameRuntimePayloadBlockState::MassDriver { common, driver },
            ) => {
                write_payload_block_build_common(write, common)?;
                write_payload_mass_driver_extra(write, driver)?;
            }
            (
                BlockDef::PayloadLoader(_),
                GameRuntimePayloadBlockState::Loader { common, loader },
            ) => {
                write_payload_block_build_common(write, common)?;
                write_payload_loader_extra(write, loader.exporting)?;
            }
            (
                BlockDef::PayloadDeconstructor(_),
                GameRuntimePayloadBlockState::Deconstructor {
                    common,
                    deconstructor,
                },
            ) => {
                write_payload_block_build_common(write, common)?;
                write_deconstructor_extra(
                    write,
                    deconstructor.progress,
                    deconstructor.accum.as_deref(),
                )?;
                write_payload_ref(write, deconstructor.deconstructing.as_ref())?;
            }
            (
                BlockDef::PayloadConstructor(_),
                GameRuntimePayloadBlockState::Constructor {
                    common,
                    producer,
                    recipe,
                },
            ) => {
                write_payload_block_build_common(write, common)?;
                write_block_producer_progress(write, producer.progress)?;
                write_constructor_recipe(write, *recipe)?;
            }
            (
                BlockDef::Sandbox(sandbox),
                GameRuntimePayloadBlockState::Source { common, source },
            ) if sandbox.kind == SandboxBlockKind::PayloadSource => {
                write_payload_block_build_common(write, common)?;
                write_payload_source_extra(
                    write,
                    source.unit,
                    source.config_block,
                    source.command_pos,
                )?;
            }
            (BlockDef::Sandbox(sandbox), GameRuntimePayloadBlockState::Void(common))
                if sandbox.kind == SandboxBlockKind::PayloadVoid =>
            {
                write_payload_block_build_common(write, common)?;
            }
            _ => {}
        }
    }

    if let Some(state) = runtime.unit_runtime_states.get(&building.tile_pos) {
        match (block, state) {
            (BlockDef::UnitFactory(_), GameRuntimeUnitBlockState::Factory { common, factory }) => {
                write_payload_block_build_common(write, common)?;
                write_unit_factory_state(write, factory)?;
            }
            (
                BlockDef::UnitReconstructor(_),
                GameRuntimeUnitBlockState::Reconstructor {
                    common,
                    reconstructor,
                },
            ) => {
                write_payload_block_build_common(write, common)?;
                write_reconstructor_state(write, reconstructor)?;
            }
            (BlockDef::UnitRepairTower(_), GameRuntimeUnitBlockState::RepairTower(state)) => {
                write_repair_turret_state(write, state)?;
            }
            (
                BlockDef::UnitAssembler(_),
                GameRuntimeUnitBlockState::Assembler { common, assembler },
            ) => {
                write_payload_block_build_common(write, common)?;
                write_unit_assembler_state(write, assembler)?;
            }
            (
                BlockDef::UnitAssemblerModule(_),
                GameRuntimeUnitBlockState::AssemblerModule(common),
            ) => {
                write_payload_block_build_common(write, common)?;
            }
            _ => {}
        }
    }

    if let (BlockDef::Liquid(liquid), Some(GameRuntimeLiquidBlockState::Bridge(state))) =
        (block, runtime.liquid_runtime_states.get(&building.tile_pos))
    {
        if liquid.kind == LiquidBlockKind::LiquidBridge {
            write_liquid_bridge_state(write, state)?;
        }
    }

    if let (BlockDef::Storage(storage), Some(GameRuntimeStorageBlockState::Core(state))) = (
        block,
        runtime.storage_runtime_states.get(&building.tile_pos),
    ) {
        if storage.kind == StorageBlockKind::Core {
            write_core_state(write, state)?;
        }
    }

    if let (BlockDef::Sandbox(sandbox), Some(state)) = (
        block,
        runtime.sandbox_runtime_states.get(&building.tile_pos),
    ) {
        match (sandbox.kind, state) {
            (SandboxBlockKind::ItemSource, GameRuntimeSandboxBlockState::ItemSource(state)) => {
                write_item_source_config(write, state.output_item)?;
            }
            (SandboxBlockKind::LiquidSource, GameRuntimeSandboxBlockState::LiquidSource(state)) => {
                write_liquid_source_config(write, state.source)?;
            }
            _ => {}
        }
    }

    if let (BlockDef::Legacy(legacy), Some(state)) =
        (block, runtime.legacy_runtime_states.get(&building.tile_pos))
    {
        match (legacy.kind, state) {
            (LegacyBlockKind::CommandCenter, GameRuntimeLegacyBlockState::CommandCenter(_)) => {
                write_legacy_command_center_extra(write)?;
            }
            (LegacyBlockKind::MechPad, GameRuntimeLegacyBlockState::MechPad(values)) => {
                write_legacy_mech_pad_extra(write, *values)?;
            }
            (LegacyBlockKind::UnitFactory, GameRuntimeLegacyBlockState::UnitFactory(extra)) => {
                write_legacy_unit_factory_extra(write, 0, extra)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn network_map_building_revision(
    runtime: &GameRuntime,
    content: &ContentLoader,
    building: &BuildingComp,
) -> u8 {
    let Some(block) = content.block(building.block.id) else {
        return 0;
    };

    if let (BlockDef::Effect(effect), Some(EffectBlockRuntimeState::BaseShield(_))) =
        (block, runtime.effect_runtime_store.get(building.tile_pos))
    {
        if effect.kind == EffectBlockKind::BaseShield {
            return 1;
        }
    }

    if let (BlockDef::Turret(turret), Some(state)) =
        (block, runtime.turret_runtime_states.get(&building.tile_pos))
    {
        match (turret.kind, state) {
            (TurretBlockKind::ItemTurret, GameRuntimeTurretBlockState::Item { .. }) => return 2,
            (
                TurretBlockKind::ContinuousTurret | TurretBlockKind::ContinuousLiquidTurret,
                GameRuntimeTurretBlockState::Continuous { .. },
            ) => return 3,
            (
                TurretBlockKind::PayloadAmmoTurret,
                GameRuntimeTurretBlockState::PayloadAmmo { .. },
            )
            | (
                TurretBlockKind::LiquidTurret
                | TurretBlockKind::PowerTurret
                | TurretBlockKind::LaserTurret,
                GameRuntimeTurretBlockState::Generic(_),
            ) => return 1,
            _ => {}
        }
    }

    if let (
        BlockDef::Production(production),
        Some(
            GameRuntimeProductionBlockState::Drill(_)
            | GameRuntimeProductionBlockState::BeamDrill(_)
            | GameRuntimeProductionBlockState::BurstDrill(_),
        ),
    ) = (
        block,
        runtime.production_runtime_states.get(&building.tile_pos),
    ) {
        if matches!(
            production.kind,
            ProductionBlockKind::Drill
                | ProductionBlockKind::BeamDrill
                | ProductionBlockKind::BurstDrill
        ) {
            return 1;
        }
    }

    if let (BlockDef::Crafting(crafting), Some(GameRuntimeCraftingBlockState::Separator(_))) = (
        block,
        runtime.crafting_runtime_states.get(&building.tile_pos),
    ) {
        if crafting.kind == CraftingBlockKind::Separator {
            return 1;
        }
    }

    if let (BlockDef::Liquid(liquid), Some(GameRuntimeLiquidBlockState::Bridge(_))) =
        (block, runtime.liquid_runtime_states.get(&building.tile_pos))
    {
        if liquid.kind == LiquidBlockKind::LiquidBridge {
            return 1;
        }
    }

    if let (BlockDef::Storage(storage), Some(GameRuntimeStorageBlockState::Core(_))) = (
        block,
        runtime.storage_runtime_states.get(&building.tile_pos),
    ) {
        if storage.kind == StorageBlockKind::Core {
            return 1;
        }
    }

    if let (BlockDef::Distribution(distribution), Some(state)) = (
        block,
        runtime.distribution_runtime_states.get(&building.tile_pos),
    ) {
        match (distribution.kind, state) {
            (
                DistributionBlockKind::Conveyor | DistributionBlockKind::ArmoredConveyor,
                GameRuntimeDistributionBlockState::Conveyor(_),
            )
            | (
                DistributionBlockKind::ItemBridge | DistributionBlockKind::DuctBridge,
                GameRuntimeDistributionBlockState::ItemBridge(_),
            )
            | (
                DistributionBlockKind::BufferedItemBridge,
                GameRuntimeDistributionBlockState::BufferedItemBridge(_),
            )
            | (DistributionBlockKind::Duct, GameRuntimeDistributionBlockState::Duct(_))
            | (
                DistributionBlockKind::DuctRouter
                | DistributionBlockKind::OverflowDuct
                | DistributionBlockKind::StackRouter,
                GameRuntimeDistributionBlockState::DuctRouter(_),
            )
            | (
                DistributionBlockKind::Junction,
                GameRuntimeDistributionBlockState::DuctJunction(_),
            )
            | (DistributionBlockKind::Unloader, GameRuntimeDistributionBlockState::Unloader(_)) => {
                return 1
            }
            (DistributionBlockKind::Sorter, GameRuntimeDistributionBlockState::Sorter(_)) => {
                return 2;
            }
            _ => {}
        }
    }

    if let (BlockDef::Logic(logic), Some(state)) =
        (block, runtime.logic_runtime_states.get(&building.tile_pos))
    {
        match (logic.kind, state) {
            (LogicBlockKind::Switch, GameRuntimeLogicBlockState::Switch { .. })
            | (
                LogicBlockKind::Display | LogicBlockKind::TileDisplay,
                GameRuntimeLogicBlockState::Display(_),
            ) => return 1,
            (LogicBlockKind::Processor, GameRuntimeLogicBlockState::Processor(_)) => return 4,
            _ => {}
        }
    }

    if let (BlockDef::Campaign(campaign), Some(state)) = (
        block,
        runtime.campaign_runtime_states.get(&building.tile_pos),
    ) {
        match (campaign.kind, state) {
            (
                CampaignBlockKind::LaunchPad | CampaignBlockKind::AdvancedLaunchPad,
                GameRuntimeCampaignBlockState::LaunchPad(_),
            )
            | (CampaignBlockKind::LandingPad, GameRuntimeCampaignBlockState::LandingPad(_))
            | (CampaignBlockKind::Accelerator, GameRuntimeCampaignBlockState::Accelerator(_)) => {
                return 1;
            }
            _ => {}
        }
    }

    if let Some(state) = runtime.payload_runtime_states.get(&building.tile_pos) {
        match (block, state) {
            (BlockDef::Payload(payload), GameRuntimePayloadBlockState::Router { .. })
                if payload.kind == PayloadBlockKind::PayloadRouter =>
            {
                return 1
            }
            (BlockDef::PayloadMassDriver(_), GameRuntimePayloadBlockState::MassDriver { .. })
            | (BlockDef::PayloadLoader(_), GameRuntimePayloadBlockState::Loader { .. }) => {
                return 1;
            }
            (BlockDef::Sandbox(sandbox), GameRuntimePayloadBlockState::Source { .. })
                if sandbox.kind == SandboxBlockKind::PayloadSource =>
            {
                return 1
            }
            _ => {}
        }
    }

    if let Some(state) = runtime.unit_runtime_states.get(&building.tile_pos) {
        match (block, state) {
            (BlockDef::UnitFactory(_), GameRuntimeUnitBlockState::Factory { .. })
            | (BlockDef::UnitReconstructor(_), GameRuntimeUnitBlockState::Reconstructor { .. }) => {
                return 3;
            }
            (BlockDef::UnitRepairTower(_), GameRuntimeUnitBlockState::RepairTower(_))
            | (BlockDef::UnitAssembler(_), GameRuntimeUnitBlockState::Assembler { .. }) => {
                return 1;
            }
            _ => {}
        }
    }

    if let (BlockDef::Sandbox(sandbox), Some(GameRuntimeSandboxBlockState::LiquidSource(_))) = (
        block,
        runtime.sandbox_runtime_states.get(&building.tile_pos),
    ) {
        if sandbox.kind == SandboxBlockKind::LiquidSource {
            return 1;
        }
    }

    match (block, runtime.power_runtime_states.get(&building.tile_pos)) {
        (
            BlockDef::Power(power),
            Some(
                GameRuntimePowerBlockState::Generator(_)
                | GameRuntimePowerBlockState::NuclearReactor(_)
                | GameRuntimePowerBlockState::ImpactReactor(_)
                | GameRuntimePowerBlockState::VariableReactor(_)
                | GameRuntimePowerBlockState::HeaterGenerator(_),
            ),
        ) if matches!(
            power.kind,
            PowerBlockKind::ConsumeGenerator
                | PowerBlockKind::ThermalGenerator
                | PowerBlockKind::SolarGenerator
                | PowerBlockKind::NuclearReactor
                | PowerBlockKind::ImpactReactor
                | PowerBlockKind::VariableReactor
                | PowerBlockKind::HeaterGenerator
        ) =>
        {
            1
        }
        _ => 0,
    }
}

fn network_map_building_payload(
    runtime: &GameRuntime,
    content: &ContentLoader,
    building: &BuildingComp,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    // The outer byte is the block/build revision used by Java save-map chunks.
    // Most migrated writers still use the base revision 0. Java PowerGenerator
    // overrides version() to 1 because generateTime was added behind
    // revision>=1, so the runtime computes the revision from the actual block
    // kind and sidecar state before writing block-specific tails.
    bytes.push(network_map_building_revision(runtime, content, building));
    building
        .write_base(&mut bytes, false)
        .expect("BuildingComp base payload should be writable into Vec<u8>");
    write_network_map_block_state_tail(runtime, content, building, &mut bytes)
        .expect("block-specific building payload should be writable into Vec<u8>");
    bytes
}

fn network_map_entity_records(
    runtime: &GameRuntime,
    content: &ContentLoader,
    width: usize,
    height: usize,
) -> BTreeMap<usize, GameRuntimeMapEntityRecord> {
    let mut records = BTreeMap::new();
    for building in &runtime.buildings {
        let center_x = building.tile_x();
        let center_y = building.tile_y();
        let center_payload = network_map_building_payload(runtime, content, building);
        for (x, y) in footprint_tiles(center_x, center_y, building.block.size) {
            if x < 0 || y < 0 {
                continue;
            }
            let x = x as usize;
            let y = y as usize;
            if x >= width || y >= height {
                continue;
            }
            let index = x + y * width;
            let is_center = x as i32 == center_x && y as i32 == center_y;
            records.insert(
                index,
                GameRuntimeMapEntityRecord {
                    block_id: building.block.id,
                    is_center,
                    building: is_center.then(|| center_payload.clone()),
                },
            );
        }
    }
    records
}

fn network_map_tile_data(tile: &Tile) -> Option<LegacyMapTileData> {
    (tile.data != 0 || tile.floor_data != 0 || tile.overlay_data != 0 || tile.extra_data != 0)
        .then_some(LegacyMapTileData {
            data: tile.data,
            floor_data: tile.floor_data,
            overlay_data: tile.overlay_data,
            extra_data: tile.extra_data,
        })
}

fn export_network_map_snapshot_from_parts(
    runtime: &GameRuntime,
    content: &ContentLoader,
) -> LegacyShortChunkMap {
    let world = &runtime.state.world;
    let width = u16::try_from(world.width()).unwrap_or(u16::MAX);
    let height = u16::try_from(world.height()).unwrap_or(u16::MAX);
    let tile_count = width as usize * height as usize;
    let tiles = world.tiles.iter().take(tile_count).collect::<Vec<_>>();
    let entities = network_map_entity_records(runtime, content, width as usize, height as usize);

    let mut floors = Vec::new();
    let mut floor_start = 0usize;
    while floor_start < tiles.len() {
        let floor = tiles[floor_start].floor;
        let overlay = tiles[floor_start].overlay;
        let mut len = 1usize;
        while floor_start + len < tiles.len()
            && len < u8::MAX as usize + 1
            && tiles[floor_start + len].floor == floor
            && tiles[floor_start + len].overlay == overlay
        {
            len += 1;
        }
        floors.push(LegacyMapFloorRecord {
            index: floor_start,
            floor_id: floor,
            ore_id: overlay,
            consecutives: (len - 1) as u8,
        });
        floor_start += len;
    }

    let mut blocks = Vec::new();
    let mut block_start = 0usize;
    while block_start < tiles.len() {
        let tile = tiles[block_start];
        let tile_data = network_map_tile_data(tile);
        if let Some(entity) = entities.get(&block_start) {
            blocks.push(LegacyMapBlockRecord {
                index: block_start,
                block_id: entity.block_id,
                packed_flags: 1 | if tile_data.is_some() { 4 } else { 0 },
                has_entity: true,
                has_old_data: false,
                has_new_data: tile_data.is_some(),
                is_center: entity.is_center,
                new_data: tile_data,
                old_data: None,
                building: entity.building.clone(),
                consecutives: 0,
            });
            block_start += 1;
            continue;
        }

        if let Some(tile_data) = tile_data {
            blocks.push(LegacyMapBlockRecord {
                index: block_start,
                block_id: tile.block,
                packed_flags: 0,
                has_entity: false,
                has_old_data: false,
                has_new_data: true,
                is_center: true,
                new_data: Some(tile_data),
                old_data: None,
                building: None,
                consecutives: 0,
            });
            block_start += 1;
            continue;
        }

        let block = tile.block;
        let mut len = 1usize;
        while block_start + len < tiles.len()
            && len < u8::MAX as usize + 1
            && !entities.contains_key(&(block_start + len))
            && tiles[block_start + len].block == block
            && tiles[block_start + len].data == 0
            && tiles[block_start + len].floor_data == 0
            && tiles[block_start + len].overlay_data == 0
            && tiles[block_start + len].extra_data == 0
        {
            len += 1;
        }
        blocks.push(LegacyMapBlockRecord {
            index: block_start,
            block_id: block,
            packed_flags: 0,
            has_entity: false,
            has_old_data: false,
            has_new_data: false,
            is_center: true,
            new_data: None,
            old_data: None,
            building: None,
            consecutives: (len - 1) as u8,
        });
        block_start += len;
    }

    LegacyShortChunkMap {
        width,
        height,
        floors,
        blocks,
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameRuntimePayloadReadMode {
    TopLevel,
    NestedExact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameRuntimeUnitPayloadSchema {
    Common,
    BaseRotation,
    Payloads,
    BuildingPayloads,
    Missile,
    Ammo,
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
    Void(PayloadBlockBuildState),
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
pub enum GameRuntimeProductionBlockState {
    Drill(DrillState),
    BeamDrill(BeamDrillState),
    BurstDrill(BurstDrillState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimeCraftingBlockState {
    GenericCrafter(GenericCrafterState),
    Separator(SeparatorState),
    HeatProducer {
        crafter: GenericCrafterState,
        heat: HeatProducerState,
    },
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
    Sorter(SorterState),
    Unloader(Option<ContentId>),
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
pub enum GameRuntimeLogicBlockState {
    Message(MessageBlockState),
    Switch { enabled: bool },
    Display(LogicDisplayState),
    Memory(MemoryBlockState),
    Canvas(CanvasBlockState),
    Processor(LogicProcessorState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimeCampaignBlockState {
    LaunchPad(LaunchPadState),
    LandingPad(LandingPadState),
    Accelerator(AcceleratorState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimeSandboxBlockState {
    ItemSource(ItemSourceState),
    LiquidSource(LiquidSourceState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimeLegacyBlockState {
    CommandCenter(u8),
    MechPad([f32; 3]),
    UnitFactory(LegacyUnitFactoryExtra),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimeUnitBlockState {
    Factory {
        common: PayloadBlockBuildState,
        factory: UnitFactoryState,
    },
    Reconstructor {
        common: PayloadBlockBuildState,
        reconstructor: ReconstructorState,
    },
    RepairTower(RepairTurretState),
    Assembler {
        common: PayloadBlockBuildState,
        assembler: UnitAssemblerState,
    },
    AssemblerModule(PayloadBlockBuildState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimeDefenseWallState {
    Door(DoorState),
    ShieldWall(ShieldWallState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuntimeTurretBlockState {
    Generic(TurretState),
    Item {
        turret: TurretState,
        ammo: Vec<ItemAmmoEntry>,
    },
    PayloadAmmo {
        turret: TurretState,
        payloads: PayloadSeq,
    },
    Continuous {
        turret: TurretState,
        continuous: ContinuousTurretState,
    },
    PointDefense(PointDefenseState),
    TractorBeam(TractorBeamState),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameRuntimeConstructBlockState {
    pub size: i32,
    pub state: ConstructBlockState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameRuntimePayloadConstructorFrameReport {
    pub visited_buildings: usize,
    pub constructor_candidates: usize,
    pub updated_constructors: usize,
    pub produced_payloads: usize,
    pub missing_runtime_states: usize,
    pub missing_recipe_build_times: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum GameRuntimeLoadedBlockState {
    Construct(GameRuntimeConstructBlockState),
    Effect(EffectBlockRuntimeState),
    Payload(GameRuntimePayloadBlockState),
    Power(GameRuntimePowerBlockState),
    Production(GameRuntimeProductionBlockState),
    Crafting(GameRuntimeCraftingBlockState),
    Distribution(GameRuntimeDistributionBlockState),
    Storage(GameRuntimeStorageBlockState),
    Liquid(GameRuntimeLiquidBlockState),
    Logic(GameRuntimeLogicBlockState),
    Campaign(GameRuntimeCampaignBlockState),
    Sandbox(GameRuntimeSandboxBlockState),
    Legacy(GameRuntimeLegacyBlockState),
    Unit(GameRuntimeUnitBlockState),
    DefenseWall(GameRuntimeDefenseWallState),
    Turret(GameRuntimeTurretBlockState),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameRuntime {
    pub state: GameState,
    pub buildings: Vec<BuildingComp>,
    pub effect_runtime_store: EffectBlockRuntimeStateStore,
    pub effect_timer_store: EffectBlockTimerStateStore,
    pub payload_runtime_states: BTreeMap<i32, GameRuntimePayloadBlockState>,
    pub power_runtime_states: BTreeMap<i32, GameRuntimePowerBlockState>,
    pub production_runtime_states: BTreeMap<i32, GameRuntimeProductionBlockState>,
    pub crafting_runtime_states: BTreeMap<i32, GameRuntimeCraftingBlockState>,
    pub distribution_runtime_states: BTreeMap<i32, GameRuntimeDistributionBlockState>,
    pub storage_runtime_states: BTreeMap<i32, GameRuntimeStorageBlockState>,
    pub liquid_runtime_states: BTreeMap<i32, GameRuntimeLiquidBlockState>,
    pub logic_runtime_states: BTreeMap<i32, GameRuntimeLogicBlockState>,
    pub campaign_runtime_states: BTreeMap<i32, GameRuntimeCampaignBlockState>,
    pub sandbox_runtime_states: BTreeMap<i32, GameRuntimeSandboxBlockState>,
    pub legacy_runtime_states: BTreeMap<i32, GameRuntimeLegacyBlockState>,
    pub unit_runtime_states: BTreeMap<i32, GameRuntimeUnitBlockState>,
    pub defense_wall_runtime_states: BTreeMap<i32, GameRuntimeDefenseWallState>,
    pub turret_runtime_states: BTreeMap<i32, GameRuntimeTurretBlockState>,
    pub construct_runtime_states: BTreeMap<i32, GameRuntimeConstructBlockState>,
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
            production_runtime_states: BTreeMap::new(),
            crafting_runtime_states: BTreeMap::new(),
            distribution_runtime_states: BTreeMap::new(),
            storage_runtime_states: BTreeMap::new(),
            liquid_runtime_states: BTreeMap::new(),
            logic_runtime_states: BTreeMap::new(),
            campaign_runtime_states: BTreeMap::new(),
            sandbox_runtime_states: BTreeMap::new(),
            legacy_runtime_states: BTreeMap::new(),
            unit_runtime_states: BTreeMap::new(),
            defense_wall_runtime_states: BTreeMap::new(),
            turret_runtime_states: BTreeMap::new(),
            construct_runtime_states: BTreeMap::new(),
        }
    }

    pub fn buildings(&self) -> &[BuildingComp] {
        &self.buildings
    }

    pub fn buildings_mut(&mut self) -> &mut [BuildingComp] {
        &mut self.buildings
    }

    pub fn export_network_map_snapshot(&self, content: &ContentLoader) -> LegacyShortChunkMap {
        export_network_map_snapshot_from_parts(self, content)
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
        self.production_runtime_states.remove(&removed.tile_pos);
        self.crafting_runtime_states.remove(&removed.tile_pos);
        self.distribution_runtime_states.remove(&removed.tile_pos);
        self.storage_runtime_states.remove(&removed.tile_pos);
        self.liquid_runtime_states.remove(&removed.tile_pos);
        self.logic_runtime_states.remove(&removed.tile_pos);
        self.campaign_runtime_states.remove(&removed.tile_pos);
        self.sandbox_runtime_states.remove(&removed.tile_pos);
        self.legacy_runtime_states.remove(&removed.tile_pos);
        self.unit_runtime_states.remove(&removed.tile_pos);
        self.defense_wall_runtime_states.remove(&removed.tile_pos);
        self.turret_runtime_states.remove(&removed.tile_pos);
        self.construct_runtime_states.remove(&removed.tile_pos);
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
                content,
                block,
                &building,
                revision,
                &mut building_bytes,
                GameRuntimePayloadReadMode::TopLevel,
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
            if block_state.is_some() && !building_bytes.is_empty() {
                report.block_state_bytes_ignored += 1;
            }

            let added_index = self.add_building(building);
            if let Some(block_state) = block_state {
                let tile_pos = self.buildings[added_index].tile_pos;
                match block_state {
                    GameRuntimeLoadedBlockState::Construct(block_state) => {
                        self.construct_runtime_states.insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
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
                    GameRuntimeLoadedBlockState::Production(block_state) => {
                        self.production_runtime_states.insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                    GameRuntimeLoadedBlockState::Crafting(block_state) => {
                        self.crafting_runtime_states.insert(tile_pos, block_state);
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
                    GameRuntimeLoadedBlockState::Logic(block_state) => {
                        self.logic_runtime_states.insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                    GameRuntimeLoadedBlockState::Campaign(block_state) => {
                        self.campaign_runtime_states.insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                    GameRuntimeLoadedBlockState::Sandbox(block_state) => {
                        self.sandbox_runtime_states.insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                    GameRuntimeLoadedBlockState::Legacy(block_state) => {
                        self.legacy_runtime_states.insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                    GameRuntimeLoadedBlockState::Unit(block_state) => {
                        self.unit_runtime_states.insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                    GameRuntimeLoadedBlockState::DefenseWall(block_state) => {
                        self.defense_wall_runtime_states
                            .insert(tile_pos, block_state);
                        report.block_states_added += 1;
                    }
                    GameRuntimeLoadedBlockState::Turret(block_state) => {
                        self.turret_runtime_states.insert(tile_pos, block_state);
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
        content: &ContentLoader,
        block: &BlockDef,
        building: &BuildingComp,
        revision: u8,
        building_payload: &mut &[u8],
        payload_read_mode: GameRuntimePayloadReadMode,
    ) -> Result<Option<GameRuntimeLoadedBlockState>, GameRuntimeBlockStateReadError> {
        match self.read_construct_runtime_state_from_building_payload(
            block,
            revision,
            building_payload,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Construct(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_effect_runtime_state_from_building_payload(
            content,
            block,
            revision,
            building_payload,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Effect(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_payload_runtime_state_from_building_payload(
            content,
            block,
            revision,
            building_payload,
            payload_read_mode,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Payload(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_power_runtime_state_from_building_payload(block, revision, building_payload)
        {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Power(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_production_runtime_state_from_building_payload(
            block,
            revision,
            building_payload,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Production(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_crafting_runtime_state_from_building_payload(
            block,
            revision,
            building_payload,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Crafting(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_distribution_runtime_state_from_building_payload(
            block,
            building,
            revision,
            building_payload,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Distribution(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_storage_runtime_state_from_building_payload(
            block,
            revision,
            building_payload,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Storage(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_liquid_runtime_state_from_building_payload(
            block,
            revision,
            building_payload,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Liquid(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_logic_runtime_state_from_building_payload(block, revision, building_payload)
        {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Logic(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_campaign_runtime_state_from_building_payload(
            block,
            revision,
            building_payload,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Campaign(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_sandbox_runtime_state_from_building_payload(
            block,
            revision,
            building_payload,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Sandbox(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_legacy_runtime_state_from_building_payload(
            block,
            revision,
            building_payload,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Legacy(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_unit_runtime_state_from_building_payload(
            content,
            block,
            revision,
            building_payload,
            payload_read_mode,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Unit(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        match self.read_turret_runtime_state_from_building_payload(
            block,
            revision,
            building_payload,
        ) {
            Ok(Some(state)) => return Ok(Some(GameRuntimeLoadedBlockState::Turret(state))),
            Ok(None) => return Ok(None),
            Err(GameRuntimeBlockStateReadError::Parse) => {
                return Err(GameRuntimeBlockStateReadError::Parse);
            }
            Err(GameRuntimeBlockStateReadError::Unsupported) => {}
        }

        self.read_defense_wall_runtime_state_from_building_payload(block, building_payload)
            .map(|state| state.map(GameRuntimeLoadedBlockState::DefenseWall))
    }

    fn read_construct_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeConstructBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::Plain(block) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };
        if !is_construct_block_name(&block.name) {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        }

        read_construct_block_state(building_payload, revision)
            .map(|state| {
                Some(GameRuntimeConstructBlockState {
                    size: block.size,
                    state,
                })
            })
            .map_err(|_| GameRuntimeBlockStateReadError::Parse)
    }

    fn read_effect_runtime_state_from_building_payload(
        &self,
        content: &ContentLoader,
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
            EffectBlockKind::BuildTurret => {
                build_turret_read_child_with_loader(building_payload, content)
                    .map(|state| Some(EffectBlockRuntimeState::BuildTurret(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            _ => Err(GameRuntimeBlockStateReadError::Unsupported),
        }
    }

    fn read_exact_payload_block_build_common(
        &self,
        content: &ContentLoader,
        read: &mut &[u8],
    ) -> io::Result<PayloadBlockBuildState> {
        let pay_vector = PayloadVec2 {
            x: type_io::read_f32(read)?,
            y: type_io::read_f32(read)?,
        };
        let pay_rotation = type_io::read_f32(read)?;
        let payload = self.read_exact_payload_ref(content, read)?;
        Ok(PayloadBlockBuildState {
            payload,
            pay_vector,
            pay_rotation,
            carried: false,
        })
    }

    fn read_exact_payload_conveyor_extra(
        &self,
        content: &ContentLoader,
        read: &mut &[u8],
    ) -> io::Result<(f32, Option<PayloadRef>)> {
        let _progress = type_io::read_f32(read)?;
        let item_rotation = type_io::read_f32(read)?;
        let item = self.read_exact_payload_ref(content, read)?;
        Ok((item_rotation, item))
    }

    fn read_exact_payload_ref(
        &self,
        content: &ContentLoader,
        read: &mut &[u8],
    ) -> io::Result<Option<PayloadRef>> {
        if !type_io::read_bool(read)? {
            return Ok(None);
        }

        let payload_type = type_io::read_u8(read)?;
        match payload_type {
            PAYLOAD_BLOCK_TYPE => {
                let block = type_io::read_i16(read)?;
                let version = type_io::read_u8(read)?;
                let Some(block_def) = content.block(block) else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "payload references unknown block id",
                    ));
                };
                if !block_def.base().has_building() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "block payload references non-building block",
                    ));
                }

                let before_build = *read;
                let mut building_bytes = before_build;
                let mut building = BuildingComp::new(
                    crate::mindustry::world::point2_pack(0, 0),
                    block_def.base().clone(),
                    crate::mindustry::io::TeamId(0),
                );
                building.read_base(&mut building_bytes)?;

                let mut state_bytes = building_bytes;
                match self.read_runtime_state_from_building_payload(
                    content,
                    block_def,
                    &building,
                    version,
                    &mut state_bytes,
                    GameRuntimePayloadReadMode::NestedExact,
                ) {
                    Ok(_) => {}
                    Err(GameRuntimeBlockStateReadError::Unsupported) => {
                        if !Self::block_has_no_java_block_specific_payload(block_def) {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "block payload contains unsupported non-terminal state",
                            ));
                        }
                    }
                    Err(GameRuntimeBlockStateReadError::Parse) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "failed to parse nested block payload state",
                        ));
                    }
                }

                let consumed = before_build.len().saturating_sub(state_bytes.len());
                *read = state_bytes;
                Ok(Some(PayloadRef::Block {
                    block,
                    version,
                    build_bytes: before_build[..consumed].to_vec(),
                }))
            }
            PAYLOAD_UNIT_TYPE => {
                let class_id = type_io::read_u8(read)?;
                let before_unit = *read;
                self.read_exact_unit_payload_body(content, class_id, read)?;
                let consumed = before_unit.len().saturating_sub(read.len());
                Ok(Some(PayloadRef::Unit {
                    class_id,
                    unit_bytes: before_unit[..consumed].to_vec(),
                }))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unknown payload type",
            )),
        }
    }

    fn read_exact_unit_payload_body(
        &self,
        content: &ContentLoader,
        class_id: u8,
        read: &mut &[u8],
    ) -> io::Result<()> {
        let revision = type_io::read_i16(read)?;
        let schema = Self::unit_payload_schema(class_id, revision).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported non-terminal unit payload class/revision",
            )
        })?;

        type_io::skip_abilities(read)?;
        if matches!(schema, GameRuntimeUnitPayloadSchema::Ammo) {
            type_io::read_f32(read)?;
        } else {
            type_io::read_f32(read)?;
            type_io::read_f32(read)?;
        }
        if matches!(schema, GameRuntimeUnitPayloadSchema::BaseRotation) {
            type_io::read_f32(read)?;
        }
        if matches!(schema, GameRuntimeUnitPayloadSchema::BuildingPayloads) {
            type_io::read_building_ref(read)?;
        }
        type_io::read_controller(read)?;
        type_io::read_f32(read)?;
        type_io::read_u64(read)?;
        type_io::read_f32(read)?;
        type_io::read_bool(read)?;
        if matches!(schema, GameRuntimeUnitPayloadSchema::Missile) {
            type_io::read_f32(read)?;
        }
        type_io::read_tile_pos(read)?;
        type_io::skip_mounts(read)?;
        if matches!(
            schema,
            GameRuntimeUnitPayloadSchema::Payloads | GameRuntimeUnitPayloadSchema::BuildingPayloads
        ) {
            self.read_exact_unit_payload_seq(content, read)?;
        }
        type_io::read_plans_queue(read, content)?;
        type_io::read_f32(read)?;
        type_io::read_f32(read)?;
        type_io::read_bool(read)?;
        type_io::read_items(read, content)?;
        type_io::read_statuses(read, content)?;
        type_io::read_team(read)?;
        type_io::read_unit_type(read, content)?;
        if matches!(schema, GameRuntimeUnitPayloadSchema::Missile) {
            type_io::read_f32(read)?;
        }
        type_io::read_bool(read)?;
        type_io::read_vec2(read)?;
        type_io::read_f32(read)?;
        type_io::read_f32(read)?;
        Ok(())
    }

    fn read_exact_unit_payload_seq(
        &self,
        content: &ContentLoader,
        read: &mut &[u8],
    ) -> io::Result<()> {
        let len = type_io::read_i32(read)?;
        if len < 0 || len as usize > type_io::MAX_ARRAY_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid unit payload sequence length",
            ));
        }
        for _ in 0..len {
            self.read_exact_payload_ref(content, read)?;
        }
        Ok(())
    }

    fn unit_payload_schema(class_id: u8, revision: i16) -> Option<GameRuntimeUnitPayloadSchema> {
        match (class_id, revision) {
            (0, 5)
            | (2, 9)
            | (3, 9)
            | (16, 8)
            | (18, 7)
            | (20, 9)
            | (21, 8)
            | (24, 9)
            | (29, 5)
            | (30, 5)
            | (31, 5)
            | (33, 5)
            | (43, 2)
            | (45, 2)
            | (46, 2) => Some(GameRuntimeUnitPayloadSchema::Common),
            (4, 9) | (17, 7) | (19, 5) | (32, 5) => {
                Some(GameRuntimeUnitPayloadSchema::BaseRotation)
            }
            (5, 7) | (23, 8) | (26, 7) => Some(GameRuntimeUnitPayloadSchema::Payloads),
            (36, 3) => Some(GameRuntimeUnitPayloadSchema::BuildingPayloads),
            (39, 3) => Some(GameRuntimeUnitPayloadSchema::Missile),
            (40, 1) | (44, 0) | (47, 1) => Some(GameRuntimeUnitPayloadSchema::Ammo),
            _ => None,
        }
    }

    fn block_has_no_java_block_specific_payload(block: &BlockDef) -> bool {
        match block {
            BlockDef::Plain(block) => !is_construct_block_name(&block.name),
            BlockDef::Production(production) => matches!(
                production.kind,
                ProductionBlockKind::SolidPump
                    | ProductionBlockKind::Fracker
                    | ProductionBlockKind::WallCrafter
            ),
            BlockDef::Storage(storage) => matches!(storage.kind, StorageBlockKind::Storage),
            BlockDef::Crafting(crafting) => matches!(
                crafting.kind,
                CraftingBlockKind::HeatConductor
                    | CraftingBlockKind::Incinerator
                    | CraftingBlockKind::ItemIncinerator
            ),
            BlockDef::DefenseWall(wall) => {
                matches!(wall.kind, DefenseWallKind::Wall | DefenseWallKind::Thruster)
            }
            BlockDef::Effect(effect) => matches!(
                effect.kind,
                EffectBlockKind::ShockMine
                    | EffectBlockKind::RegenProjector
                    | EffectBlockKind::ShockwaveTower
            ),
            BlockDef::Distribution(distribution) => {
                matches!(distribution.kind, DistributionBlockKind::Router)
            }
            BlockDef::Liquid(liquid) => matches!(
                liquid.kind,
                LiquidBlockKind::Pump
                    | LiquidBlockKind::Conduit
                    | LiquidBlockKind::ArmoredConduit
                    | LiquidBlockKind::LiquidRouter
                    | LiquidBlockKind::LiquidJunction
            ),
            BlockDef::Power(power) => matches!(
                power.kind,
                PowerBlockKind::PowerNode
                    | PowerBlockKind::PowerDiode
                    | PowerBlockKind::Battery
                    | PowerBlockKind::BeamNode
                    | PowerBlockKind::LongPowerNode
            ),
            BlockDef::Sandbox(sandbox) => matches!(
                sandbox.kind,
                SandboxBlockKind::PowerSource
                    | SandboxBlockKind::PowerVoid
                    | SandboxBlockKind::ItemVoid
                    | SandboxBlockKind::LiquidVoid
            ),
            _ => false,
        }
    }

    fn read_payload_runtime_state_from_building_payload(
        &self,
        content: &ContentLoader,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
        payload_read_mode: GameRuntimePayloadReadMode,
    ) -> Result<Option<GameRuntimePayloadBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        match block {
            BlockDef::Payload(payload) => match payload.kind {
                PayloadBlockKind::PayloadConveyor => {
                    let (item_rotation, item) =
                        if payload_read_mode == GameRuntimePayloadReadMode::TopLevel {
                            let (_progress, item_rotation, item) =
                                read_terminal_payload_conveyor_extra(building_payload)
                                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                            (item_rotation, item)
                        } else {
                            self.read_exact_payload_conveyor_extra(content, building_payload)
                                .map_err(|_| GameRuntimeBlockStateReadError::Parse)?
                        };
                    let conveyor = PayloadConveyorState {
                        item,
                        item_rotation,
                        ..PayloadConveyorState::default()
                    };
                    Ok(Some(GameRuntimePayloadBlockState::Conveyor(conveyor)))
                }
                PayloadBlockKind::PayloadRouter => {
                    let (item_rotation, item) = self
                        .read_exact_payload_conveyor_extra(content, building_payload)
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                    let conveyor = PayloadConveyorState {
                        item,
                        item_rotation,
                        ..PayloadConveyorState::default()
                    };
                    let (sorted, rec_dir) = read_payload_router_extra(building_payload, revision)
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                    Ok(Some(GameRuntimePayloadBlockState::Router {
                        conveyor,
                        sorted,
                        rec_dir,
                    }))
                }
            },
            BlockDef::PayloadMassDriver(_) => {
                let common = self
                    .read_exact_payload_block_build_common(content, building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let driver = read_payload_mass_driver_extra(building_payload, revision)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                Ok(Some(GameRuntimePayloadBlockState::MassDriver {
                    common,
                    driver,
                }))
            }
            BlockDef::PayloadLoader(_) => {
                let common = self
                    .read_exact_payload_block_build_common(content, building_payload)
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
                let common = self
                    .read_exact_payload_block_build_common(content, building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let (progress, accum) = read_deconstructor_extra(building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let deconstructing = if payload_read_mode == GameRuntimePayloadReadMode::TopLevel {
                    read_payload_ref_to_end(building_payload)
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)?
                } else {
                    self.read_exact_payload_ref(content, building_payload)
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)?
                };
                let deconstructor = PayloadDeconstructorState {
                    progress,
                    accum,
                    has_payload: common.payload.is_some(),
                    has_deconstructing: deconstructing.is_some(),
                    deconstructing,
                };
                Ok(Some(GameRuntimePayloadBlockState::Deconstructor {
                    common,
                    deconstructor,
                }))
            }
            BlockDef::PayloadConstructor(_) => {
                let common = self
                    .read_exact_payload_block_build_common(content, building_payload)
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
                let common = self
                    .read_exact_payload_block_build_common(content, building_payload)
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
            BlockDef::Sandbox(sandbox) if sandbox.kind == SandboxBlockKind::PayloadVoid => {
                if payload_read_mode == GameRuntimePayloadReadMode::TopLevel {
                    read_terminal_payload_block_build_common(building_payload)
                        .map(|common| Some(GameRuntimePayloadBlockState::Void(common)))
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)
                } else {
                    self.read_exact_payload_block_build_common(content, building_payload)
                        .map(|common| Some(GameRuntimePayloadBlockState::Void(common)))
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)
                }
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

    fn read_production_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeProductionBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::Production(production) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };

        match production.kind {
            ProductionBlockKind::Drill => read_drill_state(building_payload, revision)
                .map(|state| Some(GameRuntimeProductionBlockState::Drill(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            ProductionBlockKind::BeamDrill => read_beam_drill_state(building_payload, revision)
                .map(|state| Some(GameRuntimeProductionBlockState::BeamDrill(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            ProductionBlockKind::BurstDrill => read_burst_drill_state(building_payload, revision)
                .map(|state| Some(GameRuntimeProductionBlockState::BurstDrill(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            ProductionBlockKind::SolidPump
            | ProductionBlockKind::Fracker
            | ProductionBlockKind::WallCrafter => Err(GameRuntimeBlockStateReadError::Unsupported),
        }
    }

    fn read_crafting_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeCraftingBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::Crafting(crafting) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };

        match crafting.kind {
            CraftingBlockKind::GenericCrafter
            | CraftingBlockKind::AttributeCrafter
            | CraftingBlockKind::HeatCrafter => {
                read_generic_crafter_state(building_payload, crafting.legacy_read_warmup)
                    .map(|state| Some(GameRuntimeCraftingBlockState::GenericCrafter(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            CraftingBlockKind::Separator => read_separator_state(building_payload, revision)
                .map(|state| Some(GameRuntimeCraftingBlockState::Separator(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            CraftingBlockKind::HeatProducer => {
                let crafter =
                    read_generic_crafter_state(building_payload, crafting.legacy_read_warmup)
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let heat = read_heat_producer_state(building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                Ok(Some(GameRuntimeCraftingBlockState::HeatProducer {
                    crafter,
                    heat,
                }))
            }
            CraftingBlockKind::HeatConductor
            | CraftingBlockKind::Incinerator
            | CraftingBlockKind::ItemIncinerator => {
                Err(GameRuntimeBlockStateReadError::Unsupported)
            }
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
            DistributionBlockKind::DuctRouter
            | DistributionBlockKind::OverflowDuct
            | DistributionBlockKind::StackRouter => {
                read_duct_router_state(building_payload, revision, current_item)
                    .map(|state| Some(GameRuntimeDistributionBlockState::DuctRouter(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            DistributionBlockKind::Junction => read_duct_junction_state(building_payload)
                .map(|state| Some(GameRuntimeDistributionBlockState::DuctJunction(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            DistributionBlockKind::Sorter => read_sorter_state(building_payload, revision)
                .map(|state| Some(GameRuntimeDistributionBlockState::Sorter(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            DistributionBlockKind::OverflowGate => {
                read_overflow_gate_legacy_payload(building_payload, revision)
                    .map(|_| None)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            DistributionBlockKind::Unloader => {
                read_unloader_sort_item(building_payload, revision as i32)
                    .map(|sort_item| {
                        Some(GameRuntimeDistributionBlockState::Unloader(
                            sort_item.map(|id| id as ContentId),
                        ))
                    })
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
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

    fn read_logic_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeLogicBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::Logic(logic) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };

        match logic.kind {
            LogicBlockKind::Message => read_message_state(building_payload)
                .map(|state| Some(GameRuntimeLogicBlockState::Message(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            LogicBlockKind::Switch => read_switch_enabled(building_payload, revision, false)
                .map(|enabled| Some(GameRuntimeLogicBlockState::Switch { enabled }))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            LogicBlockKind::Display | LogicBlockKind::TileDisplay => {
                read_logic_display_state(building_payload, revision)
                    .map(|state| Some(GameRuntimeLogicBlockState::Display(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            LogicBlockKind::Memory => {
                read_memory_state(building_payload, logic.memory_capacity.max(0) as usize)
                    .map(|state| Some(GameRuntimeLogicBlockState::Memory(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            LogicBlockKind::Canvas => {
                read_canvas_state(building_payload, logic.canvas_data_bytes.max(0) as usize)
                    .map(|state| Some(GameRuntimeLogicBlockState::Canvas(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            LogicBlockKind::Processor => read_logic_processor_state(
                building_payload,
                revision,
                logic.privileged_only,
                logic.max_instructions_per_tick.max(1) as i16,
            )
            .map(|state| Some(GameRuntimeLogicBlockState::Processor(state)))
            .map_err(|_| GameRuntimeBlockStateReadError::Parse),
        }
    }

    fn read_campaign_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeCampaignBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::Campaign(campaign) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };

        match campaign.kind {
            CampaignBlockKind::LaunchPad | CampaignBlockKind::AdvancedLaunchPad => {
                read_launch_pad_state(building_payload, revision)
                    .map(|state| Some(GameRuntimeCampaignBlockState::LaunchPad(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            CampaignBlockKind::LandingPad => read_landing_pad_state(building_payload, revision)
                .map(|state| Some(GameRuntimeCampaignBlockState::LandingPad(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            CampaignBlockKind::Accelerator => read_accelerator_state(building_payload, revision)
                .map(|state| Some(GameRuntimeCampaignBlockState::Accelerator(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
        }
    }

    fn read_sandbox_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeSandboxBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::Sandbox(sandbox) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };

        match sandbox.kind {
            SandboxBlockKind::ItemSource => read_item_source_config(building_payload)
                .map(|output_item| {
                    Some(GameRuntimeSandboxBlockState::ItemSource(ItemSourceState {
                        output_item,
                        ..ItemSourceState::default()
                    }))
                })
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            SandboxBlockKind::LiquidSource => read_liquid_source_config(building_payload, revision)
                .map(|source| {
                    Some(GameRuntimeSandboxBlockState::LiquidSource(
                        LiquidSourceState {
                            source,
                            ..LiquidSourceState::default()
                        },
                    ))
                })
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            SandboxBlockKind::PowerSource
            | SandboxBlockKind::PowerVoid
            | SandboxBlockKind::ItemVoid
            | SandboxBlockKind::LiquidVoid
            | SandboxBlockKind::PayloadSource
            | SandboxBlockKind::PayloadVoid => Err(GameRuntimeBlockStateReadError::Unsupported),
        }
    }

    fn read_legacy_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeLegacyBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::Legacy(legacy) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };

        match legacy.kind {
            LegacyBlockKind::CommandCenter => read_legacy_command_center_extra(building_payload)
                .map(|value| Some(GameRuntimeLegacyBlockState::CommandCenter(value)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            LegacyBlockKind::MechPad => read_legacy_mech_pad_extra(building_payload)
                .map(|values| Some(GameRuntimeLegacyBlockState::MechPad(values)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            LegacyBlockKind::UnitFactory => {
                read_legacy_unit_factory_extra(building_payload, revision)
                    .map(|extra| Some(GameRuntimeLegacyBlockState::UnitFactory(extra)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
        }
    }

    fn read_unit_runtime_state_from_building_payload(
        &self,
        content: &ContentLoader,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
        payload_read_mode: GameRuntimePayloadReadMode,
    ) -> Result<Option<GameRuntimeUnitBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        match block {
            BlockDef::UnitFactory(_) => {
                let common = self
                    .read_exact_payload_block_build_common(content, building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                read_unit_factory_state(building_payload, revision as i32)
                    .map(|factory| Some(GameRuntimeUnitBlockState::Factory { common, factory }))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            BlockDef::UnitReconstructor(_) => {
                let common = self
                    .read_exact_payload_block_build_common(content, building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                read_reconstructor_state(building_payload, revision as i32)
                    .map(|reconstructor| {
                        Some(GameRuntimeUnitBlockState::Reconstructor {
                            common,
                            reconstructor,
                        })
                    })
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            BlockDef::UnitRepairTower(_) => {
                read_repair_turret_state(building_payload, revision as i32)
                    .map(|state| Some(GameRuntimeUnitBlockState::RepairTower(state)))
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            BlockDef::UnitAssembler(_) => {
                let common = self
                    .read_exact_payload_block_build_common(content, building_payload)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                read_unit_assembler_state(building_payload, revision as i32)
                    .map(|assembler| {
                        Some(GameRuntimeUnitBlockState::Assembler { common, assembler })
                    })
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)
            }
            BlockDef::UnitAssemblerModule(_) => {
                if payload_read_mode == GameRuntimePayloadReadMode::TopLevel {
                    read_terminal_payload_block_build_common(building_payload)
                        .map(|common| Some(GameRuntimeUnitBlockState::AssemblerModule(common)))
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)
                } else {
                    self.read_exact_payload_block_build_common(content, building_payload)
                        .map(|common| Some(GameRuntimeUnitBlockState::AssemblerModule(common)))
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)
                }
            }
            _ => Err(GameRuntimeBlockStateReadError::Unsupported),
        }
    }

    fn read_turret_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        revision: u8,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeTurretBlockState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::Turret(turret) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };

        match turret.kind {
            TurretBlockKind::ItemTurret => {
                let mut turret_state = turret_read_child(building_payload, revision)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let (ammo, total_ammo) =
                    item_turret_read_ammo(building_payload, revision, turret.max_ammo, |item_id| {
                        turret.ammo.iter().any(|ammo| ammo.item == item_id)
                    })
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                turret_state.total_ammo = total_ammo;
                Ok(Some(GameRuntimeTurretBlockState::Item {
                    turret: turret_state,
                    ammo,
                }))
            }
            TurretBlockKind::PayloadAmmoTurret => {
                let mut turret_state = turret_read_child(building_payload, revision)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let payloads = payload_ammo_turret_read_payloads(building_payload, |key| {
                    turret.payload_ammo.iter().any(|ammo| ammo.content == key)
                })
                .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                turret_state.total_ammo = payloads.total();
                Ok(Some(GameRuntimeTurretBlockState::PayloadAmmo {
                    turret: turret_state,
                    payloads,
                }))
            }
            TurretBlockKind::ContinuousTurret | TurretBlockKind::ContinuousLiquidTurret => {
                let turret_state = turret_read_child(building_payload, revision)
                    .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                let continuous =
                    continuous_turret_read_child(building_payload, revision, turret.base.size)
                        .map_err(|_| GameRuntimeBlockStateReadError::Parse)?;
                Ok(Some(GameRuntimeTurretBlockState::Continuous {
                    turret: turret_state,
                    continuous,
                }))
            }
            TurretBlockKind::PointDefenseTurret => point_defense_read_child(building_payload)
                .map(|state| Some(GameRuntimeTurretBlockState::PointDefense(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            TurretBlockKind::TractorBeamTurret => tractor_beam_read_child(building_payload)
                .map(|state| Some(GameRuntimeTurretBlockState::TractorBeam(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            TurretBlockKind::LiquidTurret
            | TurretBlockKind::PowerTurret
            | TurretBlockKind::LaserTurret => turret_read_child(building_payload, revision)
                .map(|state| Some(GameRuntimeTurretBlockState::Generic(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
        }
    }

    fn read_defense_wall_runtime_state_from_building_payload(
        &self,
        block: &BlockDef,
        building_payload: &mut &[u8],
    ) -> Result<Option<GameRuntimeDefenseWallState>, GameRuntimeBlockStateReadError> {
        if building_payload.is_empty() {
            return Ok(None);
        }

        let BlockDef::DefenseWall(wall) = block else {
            return Err(GameRuntimeBlockStateReadError::Unsupported);
        };

        match wall.kind {
            DefenseWallKind::Door => read_door_state(building_payload)
                .map(|state| Some(GameRuntimeDefenseWallState::Door(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            DefenseWallKind::AutoDoor => read_auto_door_state(building_payload)
                .map(|state| Some(GameRuntimeDefenseWallState::Door(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
            DefenseWallKind::ShieldWall => read_shield_wall_state(building_payload)
                .map(|state| Some(GameRuntimeDefenseWallState::ShieldWall(state)))
                .map_err(|_| GameRuntimeBlockStateReadError::Parse),
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
        self.production_runtime_states.clear();
        self.crafting_runtime_states.clear();
        self.distribution_runtime_states.clear();
        self.storage_runtime_states.clear();
        self.liquid_runtime_states.clear();
        self.logic_runtime_states.clear();
        self.campaign_runtime_states.clear();
        self.sandbox_runtime_states.clear();
        self.legacy_runtime_states.clear();
        self.unit_runtime_states.clear();
        self.defense_wall_runtime_states.clear();
        self.turret_runtime_states.clear();
        self.construct_runtime_states.clear();
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

    pub fn advance_owned_payload_constructors(
        &mut self,
        content: &ContentLoader,
        delta_seconds: f32,
    ) -> Option<GameRuntimePayloadConstructorFrameReport> {
        self.advance_owned_payload_constructors_with_recipe_build_time(
            content,
            delta_seconds,
            |block| Some(block.effective_build_time(content.items())),
        )
    }

    pub fn advance_owned_payload_constructors_with_recipe_build_time(
        &mut self,
        content: &ContentLoader,
        delta_seconds: f32,
        mut recipe_build_time: impl FnMut(&BlockDef) -> Option<f32>,
    ) -> Option<GameRuntimePayloadConstructorFrameReport> {
        self.consume_world_load_events_and_reset_sidecars();

        let advanced = self.state.advance_game_update_frame(delta_seconds);
        if !advanced.advanced {
            return None;
        }

        self.refresh_owned_building_update_permissions(content);

        let frame_delta = advanced.delta_ticks as f32;
        let mut report = GameRuntimePayloadConstructorFrameReport::default();

        for index in 0..self.buildings.len() {
            let (tile_pos, block_id, team, enabled, efficiency, time_scale) = {
                let building = &mut self.buildings[index];
                let can_overdrive = content
                    .block(building.block.id)
                    .map(BlockDef::can_overdrive)
                    .unwrap_or(false);
                building.advance_update_timing(frame_delta, can_overdrive);
                report.visited_buildings += 1;
                (
                    building.tile_pos,
                    building.block.id,
                    building.team,
                    building.enabled,
                    building.efficiency,
                    building.time_scale,
                )
            };

            if !enabled {
                continue;
            }

            let Some(BlockDef::PayloadConstructor(constructor)) = content.block(block_id) else {
                continue;
            };
            report.constructor_candidates += 1;

            let Some(GameRuntimePayloadBlockState::Constructor {
                common,
                producer,
                recipe,
            }) = self.payload_runtime_states.get_mut(&tile_pos)
            else {
                report.missing_runtime_states += 1;
                continue;
            };

            producer.has_payload = common.payload.is_some();
            let recipe_def = recipe.and_then(|recipe_id| content.block(recipe_id));
            let recipe_build_time = recipe_def.and_then(|block| {
                recipe_build_time(block)
                    .filter(|build_time| build_time.is_finite() && *build_time > 0.0)
            });

            if recipe.is_some() && recipe_build_time.is_none() {
                report.missing_recipe_build_times += 1;
            }

            let before_had_payload = producer.has_payload;
            let step = block_producer_update(
                producer,
                recipe_build_time,
                efficiency,
                constructor.build_speed,
                frame_delta * time_scale * efficiency,
                frame_delta * time_scale,
            );
            report.updated_constructors += 1;

            if step.produced && !before_had_payload {
                if let Some(recipe_def) = recipe_def {
                    let payload_building = BuildingComp::new(
                        crate::mindustry::world::point2_pack(0, 0),
                        recipe_def.base().clone(),
                        team,
                    );
                    let mut build_bytes = Vec::new();
                    if payload_building.write_base(&mut build_bytes, false).is_ok() {
                        common.payload = Some(PayloadRef::Block {
                            block: recipe_def.base().id,
                            version: 0,
                            build_bytes,
                        });
                        common.pay_vector = PayloadVec2 { x: 0.0, y: 0.0 };
                        common.pay_rotation = 0.0;
                        producer.has_payload = true;
                        report.produced_payloads += 1;
                    }
                }
            }
        }

        Some(report)
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
        content::blocks::{BulletKind, BulletSpec, PayloadTurretAmmo, TurretBlockData},
        core::GameStateState,
        ctype::{Content, ContentType},
        entities::units::BuildPlan,
        io::{
            LegacyMapBlockRecord, LegacyMapFloorRecord, LegacyShortChunkMap, TeamId, TypeValue,
            Vec2 as IoVec2,
        },
        r#type::{PayloadKey, PayloadSeq},
        world::{
            blocks::campaign::{
                write_accelerator_state, write_landing_pad_state, write_launch_pad_state,
            },
            blocks::defense::turrets::{
                continuous_turret_write_child, item_turret_write_ammo,
                payload_ammo_turret_write_payloads, point_defense_write_child,
                tractor_beam_write_child, turret_write_child, ContinuousTurretState, ItemAmmoEntry,
                PointDefenseState, TractorBeamState, TurretState,
            },
            blocks::defense::{
                build_turret_write_child_with_loader, write_auto_door_state,
                write_base_shield_state, write_door_state, write_force_projector_state,
                write_radar_state, write_shield_wall_state, BaseShieldState, BuildTurretState,
                DoorState, EffectBlockRuntimeState, EffectProjectorRuntimeState,
                ForceProjectorState, MendProjectorState, OverdriveProjectorState, RadarState,
                ShieldWallState,
            },
            blocks::distribution::{
                write_conveyor_state, write_directional_unloader_state, write_duct_router_state,
                write_item_bridge_state, write_mass_driver_state, write_sorter_state,
                ConveyorItemState, ConveyorState, DirectionalUnloaderState, DuctRouterState,
                ItemBridgeState, MassDriverState, MassDriverStateKind, SorterState,
            },
            blocks::heat::write_heat_producer_state,
            blocks::legacy::{
                write_legacy_command_center_extra, write_legacy_mech_pad_extra,
                write_legacy_unit_factory_extra,
            },
            blocks::liquid::{write_liquid_bridge_state, LiquidBridgeState},
            blocks::logic::{
                write_canvas_state, write_logic_display_state, write_logic_processor_state,
                write_memory_state, write_message_state, write_switch_enabled, LogicConfig,
                LogicLink, LogicProcessorVariableState, LogicProcessorWaitState,
            },
            blocks::payloads::{
                write_block_producer_progress, write_constructor_recipe, write_deconstructor_extra,
                write_payload_block_build_common, write_payload_conveyor_extra,
                write_payload_loader_extra, write_payload_mass_driver_extra, write_payload_ref,
                write_payload_router_extra, write_payload_source_extra, BlockProducerState,
                PayloadBlockBuildState, PayloadConveyorState, PayloadDeconstructorState,
                PayloadDriverState, PayloadLoaderState, PayloadMassDriverState, PayloadRef,
                PayloadSortKey, PayloadSourceState, Vec2,
            },
            blocks::power::{
                write_heater_generator_state, write_impact_reactor_state, write_light_block_state,
                write_nuclear_reactor_state, write_power_generator_state,
                write_variable_reactor_state, HeaterGeneratorState, ImpactReactorState,
                LightBlockState, NuclearReactorState, PowerGeneratorState, VariableReactorState,
            },
            blocks::production::{
                write_beam_drill_state, write_burst_drill_state, write_drill_state,
                write_generic_crafter_state, write_separator_state,
            },
            blocks::sandbox::{write_item_source_config, write_liquid_source_config},
            blocks::storage::{write_core_state, write_unloader_sort_item, CoreBuildState},
            blocks::units::{
                write_reconstructor_state, write_repair_turret_state, write_unit_assembler_state,
                write_unit_cargo_loader_state, write_unit_cargo_unload_state,
                write_unit_factory_state, RepairTurretState, UnitAssemblerState,
                UnitCargoLoaderState, UnitCargoUnloadPointState, UnitFactoryState,
            },
            blocks::{write_construct_block_state, ConstructAccumulatorEntry, ConstructBlockState},
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

    fn door_build_payload_ref(content: &ContentLoader, open: bool) -> PayloadRef {
        let door_def = content.block_by_name("door").unwrap();
        let door = BuildingComp::new(point2_pack(0, 0), door_def.base().clone(), TeamId(1));
        let mut build_bytes = Vec::new();
        door.write_base(&mut build_bytes, false).unwrap();
        write_door_state(&mut build_bytes, DoorState { open }).unwrap();
        PayloadRef::Block {
            block: door_def.base().id,
            version: 0,
            build_bytes,
        }
    }

    fn base_only_build_payload_ref(content: &ContentLoader, block_name: &str) -> PayloadRef {
        let block_def = content.block_by_name(block_name).unwrap();
        let building = BuildingComp::new(point2_pack(0, 0), block_def.base().clone(), TeamId(1));
        let mut build_bytes = Vec::new();
        building.write_base(&mut build_bytes, false).unwrap();
        PayloadRef::Block {
            block: block_def.base().id,
            version: 0,
            build_bytes,
        }
    }

    fn flare_unit_payload_ref(content: &ContentLoader) -> PayloadRef {
        let flare = content.unit_by_name("flare").unwrap();
        let mut unit_bytes = Vec::new();
        type_io::write_i16(&mut unit_bytes, 9).unwrap();
        type_io::write_u8(&mut unit_bytes, 0).unwrap();
        type_io::write_f32(&mut unit_bytes, 8.0).unwrap();
        type_io::write_f32(&mut unit_bytes, -6.0).unwrap();
        type_io::write_u8(&mut unit_bytes, 2).unwrap();
        type_io::write_f32(&mut unit_bytes, 0.0).unwrap();
        type_io::write_u64(&mut unit_bytes, 0.0f64.to_bits()).unwrap();
        type_io::write_f32(&mut unit_bytes, 120.0).unwrap();
        type_io::write_bool(&mut unit_bytes, false).unwrap();
        type_io::write_tile_pos(&mut unit_bytes, None).unwrap();
        type_io::write_u8(&mut unit_bytes, 0).unwrap();
        type_io::write_i32(&mut unit_bytes, 0).unwrap();
        type_io::write_f32(&mut unit_bytes, 135.0).unwrap();
        type_io::write_f32(&mut unit_bytes, 2.0).unwrap();
        type_io::write_bool(&mut unit_bytes, false).unwrap();
        type_io::write_i16(&mut unit_bytes, -1).unwrap();
        type_io::write_i32(&mut unit_bytes, 0).unwrap();
        type_io::write_i32(&mut unit_bytes, 0).unwrap();
        type_io::write_team(&mut unit_bytes, Some(TeamId(1))).unwrap();
        type_io::write_i16(&mut unit_bytes, flare.id()).unwrap();
        type_io::write_bool(&mut unit_bytes, false).unwrap();
        type_io::write_vec2(&mut unit_bytes, IoVec2 { x: 0.25, y: -0.5 }).unwrap();
        type_io::write_f32(&mut unit_bytes, 64.0).unwrap();
        type_io::write_f32(&mut unit_bytes, 96.0).unwrap();
        PayloadRef::Unit {
            class_id: 3,
            unit_bytes,
        }
    }

    fn payload_conveyor_build_payload_ref(
        content: &ContentLoader,
        item: &PayloadRef,
    ) -> PayloadRef {
        let conveyor_def = content.block_by_name("payload-conveyor").unwrap();
        let conveyor = BuildingComp::new(point2_pack(0, 0), conveyor_def.base().clone(), TeamId(1));
        let mut build_bytes = Vec::new();
        conveyor.write_base(&mut build_bytes, false).unwrap();
        write_payload_conveyor_extra(&mut build_bytes, 5.0, 45.0, Some(item)).unwrap();
        PayloadRef::Block {
            block: conveyor_def.base().id,
            version: 0,
            build_bytes,
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
    fn game_runtime_exports_network_map_snapshot_with_owned_building_chunks() {
        let content = ContentLoader::create_base_content().unwrap();
        let mend_def = content.block_by_name("mend-projector").unwrap();
        let tile_pos = point2_pack(5, 5);
        let mut saved = BuildingComp::new(tile_pos, mend_def.base().clone(), TeamId(3));
        saved.set_rotation(1);
        saved.health = 33.0;
        let footprint_tiles = (saved.block.size * saved.block.size) as usize;
        assert!(footprint_tiles > 1);

        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(16, 16);
        runtime.add_building(saved);

        let map = runtime.export_network_map_snapshot(&content);
        let center_index = 5 + 5 * 16;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("building center should be exported as an explicit record");
        assert_eq!(center.block_id, mend_def.base().id);
        assert!(center.has_entity);
        assert!(center.is_center);
        assert!(center
            .building
            .as_ref()
            .is_some_and(|payload| payload.len() > 1));
        assert_eq!(center.consecutives, 0);

        let entity_records = map
            .blocks
            .iter()
            .filter(|record| record.has_entity)
            .collect::<Vec<_>>();
        assert_eq!(entity_records.len(), footprint_tiles);
        assert_eq!(
            entity_records
                .iter()
                .filter(|record| record.is_center)
                .count(),
            1
        );
        assert!(entity_records
            .iter()
            .filter(|record| !record.is_center)
            .all(|record| record.building.is_none()));

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(&content, &map);
        assert_eq!(report.building_records, 1);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.building_parse_errors, 0);

        let loaded_building = &loaded.buildings()[0];
        assert_eq!(loaded_building.tile_pos, tile_pos);
        assert_eq!(loaded_building.team, TeamId(3));
        assert_eq!(loaded_building.rotation, 1);
        assert_eq!(loaded_building.health, 33.0);
        assert_eq!(loaded_building.block.id, mend_def.base().id);
    }

    fn roundtrip_exported_defense_wall_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimeDefenseWallState,
    ) -> Option<GameRuntimeDefenseWallState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(16, 16);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(4),
        ));
        runtime
            .defense_wall_runtime_states
            .insert(tile_pos, state.clone());

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 16;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("defense wall center should be exported explicitly");
        assert!(center
            .building
            .as_ref()
            .is_some_and(|payload| payload.len() > 1));

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        loaded.defense_wall_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_defense_wall_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();

        assert_eq!(
            roundtrip_exported_defense_wall_state(
                &content,
                "door",
                2,
                2,
                GameRuntimeDefenseWallState::Door(DoorState { open: true }),
            ),
            Some(GameRuntimeDefenseWallState::Door(DoorState { open: true }))
        );
        assert_eq!(
            roundtrip_exported_defense_wall_state(
                &content,
                "blast-door",
                5,
                5,
                GameRuntimeDefenseWallState::Door(DoorState { open: true }),
            ),
            Some(GameRuntimeDefenseWallState::Door(DoorState { open: true }))
        );
        assert_eq!(
            roundtrip_exported_defense_wall_state(
                &content,
                "shielded-wall",
                9,
                9,
                GameRuntimeDefenseWallState::ShieldWall(ShieldWallState {
                    shield: 44.0,
                    shield_radius: 3.0,
                    break_timer: 2.0,
                    hit: 1.0,
                }),
            ),
            Some(GameRuntimeDefenseWallState::ShieldWall(ShieldWallState {
                shield: 44.0,
                shield_radius: 1.0,
                break_timer: 0.0,
                hit: 0.0,
            }))
        );
    }

    fn exported_power_state_revision(state: &GameRuntimePowerBlockState) -> u8 {
        match state {
            GameRuntimePowerBlockState::Generator(_)
            | GameRuntimePowerBlockState::NuclearReactor(_)
            | GameRuntimePowerBlockState::ImpactReactor(_)
            | GameRuntimePowerBlockState::VariableReactor(_)
            | GameRuntimePowerBlockState::HeaterGenerator(_) => 1,
            GameRuntimePowerBlockState::Light(_) => 0,
        }
    }

    fn roundtrip_exported_power_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimePowerBlockState,
    ) -> Option<GameRuntimePowerBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let expected_revision = exported_power_state_revision(&state);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(16, 16);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(6),
        ));
        runtime.power_runtime_states.insert(tile_pos, state.clone());

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 16;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("power/light center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("power/light center should carry building payload");
        assert_eq!(payload.first().copied(), Some(expected_revision));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.power_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_power_and_light_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();

        assert_eq!(
            roundtrip_exported_power_state(
                &content,
                "thermal-generator",
                1,
                6,
                GameRuntimePowerBlockState::Generator(PowerGeneratorState {
                    production_efficiency: 0.75,
                    generate_time: 4.0,
                }),
            ),
            Some(GameRuntimePowerBlockState::Generator(PowerGeneratorState {
                production_efficiency: 0.75,
                generate_time: 4.0,
            }))
        );
        assert_eq!(
            roundtrip_exported_power_state(
                &content,
                "thorium-reactor",
                3,
                6,
                GameRuntimePowerBlockState::NuclearReactor(NuclearReactorState {
                    generator: PowerGeneratorState {
                        production_efficiency: 0.5,
                        generate_time: 2.0,
                    },
                    heat: 0.8,
                }),
            ),
            Some(GameRuntimePowerBlockState::NuclearReactor(
                NuclearReactorState {
                    generator: PowerGeneratorState {
                        production_efficiency: 0.5,
                        generate_time: 2.0,
                    },
                    heat: 0.8,
                }
            ))
        );
        assert_eq!(
            roundtrip_exported_power_state(
                &content,
                "impact-reactor",
                5,
                6,
                GameRuntimePowerBlockState::ImpactReactor(ImpactReactorState {
                    generator: PowerGeneratorState {
                        production_efficiency: 0.9,
                        generate_time: 1.5,
                    },
                    warmup: 0.6,
                }),
            ),
            Some(GameRuntimePowerBlockState::ImpactReactor(
                ImpactReactorState {
                    generator: PowerGeneratorState {
                        production_efficiency: 0.9,
                        generate_time: 1.5,
                    },
                    warmup: 0.6,
                }
            ))
        );
        assert_eq!(
            roundtrip_exported_power_state(
                &content,
                "flux-reactor",
                8,
                6,
                GameRuntimePowerBlockState::VariableReactor(VariableReactorState {
                    generator: PowerGeneratorState {
                        production_efficiency: 0.4,
                        generate_time: 3.0,
                    },
                    heat: 7.5,
                    instability: 0.25,
                    warmup: 0.5,
                }),
            ),
            Some(GameRuntimePowerBlockState::VariableReactor(
                VariableReactorState {
                    generator: PowerGeneratorState {
                        production_efficiency: 0.4,
                        generate_time: 3.0,
                    },
                    heat: 7.5,
                    instability: 0.25,
                    warmup: 0.5,
                }
            ))
        );
        assert_eq!(
            roundtrip_exported_power_state(
                &content,
                "neoplasia-reactor",
                11,
                6,
                GameRuntimePowerBlockState::HeaterGenerator(HeaterGeneratorState {
                    generator: PowerGeneratorState {
                        production_efficiency: 0.3,
                        generate_time: 2.25,
                    },
                    heat: 12.0,
                }),
            ),
            Some(GameRuntimePowerBlockState::HeaterGenerator(
                HeaterGeneratorState {
                    generator: PowerGeneratorState {
                        production_efficiency: 0.3,
                        generate_time: 2.25,
                    },
                    heat: 12.0,
                }
            ))
        );
        assert_eq!(
            roundtrip_exported_power_state(
                &content,
                "illuminator",
                14,
                6,
                GameRuntimePowerBlockState::Light(LightBlockState { color: 0x12_34_56 }),
            ),
            Some(GameRuntimePowerBlockState::Light(LightBlockState {
                color: 0x12_34_56
            }))
        );
    }

    fn exported_effect_state_revision(state: &EffectBlockRuntimeState) -> u8 {
        match state {
            EffectBlockRuntimeState::BaseShield(_) => 1,
            _ => 0,
        }
    }

    fn roundtrip_exported_effect_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: EffectBlockRuntimeState,
    ) -> Option<EffectBlockRuntimeState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let expected_revision = exported_effect_state_revision(&state);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(16, 16);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(5),
        ));
        let building_snapshot = runtime.buildings()[0].clone();
        runtime
            .effect_runtime_store
            .ensure_for_building(content, &building_snapshot, 0.0);
        *runtime.effect_runtime_store.get_mut(tile_pos).unwrap() = state.clone();

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 16;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("effect block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("effect block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(expected_revision));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.effect_runtime_store.get(tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_effect_block_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();

        assert_eq!(
            roundtrip_exported_effect_state(
                &content,
                "mend-projector",
                1,
                8,
                EffectBlockRuntimeState::Projector(EffectProjectorRuntimeState::Mend(
                    MendProjectorState {
                        heat: 1.25,
                        charge: 9.0,
                        phase_heat: 0.5,
                        smooth_efficiency: 2.0,
                    },
                )),
            ),
            Some(EffectBlockRuntimeState::Projector(
                EffectProjectorRuntimeState::Mend(MendProjectorState {
                    heat: 1.25,
                    phase_heat: 0.5,
                    ..MendProjectorState::default()
                })
            ))
        );
        assert_eq!(
            roundtrip_exported_effect_state(
                &content,
                "overdrive-projector",
                3,
                8,
                EffectBlockRuntimeState::Projector(EffectProjectorRuntimeState::Overdrive(
                    OverdriveProjectorState {
                        heat: 0.75,
                        charge: 4.0,
                        phase_heat: 0.35,
                        smooth_efficiency: 1.5,
                        use_progress: 0.25,
                    },
                )),
            ),
            Some(EffectBlockRuntimeState::Projector(
                EffectProjectorRuntimeState::Overdrive(OverdriveProjectorState {
                    heat: 0.75,
                    phase_heat: 0.35,
                    ..OverdriveProjectorState::default()
                })
            ))
        );
        assert_eq!(
            roundtrip_exported_effect_state(
                &content,
                "force-projector",
                5,
                8,
                EffectBlockRuntimeState::ForceProjector(ForceProjectorState {
                    broken: false,
                    buildup: 12.5,
                    radscl: 0.75,
                    hit: 0.8,
                    warmup: 0.25,
                    phase_heat: 0.5,
                }),
            ),
            Some(EffectBlockRuntimeState::ForceProjector(
                ForceProjectorState {
                    broken: false,
                    buildup: 12.5,
                    radscl: 0.75,
                    hit: 0.0,
                    warmup: 0.25,
                    phase_heat: 0.5,
                }
            ))
        );
        assert_eq!(
            roundtrip_exported_effect_state(
                &content,
                "radar",
                8,
                8,
                EffectBlockRuntimeState::Radar(RadarState {
                    progress: 0.625,
                    last_radius: 12.0,
                    smooth_efficiency: 0.7,
                    total_progress: 3.0,
                }),
            ),
            Some(EffectBlockRuntimeState::Radar(RadarState {
                progress: 0.625,
                ..RadarState::default()
            }))
        );
        assert_eq!(
            roundtrip_exported_effect_state(
                &content,
                "shield-projector",
                11,
                8,
                EffectBlockRuntimeState::BaseShield(BaseShieldState {
                    broken: true,
                    hit: 0.9,
                    smooth_radius: 18.25,
                }),
            ),
            Some(EffectBlockRuntimeState::BaseShield(BaseShieldState {
                broken: true,
                hit: 0.0,
                smooth_radius: 18.25,
            }))
        );
        assert_eq!(
            roundtrip_exported_effect_state(
                &content,
                "build-tower",
                14,
                8,
                EffectBlockRuntimeState::BuildTurret(BuildTurretState {
                    rotation: 135.0,
                    warmup: 0.6,
                    plans: vec![BuildPlan::new_break(1, 2)],
                    ..BuildTurretState::default()
                }),
            ),
            Some(EffectBlockRuntimeState::BuildTurret(BuildTurretState {
                rotation: 135.0,
                plans: vec![BuildPlan::new_break(1, 2)],
                ..BuildTurretState::default()
            }))
        );
    }

    fn roundtrip_exported_production_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimeProductionBlockState,
    ) -> Option<GameRuntimeProductionBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(16, 16);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(7),
        ));
        runtime
            .production_runtime_states
            .insert(tile_pos, state.clone());

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 16;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("production block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("production block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(1));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.production_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_production_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();

        assert_eq!(
            roundtrip_exported_production_state(
                &content,
                "mechanical-drill",
                1,
                10,
                GameRuntimeProductionBlockState::Drill(DrillState {
                    progress: 120.0,
                    warmup: 0.45,
                    time_drilled: 7.0,
                    last_drill_speed: 2.0,
                }),
            ),
            Some(GameRuntimeProductionBlockState::Drill(DrillState {
                progress: 120.0,
                warmup: 0.45,
                ..DrillState::default()
            }))
        );
        assert_eq!(
            roundtrip_exported_production_state(
                &content,
                "plasma-bore",
                4,
                10,
                GameRuntimeProductionBlockState::BeamDrill(BeamDrillState {
                    time: 44.0,
                    warmup: 0.65,
                    boost_warmup: 0.3,
                    last_drill_speed: 2.5,
                    facing_amount: 3,
                    last_item: Some(2),
                }),
            ),
            Some(GameRuntimeProductionBlockState::BeamDrill(BeamDrillState {
                time: 44.0,
                warmup: 0.65,
                ..BeamDrillState::default()
            }))
        );
        assert_eq!(
            roundtrip_exported_production_state(
                &content,
                "impact-drill",
                8,
                10,
                GameRuntimeProductionBlockState::BurstDrill(BurstDrillState {
                    progress: 240.0,
                    warmup: 0.72,
                    time_drilled: 9.0,
                    last_drill_speed: 1.2,
                    smooth_progress: 0.99,
                    invert_time: 0.5,
                }),
            ),
            Some(GameRuntimeProductionBlockState::BurstDrill(
                BurstDrillState {
                    progress: 240.0,
                    warmup: 0.72,
                    ..BurstDrillState::default()
                }
            ))
        );
    }

    fn exported_crafting_state_revision(state: &GameRuntimeCraftingBlockState) -> u8 {
        match state {
            GameRuntimeCraftingBlockState::Separator(_) => 1,
            _ => 0,
        }
    }

    fn roundtrip_exported_crafting_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimeCraftingBlockState,
    ) -> Option<GameRuntimeCraftingBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let expected_revision = exported_crafting_state_revision(&state);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(16, 16);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(8),
        ));
        runtime.crafting_runtime_states.insert(tile_pos, state);

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 16;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("crafting block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("crafting block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(expected_revision));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.crafting_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_crafting_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();

        assert_eq!(
            roundtrip_exported_crafting_state(
                &content,
                "graphite-press",
                1,
                12,
                GameRuntimeCraftingBlockState::GenericCrafter(GenericCrafterState {
                    progress: 0.375,
                    total_progress: 9.0,
                    warmup: 0.5,
                }),
            ),
            Some(GameRuntimeCraftingBlockState::GenericCrafter(
                GenericCrafterState {
                    progress: 0.375,
                    warmup: 0.5,
                    ..GenericCrafterState::default()
                }
            ))
        );
        assert_eq!(
            roundtrip_exported_crafting_state(
                &content,
                "cultivator",
                4,
                12,
                GameRuntimeCraftingBlockState::GenericCrafter(GenericCrafterState {
                    progress: 0.875,
                    total_progress: 4.0,
                    warmup: 0.25,
                }),
            ),
            Some(GameRuntimeCraftingBlockState::GenericCrafter(
                GenericCrafterState {
                    progress: 0.875,
                    warmup: 0.25,
                    ..GenericCrafterState::default()
                }
            ))
        );
        assert_eq!(
            roundtrip_exported_crafting_state(
                &content,
                "separator",
                7,
                12,
                GameRuntimeCraftingBlockState::Separator(SeparatorState {
                    progress: 0.8,
                    total_progress: 7.0,
                    warmup: 0.25,
                    seed: 12_345,
                }),
            ),
            Some(GameRuntimeCraftingBlockState::Separator(SeparatorState {
                progress: 0.8,
                warmup: 0.25,
                seed: 12_345,
                ..SeparatorState::default()
            }))
        );
        assert_eq!(
            roundtrip_exported_crafting_state(
                &content,
                "oxidation-chamber",
                11,
                12,
                GameRuntimeCraftingBlockState::HeatProducer {
                    crafter: GenericCrafterState {
                        progress: 0.2,
                        total_progress: 5.0,
                        warmup: 0.6,
                    },
                    heat: HeatProducerState { heat: 3.25 },
                },
            ),
            Some(GameRuntimeCraftingBlockState::HeatProducer {
                crafter: GenericCrafterState {
                    progress: 0.2,
                    warmup: 0.6,
                    ..GenericCrafterState::default()
                },
                heat: HeatProducerState { heat: 3.25 },
            })
        );
    }

    fn exported_distribution_state_revision(
        content: &ContentLoader,
        block_name: &str,
        state: &GameRuntimeDistributionBlockState,
    ) -> u8 {
        let block_def = content.block_by_name(block_name).unwrap();
        let BlockDef::Distribution(distribution) = block_def else {
            return 0;
        };

        match (distribution.kind, state) {
            (
                DistributionBlockKind::Conveyor | DistributionBlockKind::ArmoredConveyor,
                GameRuntimeDistributionBlockState::Conveyor(_),
            )
            | (
                DistributionBlockKind::ItemBridge | DistributionBlockKind::DuctBridge,
                GameRuntimeDistributionBlockState::ItemBridge(_),
            )
            | (
                DistributionBlockKind::BufferedItemBridge,
                GameRuntimeDistributionBlockState::BufferedItemBridge(_),
            )
            | (DistributionBlockKind::Duct, GameRuntimeDistributionBlockState::Duct(_))
            | (
                DistributionBlockKind::DuctRouter
                | DistributionBlockKind::OverflowDuct
                | DistributionBlockKind::StackRouter,
                GameRuntimeDistributionBlockState::DuctRouter(_),
            )
            | (
                DistributionBlockKind::Junction,
                GameRuntimeDistributionBlockState::DuctJunction(_),
            )
            | (DistributionBlockKind::Unloader, GameRuntimeDistributionBlockState::Unloader(_)) => {
                1
            }
            (DistributionBlockKind::Sorter, GameRuntimeDistributionBlockState::Sorter(_)) => 2,
            _ => 0,
        }
    }

    fn roundtrip_exported_distribution_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimeDistributionBlockState,
    ) -> Option<GameRuntimeDistributionBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let expected_revision = exported_distribution_state_revision(content, block_name, &state);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(32, 32);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(11),
        ));
        runtime.distribution_runtime_states.insert(tile_pos, state);

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 32;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("distribution block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("distribution block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(expected_revision));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.distribution_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_distribution_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();
        let copper = content
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let lead = content.item_by_name("lead").unwrap().base.mappable.base.id;

        let conveyor = ConveyorState {
            items: vec![ConveyorItemState {
                item: copper,
                x: 0.0,
                y: 0.0,
            }],
        };
        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "conveyor",
                1,
                15,
                GameRuntimeDistributionBlockState::Conveyor(conveyor.clone()),
            ),
            Some(GameRuntimeDistributionBlockState::Conveyor(conveyor))
        );

        let stack = StackConveyorState {
            link: point2_pack(5, 15),
            cooldown: 0.25,
            last_item: Some(copper),
        };
        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "plastanium-conveyor",
                3,
                15,
                GameRuntimeDistributionBlockState::StackConveyor(stack),
            ),
            Some(GameRuntimeDistributionBlockState::StackConveyor(
                StackConveyorState {
                    last_item: None,
                    ..stack
                }
            ))
        );

        let bridge = ItemBridgeState {
            link: point2_pack(8, 15),
            warmup: 0.6,
            incoming: vec![point2_pack(2, 15), point2_pack(4, 15)],
            was_moved: false,
            moved: true,
        };
        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "phase-conveyor",
                5,
                15,
                GameRuntimeDistributionBlockState::ItemBridge(bridge.clone()),
            ),
            Some(GameRuntimeDistributionBlockState::ItemBridge(
                ItemBridgeState {
                    was_moved: true,
                    moved: true,
                    ..bridge
                }
            ))
        );

        let buffered = BufferedItemBridgeState {
            bridge: ItemBridgeState {
                link: point2_pack(11, 15),
                warmup: 0.45,
                incoming: vec![point2_pack(9, 15)],
                was_moved: true,
                moved: false,
            },
            index: 2,
            buffer: vec![123, 456],
        };
        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "bridge-conveyor",
                9,
                15,
                GameRuntimeDistributionBlockState::BufferedItemBridge(buffered.clone()),
            ),
            Some(GameRuntimeDistributionBlockState::BufferedItemBridge(
                BufferedItemBridgeState {
                    bridge: ItemBridgeState {
                        was_moved: true,
                        moved: true,
                        ..buffered.bridge.clone()
                    },
                    ..buffered.clone()
                }
            ))
        );

        let mass_driver = MassDriverState {
            link: point2_pack(16, 15),
            rotation: 135.0,
            state: MassDriverStateKind::Shooting,
        };
        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "mass-driver",
                13,
                15,
                GameRuntimeDistributionBlockState::MassDriver(mass_driver),
            ),
            Some(GameRuntimeDistributionBlockState::MassDriver(mass_driver))
        );

        let duct = DuctState {
            rec_dir: 2,
            current: Some(copper),
        };
        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "duct",
                17,
                15,
                GameRuntimeDistributionBlockState::Duct(duct),
            ),
            Some(GameRuntimeDistributionBlockState::Duct(DuctState {
                current: None,
                ..duct
            }))
        );

        let duct_router = DuctRouterState {
            sort_item: Some(lead),
            current: Some(copper),
        };
        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "duct-router",
                19,
                15,
                GameRuntimeDistributionBlockState::DuctRouter(duct_router),
            ),
            Some(GameRuntimeDistributionBlockState::DuctRouter(
                DuctRouterState {
                    current: None,
                    ..duct_router
                }
            ))
        );

        let junction = DuctJunctionState {
            times: [1.0, 2.0, 3.0, 4.0],
            item_data: [Some(copper), None, Some(lead), None],
        };
        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "junction",
                21,
                15,
                GameRuntimeDistributionBlockState::DuctJunction(junction.clone()),
            ),
            Some(GameRuntimeDistributionBlockState::DuctJunction(junction))
        );

        let directional = DirectionalUnloaderState {
            unload_item: Some(lead),
            offset: 17,
        };
        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "duct-unloader",
                23,
                15,
                GameRuntimeDistributionBlockState::DirectionalUnloader(directional),
            ),
            Some(GameRuntimeDistributionBlockState::DirectionalUnloader(
                directional
            ))
        );

        let sorter = SorterState {
            sort_item: Some(copper),
        };
        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "sorter",
                25,
                15,
                GameRuntimeDistributionBlockState::Sorter(sorter),
            ),
            Some(GameRuntimeDistributionBlockState::Sorter(sorter))
        );

        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "unloader",
                27,
                15,
                GameRuntimeDistributionBlockState::Unloader(Some(lead)),
            ),
            Some(GameRuntimeDistributionBlockState::Unloader(Some(lead)))
        );

        let loader = UnitCargoLoaderState {
            read_unit_id: 77,
            build_progress: 0.5,
            total_progress: 3.0,
            warmup: 0.25,
            readyness: 0.75,
            has_unit: true,
        };
        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "unit-cargo-loader",
                4,
                19,
                GameRuntimeDistributionBlockState::UnitCargoLoader(loader),
            ),
            Some(GameRuntimeDistributionBlockState::UnitCargoLoader(
                UnitCargoLoaderState {
                    read_unit_id: 77,
                    ..UnitCargoLoaderState::default()
                }
            ))
        );

        let unload = UnitCargoUnloadPointState {
            item_id: Some(copper as i32),
            stale_timer: 99.0,
            stale: true,
        };
        assert_eq!(
            roundtrip_exported_distribution_state(
                &content,
                "unit-cargo-unload-point",
                9,
                19,
                GameRuntimeDistributionBlockState::UnitCargoUnload(unload),
            ),
            Some(GameRuntimeDistributionBlockState::UnitCargoUnload(
                UnitCargoUnloadPointState {
                    stale_timer: 0.0,
                    ..unload
                }
            ))
        );
    }

    fn exported_logic_state_revision(
        content: &ContentLoader,
        block_name: &str,
        state: &GameRuntimeLogicBlockState,
    ) -> u8 {
        let block_def = content.block_by_name(block_name).unwrap();
        let BlockDef::Logic(logic) = block_def else {
            return 0;
        };

        match (logic.kind, state) {
            (LogicBlockKind::Switch, GameRuntimeLogicBlockState::Switch { .. })
            | (
                LogicBlockKind::Display | LogicBlockKind::TileDisplay,
                GameRuntimeLogicBlockState::Display(_),
            ) => 1,
            (LogicBlockKind::Processor, GameRuntimeLogicBlockState::Processor(_)) => 4,
            _ => 0,
        }
    }

    fn roundtrip_exported_logic_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimeLogicBlockState,
    ) -> Option<GameRuntimeLogicBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let expected_revision = exported_logic_state_revision(content, block_name, &state);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(32, 32);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(13),
        ));
        runtime.logic_runtime_states.insert(tile_pos, state);

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 32;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("logic block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("logic block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(expected_revision));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.logic_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_logic_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();

        let message = MessageBlockState::new("alpha\nbeta");
        assert_eq!(
            roundtrip_exported_logic_state(
                &content,
                "message",
                1,
                20,
                GameRuntimeLogicBlockState::Message(message.clone()),
            ),
            Some(GameRuntimeLogicBlockState::Message(message))
        );

        assert_eq!(
            roundtrip_exported_logic_state(
                &content,
                "switch",
                3,
                20,
                GameRuntimeLogicBlockState::Switch { enabled: true },
            ),
            Some(GameRuntimeLogicBlockState::Switch { enabled: true })
        );

        let display =
            LogicDisplayState::with_transform([1.0, 0.0, 8.0, 0.0, 1.0, -4.0, 0.0, 0.0, 1.0]);
        assert_eq!(
            roundtrip_exported_logic_state(
                &content,
                "logic-display",
                6,
                20,
                GameRuntimeLogicBlockState::Display(display.clone()),
            ),
            Some(GameRuntimeLogicBlockState::Display(display))
        );

        let mut memory = MemoryBlockState::new(64);
        memory.memory[0] = 7.0;
        memory.memory[5] = -3.5;
        memory.memory[63] = 99.25;
        assert_eq!(
            roundtrip_exported_logic_state(
                &content,
                "memory-cell",
                10,
                20,
                GameRuntimeLogicBlockState::Memory(memory.clone()),
            ),
            Some(GameRuntimeLogicBlockState::Memory(memory))
        );

        let canvas_len = match content.block_by_name("canvas").unwrap() {
            BlockDef::Logic(logic) => logic.canvas_data_bytes as usize,
            _ => unreachable!(),
        };
        let mut canvas_data = vec![0; canvas_len];
        canvas_data[0] = 0b0101_1010;
        canvas_data[canvas_len - 1] = 0b1010_0101;
        let canvas = CanvasBlockState::from_data(canvas_data);
        assert_eq!(
            roundtrip_exported_logic_state(
                &content,
                "canvas",
                13,
                20,
                GameRuntimeLogicBlockState::Canvas(canvas.clone()),
            ),
            Some(GameRuntimeLogicBlockState::Canvas(canvas))
        );

        let config =
            LogicConfig::from_code(b"set counter 1", vec![LogicLink::new(1, 0, "cell1", false)]);
        let mut processor = LogicProcessorState::from_config(config).unwrap();
        processor.variables = vec![LogicProcessorVariableState::new(
            "counter",
            TypeValue::Double(1.0),
        )];
        processor.tag = Some("loop".into());
        processor.icon_tag = 'L' as u16;
        processor.waits = vec![LogicProcessorWaitState::new(0, 0.25)];
        processor.accumulator = 0.75;
        assert_eq!(
            roundtrip_exported_logic_state(
                &content,
                "micro-processor",
                17,
                20,
                GameRuntimeLogicBlockState::Processor(processor.clone()),
            ),
            Some(GameRuntimeLogicBlockState::Processor(processor))
        );
    }

    fn roundtrip_exported_campaign_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimeCampaignBlockState,
    ) -> Option<GameRuntimeCampaignBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(32, 32);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(14),
        ));
        runtime.campaign_runtime_states.insert(tile_pos, state);

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 32;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("campaign block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("campaign block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(1));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.campaign_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_campaign_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();
        let copper = content
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let lead = content.item_by_name("lead").unwrap().base.mappable.base.id;

        let launch = LaunchPadState {
            launch_counter: 12.5,
        };
        assert_eq!(
            roundtrip_exported_campaign_state(
                &content,
                "launch-pad",
                1,
                24,
                GameRuntimeCampaignBlockState::LaunchPad(launch),
            ),
            Some(GameRuntimeCampaignBlockState::LaunchPad(launch))
        );

        let landing = LandingPadState {
            config: Some(copper),
            priority: 2,
            cooldown: 0.5,
            arriving: Some(lead),
            arriving_timer: 0.25,
            liquid_removed: 3.0,
        };
        assert_eq!(
            roundtrip_exported_campaign_state(
                &content,
                "landing-pad",
                6,
                24,
                GameRuntimeCampaignBlockState::LandingPad(landing.clone()),
            ),
            Some(GameRuntimeCampaignBlockState::LandingPad(landing))
        );

        let accelerator = AcceleratorState {
            progress: 0.75,
            launching: true,
        };
        assert_eq!(
            roundtrip_exported_campaign_state(
                &content,
                "interplanetary-accelerator",
                12,
                24,
                GameRuntimeCampaignBlockState::Accelerator(accelerator.clone()),
            ),
            Some(GameRuntimeCampaignBlockState::Accelerator(
                AcceleratorState {
                    launching: false,
                    ..accelerator
                }
            ))
        );
    }

    fn exported_unit_state_revision(block_def: &BlockDef, state: &GameRuntimeUnitBlockState) -> u8 {
        match (block_def, state) {
            (BlockDef::UnitFactory(_), GameRuntimeUnitBlockState::Factory { .. })
            | (BlockDef::UnitReconstructor(_), GameRuntimeUnitBlockState::Reconstructor { .. }) => {
                3
            }
            (BlockDef::UnitRepairTower(_), GameRuntimeUnitBlockState::RepairTower(_))
            | (BlockDef::UnitAssembler(_), GameRuntimeUnitBlockState::Assembler { .. }) => 1,
            _ => 0,
        }
    }

    fn roundtrip_exported_unit_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimeUnitBlockState,
    ) -> Option<GameRuntimeUnitBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let expected_revision = exported_unit_state_revision(block_def, &state);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(32, 32);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(15),
        ));
        runtime.unit_runtime_states.insert(tile_pos, state);

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 32;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("unit block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("unit block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(expected_revision));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.unit_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_unit_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: 1.0, y: -2.0 },
            pay_rotation: 90.0,
            carried: false,
        };
        let factory = UnitFactoryState {
            base: crate::mindustry::world::blocks::units::UnitBlockState {
                progress: 25.0,
                ..Default::default()
            },
            current_plan: 1,
            command_pos: Some(IoVec2 { x: 12.0, y: 34.0 }),
            command_id: Some(2),
        };
        assert_eq!(
            roundtrip_exported_unit_state(
                &content,
                "ground-factory",
                2,
                27,
                GameRuntimeUnitBlockState::Factory {
                    common: common.clone(),
                    factory: factory.clone(),
                },
            ),
            Some(GameRuntimeUnitBlockState::Factory {
                common: common.clone(),
                factory
            })
        );

        let reconstructor = ReconstructorState {
            base: crate::mindustry::world::blocks::units::UnitBlockState {
                progress: 11.0,
                ..Default::default()
            },
            command_pos: Some(IoVec2 { x: 8.0, y: 16.0 }),
            command_id: Some(3),
            constructing: true,
        };
        assert_eq!(
            roundtrip_exported_unit_state(
                &content,
                "additive-reconstructor",
                6,
                27,
                GameRuntimeUnitBlockState::Reconstructor {
                    common: common.clone(),
                    reconstructor: reconstructor.clone(),
                },
            ),
            Some(GameRuntimeUnitBlockState::Reconstructor {
                common: common.clone(),
                reconstructor: ReconstructorState {
                    constructing: false,
                    ..reconstructor
                }
            })
        );

        let repair = RepairTurretState {
            target_present: true,
            strength: 0.5,
            rotation: 45.0,
        };
        assert_eq!(
            roundtrip_exported_unit_state(
                &content,
                "unit-repair-tower",
                10,
                27,
                GameRuntimeUnitBlockState::RepairTower(repair),
            ),
            Some(GameRuntimeUnitBlockState::RepairTower(RepairTurretState {
                rotation: 45.0,
                ..RepairTurretState::default()
            }))
        );

        let assembler = UnitAssemblerState {
            progress: 0.6,
            read_unit_ids: vec![101, 102],
            blocks: PayloadSeq::new(),
            command_pos: Some(IoVec2 { x: 64.0, y: 96.0 }),
            ..UnitAssemblerState::default()
        };
        assert_eq!(
            roundtrip_exported_unit_state(
                &content,
                "tank-assembler",
                14,
                27,
                GameRuntimeUnitBlockState::Assembler {
                    common: common.clone(),
                    assembler: assembler.clone(),
                },
            ),
            Some(GameRuntimeUnitBlockState::Assembler {
                common: common.clone(),
                assembler
            })
        );

        let module_common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: -0.25, y: 0.5 },
            pay_rotation: 180.0,
            carried: false,
        };
        assert_eq!(
            roundtrip_exported_unit_state(
                &content,
                "basic-assembler-module",
                18,
                27,
                GameRuntimeUnitBlockState::AssemblerModule(module_common.clone()),
            ),
            Some(GameRuntimeUnitBlockState::AssemblerModule(module_common))
        );
    }

    fn exported_turret_state_revision(
        block_def: &BlockDef,
        state: &GameRuntimeTurretBlockState,
    ) -> u8 {
        let BlockDef::Turret(turret) = block_def else {
            return 0;
        };

        match (turret.kind, state) {
            (TurretBlockKind::ItemTurret, GameRuntimeTurretBlockState::Item { .. }) => 2,
            (
                TurretBlockKind::ContinuousTurret | TurretBlockKind::ContinuousLiquidTurret,
                GameRuntimeTurretBlockState::Continuous { .. },
            ) => 3,
            (
                TurretBlockKind::PayloadAmmoTurret,
                GameRuntimeTurretBlockState::PayloadAmmo { .. },
            )
            | (
                TurretBlockKind::LiquidTurret
                | TurretBlockKind::PowerTurret
                | TurretBlockKind::LaserTurret,
                GameRuntimeTurretBlockState::Generic(_),
            ) => 1,
            _ => 0,
        }
    }

    fn roundtrip_exported_turret_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimeTurretBlockState,
    ) -> Option<GameRuntimeTurretBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let expected_revision = exported_turret_state_revision(block_def, &state);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(32, 32);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(16),
        ));
        runtime.turret_runtime_states.insert(tile_pos, state);

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 32;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("turret block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("turret block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(expected_revision));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.turret_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_turret_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();
        let copper = content
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;

        let generic = TurretState {
            reload_counter: 2.0,
            rotation: 30.0,
            ..TurretState::default()
        };
        assert_eq!(
            roundtrip_exported_turret_state(
                &content,
                "arc",
                1,
                30,
                GameRuntimeTurretBlockState::Generic(generic.clone()),
            ),
            Some(GameRuntimeTurretBlockState::Generic(generic))
        );

        let item_turret = TurretState {
            reload_counter: 3.5,
            rotation: 45.0,
            ..TurretState::default()
        };
        let ammo = vec![ItemAmmoEntry {
            item_id: copper,
            amount: 7,
        }];
        assert_eq!(
            roundtrip_exported_turret_state(
                &content,
                "duo",
                4,
                30,
                GameRuntimeTurretBlockState::Item {
                    turret: item_turret.clone(),
                    ammo: ammo.clone(),
                },
            ),
            Some(GameRuntimeTurretBlockState::Item {
                turret: TurretState {
                    total_ammo: 7,
                    ..item_turret
                },
                ammo
            })
        );

        let continuous_turret = TurretState {
            reload_counter: 6.0,
            rotation: 135.0,
            ..TurretState::default()
        };
        let continuous = ContinuousTurretState {
            last_length: 38.0,
            bullets: 2,
        };
        assert_eq!(
            roundtrip_exported_turret_state(
                &content,
                "lustre",
                7,
                30,
                GameRuntimeTurretBlockState::Continuous {
                    turret: continuous_turret.clone(),
                    continuous,
                },
            ),
            Some(GameRuntimeTurretBlockState::Continuous {
                turret: continuous_turret,
                continuous: ContinuousTurretState {
                    bullets: 0,
                    ..continuous
                },
            })
        );

        let point = PointDefenseState {
            rotation: 270.0,
            has_target: true,
            ..PointDefenseState::default()
        };
        assert_eq!(
            roundtrip_exported_turret_state(
                &content,
                "segment",
                11,
                30,
                GameRuntimeTurretBlockState::PointDefense(point),
            ),
            Some(GameRuntimeTurretBlockState::PointDefense(
                PointDefenseState {
                    rotation: 270.0,
                    ..PointDefenseState::default()
                }
            ))
        );

        let tractor = TractorBeamState {
            rotation: 315.0,
            strength: 0.5,
            any: true,
            ..TractorBeamState::default()
        };
        assert_eq!(
            roundtrip_exported_turret_state(
                &content,
                "parallax",
                14,
                30,
                GameRuntimeTurretBlockState::TractorBeam(tractor),
            ),
            Some(GameRuntimeTurretBlockState::TractorBeam(TractorBeamState {
                rotation: 315.0,
                ..TractorBeamState::default()
            }))
        );
    }

    fn exported_payload_state_revision(
        block_def: &BlockDef,
        state: &GameRuntimePayloadBlockState,
    ) -> u8 {
        match (block_def, state) {
            (BlockDef::Payload(payload), GameRuntimePayloadBlockState::Router { .. })
                if payload.kind == PayloadBlockKind::PayloadRouter =>
            {
                1
            }
            (BlockDef::PayloadMassDriver(_), GameRuntimePayloadBlockState::MassDriver { .. })
            | (BlockDef::PayloadLoader(_), GameRuntimePayloadBlockState::Loader { .. }) => 1,
            (BlockDef::Sandbox(sandbox), GameRuntimePayloadBlockState::Source { .. })
                if sandbox.kind == SandboxBlockKind::PayloadSource =>
            {
                1
            }
            _ => 0,
        }
    }

    fn roundtrip_exported_payload_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimePayloadBlockState,
    ) -> Option<GameRuntimePayloadBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let expected_revision = exported_payload_state_revision(block_def, &state);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(32, 32);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(17),
        ));
        runtime.payload_runtime_states.insert(tile_pos, state);

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 32;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("payload block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("payload block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(expected_revision));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.payload_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_payload_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();
        let router_def = content.block_by_name("router").unwrap();
        let flare = content.unit_by_name("flare").unwrap().id();
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: 1.5, y: -2.25 },
            pay_rotation: 45.0,
            carried: false,
        };

        let conveyor = PayloadConveyorState {
            progress: 0.4,
            item_rotation: 90.0,
            animation: 1.0,
            ..PayloadConveyorState::default()
        };
        assert_eq!(
            roundtrip_exported_payload_state(
                &content,
                "payload-conveyor",
                1,
                26,
                GameRuntimePayloadBlockState::Conveyor(conveyor.clone()),
            ),
            Some(GameRuntimePayloadBlockState::Conveyor(
                PayloadConveyorState {
                    item_rotation: 90.0,
                    ..PayloadConveyorState::default()
                }
            ))
        );

        let router_conveyor = PayloadConveyorState {
            progress: 0.75,
            item_rotation: 180.0,
            ..PayloadConveyorState::default()
        };
        let sorted = Some(PayloadSortKey {
            content_type: ContentType::Block.ordinal() as i8,
            id: router_def.base().id,
        });
        assert_eq!(
            roundtrip_exported_payload_state(
                &content,
                "payload-router",
                4,
                26,
                GameRuntimePayloadBlockState::Router {
                    conveyor: router_conveyor.clone(),
                    sorted,
                    rec_dir: 3,
                },
            ),
            Some(GameRuntimePayloadBlockState::Router {
                conveyor: PayloadConveyorState {
                    item_rotation: 180.0,
                    ..PayloadConveyorState::default()
                },
                sorted,
                rec_dir: 3,
            })
        );

        let driver = PayloadMassDriverState {
            link: point2_pack(9, 26),
            turret_rotation: 135.0,
            state: PayloadDriverState::Shooting,
            reload_counter: 0.5,
            charge: 0.75,
            loaded: true,
            charging: true,
        };
        assert_eq!(
            roundtrip_exported_payload_state(
                &content,
                "payload-mass-driver",
                7,
                26,
                GameRuntimePayloadBlockState::MassDriver {
                    common: common.clone(),
                    driver,
                },
            ),
            Some(GameRuntimePayloadBlockState::MassDriver {
                common: common.clone(),
                driver,
            })
        );

        let loader = PayloadLoaderState {
            exporting: true,
            payload_has_items: true,
            payload_items_total: 7,
            ..PayloadLoaderState::default()
        };
        assert_eq!(
            roundtrip_exported_payload_state(
                &content,
                "payload-loader",
                11,
                26,
                GameRuntimePayloadBlockState::Loader {
                    common: common.clone(),
                    loader,
                },
            ),
            Some(GameRuntimePayloadBlockState::Loader {
                common: common.clone(),
                loader: PayloadLoaderState {
                    exporting: true,
                    ..PayloadLoaderState::default()
                },
            })
        );

        let unloader = PayloadLoaderState {
            exporting: true,
            payload_has_items: true,
            payload_items_total: 0,
            has_battery: true,
            payload_power_status: 0.0,
            ..PayloadLoaderState::default()
        };
        assert_eq!(
            roundtrip_exported_payload_state(
                &content,
                "payload-unloader",
                2,
                28,
                GameRuntimePayloadBlockState::Loader {
                    common: common.clone(),
                    loader: unloader,
                },
            ),
            Some(GameRuntimePayloadBlockState::Loader {
                common: common.clone(),
                loader: PayloadLoaderState {
                    exporting: true,
                    ..PayloadLoaderState::default()
                },
            })
        );

        let deconstructor = PayloadDeconstructorState {
            progress: 0.625,
            accum: Some(vec![1.0, 2.0, 3.0]),
            has_payload: true,
            has_deconstructing: true,
            deconstructing: None,
        };
        assert_eq!(
            roundtrip_exported_payload_state(
                &content,
                "deconstructor",
                15,
                26,
                GameRuntimePayloadBlockState::Deconstructor {
                    common: common.clone(),
                    deconstructor: deconstructor.clone(),
                },
            ),
            Some(GameRuntimePayloadBlockState::Deconstructor {
                common: common.clone(),
                deconstructor: PayloadDeconstructorState {
                    has_payload: false,
                    has_deconstructing: false,
                    ..deconstructor
                },
            })
        );

        let producer = BlockProducerState {
            progress: 0.5,
            time: 9.0,
            heat: 0.75,
            has_payload: true,
        };
        assert_eq!(
            roundtrip_exported_payload_state(
                &content,
                "constructor",
                19,
                26,
                GameRuntimePayloadBlockState::Constructor {
                    common: common.clone(),
                    producer,
                    recipe: Some(router_def.base().id),
                },
            ),
            Some(GameRuntimePayloadBlockState::Constructor {
                common: common.clone(),
                producer: BlockProducerState {
                    progress: 0.5,
                    ..BlockProducerState::default()
                },
                recipe: Some(router_def.base().id),
            })
        );

        let source = PayloadSourceState {
            unit: Some(flare),
            command_pos: Some(PayloadVec2 { x: 8.0, y: 16.0 }),
            has_payload: true,
            scl: 0.5,
            ..PayloadSourceState::default()
        };
        assert_eq!(
            roundtrip_exported_payload_state(
                &content,
                "payload-source",
                23,
                26,
                GameRuntimePayloadBlockState::Source {
                    common: common.clone(),
                    source,
                },
            ),
            Some(GameRuntimePayloadBlockState::Source {
                common: common.clone(),
                source: PayloadSourceState {
                    unit: Some(flare),
                    command_pos: Some(PayloadVec2 { x: 8.0, y: 16.0 }),
                    ..PayloadSourceState::default()
                },
            })
        );

        let void_common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: -1.0, y: 2.0 },
            pay_rotation: 270.0,
            carried: false,
        };
        assert_eq!(
            roundtrip_exported_payload_state(
                &content,
                "payload-void",
                27,
                26,
                GameRuntimePayloadBlockState::Void(void_common.clone()),
            ),
            Some(GameRuntimePayloadBlockState::Void(void_common))
        );
    }

    fn roundtrip_exported_liquid_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimeLiquidBlockState,
    ) -> Option<GameRuntimeLiquidBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(16, 16);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(9),
        ));
        runtime.liquid_runtime_states.insert(tile_pos, state);

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 16;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("liquid block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("liquid block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(1));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.liquid_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_liquid_bridge_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();
        let state = LiquidBridgeState {
            link: point2_pack(6, 13),
            warmup: 0.8,
            incoming: vec![point2_pack(2, 13), point2_pack(3, 13)],
            was_moved: true,
            moved: false,
        };
        assert_eq!(
            roundtrip_exported_liquid_state(
                &content,
                "bridge-conduit",
                4,
                13,
                GameRuntimeLiquidBlockState::Bridge(state.clone()),
            ),
            Some(GameRuntimeLiquidBlockState::Bridge(LiquidBridgeState {
                was_moved: true,
                moved: true,
                ..state
            }))
        );
    }

    fn roundtrip_exported_storage_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimeStorageBlockState,
    ) -> Option<GameRuntimeStorageBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(16, 16);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(10),
        ));
        runtime.storage_runtime_states.insert(tile_pos, state);

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 16;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("storage block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("storage block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(1));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.storage_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_core_storage_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();
        let state = CoreBuildState {
            storage_capacity: 100,
            no_effect: true,
            iframes: 4.0,
            thruster_time: 2.0,
            command_pos: Some(IoVec2 { x: 64.0, y: 128.0 }),
        };
        assert_eq!(
            roundtrip_exported_storage_state(
                &content,
                "core-shard",
                10,
                13,
                GameRuntimeStorageBlockState::Core(state),
            ),
            Some(GameRuntimeStorageBlockState::Core(CoreBuildState {
                command_pos: Some(IoVec2 { x: 64.0, y: 128.0 }),
                ..CoreBuildState::default()
            }))
        );
    }

    fn exported_sandbox_state_revision(state: &GameRuntimeSandboxBlockState) -> u8 {
        match state {
            GameRuntimeSandboxBlockState::LiquidSource(_) => 1,
            GameRuntimeSandboxBlockState::ItemSource(_) => 0,
        }
    }

    fn roundtrip_exported_sandbox_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimeSandboxBlockState,
    ) -> Option<GameRuntimeSandboxBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let expected_revision = exported_sandbox_state_revision(&state);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(16, 16);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(11),
        ));
        runtime.sandbox_runtime_states.insert(tile_pos, state);

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 16;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("sandbox block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("sandbox block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(expected_revision));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.sandbox_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_sandbox_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();
        let copper = content
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let water = content
            .liquid_by_name("water")
            .unwrap()
            .base
            .mappable
            .base
            .id;

        assert_eq!(
            roundtrip_exported_sandbox_state(
                &content,
                "item-source",
                1,
                14,
                GameRuntimeSandboxBlockState::ItemSource(ItemSourceState {
                    counter: 42.0,
                    output_item: Some(copper),
                }),
            ),
            Some(GameRuntimeSandboxBlockState::ItemSource(ItemSourceState {
                output_item: Some(copper),
                ..ItemSourceState::default()
            }))
        );
        assert_eq!(
            roundtrip_exported_sandbox_state(
                &content,
                "liquid-source",
                3,
                14,
                GameRuntimeSandboxBlockState::LiquidSource(LiquidSourceState {
                    source: Some(water),
                    stored_liquid: Some(water),
                    amount: 9_999.0,
                }),
            ),
            Some(GameRuntimeSandboxBlockState::LiquidSource(
                LiquidSourceState {
                    source: Some(water),
                    ..LiquidSourceState::default()
                }
            ))
        );
    }

    fn roundtrip_exported_legacy_state(
        content: &ContentLoader,
        block_name: &str,
        x: i32,
        y: i32,
        state: GameRuntimeLegacyBlockState,
    ) -> Option<GameRuntimeLegacyBlockState> {
        let block_def = content.block_by_name(block_name).unwrap();
        let tile_pos = point2_pack(x, y);
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(16, 16);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            block_def.base().clone(),
            TeamId(12),
        ));
        runtime.legacy_runtime_states.insert(tile_pos, state);

        let map = runtime.export_network_map_snapshot(content);
        let center_index = x as usize + y as usize * 16;
        let center = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("legacy block center should be exported explicitly");
        let payload = center
            .building
            .as_ref()
            .expect("legacy block center should carry building payload");
        assert_eq!(payload.first().copied(), Some(0));
        assert!(payload.len() > 1);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(content, &map);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        loaded.legacy_runtime_states.get(&tile_pos).cloned()
    }

    #[test]
    fn game_runtime_exports_legacy_state_tail_in_network_map_snapshot() {
        let content = ContentLoader::create_base_content().unwrap();
        let factory_extra = LegacyUnitFactoryExtra {
            build_time: 120.0,
            spawn_count: Some(3),
        };

        assert_eq!(
            roundtrip_exported_legacy_state(
                &content,
                "command-center",
                5,
                14,
                GameRuntimeLegacyBlockState::CommandCenter(7),
            ),
            Some(GameRuntimeLegacyBlockState::CommandCenter(0))
        );
        assert_eq!(
            roundtrip_exported_legacy_state(
                &content,
                "legacy-mech-pad",
                7,
                14,
                GameRuntimeLegacyBlockState::MechPad([1.0, 2.5, -3.0]),
            ),
            Some(GameRuntimeLegacyBlockState::MechPad([1.0, 2.5, -3.0]))
        );
        assert_eq!(
            roundtrip_exported_legacy_state(
                &content,
                "legacy-unit-factory",
                11,
                14,
                GameRuntimeLegacyBlockState::UnitFactory(factory_extra),
            ),
            Some(GameRuntimeLegacyBlockState::UnitFactory(factory_extra))
        );
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
    fn game_runtime_loads_construct_block_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let build_def = content.block_by_name("build4").unwrap();
        let previous = content.block_by_name("router").map(|block| block.base().id);
        let current = content.block_by_name("duo").map(|block| block.base().id);
        let tile_pos = point2_pack(4, 2);
        let saved = BuildingComp::new(tile_pos, build_def.base().clone(), TeamId(1));
        let state = ConstructBlockState {
            progress: 0.5,
            previous,
            current,
            accumulator: Some(vec![ConstructAccumulatorEntry {
                accumulator: 1.0,
                total_accumulator: 2.0,
                items_left: 3,
            }]),
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_construct_block_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 16, build_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.construct_runtime_states.get(&tile_pos),
            Some(&GameRuntimeConstructBlockState { size: 4, state })
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
    fn game_runtime_loads_build_turret_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let turret_def = content.block_by_name("build-tower").unwrap();
        let tile_pos = point2_pack(3, 2);
        let saved = BuildingComp::new(tile_pos, turret_def.base().clone(), TeamId(1));
        let state = BuildTurretState {
            rotation: 135.0,
            plans: vec![BuildPlan::new_break(1, 2)],
            ..BuildTurretState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        build_turret_write_child_with_loader(&mut building_bytes, &content, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 15, turret_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.effect_runtime_store.get(tile_pos),
            Some(&EffectBlockRuntimeState::BuildTurret(state))
        );
    }

    #[test]
    fn game_runtime_preserves_build_turret_unparseable_raw_plans() {
        let content = ContentLoader::create_base_content().unwrap();
        let turret_def = content.block_by_name("build-tower").unwrap();
        let tile_pos = point2_pack(3, 2);
        let saved = BuildingComp::new(tile_pos, turret_def.base().clone(), TeamId(1));
        let state = BuildTurretState {
            rotation: 225.0,
            raw_plans: vec![0x00, 0x01, 0xff],
            ..BuildTurretState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        build_turret_write_child_with_loader(&mut building_bytes, &content, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 15, turret_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.effect_runtime_store.get(tile_pos),
            Some(&EffectBlockRuntimeState::BuildTurret(state))
        );
    }

    #[test]
    fn game_runtime_loads_door_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let door_def = content.block_by_name("door").unwrap();
        let tile_pos = point2_pack(1, 0);
        let saved = BuildingComp::new(tile_pos, door_def.base().clone(), TeamId(1));
        let state = DoorState { open: true };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_door_state(&mut building_bytes, state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 1, door_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.defense_wall_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDefenseWallState::Door(state))
        );
    }

    #[test]
    fn game_runtime_reports_trailing_block_state_bytes_after_successful_read() {
        let content = ContentLoader::create_base_content().unwrap();
        let door_def = content.block_by_name("door").unwrap();
        let tile_pos = point2_pack(1, 0);
        let saved = BuildingComp::new(tile_pos, door_def.base().clone(), TeamId(1));
        let state = DoorState { open: true };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_door_state(&mut building_bytes, state).unwrap();
        building_bytes.extend_from_slice(&[0xaa, 0xbb]);

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 1, door_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 1);
        assert_eq!(
            runtime.defense_wall_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDefenseWallState::Door(state))
        );
    }

    #[test]
    fn game_runtime_loads_auto_door_state_and_consumes_parent_child_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let door_def = content.block_by_name("blast-door").unwrap();
        let tile_pos = point2_pack(2, 0);
        let saved = BuildingComp::new(tile_pos, door_def.base().clone(), TeamId(1));
        let state = DoorState { open: true };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_auto_door_state(&mut building_bytes, state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 2, door_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.defense_wall_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDefenseWallState::Door(state))
        );
    }

    #[test]
    fn game_runtime_loads_shield_wall_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let wall_def = content.block_by_name("shielded-wall").unwrap();
        let tile_pos = point2_pack(2, 0);
        let saved = BuildingComp::new(tile_pos, wall_def.base().clone(), TeamId(1));
        let state = ShieldWallState {
            shield: 75.0,
            shield_radius: 1.0,
            break_timer: 0.0,
            hit: 0.0,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_shield_wall_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 2, wall_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.defense_wall_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDefenseWallState::ShieldWall(state))
        );
    }

    #[test]
    fn game_runtime_loads_item_turret_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let turret_def = content.block_by_name("duo").unwrap();
        let copper = content.item_by_name("copper").unwrap();
        let tile_pos = point2_pack(2, 1);
        let saved = BuildingComp::new(tile_pos, turret_def.base().clone(), TeamId(1));
        let mut turret = TurretState {
            reload_counter: 3.5,
            rotation: 45.0,
            ..TurretState::default()
        };
        let ammo = vec![ItemAmmoEntry {
            item_id: copper.base.mappable.base.id,
            amount: 7,
        }];
        let mut building_bytes = Vec::new();
        building_bytes.push(2);
        saved.write_base(&mut building_bytes, false).unwrap();
        turret_write_child(&mut building_bytes, &turret).unwrap();
        item_turret_write_ammo(&mut building_bytes, &ammo).unwrap();
        turret.total_ammo = 7;

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 8, turret_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.turret_runtime_states.get(&tile_pos),
            Some(&GameRuntimeTurretBlockState::Item { turret, ammo })
        );
    }

    #[test]
    fn game_runtime_reads_payload_ammo_turret_state_and_filters_invalid_payloads() {
        let content = ContentLoader::create_base_content().unwrap();
        let router = content.block_by_name("router").unwrap();
        let flare = content.unit_by_name("flare").unwrap();
        let valid_key = PayloadKey::new(ContentType::Block, router.base().id);
        let invalid_key = PayloadKey::new(ContentType::Unit, flare.id());
        let mut turret_block =
            TurretBlockData::new(900, "payload-ammo-test", TurretBlockKind::PayloadAmmoTurret);
        turret_block.payload_ammo.push(PayloadTurretAmmo {
            content: valid_key,
            bullet: BulletSpec::new(BulletKind::Basic, 1.0, 1.0),
        });
        let block = BlockDef::Turret(turret_block);
        let mut turret = TurretState {
            reload_counter: 3.5,
            rotation: 45.0,
            ..TurretState::default()
        };
        let mut payloads = PayloadSeq::new();
        payloads.add(valid_key, 2);
        payloads.add(invalid_key, 1);
        let mut building_payload = Vec::new();
        turret_write_child(&mut building_payload, &turret).unwrap();
        payload_ammo_turret_write_payloads(&mut building_payload, &payloads).unwrap();

        let runtime = GameRuntime::default();
        let mut payload_slice = building_payload.as_slice();
        let loaded = runtime
            .read_turret_runtime_state_from_building_payload(&block, 1, &mut payload_slice)
            .unwrap();

        let mut filtered = PayloadSeq::new();
        filtered.add(valid_key, 2);
        turret.total_ammo = 2;
        assert!(payload_slice.is_empty());
        assert_eq!(
            loaded,
            Some(GameRuntimeTurretBlockState::PayloadAmmo {
                turret,
                payloads: filtered,
            })
        );
    }

    #[test]
    fn game_runtime_reads_payload_ammo_turret_legacy_block_only_payloads() {
        let content = ContentLoader::create_base_content().unwrap();
        let router = content.block_by_name("router").unwrap();
        let junction = content.block_by_name("junction").unwrap();
        let valid_key = PayloadKey::new(ContentType::Block, router.base().id);
        let invalid_key = PayloadKey::new(ContentType::Block, junction.base().id);
        let mut turret_block = TurretBlockData::new(
            901,
            "payload-ammo-legacy-test",
            TurretBlockKind::PayloadAmmoTurret,
        );
        turret_block.payload_ammo.push(PayloadTurretAmmo {
            content: valid_key,
            bullet: BulletSpec::new(BulletKind::Basic, 1.0, 1.0),
        });
        let block = BlockDef::Turret(turret_block);
        let mut turret = TurretState {
            reload_counter: 2.5,
            rotation: 30.0,
            ..TurretState::default()
        };
        let mut building_payload = Vec::new();
        turret_write_child(&mut building_payload, &turret).unwrap();
        building_payload.extend_from_slice(&2i16.to_be_bytes());
        building_payload.extend_from_slice(&(valid_key.id as i16).to_be_bytes());
        building_payload.extend_from_slice(&4i32.to_be_bytes());
        building_payload.extend_from_slice(&(invalid_key.id as i16).to_be_bytes());
        building_payload.extend_from_slice(&1i32.to_be_bytes());

        let runtime = GameRuntime::default();
        let mut payload_slice = building_payload.as_slice();
        let loaded = runtime
            .read_turret_runtime_state_from_building_payload(&block, 1, &mut payload_slice)
            .unwrap();

        let mut filtered = PayloadSeq::new();
        filtered.add(valid_key, 4);
        turret.total_ammo = 4;
        assert!(payload_slice.is_empty());
        assert_eq!(
            loaded,
            Some(GameRuntimeTurretBlockState::PayloadAmmo {
                turret,
                payloads: filtered,
            })
        );
    }

    #[test]
    fn game_runtime_loads_continuous_turret_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let turret_def = content.block_by_name("lustre").unwrap();
        let tile_pos = point2_pack(3, 1);
        let saved = BuildingComp::new(tile_pos, turret_def.base().clone(), TeamId(1));
        let turret = TurretState {
            reload_counter: 6.0,
            rotation: 135.0,
            ..TurretState::default()
        };
        let continuous = ContinuousTurretState {
            last_length: 38.0,
            bullets: 0,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(3);
        saved.write_base(&mut building_bytes, false).unwrap();
        turret_write_child(&mut building_bytes, &turret).unwrap();
        continuous_turret_write_child(&mut building_bytes, &continuous).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 9, turret_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.turret_runtime_states.get(&tile_pos),
            Some(&GameRuntimeTurretBlockState::Continuous { turret, continuous })
        );
    }

    #[test]
    fn game_runtime_loads_point_defense_turret_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let turret_def = content.block_by_name("segment").unwrap();
        let tile_pos = point2_pack(4, 1);
        let saved = BuildingComp::new(tile_pos, turret_def.base().clone(), TeamId(1));
        let state = PointDefenseState {
            rotation: 270.0,
            ..PointDefenseState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        point_defense_write_child(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 10, turret_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.turret_runtime_states.get(&tile_pos),
            Some(&GameRuntimeTurretBlockState::PointDefense(state))
        );
    }

    #[test]
    fn game_runtime_loads_tractor_beam_turret_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let turret_def = content.block_by_name("parallax").unwrap();
        let tile_pos = point2_pack(5, 1);
        let saved = BuildingComp::new(tile_pos, turret_def.base().clone(), TeamId(1));
        let state = TractorBeamState {
            rotation: 315.0,
            ..TractorBeamState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        tractor_beam_write_child(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 11, turret_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.turret_runtime_states.get(&tile_pos),
            Some(&GameRuntimeTurretBlockState::TractorBeam(state))
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
    fn game_runtime_loads_payload_mass_driver_revision_zero_without_tail_fields() {
        let content = ContentLoader::create_base_content().unwrap();
        let driver_def = content.block_by_name("payload-mass-driver").unwrap();
        let tile_pos = point2_pack(2, 3);
        let saved = BuildingComp::new(tile_pos, driver_def.base().clone(), TeamId(6));
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: 0.25, y: -0.5 },
            pay_rotation: 90.0,
            carried: false,
        };
        let driver = PayloadMassDriverState {
            link: point2_pack(1, 3),
            turret_rotation: 270.0,
            state: PayloadDriverState::Accepting,
            ..PayloadMassDriverState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
        building_bytes.extend_from_slice(&driver.link.to_be_bytes());
        building_bytes.extend_from_slice(&driver.turret_rotation.to_bits().to_be_bytes());
        building_bytes.push(driver.state.ordinal());

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
    fn game_runtime_loads_payload_unloader_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let unloader_def = content.block_by_name("payload-unloader").unwrap();
        let tile_pos = point2_pack(4, 2);
        let mut saved = BuildingComp::new(tile_pos, unloader_def.base().clone(), TeamId(6));
        saved.set_rotation(1);
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: 2.0, y: -1.0 },
            pay_rotation: 90.0,
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
            &single_building_network_map(6, 6, 16, unloader_def.base().id, building_bytes),
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
    fn game_runtime_loads_payload_void_terminal_common_payload_state() {
        let content = ContentLoader::create_base_content().unwrap();
        let void_def = content.block_by_name("payload-void").unwrap();
        let router_id = content.block_by_name("router").unwrap().base().id;
        let tile_pos = point2_pack(0, 5);
        let saved = BuildingComp::new(tile_pos, void_def.base().clone(), TeamId(6));
        let common = PayloadBlockBuildState {
            payload: Some(PayloadRef::Block {
                block: router_id,
                version: 1,
                build_bytes: vec![0x33, 0x44],
            }),
            pay_vector: Vec2 { x: 0.25, y: -0.75 },
            pay_rotation: 270.0,
            carried: false,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 30, void_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.payload_runtime_states.get(&tile_pos),
            Some(&GameRuntimePayloadBlockState::Void(common))
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
    fn game_runtime_loads_terminal_payload_conveyor_raw_item_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let conveyor_def = content.block_by_name("payload-conveyor").unwrap();
        let router_id = content.block_by_name("router").unwrap().base().id;
        let tile_pos = point2_pack(1, 4);
        let saved = BuildingComp::new(tile_pos, conveyor_def.base().clone(), TeamId(6));
        let item = PayloadRef::Block {
            block: router_id,
            version: 1,
            build_bytes: vec![0xaa, 0xbb, 0xcc],
        };
        let conveyor = PayloadConveyorState {
            item: Some(item.clone()),
            item_rotation: 33.0,
            ..PayloadConveyorState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_conveyor_extra(
            &mut building_bytes,
            12.0,
            conveyor.item_rotation,
            Some(&item),
        )
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
    fn game_runtime_loads_payload_router_exact_nonterminal_build_payload_before_sort_fields() {
        let content = ContentLoader::create_base_content().unwrap();
        let router_def = content.block_by_name("payload-router").unwrap();
        let tile_pos = point2_pack(2, 4);
        let saved = BuildingComp::new(tile_pos, router_def.base().clone(), TeamId(6));
        let item = door_build_payload_ref(&content, true);
        let conveyor = PayloadConveyorState {
            item: Some(item.clone()),
            item_rotation: 90.0,
            ..PayloadConveyorState::default()
        };
        let sorted = Some(PayloadSortKey {
            content_type: ContentType::Block.ordinal() as i8,
            id: router_def.base().id,
        });
        let rec_dir = 1;
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_conveyor_extra(
            &mut building_bytes,
            6.0,
            conveyor.item_rotation,
            Some(&item),
        )
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
            deconstructing: None,
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
    fn game_runtime_loads_terminal_payload_deconstructor_raw_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let deconstructor_def = content.block_by_name("small-deconstructor").unwrap();
        let router_id = content.block_by_name("router").unwrap().base().id;
        let tile_pos = point2_pack(3, 4);
        let saved = BuildingComp::new(tile_pos, deconstructor_def.base().clone(), TeamId(6));
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: -0.5, y: 0.75 },
            pay_rotation: 15.0,
            carried: false,
        };
        let deconstructing = PayloadRef::Block {
            block: router_id,
            version: 1,
            build_bytes: vec![0x11, 0x22],
        };
        let deconstructor = PayloadDeconstructorState {
            progress: 0.4,
            accum: Some(vec![1.0, 2.5, 0.25]),
            has_payload: false,
            has_deconstructing: true,
            deconstructing: Some(deconstructing.clone()),
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
        write_payload_ref(&mut building_bytes, Some(&deconstructing)).unwrap();

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
    fn game_runtime_roundtrips_payload_constructor_block_payload_version() {
        let content = ContentLoader::create_base_content().unwrap();
        let constructor_def = content.block_by_name("constructor").unwrap();
        let recipe = content.block_by_name("router").map(|block| block.base().id);
        let tile_pos = point2_pack(5, 4);
        let mut carried = base_only_build_payload_ref(&content, "router");
        if let PayloadRef::Block { version, .. } = &mut carried {
            *version = 7;
        }
        let common = PayloadBlockBuildState {
            payload: Some(carried),
            pay_vector: Vec2 { x: 1.0, y: 1.5 },
            pay_rotation: 45.0,
            carried: false,
        };
        let producer = BlockProducerState {
            progress: 2.25,
            has_payload: true,
            ..BlockProducerState::default()
        };
        let mut runtime = GameRuntime::default();
        runtime.state.world.resize(8, 8);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            constructor_def.base().clone(),
            TeamId(6),
        ));
        runtime.payload_runtime_states.insert(
            tile_pos,
            GameRuntimePayloadBlockState::Constructor {
                common: common.clone(),
                producer,
                recipe,
            },
        );

        let map = runtime.export_network_map_snapshot(&content);
        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(&content, &map);

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(report.block_state_bytes_ignored, 0);
        assert_eq!(
            loaded.payload_runtime_states.get(&tile_pos),
            Some(&GameRuntimePayloadBlockState::Constructor {
                common,
                producer: BlockProducerState {
                    progress: 2.25,
                    has_payload: true,
                    ..BlockProducerState::default()
                },
                recipe
            })
        );
    }

    #[test]
    fn game_runtime_advances_owned_payload_constructor_into_build_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let constructor_def = content.block_by_name("constructor").unwrap();
        let router_def = content.block_by_name("router").unwrap();
        let recipe = Some(router_def.base().id);
        let tile_pos = point2_pack(6, 4);
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: 4.0, y: -2.0 },
            pay_rotation: 90.0,
            carried: false,
        };
        let producer = BlockProducerState {
            progress: 9.5,
            ..BlockProducerState::default()
        };

        let mut runtime = GameRuntime::default();
        runtime.state.set(GameStateState::Playing);
        runtime.state.world.resize(8, 8);
        runtime.add_building(BuildingComp::new(
            tile_pos,
            constructor_def.base().clone(),
            TeamId(6),
        ));
        runtime.payload_runtime_states.insert(
            tile_pos,
            GameRuntimePayloadBlockState::Constructor {
                common,
                producer,
                recipe,
            },
        );

        let report = runtime
            .advance_owned_payload_constructors(&content, 1.0)
            .unwrap();

        assert_eq!(
            report,
            GameRuntimePayloadConstructorFrameReport {
                visited_buildings: 1,
                constructor_candidates: 1,
                updated_constructors: 1,
                produced_payloads: 1,
                missing_runtime_states: 0,
                missing_recipe_build_times: 0,
            }
        );
        assert_eq!(runtime.state.update_id, 1);
        let Some(GameRuntimePayloadBlockState::Constructor {
            common, producer, ..
        }) = runtime.payload_runtime_states.get(&tile_pos)
        else {
            panic!("constructor sidecar should remain present");
        };
        assert_eq!(producer.progress, 0.5);
        assert!(producer.has_payload);
        assert_eq!(common.pay_vector, Vec2 { x: 0.0, y: 0.0 });
        assert_eq!(common.pay_rotation, 0.0);
        let Some(PayloadRef::Block {
            block,
            version,
            build_bytes,
        }) = common.payload.as_ref()
        else {
            panic!("constructor should create a build payload");
        };
        assert_eq!(*block, router_def.base().id);
        assert_eq!(*version, 0);
        assert!(!build_bytes.is_empty());
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
    fn game_runtime_loads_drill_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let drill_def = content.block_by_name("mechanical-drill").unwrap();
        let tile_pos = point2_pack(1, 6);
        let saved = BuildingComp::new(tile_pos, drill_def.base().clone(), TeamId(2));
        let state = DrillState {
            progress: 120.0,
            warmup: 0.45,
            ..Default::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_drill_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 49, drill_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.production_runtime_states.get(&tile_pos),
            Some(&GameRuntimeProductionBlockState::Drill(state))
        );
    }

    #[test]
    fn game_runtime_skips_wall_crafter_without_java_block_specific_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let crusher_def = content.block_by_name("cliff-crusher").unwrap();
        let tile_pos = point2_pack(2, 6);
        let saved = BuildingComp::new(tile_pos, crusher_def.base().clone(), TeamId(2));
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 50, crusher_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 0);
        assert_eq!(report.block_state_parse_errors, 0);
        assert!(runtime.production_runtime_states.get(&tile_pos).is_none());
    }

    #[test]
    fn game_runtime_loads_beam_drill_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let bore_def = content.block_by_name("plasma-bore").unwrap();
        let tile_pos = point2_pack(3, 6);
        let saved = BuildingComp::new(tile_pos, bore_def.base().clone(), TeamId(2));
        let state = BeamDrillState {
            time: 44.0,
            warmup: 0.65,
            ..Default::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_beam_drill_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 51, bore_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.production_runtime_states.get(&tile_pos),
            Some(&GameRuntimeProductionBlockState::BeamDrill(state))
        );
    }

    #[test]
    fn game_runtime_loads_burst_drill_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let drill_def = content.block_by_name("impact-drill").unwrap();
        let tile_pos = point2_pack(4, 6);
        let saved = BuildingComp::new(tile_pos, drill_def.base().clone(), TeamId(2));
        let state = BurstDrillState {
            progress: 240.0,
            warmup: 0.72,
            smooth_progress: 0.99,
            invert_time: 0.5,
            ..Default::default()
        };
        let expected = BurstDrillState {
            smooth_progress: 0.0,
            invert_time: 0.0,
            ..state
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_burst_drill_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 52, drill_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.production_runtime_states.get(&tile_pos),
            Some(&GameRuntimeProductionBlockState::BurstDrill(expected))
        );
    }

    #[test]
    fn game_runtime_loads_generic_crafter_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let crafter_def = content.block_by_name("graphite-press").unwrap();
        let tile_pos = point2_pack(1, 7);
        let saved = BuildingComp::new(tile_pos, crafter_def.base().clone(), TeamId(2));
        let state = GenericCrafterState {
            progress: 0.375,
            total_progress: 9.0,
            warmup: 0.5,
        };
        let expected = GenericCrafterState {
            total_progress: 0.0,
            ..state
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_generic_crafter_state(&mut building_bytes, &state, false).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 57, crafter_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.crafting_runtime_states.get(&tile_pos),
            Some(&GameRuntimeCraftingBlockState::GenericCrafter(expected))
        );
    }

    #[test]
    fn game_runtime_loads_separator_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let separator_def = content.block_by_name("separator").unwrap();
        let tile_pos = point2_pack(2, 7);
        let saved = BuildingComp::new(tile_pos, separator_def.base().clone(), TeamId(2));
        let state = SeparatorState {
            progress: 0.8,
            total_progress: 7.0,
            warmup: 0.25,
            seed: 12_345,
        };
        let expected = SeparatorState {
            total_progress: 0.0,
            ..state
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_separator_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 58, separator_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.crafting_runtime_states.get(&tile_pos),
            Some(&GameRuntimeCraftingBlockState::Separator(expected))
        );
    }

    #[test]
    fn game_runtime_loads_heat_producer_composite_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let heat_def = content.block_by_name("oxidation-chamber").unwrap();
        let tile_pos = point2_pack(3, 7);
        let saved = BuildingComp::new(tile_pos, heat_def.base().clone(), TeamId(2));
        let crafter = GenericCrafterState {
            progress: 0.2,
            total_progress: 5.0,
            warmup: 0.6,
        };
        let expected_crafter = GenericCrafterState {
            total_progress: 0.0,
            ..crafter
        };
        let heat = HeatProducerState { heat: 3.25 };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_generic_crafter_state(&mut building_bytes, &crafter, false).unwrap();
        write_heat_producer_state(&mut building_bytes, &heat).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 59, heat_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.crafting_runtime_states.get(&tile_pos),
            Some(&GameRuntimeCraftingBlockState::HeatProducer {
                crafter: expected_crafter,
                heat
            })
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
    fn game_runtime_loads_sorter_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let sorter_def = content.block_by_name("sorter").unwrap();
        let tile_pos = point2_pack(0, 2);
        let saved = BuildingComp::new(tile_pos, sorter_def.base().clone(), TeamId(1));
        let state = SorterState { sort_item: Some(0) };
        let mut building_bytes = Vec::new();
        building_bytes.push(2);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_sorter_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 12, sorter_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.distribution_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDistributionBlockState::Sorter(state))
        );
    }

    #[test]
    fn game_runtime_loads_unloader_sort_item_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let unloader_def = content.block_by_name("unloader").unwrap();
        let tile_pos = point2_pack(1, 2);
        let saved = BuildingComp::new(tile_pos, unloader_def.base().clone(), TeamId(1));
        let sort_item = Some(1);
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_unloader_sort_item(&mut building_bytes, sort_item).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 13, unloader_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.distribution_runtime_states.get(&tile_pos),
            Some(&GameRuntimeDistributionBlockState::Unloader(
                sort_item.map(|id| id as ContentId)
            ))
        );
    }

    #[test]
    fn game_runtime_loads_stack_router_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let router_def = content.block_by_name("surge-router").unwrap();
        let tile_pos = point2_pack(2, 2);
        let saved = BuildingComp::new(tile_pos, router_def.base().clone(), TeamId(1));
        let state = DuctRouterState {
            sort_item: Some(2),
            current: None,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_duct_router_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 14, router_def.base().id, building_bytes),
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
    fn game_runtime_loads_message_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let message_def = content.block_by_name("message").unwrap();
        let tile_pos = point2_pack(0, 2);
        let saved = BuildingComp::new(tile_pos, message_def.base().clone(), TeamId(1));
        let state = MessageBlockState::new("alpha\nbeta");
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_message_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 12, message_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.logic_runtime_states.get(&tile_pos),
            Some(&GameRuntimeLogicBlockState::Message(state))
        );
    }

    #[test]
    fn game_runtime_loads_switch_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let switch_def = content.block_by_name("switch").unwrap();
        let tile_pos = point2_pack(1, 2);
        let saved = BuildingComp::new(tile_pos, switch_def.base().clone(), TeamId(1));
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_switch_enabled(&mut building_bytes, true).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 13, switch_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.logic_runtime_states.get(&tile_pos),
            Some(&GameRuntimeLogicBlockState::Switch { enabled: true })
        );
    }

    #[test]
    fn game_runtime_loads_logic_display_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let display_def = content.block_by_name("logic-display").unwrap();
        let tile_pos = point2_pack(2, 2);
        let saved = BuildingComp::new(tile_pos, display_def.base().clone(), TeamId(1));
        let state =
            LogicDisplayState::with_transform([1.0, 0.0, 8.0, 0.0, 1.0, -4.0, 0.0, 0.0, 1.0]);
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_logic_display_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 18, display_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.logic_runtime_states.get(&tile_pos),
            Some(&GameRuntimeLogicBlockState::Display(state))
        );
    }

    #[test]
    fn game_runtime_loads_tile_logic_display_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let display_def = content.block_by_name("tile-logic-display").unwrap();
        let tile_pos = point2_pack(3, 2);
        let saved = BuildingComp::new(tile_pos, display_def.base().clone(), TeamId(1));
        let state = LogicDisplayState::default();
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_logic_display_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 19, display_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.logic_runtime_states.get(&tile_pos),
            Some(&GameRuntimeLogicBlockState::Display(state))
        );
    }

    #[test]
    fn game_runtime_loads_memory_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let memory_def = content.block_by_name("memory-cell").unwrap();
        let tile_pos = point2_pack(4, 2);
        let saved = BuildingComp::new(tile_pos, memory_def.base().clone(), TeamId(1));
        let mut state = MemoryBlockState::new(64);
        state.memory[0] = 7.0;
        state.memory[5] = -3.5;
        state.memory[63] = 99.25;
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_memory_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 20, memory_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.logic_runtime_states.get(&tile_pos),
            Some(&GameRuntimeLogicBlockState::Memory(state))
        );
    }

    #[test]
    fn game_runtime_loads_canvas_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let canvas_def = content.block_by_name("canvas").unwrap();
        let tile_pos = point2_pack(5, 2);
        let saved = BuildingComp::new(tile_pos, canvas_def.base().clone(), TeamId(1));
        let expected_len = match canvas_def {
            BlockDef::Logic(logic) => logic.canvas_data_bytes as usize,
            _ => unreachable!(),
        };
        let mut data = vec![0; expected_len];
        data[0] = 0b0101_1010;
        data[expected_len - 1] = 0b1010_0101;
        let state = CanvasBlockState::from_data(data);
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_canvas_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 21, canvas_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.logic_runtime_states.get(&tile_pos),
            Some(&GameRuntimeLogicBlockState::Canvas(state))
        );
    }

    #[test]
    fn game_runtime_loads_processor_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let processor_def = content.block_by_name("micro-processor").unwrap();
        let tile_pos = point2_pack(6, 2);
        let saved = BuildingComp::new(tile_pos, processor_def.base().clone(), TeamId(1));
        let config =
            LogicConfig::from_code(b"set counter 1", vec![LogicLink::new(1, 0, "cell1", false)]);
        let mut state = LogicProcessorState::from_config(config).unwrap();
        state.variables = vec![LogicProcessorVariableState::new(
            "counter",
            TypeValue::Double(1.0),
        )];
        state.tag = Some("loop".into());
        state.icon_tag = 'L' as u16;
        state.waits = vec![LogicProcessorWaitState::new(0, 0.25)];
        state.accumulator = 0.75;

        let mut building_bytes = Vec::new();
        building_bytes.push(4);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_logic_processor_state(&mut building_bytes, &state, 4, false, 40).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 22, processor_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.logic_runtime_states.get(&tile_pos),
            Some(&GameRuntimeLogicBlockState::Processor(state))
        );
    }

    #[test]
    fn game_runtime_loads_processor_revision_zero_legacy_code_and_links() {
        let content = ContentLoader::create_base_content().unwrap();
        let processor_def = content.block_by_name("micro-processor").unwrap();
        let tile_pos = point2_pack(6, 2);
        let saved = BuildingComp::new(tile_pos, processor_def.base().clone(), TeamId(1));
        let state = LogicProcessorState {
            legacy_code: Some("end".into()),
            legacy_link_positions: vec![point2_pack(1, 2), point2_pack(3, 4)],
            ..LogicProcessorState::default()
        };

        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        building_bytes.extend_from_slice(&[0, 3, b'e', b'n', b'd']);
        building_bytes.extend_from_slice(&(state.legacy_link_positions.len() as i16).to_be_bytes());
        for pos in &state.legacy_link_positions {
            building_bytes.extend_from_slice(&pos.to_be_bytes());
        }
        building_bytes.extend_from_slice(&0i32.to_be_bytes());
        building_bytes.extend_from_slice(&0i32.to_be_bytes());

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 22, processor_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.logic_runtime_states.get(&tile_pos),
            Some(&GameRuntimeLogicBlockState::Processor(state))
        );
    }

    #[test]
    fn game_runtime_loads_launch_pad_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let launch_def = content.block_by_name("launch-pad").unwrap();
        let tile_pos = point2_pack(0, 3);
        let saved = BuildingComp::new(tile_pos, launch_def.base().clone(), TeamId(1));
        let state = LaunchPadState {
            launch_counter: 600.5,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_launch_pad_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 24, launch_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.campaign_runtime_states.get(&tile_pos),
            Some(&GameRuntimeCampaignBlockState::LaunchPad(state))
        );
    }

    #[test]
    fn game_runtime_loads_landing_pad_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let landing_def = content.block_by_name("landing-pad").unwrap();
        let copper = content
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let graphite = content
            .item_by_name("graphite")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let tile_pos = point2_pack(1, 3);
        let saved = BuildingComp::new(tile_pos, landing_def.base().clone(), TeamId(1));
        let state = LandingPadState {
            config: Some(copper),
            priority: 123,
            cooldown: 0.75,
            arriving: Some(graphite),
            arriving_timer: 0.5,
            liquid_removed: 750.0,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_landing_pad_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 25, landing_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.campaign_runtime_states.get(&tile_pos),
            Some(&GameRuntimeCampaignBlockState::LandingPad(state))
        );
    }

    #[test]
    fn game_runtime_loads_accelerator_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let accelerator_def = content.block_by_name("interplanetary-accelerator").unwrap();
        let tile_pos = point2_pack(2, 3);
        let saved = BuildingComp::new(tile_pos, accelerator_def.base().clone(), TeamId(1));
        let state = AcceleratorState {
            progress: 0.9,
            launching: false,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_accelerator_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 26, accelerator_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.campaign_runtime_states.get(&tile_pos),
            Some(&GameRuntimeCampaignBlockState::Accelerator(state))
        );
    }

    #[test]
    fn game_runtime_loads_legacy_command_center_extra_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let command_def = content.block_by_name("command-center").unwrap();
        let tile_pos = point2_pack(3, 3);
        let saved = BuildingComp::new(tile_pos, command_def.base().clone(), TeamId(1));
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_legacy_command_center_extra(&mut building_bytes).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 27, command_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.legacy_runtime_states.get(&tile_pos),
            Some(&GameRuntimeLegacyBlockState::CommandCenter(0))
        );
    }

    #[test]
    fn game_runtime_loads_legacy_mech_pad_extra_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let mech_def = content.block_by_name("legacy-mech-pad").unwrap();
        let tile_pos = point2_pack(4, 3);
        let saved = BuildingComp::new(tile_pos, mech_def.base().clone(), TeamId(1));
        let values = [1.0, 2.5, -3.0];
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_legacy_mech_pad_extra(&mut building_bytes, values).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 28, mech_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.legacy_runtime_states.get(&tile_pos),
            Some(&GameRuntimeLegacyBlockState::MechPad(values))
        );
    }

    #[test]
    fn game_runtime_loads_legacy_unit_factory_extra_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let factory_def = content.block_by_name("legacy-unit-factory").unwrap();
        let tile_pos = point2_pack(5, 3);
        let saved = BuildingComp::new(tile_pos, factory_def.base().clone(), TeamId(1));
        let extra = LegacyUnitFactoryExtra {
            build_time: 120.0,
            spawn_count: Some(3),
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_legacy_unit_factory_extra(&mut building_bytes, 0, &extra).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 29, factory_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.legacy_runtime_states.get(&tile_pos),
            Some(&GameRuntimeLegacyBlockState::UnitFactory(extra))
        );
    }

    #[test]
    fn game_runtime_loads_item_source_config_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let source_def = content.block_by_name("item-source").unwrap();
        let copper = content
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let tile_pos = point2_pack(6, 3);
        let saved = BuildingComp::new(tile_pos, source_def.base().clone(), TeamId(1));
        let state = ItemSourceState {
            output_item: Some(copper),
            ..ItemSourceState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_item_source_config(&mut building_bytes, state.output_item).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 30, source_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.sandbox_runtime_states.get(&tile_pos),
            Some(&GameRuntimeSandboxBlockState::ItemSource(state))
        );
    }

    #[test]
    fn game_runtime_loads_liquid_source_config_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let source_def = content.block_by_name("liquid-source").unwrap();
        let water = content
            .liquid_by_name("water")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let tile_pos = point2_pack(7, 3);
        let saved = BuildingComp::new(tile_pos, source_def.base().clone(), TeamId(1));
        let state = LiquidSourceState {
            source: Some(water),
            ..LiquidSourceState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_liquid_source_config(&mut building_bytes, state.source).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(8, 8, 31, source_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.sandbox_runtime_states.get(&tile_pos),
            Some(&GameRuntimeSandboxBlockState::LiquidSource(state))
        );
    }

    #[test]
    fn game_runtime_loads_unit_factory_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let factory_def = content.block_by_name("ground-factory").unwrap();
        let tile_pos = point2_pack(4, 0);
        let saved = BuildingComp::new(tile_pos, factory_def.base().clone(), TeamId(1));
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: 1.0, y: -2.0 },
            pay_rotation: 90.0,
            carried: false,
        };
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
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
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
            Some(&GameRuntimeUnitBlockState::Factory {
                common,
                factory: state
            })
        );
    }

    #[test]
    fn game_runtime_loads_unit_factory_common_build_payload_before_factory_fields() {
        let content = ContentLoader::create_base_content().unwrap();
        let factory_def = content.block_by_name("ground-factory").unwrap();
        let tile_pos = point2_pack(4, 0);
        let saved = BuildingComp::new(tile_pos, factory_def.base().clone(), TeamId(1));
        let payload = door_build_payload_ref(&content, true);
        let common = PayloadBlockBuildState {
            payload: Some(payload),
            pay_vector: Vec2 { x: 1.0, y: -2.0 },
            pay_rotation: 90.0,
            carried: false,
        };
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
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
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
            Some(&GameRuntimeUnitBlockState::Factory {
                common,
                factory: state
            })
        );
    }

    #[test]
    fn game_runtime_loads_unit_factory_common_no_state_build_payload_before_factory_fields() {
        let content = ContentLoader::create_base_content().unwrap();
        let factory_def = content.block_by_name("ground-factory").unwrap();
        let tile_pos = point2_pack(4, 0);
        let saved = BuildingComp::new(tile_pos, factory_def.base().clone(), TeamId(1));
        let payload = base_only_build_payload_ref(&content, "router");
        let common = PayloadBlockBuildState {
            payload: Some(payload),
            pay_vector: Vec2 { x: 1.0, y: -2.0 },
            pay_rotation: 90.0,
            carried: false,
        };
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
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
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
            Some(&GameRuntimeUnitBlockState::Factory {
                common,
                factory: state
            })
        );
    }

    #[test]
    fn game_runtime_loads_unit_factory_common_unit_payload_before_factory_fields() {
        let content = ContentLoader::create_base_content().unwrap();
        let factory_def = content.block_by_name("ground-factory").unwrap();
        let tile_pos = point2_pack(4, 0);
        let saved = BuildingComp::new(tile_pos, factory_def.base().clone(), TeamId(1));
        let payload = flare_unit_payload_ref(&content);
        let common = PayloadBlockBuildState {
            payload: Some(payload),
            pay_vector: Vec2 { x: 1.0, y: -2.0 },
            pay_rotation: 90.0,
            carried: false,
        };
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
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
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
            Some(&GameRuntimeUnitBlockState::Factory {
                common,
                factory: state
            })
        );
    }

    #[test]
    fn game_runtime_loads_unit_factory_common_nested_payload_conveyor_without_swallowing_factory_fields(
    ) {
        let content = ContentLoader::create_base_content().unwrap();
        let factory_def = content.block_by_name("ground-factory").unwrap();
        let tile_pos = point2_pack(4, 0);
        let saved = BuildingComp::new(tile_pos, factory_def.base().clone(), TeamId(1));
        let inner = door_build_payload_ref(&content, true);
        let payload = payload_conveyor_build_payload_ref(&content, &inner);
        let common = PayloadBlockBuildState {
            payload: Some(payload),
            pay_vector: Vec2 { x: 1.0, y: -2.0 },
            pay_rotation: 90.0,
            carried: false,
        };
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
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
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
            Some(&GameRuntimeUnitBlockState::Factory {
                common,
                factory: state
            })
        );
    }

    #[test]
    fn game_runtime_loads_reconstructor_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let reconstructor_def = content.block_by_name("additive-reconstructor").unwrap();
        let tile_pos = point2_pack(3, 0);
        let saved = BuildingComp::new(tile_pos, reconstructor_def.base().clone(), TeamId(1));
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: 2.0, y: -3.0 },
            pay_rotation: 180.0,
            carried: false,
        };
        let state = ReconstructorState {
            base: crate::mindustry::world::blocks::units::UnitBlockState {
                progress: 11.0,
                ..Default::default()
            },
            command_pos: Some(IoVec2 { x: 8.0, y: 16.0 }),
            command_id: Some(3),
            constructing: false,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(3);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
        write_reconstructor_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 3, reconstructor_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.unit_runtime_states.get(&tile_pos),
            Some(&GameRuntimeUnitBlockState::Reconstructor {
                common,
                reconstructor: state
            })
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
    fn game_runtime_loads_unit_assembler_state_from_network_map_building_payload() {
        let content = ContentLoader::create_base_content().unwrap();
        let assembler_def = content.block_by_name("tank-assembler").unwrap();
        let wall_def = content.block_by_name("tungsten-wall").unwrap();
        let tile_pos = point2_pack(0, 1);
        let saved = BuildingComp::new(tile_pos, assembler_def.base().clone(), TeamId(1));
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: 0.25, y: -0.5 },
            pay_rotation: 90.0,
            carried: false,
        };
        let mut blocks = PayloadSeq::new();
        blocks.add(PayloadKey::new(ContentType::Block, wall_def.base().id), 3);
        let state = UnitAssemblerState {
            progress: 0.6,
            read_unit_ids: vec![101, 102, 103],
            blocks,
            command_pos: Some(IoVec2 { x: 64.0, y: 96.0 }),
            ..UnitAssemblerState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
        write_unit_assembler_state(&mut building_bytes, &state).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 6, assembler_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.unit_runtime_states.get(&tile_pos),
            Some(&GameRuntimeUnitBlockState::Assembler {
                common,
                assembler: state
            })
        );
    }

    #[test]
    fn game_runtime_loads_unit_assembler_state_from_legacy_block_only_payload_seq() {
        let content = ContentLoader::create_base_content().unwrap();
        let assembler_def = content.block_by_name("tank-assembler").unwrap();
        let wall_def = content.block_by_name("tungsten-wall").unwrap();
        let large_wall_def = content.block_by_name("tungsten-wall-large").unwrap();
        let tile_pos = point2_pack(0, 1);
        let saved = BuildingComp::new(tile_pos, assembler_def.base().clone(), TeamId(1));
        let common = PayloadBlockBuildState {
            payload: None,
            pay_vector: Vec2 { x: 0.25, y: -0.5 },
            pay_rotation: 90.0,
            carried: false,
        };
        let mut blocks = PayloadSeq::new();
        blocks.add(PayloadKey::new(ContentType::Block, wall_def.base().id), 3);
        blocks.add(
            PayloadKey::new(ContentType::Block, large_wall_def.base().id),
            2,
        );
        let state = UnitAssemblerState {
            progress: 0.6,
            read_unit_ids: vec![101, 102],
            blocks,
            command_pos: Some(IoVec2 { x: 64.0, y: 96.0 }),
            ..UnitAssemblerState::default()
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(1);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();
        building_bytes.extend_from_slice(&state.progress.to_bits().to_be_bytes());
        building_bytes.push(state.read_unit_ids.len() as u8);
        for id in &state.read_unit_ids {
            building_bytes.extend_from_slice(&id.to_be_bytes());
        }
        building_bytes.extend_from_slice(&(state.blocks.len() as i16).to_be_bytes());
        for (key, amount) in state.blocks.entries() {
            assert_eq!(key.content_type, ContentType::Block);
            building_bytes.extend_from_slice(&(key.id as i16).to_be_bytes());
            building_bytes.extend_from_slice(&amount.to_be_bytes());
        }
        let command_pos = state.command_pos.unwrap();
        building_bytes.extend_from_slice(&command_pos.x.to_bits().to_be_bytes());
        building_bytes.extend_from_slice(&command_pos.y.to_bits().to_be_bytes());

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 6, assembler_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.unit_runtime_states.get(&tile_pos),
            Some(&GameRuntimeUnitBlockState::Assembler {
                common,
                assembler: state
            })
        );
    }

    #[test]
    fn game_runtime_loads_unit_assembler_module_common_payload_state_from_network_map_building_payload(
    ) {
        let content = ContentLoader::create_base_content().unwrap();
        let module_def = content.block_by_name("basic-assembler-module").unwrap();
        let tile_pos = point2_pack(1, 1);
        let saved = BuildingComp::new(tile_pos, module_def.base().clone(), TeamId(1));
        let common = PayloadBlockBuildState {
            payload: Some(PayloadRef::Unit {
                class_id: 7,
                unit_bytes: vec![0x55, 0x66],
            }),
            pay_vector: Vec2 { x: -0.25, y: 0.5 },
            pay_rotation: 180.0,
            carried: false,
        };
        let mut building_bytes = Vec::new();
        building_bytes.push(0);
        saved.write_base(&mut building_bytes, false).unwrap();
        write_payload_block_build_common(&mut building_bytes, &common).unwrap();

        let mut runtime = GameRuntime::default();
        let report = runtime.load_network_map_with_buildings(
            &content,
            &single_building_network_map(6, 6, 7, module_def.base().id, building_bytes),
        );

        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.block_states_added, 1);
        assert_eq!(report.block_state_parse_errors, 0);
        assert_eq!(
            runtime.unit_runtime_states.get(&tile_pos),
            Some(&GameRuntimeUnitBlockState::AssemblerModule(common))
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
