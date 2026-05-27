pub mod server_control;

use mindustry_core::mindustry::content::blocks::{
    BlockDef, DistributionBlockKind, EffectBlockKind, StorageBlockKind,
};
use mindustry_core::mindustry::core::game_runtime::{
    GameRuntimeDistributionBlockState, GameRuntimePayloadBlockState,
    GameRuntimePowerNodeConfigResult, GameRuntimeUnitBlockState,
    GameRuntimeUnitCargoUnloadConfigureResult,
};
use mindustry_core::mindustry::core::net_server::{
    PlayerPreviewPlanSource, PLAN_PREVIEW_SYNC_INTERVAL_MS,
};
use mindustry_core::mindustry::core::{
    content_loader::ContentLoader, GameRuntime, GameRuntimeNetworkContext,
    GameRuntimeOwnedEffectResources, GameRuntimeOwnedFrameReport,
    GameRuntimeOwnedItemTransportFrameReport, GameRuntimeOwnedPayloadFrameReport, NetServer, World,
};
use mindustry_core::mindustry::ctype::ContentType;
use mindustry_core::mindustry::ctype::{Content, ContentId};
use mindustry_core::mindustry::entities::{
    bullet::BulletType,
    comp::{
        BuildingComp, BuildingTetherAction, BuildingTetherComp, BuildingTetherRef, BulletComp,
        CargoAiRuntimeState, PayloadKind, PayloadState, PlayerComp, PlayerUnitState, UnitComp,
        UnitControllerState,
    },
    entity_class_id, standard_effect_id, units_can_create, EnergyFieldAction, EnergyFieldTarget,
    FireCreateResult, FireRules, FireTile, Fires, LiquidExplodeDepositPlan, PuddleDepositContext,
    PuddleLiquidInfo, PuddleTileView, Puddles, UnitCapRules, UnitCapTeam, UnitCapType,
    UnitSpawnAbility, FIRE_CLASS_ID, PUDDLE_CLASS_ID,
};
use mindustry_core::mindustry::game::{vanilla_teams, Trigger};
use mindustry_core::mindustry::input::{
    drop_item, payload_dropped, picked_build_payload, picked_unit_payload, request_build_payload,
    request_drop_payload, request_item, request_unit_payload, take_items, transfer_inventory,
    transfer_item_to, unit_entered_payload, BuildPayloadPickupKind, DropItemContext,
    DropItemOutcome, PayloadDroppedOutcome, PickedBuildPayloadOutcome, PickedUnitPayloadOutcome,
    RequestBuildPayloadContext, RequestBuildPayloadOutcome, RequestDropPayloadContext,
    RequestDropPayloadOutcome, RequestItemContext, RequestItemOutcome, RequestUnitPayloadContext,
    RequestUnitPayloadOutcome, TakeItemsOutcome, TransferInventoryContext,
    TransferInventoryOutcome, TransferItemToOutcome,
};
use mindustry_core::mindustry::io::{
    type_io, BuildPlanWire, ContentPatchSet, EntityRef, TeamId, TypeValue, UnitRef,
};
use mindustry_core::mindustry::net::{
    write_world_data, ArcNetProvider, AssemblerDroneSpawnedCallPacket,
    AssemblerUnitSpawnedCallPacket, ClientPlanSnapshotCallPacket, CommandBuildingCallPacket,
    DropItemCallPacket, EffectCallPacket, EffectCallPacket2, EntitySnapshotCallPacket,
    HiddenSnapshotCallPacket, LandingPadLandedCallPacket, Net, NetConnection, NetworkPlayerData,
    NetworkWorldData, PacketKind, PickedBuildPayloadCallPacket, PickedUnitPayloadCallPacket,
    ProviderEvent, RequestBuildPayloadCallPacket, RequestDropPayloadCallPacket,
    RequestItemCallPacket, RequestUnitPayloadCallPacket, TileConfigCallPacket,
    TransferInventoryCallPacket, UnitBlockSpawnCallPacket, UnitDespawnCallPacket,
    UnitSpawnCallPacket, UnitTetherBlockSpawnedCallPacket,
};
use mindustry_core::mindustry::r#type::UnitType;
use mindustry_core::mindustry::vars::{
    AppContext, RuntimeMode, ITEM_TRANSFER_RANGE, MAX_PLAYER_PREVIEW_PLANS, TILE_SIZE,
};
use mindustry_core::mindustry::world::blocks::autotiler_direction;
use mindustry_core::mindustry::world::blocks::defense::EffectBlockFrameBatchReport;
use mindustry_core::mindustry::world::blocks::payloads::{payload_ref_sort_key, PayloadRef};
use mindustry_core::mindustry::world::blocks::units::{
    unit_assembler_drone_spawned, unit_assembler_drone_target,
};
use mindustry_core::mindustry::world::meta::BuildVisibility;
use mindustry_core::mindustry::world::{
    footprint_tiles, point2_pack, ORTHOGONAL_NEIGHBORS, ORTHOGONAL_WITH_CENTER_NEIGHBORS,
};
use mindustry_core::mindustry::UPSTREAM_BASELINE;
use server_control::ServerControl;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const SERVER_RUNTIME_UNIT_ID_START: i32 = 1_000_000;
const CARGO_AI_EMPTY_WAIT_TIME: f32 = 60.0 * 2.0;
const CARGO_AI_DROP_SPACING: f32 = 60.0 * 1.5;
const CARGO_AI_RETARGET_INTERVAL: f32 = 40.0;
const CARGO_AI_TRANSFER_RANGE: f32 = 20.0;
const CARGO_AI_MOVE_RANGE: f32 = 6.0;
const CARGO_AI_MOVE_SMOOTHING: f32 = 20.0;
const SERVER_FIRE_ENTITY_ID_BASE: i32 = -1_500_000_000;

fn server_fire_entity_id(x: i32, y: i32) -> i32 {
    SERVER_FIRE_ENTITY_ID_BASE.saturating_add(point2_pack(x, y))
}

#[derive(Debug, Clone)]
pub struct ServerLauncher {
    pub context: AppContext,
    pub args: Vec<String>,
    pub control: ServerControl,
    pub runtime: GameRuntime,
    pub content_loader: ContentLoader,
    pub last_runtime_effect_report: Option<EffectBlockFrameBatchReport>,
    pub last_runtime_item_transport_report: Option<GameRuntimeOwnedItemTransportFrameReport>,
    pub last_runtime_payload_report: Option<GameRuntimeOwnedPayloadFrameReport>,
    pub last_runtime_power_node_config_result: Option<GameRuntimePowerNodeConfigResult>,
    pub runtime_power_node_config_packets_seen: usize,
    pub runtime_power_node_config_packets_changed: usize,
    pub last_runtime_unit_cargo_unload_config_result:
        Option<GameRuntimeUnitCargoUnloadConfigureResult>,
    pub runtime_unit_cargo_unload_config_packets_seen: usize,
    pub runtime_unit_cargo_unload_config_packets_changed: usize,
    pub runtime_unit_cargo_unload_config_packets_forwarded: usize,
    pub server_preview_players: BTreeMap<i32, PlayerComp>,
    pub server_units: BTreeMap<i32, UnitComp>,
    pub runtime_request_item_packets_seen: usize,
    pub runtime_request_item_packets_accepted: usize,
    pub runtime_request_item_packets_rejected: usize,
    pub runtime_take_items_packets_sent: usize,
    pub runtime_transfer_item_effect_packets_sent: usize,
    pub last_runtime_request_item_outcome: Option<RequestItemOutcome>,
    pub last_runtime_take_items_outcome: Option<TakeItemsOutcome>,
    pub runtime_transfer_inventory_packets_seen: usize,
    pub runtime_transfer_inventory_packets_accepted: usize,
    pub runtime_transfer_inventory_packets_rejected: usize,
    pub runtime_transfer_item_to_packets_sent: usize,
    pub last_runtime_transfer_inventory_outcome: Option<TransferInventoryOutcome>,
    pub last_runtime_transfer_item_to_outcome: Option<TransferItemToOutcome>,
    pub runtime_request_drop_payload_packets_seen: usize,
    pub runtime_request_drop_payload_packets_accepted: usize,
    pub runtime_request_drop_payload_packets_rejected: usize,
    pub runtime_payload_dropped_packets_sent: usize,
    pub last_runtime_request_drop_payload_outcome: Option<RequestDropPayloadOutcome>,
    pub last_runtime_payload_dropped_outcome: Option<PayloadDroppedOutcome>,
    pub runtime_request_build_payload_packets_seen: usize,
    pub runtime_request_build_payload_packets_accepted: usize,
    pub runtime_request_build_payload_packets_rejected: usize,
    pub runtime_picked_build_payload_packets_sent: usize,
    pub last_runtime_request_build_payload_outcome: Option<RequestBuildPayloadOutcome>,
    pub last_runtime_picked_build_payload_outcome: Option<PickedBuildPayloadOutcome>,
    pub runtime_request_unit_payload_packets_seen: usize,
    pub runtime_request_unit_payload_packets_accepted: usize,
    pub runtime_request_unit_payload_packets_rejected: usize,
    pub runtime_picked_unit_payload_packets_sent: usize,
    pub last_runtime_request_unit_payload_outcome: Option<RequestUnitPayloadOutcome>,
    pub last_runtime_picked_unit_payload_outcome: Option<PickedUnitPayloadOutcome>,
    pub runtime_drop_item_packets_seen: usize,
    pub runtime_drop_item_packets_accepted: usize,
    pub runtime_drop_item_packets_rejected: usize,
    pub runtime_drop_item_packets_sent: usize,
    pub last_runtime_drop_item_outcome: Option<DropItemOutcome>,
    pub server_preview_plan_packets_applied: usize,
    pub next_server_preview_broadcast_at: Option<Instant>,
    pub server_preview_broadcasts_sent: usize,
    pub next_network_event_index: usize,
    pub net_server: NetServer,
    pub network_error: Option<String>,
}

impl ServerLauncher {
    pub fn new(args: Vec<String>) -> Self {
        let mut context = AppContext::server("config");
        if let Some(port) = parse_port_arg(&args) {
            context.port = port;
        }

        let mut runtime = GameRuntime::default();
        runtime.set_network_context(GameRuntimeNetworkContext::server());

        Self {
            context,
            control: ServerControl::new(args.clone()),
            runtime,
            content_loader: ContentLoader::create_base_content_or_panic(),
            last_runtime_effect_report: None,
            last_runtime_item_transport_report: None,
            last_runtime_payload_report: None,
            last_runtime_power_node_config_result: None,
            runtime_power_node_config_packets_seen: 0,
            runtime_power_node_config_packets_changed: 0,
            last_runtime_unit_cargo_unload_config_result: None,
            runtime_unit_cargo_unload_config_packets_seen: 0,
            runtime_unit_cargo_unload_config_packets_changed: 0,
            runtime_unit_cargo_unload_config_packets_forwarded: 0,
            server_preview_players: BTreeMap::new(),
            server_units: BTreeMap::new(),
            runtime_request_item_packets_seen: 0,
            runtime_request_item_packets_accepted: 0,
            runtime_request_item_packets_rejected: 0,
            runtime_take_items_packets_sent: 0,
            runtime_transfer_item_effect_packets_sent: 0,
            last_runtime_request_item_outcome: None,
            last_runtime_take_items_outcome: None,
            runtime_transfer_inventory_packets_seen: 0,
            runtime_transfer_inventory_packets_accepted: 0,
            runtime_transfer_inventory_packets_rejected: 0,
            runtime_transfer_item_to_packets_sent: 0,
            last_runtime_transfer_inventory_outcome: None,
            last_runtime_transfer_item_to_outcome: None,
            runtime_request_drop_payload_packets_seen: 0,
            runtime_request_drop_payload_packets_accepted: 0,
            runtime_request_drop_payload_packets_rejected: 0,
            runtime_payload_dropped_packets_sent: 0,
            last_runtime_request_drop_payload_outcome: None,
            last_runtime_payload_dropped_outcome: None,
            runtime_request_build_payload_packets_seen: 0,
            runtime_request_build_payload_packets_accepted: 0,
            runtime_request_build_payload_packets_rejected: 0,
            runtime_picked_build_payload_packets_sent: 0,
            last_runtime_request_build_payload_outcome: None,
            last_runtime_picked_build_payload_outcome: None,
            runtime_request_unit_payload_packets_seen: 0,
            runtime_request_unit_payload_packets_accepted: 0,
            runtime_request_unit_payload_packets_rejected: 0,
            runtime_picked_unit_payload_packets_sent: 0,
            last_runtime_request_unit_payload_outcome: None,
            last_runtime_picked_unit_payload_outcome: None,
            runtime_drop_item_packets_seen: 0,
            runtime_drop_item_packets_accepted: 0,
            runtime_drop_item_packets_rejected: 0,
            runtime_drop_item_packets_sent: 0,
            last_runtime_drop_item_outcome: None,
            server_preview_plan_packets_applied: 0,
            next_server_preview_broadcast_at: Some(
                Instant::now() + Duration::from_millis(PLAN_PREVIEW_SYNC_INTERVAL_MS as u64),
            ),
            server_preview_broadcasts_sent: 0,
            next_network_event_index: 0,
            net_server: NetServer::new(Net::new(Box::new(ArcNetProvider::new()))),
            network_error: None,
            args,
        }
    }

    pub fn init(&mut self) {
        self.context.flags.mode = RuntimeMode::Server;
        self.context.flags.headless = true;
        self.control.enable_stdin();
        self.control.enable_socket();
        self.control.enable_autosave();
        self.control.enable_logging();
        self.open_network();
    }

    pub fn open_network(&mut self) {
        if self.net_server.is_active() {
            self.network_error = None;
            return;
        }

        match self.net_server.open(self.context.port) {
            Ok(()) => self.network_error = None,
            Err(error) => self.network_error = Some(error.to_string()),
        }
    }

    pub fn close_network(&mut self) {
        self.net_server.close();
    }

    pub fn update(&mut self) {
        self.net_server.update();
        self.apply_new_network_server_events();
        if let Err(error) =
            self.broadcast_server_preview_plans_if_due(Instant::now(), Self::current_millis())
        {
            self.network_error = Some(error.to_string());
        }
        let _ = self.flush_pending_world_data();
        if let Err(error) = self.tick_server_unit_entered_payloads() {
            self.network_error = Some(error.to_string());
        }
        self.tick_runtime_unit_assembler_ai();
        let unit_cargo_loader_spawn_candidates =
            self.runtime_unit_cargo_loader_spawn_candidate_tiles();
        if let Some(report) = self.update_runtime_owned_blocks(1.0 / 60.0) {
            if report.item_transport.unit_cargo_loader_built_units > 0 {
                if let Err(error) =
                    self.apply_runtime_unit_cargo_loader_spawns(&unit_cargo_loader_spawn_candidates)
                {
                    self.network_error = Some(error.to_string());
                }
            }
            let unit_block_spawn_tiles: Vec<i32> = report
                .unit
                .factory
                .spawned_tiles
                .iter()
                .chain(report.unit.reconstructor.spawned_tiles.iter())
                .copied()
                .collect();
            if let Err(error) = self.broadcast_runtime_unit_block_spawns(&unit_block_spawn_tiles) {
                self.network_error = Some(error.to_string());
            }
            if let Err(error) = self.apply_runtime_unit_assembler_drone_spawns(
                &report.unit.assembler.spawned_drone_tiles,
            ) {
                self.network_error = Some(error.to_string());
            }
            if let Err(error) =
                self.broadcast_runtime_assembler_unit_spawns(&report.unit.assembler.spawned_tiles)
            {
                self.network_error = Some(error.to_string());
            }
            if let Err(error) =
                self.broadcast_runtime_landing_pad_landed(&report.campaign.landing_pad.landed_tiles)
            {
                self.network_error = Some(error.to_string());
            }
            if let Err(error) =
                self.apply_runtime_unit_assembler_spawns(&report.unit.assembler.spawned_tiles)
            {
                self.network_error = Some(error.to_string());
            }
            if let Err(error) = self.tick_server_unit_spawn_abilities(1.0) {
                self.network_error = Some(error.to_string());
            }
            self.tick_server_regen_abilities(1.0);
            if let Err(error) = self.tick_server_liquid_regen_abilities(1.0) {
                self.network_error = Some(error.to_string());
            }
            if let Err(error) = self.tick_server_puddles(1.0) {
                self.network_error = Some(error.to_string());
            }
            self.tick_server_force_field_abilities(1.0);
            self.tick_server_shield_arc_abilities(1.0);
            self.tick_server_shield_regen_field_abilities(1.0);
            self.tick_server_repair_field_abilities(1.0);
            self.tick_server_energy_field_abilities(1.0);
            self.tick_server_status_field_abilities(1.0);
            self.tick_server_suppression_field_abilities(1.0);
            if let Err(error) = self.apply_server_unit_death_abilities() {
                self.network_error = Some(error.to_string());
            }
            if let Err(error) = self.tick_runtime_unit_cargo_ai() {
                self.network_error = Some(error.to_string());
            }
            if let Err(error) = self.broadcast_server_unit_entity_snapshots() {
                self.network_error = Some(error.to_string());
            }
            self.last_runtime_item_transport_report = Some(report.item_transport);
            self.last_runtime_payload_report = Some(report.payload);
            self.last_runtime_effect_report = Some(report.effect);
        } else {
            self.last_runtime_item_transport_report = None;
            self.last_runtime_payload_report = None;
            self.last_runtime_effect_report = None;
        }
    }

    pub fn flush_pending_world_data(&mut self) -> io::Result<usize> {
        let template = self.network_world_data_template();
        self.net_server.send_pending_world_data(|connection_id| {
            let mut world_data = template.clone();
            world_data.player_id = connection_id;
            world_data.player = Some(NetworkPlayerData::bootstrap());
            write_world_data(&world_data).expect("runtime world data payload should be encodable")
        })
    }

    pub fn apply_new_network_server_events(&mut self) -> usize {
        let events = {
            let state = self.net_server.state();
            let state = state.lock().expect("NetServerState mutex poisoned");
            let start = self.next_network_event_index.min(state.events.len());
            let events = state.events[start..].to_vec();
            self.next_network_event_index = state.events.len();
            events
        };

        let mut changed = 0;
        for event in events {
            match event {
                ProviderEvent::ServerPacket {
                    connection_id,
                    packet: PacketKind::TileConfigCallPacket(packet),
                } => match self.apply_server_tile_config_packet(connection_id, &packet) {
                    Ok(true) => changed += 1,
                    Ok(false) => {}
                    Err(error) => {
                        self.network_error = Some(error.to_string());
                    }
                },
                ProviderEvent::ServerPacket {
                    connection_id,
                    packet: PacketKind::CommandBuildingCallPacket(packet),
                } => match self.apply_server_command_building_packet(connection_id, &packet) {
                    Ok(true) => changed += 1,
                    Ok(false) => {}
                    Err(error) => {
                        self.network_error = Some(error.to_string());
                    }
                },
                ProviderEvent::ServerPacket {
                    connection_id,
                    packet: PacketKind::RequestItemCallPacket(packet),
                } => {
                    self.runtime_request_item_packets_seen += 1;
                    match self.apply_server_request_item_packet(connection_id, &packet) {
                        Ok(true) => {
                            changed += 1;
                            self.runtime_request_item_packets_accepted += 1;
                        }
                        Ok(false) => {
                            self.runtime_request_item_packets_rejected += 1;
                        }
                        Err(error) => {
                            self.network_error = Some(error.to_string());
                            self.runtime_request_item_packets_rejected += 1;
                            continue;
                        }
                    }
                }
                ProviderEvent::ServerPacket {
                    connection_id,
                    packet: PacketKind::DropItemCallPacket(packet),
                } => {
                    self.runtime_drop_item_packets_seen += 1;
                    match self.apply_server_drop_item_packet(connection_id, &packet) {
                        Ok(true) => {
                            changed += 1;
                            self.runtime_drop_item_packets_accepted += 1;
                        }
                        Ok(false) => {
                            self.runtime_drop_item_packets_rejected += 1;
                        }
                        Err(error) => {
                            self.network_error = Some(error.to_string());
                            self.runtime_drop_item_packets_rejected += 1;
                            continue;
                        }
                    }
                }
                ProviderEvent::ServerPacket {
                    connection_id,
                    packet: PacketKind::TransferInventoryCallPacket(packet),
                } => {
                    self.runtime_transfer_inventory_packets_seen += 1;
                    match self.apply_server_transfer_inventory_packet(connection_id, &packet) {
                        Ok(true) => {
                            changed += 1;
                            self.runtime_transfer_inventory_packets_accepted += 1;
                        }
                        Ok(false) => {
                            self.runtime_transfer_inventory_packets_rejected += 1;
                        }
                        Err(error) => {
                            self.network_error = Some(error.to_string());
                            self.runtime_transfer_inventory_packets_rejected += 1;
                            continue;
                        }
                    }
                }
                ProviderEvent::ServerPacket {
                    connection_id,
                    packet: PacketKind::RequestDropPayloadCallPacket(packet),
                } => {
                    self.runtime_request_drop_payload_packets_seen += 1;
                    match self.apply_server_request_drop_payload_packet(connection_id, &packet) {
                        Ok(true) => {
                            changed += 1;
                            self.runtime_request_drop_payload_packets_accepted += 1;
                        }
                        Ok(false) => {
                            self.runtime_request_drop_payload_packets_rejected += 1;
                        }
                        Err(error) => {
                            self.network_error = Some(error.to_string());
                            self.runtime_request_drop_payload_packets_rejected += 1;
                            continue;
                        }
                    }
                }
                ProviderEvent::ServerPacket {
                    connection_id,
                    packet: PacketKind::RequestBuildPayloadCallPacket(packet),
                } => {
                    self.runtime_request_build_payload_packets_seen += 1;
                    match self.apply_server_request_build_payload_packet(connection_id, &packet) {
                        Ok(true) => {
                            changed += 1;
                            self.runtime_request_build_payload_packets_accepted += 1;
                        }
                        Ok(false) => {
                            self.runtime_request_build_payload_packets_rejected += 1;
                        }
                        Err(error) => {
                            self.network_error = Some(error.to_string());
                            self.runtime_request_build_payload_packets_rejected += 1;
                            continue;
                        }
                    }
                }
                ProviderEvent::ServerPacket {
                    connection_id,
                    packet: PacketKind::RequestUnitPayloadCallPacket(packet),
                } => {
                    self.runtime_request_unit_payload_packets_seen += 1;
                    match self.apply_server_request_unit_payload_packet(connection_id, &packet) {
                        Ok(true) => {
                            changed += 1;
                            self.runtime_request_unit_payload_packets_accepted += 1;
                        }
                        Ok(false) => {
                            self.runtime_request_unit_payload_packets_rejected += 1;
                        }
                        Err(error) => {
                            self.network_error = Some(error.to_string());
                            self.runtime_request_unit_payload_packets_rejected += 1;
                            continue;
                        }
                    }
                }
                ProviderEvent::ServerPacket {
                    connection_id,
                    packet: PacketKind::ClientPlanSnapshotCallPacket(snapshot),
                } => {
                    if connection_id < 0 {
                        continue;
                    }
                    match self.apply_server_preview_plan_packet(
                        connection_id,
                        &snapshot,
                        Self::current_millis(),
                    ) {
                        Ok(Some(_)) => changed += 1,
                        Ok(None) => continue,
                        Err(error) => {
                            self.network_error = Some(error.to_string());
                            continue;
                        }
                    }
                }
                _ => {}
            }
        }
        changed
    }

    fn apply_server_tile_config_packet(
        &mut self,
        connection_id: i32,
        packet: &TileConfigCallPacket,
    ) -> io::Result<bool> {
        let Some(source_tile_pos) = packet.build.tile_pos else {
            return Ok(false);
        };

        let source_block = self
            .runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == source_tile_pos)
            .and_then(|building| self.content_loader.block(building.block.id));
        let is_unit_cargo_unload = source_block.is_some_and(|block| {
            matches!(
                block,
                BlockDef::Distribution(distribution)
                    if distribution.kind == DistributionBlockKind::UnitCargoUnloadPoint
            )
        });

        if is_unit_cargo_unload {
            let result = self.runtime.configure_owned_unit_cargo_unload_value(
                &self.content_loader,
                source_tile_pos,
                &packet.value,
            );
            self.runtime_unit_cargo_unload_config_packets_seen += 1;
            self.last_runtime_unit_cargo_unload_config_result = Some(result);
            if result.changed() {
                self.runtime_unit_cargo_unload_config_packets_changed += 1;
                self.runtime_unit_cargo_unload_config_packets_forwarded +=
                    self.broadcast_runtime_tile_config(connection_id, packet)?;
            }
            return Ok(result.changed());
        }

        let is_unit_factory =
            source_block.is_some_and(|block| matches!(block, BlockDef::UnitFactory(_)));
        if is_unit_factory {
            let result = self.runtime.configure_owned_unit_factory_value(
                &self.content_loader,
                source_tile_pos,
                &packet.value,
            );
            if result.changed() {
                self.broadcast_runtime_tile_config(connection_id, packet)?;
            }
            return Ok(result.changed());
        }

        let is_reconstructor =
            source_block.is_some_and(|block| matches!(block, BlockDef::UnitReconstructor(_)));
        if is_reconstructor {
            let result = self.runtime.configure_owned_reconstructor_value(
                &self.content_loader,
                source_tile_pos,
                &packet.value,
            );
            if result.changed() {
                self.broadcast_runtime_tile_config(connection_id, packet)?;
            }
            return Ok(result.changed());
        }

        let result = self.runtime.configure_owned_power_node_value(
            &self.content_loader,
            source_tile_pos,
            &packet.value,
        );
        self.runtime_power_node_config_packets_seen += 1;
        if result.changed() {
            self.runtime_power_node_config_packets_changed += 1;
        }
        self.last_runtime_power_node_config_result = Some(result);
        Ok(result.changed())
    }

    fn apply_server_command_building_packet(
        &mut self,
        connection_id: i32,
        packet: &CommandBuildingCallPacket,
    ) -> io::Result<bool> {
        if connection_id < 0 || packet.buildings.is_empty() {
            return Ok(false);
        }

        let source_connection = {
            let state = self.net_server.state();
            let state = state.lock().expect("NetServerState mutex poisoned");
            state.connection_states.get(&connection_id).cloned()
        };
        let Some(source_connection) = source_connection else {
            return Ok(false);
        };
        if !Self::connection_can_receive_preview(&source_connection) {
            return Ok(false);
        }

        let mut player = PlayerComp::new(source_connection.team);
        player.id = connection_id;
        player.name = source_connection.name.clone();
        player.color = source_connection.color as u32;
        player.locale = source_connection.locale.clone();
        player.con = Some(source_connection.clone());
        let report = self.runtime.command_owned_building_positions(
            &self.content_loader,
            source_connection.team,
            &packet.buildings,
            packet.target,
            Some(player.colored_name()),
        );

        if report.changed() {
            self.broadcast_runtime_command_building(connection_id, packet)?;
        }
        Ok(report.changed())
    }

    fn broadcast_runtime_command_building(
        &mut self,
        source_connection_id: i32,
        packet: &CommandBuildingCallPacket,
    ) -> io::Result<usize> {
        let player = if source_connection_id >= 0 {
            EntityRef::new(source_connection_id)
        } else {
            EntityRef::null()
        };
        let buildings = packet.buildings.clone();
        let target = packet.target;
        let targets = self.connected_runtime_tile_config_targets();
        let mut sent = 0;
        let mut first_error = None;
        for target_connection in targets {
            let server_packet = CommandBuildingCallPacket {
                player,
                buildings: buildings.clone(),
                target,
            };
            match self
                .net_server
                .send_command_building(target_connection, server_packet)
            {
                Ok(()) => sent += 1,
                Err(error) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }
        if let Some(error) = first_error {
            Err(error)
        } else {
            Ok(sent)
        }
    }

    fn broadcast_runtime_tile_config(
        &mut self,
        source_connection_id: i32,
        packet: &TileConfigCallPacket,
    ) -> io::Result<usize> {
        let player = if source_connection_id >= 0 {
            EntityRef::new(source_connection_id)
        } else {
            EntityRef::null()
        };
        let build = packet.build;
        let value = packet.value.clone();
        let targets = self.connected_runtime_tile_config_targets();
        let mut sent = 0;
        let mut first_error = None;
        for target in targets {
            let server_packet = TileConfigCallPacket::server(player, build, value.clone());
            match self.net_server.send_tile_config(target, server_packet) {
                Ok(()) => sent += 1,
                Err(error) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }
        if let Some(error) = first_error {
            Err(error)
        } else {
            Ok(sent)
        }
    }

    fn connected_runtime_tile_config_targets(&self) -> Vec<i32> {
        let state = self.net_server.state();
        let state = state.lock().expect("NetServerState mutex poisoned");
        state
            .connection_states
            .iter()
            .filter_map(|(connection_id, connection)| {
                (connection.has_connected
                    && connection.player_added
                    && !connection.kicked
                    && !connection.has_disconnected)
                    .then_some(*connection_id)
            })
            .collect()
    }

    fn apply_server_request_item_packet(
        &mut self,
        connection_id: i32,
        packet: &RequestItemCallPacket,
    ) -> io::Result<bool> {
        if connection_id < 0 {
            return Ok(false);
        }
        self.sync_server_preview_players_from_connections();

        let source_connection = {
            let state = self.net_server.state();
            let state = state.lock().expect("NetServerState mutex poisoned");
            state.connection_states.get(&connection_id).cloned()
        };
        let Some(source_connection) = source_connection else {
            return Ok(false);
        };
        if !Self::connection_can_receive_preview(&source_connection) {
            return Ok(false);
        }

        let Some(tile_pos) = packet.build.tile_pos else {
            return Ok(false);
        };
        let Some(building_index) = self
            .runtime
            .buildings()
            .iter()
            .position(|building| building.tile_pos == tile_pos)
        else {
            return Ok(false);
        };

        let Some(unit_type) = self.default_server_unit_type() else {
            return Ok(false);
        };
        let player_snapshot = {
            let player = self
                .server_preview_players
                .entry(connection_id)
                .or_insert_with(|| PlayerComp::new(source_connection.team));
            player.id = connection_id;
            player.team = source_connection.team;
            player.name = source_connection.name.clone();
            player.color = source_connection.color as u32;
            player.locale = source_connection.locale.clone();
            player.con = Some(source_connection.clone());
            player.set_unit_state(PlayerUnitState::unit(connection_id));
            player.clone()
        };
        self.server_units
            .entry(connection_id)
            .or_insert_with(|| UnitComp::new(connection_id, unit_type, source_connection.team));

        let build_snapshot = self
            .runtime
            .buildings()
            .get(building_index)
            .expect("building index resolved above");
        let within_range =
            Self::player_within_item_transfer_range(&player_snapshot, build_snapshot);
        let context = RequestItemContext {
            player: Some(EntityRef::new(connection_id)),
            local_player: false,
            within_range,
        };
        let request_outcome = request_item(
            context,
            Some(&player_snapshot),
            self.server_units.get(&connection_id),
            Some(build_snapshot),
            packet.item.clone(),
            packet.amount,
            |build, player| build.team == player.team,
            |_player, _build| true,
            |_build, _item, _amount| true,
        );
        self.last_runtime_request_item_outcome = Some(request_outcome.clone());
        if !request_outcome.accepted {
            self.last_runtime_take_items_outcome = None;
            return Ok(false);
        }

        let requested_item_name = packet.item.clone();
        let requested_item_id = requested_item_name.as_deref().and_then(|name| {
            self.content_loader
                .item_by_name(name)
                .map(|item| item.base.mappable.base.id as i16)
        });
        let take_outcome = take_items(
            self.runtime.buildings_mut().get_mut(building_index),
            packet.item.clone(),
            request_outcome.accepted_amount,
            self.server_units.get_mut(&connection_id),
            |name| {
                if requested_item_name.as_deref() == Some(name) {
                    requested_item_id
                } else {
                    None
                }
            },
        );
        if let Some(remove_stack) = take_outcome.remove_stack.as_ref() {
            self.runtime
                .apply_item_remove_stack_plan(&self.content_loader, remove_stack);
        }
        if take_outcome.accepted {
            if let Some(packet) = take_outcome.packet.as_ref() {
                self.net_server
                    .net_mut()
                    .send(&PacketKind::TakeItemsCallPacket(packet.clone()), false)?;
                self.runtime_take_items_packets_sent += 1;
            }
            for packet in &take_outcome.transfer_effects {
                self.net_server.net_mut().send(
                    &PacketKind::TransferItemEffectCallPacket(packet.clone()),
                    false,
                )?;
                self.runtime_transfer_item_effect_packets_sent += 1;
            }
        }
        let accepted = take_outcome.accepted;
        self.last_runtime_take_items_outcome = Some(take_outcome);
        Ok(accepted)
    }

    fn apply_server_drop_item_packet(
        &mut self,
        connection_id: i32,
        packet: &DropItemCallPacket,
    ) -> io::Result<bool> {
        if connection_id < 0 {
            return Ok(false);
        }
        self.sync_server_preview_players_from_connections();

        let source_connection = {
            let state = self.net_server.state();
            let state = state.lock().expect("NetServerState mutex poisoned");
            state.connection_states.get(&connection_id).cloned()
        };
        let Some(source_connection) = source_connection else {
            return Ok(false);
        };
        if !Self::connection_can_receive_preview(&source_connection) {
            return Ok(false);
        }

        let Some(unit_type) = self.default_server_unit_type() else {
            return Ok(false);
        };
        let player_snapshot = {
            let player = self
                .server_preview_players
                .entry(connection_id)
                .or_insert_with(|| PlayerComp::new(source_connection.team));
            player.id = connection_id;
            player.team = source_connection.team;
            player.name = source_connection.name.clone();
            player.color = source_connection.color as u32;
            player.locale = source_connection.locale.clone();
            player.con = Some(source_connection.clone());
            player.set_unit_state(PlayerUnitState::unit(connection_id));
            player.clone()
        };
        self.server_units
            .entry(connection_id)
            .or_insert_with(|| UnitComp::new(connection_id, unit_type, source_connection.team));

        let drop_outcome = {
            let Some(unit) = self.server_units.get_mut(&connection_id) else {
                return Ok(false);
            };
            drop_item(
                DropItemContext {
                    local_player: false,
                },
                Some(&player_snapshot),
                Some(unit),
                packet.angle,
            )
        };
        if !drop_outcome.accepted {
            self.last_runtime_drop_item_outcome = Some(drop_outcome);
            return Ok(false);
        }

        let accepted = drop_outcome.accepted;
        self.last_runtime_drop_item_outcome = Some(drop_outcome);
        Ok(accepted)
    }

    fn apply_server_transfer_inventory_packet(
        &mut self,
        connection_id: i32,
        packet: &TransferInventoryCallPacket,
    ) -> io::Result<bool> {
        if connection_id < 0 {
            return Ok(false);
        }
        self.sync_server_preview_players_from_connections();

        let source_connection = {
            let state = self.net_server.state();
            let state = state.lock().expect("NetServerState mutex poisoned");
            state.connection_states.get(&connection_id).cloned()
        };
        let Some(source_connection) = source_connection else {
            return Ok(false);
        };
        if !Self::connection_can_receive_preview(&source_connection) {
            return Ok(false);
        }

        let Some(tile_pos) = packet.build.tile_pos else {
            return Ok(false);
        };
        let Some(building_index) = self
            .runtime
            .buildings()
            .iter()
            .position(|building| building.tile_pos == tile_pos)
        else {
            return Ok(false);
        };

        let Some(unit_type) = self.default_server_unit_type() else {
            return Ok(false);
        };
        let player_snapshot = {
            let player = self
                .server_preview_players
                .entry(connection_id)
                .or_insert_with(|| PlayerComp::new(source_connection.team));
            player.id = connection_id;
            player.team = source_connection.team;
            player.name = source_connection.name.clone();
            player.color = source_connection.color as u32;
            player.locale = source_connection.locale.clone();
            player.con = Some(source_connection.clone());
            player.set_unit_state(PlayerUnitState::unit(connection_id));
            player.clone()
        };
        self.server_units
            .entry(connection_id)
            .or_insert_with(|| UnitComp::new(connection_id, unit_type, source_connection.team));

        let unit_item_name = self
            .server_units
            .get(&connection_id)
            .and_then(|unit| unit.items.item().map(str::to_owned));
        let unit_item_id = unit_item_name.as_deref().and_then(|name| {
            self.content_loader
                .item_by_name(name)
                .map(|item| item.base.mappable.base.id as i16)
        });

        let build_snapshot = self
            .runtime
            .buildings()
            .get(building_index)
            .expect("building index resolved above");
        let within_range =
            Self::player_within_item_transfer_range(&player_snapshot, build_snapshot);
        let context = TransferInventoryContext {
            player: Some(EntityRef::new(connection_id)),
            local_player: false,
            within_range,
            deposit_rate_allowed: true,
        };
        let transfer_inventory_outcome = transfer_inventory(
            context,
            Some(&player_snapshot),
            self.server_units.get(&connection_id),
            Some(build_snapshot),
            |build| build.items.is_some(),
            |player, build| player.team == build.team,
            |_player, _build, _item, _amount| true,
            |build, _unit, item, amount| {
                if unit_item_name.as_deref() != Some(item) {
                    return 0;
                }
                let Some(item_id) = unit_item_id else {
                    return 0;
                };
                Self::building_accept_stack_amount(build, item_id, amount)
            },
        );
        self.last_runtime_transfer_inventory_outcome = Some(transfer_inventory_outcome.clone());
        if !transfer_inventory_outcome.accepted {
            self.last_runtime_transfer_item_to_outcome = None;
            return Ok(false);
        }

        let Some(planned_packet) = transfer_inventory_outcome.packet.as_ref() else {
            self.last_runtime_transfer_item_to_outcome = None;
            return Ok(false);
        };
        let transfer_item_name = planned_packet.item.clone();
        let transfer_item_id = transfer_item_name.as_deref().and_then(|name| {
            self.content_loader
                .item_by_name(name)
                .map(|item| item.base.mappable.base.id as i16)
        });
        let transfer_item_to_outcome = transfer_item_to(
            self.server_units.get_mut(&connection_id),
            planned_packet.item.clone(),
            planned_packet.amount,
            planned_packet.x,
            planned_packet.y,
            self.runtime.buildings_mut().get_mut(building_index),
            |name| {
                if transfer_item_name.as_deref() == Some(name) {
                    transfer_item_id
                } else {
                    None
                }
            },
        );
        if transfer_item_to_outcome.accepted {
            if let (Some(packet), Some(item_id)) =
                (transfer_item_to_outcome.packet.as_ref(), transfer_item_id)
            {
                self.runtime.apply_item_handle_stack_side_effects(
                    &self.content_loader,
                    packet.build,
                    item_id as ContentId,
                    packet.amount,
                );
                self.net_server
                    .net_mut()
                    .send(&PacketKind::TransferItemToCallPacket(packet.clone()), false)?;
                self.runtime_transfer_item_to_packets_sent += 1;
            }
        }
        let accepted = transfer_item_to_outcome.accepted;
        self.last_runtime_transfer_item_to_outcome = Some(transfer_item_to_outcome);
        Ok(accepted)
    }

    fn apply_server_request_drop_payload_packet(
        &mut self,
        connection_id: i32,
        packet: &RequestDropPayloadCallPacket,
    ) -> io::Result<bool> {
        if connection_id < 0 {
            return Ok(false);
        }
        self.sync_server_preview_players_from_connections();

        let source_connection = {
            let state = self.net_server.state();
            let state = state.lock().expect("NetServerState mutex poisoned");
            state.connection_states.get(&connection_id).cloned()
        };
        let Some(source_connection) = source_connection else {
            return Ok(false);
        };
        if !Self::connection_can_receive_preview(&source_connection) {
            return Ok(false);
        }

        let Some(unit_type) = self.default_server_unit_type() else {
            return Ok(false);
        };
        let player_snapshot = {
            let player = self
                .server_preview_players
                .entry(connection_id)
                .or_insert_with(|| PlayerComp::new(source_connection.team));
            player.id = connection_id;
            player.team = source_connection.team;
            player.name = source_connection.name.clone();
            player.color = source_connection.color as u32;
            player.locale = source_connection.locale.clone();
            player.con = Some(source_connection.clone());
            player.set_unit_state(PlayerUnitState::unit(connection_id));
            player.clone()
        };
        self.server_units
            .entry(connection_id)
            .or_insert_with(|| UnitComp::new(connection_id, unit_type, source_connection.team));

        let request_drop_outcome = request_drop_payload(
            RequestDropPayloadContext {
                player: Some(EntityRef::new(connection_id)),
                local_player: false,
                net_client: false,
            },
            Some(&player_snapshot),
            self.server_units.get(&connection_id),
            packet.x,
            packet.y,
            |player, unit| player.team == unit.team_id(),
        );
        self.last_runtime_request_drop_payload_outcome = Some(request_drop_outcome.clone());
        if !request_drop_outcome.accepted {
            self.last_runtime_payload_dropped_outcome = None;
            return Ok(false);
        }

        let Some(planned_packet) = request_drop_outcome.packet.as_ref() else {
            self.last_runtime_payload_dropped_outcome = None;
            return Ok(false);
        };
        if !self.apply_payload_drop_to_server_unit(
            planned_packet.unit,
            planned_packet.x,
            planned_packet.y,
        ) {
            self.last_runtime_payload_dropped_outcome = None;
            return Ok(false);
        }
        let payload_dropped_outcome = payload_dropped(
            Some(planned_packet.unit),
            planned_packet.x,
            planned_packet.y,
        );
        if let Some(packet) = payload_dropped_outcome.packet.as_ref() {
            self.net_server
                .net_mut()
                .send(&PacketKind::PayloadDroppedCallPacket(packet.clone()), true)?;
            self.runtime_payload_dropped_packets_sent += 1;
        }
        let accepted = payload_dropped_outcome.accepted;
        self.last_runtime_payload_dropped_outcome = Some(payload_dropped_outcome);
        Ok(accepted)
    }

    fn apply_server_request_build_payload_packet(
        &mut self,
        connection_id: i32,
        packet: &RequestBuildPayloadCallPacket,
    ) -> io::Result<bool> {
        if connection_id < 0 {
            return Ok(false);
        }
        self.sync_server_preview_players_from_connections();

        let source_connection = {
            let state = self.net_server.state();
            let state = state.lock().expect("NetServerState mutex poisoned");
            state.connection_states.get(&connection_id).cloned()
        };
        let Some(source_connection) = source_connection else {
            return Ok(false);
        };
        if !Self::connection_can_receive_preview(&source_connection) {
            return Ok(false);
        }

        let Some(tile_pos) = packet.build.tile_pos else {
            return Ok(false);
        };
        let Some(building_index) = self
            .runtime
            .buildings()
            .iter()
            .position(|building| building.tile_pos == tile_pos)
        else {
            return Ok(false);
        };

        let Some(unit_type) = self.default_server_unit_type() else {
            return Ok(false);
        };
        let player_snapshot = {
            let player = self
                .server_preview_players
                .entry(connection_id)
                .or_insert_with(|| PlayerComp::new(source_connection.team));
            player.id = connection_id;
            player.team = source_connection.team;
            player.name = source_connection.name.clone();
            player.color = source_connection.color as u32;
            player.locale = source_connection.locale.clone();
            player.con = Some(source_connection.clone());
            player.set_unit_state(PlayerUnitState::unit(connection_id));
            player.clone()
        };
        self.server_units
            .entry(connection_id)
            .or_insert_with(|| UnitComp::new(connection_id, unit_type, source_connection.team));

        let build_snapshot = self.runtime.buildings()[building_index].clone();
        let stored_payload_ref = self.runtime_payload_ref_for_tile(tile_pos).cloned();
        let stored_payload_state = stored_payload_ref
            .as_ref()
            .and_then(|payload| self.payload_ref_to_payload_state(payload));
        let build_can_pickup = self.server_build_can_pickup(&build_snapshot);
        let context = RequestBuildPayloadContext {
            player: Some(EntityRef::new(connection_id)),
            local_player: false,
            within_range: Self::player_within_payload_pickup_range(
                &player_snapshot,
                &build_snapshot,
            ),
            teams_can_interact: player_snapshot.team == build_snapshot.team,
        };

        let request_outcome = request_build_payload(
            context,
            Some(&player_snapshot),
            self.server_units.get(&connection_id),
            Some(&build_snapshot),
            stored_payload_state.as_ref(),
            build_can_pickup,
            |_player, _build, _unit| true,
        );
        self.last_runtime_request_build_payload_outcome = Some(request_outcome.clone());
        if !request_outcome.accepted {
            self.last_runtime_picked_build_payload_outcome = None;
            return Ok(false);
        }

        let Some(planned_packet) = request_outcome.packet.as_ref() else {
            self.last_runtime_picked_build_payload_outcome = None;
            return Ok(false);
        };
        let Some(pickup) = request_outcome.pickup else {
            self.last_runtime_picked_build_payload_outcome = None;
            return Ok(false);
        };

        let picked_outcome = picked_build_payload(
            Some(planned_packet.unit),
            Some(&build_snapshot),
            planned_packet.on_ground,
        );
        if !picked_outcome.accepted {
            self.last_runtime_picked_build_payload_outcome = Some(picked_outcome);
            return Ok(false);
        }
        let Some(picked_packet) = picked_outcome.packet.as_ref() else {
            self.last_runtime_picked_build_payload_outcome = Some(picked_outcome);
            return Ok(false);
        };
        let picked_packet = *picked_packet;

        if !self.apply_picked_build_payload_to_server_unit(&picked_packet, pickup) {
            self.last_runtime_picked_build_payload_outcome = None;
            return Ok(false);
        }

        self.net_server.net_mut().send(
            &PacketKind::PickedBuildPayloadCallPacket(picked_packet),
            true,
        )?;
        self.runtime_picked_build_payload_packets_sent += 1;
        self.last_runtime_picked_build_payload_outcome = Some(picked_outcome);
        Ok(true)
    }

    fn apply_server_request_unit_payload_packet(
        &mut self,
        connection_id: i32,
        packet: &RequestUnitPayloadCallPacket,
    ) -> io::Result<bool> {
        if connection_id < 0 {
            return Ok(false);
        }
        self.sync_server_preview_players_from_connections();

        let source_connection = {
            let state = self.net_server.state();
            let state = state.lock().expect("NetServerState mutex poisoned");
            state.connection_states.get(&connection_id).cloned()
        };
        let Some(source_connection) = source_connection else {
            return Ok(false);
        };
        if !Self::connection_can_receive_preview(&source_connection) {
            return Ok(false);
        }

        let UnitRef::Unit { id: target_id } = packet.target else {
            return Ok(false);
        };
        if target_id == connection_id {
            return Ok(false);
        }

        let Some(unit_type) = self.default_server_unit_type() else {
            return Ok(false);
        };
        let player_snapshot = {
            let player = self
                .server_preview_players
                .entry(connection_id)
                .or_insert_with(|| PlayerComp::new(source_connection.team));
            player.id = connection_id;
            player.team = source_connection.team;
            player.name = source_connection.name.clone();
            player.color = source_connection.color as u32;
            player.locale = source_connection.locale.clone();
            player.con = Some(source_connection.clone());
            player.set_unit_state(PlayerUnitState::unit(connection_id));
            player.clone()
        };
        self.server_units
            .entry(connection_id)
            .or_insert_with(|| UnitComp::new(connection_id, unit_type, source_connection.team));

        let Some(unit_snapshot) = self.server_units.get(&connection_id).cloned() else {
            return Ok(false);
        };
        let Some(target_snapshot) = self.server_units.get(&target_id).cloned() else {
            return Ok(false);
        };

        let request_outcome = request_unit_payload(
            RequestUnitPayloadContext {
                player: Some(EntityRef::new(connection_id)),
                within_range: Self::unit_payload_target_within_range(
                    &unit_snapshot,
                    &target_snapshot,
                ),
            },
            Some(&player_snapshot),
            Some(&unit_snapshot),
            Some(&target_snapshot),
            !target_snapshot.controller.is_player(),
            target_snapshot.type_info.allowed_in_payloads,
        );
        self.last_runtime_request_unit_payload_outcome = Some(request_outcome.clone());
        if !request_outcome.accepted {
            self.last_runtime_picked_unit_payload_outcome = None;
            return Ok(false);
        }

        let Some(planned_packet) = request_outcome.packet.as_ref() else {
            self.last_runtime_picked_unit_payload_outcome = None;
            return Ok(false);
        };
        let picked_outcome =
            picked_unit_payload(Some(planned_packet.unit), Some(planned_packet.target));
        if !picked_outcome.accepted {
            self.last_runtime_picked_unit_payload_outcome = Some(picked_outcome);
            return Ok(false);
        }
        let Some(picked_packet) = picked_outcome.packet.as_ref() else {
            self.last_runtime_picked_unit_payload_outcome = Some(picked_outcome);
            return Ok(false);
        };
        let picked_packet = *picked_packet;

        if !self.apply_picked_unit_payload_to_server_unit(&picked_packet) {
            self.last_runtime_picked_unit_payload_outcome = None;
            return Ok(false);
        }

        self.net_server.net_mut().send(
            &PacketKind::PickedUnitPayloadCallPacket(picked_packet),
            true,
        )?;
        self.runtime_picked_unit_payload_packets_sent += 1;
        self.last_runtime_picked_unit_payload_outcome = Some(picked_outcome);
        Ok(true)
    }

    pub fn apply_server_unit_entered_payload(
        &mut self,
        unit_id: i32,
        build_tile_pos: i32,
    ) -> io::Result<bool> {
        let Some(unit_snapshot) = self.server_units.get(&unit_id).cloned() else {
            return Ok(false);
        };
        if !unit_snapshot.type_info.allowed_in_payloads {
            return Ok(false);
        }
        let Some(building_snapshot) = self
            .runtime
            .buildings
            .iter()
            .find(|building| building.tile_pos == build_tile_pos)
            .cloned()
        else {
            return Ok(false);
        };

        let outcome = unit_entered_payload(Some(&unit_snapshot), Some(&building_snapshot));
        if !outcome.accepted {
            return Ok(false);
        }
        if !self.runtime.attach_unit_payload_to_building(
            &self.content_loader,
            build_tile_pos,
            &unit_snapshot,
        ) {
            return Ok(false);
        }

        self.server_units.remove(&unit_id);
        if let Some(packet) = outcome.packet.as_ref() {
            self.net_server.net_mut().send(
                &PacketKind::UnitEnteredPayloadCallPacket(packet.clone()),
                true,
            )?;
        }
        Ok(true)
    }

    fn tick_server_unit_entered_payloads(&mut self) -> io::Result<usize> {
        let candidates: Vec<_> = self
            .server_units
            .iter()
            .filter_map(|(&unit_id, unit)| {
                self.server_unit_enter_payload_build_tile_pos(unit)
                    .map(|build_tile_pos| (unit_id, build_tile_pos))
            })
            .collect();

        let mut applied = 0;
        for (unit_id, build_tile_pos) in candidates {
            if self.apply_server_unit_entered_payload(unit_id, build_tile_pos)? {
                applied += 1;
            }
        }
        Ok(applied)
    }

    fn runtime_unit_cargo_loader_spawn_candidate_tiles(&self) -> Vec<i32> {
        self.runtime
            .buildings
            .iter()
            .filter_map(|building| {
                let Some(BlockDef::Distribution(distribution)) =
                    self.content_loader.block(building.block.id)
                else {
                    return None;
                };
                if distribution.kind != DistributionBlockKind::UnitCargoLoader {
                    return None;
                }

                let has_unit = matches!(
                    self.runtime
                        .distribution_runtime_states
                        .get(&building.tile_pos),
                    Some(GameRuntimeDistributionBlockState::UnitCargoLoader(state))
                        if state.has_unit
                );
                (!has_unit).then_some(building.tile_pos)
            })
            .collect()
    }

    fn runtime_unit_cargo_loader_has_unit(&self, tile_pos: i32) -> bool {
        matches!(
            self.runtime.distribution_runtime_states.get(&tile_pos),
            Some(GameRuntimeDistributionBlockState::UnitCargoLoader(state)) if state.has_unit
        )
    }

    fn next_server_runtime_unit_id(&self) -> i32 {
        let server_max = self
            .server_units
            .keys()
            .copied()
            .max()
            .unwrap_or(SERVER_RUNTIME_UNIT_ID_START - 1);
        let cargo_loader_max = self
            .runtime
            .distribution_runtime_states
            .values()
            .filter_map(|state| match state {
                GameRuntimeDistributionBlockState::UnitCargoLoader(state)
                    if state.read_unit_id >= 0 =>
                {
                    Some(state.read_unit_id)
                }
                _ => None,
            })
            .max()
            .unwrap_or(SERVER_RUNTIME_UNIT_ID_START - 1);

        server_max
            .max(cargo_loader_max)
            .max(SERVER_RUNTIME_UNIT_ID_START - 1)
            .saturating_add(1)
    }

    fn tick_server_unit_spawn_abilities(&mut self, delta_ticks: f32) -> io::Result<usize> {
        let parent_ids: Vec<i32> = self.server_units.keys().copied().collect();
        let team_registry = vanilla_teams();
        let mut spawned = 0;

        for parent_id in parent_ids {
            let Some(parent_snapshot) = self.server_units.get(&parent_id) else {
                continue;
            };
            let team = parent_snapshot.team_id();
            let unit_build_speed = self.runtime.state.rules.unit_build_speed(team.0 as usize);
            let cap_rules = UnitCapRules {
                wave_team: TeamId(self.runtime.state.rules.wave_team.clamp(0, u8::MAX as i32) as u8),
                pvp: self.runtime.state.rules.pvp,
                campaign: self.runtime.state.is_campaign(),
                disable_unit_cap: self.runtime.state.rules.disable_unit_cap,
                unit_cap_variable: self.runtime.state.rules.unit_cap_variable,
                unit_cap: self.runtime.state.rules.unit_cap,
            };
            let team_data_unit_cap = self
                .runtime
                .state
                .teams
                .get_or_null(team.0)
                .map_or(0, |data| data.unit_cap);
            let ignore_unit_cap = team_registry.get(team.0 as i32).ignore_unit_cap;
            let mut spawn_targets: BTreeMap<String, (ContentId, bool, bool, i32)> = BTreeMap::new();

            for descriptor in &parent_snapshot.type_info.abilities {
                let Some(ability) = UnitSpawnAbility::from_descriptor(descriptor) else {
                    continue;
                };
                if spawn_targets.contains_key(&ability.unit) {
                    continue;
                }
                if let Some(unit_type) = self.content_loader.unit_by_name(&ability.unit) {
                    let unit_type_id = unit_type.id();
                    let type_count = self
                        .server_units
                        .values()
                        .filter(|unit| {
                            unit.team_id() == team && unit.type_info.id() == unit_type_id
                        })
                        .count()
                        .min(i32::MAX as usize) as i32;
                    spawn_targets.insert(
                        ability.unit,
                        (
                            unit_type_id,
                            unit_type.use_unit_cap,
                            self.runtime.state.rules.is_unit_banned(unit_type.name()),
                            type_count,
                        ),
                    );
                }
            }

            let mut pending_counts: BTreeMap<ContentId, i32> = BTreeMap::new();
            let Some(parent) = self.server_units.get_mut(&parent_id) else {
                continue;
            };
            let plans =
                parent.update_unit_spawn_abilities(delta_ticks, unit_build_speed, |unit_name| {
                    let Some((unit_type_id, use_unit_cap, banned, base_count)) =
                        spawn_targets.get(unit_name)
                    else {
                        return false;
                    };
                    let type_count =
                        *base_count + pending_counts.get(unit_type_id).copied().unwrap_or(0);
                    let can_create = units_can_create(
                        UnitCapTeam {
                            team,
                            ignore_unit_cap,
                            data_unit_cap: team_data_unit_cap,
                            type_count,
                        },
                        UnitCapType {
                            use_unit_cap: *use_unit_cap,
                            banned: *banned,
                        },
                        cap_rules,
                    );
                    if can_create {
                        *pending_counts.entry(*unit_type_id).or_insert(0) += 1;
                    }
                    can_create
                });

            for plan in plans {
                let Some(unit_type) = self.content_loader.unit_by_name(&plan.unit).cloned() else {
                    continue;
                };
                let unit_id = self.next_server_runtime_unit_id();
                let mut unit = UnitComp::new(unit_id, unit_type, team);
                unit.set_pos(plan.x, plan.y);
                unit.set_rotation(plan.rotation);
                self.runtime.note_unit_create_event(
                    Some(unit_id),
                    plan.unit,
                    team,
                    None,
                    Some(parent_id),
                );
                unit.add();
                self.broadcast_server_unit_spawn(&unit)?;
                self.server_units.insert(unit_id, unit);
                spawned += 1;
            }
        }

        Ok(spawned)
    }

    fn tick_server_energy_field_abilities(&mut self, delta_ticks: f32) -> usize {
        let parent_ids: Vec<i32> = self.server_units.keys().copied().collect();
        let mut pulses = 0;

        for parent_id in parent_ids {
            let Some(parent_snapshot) = self.server_units.get(&parent_id) else {
                continue;
            };
            let parent_team = parent_snapshot.team_id();
            let parent_type_id = parent_snapshot.type_info.id();
            let parent_damage_scale = self.runtime.state.rules.unit_damage(parent_team.0 as usize)
                * parent_snapshot.status.damage_multiplier;
            let targets: Vec<EnergyFieldTarget> = self
                .server_units
                .iter()
                .filter(|(unit_id, unit)| **unit_id != parent_id && !unit.health.dead)
                .filter_map(|(unit_id, unit)| {
                    let id = u32::try_from(*unit_id).ok()?;
                    let same_team = unit.team_id() == parent_team;
                    Some(EnergyFieldTarget {
                        id,
                        x: unit.x(),
                        y: unit.y(),
                        air: unit.type_info.flying || unit.elevation > 0.001,
                        targetable: unit.type_info.targetable,
                        same_team,
                        damaged: unit.health.damaged(),
                        max_health: unit.health.max_health,
                        same_type: unit.type_info.id() == parent_type_id,
                    })
                })
                .collect();

            let Some(parent) = self.server_units.get_mut(&parent_id) else {
                continue;
            };
            let field_pulses = parent.update_energy_field_abilities(
                delta_ticks,
                parent_damage_scale,
                false,
                &targets,
            );
            pulses += field_pulses.len();

            for pulse in field_pulses {
                for hit in pulse.hits {
                    let Some(target_id) = i32::try_from(hit.id).ok() else {
                        continue;
                    };
                    let status_effect = hit
                        .status
                        .as_deref()
                        .and_then(|name| self.content_loader.status_effect_by_name(name))
                        .cloned();
                    let Some(target) = self.server_units.get_mut(&target_id) else {
                        continue;
                    };

                    match hit.action {
                        EnergyFieldAction::Heal => {
                            target.heal_mark(hit.amount);
                            target.health.heal(hit.amount);
                        }
                        EnergyFieldAction::Damage => {
                            target.health.damage(hit.amount);
                            if let Some(effect) = status_effect {
                                target.status.apply(effect, hit.status_duration);
                            }
                        }
                    }
                    target.refresh_component_views();
                }
            }
        }

        pulses
    }

    fn tick_server_regen_abilities(&mut self, delta_ticks: f32) -> usize {
        let parent_ids: Vec<i32> = self.server_units.keys().copied().collect();
        let mut updates = 0;

        for parent_id in parent_ids {
            let Some(parent) = self.server_units.get_mut(&parent_id) else {
                continue;
            };
            let heals = parent.update_regen_abilities(delta_ticks);
            if !heals.is_empty() {
                parent.refresh_component_views();
            }
            updates += heals.len();
        }

        updates
    }

    fn tick_server_liquid_regen_abilities(&mut self, delta_ticks: f32) -> io::Result<usize> {
        let unit_ids: Vec<i32> = self.server_units.keys().copied().collect();
        let mut updates = 0;

        for unit_id in unit_ids {
            let Some(unit_snapshot) = self.server_units.get(&unit_id) else {
                continue;
            };
            if unit_snapshot.health.dead
                || !unit_snapshot.health.damaged()
                || unit_snapshot.type_info.flying
                || unit_snapshot.elevation > 0.001
            {
                continue;
            }

            let unit_x = unit_snapshot.x();
            let unit_y = unit_snapshot.y();
            let unit_rotation = unit_snapshot.rotation();
            let hit_size = unit_snapshot.type_info.hit_size;
            let abilities = unit_snapshot.liquid_regen_abilities();
            let mut total_heal = 0.0;
            let mut slurp_effects = Vec::new();

            for ability in abilities {
                let mut ability_taken = 0.0;
                for (tile_x, tile_y) in
                    ability.slurp_tiles(unit_x, unit_y, hit_size, TILE_SIZE as f32)
                {
                    let taken = self.runtime.server_puddles.slurp_matching_liquid(
                        tile_x,
                        tile_y,
                        &ability.liquid_name,
                        ability.slurp_speed * delta_ticks,
                    );
                    ability_taken += taken;
                    total_heal += ability.planned_heal_amount(taken);
                }
                if ability_taken > 0.0
                    && ability.slurp_effect != "none"
                    && ability.slurp_effect_chance > 0.0
                {
                    slurp_effects.push(ability.slurp_effect);
                }
            }

            if total_heal <= 0.0 {
                continue;
            }
            let Some(unit) = self.server_units.get_mut(&unit_id) else {
                continue;
            };
            unit.heal_mark(total_heal);
            unit.health.heal(total_heal);
            unit.refresh_component_views();
            updates += 1;
            for effect in slurp_effects {
                self.broadcast_server_effect_with_data(
                    &effect,
                    unit_x,
                    unit_y,
                    unit_rotation,
                    TypeValue::Unit(unit_id),
                )?;
            }
        }

        Ok(updates)
    }

    fn tick_server_puddles(&mut self, delta_ticks: f32) -> io::Result<usize> {
        let world = &self.runtime.state.world;
        let content = &self.content_loader;
        let width = world.width() as i32;
        let height = world.height() as i32;
        if self.runtime.server_fires.width() != width
            || self.runtime.server_fires.height() != height
        {
            self.runtime.server_fires = Fires::new(width, height);
        }

        let report = self
            .runtime
            .server_puddles
            .update_all_with_passability_report(
                delta_ticks,
                true,
                |x, y, liquid| {
                    world.tile(x, y).is_some()
                        && (liquid.move_through_blocks
                            || !world.wall_solid_with_content(x, y, content))
                },
                |x, y| world.build(x, y).is_some(),
                |x, y, liquid| {
                    let hash = (x as i64)
                        .wrapping_mul(734_287)
                        .wrapping_add((y as i64).wrapping_mul(912_271))
                        .wrapping_add(liquid.name.len() as i64);
                    hash & 1 == 0
                },
            );
        let mut updates = report.events.len();
        let mut ripple_effects = Vec::new();
        for event in &report.events {
            if !event.affect_units {
                continue;
            }
            let status_effect = event
                .liquid
                .effect
                .as_deref()
                .filter(|effect| *effect != "none")
                .and_then(|effect| self.content_loader.status_effect_by_name(effect))
                .cloned();
            let size = (event.amount
                / (mindustry_core::mindustry::entities::puddles::MAX_LIQUID / 1.5))
                .clamp(0.0, 1.0)
                * 10.0;
            let rect_x = event.x - size / 2.0;
            let rect_y = event.y - size / 2.0;
            for unit in self.server_units.values_mut() {
                if unit.health.dead || !unit.is_grounded() || unit.type_info.hovering {
                    continue;
                }
                let hit_size = unit.hitbox.hit_size;
                let unit_x = unit.x() - hit_size / 2.0;
                let unit_y = unit.y() - hit_size / 2.0;
                let overlaps = rect_x < unit_x + hit_size
                    && rect_x + size > unit_x
                    && rect_y < unit_y + hit_size
                    && rect_y + size > unit_y;
                if !overlaps {
                    continue;
                }
                if let Some(status_effect) = status_effect.clone() {
                    unit.status.apply(status_effect, 60.0 * 2.0);
                    unit.refresh_component_views();
                    updates += 1;
                }
                let velocity_len2 =
                    unit.vel.vel.x * unit.vel.vel.x + unit.vel.vel.y * unit.vel.vel.y;
                if velocity_len2 > 0.1 * 0.1 {
                    ripple_effects.push((
                        unit.x(),
                        unit.y(),
                        unit.type_info.ripple_scale,
                        event.liquid.color_rgba,
                    ));
                }
            }
        }
        for (x, y, rotation, color) in ripple_effects {
            if self.broadcast_server_effect_colored("ripple", x, y, rotation, color)? {
                updates += 1;
            }
        }
        let mut cell_removed_ids = Vec::new();
        for event in &report.events {
            if !event.liquid_update {
                continue;
            }
            let Some(target_name) = event.liquid.reaction_target.as_deref() else {
                continue;
            };
            if !self.runtime.state.rules.fire {
                continue;
            }
            let Some(target_liquid_id) = self
                .content_loader
                .liquid_by_name(target_name)
                .map(|liquid| liquid.base.mappable.base.id)
            else {
                continue;
            };
            let scaling = (event.amount / mindustry_core::mindustry::entities::puddles::MAX_LIQUID)
                .clamp(0.0, 1.0)
                .powf(2.0);
            if scaling <= 0.0 {
                continue;
            }
            let mut reacted = false;

            for point in ORTHOGONAL_WITH_CENTER_NEIGHBORS {
                let nx = event.tile.x + point.x;
                let ny = event.tile.y + point.y;
                let Some(build_ref) = self.runtime.state.world.build(nx, ny) else {
                    continue;
                };
                let Some((team, deposit_amount)) = ({
                    let Some(building) = self
                        .runtime
                        .buildings
                        .iter_mut()
                        .find(|building| building.tile_pos == build_ref.tile_pos)
                    else {
                        continue;
                    };
                    let team = building.team;
                    let Some(liquids) = building.liquids.as_mut() else {
                        continue;
                    };
                    let available = liquids.get(target_liquid_id);
                    if available <= 0.0001 {
                        continue;
                    }
                    let amount =
                        available.min(event.liquid.cell_max_spread * delta_ticks * scaling);
                    if amount <= 0.0 {
                        continue;
                    }
                    liquids.remove(target_liquid_id, amount * event.liquid.cell_remove_scaling);
                    Some((team, amount * event.liquid.cell_spread_conversion))
                }) else {
                    continue;
                };
                self.runtime.server_puddles.deposit(
                    Some(PuddleTileView::new(nx, ny).with_build(team.0 as i32)),
                    Some(PuddleTileView::new(event.tile.x, event.tile.y)),
                    event.liquid.clone(),
                    deposit_amount,
                    PuddleDepositContext::default(),
                );
                reacted = true;
                updates += 1;
            }

            if event.liquid.cell_spread_damage > 0.0 {
                let build_tile_pos = self
                    .runtime
                    .state
                    .world
                    .build(event.tile.x, event.tile.y)
                    .map(|build_ref| build_ref.tile_pos);
                if let Some(build_tile_pos) = build_tile_pos {
                    let spread = self
                        .runtime
                        .buildings
                        .iter()
                        .find(|building| building.tile_pos == build_tile_pos)
                        .and_then(|building| {
                            building.liquids.as_ref().and_then(|liquids| {
                                let available = liquids.get(target_liquid_id);
                                if available > 0.0001 {
                                    Some((
                                        building.team,
                                        (available * event.liquid.cell_spread_conversion)
                                            .min(event.liquid.cell_max_spread * delta_ticks)
                                            / 2.0,
                                    ))
                                } else {
                                    None
                                }
                            })
                        });
                    if let Some((team, amount_spread)) = spread {
                        for point in ORTHOGONAL_NEIGHBORS {
                            self.runtime.server_puddles.deposit(
                                Some(
                                    PuddleTileView::new(event.tile.x, event.tile.y)
                                        .with_build(team.0 as i32),
                                ),
                                Some(PuddleTileView::new(
                                    event.tile.x + point.x,
                                    event.tile.y + point.y,
                                )),
                                event.liquid.clone(),
                                amount_spread,
                                PuddleDepositContext::default(),
                            );
                        }
                        if let Some(building) = self
                            .runtime
                            .buildings
                            .iter_mut()
                            .find(|building| building.tile_pos == build_tile_pos)
                        {
                            building.damage(
                                event.liquid.cell_spread_damage * delta_ticks * scaling,
                                Self::current_millis() as f32,
                            );
                        }
                        reacted = true;
                        updates += 1;
                    }
                }
            }

            let absorb_report = self.runtime.server_puddles.absorb_neighbor_target_puddles(
                event.tile.x,
                event.tile.y,
                &event.liquid,
                event.amount,
                delta_ticks,
            );
            if absorb_report.absorbed > 0 {
                reacted = true;
                updates += absorb_report.absorbed;
            }
            cell_removed_ids.extend(absorb_report.removed_ids);
            if reacted && event.liquid.name == "neoplasm" {
                self.runtime.note_trigger_event(Trigger::NeoplasmReact);
                updates += 1;
            }
        }
        for event in report.events {
            if !event.create_fire {
                continue;
            }
            let result = self.runtime.server_fires.create(
                Some(FireTile {
                    x: event.tile.x,
                    y: event.tile.y,
                    build_present: event.tile.build_present,
                    flammability: 0.0,
                }),
                FireRules {
                    net_client: false,
                    fire_enabled: self.runtime.state.rules.fire,
                    has_oxygen: true,
                },
            );
            if result != FireCreateResult::Ignored {
                updates += 1;
            }
        }

        let mut removed_ids = report.removed_ids;
        removed_ids.extend(cell_removed_ids);
        if !removed_ids.is_empty() {
            self.broadcast_server_hidden_snapshot(&removed_ids)?;
            updates += removed_ids.len();
        }
        Ok(updates)
    }

    fn tick_server_force_field_abilities(&mut self, delta_ticks: f32) -> usize {
        let parent_ids: Vec<i32> = self.server_units.keys().copied().collect();
        let mut updates = 0;

        for parent_id in parent_ids {
            let Some(parent) = self.server_units.get_mut(&parent_id) else {
                continue;
            };
            let force_updates = parent.update_force_field_abilities(delta_ticks);
            if !force_updates.is_empty() {
                parent.refresh_component_views();
            }
            updates += force_updates.len();
        }

        updates
    }

    fn tick_server_shield_arc_abilities(&mut self, delta_ticks: f32) -> usize {
        let parent_ids: Vec<i32> = self.server_units.keys().copied().collect();
        let mut updates = 0;

        for parent_id in parent_ids {
            let Some(parent) = self.server_units.get_mut(&parent_id) else {
                continue;
            };
            updates += parent.update_shield_arc_abilities(delta_ticks).len();
        }

        updates
    }

    fn tick_server_shield_regen_field_abilities(&mut self, delta_ticks: f32) -> usize {
        let parent_ids: Vec<i32> = self.server_units.keys().copied().collect();
        let mut pulses = 0;

        for parent_id in parent_ids {
            let Some(parent_snapshot) = self.server_units.get(&parent_id) else {
                continue;
            };
            let parent_team = parent_snapshot.team_id();
            let parent_x = parent_snapshot.x();
            let parent_y = parent_snapshot.y();
            let team_units: Vec<(u32, f32, f32, f32)> = self
                .server_units
                .iter()
                .filter(|(_unit_id, unit)| unit.team_id() == parent_team && !unit.health.dead)
                .filter_map(|(unit_id, unit)| {
                    Some((
                        u32::try_from(*unit_id).ok()?,
                        unit.x(),
                        unit.y(),
                        unit.shield.shield,
                    ))
                })
                .collect();

            let Some(parent) = self.server_units.get_mut(&parent_id) else {
                continue;
            };
            let shield_pulses =
                parent.update_shield_regen_field_abilities(delta_ticks, |ability| {
                    let range2 = ability.range * ability.range;
                    team_units
                        .iter()
                        .copied()
                        .filter(|(_id, x, y, _shield)| {
                            let dx = *x - parent_x;
                            let dy = *y - parent_y;
                            dx * dx + dy * dy <= range2
                        })
                        .map(|(id, _x, _y, shield)| {
                            (
                                id,
                                mindustry_core::mindustry::entities::ShieldRegenFieldTarget {
                                    shield,
                                },
                            )
                        })
                        .collect()
                });
            pulses += shield_pulses.len();

            for pulse in shield_pulses {
                for (target_id, shield_after) in pulse.target_ids.into_iter().zip(pulse.shields) {
                    let Some(target_id) = i32::try_from(target_id).ok() else {
                        continue;
                    };
                    let Some(target) = self.server_units.get_mut(&target_id) else {
                        continue;
                    };
                    if shield_after > target.shield.shield {
                        target.shield.shield = shield_after;
                        target.shield.shield_alpha = 1.0;
                        target.refresh_component_views();
                    }
                }
            }
        }

        pulses
    }

    fn tick_server_repair_field_abilities(&mut self, delta_ticks: f32) -> usize {
        let parent_ids: Vec<i32> = self.server_units.keys().copied().collect();
        let mut pulses = 0;

        for parent_id in parent_ids {
            let Some(parent_snapshot) = self.server_units.get(&parent_id) else {
                continue;
            };
            let parent_team = parent_snapshot.team_id();
            let parent_type_id = parent_snapshot.type_info.id();
            let parent_x = parent_snapshot.x();
            let parent_y = parent_snapshot.y();
            let team_units: Vec<(u32, f32, f32, bool, f32, bool)> = self
                .server_units
                .iter()
                .filter(|(_unit_id, unit)| unit.team_id() == parent_team && !unit.health.dead)
                .filter_map(|(unit_id, unit)| {
                    Some((
                        u32::try_from(*unit_id).ok()?,
                        unit.x(),
                        unit.y(),
                        unit.health.damaged(),
                        unit.health.max_health,
                        unit.type_info.id() == parent_type_id,
                    ))
                })
                .collect();

            let Some(parent) = self.server_units.get_mut(&parent_id) else {
                continue;
            };
            let repair_pulses = parent.update_repair_field_abilities(delta_ticks, |ability| {
                let range2 = ability.range * ability.range;
                team_units
                    .iter()
                    .copied()
                    .filter(|(_id, x, y, _damaged, _max_health, _same_type)| {
                        let dx = *x - parent_x;
                        let dy = *y - parent_y;
                        dx * dx + dy * dy <= range2
                    })
                    .map(|(id, _x, _y, damaged, max_health, same_type)| {
                        (
                            id,
                            mindustry_core::mindustry::entities::RepairFieldTarget {
                                damaged,
                                max_health,
                                same_type,
                            },
                        )
                    })
                    .collect()
            });
            pulses += repair_pulses.len();

            for pulse in repair_pulses {
                for (target_id, heal_amount) in pulse.target_ids.into_iter().zip(pulse.heals) {
                    if heal_amount <= 0.0 {
                        continue;
                    }
                    let Some(target_id) = i32::try_from(target_id).ok() else {
                        continue;
                    };
                    let Some(target) = self.server_units.get_mut(&target_id) else {
                        continue;
                    };
                    target.heal_mark(heal_amount);
                    target.health.heal(heal_amount);
                    target.refresh_component_views();
                }
            }
        }

        pulses
    }

    fn tick_server_status_field_abilities(&mut self, delta_ticks: f32) -> usize {
        let parent_ids: Vec<i32> = self.server_units.keys().copied().collect();
        let mut pulses = 0;

        for parent_id in parent_ids {
            let Some(parent_snapshot) = self.server_units.get(&parent_id) else {
                continue;
            };
            let parent_team = parent_snapshot.team_id();
            let parent_x = parent_snapshot.x();
            let parent_y = parent_snapshot.y();
            let team_units: Vec<(u32, f32, f32)> = self
                .server_units
                .iter()
                .filter(|(_unit_id, unit)| unit.team_id() == parent_team && !unit.health.dead)
                .filter_map(|(unit_id, unit)| {
                    Some((u32::try_from(*unit_id).ok()?, unit.x(), unit.y()))
                })
                .collect();

            let Some(parent) = self.server_units.get_mut(&parent_id) else {
                continue;
            };
            let status_pulses = parent.update_status_field_abilities(delta_ticks, |ability| {
                let range2 = ability.range * ability.range;
                team_units
                    .iter()
                    .copied()
                    .filter(|(_id, x, y)| {
                        let dx = *x - parent_x;
                        let dy = *y - parent_y;
                        dx * dx + dy * dy <= range2
                    })
                    .map(|(id, _x, _y)| id)
                    .collect()
            });
            pulses += status_pulses.len();

            for pulse in status_pulses {
                let Some(effect) = self
                    .content_loader
                    .status_effect_by_name(&pulse.effect)
                    .cloned()
                else {
                    continue;
                };
                for target_id in pulse.target_ids {
                    let Some(target_id) = i32::try_from(target_id).ok() else {
                        continue;
                    };
                    let Some(target) = self.server_units.get_mut(&target_id) else {
                        continue;
                    };
                    target.status.apply(effect.clone(), pulse.duration);
                    target.refresh_component_views();
                }
            }
        }

        pulses
    }

    fn tick_server_suppression_field_abilities(&mut self, delta_ticks: f32) -> usize {
        let parent_ids: Vec<i32> = self.server_units.keys().copied().collect();
        let now = self.runtime.state.tick as f32;
        let mut suppressed_buildings = 0;

        for parent_id in parent_ids {
            let Some(parent_snapshot) = self.server_units.get(&parent_id) else {
                continue;
            };
            if parent_snapshot.health.dead {
                continue;
            }
            let parent_team = parent_snapshot.team_id();

            let Some(parent) = self.server_units.get_mut(&parent_id) else {
                continue;
            };
            let suppression_pulses = parent.update_suppression_field_abilities(delta_ticks);

            for pulse in suppression_pulses {
                let range2 = pulse.range * pulse.range;
                for building in self.runtime.buildings_mut() {
                    if building.dead || building.team == parent_team {
                        continue;
                    }
                    let dx = building.x - pulse.x;
                    let dy = building.y - pulse.y;
                    if dx * dx + dy * dy <= range2 {
                        building.apply_heal_suppression(now, pulse.reload + 1.0);
                        suppressed_buildings += 1;
                    }
                }
            }
        }

        suppressed_buildings
    }

    fn apply_server_unit_death_abilities(&mut self) -> io::Result<usize> {
        let dead_ids: Vec<i32> = self
            .server_units
            .iter()
            .filter_map(|(&unit_id, unit)| unit.health.dead.then_some(unit_id))
            .collect();
        let mut spawned = 0;

        for parent_id in dead_ids {
            let Some(parent) = self.server_units.remove(&parent_id) else {
                continue;
            };
            if self.net_server.is_active() {
                self.net_server.net_mut().send(
                    &PacketKind::UnitDespawnCallPacket(UnitDespawnCallPacket {
                        unit: UnitRef::Unit { id: parent_id },
                    }),
                    false,
                )?;
            }

            let liquid_deposit_plans = parent.liquid_explode_ability_deposit_plans();
            self.apply_server_liquid_explode_deposits(&liquid_deposit_plans);

            for (unit_name, plan) in parent.spawn_death_ability_plans() {
                let Some(unit_type) = self.content_loader.unit_by_name(&unit_name).cloned() else {
                    continue;
                };
                let unit_id = self.next_server_runtime_unit_id();
                let mut unit = UnitComp::new(unit_id, unit_type, parent.team_id());
                unit.set_pos(parent.x() + plan.offset_x, parent.y() + plan.offset_y);
                unit.set_rotation(plan.rotation);
                self.runtime.note_unit_create_event(
                    Some(unit_id),
                    unit_name,
                    parent.team_id(),
                    None,
                    Some(parent_id),
                );
                unit.add();
                self.broadcast_server_unit_spawn(&unit)?;
                self.server_units.insert(unit_id, unit);
                spawned += 1;
            }
        }

        Ok(spawned)
    }

    fn apply_server_liquid_explode_deposits(
        &mut self,
        plans: &[LiquidExplodeDepositPlan],
    ) -> usize {
        if plans.is_empty() {
            return 0;
        }

        let width = self.runtime.state.world.width() as i32;
        let height = self.runtime.state.world.height() as i32;
        if self.runtime.server_puddles.width() != width
            || self.runtime.server_puddles.height() != height
        {
            self.runtime.server_puddles = Puddles::new(width, height);
        }

        let mut applied = 0;
        for plan in plans {
            let Some(liquid) = self.content_loader.liquid_by_name(&plan.liquid_name) else {
                continue;
            };
            let result = self.runtime.server_puddles.deposit_at(
                Some(PuddleTileView::new(plan.tile_x, plan.tile_y)),
                PuddleLiquidInfo::from(liquid),
                plan.amount,
                PuddleDepositContext {
                    time: self.runtime.state.tick as f32,
                    ..PuddleDepositContext::default()
                },
            );
            if result.amount > 0.0 || result.accepting > 0.0 || result.added != 0.0 {
                applied += 1;
            }
        }

        applied
    }

    fn apply_runtime_unit_cargo_loader_spawns(
        &mut self,
        candidate_tiles: &[i32],
    ) -> io::Result<usize> {
        let spawned_tiles: Vec<_> = candidate_tiles
            .iter()
            .copied()
            .filter(|tile_pos| self.runtime_unit_cargo_loader_has_unit(*tile_pos))
            .collect();
        let Some(unit_type) = self.content_loader.unit_by_name("manifold").cloned() else {
            return Ok(0);
        };

        let mut applied = 0;
        for tile_pos in spawned_tiles {
            let Some(building) = self
                .runtime
                .buildings
                .iter()
                .find(|building| building.tile_pos == tile_pos)
                .cloned()
            else {
                continue;
            };

            let unit_id = self.next_server_runtime_unit_id();
            let mut unit = UnitComp::new(unit_id, unit_type.clone(), building.team);
            unit.set_pos(building.x, building.y);
            unit.set_rotation(90.0);
            unit.set_controller(UnitControllerState::Cargo);
            unit.building_tether = Some(BuildingTetherComp {
                team: building.team,
                building: Some(BuildingTetherRef {
                    tile_pos,
                    team: building.team,
                    valid: building.is_valid(),
                }),
            });
            unit.cargo_ai = Some(CargoAiRuntimeState::new(Some(tile_pos)));
            self.server_units.insert(unit_id, unit);

            if let Some(GameRuntimeDistributionBlockState::UnitCargoLoader(state)) =
                self.runtime.distribution_runtime_states.get_mut(&tile_pos)
            {
                state.read_unit_id = unit_id;
            }

            if self.net_server.is_active() {
                self.net_server.net_mut().send(
                    &PacketKind::UnitTetherBlockSpawnedCallPacket(
                        UnitTetherBlockSpawnedCallPacket {
                            tile: Some(tile_pos),
                            id: unit_id,
                        },
                    ),
                    true,
                )?;
            }
            applied += 1;
        }

        Ok(applied)
    }

    fn apply_runtime_unit_assembler_spawns(&mut self, tile_positions: &[i32]) -> io::Result<usize> {
        if tile_positions.is_empty() {
            return Ok(0);
        }

        let mut applied = 0;
        let mut seen = BTreeSet::new();
        for tile_pos in tile_positions {
            if !seen.insert(*tile_pos) {
                continue;
            }
            let Some(building) = self
                .runtime
                .buildings
                .iter()
                .find(|building| building.tile_pos == *tile_pos)
                .cloned()
            else {
                continue;
            };
            let Some(BlockDef::UnitAssembler(assembler_block)) =
                self.content_loader.block(building.block.id)
            else {
                continue;
            };
            let current_tier = self
                .runtime
                .unit_runtime_states
                .get(tile_pos)
                .and_then(|state| match state {
                    GameRuntimeUnitBlockState::Assembler { assembler, .. } => {
                        Some(assembler.current_tier.max(0) as usize)
                    }
                    _ => None,
                })
                .unwrap_or(0);
            let Some(plan) = assembler_block
                .plans
                .get(current_tier.min(assembler_block.plans.len().saturating_sub(1)))
            else {
                continue;
            };
            let Some(unit_type) = self.content_loader.unit_by_name(&plan.unit).cloned() else {
                continue;
            };
            let command_pos = self
                .runtime
                .unit_runtime_states
                .get(tile_pos)
                .and_then(|state| match state {
                    GameRuntimeUnitBlockState::Assembler { assembler, .. } => assembler.command_pos,
                    _ => None,
                });

            let (dx, dy) = autotiler_direction(building.rotation);
            let len = TILE_SIZE as f32
                * (assembler_block.area_size + assembler_block.base.size) as f32
                / 2.0;
            let spawn_x = building.x + dx as f32 * len;
            let spawn_y = building.y + dy as f32 * len;
            let unit_id = self.next_server_runtime_unit_id();
            let mut unit = UnitComp::new(unit_id, unit_type, building.team);
            unit.set_pos(spawn_x, spawn_y);
            unit.set_rotation(building.rotdeg());
            if let Some(target_pos) = command_pos {
                unit.set_controller(UnitControllerState::Command(type_io::CommandWire {
                    target_pos: Some(target_pos),
                    ..type_io::CommandWire::default()
                }));
            }
            if !self.try_deliver_runtime_spawned_unit_payload(&unit)? {
                self.broadcast_server_unit_spawn(&unit)?;
                self.server_units.insert(unit_id, unit);
            }
            applied += 1;
        }

        Ok(applied)
    }

    fn try_deliver_runtime_spawned_unit_payload(&mut self, unit: &UnitComp) -> io::Result<bool> {
        let Some(build_tile_pos) = self.server_unit_build_on_tile_pos(unit) else {
            return Ok(false);
        };
        let Some(building_snapshot) = self
            .runtime
            .buildings
            .iter()
            .find(|building| building.tile_pos == build_tile_pos)
            .cloned()
        else {
            return Ok(false);
        };

        let outcome = unit_entered_payload(Some(unit), Some(&building_snapshot));
        if !outcome.accepted {
            return Ok(false);
        }
        if !self
            .runtime
            .attach_unit_payload_to_building(&self.content_loader, build_tile_pos, unit)
        {
            return Ok(false);
        }

        if let Some(packet) = outcome.packet.as_ref() {
            self.net_server.net_mut().send(
                &PacketKind::UnitEnteredPayloadCallPacket(packet.clone()),
                true,
            )?;
        }
        Ok(true)
    }

    fn apply_runtime_unit_assembler_drone_spawns(
        &mut self,
        tile_positions: &[i32],
    ) -> io::Result<usize> {
        if tile_positions.is_empty() {
            return Ok(0);
        }

        let mut applied = 0;
        let mut seen = BTreeSet::new();
        let mut first_error = None;
        for tile_pos in tile_positions {
            if !seen.insert(*tile_pos) {
                continue;
            }
            let Some(building) = self
                .runtime
                .buildings
                .iter()
                .find(|building| building.tile_pos == *tile_pos)
                .cloned()
            else {
                continue;
            };
            let Some(BlockDef::UnitAssembler(assembler_block)) =
                self.content_loader.block(building.block.id)
            else {
                continue;
            };
            let tracked_drones = self
                .runtime
                .unit_runtime_states
                .get(tile_pos)
                .and_then(|state| match state {
                    GameRuntimeUnitBlockState::Assembler { assembler, .. } => Some(
                        assembler
                            .read_unit_ids
                            .iter()
                            .filter(|id| **id >= 0)
                            .count(),
                    ),
                    _ => None,
                })
                .unwrap_or(0);
            if tracked_drones >= assembler_block.drones_created.max(0) as usize {
                continue;
            }
            let Some(unit_type) = self
                .content_loader
                .unit_by_name(&assembler_block.drone_type)
                .cloned()
            else {
                continue;
            };

            let unit_id = self.next_server_runtime_unit_id();
            let mut unit = UnitComp::new(unit_id, unit_type, building.team);
            unit.set_pos(building.x, building.y);
            unit.set_rotation(90.0);
            unit.set_controller(UnitControllerState::Assembler);
            unit.building_tether = Some(BuildingTetherComp {
                team: building.team,
                building: Some(BuildingTetherRef {
                    tile_pos: *tile_pos,
                    team: building.team,
                    valid: building.is_valid(),
                }),
            });
            self.server_units.insert(unit_id, unit);

            if let Some(GameRuntimeUnitBlockState::Assembler { assembler, .. }) =
                self.runtime.unit_runtime_states.get_mut(tile_pos)
            {
                unit_assembler_drone_spawned(assembler, unit_id, true);
            }

            if self.net_server.is_active() {
                let packet =
                    PacketKind::AssemblerDroneSpawnedCallPacket(AssemblerDroneSpawnedCallPacket {
                        tile: Some(*tile_pos),
                        id: unit_id,
                    });
                if let Err(error) = self.net_server.net_mut().send(&packet, true) {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
            applied += 1;
        }

        if let Some(error) = first_error {
            Err(error)
        } else {
            Ok(applied)
        }
    }

    fn tick_runtime_unit_assembler_ai(&mut self) -> usize {
        let mut targets = Vec::new();
        for building in &self.runtime.buildings {
            let Some(BlockDef::UnitAssembler(assembler_block)) =
                self.content_loader.block(building.block.id)
            else {
                continue;
            };
            let Some(GameRuntimeUnitBlockState::Assembler { assembler, .. }) =
                self.runtime.unit_runtime_states.get(&building.tile_pos)
            else {
                continue;
            };
            let (dir_x, dir_y) = autotiler_direction(building.rotation);
            let spawn_len = TILE_SIZE as f32
                * (assembler_block.area_size + assembler_block.base.size) as f32
                / 2.0;
            let spawn_x = building.x + dir_x as f32 * spawn_len;
            let spawn_y = building.y + dir_y as f32 * spawn_len;
            let mut seen = BTreeSet::new();
            for (slot_index, unit_id) in assembler
                .read_unit_ids
                .iter()
                .copied()
                .filter(|id| *id >= 0 && seen.insert(*id))
                .take(assembler_block.drones_created.max(0) as usize)
                .enumerate()
            {
                targets.push((
                    unit_id,
                    building.tile_pos,
                    building.team,
                    unit_assembler_drone_target(
                        spawn_x,
                        spawn_y,
                        assembler_block.area_size,
                        TILE_SIZE as f32,
                        slot_index,
                    ),
                ));
            }
        }

        let mut updated_snapshots = Vec::new();
        for (unit_id, tile_pos, team, target) in targets {
            let Some(unit) = self.server_units.get_mut(&unit_id) else {
                continue;
            };
            if unit.team_id() != team || !matches!(unit.controller, UnitControllerState::Assembler)
            {
                continue;
            }
            let tether_valid = unit
                .building_tether
                .as_ref()
                .and_then(|tether| tether.building.as_ref())
                .is_some_and(|building| building.tile_pos == tile_pos && building.valid);
            if !tether_valid {
                continue;
            }

            let dx = target.pos.x - unit.x();
            let dy = target.pos.y - unit.y();
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > f32::EPSILON {
                let travel = (unit.type_info.speed * 3.0).max(1.0).min(distance);
                let scale = travel / distance;
                unit.set_pos(unit.x() + dx * scale, unit.y() + dy * scale);
            }
            if distance <= 5.0 {
                unit.look_at_angle(target.angle, 1.0);
            }
            updated_snapshots.push((unit_id, unit.clone()));
        }

        let updated = updated_snapshots.len();
        for (unit_id, unit) in updated_snapshots {
            self.runtime
                .client_unit_snapshot_entities
                .insert(unit_id, unit);
        }
        updated
    }

    fn broadcast_runtime_unit_block_spawns(&mut self, tile_positions: &[i32]) -> io::Result<usize> {
        if tile_positions.is_empty() || !self.net_server.is_active() {
            return Ok(0);
        }

        let mut sent = 0;
        let mut first_error = None;
        let mut seen = BTreeSet::new();
        for tile_pos in tile_positions {
            if !seen.insert(*tile_pos) {
                continue;
            }
            let packet = PacketKind::UnitBlockSpawnCallPacket(UnitBlockSpawnCallPacket {
                tile: Some(*tile_pos),
            });
            match self.net_server.net_mut().send(&packet, true) {
                Ok(()) => sent += 1,
                Err(error) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }

        if let Some(error) = first_error {
            Err(error)
        } else {
            Ok(sent)
        }
    }

    fn broadcast_runtime_assembler_unit_spawns(
        &mut self,
        tile_positions: &[i32],
    ) -> io::Result<usize> {
        if tile_positions.is_empty() || !self.net_server.is_active() {
            return Ok(0);
        }

        let mut sent = 0;
        let mut first_error = None;
        let mut seen = BTreeSet::new();
        for tile_pos in tile_positions {
            if !seen.insert(*tile_pos) {
                continue;
            }
            let packet =
                PacketKind::AssemblerUnitSpawnedCallPacket(AssemblerUnitSpawnedCallPacket {
                    tile: Some(*tile_pos),
                });
            match self.net_server.net_mut().send(&packet, true) {
                Ok(()) => sent += 1,
                Err(error) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }

        if let Some(error) = first_error {
            Err(error)
        } else {
            Ok(sent)
        }
    }

    fn broadcast_runtime_landing_pad_landed(
        &mut self,
        tile_positions: &[i32],
    ) -> io::Result<usize> {
        if tile_positions.is_empty() || !self.net_server.is_active() {
            return Ok(0);
        }

        let mut sent = 0;
        let mut first_error = None;
        let mut seen = BTreeSet::new();
        for tile_pos in tile_positions {
            if !seen.insert(*tile_pos) {
                continue;
            }
            let packet = PacketKind::LandingPadLandedCallPacket(LandingPadLandedCallPacket {
                tile: Some(*tile_pos),
            });
            match self.net_server.net_mut().send(&packet, true) {
                Ok(()) => sent += 1,
                Err(error) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }

        if let Some(error) = first_error {
            Err(error)
        } else {
            Ok(sent)
        }
    }

    fn broadcast_server_unit_spawn(&mut self, unit: &UnitComp) -> io::Result<bool> {
        if !self.net_server.is_active() {
            return Ok(false);
        }
        let Some(packet) = self.server_unit_spawn_packet(unit)? else {
            return Ok(false);
        };
        self.net_server
            .net_mut()
            .send(&PacketKind::UnitSpawnCallPacket(packet), false)?;
        Ok(true)
    }

    fn broadcast_server_effect_with_data(
        &mut self,
        effect: &str,
        x: f32,
        y: f32,
        rotation: f32,
        data: TypeValue,
    ) -> io::Result<bool> {
        if !self.net_server.is_active() {
            return Ok(false);
        }
        let Some(effect_id) = standard_effect_id(effect) else {
            return Ok(false);
        };
        self.net_server.net_mut().send(
            &PacketKind::EffectCallPacket2(EffectCallPacket2 {
                effect: EffectCallPacket {
                    effect_id: effect_id as u16,
                    x,
                    y,
                    rotation,
                    color: type_io::RgbaColor::new(-1),
                },
                data,
            }),
            false,
        )?;
        Ok(true)
    }

    fn broadcast_server_effect_colored(
        &mut self,
        effect: &str,
        x: f32,
        y: f32,
        rotation: f32,
        color_rgba: u32,
    ) -> io::Result<bool> {
        if !self.net_server.is_active() {
            return Ok(false);
        }
        let Some(effect_id) = standard_effect_id(effect) else {
            return Ok(false);
        };
        self.net_server.net_mut().send(
            &PacketKind::EffectCallPacket(EffectCallPacket {
                effect_id: effect_id as u16,
                x,
                y,
                rotation,
                color: type_io::RgbaColor::new(color_rgba as i32),
            }),
            false,
        )?;
        Ok(true)
    }

    fn broadcast_server_hidden_snapshot(&mut self, ids: &[i32]) -> io::Result<bool> {
        if ids.is_empty() || !self.net_server.is_active() {
            return Ok(false);
        }
        self.net_server.net_mut().send(
            &PacketKind::HiddenSnapshotCallPacket(HiddenSnapshotCallPacket { ids: ids.to_vec() }),
            false,
        )?;
        Ok(true)
    }

    fn server_unit_spawn_packet(&self, unit: &UnitComp) -> io::Result<Option<UnitSpawnCallPacket>> {
        let Some(unit_type_id) = entity_class_id(unit.type_info.name()) else {
            return Ok(None);
        };
        let mut sync = Vec::new();
        type_io::write_unit_sync(&mut sync, &self.content_loader, &unit.to_sync_wire())?;
        Ok(Some(UnitSpawnCallPacket {
            container: type_io::UnitSyncContainer::new(unit.id(), unit_type_id, sync),
        }))
    }

    fn broadcast_server_unit_entity_snapshots(&mut self) -> io::Result<usize> {
        if !self.net_server.is_active() {
            return Ok(0);
        }
        let packet = self.server_unit_entity_snapshot_packet()?;
        if packet.amount <= 0 {
            return Ok(0);
        }
        self.net_server
            .net_mut()
            .send(&PacketKind::EntitySnapshotCallPacket(packet), false)?;
        Ok(1)
    }

    fn server_unit_entity_snapshot_packet(&self) -> io::Result<EntitySnapshotCallPacket> {
        let mut data = Vec::new();
        let mut amount: i16 = 0;
        for unit in self
            .server_units
            .values()
            .filter(|unit| unit.controller.is_cargo())
        {
            let Some(type_id) = entity_class_id(unit.type_info.name()) else {
                continue;
            };
            if amount == i16::MAX {
                break;
            }
            data.extend_from_slice(&unit.id().to_be_bytes());
            data.push(type_id);
            type_io::write_unit_sync(&mut data, &self.content_loader, &unit.to_sync_wire())?;
            amount += 1;
        }
        for (_tile, entry) in self.runtime.server_puddles.entries() {
            if amount == i16::MAX {
                break;
            }
            if entry.puddle.removed || entry.puddle.amount <= 0.0 {
                continue;
            }
            let Some(liquid_id) = self
                .content_loader
                .liquid_by_name(&entry.liquid.name)
                .map(|liquid| liquid.base.mappable.base.id)
            else {
                continue;
            };
            let tile_pos = entry.puddle.tile.map(|tile| point2_pack(tile.x, tile.y));
            data.extend_from_slice(&entry.puddle.id.to_be_bytes());
            data.push(PUDDLE_CLASS_ID);
            type_io::write_puddle_sync(
                &mut data,
                &type_io::PuddleSyncWire {
                    amount: entry.puddle.amount,
                    liquid_id: Some(liquid_id),
                    tile_pos,
                    x: entry.puddle.x,
                    y: entry.puddle.y,
                },
            )?;
            amount += 1;
        }
        for ((x, y), fire) in self.runtime.server_fires.entries() {
            if amount == i16::MAX {
                break;
            }
            if fire.removed || fire.lifetime <= 0.0 || fire.time >= fire.lifetime {
                continue;
            }
            data.extend_from_slice(&server_fire_entity_id(*x, *y).to_be_bytes());
            data.push(FIRE_CLASS_ID);
            type_io::write_fire_sync(
                &mut data,
                &type_io::FireSyncWire {
                    lifetime: fire.lifetime,
                    tile_pos: fire.tile.map(|tile| point2_pack(tile.x, tile.y)),
                    time: fire.time,
                    x: fire.x,
                    y: fire.y,
                },
            )?;
            amount += 1;
        }
        Ok(EntitySnapshotCallPacket { amount, data })
    }

    fn tick_runtime_unit_cargo_ai(&mut self) -> io::Result<usize> {
        let linked_units: Vec<_> = self
            .runtime
            .distribution_runtime_states
            .iter()
            .filter_map(|(&tile_pos, state)| match state {
                GameRuntimeDistributionBlockState::UnitCargoLoader(state)
                    if state.has_unit && state.read_unit_id >= 0 =>
                {
                    Some((tile_pos, state.read_unit_id))
                }
                _ => None,
            })
            .collect();

        let mut applied = 0;
        for (loader_tile_pos, unit_id) in linked_units {
            applied += self.tick_runtime_unit_cargo_ai_for_loader(loader_tile_pos, unit_id)?;
        }
        Ok(applied)
    }

    fn tick_runtime_unit_cargo_ai_for_loader(
        &mut self,
        loader_tile_pos: i32,
        unit_id: i32,
    ) -> io::Result<usize> {
        let Some(loader_index) = self
            .runtime
            .buildings
            .iter()
            .position(|building| building.tile_pos == loader_tile_pos)
        else {
            self.remove_runtime_unit_cargo_loader_unit(loader_tile_pos, unit_id)?;
            return Ok(0);
        };

        let (loader_x, loader_y, loader_team, loader_valid, loader_has_items) = {
            let building = &self.runtime.buildings[loader_index];
            (
                building.x,
                building.y,
                building.team,
                building.is_valid(),
                building.items.is_some(),
            )
        };
        if !self.server_units.contains_key(&unit_id) {
            self.clear_runtime_unit_cargo_loader_state(loader_tile_pos);
            return Ok(0);
        }
        if let Some(unit) = self.server_units.get_mut(&unit_id) {
            unit.building_tether = Some(BuildingTetherComp {
                team: unit.team_id(),
                building: Some(BuildingTetherRef {
                    tile_pos: loader_tile_pos,
                    team: loader_team,
                    valid: loader_valid,
                }),
            });
        }
        let Some(unit_snapshot) = self.server_units.get(&unit_id).cloned() else {
            self.clear_runtime_unit_cargo_loader_state(loader_tile_pos);
            return Ok(0);
        };
        if unit_snapshot
            .building_tether
            .as_ref()
            .is_some_and(|tether| tether.update() == BuildingTetherAction::Despawn)
        {
            self.remove_runtime_unit_cargo_loader_unit(loader_tile_pos, unit_id)?;
            return Ok(0);
        }
        if !loader_has_items {
            return Ok(0);
        }

        if !unit_snapshot.controller.is_cargo() {
            if let Some(unit) = self.server_units.get_mut(&unit_id) {
                unit.set_controller(UnitControllerState::Cargo);
            }
        }
        if let Some(unit) = self.server_units.get_mut(&unit_id) {
            let cargo = unit
                .cargo_ai
                .get_or_insert_with(|| CargoAiRuntimeState::new(Some(loader_tile_pos)));
            cargo.tether_tile_pos = Some(loader_tile_pos);
        }

        if !unit_snapshot.items.has_item() {
            let within_loader = self
                .move_runtime_unit_cargo_towards(unit_id, loader_x, loader_y)
                .unwrap_or(false);
            if !within_loader {
                return Ok(0);
            }
            if !self.runtime_unit_cargo_retarget_ready(unit_id, loader_tile_pos) {
                return Ok(0);
            }
            let unit_capacity = unit_snapshot.items.item_capacity();
            let target_index = unit_snapshot
                .cargo_ai
                .as_ref()
                .map(|state| state.target_index)
                .unwrap_or(0);
            let Some((item_id, item_name, amount, target_tile_pos, next_target_index)) = self
                .find_runtime_unit_cargo_pickup_plan(
                    loader_index,
                    loader_team,
                    unit_capacity,
                    target_index,
                )
            else {
                if let Some(unit) = self.server_units.get_mut(&unit_id) {
                    if let Some(cargo) = unit.cargo_ai.as_mut() {
                        cargo.unload_target_tile_pos = None;
                        cargo.item_target = None;
                    }
                }
                return Ok(0);
            };

            if let Some(unit) = self.server_units.get_mut(&unit_id) {
                unit.set_rotation(90.0);
                let cargo = unit
                    .cargo_ai
                    .get_or_insert_with(|| CargoAiRuntimeState::new(Some(loader_tile_pos)));
                cargo.tether_tile_pos = Some(loader_tile_pos);
                cargo.unload_target_tile_pos = Some(target_tile_pos);
                cargo.item_target = Some(item_name.clone());
                cargo.target_index = next_target_index;
                cargo.no_dest_timer = 0.0;
                cargo.drop_timer = CARGO_AI_DROP_SPACING;
            }

            let outcome = take_items(
                self.runtime.buildings_mut().get_mut(loader_index),
                Some(item_name.clone()),
                amount,
                self.server_units.get_mut(&unit_id),
                |name| {
                    if name == item_name.as_str() {
                        Some(item_id)
                    } else {
                        None
                    }
                },
            );
            if let Some(remove_stack) = outcome.remove_stack.as_ref() {
                self.runtime
                    .apply_item_remove_stack_plan(&self.content_loader, remove_stack);
            }
            if outcome.accepted {
                if let Some(packet) = outcome.packet.as_ref() {
                    if self.net_server.is_active() {
                        self.net_server
                            .net_mut()
                            .send(&PacketKind::TakeItemsCallPacket(packet.clone()), false)?;
                        self.runtime_take_items_packets_sent += 1;
                    }
                }
                if self.net_server.is_active() {
                    for packet in &outcome.transfer_effects {
                        self.net_server.net_mut().send(
                            &PacketKind::TransferItemEffectCallPacket(packet.clone()),
                            false,
                        )?;
                        self.runtime_transfer_item_effect_packets_sent += 1;
                    }
                }
                self.last_runtime_take_items_outcome = Some(outcome);
                return Ok(1);
            }
            self.last_runtime_take_items_outcome = Some(outcome);
            return Ok(0);
        }

        let Some(item_name) = unit_snapshot.items.item().map(str::to_owned) else {
            return Ok(0);
        };
        let Some(item_id) = self
            .content_loader
            .item_by_name(&item_name)
            .map(|item| item.base.mappable.base.id as ContentId)
        else {
            if let Some(unit) = self.server_units.get_mut(&unit_id) {
                unit.items.clear_item();
                if let Some(cargo) = unit.cargo_ai.as_mut() {
                    cargo.unload_target_tile_pos = None;
                    cargo.item_target = None;
                }
            }
            return Ok(0);
        };

        let cargo_snapshot = unit_snapshot
            .cargo_ai
            .clone()
            .unwrap_or_else(|| CargoAiRuntimeState::new(Some(loader_tile_pos)));
        let (target_tile_pos, target_building_index, target_index_for_state) =
            if let Some(existing_target_tile_pos) = cargo_snapshot.unload_target_tile_pos {
                let Some(target_building_index) = self.runtime_unit_cargo_drop_target_for_tile(
                    item_id,
                    loader_team,
                    existing_target_tile_pos,
                ) else {
                    if let Some(unit) = self.server_units.get_mut(&unit_id) {
                        let cargo = unit
                            .cargo_ai
                            .get_or_insert_with(|| CargoAiRuntimeState::new(Some(loader_tile_pos)));
                        cargo.unload_target_tile_pos = None;
                        cargo.item_target = None;
                        cargo.no_dest_timer = 0.0;
                    }
                    return Ok(0);
                };
                (
                    existing_target_tile_pos,
                    target_building_index,
                    cargo_snapshot.target_index,
                )
            } else {
                if !self.runtime_unit_cargo_retarget_ready(unit_id, loader_tile_pos) {
                    return Ok(0);
                }
                let Some((target_tile_pos, _target_building_index, next_target_index)) =
                    self.find_runtime_unit_cargo_drop_target(item_id, loader_team, 0, None)
                else {
                    if let Some(unit) = self.server_units.get_mut(&unit_id) {
                        unit.items.clear_item();
                        if let Some(cargo) = unit.cargo_ai.as_mut() {
                            cargo.unload_target_tile_pos = None;
                            cargo.item_target = None;
                            cargo.no_dest_timer = 0.0;
                            cargo.drop_timer = CARGO_AI_DROP_SPACING;
                        }
                    }
                    return Ok(0);
                };
                if let Some(unit) = self.server_units.get_mut(&unit_id) {
                    let cargo = unit
                        .cargo_ai
                        .get_or_insert_with(|| CargoAiRuntimeState::new(Some(loader_tile_pos)));
                    cargo.tether_tile_pos = Some(loader_tile_pos);
                    cargo.unload_target_tile_pos = Some(target_tile_pos);
                    cargo.item_target = Some(item_name.clone());
                    cargo.target_index = next_target_index;
                    cargo.no_dest_timer = 0.0;
                }
                return Ok(0);
            };

        let (target_x, target_y) = {
            let target = &self.runtime.buildings[target_building_index];
            (target.x, target.y)
        };
        let within_target = self
            .move_runtime_unit_cargo_towards(unit_id, target_x, target_y)
            .unwrap_or(false);
        if !within_target {
            return Ok(0);
        }

        let drop_ready = if let Some(unit) = self.server_units.get_mut(&unit_id) {
            let cargo = unit
                .cargo_ai
                .get_or_insert_with(|| CargoAiRuntimeState::new(Some(loader_tile_pos)));
            cargo.tether_tile_pos = Some(loader_tile_pos);
            cargo.unload_target_tile_pos = Some(target_tile_pos);
            cargo.item_target = Some(item_name.clone());
            cargo.drop_timer += 1.0;
            if cargo.drop_timer + f32::EPSILON < CARGO_AI_DROP_SPACING {
                false
            } else {
                cargo.drop_timer = 0.0;
                true
            }
        } else {
            false
        };
        if !drop_ready {
            return Ok(0);
        }

        let accepted_amount = {
            let target = &self.runtime.buildings[target_building_index];
            Self::building_accept_stack_amount(target, item_id, unit_snapshot.items.stack.amount)
        };
        if accepted_amount <= 0 {
            let mut retarget_from_index = None;
            if let Some(unit) = self.server_units.get_mut(&unit_id) {
                let cargo = unit
                    .cargo_ai
                    .get_or_insert_with(|| CargoAiRuntimeState::new(Some(loader_tile_pos)));
                cargo.unload_target_tile_pos = Some(target_tile_pos);
                cargo.item_target = Some(item_name);
                cargo.no_dest_timer += CARGO_AI_DROP_SPACING;
                if cargo.no_dest_timer >= CARGO_AI_EMPTY_WAIT_TIME {
                    retarget_from_index = Some(cargo.target_index);
                }
            }
            if let Some(target_index) = retarget_from_index {
                if let Some((next_tile_pos, _, next_target_index)) = self
                    .find_runtime_unit_cargo_drop_target(item_id, loader_team, target_index, None)
                {
                    if let Some(unit) = self.server_units.get_mut(&unit_id) {
                        let cargo = unit
                            .cargo_ai
                            .get_or_insert_with(|| CargoAiRuntimeState::new(Some(loader_tile_pos)));
                        cargo.unload_target_tile_pos = Some(next_tile_pos);
                        cargo.item_target = Some(
                            self.content_loader
                                .item(item_id)
                                .map(|item| item.name().to_string())
                                .unwrap_or_default(),
                        );
                        cargo.target_index = next_target_index;
                        cargo.no_dest_timer = 0.0;
                    }
                } else if let Some(unit) = self.server_units.get_mut(&unit_id) {
                    unit.items.clear_item();
                    if let Some(cargo) = unit.cargo_ai.as_mut() {
                        cargo.unload_target_tile_pos = None;
                        cargo.item_target = None;
                        cargo.no_dest_timer = 0.0;
                        cargo.drop_timer = CARGO_AI_DROP_SPACING;
                    }
                }
            }
            return Ok(0);
        }

        if let Some(unit) = self.server_units.get_mut(&unit_id) {
            let cargo = unit
                .cargo_ai
                .get_or_insert_with(|| CargoAiRuntimeState::new(Some(loader_tile_pos)));
            cargo.unload_target_tile_pos = Some(target_tile_pos);
            cargo.item_target = Some(item_name.clone());
            cargo.target_index = target_index_for_state;
            cargo.no_dest_timer = 0.0;
        }

        let (unit_x, unit_y) = self
            .server_units
            .get(&unit_id)
            .map(|unit| (unit.x(), unit.y()))
            .unwrap_or((loader_x, loader_y));
        let outcome = transfer_item_to(
            self.server_units.get_mut(&unit_id),
            Some(item_name.clone()),
            accepted_amount,
            unit_x,
            unit_y,
            self.runtime.buildings_mut().get_mut(target_building_index),
            |name| {
                if name == item_name.as_str() {
                    Some(item_id)
                } else {
                    None
                }
            },
        );
        if outcome.accepted {
            if let Some(packet) = outcome.packet.as_ref() {
                self.runtime.apply_item_handle_stack_side_effects(
                    &self.content_loader,
                    packet.build,
                    item_id,
                    packet.amount,
                );
                if self.net_server.is_active() {
                    self.net_server
                        .net_mut()
                        .send(&PacketKind::TransferItemToCallPacket(packet.clone()), false)?;
                    self.runtime_transfer_item_to_packets_sent += 1;
                }
            }
            let mut retarget_after_transfer_from_index = None;
            if let Some(unit) = self.server_units.get_mut(&unit_id) {
                let cargo = unit
                    .cargo_ai
                    .get_or_insert_with(|| CargoAiRuntimeState::new(Some(loader_tile_pos)));
                if unit.items.stack.amount == 0 {
                    cargo.unload_target_tile_pos = None;
                    cargo.item_target = None;
                    cargo.no_dest_timer = 0.0;
                    cargo.drop_timer = CARGO_AI_DROP_SPACING;
                } else {
                    cargo.no_dest_timer += CARGO_AI_DROP_SPACING;
                    if cargo.no_dest_timer >= CARGO_AI_EMPTY_WAIT_TIME {
                        retarget_after_transfer_from_index = Some(cargo.target_index);
                        cargo.no_dest_timer = 0.0;
                    }
                }
            }
            if let Some(target_index) = retarget_after_transfer_from_index {
                if let Some((next_tile_pos, _, next_target_index)) = self
                    .find_runtime_unit_cargo_drop_target(item_id, loader_team, target_index, None)
                {
                    if let Some(unit) = self.server_units.get_mut(&unit_id) {
                        let cargo = unit
                            .cargo_ai
                            .get_or_insert_with(|| CargoAiRuntimeState::new(Some(loader_tile_pos)));
                        cargo.unload_target_tile_pos = Some(next_tile_pos);
                        cargo.item_target = Some(item_name.clone());
                        cargo.target_index = next_target_index;
                    }
                } else if let Some(unit) = self.server_units.get_mut(&unit_id) {
                    unit.items.clear_item();
                    if let Some(cargo) = unit.cargo_ai.as_mut() {
                        cargo.unload_target_tile_pos = None;
                        cargo.item_target = None;
                        cargo.no_dest_timer = 0.0;
                        cargo.drop_timer = CARGO_AI_DROP_SPACING;
                    }
                }
            }
            self.last_runtime_transfer_item_to_outcome = Some(outcome);
            return Ok(1);
        }
        self.last_runtime_transfer_item_to_outcome = Some(outcome);
        Ok(0)
    }

    fn find_runtime_unit_cargo_pickup_plan(
        &self,
        loader_index: usize,
        team: TeamId,
        unit_capacity: i32,
        target_index: usize,
    ) -> Option<(ContentId, String, i32, i32, usize)> {
        let building = self.runtime.buildings.get(loader_index)?;
        let items = building.items.as_ref()?;
        let mut item_entries: Vec<_> = items
            .each()
            .filter(|(_, amount)| *amount > 0)
            .map(|(item_id, amount)| (item_id as ContentId, amount))
            .collect();
        item_entries.sort_by(|(left_id, left_amount), (right_id, right_amount)| {
            right_amount
                .cmp(left_amount)
                .then_with(|| left_id.cmp(right_id))
        });

        let mut stale_fallback = None;
        for (item_id, amount) in item_entries {
            let targets = self.runtime_unit_cargo_drop_targets(item_id, team, None);
            if targets.is_empty() {
                continue;
            }
            let item_name = self.content_loader.item(item_id)?.name().to_string();
            let transfer_amount = amount.min(unit_capacity).max(0);
            if transfer_amount <= 0 {
                continue;
            }
            for i in 0..targets.len() {
                let index = (i + target_index) % targets.len();
                let (target_tile_pos, _, stale) = targets[index];
                stale_fallback = Some((
                    item_id,
                    item_name.clone(),
                    transfer_amount,
                    target_tile_pos,
                    index,
                ));
                if !stale {
                    return Some((item_id, item_name, transfer_amount, target_tile_pos, index));
                }
            }
        }
        stale_fallback
    }

    fn runtime_unit_cargo_drop_targets(
        &self,
        item_id: ContentId,
        team: TeamId,
        ignore_tile_pos: Option<i32>,
    ) -> Vec<(i32, usize, bool)> {
        let mut targets = Vec::new();
        for (building_index, building) in self.runtime.buildings.iter().enumerate() {
            if building.team != team || Some(building.tile_pos) == ignore_tile_pos {
                continue;
            }
            let Some(BlockDef::Distribution(distribution)) =
                self.content_loader.block(building.block.id)
            else {
                continue;
            };
            if distribution.kind != DistributionBlockKind::UnitCargoUnloadPoint {
                continue;
            }
            let Some(GameRuntimeDistributionBlockState::UnitCargoUnload(state)) = self
                .runtime
                .distribution_runtime_states
                .get(&building.tile_pos)
            else {
                continue;
            };
            if state.item_id == Some(item_id as i32) {
                targets.push((building.tile_pos, building_index, state.stale));
            }
        }
        targets
    }

    fn find_runtime_unit_cargo_drop_target(
        &self,
        item_id: ContentId,
        team: TeamId,
        offset: usize,
        ignore_tile_pos: Option<i32>,
    ) -> Option<(i32, usize, usize)> {
        let targets = self.runtime_unit_cargo_drop_targets(item_id, team, ignore_tile_pos);
        if targets.is_empty() {
            return None;
        }

        let start = if targets.is_empty() {
            0
        } else {
            (offset + 1) % targets.len()
        };
        for i in 0..targets.len() {
            let index = (start + i) % targets.len();
            let (tile_pos, building_index, stale) = targets[index];
            if !stale {
                return Some((tile_pos, building_index, index));
            }
        }

        let (tile_pos, building_index, _) = targets[0];
        Some((tile_pos, building_index, 0))
    }

    fn runtime_unit_cargo_drop_target_for_tile(
        &self,
        item_id: ContentId,
        team: TeamId,
        tile_pos: i32,
    ) -> Option<usize> {
        let building_index = self
            .runtime
            .buildings
            .iter()
            .position(|building| building.tile_pos == tile_pos)?;
        let building = &self.runtime.buildings[building_index];
        if building.team != team || !building.is_valid() {
            return None;
        }
        let Some(BlockDef::Distribution(distribution)) =
            self.content_loader.block(building.block.id)
        else {
            return None;
        };
        if distribution.kind != DistributionBlockKind::UnitCargoUnloadPoint {
            return None;
        }
        let Some(GameRuntimeDistributionBlockState::UnitCargoUnload(state)) =
            self.runtime.distribution_runtime_states.get(&tile_pos)
        else {
            return None;
        };
        if state.item_id == Some(item_id as i32) {
            Some(building_index)
        } else {
            None
        }
    }

    fn clear_runtime_unit_cargo_loader_state(&mut self, tile_pos: i32) {
        if let Some(GameRuntimeDistributionBlockState::UnitCargoLoader(state)) =
            self.runtime.distribution_runtime_states.get_mut(&tile_pos)
        {
            state.has_unit = false;
            state.read_unit_id = -1;
        }
    }

    fn remove_runtime_unit_cargo_loader_unit(
        &mut self,
        tile_pos: i32,
        unit_id: i32,
    ) -> io::Result<()> {
        let removed = self.server_units.remove(&unit_id).is_some();
        self.clear_runtime_unit_cargo_loader_state(tile_pos);
        if removed && self.net_server.is_active() {
            self.net_server.net_mut().send(
                &PacketKind::UnitDespawnCallPacket(UnitDespawnCallPacket {
                    unit: UnitRef::Unit { id: unit_id },
                }),
                false,
            )?;
        }
        Ok(())
    }

    fn server_unit_enter_payload_build_tile_pos(&self, unit: &UnitComp) -> Option<i32> {
        if !unit.type_info.allowed_in_payloads {
            return None;
        }
        let UnitControllerState::Command(command) = &unit.controller else {
            return None;
        };
        let enter_payload_id = self
            .content_loader
            .unit_command_by_name("enterPayload")?
            .id();
        if command.command_id != Some(enter_payload_id) {
            return None;
        }

        let build_tile_pos = self.server_unit_build_on_tile_pos(unit)?;
        if !self.server_enter_payload_target_matches_build(command.target_pos, build_tile_pos) {
            return None;
        }
        Some(build_tile_pos)
    }

    fn server_unit_build_on_tile_pos(&self, unit: &UnitComp) -> Option<i32> {
        let tile_x = World::to_tile(unit.x());
        let tile_y = World::to_tile(unit.y());
        self.runtime
            .state
            .world
            .build_world(unit.x(), unit.y())
            .map(|build| build.tile_pos)
            .or_else(|| {
                let tile_pos = point2_pack(World::to_tile(unit.x()), World::to_tile(unit.y()));
                self.runtime
                    .buildings
                    .iter()
                    .find(|building| building.tile_pos == tile_pos)
                    .map(|building| building.tile_pos)
            })
            .or_else(|| {
                self.runtime
                    .buildings
                    .iter()
                    .find(|building| {
                        footprint_tiles(building.tile_x(), building.tile_y(), building.block.size)
                            .contains(&(tile_x, tile_y))
                    })
                    .map(|building| building.tile_pos)
            })
    }

    fn server_enter_payload_target_matches_build(
        &self,
        target_pos: Option<mindustry_core::mindustry::io::Vec2>,
        build_tile_pos: i32,
    ) -> bool {
        let Some(target_pos) = target_pos else {
            return true;
        };
        self.runtime
            .state
            .world
            .build_world(target_pos.x, target_pos.y)
            .map(|build| build.tile_pos == build_tile_pos)
            .unwrap_or_else(|| {
                point2_pack(World::to_tile(target_pos.x), World::to_tile(target_pos.y))
                    == build_tile_pos
            })
    }

    fn default_server_unit_type(&self) -> Option<UnitType> {
        self.content_loader
            .unit_by_name("alpha")
            .cloned()
            .or_else(|| {
                self.content_loader
                    .units()
                    .iter()
                    .find(|unit| unit.resolved_item_capacity() > 0)
                    .cloned()
            })
    }

    fn player_within_payload_pickup_range(player: &PlayerComp, build: &BuildingComp) -> bool {
        // The launcher mirror still bootstraps player/unit position from network
        // identity, not the full synced entity stream. Match the item-transfer
        // bridge behavior: keep Java's distance formula whenever coordinates are
        // known, but do not reject zeroed bootstrap players before entity sync is
        // fully authoritative.
        if player.x.abs() <= f32::EPSILON && player.y.abs() <= f32::EPSILON {
            return true;
        }
        let range = TILE_SIZE as f32 * build.block.size as f32 * 1.2 + TILE_SIZE as f32 * 5.0;
        let dx = player.x - build.x;
        let dy = player.y - build.y;
        dx * dx + dy * dy <= range * range
    }

    fn unit_payload_target_within_range(unit: &UnitComp, target: &UnitComp) -> bool {
        if unit.x().abs() <= f32::EPSILON && unit.y().abs() <= f32::EPSILON {
            return true;
        }
        let range = unit.type_info.hit_size * 2.0 + target.type_info.hit_size * 2.0;
        let dx = target.x() - unit.x();
        let dy = target.y() - unit.y();
        dx * dx + dy * dy <= range * range
    }

    fn player_within_item_transfer_range(player: &PlayerComp, build: &BuildingComp) -> bool {
        // The current Rust server does not yet receive authoritative player/unit
        // position snapshots. Do not reject otherwise valid inventory requests
        // solely because the launcher-side mirror is still at its zeroed
        // bootstrap position; once player sync is fully wired, this fallback
        // should be removed in favor of strict Java `player.within(...)`.
        if player.x.abs() <= f32::EPSILON && player.y.abs() <= f32::EPSILON {
            return true;
        }
        let dx = player.x - build.x;
        let dy = player.y - build.y;
        dx * dx + dy * dy <= ITEM_TRANSFER_RANGE * ITEM_TRANSFER_RANGE
    }

    fn server_build_can_pickup(&self, build: &BuildingComp) -> bool {
        let Some(block_def) = self.content_loader.block(build.block.id) else {
            return true;
        };

        match block_def {
            BlockDef::Storage(storage) if storage.kind == StorageBlockKind::Core => false,
            BlockDef::Storage(_) => !self
                .runtime
                .storage_linked_cores
                .contains_key(&build.tile_pos),
            BlockDef::Logic(logic) => logic.can_pickup,
            BlockDef::Effect(effect) if effect.kind == EffectBlockKind::Radar => false,
            _ => true,
        }
    }

    fn runtime_payload_ref_for_tile(&self, tile_pos: i32) -> Option<&PayloadRef> {
        match self.runtime.payload_runtime_states.get(&tile_pos)? {
            GameRuntimePayloadBlockState::MassDriver { common, .. }
            | GameRuntimePayloadBlockState::Loader { common, .. }
            | GameRuntimePayloadBlockState::Source { common, .. }
            | GameRuntimePayloadBlockState::Deconstructor { common, .. }
            | GameRuntimePayloadBlockState::Constructor { common, .. }
            | GameRuntimePayloadBlockState::Void(common) => common.payload.as_ref(),
            GameRuntimePayloadBlockState::Conveyor(conveyor) => conveyor.item.as_ref(),
            GameRuntimePayloadBlockState::Router { conveyor, .. } => conveyor.item.as_ref(),
        }
    }

    fn take_runtime_payload_ref_for_tile(&mut self, tile_pos: i32) -> Option<PayloadRef> {
        match self.runtime.payload_runtime_states.get_mut(&tile_pos)? {
            GameRuntimePayloadBlockState::MassDriver { common, .. }
            | GameRuntimePayloadBlockState::Loader { common, .. }
            | GameRuntimePayloadBlockState::Source { common, .. }
            | GameRuntimePayloadBlockState::Deconstructor { common, .. }
            | GameRuntimePayloadBlockState::Constructor { common, .. }
            | GameRuntimePayloadBlockState::Void(common) => common.payload.take(),
            GameRuntimePayloadBlockState::Conveyor(conveyor) => conveyor.item.take(),
            GameRuntimePayloadBlockState::Router { conveyor, .. } => conveyor.item.take(),
        }
    }

    fn payload_ref_to_payload_state(&self, payload: &PayloadRef) -> Option<PayloadState> {
        match payload {
            PayloadRef::Block { block, .. } => {
                let size = self.content_loader.block(*block)?.base().size as f32 * TILE_SIZE as f32;
                Some(PayloadState {
                    kind: PayloadKind::Build,
                    size,
                })
            }
            PayloadRef::Unit { .. } => {
                let key = payload_ref_sort_key(payload)?;
                if key.content_type != ContentType::Unit.ordinal() as i8 {
                    return None;
                }
                let size = self.content_loader.unit(key.id)?.hit_size;
                Some(PayloadState {
                    kind: PayloadKind::Unit,
                    size,
                })
            }
        }
    }

    fn apply_picked_build_payload_to_server_unit(
        &mut self,
        packet: &PickedBuildPayloadCallPacket,
        pickup: BuildPayloadPickupKind,
    ) -> bool {
        let UnitRef::Unit { id } = packet.unit else {
            return false;
        };
        let Some(tile_pos) = packet.build_pos else {
            return false;
        };

        match pickup {
            BuildPayloadPickupKind::StoredPayload => {
                self.apply_stored_build_payload_pickup_to_server_unit(id, tile_pos)
            }
            BuildPayloadPickupKind::WholeBuild => {
                self.apply_whole_build_payload_pickup_to_server_unit(id, tile_pos)
            }
        }
    }

    fn apply_stored_build_payload_pickup_to_server_unit(
        &mut self,
        unit_id: i32,
        tile_pos: i32,
    ) -> bool {
        let Some(stored_ref) = self.runtime_payload_ref_for_tile(tile_pos).cloned() else {
            return false;
        };
        let Some(stored_state) = self.payload_ref_to_payload_state(&stored_ref) else {
            return false;
        };
        let Some(unit) = self.server_units.get(&unit_id) else {
            return false;
        };
        if !unit
            .payload
            .as_ref()
            .is_some_and(|payload| payload.can_pickup_payload(&stored_state))
        {
            return false;
        }

        let Some(taken_ref) = self.take_runtime_payload_ref_for_tile(tile_pos) else {
            return false;
        };
        let Some(taken_state) = self.payload_ref_to_payload_state(&taken_ref) else {
            return false;
        };
        let Some(unit) = self.server_units.get_mut(&unit_id) else {
            return false;
        };
        let Some(payload) = unit.payload.as_mut() else {
            return false;
        };
        if !payload.can_pickup_payload(&taken_state) {
            return false;
        }
        payload.add_payload(taken_state);
        true
    }

    fn apply_whole_build_payload_pickup_to_server_unit(
        &mut self,
        unit_id: i32,
        tile_pos: i32,
    ) -> bool {
        let Some(build_snapshot) = self
            .runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == tile_pos)
            .cloned()
        else {
            return false;
        };
        let can_pickup = build_snapshot.block.build_visibility != BuildVisibility::Hidden
            && self.server_build_can_pickup(&build_snapshot)
            && self
                .server_units
                .get(&unit_id)
                .and_then(|unit| unit.payload.as_ref())
                .is_some_and(|payload| {
                    payload.can_pickup_build(
                        build_snapshot.block.size as f32,
                        build_snapshot.team,
                        true,
                    )
                });

        // Java `pickedBuildPayload(..., onGround=true)` removes the tile even if
        // the second live-state validation fails after the request packet was
        // accepted. When validation succeeds, the build becomes the carried
        // payload; otherwise it is still removed after the pickup effect.
        let Some(removed) = self.runtime.remove_building_by_tile_pos(tile_pos) else {
            return false;
        };
        if !can_pickup {
            return true;
        }

        let Some(unit) = self.server_units.get_mut(&unit_id) else {
            return false;
        };
        let Some(payload) = unit.payload.as_mut() else {
            return false;
        };
        payload.add_payload(PayloadState {
            kind: PayloadKind::Build,
            size: removed.block.size as f32 * TILE_SIZE as f32,
        });
        true
    }

    fn apply_picked_unit_payload_to_server_unit(
        &mut self,
        packet: &PickedUnitPayloadCallPacket,
    ) -> bool {
        let UnitRef::Unit { id: unit_id } = packet.unit else {
            return false;
        };
        let UnitRef::Unit { id: target_id } = packet.target else {
            return false;
        };
        if unit_id == target_id {
            return false;
        }

        let Some(target) = self.server_units.get(&target_id) else {
            return false;
        };
        let target_payload = PayloadState {
            kind: PayloadKind::Unit,
            size: target.type_info.hit_size,
        };
        if !self
            .server_units
            .get(&unit_id)
            .and_then(|unit| unit.payload.as_ref())
            .is_some_and(|payload| payload.can_pickup_payload(&target_payload))
        {
            return false;
        }

        let Some(removed_target) = self.server_units.remove(&target_id) else {
            return false;
        };
        let Some(unit) = self.server_units.get_mut(&unit_id) else {
            self.server_units.insert(target_id, removed_target);
            return false;
        };
        let Some(payload) = unit.payload.as_mut() else {
            self.server_units.insert(target_id, removed_target);
            return false;
        };
        payload.add_payload(target_payload);
        true
    }

    fn building_accept_stack_amount(build: &BuildingComp, _item_id: i16, amount: i32) -> i32 {
        let Some(items) = build.items.as_ref() else {
            return 0;
        };
        let remaining = build.block.item_capacity.max(0) - items.total();
        amount.min(remaining).max(0)
    }

    fn move_runtime_unit_cargo_towards(
        &mut self,
        unit_id: i32,
        target_x: f32,
        target_y: f32,
    ) -> Option<bool> {
        let unit = self.server_units.get_mut(&unit_id)?;
        let dx = target_x - unit.x();
        let dy = target_y - unit.y();
        let distance = (dx * dx + dy * dy).sqrt();
        if distance <= CARGO_AI_TRANSFER_RANGE {
            return Some(true);
        }
        if distance <= f32::EPSILON {
            return Some(true);
        }

        let travel = (distance - CARGO_AI_MOVE_RANGE)
            .max(0.0)
            .min(CARGO_AI_MOVE_SMOOTHING)
            .min(distance);
        let scale = travel / distance;
        unit.set_pos(unit.x() + dx * scale, unit.y() + dy * scale);
        let remaining = (distance - travel).max(0.0);
        Some(remaining <= CARGO_AI_TRANSFER_RANGE)
    }

    fn runtime_unit_cargo_retarget_ready(&mut self, unit_id: i32, loader_tile_pos: i32) -> bool {
        let Some(unit) = self.server_units.get_mut(&unit_id) else {
            return false;
        };
        let cargo = unit
            .cargo_ai
            .get_or_insert_with(|| CargoAiRuntimeState::new(Some(loader_tile_pos)));
        cargo.tether_tile_pos = Some(loader_tile_pos);
        cargo.retarget_timer += 1.0;
        if cargo.retarget_timer + f32::EPSILON < CARGO_AI_RETARGET_INTERVAL {
            false
        } else {
            cargo.retarget_timer = 0.0;
            true
        }
    }

    fn apply_payload_drop_to_server_unit(&mut self, unit_ref: UnitRef, x: f32, y: f32) -> bool {
        let UnitRef::Unit { id } = unit_ref else {
            return false;
        };
        let Some(unit) = self.server_units.get_mut(&id) else {
            return false;
        };
        let prev_x = unit.x();
        let prev_y = unit.y();
        unit.set_pos(x, y);
        let dropped = unit
            .payload
            .as_mut()
            .is_some_and(|payload| payload.drop_last_payload(|_| true));
        unit.set_pos(prev_x, prev_y);
        dropped
    }

    fn apply_server_preview_plan_packet(
        &mut self,
        connection_id: i32,
        snapshot: &ClientPlanSnapshotCallPacket,
        now_millis: i64,
    ) -> io::Result<Option<TeamId>> {
        let source_connection = {
            let state = self.net_server.state();
            let state = state.lock().expect("NetServerState mutex poisoned");
            state.connection_states.get(&connection_id).cloned()
        };
        let Some(source_connection) = source_connection else {
            return Ok(None);
        };
        if !Self::connection_can_receive_preview(&source_connection) {
            return Ok(None);
        }

        let plans = snapshot
            .plans
            .as_ref()
            .map(|plans| {
                plans
                    .iter()
                    .map(|plan| plan.to_build_plan())
                    .collect::<io::Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();
        let source_team = source_connection.team;
        let player = self
            .server_preview_players
            .entry(connection_id)
            .or_insert_with(|| PlayerComp::new(source_team));
        player.id = connection_id;
        player.team = source_team;
        player.name = source_connection.name.clone();
        player.color = source_connection.color as u32;
        player.locale = source_connection.locale.clone();
        player.con = Some(source_connection);
        player.handle_preview_plans(
            snapshot.group_id,
            &plans,
            now_millis,
            MAX_PLAYER_PREVIEW_PLANS,
        );
        self.server_preview_plan_packets_applied += 1;
        Ok(Some(source_team))
    }

    fn connection_can_receive_preview(connection: &NetConnection) -> bool {
        connection.has_connected
            && connection.player_added
            && !connection.kicked
            && !connection.has_disconnected
    }

    pub fn broadcast_server_preview_plans_if_due(
        &mut self,
        now: Instant,
        now_millis: i64,
    ) -> io::Result<usize> {
        self.sync_server_preview_players_from_connections();
        if self.server_preview_players.is_empty() {
            return Ok(0);
        }
        if self
            .next_server_preview_broadcast_at
            .is_some_and(|deadline| now < deadline)
        {
            return Ok(0);
        }

        self.next_server_preview_broadcast_at =
            Some(now + Duration::from_millis(PLAN_PREVIEW_SYNC_INTERVAL_MS as u64));
        self.broadcast_server_preview_plans(now_millis)
    }

    pub fn broadcast_server_preview_plans(&mut self, now_millis: i64) -> io::Result<usize> {
        self.sync_server_preview_players_from_connections();
        let mut players = Vec::new();

        for (connection_id, player) in self.server_preview_players.iter_mut() {
            let plans = player
                .get_preview_plans(now_millis)
                .iter()
                .take(MAX_PLAYER_PREVIEW_PLANS)
                .map(BuildPlanWire::from_build_plan)
                .collect();
            players.push(PlayerPreviewPlanSource {
                player_id: player.id,
                team: player.team,
                connection_id: Some(*connection_id),
                local: false,
                connected: true,
                last_preview_plan_group_server: player.last_preview_plan_group_server,
                plans,
            });
        }

        let sent = self
            .net_server
            .broadcast_client_plan_previews(&mut players)?;
        for player_source in players {
            if let Some(connection_id) = player_source.connection_id {
                if let Some(player) = self.server_preview_players.get_mut(&connection_id) {
                    player.last_preview_plan_group_server =
                        player_source.last_preview_plan_group_server;
                }
            }
        }
        self.server_preview_broadcasts_sent += sent;
        Ok(sent)
    }

    fn sync_server_preview_players_from_connections(&mut self) {
        let ready_connections = {
            let state = self.net_server.state();
            let state = state.lock().expect("NetServerState mutex poisoned");
            state
                .connection_states
                .iter()
                .filter_map(|(connection_id, connection)| {
                    Self::connection_can_receive_preview(connection)
                        .then_some((*connection_id, connection.clone()))
                })
                .collect::<Vec<_>>()
        };
        self.server_preview_players
            .retain(|connection_id, _| ready_connections.iter().any(|(id, _)| id == connection_id));
        for (connection_id, connection) in ready_connections {
            let player = self
                .server_preview_players
                .entry(connection_id)
                .or_insert_with(|| PlayerComp::new(connection.team));
            player.id = connection_id;
            player.team = connection.team;
            player.name = connection.name.clone();
            player.color = connection.color as u32;
            player.locale = connection.locale.clone();
            player.con = Some(connection);
        }
    }

    fn current_millis() -> i64 {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        millis.min(i64::MAX as u128) as i64
    }

    pub fn network_world_data_template(&mut self) -> NetworkWorldData {
        let mut map_tags = self.runtime.state.map.tags.clone();
        map_tags
            .entry("name".into())
            .or_insert_with(|| self.runtime.state.map.name().to_string());
        map_tags
            .entry("width".into())
            .or_insert_with(|| self.runtime.state.world.width().to_string());
        map_tags
            .entry("height".into())
            .or_insert_with(|| self.runtime.state.world.height().to_string());

        let team_blocks_snapshot = self.runtime.state.export_legacy_team_blocks(
            |name| {
                self.content_loader
                    .block_by_name(name)
                    .map(|block| block.base().id)
            },
            true,
        );

        NetworkWorldData {
            map_tags,
            wave: self.runtime.state.wave,
            wave_time: self.runtime.state.wavetime,
            tick: self.runtime.state.tick,
            rand_seed0: self.runtime.state.rand_seed0,
            rand_seed1: self.runtime.state.rand_seed1,
            content_header_snapshot: Some(self.content_loader.content_header_snapshot()),
            content_patches_snapshot: Some(ContentPatchSet::default()),
            map_snapshot: Some(
                self.runtime
                    .export_network_map_snapshot(&self.content_loader),
            ),
            team_blocks_snapshot: Some(team_blocks_snapshot),
            markers_snapshot: Some(self.runtime.state.markers.clone()),
            custom_chunks_snapshot: Some(self.runtime.state.custom_chunks.clone()),
            ..NetworkWorldData::default()
        }
    }

    pub fn update_runtime_effect_blocks(
        &mut self,
        delta_seconds: f32,
    ) -> Option<EffectBlockFrameBatchReport> {
        let mut bullets: Vec<BulletComp> = Vec::new();
        let mut units: Vec<UnitComp> = Vec::new();
        let mut bullet_type = |_: ContentId| -> Option<&BulletType> { None };
        let mut suppressed = |_: &BuildingComp| false;
        let mut force_coolant = |_: &BuildingComp| (0.0, 0.0);
        let mut spark_random = |_: &UnitComp| 1.0;

        self.runtime.advance_owned_effect_blocks(
            &self.content_loader,
            delta_seconds,
            GameRuntimeOwnedEffectResources {
                bullets: &mut bullets,
                bullet_type: &mut bullet_type,
                units: &mut units,
                suppressed: &mut suppressed,
                force_coolant: &mut force_coolant,
                spark_random: &mut spark_random,
            },
        )
    }

    pub fn update_runtime_owned_blocks(
        &mut self,
        delta_seconds: f32,
    ) -> Option<GameRuntimeOwnedFrameReport> {
        let mut bullets: Vec<BulletComp> = Vec::new();
        let mut units: Vec<UnitComp> = Vec::new();
        let mut bullet_type = |_: ContentId| -> Option<&BulletType> { None };
        let mut suppressed = |_: &BuildingComp| false;
        let mut force_coolant = |_: &BuildingComp| (0.0, 0.0);
        let mut spark_random = |_: &UnitComp| 1.0;

        self.runtime.advance_owned_runtime_blocks(
            &self.content_loader,
            delta_seconds,
            GameRuntimeOwnedEffectResources {
                bullets: &mut bullets,
                bullet_type: &mut bullet_type,
                units: &mut units,
                suppressed: &mut suppressed,
                force_coolant: &mut force_coolant,
                spark_random: &mut spark_random,
            },
        )
    }
}

pub fn run(args: Vec<String>) -> ServerLauncher {
    let mut launcher = ServerLauncher::new(args);
    launcher.init();
    launcher
}

pub fn banner() -> String {
    format!("mindustry server bootstrap ({UPSTREAM_BASELINE})")
}

fn parse_port_arg(args: &[String]) -> Option<u16> {
    for (index, arg) in args.iter().enumerate() {
        if arg == "--port" || arg == "-p" {
            if let Some(next) = args.get(index + 1) {
                if let Ok(port) = next.parse() {
                    return Some(port);
                }
            }
        } else if let Some(value) = arg.strip_prefix("--port=") {
            if let Ok(port) = value.parse() {
                return Some(port);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        ServerLauncher, CARGO_AI_DROP_SPACING, CARGO_AI_RETARGET_INTERVAL, CARGO_AI_TRANSFER_RANGE,
    };
    use mindustry_core::mindustry::content::blocks::{BlockDef, UnitAssemblerBlockData};
    use mindustry_core::mindustry::core::game_runtime::{
        GameRuntimeCampaignBlockState, GameRuntimeDistributionBlockState,
        GameRuntimePayloadBlockState, GameRuntimePowerNodeBatchLinkReport,
        GameRuntimePowerNodeConfigResult, GameRuntimePowerNodeLinkResult,
        GameRuntimeUnitBlockState, GameRuntimeUnitCargoUnloadConfigureResult,
    };
    use mindustry_core::mindustry::core::{
        content_loader::ContentLoader, GameRuntime, GameRuntimeNetworkContext, GameStateState,
        NetServer,
    };
    use mindustry_core::mindustry::ctype::{Content, ContentType};
    use mindustry_core::mindustry::entities::comp::{
        BuildingComp, BuildingTetherAction, BuildingTetherComp, BuildingTetherRef,
        CargoAiRuntimeState, PayloadComp, PayloadKind, PayloadState, UnitComp, UnitControllerState,
    };
    use mindustry_core::mindustry::entities::{
        standard_effect_id, FireRules, FireTile, Fires, PuddleDepositContext, PuddleLiquidInfo,
        PuddleTileView, Puddles,
    };
    use mindustry_core::mindustry::game::{BlockPlan, ExportStat, Trigger, TEAM_SHARDED};
    use mindustry_core::mindustry::io::type_io::CommandWire;
    use mindustry_core::mindustry::io::{
        BuildPlanWire, BuildingRef, EntityRef, Point2, TeamId, TypeValue, UnitRef, Vec2,
    };
    use mindustry_core::mindustry::net::{
        packet_ids, read_world_data, ClientPlanSnapshotCallPacket, CommandBuildingCallPacket,
        Connect, ConnectFilter, ConnectPacket, DoneCallback, DropItemCallPacket, Host,
        HostCallback, Net, NetConnection, NetProvider, NetworkWorldData, PacketKind,
        PacketSerializer, ProviderEvent, RequestBuildPayloadCallPacket,
        RequestDropPayloadCallPacket, RequestItemCallPacket, RequestUnitPayloadCallPacket,
        TileConfigCallPacket, TransferInventoryCallPacket,
    };
    use mindustry_core::mindustry::r#type::{PayloadKey, PayloadSeq, Sector, UnitType};
    use mindustry_core::mindustry::vars::{AppContext, RuntimeMode, TILE_SIZE};
    use mindustry_core::mindustry::world::blocks::autotiler_direction;
    use mindustry_core::mindustry::world::blocks::campaign::LandingPadState;
    use mindustry_core::mindustry::world::blocks::payloads::{
        payload_mass_driver_loaded_pay_length, BlockProducerState, PayloadBlockBuildState,
        PayloadConveyorState, PayloadDeconstructorState, PayloadDriverState, PayloadLoaderState,
        PayloadMassDriverState, PayloadRef, PayloadSortKey, PayloadSourceState,
    };
    use mindustry_core::mindustry::world::blocks::units::{
        unit_assembler_drone_target, ReconstructorState, UnitAssemblerState, UnitBlockState,
        UnitCargoLoaderState, UnitCargoUnloadPointState, UnitFactoryState,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use std::collections::BTreeMap;
    use std::io;
    use std::net::{TcpListener, UdpSocket};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

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

    fn seed_server_assembler_drones_in_position(
        launcher: &mut ServerLauncher,
        tile_pos: i32,
        assembler_block: &UnitAssemblerBlockData,
    ) {
        let building = launcher
            .runtime
            .buildings
            .iter()
            .find(|building| building.tile_pos == tile_pos)
            .cloned()
            .expect("assembler building should exist");
        let Some(drone_type) = launcher
            .content_loader
            .unit_by_name(&assembler_block.drone_type)
            .cloned()
        else {
            panic!("assembler drone type should exist");
        };
        let (dir_x, dir_y) = autotiler_direction(building.rotation);
        let spawn_len =
            TILE_SIZE as f32 * (assembler_block.area_size + assembler_block.base.size) as f32 / 2.0;
        let spawn_x = building.x + dir_x as f32 * spawn_len;
        let spawn_y = building.y + dir_y as f32 * spawn_len;
        let mut ids = Vec::new();
        for slot_index in 0..assembler_block.drones_created.max(0) as usize {
            let unit_id = 80_000 + slot_index as i32;
            let target = unit_assembler_drone_target(
                spawn_x,
                spawn_y,
                assembler_block.area_size,
                TILE_SIZE as f32,
                slot_index,
            );
            let mut unit = UnitComp::new(unit_id, drone_type.clone(), building.team);
            unit.set_pos(target.pos.x, target.pos.y);
            unit.set_rotation(target.angle);
            unit.set_controller(UnitControllerState::Assembler);
            unit.building_tether = Some(BuildingTetherComp {
                team: building.team,
                building: Some(BuildingTetherRef {
                    tile_pos,
                    team: building.team,
                    valid: true,
                }),
            });
            launcher.server_units.insert(unit_id, unit.clone());
            launcher
                .runtime
                .client_unit_snapshot_entities
                .insert(unit_id, unit);
            ids.push(unit_id);
        }
        let Some(GameRuntimeUnitBlockState::Assembler { assembler, .. }) =
            launcher.runtime.unit_runtime_states.get_mut(&tile_pos)
        else {
            panic!("assembler sidecar should exist");
        };
        assembler.read_unit_ids = ids;
    }

    #[test]
    fn server_launcher_reads_port_arg_before_opening_network() {
        let port = free_local_port();
        let mut launcher = ServerLauncher::new(vec![
            "mindustry-server".into(),
            "--port".into(),
            port.to_string(),
        ]);

        assert_eq!(launcher.context.port, port);
        assert_eq!(
            launcher.runtime.network_context,
            GameRuntimeNetworkContext::server()
        );
        assert!(launcher.runtime.buildings().is_empty());
        assert!(launcher.runtime.effect_runtime_store.is_empty());
        assert!(launcher.runtime.effect_timer_store.is_empty());

        launcher.init();

        assert!(launcher.net_server.is_active());
        assert_eq!(launcher.network_error, None);
        launcher.close_network();
    }

    #[test]
    fn server_launcher_opens_real_arc_network_on_configured_port() {
        let port = free_local_port();
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.context.port = port;

        launcher.init();

        assert_eq!(launcher.context.flags.mode, RuntimeMode::Server);
        assert!(launcher.context.flags.headless);
        assert!(launcher.net_server.is_active());
        assert!(launcher.net_server.is_server());
        assert_eq!(launcher.network_error, None);
        {
            let state = launcher.net_server.state();
            let state = state.lock().unwrap();
            assert_eq!(state.listen_port, Some(port));
        }

        launcher.close_network();
        assert!(!launcher.net_server.is_active());
    }

    #[test]
    fn server_can_decode_connect_packet_envelope_from_core_net_layer() {
        let packet = ConnectPacket {
            version: 157,
            version_type: "official".into(),
            mods: Vec::new(),
            name: "server-test".into(),
            locale: "en_US".into(),
            uuid: "AQIDBAUGBwg=".into(),
            usid: "usid".into(),
            mobile: false,
            color: 0x123456,
            uuid_crc32: None,
        };
        let bytes = PacketSerializer::write_packet_kind(&PacketKind::ConnectPacket(packet))
            .expect("connect packet should encode");
        assert_eq!(bytes[0], packet_ids::CONNECT_PACKET);

        match PacketSerializer::read_packet_kind(&bytes).expect("connect packet should decode") {
            PacketKind::ConnectPacket(decoded) => {
                assert_eq!(decoded.version, 157);
                assert_eq!(decoded.name, "server-test");
                // Java read() consumes 16 bytes for uuid, so Rust intentionally
                // preserves that upstream asymmetry.
                assert_ne!(decoded.uuid, "AQIDBAUGBwg=");
                assert!(!decoded.mobile);
                assert_eq!(decoded.color, 0x123456);
            }
            other => panic!("unexpected packet: {other:?}"),
        }
    }

    #[derive(Default)]
    struct CaptureProvider {
        sent: Arc<Mutex<Vec<(i32, PacketKind, bool)>>>,
    }

    impl NetProvider for CaptureProvider {
        fn connect_client(
            &mut self,
            _ip: &str,
            _port: u16,
            _success: Box<dyn Fn() + Send + 'static>,
        ) -> io::Result<()> {
            Ok(())
        }

        fn send_client(&mut self, _object: &PacketKind, _reliable: bool) -> io::Result<()> {
            Ok(())
        }

        fn send_server(&mut self, object: &PacketKind, reliable: bool) -> io::Result<()> {
            self.sent
                .lock()
                .unwrap()
                .push((-1, object.clone(), reliable));
            Ok(())
        }

        fn disconnect_client(&mut self) {}

        fn discover_servers(&self, _callback: HostCallback, done: DoneCallback) {
            done();
        }

        fn ping_host(&self, _address: &str, _port: u16, _timeout: Duration) -> io::Result<Host> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "capture provider does not ping hosts",
            ))
        }

        fn host_server(&mut self, _port: u16) -> io::Result<()> {
            Ok(())
        }

        fn get_connections(&self) -> Vec<NetConnection> {
            Vec::new()
        }

        fn close_server(&mut self) {}

        fn send_server_to(
            &mut self,
            connection_id: i32,
            object: &PacketKind,
            reliable: bool,
        ) -> io::Result<()> {
            self.sent
                .lock()
                .unwrap()
                .push((connection_id, object.clone(), reliable));
            Ok(())
        }

        fn drain_events(&mut self) -> Vec<ProviderEvent> {
            Vec::new()
        }

        fn set_connect_filter(&mut self, _connect_filter: Option<ConnectFilter>) {}
    }

    fn connect_packet(name: &str) -> ConnectPacket {
        ConnectPacket {
            version: 158,
            version_type: "official".into(),
            mods: Vec::new(),
            name: name.into(),
            locale: "en_US".into(),
            uuid: "uuid".into(),
            usid: "usid".into(),
            mobile: false,
            color: 0,
            uuid_crc32: None,
        }
    }

    fn decode_captured_world_data(
        sent: &[(i32, PacketKind, bool)],
        connection_id: i32,
    ) -> NetworkWorldData {
        let mut total = None;
        let mut bytes = Vec::new();
        for (target, packet, reliable) in sent {
            if *target != connection_id {
                continue;
            }
            assert!(*reliable);
            match packet {
                PacketKind::StreamBegin(begin) if begin.packet_type == packet_ids::WORLD_STREAM => {
                    total = Some(begin.total as usize);
                }
                PacketKind::StreamChunk(chunk) => bytes.extend_from_slice(&chunk.data),
                _ => {}
            }
        }

        assert_eq!(Some(bytes.len()), total);
        read_world_data(&bytes).expect("captured world stream should decode")
    }

    #[test]
    fn server_launcher_applies_request_item_packet_to_runtime_and_broadcasts_take_items() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6567).unwrap();
        launcher.runtime = GameRuntime::default();
        launcher
            .runtime
            .set_network_context(GameRuntimeNetworkContext::server());
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.set_sector(Some(Sector::new(17)));
        launcher.runtime.state.world.resize(8, 8);

        let default_team = TeamId(launcher.runtime.state.rules.default_team as u8);
        let core_def = launcher.content_loader.block_by_name("core-shard").unwrap();
        let copper = launcher
            .content_loader
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let core_tile = point2_pack(2, 2);
        let mut core_building = BuildingComp::new(core_tile, core_def.base().clone(), default_team);
        core_building.items.as_mut().unwrap().add(copper, 10);
        launcher.runtime.add_building(core_building);

        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            let mut connection = NetConnection::new("127.0.0.1:7007");
            connection.has_connected = true;
            connection.player_added = true;
            connection.team = default_team;
            connection.name = "rust-client".into();
            state.connection_states.insert(7, connection);
            state.events.push(ProviderEvent::ServerPacket {
                connection_id: 7,
                packet: PacketKind::RequestItemCallPacket(RequestItemCallPacket {
                    player: EntityRef::null(),
                    build: BuildingRef::new(core_tile),
                    item: Some("copper".into()),
                    amount: 5,
                }),
            });
        }

        let changed = launcher.apply_new_network_server_events();

        assert_eq!(changed, 1);
        assert_eq!(launcher.runtime_request_item_packets_seen, 1);
        assert_eq!(launcher.runtime_request_item_packets_accepted, 1);
        assert_eq!(launcher.runtime_request_item_packets_rejected, 0);
        assert_eq!(launcher.runtime_take_items_packets_sent, 1);
        assert_eq!(launcher.runtime_transfer_item_effect_packets_sent, 1);
        assert!(launcher
            .last_runtime_request_item_outcome
            .as_ref()
            .is_some_and(|outcome| outcome.accepted && outcome.accepted_amount == 5));
        assert!(launcher
            .last_runtime_take_items_outcome
            .as_ref()
            .is_some_and(|outcome| outcome.accepted && outcome.removed_amount == 5));
        assert_eq!(
            launcher.runtime.buildings()[0]
                .items
                .as_ref()
                .unwrap()
                .get(copper),
            5
        );
        let unit = launcher.server_units.get(&7).unwrap();
        assert_eq!(unit.items.item(), Some("copper"));
        assert_eq!(unit.items.stack.amount, 5);
        assert_eq!(
            launcher
                .runtime
                .state
                .sector
                .as_ref()
                .unwrap()
                .info
                .core_deltas
                .get("copper"),
            Some(&-5)
        );

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                    packet,
                    PacketKind::TakeItemsCallPacket(packet)
                        if packet.build == BuildingRef::new(core_tile)
                            && packet.item.as_deref() == Some("copper")
                            && packet.amount == 5
                            && packet.to == UnitRef::Unit { id: 7 }
                )
        }));
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                        packet,
                        PacketKind::TransferItemEffectCallPacket(packet)
                            if packet.item.as_deref() == Some("copper")
                                && packet.to == EntityRef::new(7)
                )
        }));
    }

    #[test]
    fn server_launcher_applies_drop_item_packet_to_unit_stack() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6577).unwrap();

        let default_team = TeamId(launcher.runtime.state.rules.default_team as u8);
        let mut unit = UnitComp::new(
            7,
            launcher
                .content_loader
                .unit_by_name("alpha")
                .unwrap()
                .clone(),
            default_team,
        );
        unit.items.add_item_amount("copper", 3);
        launcher.server_units.insert(7, unit);

        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            let mut connection = NetConnection::new("127.0.0.1:7017");
            connection.has_connected = true;
            connection.player_added = true;
            connection.team = default_team;
            connection.name = "rust-client".into();
            state.connection_states.insert(7, connection);
            state.events.push(ProviderEvent::ServerPacket {
                connection_id: 7,
                packet: PacketKind::DropItemCallPacket(DropItemCallPacket { angle: 37.5 }),
            });
        }

        let changed = launcher.apply_new_network_server_events();

        assert_eq!(changed, 1);
        assert_eq!(launcher.runtime_drop_item_packets_seen, 1);
        assert_eq!(launcher.runtime_drop_item_packets_accepted, 1);
        assert_eq!(launcher.runtime_drop_item_packets_rejected, 0);
        assert_eq!(launcher.runtime_drop_item_packets_sent, 0);
        assert_eq!(launcher.server_units.get(&7).unwrap().items.stack.amount, 0);
        assert!(launcher
            .last_runtime_drop_item_outcome
            .as_ref()
            .is_some_and(|outcome| {
                outcome.accepted
                    && outcome.previous_item.as_deref() == Some("copper")
                    && outcome.previous_amount == 3
            }));

        let sent = sent.lock().unwrap();
        assert!(sent
            .iter()
            .all(|(_connection_id, packet, _reliable)| !matches!(
                packet,
                PacketKind::DropItemCallPacket(_)
            )));
    }

    #[test]
    fn server_launcher_applies_transfer_inventory_packet_to_runtime_and_broadcasts_transfer_item_to(
    ) {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6568).unwrap();
        launcher.runtime = GameRuntime::default();
        launcher
            .runtime
            .set_network_context(GameRuntimeNetworkContext::server());
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.set_sector(Some(Sector::new(18)));
        launcher.runtime.state.world.resize(8, 8);

        let default_team = TeamId(launcher.runtime.state.rules.default_team as u8);
        let core_def = launcher.content_loader.block_by_name("core-shard").unwrap();
        let copper = launcher
            .content_loader
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let core_tile = point2_pack(2, 2);
        let mut core_building = BuildingComp::new(core_tile, core_def.base().clone(), default_team);
        core_building.items.as_mut().unwrap().add(copper, 4);
        launcher.runtime.add_building(core_building);

        let mut unit = UnitComp::new(
            7,
            launcher
                .content_loader
                .unit_by_name("alpha")
                .unwrap()
                .clone(),
            default_team,
        );
        unit.set_pos(32.0, 40.0);
        unit.items.add_item_amount("copper", 6);
        launcher.server_units.insert(7, unit);

        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            let mut connection = NetConnection::new("127.0.0.1:7008");
            connection.has_connected = true;
            connection.player_added = true;
            connection.team = default_team;
            connection.name = "rust-client".into();
            state.connection_states.insert(7, connection);
            state.events.push(ProviderEvent::ServerPacket {
                connection_id: 7,
                packet: PacketKind::TransferInventoryCallPacket(TransferInventoryCallPacket {
                    player: EntityRef::null(),
                    build: BuildingRef::new(core_tile),
                }),
            });
        }

        let changed = launcher.apply_new_network_server_events();

        assert_eq!(changed, 1);
        assert_eq!(launcher.runtime_transfer_inventory_packets_seen, 1);
        assert_eq!(launcher.runtime_transfer_inventory_packets_accepted, 1);
        assert_eq!(launcher.runtime_transfer_inventory_packets_rejected, 0);
        assert_eq!(launcher.runtime_transfer_item_to_packets_sent, 1);
        assert!(launcher
            .last_runtime_transfer_inventory_outcome
            .as_ref()
            .is_some_and(|outcome| outcome.accepted && outcome.accepted_amount == 6));
        assert!(launcher
            .last_runtime_transfer_item_to_outcome
            .as_ref()
            .is_some_and(|outcome| {
                outcome.accepted
                    && outcome.unit_previous_amount == Some(6)
                    && outcome.unit_new_amount == Some(0)
                    && outcome.building_previous_amount == 4
                    && outcome.building_new_amount == 10
            }));
        assert_eq!(
            launcher.runtime.buildings()[0]
                .items
                .as_ref()
                .unwrap()
                .get(copper),
            10
        );
        assert_eq!(launcher.server_units.get(&7).unwrap().items.stack.amount, 0);
        assert_eq!(
            launcher
                .runtime
                .state
                .sector
                .as_ref()
                .unwrap()
                .info
                .core_deltas
                .get("copper"),
            Some(&6)
        );

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                    packet,
                    PacketKind::TransferItemToCallPacket(packet)
                        if packet.unit == UnitRef::Unit { id: 7 }
                            && packet.item.as_deref() == Some("copper")
                            && packet.amount == 6
                            && (packet.x, packet.y) == (32.0, 40.0)
                            && packet.build == BuildingRef::new(core_tile)
                )
        }));
    }

    #[test]
    fn server_launcher_applies_request_drop_payload_packet_to_unit_payload_and_broadcasts_drop() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6569).unwrap();

        let default_team = TeamId(launcher.runtime.state.rules.default_team as u8);
        let mut unit_type = launcher
            .content_loader
            .unit_by_name("alpha")
            .unwrap()
            .clone();
        unit_type.payload_capacity = 512.0;
        let mut unit = UnitComp::new(7, unit_type, default_team);
        unit.set_pos(0.0, 0.0);
        unit.payload = Some(PayloadComp::new(default_team, 512.0));
        unit.payload.as_mut().unwrap().add_payload(PayloadState {
            kind: PayloadKind::Build,
            size: 2.0,
        });
        launcher.server_units.insert(7, unit);

        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            let mut connection = NetConnection::new("127.0.0.1:7009");
            connection.has_connected = true;
            connection.player_added = true;
            connection.team = default_team;
            connection.name = "rust-client".into();
            state.connection_states.insert(7, connection);
            state.events.push(ProviderEvent::ServerPacket {
                connection_id: 7,
                packet: PacketKind::RequestDropPayloadCallPacket(RequestDropPayloadCallPacket {
                    player: EntityRef::null(),
                    x: 100.0,
                    y: 0.0,
                }),
            });
        }

        let changed = launcher.apply_new_network_server_events();

        assert_eq!(changed, 1);
        assert_eq!(launcher.runtime_request_drop_payload_packets_seen, 1);
        assert_eq!(launcher.runtime_request_drop_payload_packets_accepted, 1);
        assert_eq!(launcher.runtime_request_drop_payload_packets_rejected, 0);
        assert_eq!(launcher.runtime_payload_dropped_packets_sent, 1);
        let drop_outcome = launcher
            .last_runtime_request_drop_payload_outcome
            .as_ref()
            .expect("request drop outcome should be recorded");
        assert!(drop_outcome.accepted);
        assert_eq!(drop_outcome.requested_x, 100.0);
        assert_eq!(drop_outcome.requested_y, 0.0);
        assert!(drop_outcome.clamped_x < 100.0);
        assert_eq!(drop_outcome.clamped_y, 0.0);
        assert!(launcher
            .last_runtime_payload_dropped_outcome
            .as_ref()
            .is_some_and(|outcome| outcome.accepted));
        let unit = launcher.server_units.get(&7).unwrap();
        assert_eq!(unit.payload.as_ref().unwrap().payloads.len(), 0);
        assert_eq!((unit.x(), unit.y()), (0.0, 0.0));

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                        packet,
                        PacketKind::PayloadDroppedCallPacket(packet)
                            if packet.unit == UnitRef::Unit { id: 7 }
                                && packet.x == drop_outcome.clamped_x
                                && packet.y == drop_outcome.clamped_y
                )
        }));
    }

    #[test]
    fn server_launcher_applies_request_unit_payload_packet_to_target_unit_and_broadcasts_pickup() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6576).unwrap();

        let default_team = TeamId(launcher.runtime.state.rules.default_team as u8);
        let mut carrier_type = launcher
            .content_loader
            .unit_by_name("alpha")
            .unwrap()
            .clone();
        carrier_type.payload_capacity = 512.0;
        carrier_type.pickup_units = true;
        let mut carrier = UnitComp::new(7, carrier_type, default_team);
        carrier.set_pos(0.0, 0.0);
        carrier.payload = Some(PayloadComp::new(default_team, 512.0));
        carrier.payload.as_mut().unwrap().pickup_units = true;
        launcher.server_units.insert(7, carrier);

        let mut target_type = launcher
            .content_loader
            .unit_by_name("dagger")
            .unwrap()
            .clone();
        target_type.allowed_in_payloads = true;
        let target_size = target_type.hit_size;
        let mut target = UnitComp::new(8, target_type, default_team);
        target.set_pos(4.0, 0.0);
        launcher.server_units.insert(8, target);

        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            let mut connection = NetConnection::new("127.0.0.1:7016");
            connection.has_connected = true;
            connection.player_added = true;
            connection.team = default_team;
            connection.name = "rust-client".into();
            state.connection_states.insert(7, connection);
            state.events.push(ProviderEvent::ServerPacket {
                connection_id: 7,
                packet: PacketKind::RequestUnitPayloadCallPacket(RequestUnitPayloadCallPacket {
                    player: EntityRef::null(),
                    target: UnitRef::Unit { id: 8 },
                }),
            });
        }

        let changed = launcher.apply_new_network_server_events();

        assert_eq!(changed, 1);
        assert_eq!(launcher.runtime_request_unit_payload_packets_seen, 1);
        assert_eq!(launcher.runtime_request_unit_payload_packets_accepted, 1);
        assert_eq!(launcher.runtime_request_unit_payload_packets_rejected, 0);
        assert_eq!(launcher.runtime_picked_unit_payload_packets_sent, 1);
        assert!(launcher
            .last_runtime_request_unit_payload_outcome
            .as_ref()
            .is_some_and(|outcome| outcome.accepted));
        assert!(launcher
            .last_runtime_picked_unit_payload_outcome
            .as_ref()
            .is_some_and(|outcome| outcome.accepted));
        assert!(!launcher.server_units.contains_key(&8));
        let carrier = launcher.server_units.get(&7).unwrap();
        let payloads = &carrier.payload.as_ref().unwrap().payloads;
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].kind, PayloadKind::Unit);
        assert_eq!(payloads[0].size, target_size);

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                    packet,
                    PacketKind::PickedUnitPayloadCallPacket(packet)
                        if packet.unit == UnitRef::Unit { id: 7 }
                            && packet.target == UnitRef::Unit { id: 8 }
                )
        }));
    }

    #[test]
    fn server_launcher_broadcasts_unit_entered_payload_from_runtime_unit_and_building() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6578).unwrap();

        let default_team = TeamId(launcher.runtime.state.rules.default_team as u8);
        let driver_block = launcher
            .content_loader
            .block_by_name("payload-mass-driver")
            .unwrap()
            .base()
            .clone();
        let tile_pos = point2_pack(11, 12);
        launcher
            .runtime
            .buildings
            .push(BuildingComp::new(tile_pos, driver_block, default_team));

        let mut unit_type = launcher
            .content_loader
            .unit_by_name("flare")
            .unwrap()
            .clone();
        unit_type.allowed_in_payloads = true;
        let mut unit = UnitComp::new(5151, unit_type, default_team);
        unit.set_pos(88.0, 96.0);
        unit.set_rotation(180.0);
        launcher.server_units.insert(5151, unit);

        assert!(launcher
            .apply_server_unit_entered_payload(5151, tile_pos)
            .unwrap());

        assert!(!launcher.server_units.contains_key(&5151));
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

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                    packet,
                    PacketKind::UnitEnteredPayloadCallPacket(packet)
                        if packet.unit == UnitRef::Unit { id: 5151 }
                            && packet.build == BuildingRef::new(tile_pos)
                )
        }));
    }

    #[test]
    fn server_launcher_update_applies_enter_payload_command_to_payload_building_and_broadcasts_packet(
    ) {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6579).unwrap();

        let default_team = TeamId(launcher.runtime.state.rules.default_team as u8);
        let driver_block = launcher
            .content_loader
            .block_by_name("payload-mass-driver")
            .unwrap()
            .base()
            .clone();
        let tile_x = 13;
        let tile_y = 14;
        let tile_pos = point2_pack(tile_x, tile_y);
        launcher
            .runtime
            .buildings
            .push(BuildingComp::new(tile_pos, driver_block, default_team));
        launcher.runtime.sync_world_footprint_refs(0);

        let enter_payload_id = launcher
            .content_loader
            .unit_command_by_name("enterPayload")
            .unwrap()
            .id();
        let mut unit_type = launcher
            .content_loader
            .unit_by_name("flare")
            .unwrap()
            .clone();
        unit_type.allowed_in_payloads = true;
        let mut unit = UnitComp::new(6161, unit_type, default_team);
        unit.set_pos(
            tile_x as f32 * TILE_SIZE as f32,
            tile_y as f32 * TILE_SIZE as f32,
        );
        unit.set_controller(UnitControllerState::Command(CommandWire {
            command_id: Some(enter_payload_id),
            target_pos: Some(Vec2::new(
                tile_x as f32 * TILE_SIZE as f32,
                tile_y as f32 * TILE_SIZE as f32,
            )),
            ..CommandWire::default()
        }));
        launcher.server_units.insert(6161, unit);

        launcher.update();

        assert!(!launcher.server_units.contains_key(&6161));
        let Some(GameRuntimePayloadBlockState::MassDriver { common, driver }) =
            launcher.runtime.payload_runtime_states.get(&tile_pos)
        else {
            panic!("payload-mass-driver should receive entered unit payload from server update");
        };
        assert!(driver.loaded);
        assert!(matches!(
            common.payload,
            Some(PayloadRef::Unit {
                class_id: 3,
                ref unit_bytes
            }) if !unit_bytes.is_empty()
        ));

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                    packet,
                    PacketKind::UnitEnteredPayloadCallPacket(packet)
                        if packet.unit == UnitRef::Unit { id: 6161 }
                            && packet.build == BuildingRef::new(tile_pos)
                )
        }));
    }

    #[test]
    fn server_launcher_update_skips_enter_payload_when_target_building_mismatch() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6580).unwrap();

        let default_team = TeamId(launcher.runtime.state.rules.default_team as u8);
        let driver_block = launcher
            .content_loader
            .block_by_name("payload-mass-driver")
            .unwrap()
            .base()
            .clone();
        let tile_pos = point2_pack(15, 16);
        launcher
            .runtime
            .buildings
            .push(BuildingComp::new(tile_pos, driver_block, default_team));
        launcher.runtime.sync_world_footprint_refs(0);

        let enter_payload_id = launcher
            .content_loader
            .unit_command_by_name("enterPayload")
            .unwrap()
            .id();
        let mut unit_type = launcher
            .content_loader
            .unit_by_name("flare")
            .unwrap()
            .clone();
        unit_type.allowed_in_payloads = true;
        let mut unit = UnitComp::new(6262, unit_type, default_team);
        unit.set_pos(15.0 * TILE_SIZE as f32, 16.0 * TILE_SIZE as f32);
        unit.set_controller(UnitControllerState::Command(CommandWire {
            command_id: Some(enter_payload_id),
            target_pos: Some(Vec2::new(1.0 * TILE_SIZE as f32, 1.0 * TILE_SIZE as f32)),
            ..CommandWire::default()
        }));
        launcher.server_units.insert(6262, unit);

        launcher.update();

        assert!(launcher.server_units.contains_key(&6262));
        if let Some(GameRuntimePayloadBlockState::MassDriver { common, driver }) =
            launcher.runtime.payload_runtime_states.get(&tile_pos)
        {
            assert!(!driver.loaded);
            assert!(common.payload.is_none());
        }

        let sent = sent.lock().unwrap();
        assert!(!sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable && matches!(packet, PacketKind::UnitEnteredPayloadCallPacket(_))
        }));
    }

    #[test]
    fn server_launcher_applies_request_build_payload_packet_to_stored_payload_and_broadcasts_pickup(
    ) {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6570).unwrap();

        let default_team = TeamId(launcher.runtime.state.rules.default_team as u8);
        let loader_block = launcher
            .content_loader
            .block_by_name("payload-loader")
            .unwrap()
            .base()
            .clone();
        let router_block = launcher
            .content_loader
            .block_by_name("router")
            .unwrap()
            .base()
            .clone();
        let router_id = router_block.id;
        let router_payload_size = router_block.size as f32 * TILE_SIZE as f32;
        let loader_tile = point2_pack(3, 3);
        let mut router_payload_bytes = Vec::new();
        BuildingComp::new(point2_pack(0, 0), router_block, default_team)
            .write_base(&mut router_payload_bytes, false)
            .unwrap();
        launcher
            .runtime
            .add_building(BuildingComp::new(loader_tile, loader_block, default_team));
        launcher.runtime.payload_runtime_states.insert(
            loader_tile,
            GameRuntimePayloadBlockState::Loader {
                common: PayloadBlockBuildState {
                    payload: Some(PayloadRef::Block {
                        block: router_id,
                        version: 1,
                        build_bytes: router_payload_bytes,
                    }),
                    ..PayloadBlockBuildState::default()
                },
                loader: PayloadLoaderState::default(),
            },
        );

        let mut unit_type = launcher
            .content_loader
            .unit_by_name("alpha")
            .unwrap()
            .clone();
        unit_type.payload_capacity = 512.0;
        let mut unit = UnitComp::new(7, unit_type, default_team);
        unit.payload = Some(PayloadComp::new(default_team, 512.0));
        launcher.server_units.insert(7, unit);

        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            let mut connection = NetConnection::new("127.0.0.1:7010");
            connection.has_connected = true;
            connection.player_added = true;
            connection.team = default_team;
            connection.name = "rust-client".into();
            state.connection_states.insert(7, connection);
            state.events.push(ProviderEvent::ServerPacket {
                connection_id: 7,
                packet: PacketKind::RequestBuildPayloadCallPacket(RequestBuildPayloadCallPacket {
                    player: EntityRef::null(),
                    build: BuildingRef::new(loader_tile),
                }),
            });
        }

        let changed = launcher.apply_new_network_server_events();

        assert_eq!(changed, 1);
        assert_eq!(launcher.runtime_request_build_payload_packets_seen, 1);
        assert_eq!(launcher.runtime_request_build_payload_packets_accepted, 1);
        assert_eq!(launcher.runtime_request_build_payload_packets_rejected, 0);
        assert_eq!(launcher.runtime_picked_build_payload_packets_sent, 1);
        assert!(launcher
            .last_runtime_request_build_payload_outcome
            .as_ref()
            .is_some_and(|outcome| {
                outcome.accepted
                    && outcome.pickup == Some(super::BuildPayloadPickupKind::StoredPayload)
            }));
        assert!(launcher
            .last_runtime_picked_build_payload_outcome
            .as_ref()
            .is_some_and(|outcome| outcome.accepted));
        let Some(GameRuntimePayloadBlockState::Loader { common, .. }) =
            launcher.runtime.payload_runtime_states.get(&loader_tile)
        else {
            panic!("loader payload runtime state should remain");
        };
        assert!(common.payload.is_none());
        let unit = launcher.server_units.get(&7).unwrap();
        let payloads = &unit.payload.as_ref().unwrap().payloads;
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].kind, PayloadKind::Build);
        assert_eq!(payloads[0].size, router_payload_size);

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                    packet,
                    PacketKind::PickedBuildPayloadCallPacket(packet)
                        if packet.unit == UnitRef::Unit { id: 7 }
                            && packet.build_pos == Some(loader_tile)
                            && !packet.on_ground
                )
        }));
    }

    #[test]
    fn server_launcher_applies_request_build_payload_packet_to_whole_build_and_broadcasts_pickup() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6571).unwrap();

        let default_team = TeamId(launcher.runtime.state.rules.default_team as u8);
        let router_block = launcher
            .content_loader
            .block_by_name("router")
            .unwrap()
            .base()
            .clone();
        let router_payload_size = router_block.size as f32 * TILE_SIZE as f32;
        let router_tile = point2_pack(4, 4);
        launcher
            .runtime
            .add_building(BuildingComp::new(router_tile, router_block, default_team));

        let mut unit_type = launcher
            .content_loader
            .unit_by_name("alpha")
            .unwrap()
            .clone();
        unit_type.payload_capacity = 512.0;
        let mut unit = UnitComp::new(7, unit_type, default_team);
        unit.payload = Some(PayloadComp::new(default_team, 512.0));
        launcher.server_units.insert(7, unit);

        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            let mut connection = NetConnection::new("127.0.0.1:7011");
            connection.has_connected = true;
            connection.player_added = true;
            connection.team = default_team;
            connection.name = "rust-client".into();
            state.connection_states.insert(7, connection);
            state.events.push(ProviderEvent::ServerPacket {
                connection_id: 7,
                packet: PacketKind::RequestBuildPayloadCallPacket(RequestBuildPayloadCallPacket {
                    player: EntityRef::null(),
                    build: BuildingRef::new(router_tile),
                }),
            });
        }

        let changed = launcher.apply_new_network_server_events();

        assert_eq!(changed, 1);
        assert_eq!(launcher.runtime_request_build_payload_packets_seen, 1);
        assert_eq!(launcher.runtime_request_build_payload_packets_accepted, 1);
        assert_eq!(launcher.runtime_request_build_payload_packets_rejected, 0);
        assert_eq!(launcher.runtime_picked_build_payload_packets_sent, 1);
        assert!(launcher
            .last_runtime_request_build_payload_outcome
            .as_ref()
            .is_some_and(|outcome| {
                outcome.accepted
                    && outcome.pickup == Some(super::BuildPayloadPickupKind::WholeBuild)
            }));
        assert!(!launcher
            .runtime
            .buildings()
            .iter()
            .any(|building| building.tile_pos == router_tile));
        let unit = launcher.server_units.get(&7).unwrap();
        let payloads = &unit.payload.as_ref().unwrap().payloads;
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].kind, PayloadKind::Build);
        assert_eq!(payloads[0].size, router_payload_size);

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                    packet,
                    PacketKind::PickedBuildPayloadCallPacket(packet)
                        if packet.unit == UnitRef::Unit { id: 7 }
                            && packet.build_pos == Some(router_tile)
                            && packet.on_ground
                )
        }));
    }

    #[test]
    fn server_update_flushes_pending_world_data_after_connect_packet() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher {
            context: AppContext::server("config"),
            args: Vec::new(),
            control: super::ServerControl::new(Vec::new()),
            runtime: GameRuntime::default(),
            content_loader: ContentLoader::create_base_content_or_panic(),
            last_runtime_effect_report: None,
            last_runtime_item_transport_report: None,
            last_runtime_payload_report: None,
            last_runtime_power_node_config_result: None,
            runtime_power_node_config_packets_seen: 0,
            runtime_power_node_config_packets_changed: 0,
            last_runtime_unit_cargo_unload_config_result: None,
            runtime_unit_cargo_unload_config_packets_seen: 0,
            runtime_unit_cargo_unload_config_packets_changed: 0,
            runtime_unit_cargo_unload_config_packets_forwarded: 0,
            server_preview_players: BTreeMap::new(),
            server_units: BTreeMap::new(),
            runtime_request_item_packets_seen: 0,
            runtime_request_item_packets_accepted: 0,
            runtime_request_item_packets_rejected: 0,
            runtime_take_items_packets_sent: 0,
            runtime_transfer_item_effect_packets_sent: 0,
            last_runtime_request_item_outcome: None,
            last_runtime_take_items_outcome: None,
            runtime_transfer_inventory_packets_seen: 0,
            runtime_transfer_inventory_packets_accepted: 0,
            runtime_transfer_inventory_packets_rejected: 0,
            runtime_transfer_item_to_packets_sent: 0,
            last_runtime_transfer_inventory_outcome: None,
            last_runtime_transfer_item_to_outcome: None,
            runtime_request_drop_payload_packets_seen: 0,
            runtime_request_drop_payload_packets_accepted: 0,
            runtime_request_drop_payload_packets_rejected: 0,
            runtime_payload_dropped_packets_sent: 0,
            last_runtime_request_drop_payload_outcome: None,
            last_runtime_payload_dropped_outcome: None,
            runtime_request_build_payload_packets_seen: 0,
            runtime_request_build_payload_packets_accepted: 0,
            runtime_request_build_payload_packets_rejected: 0,
            runtime_picked_build_payload_packets_sent: 0,
            last_runtime_request_build_payload_outcome: None,
            last_runtime_picked_build_payload_outcome: None,
            runtime_request_unit_payload_packets_seen: 0,
            runtime_request_unit_payload_packets_accepted: 0,
            runtime_request_unit_payload_packets_rejected: 0,
            runtime_picked_unit_payload_packets_sent: 0,
            last_runtime_request_unit_payload_outcome: None,
            last_runtime_picked_unit_payload_outcome: None,
            runtime_drop_item_packets_seen: 0,
            runtime_drop_item_packets_accepted: 0,
            runtime_drop_item_packets_rejected: 0,
            runtime_drop_item_packets_sent: 0,
            last_runtime_drop_item_outcome: None,
            server_preview_plan_packets_applied: 0,
            next_server_preview_broadcast_at: None,
            server_preview_broadcasts_sent: 0,
            next_network_event_index: 0,
            net_server: NetServer::new(Net::new(Box::new(provider))),
            network_error: None,
        };

        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(71),
                false,
                PacketKind::Connect(Connect {
                    address_tcp: "127.0.0.1:6567".into(),
                }),
            );
            net.handle_server_received_from_connection(
                Some(71),
                false,
                PacketKind::ConnectPacket(connect_packet("rust-player")),
            );
        }

        launcher.update();

        {
            let state = launcher.net_server.state();
            let state = state.lock().unwrap();
            assert!(state.pending_world_data_connections.is_empty());
            assert_eq!(state.last_world_data_connection_id, Some(71));
            assert_eq!(state.world_streams_sent, 1);
            assert!(state.last_world_data_bytes.is_some_and(|bytes| bytes > 0));
        }

        let sent = sent.lock().unwrap();
        assert!(sent.len() >= 2);
        assert!(sent
            .iter()
            .all(|(connection_id, _, _)| *connection_id == 71));
        assert!(sent.iter().all(|(_, _, reliable)| *reliable));
        assert!(matches!(sent[0].1, PacketKind::StreamBegin(_)));
        assert!(sent[1..]
            .iter()
            .all(|(_, packet, _)| matches!(packet, PacketKind::StreamChunk(_))));
        assert_eq!(decode_captured_world_data(&sent, 71).player_id, 71);
        assert!(!sent
            .iter()
            .any(|(_, packet, _)| matches!(packet, PacketKind::WorldDataBeginCallPacket(_))));
    }

    #[test]
    fn server_update_flushes_pending_world_data_with_runtime_team_blocks_snapshot() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher {
            context: AppContext::server("config"),
            args: Vec::new(),
            control: super::ServerControl::new(Vec::new()),
            runtime: GameRuntime::default(),
            content_loader: ContentLoader::create_base_content_or_panic(),
            last_runtime_effect_report: None,
            last_runtime_item_transport_report: None,
            last_runtime_payload_report: None,
            last_runtime_power_node_config_result: None,
            runtime_power_node_config_packets_seen: 0,
            runtime_power_node_config_packets_changed: 0,
            last_runtime_unit_cargo_unload_config_result: None,
            runtime_unit_cargo_unload_config_packets_seen: 0,
            runtime_unit_cargo_unload_config_packets_changed: 0,
            runtime_unit_cargo_unload_config_packets_forwarded: 0,
            server_preview_players: BTreeMap::new(),
            server_units: BTreeMap::new(),
            runtime_request_item_packets_seen: 0,
            runtime_request_item_packets_accepted: 0,
            runtime_request_item_packets_rejected: 0,
            runtime_take_items_packets_sent: 0,
            runtime_transfer_item_effect_packets_sent: 0,
            last_runtime_request_item_outcome: None,
            last_runtime_take_items_outcome: None,
            runtime_transfer_inventory_packets_seen: 0,
            runtime_transfer_inventory_packets_accepted: 0,
            runtime_transfer_inventory_packets_rejected: 0,
            runtime_transfer_item_to_packets_sent: 0,
            last_runtime_transfer_inventory_outcome: None,
            last_runtime_transfer_item_to_outcome: None,
            runtime_request_drop_payload_packets_seen: 0,
            runtime_request_drop_payload_packets_accepted: 0,
            runtime_request_drop_payload_packets_rejected: 0,
            runtime_payload_dropped_packets_sent: 0,
            last_runtime_request_drop_payload_outcome: None,
            last_runtime_payload_dropped_outcome: None,
            runtime_request_build_payload_packets_seen: 0,
            runtime_request_build_payload_packets_accepted: 0,
            runtime_request_build_payload_packets_rejected: 0,
            runtime_picked_build_payload_packets_sent: 0,
            last_runtime_request_build_payload_outcome: None,
            last_runtime_picked_build_payload_outcome: None,
            runtime_request_unit_payload_packets_seen: 0,
            runtime_request_unit_payload_packets_accepted: 0,
            runtime_request_unit_payload_packets_rejected: 0,
            runtime_picked_unit_payload_packets_sent: 0,
            last_runtime_request_unit_payload_outcome: None,
            last_runtime_picked_unit_payload_outcome: None,
            runtime_drop_item_packets_seen: 0,
            runtime_drop_item_packets_accepted: 0,
            runtime_drop_item_packets_rejected: 0,
            runtime_drop_item_packets_sent: 0,
            last_runtime_drop_item_outcome: None,
            server_preview_plan_packets_applied: 0,
            next_server_preview_broadcast_at: None,
            server_preview_broadcasts_sent: 0,
            next_network_event_index: 0,
            net_server: NetServer::new(Net::new(Box::new(provider))),
            network_error: None,
        };
        let router_id = launcher
            .content_loader
            .block_by_name("router")
            .expect("base content should include router")
            .base()
            .id;
        launcher.runtime.state.teams.replace_plans([(
            TEAM_SHARDED,
            vec![BlockPlan::new(5, 6, 1, "router", Some("cfg".into()))],
        )]);

        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(72),
                false,
                PacketKind::Connect(Connect {
                    address_tcp: "127.0.0.1:6567".into(),
                }),
            );
            net.handle_server_received_from_connection(
                Some(72),
                false,
                PacketKind::ConnectPacket(connect_packet("rust-builder")),
            );
        }

        launcher.update();

        let sent = sent.lock().unwrap();
        let world_data = decode_captured_world_data(&sent, 72);
        assert_eq!(world_data.player_id, 72);
        assert!(world_data.content_header_snapshot.is_some());
        assert!(world_data.map_snapshot.is_some());
        let team_blocks = world_data
            .team_blocks_snapshot
            .expect("runtime team plans should be exported into world data");
        let sharded = team_blocks
            .groups
            .iter()
            .find(|group| group.team_id == TEAM_SHARDED as i32)
            .expect("Java writeTeamBlocks includes sharded data");
        assert_eq!(sharded.plans.len(), 1);
        assert_eq!(sharded.plans[0].x, 5);
        assert_eq!(sharded.plans[0].y, 6);
        assert_eq!(sharded.plans[0].rotation, 1);
        assert_eq!(sharded.plans[0].block_id, router_id);
        assert_eq!(sharded.plans[0].config, TypeValue::String("cfg".into()));
    }

    #[test]
    fn server_update_applies_power_node_tile_config_packets_to_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let node_def = launcher
            .content_loader
            .block_by_name("power-node")
            .expect("base content should include power-node")
            .base()
            .clone();
        let source_pos = point2_pack(10, 10);
        let int_target_pos = point2_pack(14, 10);
        let array_target_pos = point2_pack(10, 15);

        launcher.runtime.state.world.resize(32, 32);
        for pos in [source_pos, int_target_pos, array_target_pos] {
            launcher
                .runtime
                .add_building(BuildingComp::new(pos, node_def.clone(), TeamId(1)));
        }

        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(91),
                true,
                PacketKind::TileConfigCallPacket(TileConfigCallPacket::client(
                    BuildingRef::new(source_pos),
                    TypeValue::Int(int_target_pos),
                )),
            );
        }

        assert_eq!(launcher.apply_new_network_server_events(), 1);
        assert_eq!(
            launcher.last_runtime_power_node_config_result,
            Some(GameRuntimePowerNodeConfigResult::Link(
                GameRuntimePowerNodeLinkResult::Linked
            ))
        );
        assert_eq!(launcher.runtime_power_node_config_packets_seen, 1);
        assert_eq!(launcher.runtime_power_node_config_packets_changed, 1);

        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(91),
                true,
                PacketKind::TileConfigCallPacket(TileConfigCallPacket::client(
                    BuildingRef::new(source_pos),
                    TypeValue::Point2Array(vec![Point2::new(0, 5)]),
                )),
            );
        }

        assert_eq!(launcher.apply_new_network_server_events(), 1);
        assert_eq!(
            launcher.last_runtime_power_node_config_result,
            Some(GameRuntimePowerNodeConfigResult::Batch(
                GameRuntimePowerNodeBatchLinkReport {
                    cleared: 1,
                    linked: 1,
                    ..GameRuntimePowerNodeBatchLinkReport::default()
                }
            ))
        );
        assert_eq!(launcher.runtime_power_node_config_packets_seen, 2);
        assert_eq!(launcher.runtime_power_node_config_packets_changed, 2);

        let source = launcher
            .runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == source_pos)
            .unwrap();
        assert_eq!(source.power.as_ref().unwrap().links, vec![array_target_pos]);
    }

    #[test]
    fn server_update_applies_unit_cargo_unload_tile_config_and_forwards_to_clients() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6581).unwrap();

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
        let unload_tile = point2_pack(7, 6);
        launcher.runtime.state.world.resize(16, 16);
        launcher.runtime.add_building(BuildingComp::new(
            unload_tile,
            unload_def.base().clone(),
            TeamId(6),
        ));

        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            for connection_id in [10, 11] {
                let mut connection = NetConnection::new(format!("127.0.0.1:{connection_id}"));
                connection.has_connected = true;
                connection.player_added = true;
                connection.team = TeamId(6);
                state.connection_states.insert(connection_id, connection);
            }
        }

        let value = TypeValue::Content(mindustry_core::mindustry::io::type_io::ContentRef::new(
            ContentType::Item,
            copper,
        ));
        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(10),
                true,
                PacketKind::TileConfigCallPacket(TileConfigCallPacket::client(
                    BuildingRef::new(unload_tile),
                    value.clone(),
                )),
            );
        }

        assert_eq!(launcher.apply_new_network_server_events(), 1);
        assert_eq!(
            launcher.last_runtime_unit_cargo_unload_config_result,
            Some(GameRuntimeUnitCargoUnloadConfigureResult::Configured)
        );
        assert_eq!(launcher.runtime_unit_cargo_unload_config_packets_seen, 1);
        assert_eq!(launcher.runtime_unit_cargo_unload_config_packets_changed, 1);
        assert_eq!(
            launcher.runtime_unit_cargo_unload_config_packets_forwarded,
            2
        );
        assert_eq!(launcher.runtime.buildings()[0].config, Some(value.clone()));
        let Some(GameRuntimeDistributionBlockState::UnitCargoUnload(state)) = launcher
            .runtime
            .distribution_runtime_states
            .get(&unload_tile)
        else {
            panic!("unit cargo unload state should be configured");
        };
        assert_eq!(state.item_id, Some(copper as i32));

        let sent = sent.lock().unwrap();
        for connection_id in [10, 11] {
            assert!(sent.iter().any(|(target, packet, reliable)| {
                *target == connection_id
                    && *reliable
                    && matches!(
                        packet,
                        PacketKind::TileConfigCallPacket(packet)
                            if packet.player == EntityRef::new(10)
                                && packet.build == BuildingRef::new(unload_tile)
                                && packet.value == value
                    )
            }));
        }
    }

    #[test]
    fn server_update_applies_unit_factory_command_tile_config_and_forwards_to_clients() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6582).unwrap();

        let factory_def = launcher
            .content_loader
            .block_by_name("air-factory")
            .unwrap();
        let rebuild_id = launcher
            .content_loader
            .unit_command_by_name("rebuild")
            .unwrap()
            .id();
        let factory_tile = point2_pack(8, 6);
        launcher.runtime.state.world.resize(16, 16);
        let mut factory_building =
            BuildingComp::new(factory_tile, factory_def.base().clone(), TeamId(6));
        factory_building.config = Some(TypeValue::Int(0));
        launcher.runtime.add_building(factory_building);
        launcher.runtime.unit_runtime_states.insert(
            factory_tile,
            GameRuntimeUnitBlockState::Factory {
                common: PayloadBlockBuildState::default(),
                factory: UnitFactoryState {
                    current_plan: 0,
                    base: UnitBlockState {
                        progress: 11.0,
                        ..UnitBlockState::default()
                    },
                    ..UnitFactoryState::default()
                },
            },
        );

        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            for connection_id in [21, 22] {
                let mut connection = NetConnection::new(format!("127.0.0.1:{connection_id}"));
                connection.has_connected = true;
                connection.player_added = true;
                connection.team = TeamId(6);
                state.connection_states.insert(connection_id, connection);
            }
        }

        let command_value =
            TypeValue::Content(mindustry_core::mindustry::io::type_io::ContentRef::new(
                ContentType::UnitCommand,
                rebuild_id,
            ));
        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(21),
                true,
                PacketKind::TileConfigCallPacket(TileConfigCallPacket::client(
                    BuildingRef::new(factory_tile),
                    command_value.clone(),
                )),
            );
        }

        assert_eq!(launcher.apply_new_network_server_events(), 1);
        let Some(GameRuntimeUnitBlockState::Factory { factory, .. }) =
            launcher.runtime.unit_runtime_states.get(&factory_tile)
        else {
            panic!("unit factory sidecar should stay present after command config");
        };
        assert_eq!(factory.command_id, Some(rebuild_id as u8));
        assert_eq!(factory.current_plan, 0);
        assert_eq!(factory.base.progress, 11.0);
        assert_eq!(
            launcher.runtime.buildings()[0].config,
            Some(TypeValue::Int(0))
        );

        {
            let sent = sent.lock().unwrap();
            for connection_id in [21, 22] {
                assert!(sent.iter().any(|(target, packet, reliable)| {
                    *target == connection_id
                        && *reliable
                        && matches!(
                            packet,
                            PacketKind::TileConfigCallPacket(packet)
                                if packet.player == EntityRef::new(21)
                                    && packet.build == BuildingRef::new(factory_tile)
                                    && packet.value == command_value
                        )
                }));
            }
        }

        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(22),
                true,
                PacketKind::TileConfigCallPacket(TileConfigCallPacket::client(
                    BuildingRef::new(factory_tile),
                    TypeValue::Null,
                )),
            );
        }

        assert_eq!(launcher.apply_new_network_server_events(), 1);
        let Some(GameRuntimeUnitBlockState::Factory { factory, .. }) =
            launcher.runtime.unit_runtime_states.get(&factory_tile)
        else {
            panic!("unit factory sidecar should stay present after command clear");
        };
        assert_eq!(factory.command_id, None);
        assert_eq!(factory.current_plan, 0);
        assert_eq!(
            launcher.runtime.buildings()[0].config,
            Some(TypeValue::Int(0))
        );
    }

    #[test]
    fn server_update_applies_reconstructor_command_tile_config_and_forwards_to_clients() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6583).unwrap();

        let reconstructor_def = launcher
            .content_loader
            .block_by_name("additive-reconstructor")
            .unwrap();
        let rebuild_id = launcher
            .content_loader
            .unit_command_by_name("rebuild")
            .unwrap()
            .id();
        let reconstructor_tile = point2_pack(10, 6);
        launcher.runtime.state.world.resize(16, 16);
        launcher.runtime.add_building(BuildingComp::new(
            reconstructor_tile,
            reconstructor_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.unit_runtime_states.insert(
            reconstructor_tile,
            GameRuntimeUnitBlockState::Reconstructor {
                common: PayloadBlockBuildState::default(),
                reconstructor: ReconstructorState {
                    base: UnitBlockState {
                        progress: 17.0,
                        ..UnitBlockState::default()
                    },
                    constructing: true,
                    ..ReconstructorState::default()
                },
            },
        );

        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            for connection_id in [41, 42] {
                let mut connection = NetConnection::new(format!("127.0.0.1:{connection_id}"));
                connection.has_connected = true;
                connection.player_added = true;
                connection.team = TeamId(6);
                state.connection_states.insert(connection_id, connection);
            }
        }

        let command_value =
            TypeValue::Content(mindustry_core::mindustry::io::type_io::ContentRef::new(
                ContentType::UnitCommand,
                rebuild_id,
            ));
        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(41),
                true,
                PacketKind::TileConfigCallPacket(TileConfigCallPacket::client(
                    BuildingRef::new(reconstructor_tile),
                    command_value.clone(),
                )),
            );
        }

        assert_eq!(launcher.apply_new_network_server_events(), 1);
        let Some(GameRuntimeUnitBlockState::Reconstructor { reconstructor, .. }) = launcher
            .runtime
            .unit_runtime_states
            .get(&reconstructor_tile)
        else {
            panic!("reconstructor sidecar should stay present after command config");
        };
        assert_eq!(reconstructor.command_id, Some(rebuild_id as u8));
        assert_eq!(reconstructor.base.progress, 17.0);
        assert!(reconstructor.constructing);
        assert_eq!(launcher.runtime.buildings()[0].config, None);

        {
            let sent = sent.lock().unwrap();
            for connection_id in [41, 42] {
                assert!(sent.iter().any(|(target, packet, reliable)| {
                    *target == connection_id
                        && *reliable
                        && matches!(
                            packet,
                            PacketKind::TileConfigCallPacket(packet)
                                if packet.player == EntityRef::new(41)
                                    && packet.build == BuildingRef::new(reconstructor_tile)
                                    && packet.value == command_value
                        )
                }));
            }
        }

        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(42),
                true,
                PacketKind::TileConfigCallPacket(TileConfigCallPacket::client(
                    BuildingRef::new(reconstructor_tile),
                    TypeValue::Null,
                )),
            );
        }

        assert_eq!(launcher.apply_new_network_server_events(), 1);
        let Some(GameRuntimeUnitBlockState::Reconstructor { reconstructor, .. }) = launcher
            .runtime
            .unit_runtime_states
            .get(&reconstructor_tile)
        else {
            panic!("reconstructor sidecar should stay present after command clear");
        };
        assert_eq!(reconstructor.command_id, None);
        assert_eq!(reconstructor.base.progress, 17.0);
        assert_eq!(launcher.runtime.buildings()[0].config, None);

        let sent = sent.lock().unwrap();
        for connection_id in [41, 42] {
            assert!(sent.iter().any(|(target, packet, reliable)| {
                *target == connection_id
                    && *reliable
                    && matches!(
                        packet,
                        PacketKind::TileConfigCallPacket(packet)
                            if packet.player == EntityRef::new(42)
                                && packet.build == BuildingRef::new(reconstructor_tile)
                                && packet.value == TypeValue::Null
                    )
            }));
        }
    }

    #[test]
    fn server_update_applies_command_building_packet_to_unit_factory_and_forwards() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6583).unwrap();

        let factory_def = launcher
            .content_loader
            .block_by_name("air-factory")
            .unwrap();
        let factory_tile = point2_pack(9, 6);
        launcher.runtime.state.world.resize(16, 16);
        let mut factory_building =
            BuildingComp::new(factory_tile, factory_def.base().clone(), TeamId(6));
        factory_building.config = Some(TypeValue::Int(0));
        launcher.runtime.add_building(factory_building);
        launcher.runtime.unit_runtime_states.insert(
            factory_tile,
            GameRuntimeUnitBlockState::Factory {
                common: PayloadBlockBuildState::default(),
                factory: UnitFactoryState {
                    current_plan: 0,
                    ..UnitFactoryState::default()
                },
            },
        );

        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            for connection_id in [31, 32] {
                let mut connection = NetConnection::new(format!("127.0.0.1:{connection_id}"));
                connection.has_connected = true;
                connection.player_added = true;
                connection.team = TeamId(6);
                connection.name = format!("player-{connection_id}");
                connection.color = 0xAA_BB_CC_DD_u32 as i32;
                state.connection_states.insert(connection_id, connection);
            }
        }

        let target = Vec2::new(48.0, 80.0);
        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(31),
                true,
                PacketKind::CommandBuildingCallPacket(CommandBuildingCallPacket {
                    player: EntityRef::null(),
                    buildings: vec![factory_tile],
                    target,
                }),
            );
        }

        assert_eq!(launcher.apply_new_network_server_events(), 1);
        let Some(GameRuntimeUnitBlockState::Factory { factory, .. }) =
            launcher.runtime.unit_runtime_states.get(&factory_tile)
        else {
            panic!("unit factory sidecar should stay present after command building");
        };
        assert_eq!(factory.command_pos, Some(target));
        assert_eq!(
            launcher.runtime.buildings()[0].last_accessed,
            "[#AABBCCDD]player-31"
        );

        let sent = sent.lock().unwrap();
        for connection_id in [31, 32] {
            assert!(sent.iter().any(|(target_connection, packet, reliable)| {
                *target_connection == connection_id
                    && *reliable
                    && matches!(
                        packet,
                        PacketKind::CommandBuildingCallPacket(packet)
                            if packet.player == EntityRef::new(31)
                                && packet.buildings == vec![factory_tile]
                                && packet.target == target
                    )
            }));
        }
    }

    #[test]
    fn server_update_broadcasts_unit_block_spawn_when_unit_factory_payload_dumps() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6585).unwrap();
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(24, 24);

        let factory_def = launcher
            .content_loader
            .block_by_name("air-factory")
            .unwrap();
        let conveyor_def = launcher
            .content_loader
            .block_by_name("payload-conveyor")
            .unwrap();
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        let BlockDef::UnitFactory(factory_block) = factory_def else {
            panic!("air-factory should be a unit factory");
        };
        let BlockDef::Payload(conveyor_block) = conveyor_def else {
            panic!("payload-conveyor should be a payload block");
        };
        let factory_tile = point2_pack(8, 8);
        let conveyor_tile = point2_pack(
            8 + factory_block.base.size / 2 + 1 + (conveyor_block.base.size - 1) / 2,
            8,
        );
        let mut factory_building =
            BuildingComp::new(factory_tile, factory_def.base().clone(), TeamId(6));
        factory_building.set_rotation(0);
        launcher.runtime.add_building(factory_building);
        launcher.runtime.add_building(BuildingComp::new(
            conveyor_tile,
            conveyor_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.payload_runtime_states.insert(
            conveyor_tile,
            GameRuntimePayloadBlockState::Conveyor(PayloadConveyorState::default()),
        );
        let mut common = PayloadBlockBuildState {
            payload: Some(PayloadRef::Block {
                block: router_def.base().id,
                version: 0,
                build_bytes: Vec::new(),
            }),
            ..PayloadBlockBuildState::default()
        };
        common.pay_vector.x = 12.0;
        common.pay_vector.y = 0.0;
        launcher.runtime.unit_runtime_states.insert(
            factory_tile,
            GameRuntimeUnitBlockState::Factory {
                common,
                factory: UnitFactoryState {
                    current_plan: -1,
                    base: UnitBlockState {
                        has_payload: true,
                        progress: 4.0,
                        ..UnitBlockState::default()
                    },
                    ..UnitFactoryState::default()
                },
            },
        );

        launcher.update();

        let Some(GameRuntimeUnitBlockState::Factory { common, factory }) =
            launcher.runtime.unit_runtime_states.get(&factory_tile)
        else {
            panic!("factory sidecar should remain present");
        };
        assert!(common.payload.is_none());
        assert!(!factory.base.has_payload);

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                    packet,
                    PacketKind::UnitBlockSpawnCallPacket(packet)
                        if packet.tile == Some(factory_tile)
                )
        }));
    }

    #[test]
    fn server_update_broadcasts_assembler_unit_spawn_packet_when_assembler_completes() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6586).unwrap();
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.rules.default_team = 6;
        launcher.runtime.state.set_sector(Some(Sector::new(21)));
        launcher.runtime.state.world.resize(24, 24);

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
        let BlockDef::UnitAssembler(assembler_block) = assembler_def else {
            panic!("tank-assembler should be a unit assembler");
        };
        let assembler_block = assembler_block.clone();
        let plan = &assembler_block.plans[0];
        let plan_time = plan.time;
        let plan_unit_name = plan.unit.clone();
        let assembler_tile = point2_pack(8, 8);
        launcher.runtime.add_building(BuildingComp::new(
            assembler_tile,
            assembler_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.buildings[0].enabled = true;
        if let Some(power) = launcher.runtime.buildings[0].power.as_mut() {
            power.status = 1.0;
        }
        let expected_spawn_x = launcher.runtime.buildings()[0].x
            + TILE_SIZE as f32 * (assembler_block.area_size + assembler_block.base.size) as f32
                / 2.0;
        let expected_spawn_y = launcher.runtime.buildings()[0].y;

        let mut blocks = PayloadSeq::new();
        blocks.add(PayloadKey::new(ContentType::Unit, stell.id()), 4);
        blocks.add(
            PayloadKey::new(ContentType::Block, large_wall.base().id),
            10,
        );
        blocks.add(PayloadKey::new(ContentType::Block, router.base().id), 2);
        let command_pos = Vec2::new(96.0, 128.0);
        launcher.runtime.unit_runtime_states.insert(
            assembler_tile,
            GameRuntimeUnitBlockState::Assembler {
                common: PayloadBlockBuildState::default(),
                assembler: UnitAssemblerState {
                    progress: 1.0 - 1.0 / plan_time,
                    blocks,
                    command_pos: Some(command_pos),
                    ..UnitAssemblerState::default()
                },
            },
        );
        seed_server_assembler_drones_in_position(&mut launcher, assembler_tile, &assembler_block);

        launcher.update();

        let Some(GameRuntimeUnitBlockState::Assembler { assembler, .. }) =
            launcher.runtime.unit_runtime_states.get(&assembler_tile)
        else {
            panic!("assembler sidecar should remain present");
        };
        assert_eq!(assembler.progress, 0.0);
        assert_eq!(assembler.blocks.total(), 0);
        assert_eq!(launcher.runtime.unit_create_events.len(), 1);
        assert_eq!(launcher.runtime.unit_create_events[0].unit_id, None);
        assert_eq!(
            launcher.runtime.unit_create_events[0].unit_name,
            plan_unit_name
        );
        assert_eq!(launcher.runtime.unit_create_events[0].team, TeamId(6));
        assert_eq!(
            launcher.runtime.unit_create_events[0].spawner_tile,
            Some(assembler_tile)
        );
        assert_eq!(launcher.runtime.state.stats.units_created, 1);
        assert_eq!(
            launcher
                .runtime
                .campaign_stats
                .get_unit_produced(&plan_unit_name),
            1
        );

        let spawned_unit = launcher
            .server_units
            .values()
            .find(|unit| unit.type_info.name() == plan_unit_name)
            .expect("assembler completion should materialize the output unit server-side");
        assert_eq!(spawned_unit.team_id(), TeamId(6));
        assert_eq!(spawned_unit.x(), expected_spawn_x);
        assert_eq!(spawned_unit.y(), expected_spawn_y);
        assert_eq!(spawned_unit.rotation(), 0.0);
        let UnitControllerState::Command(command) = &spawned_unit.controller else {
            panic!("commanded assembler output should preserve command controller");
        };
        assert_eq!(command.target_pos, Some(command_pos));

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                    packet,
                    PacketKind::AssemblerUnitSpawnedCallPacket(packet)
                        if packet.tile == Some(assembler_tile)
                )
        }));
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                    packet,
                    PacketKind::UnitSpawnCallPacket(packet)
                        if packet.container.unit_id == spawned_unit.id()
                )
        }));
    }

    #[test]
    fn server_update_ticks_unit_spawn_ability_and_broadcasts_spawned_unit() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6592).unwrap();
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.rules.disable_unit_cap = true;
        launcher.runtime.state.set_sector(Some(Sector::new(22)));

        let mut carrier_type = UnitType::new(9000, "unit-spawn-carrier");
        carrier_type
            .abilities
            .push("UnitSpawnAbility:flare:1:2:4".into());
        let mut carrier = UnitComp::new(7, carrier_type, TeamId(1));
        carrier.set_pos(100.0, 200.0);
        carrier.set_rotation(90.0);
        launcher.server_units.insert(7, carrier);

        launcher.update();

        let spawned_unit = launcher
            .server_units
            .values()
            .find(|unit| unit.id() != 7 && unit.type_info.name() == "flare")
            .expect("UnitSpawnAbility should materialize child unit server-side");
        assert_eq!(spawned_unit.team_id(), TeamId(1));
        assert!((spawned_unit.x() - 102.0).abs() < 0.0001);
        assert!((spawned_unit.y() - 204.0).abs() < 0.0001);
        assert_eq!(spawned_unit.rotation(), 90.0);

        assert_eq!(launcher.runtime.unit_create_events.len(), 1);
        assert_eq!(
            launcher.runtime.unit_create_events[0].unit_id,
            Some(spawned_unit.id())
        );
        assert_eq!(launcher.runtime.unit_create_events[0].unit_name, "flare");
        assert_eq!(launcher.runtime.unit_create_events[0].team, TeamId(1));
        assert_eq!(launcher.runtime.unit_create_events[0].spawner_tile, None);
        assert_eq!(
            launcher.runtime.unit_create_events[0].spawner_unit_id,
            Some(7)
        );
        assert_eq!(launcher.runtime.state.stats.units_created, 1);
        assert_eq!(
            launcher.runtime.campaign_stats.get_unit_produced("flare"),
            1
        );

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                    packet,
                    PacketKind::UnitSpawnCallPacket(packet)
                        if packet.container.unit_id == spawned_unit.id()
                )
        }));
    }

    #[test]
    fn server_update_ticks_aegires_energy_field_against_units() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);

        let aegires = launcher
            .content_loader
            .unit_by_name("aegires")
            .unwrap()
            .clone();
        let flare = launcher
            .content_loader
            .unit_by_name("flare")
            .unwrap()
            .clone();
        let mut parent = UnitComp::new(10, aegires.clone(), TeamId(1));
        parent.set_pos(100.0, 100.0);
        parent.abilities[0].data = 64.0;
        parent.weapons.ammo = 0.0;

        let mut ally = UnitComp::new(11, aegires, TeamId(1));
        ally.set_pos(120.0, 100.0);
        ally.health.health = ally.health.max_health - 200.0;

        let mut enemy = UnitComp::new(12, flare, TeamId(2));
        enemy.set_pos(140.0, 100.0);
        let enemy_health_before = enemy.health.health;

        launcher.server_units.insert(parent.id(), parent);
        launcher.server_units.insert(ally.id(), ally);
        launcher.server_units.insert(enemy.id(), enemy);

        launcher.update();

        let parent = launcher.server_units.get(&10).unwrap();
        assert_eq!(parent.abilities[0].data, 0.0);
        assert_eq!(parent.weapons.ammo, 0.0);

        let ally = launcher.server_units.get(&11).unwrap();
        assert!((ally.health.health - (ally.health.max_health - 110.0)).abs() < 0.0001);
        assert!(ally.was_healed);

        let enemy = launcher.server_units.get(&12).unwrap();
        assert!((enemy.health.health - (enemy_health_before - 40.0)).abs() < 0.0001);
        assert_eq!(enemy.status.get_duration("electrified"), 60.0 * 6.0);
    }

    #[test]
    fn server_update_ticks_quasar_force_field_regen() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);

        let quasar = launcher
            .content_loader
            .unit_by_name("quasar")
            .unwrap()
            .clone();
        let mut parent = UnitComp::new(32, quasar, TeamId(1));
        assert_eq!(parent.shield.shield, 500.0);
        parent.shield.shield = 400.0;

        launcher.server_units.insert(parent.id(), parent);
        launcher.update();

        let parent = launcher.server_units.get(&32).unwrap();
        assert!((parent.shield.shield - 400.4).abs() < 0.0001);
        assert!(parent.abilities[0].data > 0.0);
    }

    #[test]
    fn server_update_ticks_tecta_shield_arc_regen() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);

        let tecta = launcher
            .content_loader
            .unit_by_name("tecta")
            .unwrap()
            .clone();
        let mut parent = UnitComp::new(33, tecta, TeamId(1));
        assert_eq!(parent.abilities[0].data, 2500.0);
        parent.abilities[0].data = 2000.0;

        launcher.server_units.insert(parent.id(), parent);
        launcher.update();

        let parent = launcher.server_units.get(&33).unwrap();
        assert!((parent.abilities[0].data - 2000.75).abs() < 0.0001);
    }

    #[test]
    fn server_update_spawns_renales_when_latum_dies() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);

        let latum = launcher
            .content_loader
            .unit_by_name("latum")
            .unwrap()
            .clone();
        let team = TeamId(launcher.runtime.state.rules.default_team as u8);
        let mut parent = UnitComp::new(34, latum, team);
        parent.set_pos(100.0, 200.0);
        parent.health.kill();
        launcher.server_units.insert(parent.id(), parent);

        launcher.update();

        assert!(!launcher.server_units.contains_key(&34));
        let renales = launcher
            .server_units
            .values()
            .filter(|unit| unit.type_info.name() == "renale" && unit.team_id() == team)
            .collect::<Vec<_>>();
        assert_eq!(renales.len(), 5);
        assert!(renales.iter().all(|unit| {
            let dx = unit.x() - 100.0;
            let dy = unit.y() - 200.0;
            (dx * dx + dy * dy).sqrt() <= 11.001
        }));
        assert_eq!(launcher.runtime.unit_create_events.len(), 5);
        assert_eq!(launcher.runtime.state.stats.units_created, 5);
    }

    #[test]
    fn server_update_ticks_renale_neoplasm_regen() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);

        let renale = launcher
            .content_loader
            .unit_by_name("renale")
            .unwrap()
            .clone();
        let mut unit = UnitComp::new(35, renale, TeamId(1));
        unit.health.damage(42.0);
        let damaged = unit.health.health;
        launcher.server_units.insert(unit.id(), unit);

        launcher.update();

        let unit = launcher.server_units.get(&35).unwrap();
        let expected = damaged + unit.health.max_health * (1.0 / (70.0 * 60.0));
        assert!((unit.health.health - expected).abs() < 0.0001);
        assert!(unit.was_healed);
    }

    #[test]
    fn server_update_deposits_neoplasm_when_renale_dies() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(32, 32);

        let renale = launcher
            .content_loader
            .unit_by_name("renale")
            .unwrap()
            .clone();
        let mut unit = UnitComp::new(36, renale, TeamId(1));
        unit.set_pos(80.0, 96.0);
        unit.health.kill();
        launcher.server_units.insert(unit.id(), unit);

        launcher.update();

        assert!(!launcher.server_units.contains_key(&36));
        let entry = launcher
            .runtime
            .server_puddles
            .get_entry(10, 12)
            .expect("renale death should deposit neoplasm on its tile");
        assert_eq!(entry.puddle.amount, 70.0);
        assert_eq!(entry.liquid.name, "neoplasm");
    }

    #[test]
    fn server_update_slurps_neoplasm_puddle_to_regen_renale() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6593).unwrap();
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(32, 32);
        launcher.runtime.server_puddles = Puddles::new(32, 32);
        let neoplasm = launcher.content_loader.liquid_by_name("neoplasm").unwrap();
        launcher.runtime.server_puddles.deposit_at(
            Some(PuddleTileView::new(10, 12)),
            PuddleLiquidInfo::from(neoplasm),
            20.0,
            PuddleDepositContext::default(),
        );

        let renale = launcher
            .content_loader
            .unit_by_name("renale")
            .unwrap()
            .clone();
        let mut unit = UnitComp::new(37, renale, TeamId(1));
        unit.set_pos(80.0, 96.0);
        unit.health.damage(100.0);
        let damaged = unit.health.health;
        launcher.server_units.insert(unit.id(), unit);

        launcher.update();

        let unit = launcher.server_units.get(&37).unwrap();
        let passive_regen = unit.health.max_health * (1.0 / (70.0 * 60.0));
        assert!((unit.health.health - (damaged + passive_regen + 30.0)).abs() < 0.0001);
        assert!(unit.was_healed);
        assert!(
            (launcher.runtime.server_puddles.get(10, 12).unwrap().amount - 14.97).abs() < 0.0001
        );
        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                    packet,
                    PacketKind::EffectCallPacket2(packet)
                        if packet.effect.effect_id
                            == standard_effect_id("neoplasmHeal").unwrap() as u16
                            && packet.effect.x == 80.0
                            && packet.effect.y == 96.0
                            && packet.data == TypeValue::Unit(37)
                )
        }));
    }

    #[test]
    fn server_update_hides_puddle_entity_when_liquid_regen_drains_it_empty() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6594).unwrap();
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(32, 32);
        launcher.runtime.server_puddles = Puddles::new(32, 32);
        let neoplasm = launcher.content_loader.liquid_by_name("neoplasm").unwrap();
        launcher.runtime.server_puddles.deposit_at(
            Some(PuddleTileView::new(10, 12)),
            PuddleLiquidInfo::from(neoplasm),
            5.0,
            PuddleDepositContext::default(),
        );
        let puddle_id = launcher.runtime.server_puddles.get(10, 12).unwrap().id;

        let renale = launcher
            .content_loader
            .unit_by_name("renale")
            .unwrap()
            .clone();
        let mut unit = UnitComp::new(38, renale, TeamId(1));
        unit.set_pos(80.0, 96.0);
        unit.health.damage(100.0);
        launcher.server_units.insert(unit.id(), unit);

        launcher.update();

        assert!(launcher.runtime.server_puddles.get(10, 12).is_none());
        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                    packet,
                    PacketKind::HiddenSnapshotCallPacket(packet)
                        if packet.ids == vec![puddle_id]
                )
        }));
    }

    #[test]
    fn server_update_spreads_overfilled_puddle_and_snapshots_neighbors() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6595).unwrap();
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(8, 8);
        let copper_wall = launcher
            .content_loader
            .block_by_name("copper-wall")
            .unwrap()
            .base()
            .id;
        launcher.runtime.state.world.tile_mut(3, 2).unwrap().block = copper_wall;
        launcher.runtime.server_puddles = Puddles::new(8, 8);
        let water = launcher.content_loader.liquid_by_name("water").unwrap();
        launcher.runtime.server_puddles.deposit_at(
            Some(PuddleTileView::new(2, 2)),
            PuddleLiquidInfo::from(water),
            70.0,
            PuddleDepositContext::default(),
        );

        launcher.update();

        assert_eq!(launcher.runtime.server_puddles.len(), 4);
        assert!(launcher.runtime.server_puddles.get(3, 2).is_none());
        assert!((launcher.runtime.server_puddles.get(2, 2).unwrap().amount - 69.0).abs() < 0.0001);
        let snapshot = sent
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find_map(|(_connection_id, packet, reliable)| {
                if !*reliable {
                    if let PacketKind::EntitySnapshotCallPacket(packet) = packet {
                        return Some(packet.clone());
                    }
                }
                None
            })
            .expect("puddle spread should be broadcast as entity snapshots");
        assert_eq!(snapshot.amount, 4);
    }

    #[test]
    fn server_update_ticks_scepter_shield_regen_field_for_nearby_allies() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);

        let scepter = launcher
            .content_loader
            .unit_by_name("scepter")
            .unwrap()
            .clone();
        let dagger = launcher
            .content_loader
            .unit_by_name("dagger")
            .unwrap()
            .clone();
        let mut parent = UnitComp::new(24, scepter, TeamId(1));
        parent.set_pos(100.0, 100.0);
        parent.abilities[0].data = 59.0;

        let mut ally = UnitComp::new(25, dagger.clone(), TeamId(1));
        ally.set_pos(130.0, 100.0);
        ally.shield.shield = 240.0;
        let mut far_ally = UnitComp::new(26, dagger.clone(), TeamId(1));
        far_ally.set_pos(300.0, 100.0);
        let mut enemy = UnitComp::new(27, dagger, TeamId(2));
        enemy.set_pos(120.0, 100.0);

        launcher.server_units.insert(parent.id(), parent);
        launcher.server_units.insert(ally.id(), ally);
        launcher.server_units.insert(far_ally.id(), far_ally);
        launcher.server_units.insert(enemy.id(), enemy);

        launcher.update();

        let parent = launcher.server_units.get(&24).unwrap();
        assert_eq!(parent.abilities[0].data, 0.0);
        assert_eq!(parent.shield.shield, 25.0);
        assert_eq!(parent.shield.shield_alpha, 1.0);

        let ally = launcher.server_units.get(&25).unwrap();
        assert_eq!(ally.shield.shield, 250.0);
        assert_eq!(ally.shield.shield_alpha, 1.0);
        let far_ally = launcher.server_units.get(&26).unwrap();
        assert_eq!(far_ally.shield.shield, 0.0);
        let enemy = launcher.server_units.get(&27).unwrap();
        assert_eq!(enemy.shield.shield, 0.0);
    }

    #[test]
    fn server_update_ticks_nova_repair_field_for_nearby_allies() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);

        let nova = launcher
            .content_loader
            .unit_by_name("nova")
            .unwrap()
            .clone();
        let dagger = launcher
            .content_loader
            .unit_by_name("dagger")
            .unwrap()
            .clone();
        let mut parent = UnitComp::new(28, nova, TeamId(1));
        parent.set_pos(100.0, 100.0);
        parent.health.damage(30.0);
        parent.abilities[0].data = 239.0;

        let mut ally = UnitComp::new(29, dagger.clone(), TeamId(1));
        ally.set_pos(130.0, 100.0);
        ally.health.damage(50.0);
        let mut far_ally = UnitComp::new(30, dagger.clone(), TeamId(1));
        far_ally.set_pos(300.0, 100.0);
        far_ally.health.damage(50.0);
        let mut enemy = UnitComp::new(31, dagger, TeamId(2));
        enemy.set_pos(120.0, 100.0);
        enemy.health.damage(50.0);

        launcher.server_units.insert(parent.id(), parent);
        launcher.server_units.insert(ally.id(), ally);
        launcher.server_units.insert(far_ally.id(), far_ally);
        launcher.server_units.insert(enemy.id(), enemy);

        launcher.update();

        let parent = launcher.server_units.get(&28).unwrap();
        assert_eq!(parent.abilities[0].data, 0.0);
        assert_eq!(parent.health.health, 100.0);
        assert!(parent.was_healed);

        let ally = launcher.server_units.get(&29).unwrap();
        assert_eq!(ally.health.health, 110.0);
        assert!(ally.was_healed);
        let far_ally = launcher.server_units.get(&30).unwrap();
        assert_eq!(far_ally.health.health, 100.0);
        assert!(!far_ally.was_healed);
        let enemy = launcher.server_units.get(&31).unwrap();
        assert_eq!(enemy.health.health, 100.0);
        assert!(!enemy.was_healed);
    }

    #[test]
    fn server_update_ticks_oxynoe_status_field_for_nearby_allies() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);

        let oxynoe = launcher
            .content_loader
            .unit_by_name("oxynoe")
            .unwrap()
            .clone();
        let risso = launcher
            .content_loader
            .unit_by_name("risso")
            .unwrap()
            .clone();
        let mut parent = UnitComp::new(20, oxynoe, TeamId(1));
        parent.set_pos(100.0, 100.0);
        parent.abilities[0].data = 359.0;

        let mut ally = UnitComp::new(21, risso.clone(), TeamId(1));
        ally.set_pos(140.0, 100.0);
        let mut far_ally = UnitComp::new(22, risso.clone(), TeamId(1));
        far_ally.set_pos(300.0, 100.0);
        let mut enemy = UnitComp::new(23, risso, TeamId(2));
        enemy.set_pos(120.0, 100.0);

        launcher.server_units.insert(parent.id(), parent);
        launcher.server_units.insert(ally.id(), ally);
        launcher.server_units.insert(far_ally.id(), far_ally);
        launcher.server_units.insert(enemy.id(), enemy);

        launcher.update();

        let parent = launcher.server_units.get(&20).unwrap();
        assert_eq!(parent.abilities[0].data, 0.0);
        assert_eq!(parent.status.get_duration("overclock"), 60.0 * 6.0);

        let ally = launcher.server_units.get(&21).unwrap();
        assert_eq!(ally.status.get_duration("overclock"), 60.0 * 6.0);
        let far_ally = launcher.server_units.get(&22).unwrap();
        assert_eq!(far_ally.status.get_duration("overclock"), 0.0);
        let enemy = launcher.server_units.get(&23).unwrap();
        assert_eq!(enemy.status.get_duration("overclock"), 0.0);
    }

    #[test]
    fn server_update_ticks_quell_suppression_field_for_enemy_buildings() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(64, 64);

        let quell = launcher
            .content_loader
            .unit_by_name("quell")
            .unwrap()
            .clone();
        let router = launcher.content_loader.block_by_name("router").unwrap();
        let mut parent = UnitComp::new(30, quell, TeamId(1));
        parent.set_pos(80.0, 80.0);
        parent.set_rotation(0.0);
        parent.abilities[0].data = 89.0;

        let near_enemy_tile = point2_pack(11, 10);
        let near_same_team_tile = point2_pack(12, 10);
        let far_enemy_tile = point2_pack(40, 40);
        launcher.server_units.insert(parent.id(), parent);
        launcher.runtime.add_building(BuildingComp::new(
            near_enemy_tile,
            router.base().clone(),
            TeamId(2),
        ));
        launcher.runtime.add_building(BuildingComp::new(
            near_same_team_tile,
            router.base().clone(),
            TeamId(1),
        ));
        launcher.runtime.add_building(BuildingComp::new(
            far_enemy_tile,
            router.base().clone(),
            TeamId(2),
        ));

        launcher.update();

        let now = launcher.runtime.state.tick as f32;
        let parent = launcher.server_units.get(&30).unwrap();
        assert_eq!(parent.abilities[0].data, 0.0);
        assert!(launcher.runtime.buildings()[0].is_heal_suppressed(now));
        assert!(!launcher.runtime.buildings()[1].is_heal_suppressed(now));
        assert!(!launcher.runtime.buildings()[2].is_heal_suppressed(now));
        assert_eq!(
            launcher.runtime.buildings()[0].heal_suppression_time,
            now + 481.0
        );
    }

    #[test]
    fn server_launcher_unit_assembler_spawn_delivers_payload_to_build_on_target() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6587).unwrap();
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(24, 24);

        let assembler_def = launcher
            .content_loader
            .block_by_name("tank-assembler")
            .unwrap();
        let BlockDef::UnitAssembler(assembler_block) = assembler_def else {
            panic!("tank-assembler should be a unit assembler");
        };
        let assembler_block = assembler_block.clone();
        let plan = &assembler_block.plans[0];
        let plan_unit_name = plan.unit.clone();
        let assembler_tile = point2_pack(8, 8);
        let stell = launcher.content_loader.unit_by_name("stell").unwrap();
        let large_wall = launcher
            .content_loader
            .block_by_name("tungsten-wall-large")
            .unwrap();
        launcher.runtime.add_building(BuildingComp::new(
            assembler_tile,
            assembler_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.buildings[0].enabled = true;
        if let Some(power) = launcher.runtime.buildings[0].power.as_mut() {
            power.status = 1.0;
        }
        let expected_spawn_x = launcher.runtime.buildings()[0].x
            + TILE_SIZE as f32 * (assembler_block.area_size + assembler_block.base.size) as f32
                / 2.0;
        let expected_spawn_y = launcher.runtime.buildings()[0].y;
        let spawn_tile = point2_pack(
            mindustry_core::mindustry::core::World::to_tile(expected_spawn_x),
            mindustry_core::mindustry::core::World::to_tile(expected_spawn_y),
        );

        let payload_driver_def = launcher
            .content_loader
            .block_by_name("large-payload-mass-driver")
            .unwrap();
        launcher.runtime.add_building(BuildingComp::new(
            spawn_tile,
            payload_driver_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.buildings[1].enabled = true;

        let mut blocks = PayloadSeq::new();
        blocks.add(PayloadKey::new(ContentType::Unit, stell.id()), 4);
        blocks.add(
            PayloadKey::new(ContentType::Block, large_wall.base().id),
            10,
        );
        launcher.runtime.unit_runtime_states.insert(
            assembler_tile,
            GameRuntimeUnitBlockState::Assembler {
                common: PayloadBlockBuildState::default(),
                assembler: UnitAssemblerState {
                    progress: 1.0 - 1.0 / plan.time,
                    blocks,
                    command_pos: Some(Vec2::new(96.0, 128.0)),
                    ..UnitAssemblerState::default()
                },
            },
        );
        seed_server_assembler_drones_in_position(&mut launcher, assembler_tile, &assembler_block);

        for _ in 0..120 {
            launcher.update();
        }

        let Some(GameRuntimeUnitBlockState::Assembler { assembler, .. }) =
            launcher.runtime.unit_runtime_states.get(&assembler_tile)
        else {
            panic!("assembler sidecar should remain present");
        };
        assert_eq!(assembler.progress, 0.0);
        assert_eq!(assembler.blocks.total(), 0);
        assert!(!launcher
            .server_units
            .values()
            .any(|unit| unit.type_info.name() == plan_unit_name));
        let Some(GameRuntimePayloadBlockState::MassDriver { common, driver }) =
            launcher.runtime.payload_runtime_states.get(&spawn_tile)
        else {
            panic!("build_on payload mass driver should receive assembler unit payload");
        };
        assert!(common.payload.is_some());
        assert!(driver.loaded);

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                    packet,
                    PacketKind::AssemblerUnitSpawnedCallPacket(packet)
                        if packet.tile == Some(assembler_tile)
                )
        }));
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                    packet,
                    PacketKind::UnitEnteredPayloadCallPacket(packet)
                        if packet.build.tile_pos == Some(spawn_tile)
                )
        }));
    }

    #[test]
    fn server_update_broadcasts_assembler_drone_spawn_packet_and_tethers_unit() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6587).unwrap();
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(24, 24);

        let assembler_def = launcher
            .content_loader
            .block_by_name("tank-assembler")
            .unwrap();
        let BlockDef::UnitAssembler(assembler_block) = assembler_def else {
            panic!("tank-assembler should be a unit assembler");
        };
        let drone_type_name = assembler_block.drone_type.clone();
        let assembler_tile = point2_pack(9, 8);
        launcher.runtime.add_building(BuildingComp::new(
            assembler_tile,
            assembler_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.buildings[0].enabled = true;
        if let Some(power) = launcher.runtime.buildings[0].power.as_mut() {
            power.status = 1.0;
        }
        launcher.runtime.unit_runtime_states.insert(
            assembler_tile,
            GameRuntimeUnitBlockState::Assembler {
                common: PayloadBlockBuildState::default(),
                assembler: UnitAssemblerState {
                    drone_progress: 1.0,
                    ..UnitAssemblerState::default()
                },
            },
        );

        launcher.update();

        let Some(GameRuntimeUnitBlockState::Assembler { assembler, .. }) =
            launcher.runtime.unit_runtime_states.get(&assembler_tile)
        else {
            panic!("assembler sidecar should remain present");
        };
        assert_eq!(assembler.drone_progress, 0.0);
        assert_eq!(assembler.read_unit_ids.len(), 1);
        let drone_id = assembler.read_unit_ids[0];
        let drone = launcher
            .server_units
            .get(&drone_id)
            .expect("assembler drone should be materialized server-side");
        assert_eq!(drone.type_info.name(), drone_type_name);
        assert_eq!(drone.team_id(), TeamId(6));
        assert_eq!(drone.x(), launcher.runtime.buildings()[0].x);
        assert_eq!(drone.y(), launcher.runtime.buildings()[0].y);
        assert_eq!(drone.rotation(), 90.0);
        assert!(matches!(drone.controller, UnitControllerState::Assembler));
        let tether = drone
            .building_tether
            .as_ref()
            .expect("assembler drone should keep a formal building tether");
        assert_eq!(tether.team, TeamId(6));
        assert_eq!(
            tether.building,
            Some(BuildingTetherRef {
                tile_pos: assembler_tile,
                team: TeamId(6),
                valid: true,
            })
        );
        assert_eq!(tether.update(), BuildingTetherAction::Keep);

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                    packet,
                    PacketKind::AssemblerDroneSpawnedCallPacket(packet)
                        if packet.tile == Some(assembler_tile) && packet.id == drone_id
                )
        }));
    }

    #[test]
    fn server_update_moves_assembler_drone_toward_slot_target() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(24, 24);

        let assembler_def = launcher
            .content_loader
            .block_by_name("tank-assembler")
            .unwrap();
        let BlockDef::UnitAssembler(assembler_block) = assembler_def else {
            panic!("tank-assembler should be a unit assembler");
        };
        let assembler_block = assembler_block.clone();
        let drone_type = launcher
            .content_loader
            .unit_by_name(&assembler_block.drone_type)
            .unwrap()
            .clone();
        let assembler_tile = point2_pack(9, 8);
        launcher.runtime.add_building(BuildingComp::new(
            assembler_tile,
            assembler_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.unit_runtime_states.insert(
            assembler_tile,
            GameRuntimeUnitBlockState::Assembler {
                common: PayloadBlockBuildState::default(),
                assembler: UnitAssemblerState {
                    read_unit_ids: vec![7],
                    ..UnitAssemblerState::default()
                },
            },
        );
        let building = launcher.runtime.buildings()[0].clone();
        let (dir_x, dir_y) = autotiler_direction(building.rotation);
        let spawn_len =
            TILE_SIZE as f32 * (assembler_block.area_size + assembler_block.base.size) as f32 / 2.0;
        let spawn_x = building.x + dir_x as f32 * spawn_len;
        let spawn_y = building.y + dir_y as f32 * spawn_len;
        let target = unit_assembler_drone_target(
            spawn_x,
            spawn_y,
            assembler_block.area_size,
            TILE_SIZE as f32,
            0,
        );

        let mut drone = UnitComp::new(7, drone_type, TeamId(6));
        drone.set_pos(building.x, building.y);
        drone.set_rotation(90.0);
        drone.set_controller(UnitControllerState::Assembler);
        drone.building_tether = Some(BuildingTetherComp {
            team: TeamId(6),
            building: Some(BuildingTetherRef {
                tile_pos: assembler_tile,
                team: TeamId(6),
                valid: true,
            }),
        });
        let before =
            ((target.pos.x - drone.x()).powi(2) + (target.pos.y - drone.y()).powi(2)).sqrt();
        launcher.server_units.insert(7, drone);

        assert_eq!(launcher.tick_runtime_unit_assembler_ai(), 1);

        let moved = launcher.server_units.get(&7).unwrap();
        let after =
            ((target.pos.x - moved.x()).powi(2) + (target.pos.y - moved.y()).powi(2)).sqrt();
        assert!(after < before);
        assert!(launcher
            .runtime
            .client_unit_snapshot_entities
            .contains_key(&7));
    }

    #[test]
    fn server_update_broadcasts_landing_pad_landed_packet_when_waiting_pad_selected() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6588).unwrap();
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(24, 24);
        launcher.runtime.state.set_sector(Some(Sector::new(13)));

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
        if let Some(sector) = launcher.runtime.state.rules.sector.as_mut() {
            sector.info.imports.insert(
                "copper".into(),
                ExportStat {
                    mean: 6000.0,
                    ..ExportStat::default()
                },
            );
            sector
                .info
                .import_cooldown_timers
                .insert("copper".into(), 1.0);
        }
        if let Some(sector) = launcher.runtime.state.sector.as_mut() {
            sector.info.imports.insert(
                "copper".into(),
                ExportStat {
                    mean: 6000.0,
                    ..ExportStat::default()
                },
            );
            sector
                .info
                .import_cooldown_timers
                .insert("copper".into(), 1.0);
        }

        let tile_pos = point2_pack(10, 9);
        launcher.runtime.add_building(BuildingComp::new(
            tile_pos,
            landing_def.base().clone(),
            TeamId(launcher.runtime.state.rules.default_team as u8),
        ));
        launcher.runtime.campaign_runtime_states.insert(
            tile_pos,
            GameRuntimeCampaignBlockState::LandingPad(LandingPadState {
                config: Some(copper),
                ..LandingPadState::default()
            }),
        );
        launcher
            .runtime
            .landing_pad_waiting
            .insert(copper, vec![tile_pos]);

        launcher.update();

        let Some(GameRuntimeCampaignBlockState::LandingPad(state)) =
            launcher.runtime.campaign_runtime_states.get(&tile_pos)
        else {
            panic!("landing pad state should remain present");
        };
        assert_eq!(state.arriving, Some(copper));
        assert_eq!(state.cooldown, 1.0);

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                    packet,
                    PacketKind::LandingPadLandedCallPacket(packet)
                        if packet.tile == Some(tile_pos)
                )
        }));
    }

    #[test]
    fn server_update_records_client_plan_snapshot_and_broadcasts_preview_to_teammates() {
        let sent_packets = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent_packets),
        };
        let mut launcher = ServerLauncher {
            context: AppContext::server("config"),
            args: Vec::new(),
            control: super::ServerControl::new(Vec::new()),
            runtime: GameRuntime::default(),
            content_loader: ContentLoader::create_base_content_or_panic(),
            last_runtime_effect_report: None,
            last_runtime_item_transport_report: None,
            last_runtime_payload_report: None,
            last_runtime_power_node_config_result: None,
            runtime_power_node_config_packets_seen: 0,
            runtime_power_node_config_packets_changed: 0,
            last_runtime_unit_cargo_unload_config_result: None,
            runtime_unit_cargo_unload_config_packets_seen: 0,
            runtime_unit_cargo_unload_config_packets_changed: 0,
            runtime_unit_cargo_unload_config_packets_forwarded: 0,
            server_preview_players: BTreeMap::new(),
            server_units: BTreeMap::new(),
            runtime_request_item_packets_seen: 0,
            runtime_request_item_packets_accepted: 0,
            runtime_request_item_packets_rejected: 0,
            runtime_take_items_packets_sent: 0,
            runtime_transfer_item_effect_packets_sent: 0,
            last_runtime_request_item_outcome: None,
            last_runtime_take_items_outcome: None,
            runtime_transfer_inventory_packets_seen: 0,
            runtime_transfer_inventory_packets_accepted: 0,
            runtime_transfer_inventory_packets_rejected: 0,
            runtime_transfer_item_to_packets_sent: 0,
            last_runtime_transfer_inventory_outcome: None,
            last_runtime_transfer_item_to_outcome: None,
            runtime_request_drop_payload_packets_seen: 0,
            runtime_request_drop_payload_packets_accepted: 0,
            runtime_request_drop_payload_packets_rejected: 0,
            runtime_payload_dropped_packets_sent: 0,
            last_runtime_request_drop_payload_outcome: None,
            last_runtime_payload_dropped_outcome: None,
            runtime_request_build_payload_packets_seen: 0,
            runtime_request_build_payload_packets_accepted: 0,
            runtime_request_build_payload_packets_rejected: 0,
            runtime_picked_build_payload_packets_sent: 0,
            last_runtime_request_build_payload_outcome: None,
            last_runtime_picked_build_payload_outcome: None,
            runtime_request_unit_payload_packets_seen: 0,
            runtime_request_unit_payload_packets_accepted: 0,
            runtime_request_unit_payload_packets_rejected: 0,
            runtime_picked_unit_payload_packets_sent: 0,
            last_runtime_request_unit_payload_outcome: None,
            last_runtime_picked_unit_payload_outcome: None,
            runtime_drop_item_packets_seen: 0,
            runtime_drop_item_packets_accepted: 0,
            runtime_drop_item_packets_rejected: 0,
            runtime_drop_item_packets_sent: 0,
            last_runtime_drop_item_outcome: None,
            server_preview_plan_packets_applied: 0,
            next_server_preview_broadcast_at: None,
            server_preview_broadcasts_sent: 0,
            next_network_event_index: 0,
            net_server: NetServer::new(Net::new(Box::new(provider))),
            network_error: None,
        };
        let plans = vec![BuildPlanWire::new_place(5, 6, 1, "router")];

        {
            let mut net = launcher.net_server.net_mut();
            for connection_id in [11, 22, 33, 44] {
                net.handle_server_received_from_connection(
                    Some(connection_id),
                    true,
                    PacketKind::Connect(Connect {
                        address_tcp: format!("127.0.0.1:{connection_id}"),
                    }),
                );
            }
        }
        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            for (connection_id, name, team) in [
                (11, "source", TeamId(1)),
                (22, "teammate", TeamId(1)),
                (33, "enemy", TeamId(2)),
                (44, "disconnected-teammate", TeamId(1)),
            ] {
                let connection = state.connection_states.get_mut(&connection_id).unwrap();
                connection.name = name.into();
                connection.team = team;
                connection.has_connected = true;
                connection.player_added = true;
            }
            state
                .connection_states
                .get_mut(&44)
                .unwrap()
                .has_disconnected = true;
        }
        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(11),
                true,
                PacketKind::ClientPlanSnapshotCallPacket(ClientPlanSnapshotCallPacket {
                    group_id: 7,
                    plans: Some(plans.clone()),
                }),
            );
        }

        assert_eq!(launcher.apply_new_network_server_events(), 1);
        assert_eq!(launcher.network_error, None);

        let sent = sent_packets.lock().unwrap();
        assert!(sent.is_empty());
        drop(sent);
        {
            let player = launcher.server_preview_players.get(&11).unwrap();
            assert_eq!(player.id, 11);
            assert_eq!(player.name, "source");
            assert_eq!(player.team, TeamId(1));
            assert_eq!(player.last_preview_plan_group, 7);
            assert_eq!(player.preview_plans_assembling.len(), 1);
            assert!(player.preview_plans_current.is_empty());
            assert!(player.con.is_some());
        }

        let broadcasted = launcher
            .broadcast_server_preview_plans(i64::MAX / 4)
            .expect("preview broadcast should write to capture provider");
        assert_eq!(broadcasted, 2);

        let sent = sent_packets.lock().unwrap();
        assert_eq!(sent.len(), 2);
        assert!(sent.iter().any(|(connection_id, packet, reliable)| {
            *connection_id == 22
                && !*reliable
                && matches!(
                    packet,
                    PacketKind::ClientPlanSnapshotReceivedCallPacket(packet)
                        if packet.player_id == 11
                            && packet.group_id == 0
                            && packet.plans.as_ref() == Some(&plans)
                )
        }));
        assert!(sent.iter().any(|(connection_id, packet, reliable)| {
            *connection_id == 11
                && !*reliable
                && matches!(
                    packet,
                    PacketKind::ClientPlanSnapshotReceivedCallPacket(packet)
                        if packet.player_id == 22 && packet.group_id == 0 && packet.plans.is_none()
                )
        }));
        assert!(!sent
            .iter()
            .any(|(connection_id, _, _)| *connection_id == 33));
        assert!(!sent
            .iter()
            .any(|(connection_id, _, _)| *connection_id == 44));
        drop(sent);

        let state = launcher.net_server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.client_plan_snapshot_packets_seen, 1);
        assert_eq!(state.client_plan_snapshots_forwarded, 2);
        assert!(state.last_client_plan_snapshot_forwarded_error.is_none());
        drop(state);
        assert_eq!(launcher.server_preview_plan_packets_applied, 1);
        assert_eq!(launcher.server_preview_broadcasts_sent, 2);
        let player = launcher.server_preview_players.get(&11).unwrap();
        assert_eq!(player.last_preview_plan_group, 7);
        assert_eq!(player.last_preview_plan_group_server, 0);
        assert!(player.preview_plans_assembling.is_empty());
        assert_eq!(player.preview_plans_current.len(), 1);
        let target = launcher.server_preview_players.get(&22).unwrap();
        assert_eq!(target.last_preview_plan_group_server, 0);
        assert!(target.preview_plans_current.is_empty());
    }

    #[test]
    fn server_preview_due_broadcast_syncs_empty_ready_players() {
        let sent_packets = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent_packets),
        };
        let mut launcher = ServerLauncher {
            context: AppContext::server("config"),
            args: Vec::new(),
            control: super::ServerControl::new(Vec::new()),
            runtime: GameRuntime::default(),
            content_loader: ContentLoader::create_base_content_or_panic(),
            last_runtime_effect_report: None,
            last_runtime_item_transport_report: None,
            last_runtime_payload_report: None,
            last_runtime_power_node_config_result: None,
            runtime_power_node_config_packets_seen: 0,
            runtime_power_node_config_packets_changed: 0,
            last_runtime_unit_cargo_unload_config_result: None,
            runtime_unit_cargo_unload_config_packets_seen: 0,
            runtime_unit_cargo_unload_config_packets_changed: 0,
            runtime_unit_cargo_unload_config_packets_forwarded: 0,
            server_preview_players: BTreeMap::new(),
            server_units: BTreeMap::new(),
            runtime_request_item_packets_seen: 0,
            runtime_request_item_packets_accepted: 0,
            runtime_request_item_packets_rejected: 0,
            runtime_take_items_packets_sent: 0,
            runtime_transfer_item_effect_packets_sent: 0,
            last_runtime_request_item_outcome: None,
            last_runtime_take_items_outcome: None,
            runtime_transfer_inventory_packets_seen: 0,
            runtime_transfer_inventory_packets_accepted: 0,
            runtime_transfer_inventory_packets_rejected: 0,
            runtime_transfer_item_to_packets_sent: 0,
            last_runtime_transfer_inventory_outcome: None,
            last_runtime_transfer_item_to_outcome: None,
            runtime_request_drop_payload_packets_seen: 0,
            runtime_request_drop_payload_packets_accepted: 0,
            runtime_request_drop_payload_packets_rejected: 0,
            runtime_payload_dropped_packets_sent: 0,
            last_runtime_request_drop_payload_outcome: None,
            last_runtime_payload_dropped_outcome: None,
            runtime_request_build_payload_packets_seen: 0,
            runtime_request_build_payload_packets_accepted: 0,
            runtime_request_build_payload_packets_rejected: 0,
            runtime_picked_build_payload_packets_sent: 0,
            last_runtime_request_build_payload_outcome: None,
            last_runtime_picked_build_payload_outcome: None,
            runtime_request_unit_payload_packets_seen: 0,
            runtime_request_unit_payload_packets_accepted: 0,
            runtime_request_unit_payload_packets_rejected: 0,
            runtime_picked_unit_payload_packets_sent: 0,
            last_runtime_request_unit_payload_outcome: None,
            last_runtime_picked_unit_payload_outcome: None,
            runtime_drop_item_packets_seen: 0,
            runtime_drop_item_packets_accepted: 0,
            runtime_drop_item_packets_rejected: 0,
            runtime_drop_item_packets_sent: 0,
            last_runtime_drop_item_outcome: None,
            server_preview_plan_packets_applied: 0,
            next_server_preview_broadcast_at: None,
            server_preview_broadcasts_sent: 0,
            next_network_event_index: 0,
            net_server: NetServer::new(Net::new(Box::new(provider))),
            network_error: None,
        };

        {
            let mut net = launcher.net_server.net_mut();
            for connection_id in [101, 102] {
                net.handle_server_received_from_connection(
                    Some(connection_id),
                    true,
                    PacketKind::Connect(Connect {
                        address_tcp: format!("127.0.0.1:{connection_id}"),
                    }),
                );
            }
        }
        {
            let state = launcher.net_server.state();
            let mut state = state.lock().unwrap();
            for connection_id in [101, 102] {
                let connection = state.connection_states.get_mut(&connection_id).unwrap();
                connection.team = TeamId(3);
                connection.has_connected = true;
                connection.player_added = true;
            }
        }

        let now = Instant::now();
        let sent = launcher
            .broadcast_server_preview_plans_if_due(now, i64::MAX / 4)
            .expect("empty preview broadcast should be written to capture provider");
        assert_eq!(sent, 2);
        assert_eq!(launcher.server_preview_players.len(), 2);
        assert_eq!(launcher.server_preview_broadcasts_sent, 2);

        let sent = sent_packets.lock().unwrap();
        assert_eq!(sent.len(), 2);
        assert!(sent.iter().any(|(connection_id, packet, reliable)| {
            *connection_id == 102
                && !*reliable
                && matches!(
                    packet,
                    PacketKind::ClientPlanSnapshotReceivedCallPacket(packet)
                        if packet.player_id == 101 && packet.group_id == 0 && packet.plans.is_none()
                )
        }));
        assert!(sent.iter().any(|(connection_id, packet, reliable)| {
            *connection_id == 101
                && !*reliable
                && matches!(
                    packet,
                    PacketKind::ClientPlanSnapshotReceivedCallPacket(packet)
                        if packet.player_id == 102 && packet.group_id == 0 && packet.plans.is_none()
                )
        }));
    }

    #[test]
    fn server_world_data_exports_owned_building_chunks_for_runtime_loader() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher {
            context: AppContext::server("config"),
            args: Vec::new(),
            control: super::ServerControl::new(Vec::new()),
            runtime: GameRuntime::default(),
            content_loader: ContentLoader::create_base_content_or_panic(),
            last_runtime_effect_report: None,
            last_runtime_item_transport_report: None,
            last_runtime_payload_report: None,
            last_runtime_power_node_config_result: None,
            runtime_power_node_config_packets_seen: 0,
            runtime_power_node_config_packets_changed: 0,
            last_runtime_unit_cargo_unload_config_result: None,
            runtime_unit_cargo_unload_config_packets_seen: 0,
            runtime_unit_cargo_unload_config_packets_changed: 0,
            runtime_unit_cargo_unload_config_packets_forwarded: 0,
            server_preview_players: BTreeMap::new(),
            server_units: BTreeMap::new(),
            runtime_request_item_packets_seen: 0,
            runtime_request_item_packets_accepted: 0,
            runtime_request_item_packets_rejected: 0,
            runtime_take_items_packets_sent: 0,
            runtime_transfer_item_effect_packets_sent: 0,
            last_runtime_request_item_outcome: None,
            last_runtime_take_items_outcome: None,
            runtime_transfer_inventory_packets_seen: 0,
            runtime_transfer_inventory_packets_accepted: 0,
            runtime_transfer_inventory_packets_rejected: 0,
            runtime_transfer_item_to_packets_sent: 0,
            last_runtime_transfer_inventory_outcome: None,
            last_runtime_transfer_item_to_outcome: None,
            runtime_request_drop_payload_packets_seen: 0,
            runtime_request_drop_payload_packets_accepted: 0,
            runtime_request_drop_payload_packets_rejected: 0,
            runtime_payload_dropped_packets_sent: 0,
            last_runtime_request_drop_payload_outcome: None,
            last_runtime_payload_dropped_outcome: None,
            runtime_request_build_payload_packets_seen: 0,
            runtime_request_build_payload_packets_accepted: 0,
            runtime_request_build_payload_packets_rejected: 0,
            runtime_picked_build_payload_packets_sent: 0,
            last_runtime_request_build_payload_outcome: None,
            last_runtime_picked_build_payload_outcome: None,
            runtime_request_unit_payload_packets_seen: 0,
            runtime_request_unit_payload_packets_accepted: 0,
            runtime_request_unit_payload_packets_rejected: 0,
            runtime_picked_unit_payload_packets_sent: 0,
            last_runtime_request_unit_payload_outcome: None,
            last_runtime_picked_unit_payload_outcome: None,
            runtime_drop_item_packets_seen: 0,
            runtime_drop_item_packets_accepted: 0,
            runtime_drop_item_packets_rejected: 0,
            runtime_drop_item_packets_sent: 0,
            last_runtime_drop_item_outcome: None,
            server_preview_plan_packets_applied: 0,
            next_server_preview_broadcast_at: None,
            server_preview_broadcasts_sent: 0,
            next_network_event_index: 0,
            net_server: NetServer::new(Net::new(Box::new(provider))),
            network_error: None,
        };

        let router_def = launcher
            .content_loader
            .block_by_name("router")
            .expect("base content should include router");
        let router_block = router_def.base().clone();
        let router_id = router_block.id;
        let tile_pos = point2_pack(4, 4);
        let mut router = BuildingComp::new(tile_pos, router_block, TeamId(1));
        router.set_rotation(2);
        router.health = 12.5;
        launcher.runtime.state.world.resize(12, 12);
        launcher.runtime.add_building(router);

        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(73),
                false,
                PacketKind::Connect(Connect {
                    address_tcp: "127.0.0.1:6567".into(),
                }),
            );
            net.handle_server_received_from_connection(
                Some(73),
                false,
                PacketKind::ConnectPacket(connect_packet("rust-loader")),
            );
        }

        launcher.update();

        let sent = sent.lock().unwrap();
        let world_data = decode_captured_world_data(&sent, 73);
        let map = world_data
            .map_snapshot
            .expect("runtime map snapshot should be sent in world data");
        let center_index = 4 + 4 * 12;
        let record = map
            .blocks
            .iter()
            .find(|record| record.index == center_index)
            .expect("owned building center should be an explicit block record");
        assert_eq!(record.block_id, router_id);
        assert!(record.has_entity);
        assert!(record.is_center);
        assert!(record
            .building
            .as_ref()
            .is_some_and(|bytes| bytes.len() > 1));
        assert_eq!(record.consecutives, 0);

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(&launcher.content_loader, &map);
        assert_eq!(report.building_records, 1);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.building_parse_errors, 0);
        assert_eq!(loaded.buildings().len(), 1);
        let loaded_router = &loaded.buildings()[0];
        assert_eq!(loaded_router.tile_pos, tile_pos);
        assert_eq!(loaded_router.block.id, router_id);
        assert_eq!(loaded_router.team, TeamId(1));
        assert_eq!(loaded_router.rotation, 2);
        assert_eq!(loaded_router.health, 12.5);
    }

    #[test]
    fn server_world_data_roundtrips_payload_loader_state_through_runtime_loader() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher {
            context: AppContext::server("config"),
            args: Vec::new(),
            control: super::ServerControl::new(Vec::new()),
            runtime: GameRuntime::default(),
            content_loader: ContentLoader::create_base_content_or_panic(),
            last_runtime_effect_report: None,
            last_runtime_item_transport_report: None,
            last_runtime_payload_report: None,
            last_runtime_power_node_config_result: None,
            runtime_power_node_config_packets_seen: 0,
            runtime_power_node_config_packets_changed: 0,
            last_runtime_unit_cargo_unload_config_result: None,
            runtime_unit_cargo_unload_config_packets_seen: 0,
            runtime_unit_cargo_unload_config_packets_changed: 0,
            runtime_unit_cargo_unload_config_packets_forwarded: 0,
            server_preview_players: BTreeMap::new(),
            server_units: BTreeMap::new(),
            runtime_request_item_packets_seen: 0,
            runtime_request_item_packets_accepted: 0,
            runtime_request_item_packets_rejected: 0,
            runtime_take_items_packets_sent: 0,
            runtime_transfer_item_effect_packets_sent: 0,
            last_runtime_request_item_outcome: None,
            last_runtime_take_items_outcome: None,
            runtime_transfer_inventory_packets_seen: 0,
            runtime_transfer_inventory_packets_accepted: 0,
            runtime_transfer_inventory_packets_rejected: 0,
            runtime_transfer_item_to_packets_sent: 0,
            last_runtime_transfer_inventory_outcome: None,
            last_runtime_transfer_item_to_outcome: None,
            runtime_request_drop_payload_packets_seen: 0,
            runtime_request_drop_payload_packets_accepted: 0,
            runtime_request_drop_payload_packets_rejected: 0,
            runtime_payload_dropped_packets_sent: 0,
            last_runtime_request_drop_payload_outcome: None,
            last_runtime_payload_dropped_outcome: None,
            runtime_request_build_payload_packets_seen: 0,
            runtime_request_build_payload_packets_accepted: 0,
            runtime_request_build_payload_packets_rejected: 0,
            runtime_picked_build_payload_packets_sent: 0,
            last_runtime_request_build_payload_outcome: None,
            last_runtime_picked_build_payload_outcome: None,
            runtime_request_unit_payload_packets_seen: 0,
            runtime_request_unit_payload_packets_accepted: 0,
            runtime_request_unit_payload_packets_rejected: 0,
            runtime_picked_unit_payload_packets_sent: 0,
            last_runtime_request_unit_payload_outcome: None,
            last_runtime_picked_unit_payload_outcome: None,
            runtime_drop_item_packets_seen: 0,
            runtime_drop_item_packets_accepted: 0,
            runtime_drop_item_packets_rejected: 0,
            runtime_drop_item_packets_sent: 0,
            last_runtime_drop_item_outcome: None,
            server_preview_plan_packets_applied: 0,
            next_server_preview_broadcast_at: None,
            server_preview_broadcasts_sent: 0,
            next_network_event_index: 0,
            net_server: NetServer::new(Net::new(Box::new(provider))),
            network_error: None,
        };

        let loader_def = launcher
            .content_loader
            .block_by_name("payload-loader")
            .expect("base content should include payload-loader");
        let container_def = launcher
            .content_loader
            .block_by_name("container")
            .expect("base content should include container");
        let loader_tile = point2_pack(4, 4);
        let container_id = container_def.base().id;
        let mut payload_bytes = Vec::new();
        BuildingComp::new(point2_pack(0, 0), container_def.base().clone(), TeamId(6))
            .write_base(&mut payload_bytes, false)
            .unwrap();
        let mut loader_building =
            BuildingComp::new(loader_tile, loader_def.base().clone(), TeamId(6));
        loader_building.set_rotation(2);

        launcher.runtime.state.world.resize(12, 12);
        launcher.runtime.add_building(loader_building);
        launcher.runtime.payload_runtime_states.insert(
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

        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(74),
                false,
                PacketKind::Connect(Connect {
                    address_tcp: "127.0.0.1:6567".into(),
                }),
            );
            net.handle_server_received_from_connection(
                Some(74),
                false,
                PacketKind::ConnectPacket(connect_packet("rust-payload-loader")),
            );
        }

        launcher.update();

        let sent = sent.lock().unwrap();
        let world_data = decode_captured_world_data(&sent, 74);
        assert_eq!(world_data.player_id, 74);
        let map = world_data
            .map_snapshot
            .expect("runtime map snapshot should be sent in world data");

        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(&launcher.content_loader, &map);
        assert_eq!(report.building_records, 1);
        assert_eq!(report.buildings_added, 1);
        assert_eq!(report.building_parse_errors, 0);
        assert_eq!(loaded.buildings().len(), 1);
        assert_eq!(loaded.buildings()[0].tile_pos, loader_tile);
        assert_eq!(loaded.buildings()[0].rotation, 2);

        let Some(GameRuntimePayloadBlockState::Loader { common, loader }) =
            loaded.payload_runtime_states.get(&loader_tile)
        else {
            panic!("payload loader sidecar should roundtrip through server world data");
        };
        assert!(loader.exporting);
        assert!(matches!(
            common.payload.as_ref(),
            Some(PayloadRef::Block { block, .. }) if *block == container_id
        ));
    }

    #[test]
    fn server_world_data_roundtrips_payload_router_mass_driver_and_deconstructor_states() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher {
            context: AppContext::server("config"),
            args: Vec::new(),
            control: super::ServerControl::new(Vec::new()),
            runtime: GameRuntime::default(),
            content_loader: ContentLoader::create_base_content_or_panic(),
            last_runtime_effect_report: None,
            last_runtime_item_transport_report: None,
            last_runtime_payload_report: None,
            last_runtime_power_node_config_result: None,
            runtime_power_node_config_packets_seen: 0,
            runtime_power_node_config_packets_changed: 0,
            last_runtime_unit_cargo_unload_config_result: None,
            runtime_unit_cargo_unload_config_packets_seen: 0,
            runtime_unit_cargo_unload_config_packets_changed: 0,
            runtime_unit_cargo_unload_config_packets_forwarded: 0,
            server_preview_players: BTreeMap::new(),
            server_units: BTreeMap::new(),
            runtime_request_item_packets_seen: 0,
            runtime_request_item_packets_accepted: 0,
            runtime_request_item_packets_rejected: 0,
            runtime_take_items_packets_sent: 0,
            runtime_transfer_item_effect_packets_sent: 0,
            last_runtime_request_item_outcome: None,
            last_runtime_take_items_outcome: None,
            runtime_transfer_inventory_packets_seen: 0,
            runtime_transfer_inventory_packets_accepted: 0,
            runtime_transfer_inventory_packets_rejected: 0,
            runtime_transfer_item_to_packets_sent: 0,
            last_runtime_transfer_inventory_outcome: None,
            last_runtime_transfer_item_to_outcome: None,
            runtime_request_drop_payload_packets_seen: 0,
            runtime_request_drop_payload_packets_accepted: 0,
            runtime_request_drop_payload_packets_rejected: 0,
            runtime_payload_dropped_packets_sent: 0,
            last_runtime_request_drop_payload_outcome: None,
            last_runtime_payload_dropped_outcome: None,
            runtime_request_build_payload_packets_seen: 0,
            runtime_request_build_payload_packets_accepted: 0,
            runtime_request_build_payload_packets_rejected: 0,
            runtime_picked_build_payload_packets_sent: 0,
            last_runtime_request_build_payload_outcome: None,
            last_runtime_picked_build_payload_outcome: None,
            runtime_request_unit_payload_packets_seen: 0,
            runtime_request_unit_payload_packets_accepted: 0,
            runtime_request_unit_payload_packets_rejected: 0,
            runtime_picked_unit_payload_packets_sent: 0,
            last_runtime_request_unit_payload_outcome: None,
            last_runtime_picked_unit_payload_outcome: None,
            runtime_drop_item_packets_seen: 0,
            runtime_drop_item_packets_accepted: 0,
            runtime_drop_item_packets_rejected: 0,
            runtime_drop_item_packets_sent: 0,
            last_runtime_drop_item_outcome: None,
            server_preview_plan_packets_applied: 0,
            next_server_preview_broadcast_at: None,
            server_preview_broadcasts_sent: 0,
            next_network_event_index: 0,
            net_server: NetServer::new(Net::new(Box::new(provider))),
            network_error: None,
        };

        let payload_router_def = launcher
            .content_loader
            .block_by_name("payload-router")
            .expect("base content should include payload-router");
        let mass_driver_def = launcher
            .content_loader
            .block_by_name("payload-mass-driver")
            .expect("base content should include payload-mass-driver");
        let deconstructor_def = launcher
            .content_loader
            .block_by_name("small-deconstructor")
            .expect("base content should include small-deconstructor");
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        let router_id = router_def.base().id;
        let router_tile = point2_pack(4, 4);
        let mass_driver_tile = point2_pack(9, 4);
        let deconstructor_tile = point2_pack(14, 4);
        let mut payload_bytes = Vec::new();
        BuildingComp::new(point2_pack(0, 0), router_def.base().clone(), TeamId(6))
            .write_base(&mut payload_bytes, false)
            .unwrap();
        let router_payload = PayloadRef::Block {
            block: router_id,
            version: 0,
            build_bytes: payload_bytes,
        };

        launcher.runtime.state.world.resize(24, 10);
        launcher.runtime.add_building(BuildingComp::new(
            router_tile,
            payload_router_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.add_building(BuildingComp::new(
            mass_driver_tile,
            mass_driver_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.add_building(BuildingComp::new(
            deconstructor_tile,
            deconstructor_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.payload_runtime_states.insert(
            router_tile,
            GameRuntimePayloadBlockState::Router {
                conveyor: PayloadConveyorState {
                    item: Some(router_payload.clone()),
                    step: 1,
                    step_accepted: 0,
                    item_rotation: 45.0,
                    ..PayloadConveyorState::default()
                },
                sorted: Some(PayloadSortKey {
                    content_type: ContentType::Block.ordinal() as i8,
                    id: router_id,
                }),
                rec_dir: 2,
                matches: true,
                smooth_rot: 180.0,
                control_time: -1.0,
            },
        );
        launcher.runtime.payload_runtime_states.insert(
            mass_driver_tile,
            GameRuntimePayloadBlockState::MassDriver {
                common: PayloadBlockBuildState::default(),
                driver: PayloadMassDriverState {
                    link: -1,
                    turret_rotation: 45.0,
                    state: PayloadDriverState::Shooting,
                    reload_counter: 0.25,
                    charge: 0.5,
                    loaded: true,
                    charging: true,
                    ..PayloadMassDriverState::default()
                },
            },
        );
        launcher.runtime.payload_runtime_states.insert(
            deconstructor_tile,
            GameRuntimePayloadBlockState::Deconstructor {
                common: PayloadBlockBuildState::default(),
                deconstructor: PayloadDeconstructorState {
                    progress: 0.5,
                    has_deconstructing: true,
                    deconstructing: Some(router_payload),
                    accum: Some(vec![1.0, 2.0]),
                    ..PayloadDeconstructorState::default()
                },
            },
        );

        {
            let mut net = launcher.net_server.net_mut();
            net.handle_server_received_from_connection(
                Some(75),
                false,
                PacketKind::Connect(Connect {
                    address_tcp: "127.0.0.1:6567".into(),
                }),
            );
            net.handle_server_received_from_connection(
                Some(75),
                false,
                PacketKind::ConnectPacket(connect_packet("rust-payload-state")),
            );
        }

        launcher.update();

        let sent = sent.lock().unwrap();
        let world_data = decode_captured_world_data(&sent, 75);
        let map = world_data
            .map_snapshot
            .expect("runtime map snapshot should be sent in world data");
        let mut loaded = GameRuntime::default();
        let report = loaded.load_network_map_with_buildings(&launcher.content_loader, &map);
        assert_eq!(report.buildings_added, 3);
        assert_eq!(report.building_parse_errors, 0);

        let Some(GameRuntimePayloadBlockState::Router {
            conveyor,
            sorted,
            rec_dir,
            ..
        }) = loaded.payload_runtime_states.get(&router_tile)
        else {
            panic!("payload router sidecar should roundtrip through server world data");
        };
        assert!(matches!(
            conveyor.item.as_ref(),
            Some(PayloadRef::Block { block, .. }) if *block == router_id
        ));
        assert_eq!(
            *sorted,
            Some(PayloadSortKey {
                content_type: ContentType::Block.ordinal() as i8,
                id: router_id,
            })
        );
        assert_eq!(*rec_dir, 2);

        let Some(GameRuntimePayloadBlockState::MassDriver { driver, .. }) =
            loaded.payload_runtime_states.get(&mass_driver_tile)
        else {
            panic!("payload mass driver sidecar should roundtrip through server world data");
        };
        assert_eq!(driver.turret_rotation, 45.0);
        assert_eq!(driver.state, PayloadDriverState::Shooting);
        assert_eq!(driver.reload_counter, 0.25);
        assert_eq!(driver.charge, 0.5);
        assert!(driver.loaded);
        assert!(driver.charging);

        let Some(GameRuntimePayloadBlockState::Deconstructor { deconstructor, .. }) =
            loaded.payload_runtime_states.get(&deconstructor_tile)
        else {
            panic!("payload deconstructor sidecar should roundtrip through server world data");
        };
        assert_eq!(deconstructor.progress, 0.5);
        assert_eq!(deconstructor.accum.as_deref(), Some(&[1.0, 2.0][..]));
        assert!(matches!(
            deconstructor.deconstructing.as_ref(),
            Some(PayloadRef::Block { block, .. }) if *block == router_id
        ));
    }

    #[test]
    fn server_runtime_effect_update_is_wired_to_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        assert!(launcher.update_runtime_effect_blocks(1.0 / 60.0).is_none());

        launcher
            .runtime
            .state
            .set(mindustry_core::mindustry::core::GameStateState::Playing);
        let report = launcher
            .update_runtime_effect_blocks(1.0 / 60.0)
            .expect("playing runtime should produce an empty owned-building batch");
        assert_eq!(report.visited_buildings, 0);
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_drives_owned_effect_building_from_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let mend_def = launcher
            .content_loader
            .block_by_name("mend-projector")
            .unwrap();
        let mend_block = match mend_def {
            BlockDef::Effect(effect) => effect,
            _ => unreachable!(),
        };
        let silicon = mend_block.boost_items[0].item;
        let mut mend = BuildingComp::new(point2_pack(8, 8), mend_def.base().clone(), TeamId(1));
        mend.efficiency = 1.0;
        mend.optional_efficiency = 1.0;
        mend.items.as_mut().unwrap().set(silicon, 1);

        launcher.runtime.state.world.resize(32, 32);
        launcher.runtime.add_building(mend);
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.tick = mend_block.use_time as f64 - 1.0;

        launcher.update();

        let report = launcher
            .last_runtime_effect_report
            .as_ref()
            .expect("server update should keep the latest runtime effect batch");
        assert_eq!(report.visited_buildings, 1);
        assert_eq!(report.effect_candidates, 1);
        assert_eq!(report.reports.len(), 1);
        assert_eq!(
            launcher.runtime.buildings()[0]
                .items
                .as_ref()
                .unwrap()
                .get(silicon),
            0
        );
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_drives_owned_item_transport_from_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        let item_void_def = launcher.content_loader.block_by_name("item-void").unwrap();
        let copper = launcher
            .content_loader
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let router_tile = point2_pack(4, 4);
        let void_tile = point2_pack(5, 4);
        let mut router = BuildingComp::new(router_tile, router_def.base().clone(), TeamId(6));
        router.items.as_mut().unwrap().add(copper, 1);

        launcher.runtime.state.world.resize(10, 10);
        launcher.runtime.add_building(router);
        launcher.runtime.add_building(BuildingComp::new(
            void_tile,
            item_void_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let report = launcher
            .last_runtime_item_transport_report
            .expect("server update should cache the latest item transport batch");
        assert_eq!(report.router_forwarded_items, 1);
        assert_eq!(
            launcher.runtime.buildings()[0]
                .items
                .as_ref()
                .unwrap()
                .get(copper),
            0
        );
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_drives_owned_unit_cargo_loader_from_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let loader_def = launcher
            .content_loader
            .block_by_name("unit-cargo-loader")
            .unwrap();
        let power_source_def = launcher
            .content_loader
            .block_by_name("power-source")
            .unwrap();
        let BlockDef::Distribution(loader_block) = loader_def else {
            panic!("unit-cargo-loader should be a distribution block");
        };
        let nitrogen = launcher
            .content_loader
            .liquid_by_name("nitrogen")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let loader_tile = point2_pack(6, 6);
        let power_source_tile = point2_pack(8, 6);

        launcher.runtime.state.world.resize(16, 16);
        launcher.runtime.add_building(BuildingComp::new(
            loader_tile,
            loader_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.add_building(BuildingComp::new(
            power_source_tile,
            power_source_def.base().clone(),
            TeamId(6),
        ));
        if let Some(power) = launcher.runtime.buildings[0].power.as_mut() {
            power.status = 1.0;
        }
        if let Some(liquids) = launcher.runtime.buildings[0].liquids.as_mut() {
            liquids.set(nitrogen, 20.0);
        }
        launcher.runtime.distribution_runtime_states.insert(
            loader_tile,
            GameRuntimeDistributionBlockState::UnitCargoLoader(UnitCargoLoaderState {
                build_progress: 1.0 - 1.0 / loader_block.unit_build_time,
                ..UnitCargoLoaderState::default()
            }),
        );
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let report = launcher
            .last_runtime_item_transport_report
            .expect("server update should cache the latest item transport batch");
        assert_eq!(report.unit_cargo_loader_built_units, 1);
        let Some(GameRuntimeDistributionBlockState::UnitCargoLoader(state)) = launcher
            .runtime
            .distribution_runtime_states
            .get(&loader_tile)
        else {
            panic!("unit cargo loader state should remain present");
        };
        assert!(state.has_unit);
        assert_eq!(state.build_progress, 0.0);
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_broadcasts_unit_tether_block_spawned_for_owned_unit_cargo_loader() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6580).unwrap();

        let loader_def = launcher
            .content_loader
            .block_by_name("unit-cargo-loader")
            .unwrap();
        let power_source_def = launcher
            .content_loader
            .block_by_name("power-source")
            .unwrap();
        let BlockDef::Distribution(loader_block) = loader_def else {
            panic!("unit-cargo-loader should be a distribution block");
        };
        let nitrogen = launcher
            .content_loader
            .liquid_by_name("nitrogen")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let loader_tile = point2_pack(6, 6);
        let power_source_tile = point2_pack(8, 6);

        launcher.runtime.state.world.resize(16, 16);
        launcher.runtime.add_building(BuildingComp::new(
            loader_tile,
            loader_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.add_building(BuildingComp::new(
            power_source_tile,
            power_source_def.base().clone(),
            TeamId(6),
        ));
        if let Some(power) = launcher.runtime.buildings[0].power.as_mut() {
            power.status = 1.0;
        }
        if let Some(liquids) = launcher.runtime.buildings[0].liquids.as_mut() {
            liquids.set(nitrogen, 20.0);
        }
        launcher.runtime.distribution_runtime_states.insert(
            loader_tile,
            GameRuntimeDistributionBlockState::UnitCargoLoader(UnitCargoLoaderState {
                build_progress: 1.0 - 1.0 / loader_block.unit_build_time,
                ..UnitCargoLoaderState::default()
            }),
        );
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let Some(GameRuntimeDistributionBlockState::UnitCargoLoader(state)) = launcher
            .runtime
            .distribution_runtime_states
            .get(&loader_tile)
        else {
            panic!("unit cargo loader state should remain present");
        };
        let spawned_id = state.read_unit_id;
        assert!(spawned_id >= super::SERVER_RUNTIME_UNIT_ID_START);
        let spawned_unit = launcher
            .server_units
            .get(&spawned_id)
            .expect("server update should materialize spawned manifold unit");
        assert_eq!(spawned_unit.type_info.name(), "manifold");
        assert_eq!(spawned_unit.team_id(), TeamId(6));
        assert_eq!(spawned_unit.x(), launcher.runtime.buildings()[0].x);
        assert_eq!(spawned_unit.y(), launcher.runtime.buildings()[0].y);
        assert_eq!(spawned_unit.rotation(), 90.0);
        assert!(spawned_unit.controller.is_cargo());
        assert_eq!(
            spawned_unit
                .cargo_ai
                .as_ref()
                .and_then(|cargo| cargo.tether_tile_pos),
            Some(loader_tile)
        );
        let tether = spawned_unit
            .building_tether
            .as_ref()
            .expect("spawned cargo unit should have a formal building tether");
        assert_eq!(tether.team, TeamId(6));
        assert_eq!(
            tether.building,
            Some(BuildingTetherRef {
                tile_pos: loader_tile,
                team: TeamId(6),
                valid: true,
            })
        );
        assert_eq!(tether.update(), BuildingTetherAction::Keep);

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            *reliable
                && matches!(
                    packet,
                    PacketKind::UnitTetherBlockSpawnedCallPacket(packet)
                        if packet.tile == Some(loader_tile) && packet.id == spawned_id
                )
        }));
    }

    #[test]
    fn server_update_drives_spawned_unit_cargo_ai_between_loader_and_unload_point() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6581).unwrap();

        let loader_def = launcher
            .content_loader
            .block_by_name("unit-cargo-loader")
            .unwrap();
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
        let loader_tile = point2_pack(6, 6);
        let unload_tile = point2_pack(10, 6);

        launcher.runtime.state.world.resize(18, 12);
        let mut loader_building =
            BuildingComp::new(loader_tile, loader_def.base().clone(), TeamId(6));
        loader_building.items.as_mut().unwrap().add(copper, 12);
        launcher.runtime.add_building(loader_building);
        launcher.runtime.add_building(BuildingComp::new(
            unload_tile,
            unload_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.distribution_runtime_states.insert(
            loader_tile,
            GameRuntimeDistributionBlockState::UnitCargoLoader(UnitCargoLoaderState {
                read_unit_id: 7,
                has_unit: true,
                ..UnitCargoLoaderState::default()
            }),
        );
        launcher.runtime.distribution_runtime_states.insert(
            unload_tile,
            GameRuntimeDistributionBlockState::UnitCargoUnload(UnitCargoUnloadPointState {
                item_id: Some(copper as i32),
                stale_timer: 0.0,
                stale: false,
            }),
        );
        let mut cargo_unit = UnitComp::new(
            7,
            launcher
                .content_loader
                .unit_by_name("manifold")
                .unwrap()
                .clone(),
            TeamId(6),
        );
        cargo_unit.set_controller(UnitControllerState::Cargo);
        cargo_unit.set_pos(
            launcher.runtime.buildings()[0].x,
            launcher.runtime.buildings()[0].y,
        );
        launcher.server_units.insert(7, cargo_unit);
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        assert_eq!(
            launcher.runtime.buildings()[0]
                .items
                .as_ref()
                .unwrap()
                .get(copper),
            0
        );
        let unit = launcher.server_units.get(&7).unwrap();
        assert_eq!(unit.items.item(), Some("copper"));
        assert_eq!(unit.items.stack.amount, 12);
        assert_eq!(
            unit.cargo_ai
                .as_ref()
                .and_then(|cargo| cargo.unload_target_tile_pos),
            Some(unload_tile)
        );

        launcher.update();

        let unit = launcher.server_units.get(&7).unwrap();
        assert_eq!(unit.items.stack.amount, 0);
        assert_eq!(
            launcher.runtime.buildings()[1]
                .items
                .as_ref()
                .unwrap()
                .get(copper),
            12
        );

        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                    packet,
                    PacketKind::TakeItemsCallPacket(packet)
                        if packet.build == BuildingRef::new(loader_tile)
                            && packet.item.as_deref() == Some("copper")
                            && packet.amount == 12
                            && packet.to == UnitRef::Unit { id: 7 }
                )
        }));
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                    packet,
                    PacketKind::TransferItemToCallPacket(packet)
                        if packet.unit == UnitRef::Unit { id: 7 }
                            && packet.item.as_deref() == Some("copper")
                            && packet.amount == 12
                            && packet.build == BuildingRef::new(unload_tile)
                )
        }));
        let snapshot_packet = sent
            .iter()
            .rev()
            .find_map(|(_connection_id, packet, reliable)| {
                if !*reliable {
                    if let PacketKind::EntitySnapshotCallPacket(packet) = packet {
                        return Some(packet.clone());
                    }
                }
                None
            })
            .expect("server update should broadcast cargo unit entity snapshot");
        let mut client_runtime = GameRuntime::default();
        let report = client_runtime.apply_client_entity_snapshot_packet_with_content(
            &launcher.content_loader,
            snapshot_packet.amount,
            &snapshot_packet.data,
        );
        assert_eq!(report.entity_typed_records_applied, 1);
        let client_unit = client_runtime
            .client_unit_snapshot_entities
            .get(&7)
            .unwrap();
        let dx = client_unit.x() - launcher.runtime.buildings()[1].x;
        let dy = client_unit.y() - launcher.runtime.buildings()[1].y;
        assert_ne!(client_unit.x(), launcher.runtime.buildings()[1].x);
        assert!(
            (dx * dx + dy * dy).sqrt() <= CARGO_AI_TRANSFER_RANGE,
            "cargo unit snapshot should be within transfer range after moveTo"
        );
        assert_eq!(client_unit.items.stack.amount, 0);
    }

    #[test]
    fn server_entity_snapshot_packet_includes_runtime_puddles_for_client_sync() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.server_puddles = Puddles::new(32, 32);
        let neoplasm = launcher
            .content_loader
            .liquid_by_name("neoplasm")
            .expect("v158.1 content should include neoplasm");
        launcher.runtime.server_puddles.deposit_at(
            Some(PuddleTileView::new(10, 12)),
            PuddleLiquidInfo::from(neoplasm),
            18.0,
            PuddleDepositContext::default(),
        );
        let server_puddle = launcher
            .runtime
            .server_puddles
            .get(10, 12)
            .expect("server puddle should be registered")
            .clone();

        let packet = launcher
            .server_unit_entity_snapshot_packet()
            .expect("puddle snapshot should encode");

        assert_eq!(packet.amount, 1);
        let mut client_runtime = GameRuntime::default();
        let report = client_runtime.apply_client_entity_snapshot_packet_with_content(
            &launcher.content_loader,
            packet.amount,
            &packet.data,
        );
        assert_eq!(report.entity_parse_errors, 0);
        assert_eq!(report.entity_typed_records_applied, 1);
        let client_puddle = client_runtime
            .client_puddle_snapshot_entities
            .get(&server_puddle.id)
            .expect("client should materialize puddle entity from server snapshot");
        assert_eq!(client_puddle.amount, server_puddle.amount);
        assert_eq!(client_puddle.tile.unwrap().x, 10);
        assert_eq!(client_puddle.tile.unwrap().y, 12);
        assert_eq!(
            client_puddle
                .liquid
                .expect("client puddle should resolve neoplasm liquid")
                .cap_puddles,
            server_puddle.liquid.unwrap().cap_puddles
        );
    }

    #[test]
    fn server_entity_snapshot_packet_includes_runtime_fires_for_client_sync() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.server_fires = Fires::new(8, 8);
        assert_eq!(
            launcher.runtime.server_fires.create(
                Some(FireTile {
                    x: 1,
                    y: 2,
                    build_present: true,
                    flammability: 0.0,
                }),
                FireRules::default(),
            ),
            mindustry_core::mindustry::entities::FireCreateResult::Created
        );

        let packet = launcher
            .server_unit_entity_snapshot_packet()
            .expect("fire snapshot should encode");

        assert_eq!(packet.amount, 1);
        let mut client_runtime = GameRuntime::default();
        let report = client_runtime.apply_client_entity_snapshot_packet_with_content(
            &launcher.content_loader,
            packet.amount,
            &packet.data,
        );
        assert_eq!(report.entity_parse_errors, 0);
        assert_eq!(report.entity_typed_records_applied, 1);
        let client_fire = client_runtime
            .client_fire_snapshot_entities
            .get(&super::server_fire_entity_id(1, 2))
            .expect("client should materialize fire entity from server snapshot");
        assert_eq!(client_fire.tile.unwrap().x, 1);
        assert_eq!(client_fire.tile.unwrap().y, 2);
        assert_eq!(
            client_fire.lifetime,
            mindustry_core::mindustry::entities::BASE_FIRE_LIFETIME
        );
    }

    #[test]
    fn server_update_creates_fire_when_hot_puddle_touches_building() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6596).unwrap();
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(4, 4);
        let wall = launcher
            .content_loader
            .block_by_name("copper-wall")
            .unwrap()
            .base()
            .clone();
        launcher
            .runtime
            .add_building(BuildingComp::new(point2_pack(1, 1), wall, TeamId(1)));
        launcher.runtime.server_puddles = Puddles::new(4, 4);
        let slag = launcher.content_loader.liquid_by_name("slag").unwrap();
        launcher.runtime.server_puddles.deposit_at(
            Some(PuddleTileView::new(1, 1)),
            PuddleLiquidInfo::from(slag),
            40.0,
            PuddleDepositContext::default(),
        );

        launcher.update();

        assert!(launcher.runtime.server_fires.has(1, 1));
        let sent = sent.lock().unwrap();
        let snapshot = sent
            .iter()
            .rev()
            .find_map(|(_connection_id, packet, reliable)| {
                if !*reliable {
                    if let PacketKind::EntitySnapshotCallPacket(packet) = packet {
                        return Some(packet.clone());
                    }
                }
                None
            })
            .expect("puddle-created fire should be broadcast as entity snapshot");
        let mut client_runtime = GameRuntime::default();
        let report = client_runtime.apply_client_entity_snapshot_packet_with_content(
            &launcher.content_loader,
            snapshot.amount,
            &snapshot.data,
        );
        assert_eq!(report.entity_parse_errors, 0);
        assert!(
            client_runtime
                .client_fire_snapshot_entities
                .contains_key(&super::server_fire_entity_id(1, 1)),
            "client should receive the fire created from the hot puddle/building contact"
        );
    }

    #[test]
    fn server_update_applies_puddle_liquid_status_and_ripple_to_ground_units() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6597).unwrap();
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(4, 4);
        launcher.runtime.server_puddles = Puddles::new(4, 4);
        let water = launcher.content_loader.liquid_by_name("water").unwrap();
        let water_color = water.color_rgba;
        launcher.runtime.server_puddles.deposit_at(
            Some(PuddleTileView::new(1, 1)),
            PuddleLiquidInfo::from(water),
            40.0,
            PuddleDepositContext::default(),
        );

        let dagger = launcher
            .content_loader
            .unit_by_name("dagger")
            .unwrap()
            .clone();
        let mut unit = UnitComp::new(71, dagger, TeamId(1));
        unit.set_pos(8.0, 8.0);
        unit.vel.vel.x = 1.0;
        launcher.server_units.insert(unit.id(), unit);

        launcher.update();

        let unit = launcher.server_units.get(&71).unwrap();
        assert_eq!(unit.status.get_duration("wet"), 60.0 * 2.0);
        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                    packet,
                    PacketKind::EffectCallPacket(packet)
                        if packet.effect_id == standard_effect_id("ripple").unwrap() as u16
                            && packet.x == 8.0
                            && packet.y == 8.0
                            && packet.rotation == unit.type_info.ripple_scale
                            && packet.color.rgba() == water_color as i32
                )
        }));
    }

    #[test]
    fn server_puddle_cell_liquid_update_absorbs_spread_target_from_neighbor_building() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.world.resize(4, 4);
        launcher.runtime.server_puddles = Puddles::new(4, 4);
        let router = launcher
            .content_loader
            .block_by_name("liquid-router")
            .unwrap()
            .base()
            .clone();
        let neighbor_tile = point2_pack(2, 1);
        launcher
            .runtime
            .add_building(BuildingComp::new(neighbor_tile, router, TeamId(1)));
        let water_id = launcher
            .content_loader
            .liquid_by_name("water")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        launcher
            .runtime
            .buildings
            .iter_mut()
            .find(|building| building.tile_pos == neighbor_tile)
            .unwrap()
            .liquids
            .as_mut()
            .unwrap()
            .add(water_id, 10.0);
        let neoplasm = launcher.content_loader.liquid_by_name("neoplasm").unwrap();
        let neoplasm_info = PuddleLiquidInfo::from(neoplasm);
        assert_eq!(neoplasm_info.reaction_target.as_deref(), Some("water"));
        launcher.runtime.server_puddles.deposit_at(
            Some(PuddleTileView::new(1, 1)),
            neoplasm_info,
            70.0,
            PuddleDepositContext::default(),
        );

        launcher.tick_server_puddles(1.0).unwrap();

        let neighbor = launcher
            .runtime
            .buildings
            .iter()
            .find(|building| building.tile_pos == neighbor_tile)
            .unwrap();
        assert!(neighbor.liquids.as_ref().unwrap().get(water_id) < 10.0);
        let neoplasm_puddle = launcher
            .runtime
            .server_puddles
            .get_entry(2, 1)
            .expect("CellLiquid.update should deposit neoplasm onto target-liquid building");
        assert_eq!(neoplasm_puddle.liquid.name, "neoplasm");
        assert!(
            neoplasm_puddle.puddle.amount + neoplasm_puddle.puddle.accepting > 1.1,
            "absorbed water should be converted by CellLiquid spreadConversion and accepted by the target puddle"
        );
    }

    #[test]
    fn server_puddle_cell_liquid_update_absorbs_neighbor_target_puddle_and_hides_removed_id() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6598).unwrap();
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.set_sector(Some(Sector::new(42)));
        launcher.runtime.state.world.resize(4, 4);
        launcher.runtime.server_puddles = Puddles::new(4, 4);
        let neoplasm = launcher.content_loader.liquid_by_name("neoplasm").unwrap();
        let water = launcher.content_loader.liquid_by_name("water").unwrap();
        let neoplasm_info = PuddleLiquidInfo::from(neoplasm);
        assert_eq!(neoplasm_info.reaction_target.as_deref(), Some("water"));
        launcher.runtime.server_puddles.deposit_at(
            Some(PuddleTileView::new(1, 1)),
            neoplasm_info,
            70.0,
            PuddleDepositContext::default(),
        );
        launcher.runtime.server_puddles.deposit_at(
            Some(PuddleTileView::new(2, 1)),
            PuddleLiquidInfo::from(water),
            20.0,
            PuddleDepositContext::default(),
        );
        let water_puddle_id = launcher.runtime.server_puddles.get(2, 1).unwrap().id;

        launcher.tick_server_puddles(1.0).unwrap();

        let source = launcher.runtime.server_puddles.get_entry(1, 1).unwrap();
        assert_eq!(source.liquid.name, "neoplasm");
        assert!(
            source.puddle.amount > 70.0,
            "CellLiquid.update should add absorbed neighbor puddle amount to the source puddle"
        );
        let replacement = launcher
            .runtime
            .server_puddles
            .get_entry(2, 1)
            .expect("low-residue target puddle should be replaced by neoplasm");
        assert_eq!(replacement.liquid.name, "neoplasm");
        assert!(
            replacement.puddle.amount
                >= mindustry_core::mindustry::entities::puddles::MAX_LIQUID / 3.0,
            "replacement deposit should use at least maxLiquid / 3 like Java CellLiquid.update"
        );
        assert!(launcher
            .runtime
            .trigger_events
            .iter()
            .any(|event| { event.trigger == Trigger::NeoplasmReact && event.campaign }));
        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                    packet,
                    PacketKind::HiddenSnapshotCallPacket(packet)
                        if packet.ids.contains(&water_puddle_id)
                )
        }));
    }

    #[test]
    fn server_puddle_cell_liquid_building_deposit_precedes_neighbor_absorb_like_java() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(4, 4);
        launcher.runtime.server_puddles = Puddles::new(4, 4);
        let router = launcher
            .content_loader
            .block_by_name("liquid-router")
            .unwrap()
            .base()
            .clone();
        let neighbor_tile = point2_pack(2, 1);
        launcher
            .runtime
            .add_building(BuildingComp::new(neighbor_tile, router, TeamId(1)));
        let water_id = launcher
            .content_loader
            .liquid_by_name("water")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        launcher
            .runtime
            .buildings
            .iter_mut()
            .find(|building| building.tile_pos == neighbor_tile)
            .unwrap()
            .liquids
            .as_mut()
            .unwrap()
            .add(water_id, 10.0);
        let neoplasm = launcher.content_loader.liquid_by_name("neoplasm").unwrap();
        let water = launcher.content_loader.liquid_by_name("water").unwrap();
        launcher.runtime.server_puddles.deposit_at(
            Some(PuddleTileView::new(1, 1)),
            PuddleLiquidInfo::from(neoplasm),
            70.0,
            PuddleDepositContext::default(),
        );
        launcher.runtime.server_puddles.deposit_at(
            Some(PuddleTileView::new(2, 1)),
            PuddleLiquidInfo::from(water),
            20.0,
            PuddleDepositContext::default(),
        );

        launcher.tick_server_puddles(1.0).unwrap();

        let replacement = launcher
            .runtime
            .server_puddles
            .get_entry(2, 1)
            .expect("low-residue target puddle should still be replaced by neoplasm");
        assert_eq!(replacement.liquid.name, "neoplasm");
        assert!(
            replacement.puddle.accepting.abs() <= f32::EPSILON,
            "Java applies building conversion deposit before nearby puddle absorption; neoplasm poured onto an existing water puddle reacts through water and must not become same-liquid accepting after replacement"
        );
        assert!(
            (replacement.puddle.amount
                - mindustry_core::mindustry::entities::puddles::MAX_LIQUID / 3.0)
                .abs()
                < 0.001,
            "replacement amount should come only from CellLiquid nearby-puddle replacement, not from a delayed building deposit"
        );
    }

    #[test]
    fn server_puddle_cell_liquid_update_damages_target_liquid_building_and_reaccepts_spread() {
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.world.resize(4, 4);
        launcher.runtime.server_puddles = Puddles::new(4, 4);
        let router = launcher
            .content_loader
            .block_by_name("liquid-router")
            .unwrap()
            .base()
            .clone();
        let tile_pos = point2_pack(1, 1);
        launcher
            .runtime
            .add_building(BuildingComp::new(tile_pos, router, TeamId(1)));
        let water_id = launcher
            .content_loader
            .liquid_by_name("water")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let building = launcher
            .runtime
            .buildings
            .iter_mut()
            .find(|building| building.tile_pos == tile_pos)
            .unwrap();
        let health_before = building.health;
        building.liquids.as_mut().unwrap().add(water_id, 10.0);
        let neoplasm = launcher.content_loader.liquid_by_name("neoplasm").unwrap();
        launcher.runtime.server_puddles.deposit_at(
            Some(PuddleTileView::new(1, 1).with_build(1)),
            PuddleLiquidInfo::from(neoplasm),
            70.0,
            PuddleDepositContext::default(),
        );

        launcher.tick_server_puddles(1.0).unwrap();

        let building = launcher
            .runtime
            .buildings
            .iter()
            .find(|building| building.tile_pos == tile_pos)
            .unwrap();
        assert!(
            building.health < health_before,
            "CellLiquid.update should damage the building underneath when it contains spreadTarget"
        );
        assert!(
            building.liquids.as_ref().unwrap().get(water_id) < 10.0,
            "Geometry.d4c includes the center tile, so CellLiquid.update should remove a scaled portion of current-building spreadTarget before the damage branch"
        );
        let source = launcher.runtime.server_puddles.get(1, 1).unwrap();
        assert!(
            source.accepting >= 0.85,
            "CellLiquid.update should re-deposit converted center absorption and amountSpread onto the source puddle through accepting"
        );
    }

    #[test]
    fn server_update_unit_cargo_pickup_prefers_non_stale_later_item_target() {
        let mut launcher = ServerLauncher::new(Vec::new());

        let loader_def = launcher
            .content_loader
            .block_by_name("unit-cargo-loader")
            .unwrap();
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
        let lead = launcher
            .content_loader
            .item_by_name("lead")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let loader_tile = point2_pack(6, 6);
        let stale_copper_tile = point2_pack(10, 6);
        let fresh_lead_tile = point2_pack(12, 6);

        launcher.runtime.state.world.resize(18, 12);
        let mut loader_building =
            BuildingComp::new(loader_tile, loader_def.base().clone(), TeamId(6));
        loader_building.items.as_mut().unwrap().add(copper, 12);
        loader_building.items.as_mut().unwrap().add(lead, 8);
        launcher.runtime.add_building(loader_building);
        let mut stale_copper_unload =
            BuildingComp::new(stale_copper_tile, unload_def.base().clone(), TeamId(6));
        stale_copper_unload
            .items
            .as_mut()
            .unwrap()
            .add(copper, stale_copper_unload.block.item_capacity);
        launcher.runtime.add_building(stale_copper_unload);
        launcher.runtime.add_building(BuildingComp::new(
            fresh_lead_tile,
            unload_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.distribution_runtime_states.insert(
            loader_tile,
            GameRuntimeDistributionBlockState::UnitCargoLoader(UnitCargoLoaderState {
                read_unit_id: 7,
                has_unit: true,
                ..UnitCargoLoaderState::default()
            }),
        );
        launcher.runtime.distribution_runtime_states.insert(
            stale_copper_tile,
            GameRuntimeDistributionBlockState::UnitCargoUnload(UnitCargoUnloadPointState {
                item_id: Some(copper as i32),
                stale_timer: 360.0,
                stale: true,
            }),
        );
        launcher.runtime.distribution_runtime_states.insert(
            fresh_lead_tile,
            GameRuntimeDistributionBlockState::UnitCargoUnload(UnitCargoUnloadPointState {
                item_id: Some(lead as i32),
                stale_timer: 0.0,
                stale: false,
            }),
        );
        let mut cargo_unit = UnitComp::new(
            7,
            launcher
                .content_loader
                .unit_by_name("manifold")
                .unwrap()
                .clone(),
            TeamId(6),
        );
        cargo_unit.set_controller(UnitControllerState::Cargo);
        cargo_unit.set_pos(
            launcher.runtime.buildings()[0].x,
            launcher.runtime.buildings()[0].y,
        );
        launcher.server_units.insert(7, cargo_unit);
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let unit = launcher.server_units.get(&7).unwrap();
        assert_eq!(unit.items.item(), Some("lead"));
        assert_eq!(unit.items.stack.amount, 8);
        assert_eq!(
            unit.cargo_ai
                .as_ref()
                .and_then(|cargo| cargo.unload_target_tile_pos),
            Some(fresh_lead_tile)
        );
        assert_eq!(
            launcher.runtime.buildings()[0]
                .items
                .as_ref()
                .unwrap()
                .get(copper),
            12
        );
        assert_eq!(
            launcher.runtime.buildings()[0]
                .items
                .as_ref()
                .unwrap()
                .get(lead),
            0
        );
    }

    #[test]
    fn server_update_unit_cargo_pickup_respects_retarget_interval() {
        let mut launcher = ServerLauncher::new(Vec::new());

        let loader_def = launcher
            .content_loader
            .block_by_name("unit-cargo-loader")
            .unwrap();
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
        let loader_tile = point2_pack(6, 6);
        let unload_tile = point2_pack(10, 6);

        launcher.runtime.state.world.resize(18, 12);
        let mut loader_building =
            BuildingComp::new(loader_tile, loader_def.base().clone(), TeamId(6));
        loader_building.items.as_mut().unwrap().add(copper, 12);
        launcher.runtime.add_building(loader_building);
        launcher.runtime.add_building(BuildingComp::new(
            unload_tile,
            unload_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.distribution_runtime_states.insert(
            loader_tile,
            GameRuntimeDistributionBlockState::UnitCargoLoader(UnitCargoLoaderState {
                read_unit_id: 7,
                has_unit: true,
                ..UnitCargoLoaderState::default()
            }),
        );
        launcher.runtime.distribution_runtime_states.insert(
            unload_tile,
            GameRuntimeDistributionBlockState::UnitCargoUnload(UnitCargoUnloadPointState {
                item_id: Some(copper as i32),
                stale_timer: 0.0,
                stale: false,
            }),
        );
        let mut cargo_unit = UnitComp::new(
            7,
            launcher
                .content_loader
                .unit_by_name("manifold")
                .unwrap()
                .clone(),
            TeamId(6),
        );
        cargo_unit.set_controller(UnitControllerState::Cargo);
        cargo_unit.set_pos(
            launcher.runtime.buildings()[0].x,
            launcher.runtime.buildings()[0].y,
        );
        cargo_unit.cargo_ai = Some(CargoAiRuntimeState {
            tether_tile_pos: Some(loader_tile),
            unload_target_tile_pos: None,
            item_target: None,
            no_dest_timer: 0.0,
            drop_timer: CARGO_AI_DROP_SPACING,
            retarget_timer: 0.0,
            target_index: 0,
        });
        launcher.server_units.insert(7, cargo_unit);
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();
        assert!(!launcher.server_units.get(&7).unwrap().items.has_item());
        assert_eq!(
            launcher
                .server_units
                .get(&7)
                .unwrap()
                .cargo_ai
                .as_ref()
                .unwrap()
                .retarget_timer,
            1.0
        );

        for _ in 0..CARGO_AI_RETARGET_INTERVAL as usize {
            launcher.update();
            if launcher.server_units.get(&7).unwrap().items.has_item() {
                break;
            }
        }

        let unit = launcher.server_units.get(&7).unwrap();
        assert_eq!(unit.items.item(), Some("copper"));
        assert_eq!(unit.items.stack.amount, 12);
        assert_eq!(
            unit.cargo_ai
                .as_ref()
                .and_then(|cargo| cargo.unload_target_tile_pos),
            Some(unload_tile)
        );
    }

    #[test]
    fn server_update_keeps_unit_cargo_item_when_unload_target_is_reconfigured() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6583).unwrap();

        let loader_def = launcher
            .content_loader
            .block_by_name("unit-cargo-loader")
            .unwrap();
        let unload_def = launcher
            .content_loader
            .block_by_name("unit-cargo-unload-point")
            .unwrap();
        let loader_tile = point2_pack(6, 6);
        let unload_tile = point2_pack(10, 6);

        launcher.runtime.state.world.resize(18, 12);
        launcher.runtime.add_building(BuildingComp::new(
            loader_tile,
            loader_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.add_building(BuildingComp::new(
            unload_tile,
            unload_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.distribution_runtime_states.insert(
            loader_tile,
            GameRuntimeDistributionBlockState::UnitCargoLoader(UnitCargoLoaderState {
                read_unit_id: 7,
                has_unit: true,
                ..UnitCargoLoaderState::default()
            }),
        );
        launcher.runtime.distribution_runtime_states.insert(
            unload_tile,
            GameRuntimeDistributionBlockState::UnitCargoUnload(UnitCargoUnloadPointState {
                item_id: None,
                stale_timer: 0.0,
                stale: false,
            }),
        );
        let mut cargo_unit = UnitComp::new(
            7,
            launcher
                .content_loader
                .unit_by_name("manifold")
                .unwrap()
                .clone(),
            TeamId(6),
        );
        cargo_unit.set_controller(UnitControllerState::Cargo);
        cargo_unit.items.add_item_amount("copper", 5);
        cargo_unit.cargo_ai = Some(CargoAiRuntimeState {
            tether_tile_pos: Some(loader_tile),
            unload_target_tile_pos: Some(unload_tile),
            item_target: Some("copper".into()),
            no_dest_timer: 0.0,
            drop_timer: CARGO_AI_DROP_SPACING,
            retarget_timer: CARGO_AI_RETARGET_INTERVAL,
            target_index: 0,
        });
        launcher.server_units.insert(7, cargo_unit);
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let unit = launcher.server_units.get(&7).unwrap();
        assert_eq!(unit.items.item(), Some("copper"));
        assert_eq!(unit.items.stack.amount, 5);
        assert_eq!(
            unit.cargo_ai
                .as_ref()
                .and_then(|cargo| cargo.unload_target_tile_pos),
            None
        );
        let sent = sent.lock().unwrap();
        assert!(sent.iter().all(|(_connection_id, packet, _reliable)| {
            !matches!(packet, PacketKind::TransferItemToCallPacket(_))
        }));
    }

    #[test]
    fn server_update_switches_full_unit_cargo_unload_target_after_empty_wait_time() {
        let mut launcher = ServerLauncher::new(Vec::new());

        let loader_def = launcher
            .content_loader
            .block_by_name("unit-cargo-loader")
            .unwrap();
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
        let loader_tile = point2_pack(6, 6);
        let full_unload_tile = point2_pack(10, 6);
        let next_unload_tile = point2_pack(12, 6);

        launcher.runtime.state.world.resize(18, 12);
        launcher.runtime.add_building(BuildingComp::new(
            loader_tile,
            loader_def.base().clone(),
            TeamId(6),
        ));
        let mut full_unload =
            BuildingComp::new(full_unload_tile, unload_def.base().clone(), TeamId(6));
        let full_capacity = full_unload.block.item_capacity;
        full_unload
            .items
            .as_mut()
            .unwrap()
            .add(copper, full_capacity);
        launcher.runtime.add_building(full_unload);
        launcher.runtime.add_building(BuildingComp::new(
            next_unload_tile,
            unload_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.distribution_runtime_states.insert(
            loader_tile,
            GameRuntimeDistributionBlockState::UnitCargoLoader(UnitCargoLoaderState {
                read_unit_id: 7,
                has_unit: true,
                ..UnitCargoLoaderState::default()
            }),
        );
        for tile in [full_unload_tile, next_unload_tile] {
            launcher.runtime.distribution_runtime_states.insert(
                tile,
                GameRuntimeDistributionBlockState::UnitCargoUnload(UnitCargoUnloadPointState {
                    item_id: Some(copper as i32),
                    stale_timer: 0.0,
                    stale: false,
                }),
            );
        }
        let mut cargo_unit = UnitComp::new(
            7,
            launcher
                .content_loader
                .unit_by_name("manifold")
                .unwrap()
                .clone(),
            TeamId(6),
        );
        cargo_unit.set_controller(UnitControllerState::Cargo);
        cargo_unit.set_pos(
            launcher.runtime.buildings()[1].x,
            launcher.runtime.buildings()[1].y,
        );
        cargo_unit.items.add_item_amount("copper", 12);
        cargo_unit.cargo_ai = Some(CargoAiRuntimeState {
            tether_tile_pos: Some(loader_tile),
            unload_target_tile_pos: Some(full_unload_tile),
            item_target: Some("copper".into()),
            no_dest_timer: 0.0,
            drop_timer: CARGO_AI_DROP_SPACING,
            retarget_timer: CARGO_AI_RETARGET_INTERVAL,
            target_index: 0,
        });
        launcher.server_units.insert(7, cargo_unit);
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();
        assert_eq!(
            launcher
                .server_units
                .get(&7)
                .unwrap()
                .cargo_ai
                .as_ref()
                .unwrap()
                .no_dest_timer,
            CARGO_AI_DROP_SPACING
        );

        for _ in 0..CARGO_AI_DROP_SPACING as usize {
            launcher.update();
        }

        let unit = launcher.server_units.get(&7).unwrap();
        assert_eq!(unit.items.item(), Some("copper"));
        assert_eq!(unit.items.stack.amount, 12);
        assert_eq!(
            unit.cargo_ai
                .as_ref()
                .and_then(|cargo| cargo.unload_target_tile_pos),
            Some(next_unload_tile)
        );
        assert_eq!(
            launcher.runtime.buildings()[2]
                .items
                .as_ref()
                .unwrap()
                .get(copper),
            0
        );
    }

    #[test]
    fn server_update_despawns_tethered_unit_when_cargo_loader_is_missing() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6582).unwrap();

        let loader_tile = point2_pack(6, 6);
        launcher.runtime.distribution_runtime_states.insert(
            loader_tile,
            GameRuntimeDistributionBlockState::UnitCargoLoader(UnitCargoLoaderState {
                read_unit_id: 7,
                has_unit: true,
                ..UnitCargoLoaderState::default()
            }),
        );
        let mut cargo_unit = UnitComp::new(
            7,
            launcher
                .content_loader
                .unit_by_name("manifold")
                .unwrap()
                .clone(),
            TeamId(6),
        );
        cargo_unit.set_controller(UnitControllerState::Cargo);
        launcher.server_units.insert(7, cargo_unit);
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        assert!(!launcher.server_units.contains_key(&7));
        let Some(GameRuntimeDistributionBlockState::UnitCargoLoader(state)) = launcher
            .runtime
            .distribution_runtime_states
            .get(&loader_tile)
        else {
            panic!("loader runtime state should remain present");
        };
        assert!(!state.has_unit);
        assert_eq!(state.read_unit_id, -1);
        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                    packet,
                    PacketKind::UnitDespawnCallPacket(packet)
                        if packet.unit == UnitRef::Unit { id: 7 }
                )
        }));
    }

    #[test]
    fn server_update_despawns_tethered_unit_when_cargo_loader_team_changes() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let mut launcher = ServerLauncher::new(Vec::new());
        launcher.net_server = NetServer::new(Net::new(Box::new(provider)));
        launcher.net_server.open(6584).unwrap();

        let loader_def = launcher
            .content_loader
            .block_by_name("unit-cargo-loader")
            .unwrap();
        let loader_tile = point2_pack(6, 6);
        launcher.runtime.state.world.resize(12, 12);
        launcher.runtime.add_building(BuildingComp::new(
            loader_tile,
            loader_def.base().clone(),
            TeamId(7),
        ));
        launcher.runtime.distribution_runtime_states.insert(
            loader_tile,
            GameRuntimeDistributionBlockState::UnitCargoLoader(UnitCargoLoaderState {
                read_unit_id: 7,
                has_unit: true,
                ..UnitCargoLoaderState::default()
            }),
        );
        let mut cargo_unit = UnitComp::new(
            7,
            launcher
                .content_loader
                .unit_by_name("manifold")
                .unwrap()
                .clone(),
            TeamId(6),
        );
        cargo_unit.set_controller(UnitControllerState::Cargo);
        cargo_unit.cargo_ai = Some(CargoAiRuntimeState::new(Some(loader_tile)));
        launcher.server_units.insert(7, cargo_unit);
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        assert!(!launcher.server_units.contains_key(&7));
        let Some(GameRuntimeDistributionBlockState::UnitCargoLoader(state)) = launcher
            .runtime
            .distribution_runtime_states
            .get(&loader_tile)
        else {
            panic!("loader runtime state should remain present");
        };
        assert!(!state.has_unit);
        assert_eq!(state.read_unit_id, -1);
        let sent = sent.lock().unwrap();
        assert!(sent.iter().any(|(_connection_id, packet, reliable)| {
            !*reliable
                && matches!(
                    packet,
                    PacketKind::UnitDespawnCallPacket(packet)
                        if packet.unit == UnitRef::Unit { id: 7 }
                )
        }));
    }

    #[test]
    fn server_update_drives_owned_unit_cargo_unload_stale_from_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
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
        let BlockDef::Distribution(unload_block) = unload_def else {
            panic!("unit-cargo-unload-point should be a distribution block");
        };
        let unload_tile = point2_pack(7, 6);
        let mut unload = BuildingComp::new(unload_tile, unload_def.base().clone(), TeamId(6));
        unload
            .items
            .as_mut()
            .unwrap()
            .set(copper, unload_def.base().item_capacity);

        launcher.runtime.state.world.resize(16, 16);
        launcher.runtime.add_building(unload);
        launcher.runtime.distribution_runtime_states.insert(
            unload_tile,
            GameRuntimeDistributionBlockState::UnitCargoUnload(UnitCargoUnloadPointState {
                item_id: Some(copper as i32),
                stale_timer: unload_block.stale_time_duration - 1.0,
                stale: false,
            }),
        );
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let report = launcher
            .last_runtime_item_transport_report
            .expect("server update should cache the latest item transport batch");
        assert_eq!(report.unit_cargo_unload_stale_points, 1);
        let Some(GameRuntimeDistributionBlockState::UnitCargoUnload(state)) = launcher
            .runtime
            .distribution_runtime_states
            .get(&unload_tile)
        else {
            panic!("unit cargo unload state should remain present");
        };
        assert!(state.stale);
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_drives_owned_unit_cargo_unload_dump_from_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let unload_def = launcher
            .content_loader
            .block_by_name("unit-cargo-unload-point")
            .unwrap();
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        let copper = launcher
            .content_loader
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let BlockDef::Distribution(unload_block) = unload_def else {
            panic!("unit-cargo-unload-point should be a distribution block");
        };
        let unload_capacity = unload_def.base().item_capacity;
        let unload_tile = point2_pack(7, 6);
        let router_tile = point2_pack(9, 6);
        let mut unload = BuildingComp::new(unload_tile, unload_def.base().clone(), TeamId(6));
        unload.items.as_mut().unwrap().set(copper, unload_capacity);

        launcher.runtime.state.world.resize(16, 16);
        launcher.runtime.add_building(unload);
        launcher.runtime.add_building(BuildingComp::new(
            router_tile,
            router_def.base().clone(),
            TeamId(6),
        ));
        assert!(
            launcher.runtime.buildings()[0]
                .proximity
                .iter()
                .any(|reference| reference.tile_pos == router_tile),
            "unit cargo unload point should see the adjacent router"
        );
        launcher.runtime.distribution_runtime_states.insert(
            unload_tile,
            GameRuntimeDistributionBlockState::UnitCargoUnload(UnitCargoUnloadPointState {
                item_id: Some(copper as i32),
                stale_timer: unload_block.stale_time_duration - 1.0,
                stale: false,
            }),
        );
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let report = launcher
            .last_runtime_item_transport_report
            .expect("server update should cache the latest item transport batch");
        assert_eq!(report.unit_cargo_unload_dumped_items, 1);
        assert_eq!(report.unit_cargo_unload_stale_points, 0);
        let unload = launcher
            .runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == unload_tile)
            .unwrap();
        let router = launcher
            .runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == router_tile)
            .unwrap();
        assert_eq!(
            unload.items.as_ref().unwrap().get(copper),
            unload_capacity - 1
        );
        assert_eq!(router.items.as_ref().unwrap().get(copper), 1);
        let Some(GameRuntimeDistributionBlockState::UnitCargoUnload(state)) = launcher
            .runtime
            .distribution_runtime_states
            .get(&unload_tile)
        else {
            panic!("unit cargo unload state should remain present");
        };
        assert!(!state.stale);
        assert_eq!(state.stale_timer, 0.0);
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_drives_owned_payload_void_from_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let void_def = launcher
            .content_loader
            .block_by_name("payload-void")
            .unwrap();
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        let void_tile = point2_pack(4, 4);
        let mut build_bytes = Vec::new();
        BuildingComp::new(point2_pack(0, 0), router_def.base().clone(), TeamId(6))
            .write_base(&mut build_bytes, false)
            .unwrap();

        launcher.runtime.state.world.resize(10, 10);
        launcher.runtime.add_building(BuildingComp::new(
            void_tile,
            void_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.payload_runtime_states.insert(
            void_tile,
            GameRuntimePayloadBlockState::Void(PayloadBlockBuildState {
                payload: Some(PayloadRef::Block {
                    block: router_def.base().id,
                    version: 0,
                    build_bytes,
                }),
                ..PayloadBlockBuildState::default()
            }),
        );
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let report = launcher
            .last_runtime_payload_report
            .expect("server update should cache the latest payload batch");
        assert_eq!(report.void.incinerated_payloads, 1);
        let Some(GameRuntimePayloadBlockState::Void(common)) =
            launcher.runtime.payload_runtime_states.get(&void_tile)
        else {
            panic!("payload void sidecar should remain present");
        };
        assert!(common.payload.is_none());
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_drives_owned_payload_source_from_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let source_def = launcher
            .content_loader
            .block_by_name("payload-source")
            .unwrap();
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        let router_id = router_def.base().id;
        let source_tile = point2_pack(4, 4);
        let mut source_building =
            BuildingComp::new(source_tile, source_def.base().clone(), TeamId(6));
        source_building.set_rotation(1);

        launcher.runtime.state.world.resize(10, 10);
        launcher.runtime.add_building(source_building);
        launcher.runtime.payload_runtime_states.insert(
            source_tile,
            GameRuntimePayloadBlockState::Source {
                common: PayloadBlockBuildState::default(),
                source: PayloadSourceState {
                    config_block: Some(router_id),
                    ..PayloadSourceState::default()
                },
            },
        );
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let report = launcher
            .last_runtime_payload_report
            .expect("server update should cache the latest payload batch");
        assert_eq!(report.source.spawned_block_payloads, 1);
        assert_eq!(report.source.moved_out_payloads, 1);
        let Some(GameRuntimePayloadBlockState::Source { common, source }) =
            launcher.runtime.payload_runtime_states.get(&source_tile)
        else {
            panic!("payload source sidecar should remain present");
        };
        assert!(source.has_payload);
        assert!(matches!(
            common.payload.as_ref(),
            Some(PayloadRef::Block { block, .. }) if *block == router_id
        ));
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_drives_owned_payload_conveyor_from_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let conveyor_def = launcher
            .content_loader
            .block_by_name("payload-conveyor")
            .unwrap();
        let void_def = launcher
            .content_loader
            .block_by_name("payload-void")
            .unwrap();
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        let conveyor_block = conveyor_def.base().clone();
        let void_block = void_def.base().clone();
        let router_id = router_def.base().id;
        let move_time = match conveyor_def {
            BlockDef::Payload(payload) => payload.move_time,
            _ => unreachable!(),
        };
        let conveyor_tile = point2_pack(4, 4);
        let trns = conveyor_block.size / 2 + 1;
        let void_tile = point2_pack(4 + trns + (void_block.size - 1) / 2, 4);
        let mut conveyor_building = BuildingComp::new(conveyor_tile, conveyor_block, TeamId(6));
        conveyor_building.set_rotation(0);
        let mut build_bytes = Vec::new();
        BuildingComp::new(point2_pack(0, 0), router_def.base().clone(), TeamId(6))
            .write_base(&mut build_bytes, false)
            .unwrap();

        launcher.runtime.state.world.resize(12, 9);
        launcher.runtime.add_building(conveyor_building);
        launcher
            .runtime
            .add_building(BuildingComp::new(void_tile, void_block, TeamId(6)));
        launcher.runtime.payload_runtime_states.insert(
            conveyor_tile,
            GameRuntimePayloadBlockState::Conveyor(PayloadConveyorState {
                item: Some(PayloadRef::Block {
                    block: router_id,
                    version: 0,
                    build_bytes,
                }),
                step: 0,
                step_accepted: 0,
                ..PayloadConveyorState::default()
            }),
        );
        launcher.runtime.payload_runtime_states.insert(
            void_tile,
            GameRuntimePayloadBlockState::Void(PayloadBlockBuildState::default()),
        );
        launcher.runtime.state.set(GameStateState::Playing);
        launcher.runtime.state.tick = move_time as f64 - 1.0;

        launcher.update();

        let report = launcher
            .last_runtime_payload_report
            .expect("server update should cache the latest payload batch");
        assert_eq!(report.conveyor.attempted_moves, 1);
        assert_eq!(report.conveyor.transferred_payloads, 1);
        let Some(GameRuntimePayloadBlockState::Conveyor(conveyor)) =
            launcher.runtime.payload_runtime_states.get(&conveyor_tile)
        else {
            panic!("payload conveyor sidecar should remain present");
        };
        assert!(conveyor.item.is_none());
        let Some(GameRuntimePayloadBlockState::Void(common)) =
            launcher.runtime.payload_runtime_states.get(&void_tile)
        else {
            panic!("payload void sidecar should remain present");
        };
        assert!(matches!(
            common.payload.as_ref(),
            Some(PayloadRef::Block { block, .. }) if *block == router_id
        ));
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_drives_owned_payload_constructor_from_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let constructor_def = launcher
            .content_loader
            .block_by_name("constructor")
            .unwrap();
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        let router_id = router_def.base().id;
        let constructor_tile = point2_pack(4, 4);
        let mut constructor_building =
            BuildingComp::new(constructor_tile, constructor_def.base().clone(), TeamId(6));
        for requirement in router_def.requirements() {
            constructor_building
                .items
                .as_mut()
                .unwrap()
                .set(requirement.item, requirement.amount);
        }

        launcher.runtime.state.world.resize(10, 10);
        launcher.runtime.add_building(constructor_building);
        launcher.runtime.payload_runtime_states.insert(
            constructor_tile,
            GameRuntimePayloadBlockState::Constructor {
                common: PayloadBlockBuildState::default(),
                producer: BlockProducerState {
                    progress: 9.5,
                    ..BlockProducerState::default()
                },
                recipe: Some(router_id),
            },
        );
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let report = launcher
            .last_runtime_payload_report
            .expect("server update should cache the latest payload batch");
        assert_eq!(report.constructor.produced_payloads, 1);
        assert_eq!(report.constructor.moved_out_payloads, 1);
        let Some(GameRuntimePayloadBlockState::Constructor {
            common, producer, ..
        }) = launcher
            .runtime
            .payload_runtime_states
            .get(&constructor_tile)
        else {
            panic!("payload constructor sidecar should remain present");
        };
        assert!(producer.has_payload);
        assert!(matches!(
            common.payload.as_ref(),
            Some(PayloadRef::Block { block, .. }) if *block == router_id
        ));
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_drives_owned_payload_deconstructor_from_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let deconstructor_def = launcher
            .content_loader
            .block_by_name("small-deconstructor")
            .unwrap();
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        let router_id = router_def.base().id;
        let deconstructor_tile = point2_pack(4, 4);
        let mut build_bytes = Vec::new();
        BuildingComp::new(point2_pack(0, 0), router_def.base().clone(), TeamId(6))
            .write_base(&mut build_bytes, false)
            .unwrap();

        launcher.runtime.state.world.resize(10, 10);
        launcher.runtime.add_building(BuildingComp::new(
            deconstructor_tile,
            deconstructor_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.payload_runtime_states.insert(
            deconstructor_tile,
            GameRuntimePayloadBlockState::Deconstructor {
                common: PayloadBlockBuildState {
                    payload: Some(PayloadRef::Block {
                        block: router_id,
                        version: 0,
                        build_bytes,
                    }),
                    ..PayloadBlockBuildState::default()
                },
                deconstructor: PayloadDeconstructorState::default(),
            },
        );
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let report = launcher
            .last_runtime_payload_report
            .expect("server update should cache the latest payload batch");
        assert_eq!(report.deconstructor.moved_in_payloads, 1);
        assert_eq!(report.deconstructor.started_deconstructions, 1);
        let Some(GameRuntimePayloadBlockState::Deconstructor {
            common,
            deconstructor,
        }) = launcher
            .runtime
            .payload_runtime_states
            .get(&deconstructor_tile)
        else {
            panic!("payload deconstructor sidecar should remain present");
        };
        assert!(common.payload.is_none());
        assert!(deconstructor.has_deconstructing);
        assert!(matches!(
            deconstructor.deconstructing.as_ref(),
            Some(PayloadRef::Block { block, .. }) if *block == router_id
        ));
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_drives_owned_payload_loader_from_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let loader_def = launcher
            .content_loader
            .block_by_name("payload-loader")
            .unwrap();
        let container_def = launcher.content_loader.block_by_name("container").unwrap();
        let copper = launcher
            .content_loader
            .item_by_name("copper")
            .unwrap()
            .base
            .mappable
            .base
            .id;
        let loader_tile = point2_pack(4, 4);
        let mut loader_building =
            BuildingComp::new(loader_tile, loader_def.base().clone(), TeamId(6));
        loader_building.items.as_mut().unwrap().add(copper, 5);
        loader_building.power.as_mut().unwrap().status = 1.0;
        let mut build_bytes = Vec::new();
        BuildingComp::new(point2_pack(0, 0), container_def.base().clone(), TeamId(6))
            .write_base(&mut build_bytes, false)
            .unwrap();

        launcher.runtime.state.world.resize(10, 10);
        launcher.runtime.add_building(loader_building);
        launcher.runtime.payload_runtime_states.insert(
            loader_tile,
            GameRuntimePayloadBlockState::Loader {
                common: PayloadBlockBuildState {
                    payload: Some(PayloadRef::Block {
                        block: container_def.base().id,
                        version: 0,
                        build_bytes,
                    }),
                    ..PayloadBlockBuildState::default()
                },
                loader: PayloadLoaderState {
                    load_timer: 1.0,
                    ..PayloadLoaderState::default()
                },
            },
        );
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let report = launcher
            .last_runtime_payload_report
            .expect("server update should cache the latest payload batch");
        assert_eq!(report.loader.loader_candidates, 1);
        assert_eq!(report.loader.updated_loaders, 1);
        assert_eq!(report.loader.moved_in_payloads, 1);
        assert_eq!(report.loader.loaded_items, 5);
        assert_eq!(
            launcher.runtime.buildings()[0]
                .items
                .as_ref()
                .unwrap()
                .get(copper),
            0
        );
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_drives_owned_payload_mass_driver_from_launcher_runtime() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let driver_def = launcher
            .content_loader
            .block_by_name("payload-mass-driver")
            .unwrap();
        let (driver_block, length, knockback, charge_time) = match driver_def {
            BlockDef::PayloadMassDriver(driver) => (
                driver.base.clone(),
                driver.length,
                driver.knockback,
                driver.charge_time,
            ),
            _ => panic!("payload-mass-driver should use payload mass driver data"),
        };
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        let router_id = router_def.base().id;
        let source_tile = point2_pack(4, 4);
        let target_tile = point2_pack(8, 4);
        let mut build_bytes = Vec::new();
        BuildingComp::new(point2_pack(0, 0), router_def.base().clone(), TeamId(6))
            .write_base(&mut build_bytes, false)
            .unwrap();
        let payload = PayloadRef::Block {
            block: router_id,
            version: 0,
            build_bytes,
        };

        launcher.runtime.state.world.resize(16, 10);
        launcher.runtime.add_building(BuildingComp::new(
            source_tile,
            driver_block.clone(),
            TeamId(6),
        ));
        launcher
            .runtime
            .add_building(BuildingComp::new(target_tile, driver_block, TeamId(6)));
        launcher.runtime.payload_runtime_states.insert(
            source_tile,
            GameRuntimePayloadBlockState::MassDriver {
                common: PayloadBlockBuildState {
                    payload: Some(payload.clone()),
                    ..PayloadBlockBuildState::default()
                },
                driver: PayloadMassDriverState {
                    link: target_tile,
                    turret_rotation: 0.0,
                    state: PayloadDriverState::Shooting,
                    reload_counter: 0.0,
                    charge: charge_time,
                    loaded: true,
                    charging: true,
                    pay_length: payload_mass_driver_loaded_pay_length(length, 0.0, knockback),
                    ..PayloadMassDriverState::default()
                },
            },
        );
        launcher.runtime.payload_runtime_states.insert(
            target_tile,
            GameRuntimePayloadBlockState::MassDriver {
                common: PayloadBlockBuildState::default(),
                driver: PayloadMassDriverState {
                    turret_rotation: 180.0,
                    state: PayloadDriverState::Accepting,
                    waiting_shooters: vec![source_tile],
                    ..PayloadMassDriverState::default()
                },
            },
        );
        launcher.runtime.state.set(GameStateState::Playing);

        launcher.update();

        let report = launcher
            .last_runtime_payload_report
            .expect("server update should cache the latest payload batch");
        assert_eq!(report.mass_driver.mass_driver_candidates, 2);
        assert_eq!(report.mass_driver.charged_shots, 1);
        assert_eq!(report.mass_driver.fired_payloads, 1);
        assert_eq!(report.mass_driver.received_payloads, 1);
        let Some(GameRuntimePayloadBlockState::MassDriver {
            common: source_common,
            driver: source_driver,
        }) = launcher.runtime.payload_runtime_states.get(&source_tile)
        else {
            panic!("source mass driver state should remain present");
        };
        assert!(source_common.payload.is_none());
        assert_eq!(source_driver.state, PayloadDriverState::Idle);
        let Some(GameRuntimePayloadBlockState::MassDriver {
            common: target_common,
            driver: target_driver,
        }) = launcher.runtime.payload_runtime_states.get(&target_tile)
        else {
            panic!("target mass driver state should remain present");
        };
        assert_eq!(target_common.payload, Some(payload));
        assert_eq!(target_driver.last_other, Some(source_tile));
        assert!(target_driver.effect_delay_timer > 0.0);
        assert_eq!(launcher.runtime.state.update_id, 1);
    }

    #[test]
    fn server_update_drives_owned_payload_constructor_conveyor_void_chain() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let constructor_def = launcher
            .content_loader
            .block_by_name("constructor")
            .unwrap();
        let conveyor_def = launcher
            .content_loader
            .block_by_name("payload-conveyor")
            .unwrap();
        let void_def = launcher
            .content_loader
            .block_by_name("payload-void")
            .unwrap();
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        let router_id = router_def.base().id;
        let constructor_tile = point2_pack(4, 4);
        let constructor_trns = constructor_def.base().size / 2 + 1;
        let conveyor_x = 4 + constructor_trns + (conveyor_def.base().size - 1) / 2;
        let conveyor_tile = point2_pack(conveyor_x, 4);
        let conveyor_trns = conveyor_def.base().size / 2 + 1;
        let void_x = conveyor_x + conveyor_trns + (void_def.base().size - 1) / 2;
        let void_tile = point2_pack(void_x, 4);
        let mut constructor_building =
            BuildingComp::new(constructor_tile, constructor_def.base().clone(), TeamId(6));
        constructor_building.set_rotation(0);
        for requirement in router_def.requirements() {
            constructor_building
                .items
                .as_mut()
                .unwrap()
                .set(requirement.item, requirement.amount);
        }
        let mut conveyor_building =
            BuildingComp::new(conveyor_tile, conveyor_def.base().clone(), TeamId(6));
        conveyor_building.set_rotation(0);

        launcher.runtime.state.world.resize(20, 10);
        launcher.runtime.add_building(constructor_building);
        launcher.runtime.add_building(conveyor_building);
        launcher.runtime.add_building(BuildingComp::new(
            void_tile,
            void_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.payload_runtime_states.insert(
            constructor_tile,
            GameRuntimePayloadBlockState::Constructor {
                common: PayloadBlockBuildState::default(),
                producer: BlockProducerState {
                    progress: 9.5,
                    ..BlockProducerState::default()
                },
                recipe: Some(router_id),
            },
        );
        launcher.runtime.payload_runtime_states.insert(
            conveyor_tile,
            GameRuntimePayloadBlockState::Conveyor(PayloadConveyorState::default()),
        );
        launcher.runtime.payload_runtime_states.insert(
            void_tile,
            GameRuntimePayloadBlockState::Void(PayloadBlockBuildState::default()),
        );
        launcher.runtime.state.set(GameStateState::Playing);

        let mut produced_payloads = 0;
        let mut constructor_transfers = 0;
        let mut conveyor_transfers = 0;
        let mut void_incinerations = 0;
        for frame in 1..=240 {
            launcher.update();
            assert_eq!(launcher.runtime.state.update_id, frame);
            if let Some(report) = launcher.last_runtime_payload_report {
                produced_payloads += report.constructor.produced_payloads;
                constructor_transfers += report.constructor.transferred_payloads;
                conveyor_transfers += report.conveyor.transferred_payloads;
                void_incinerations += report.void.incinerated_payloads;
            }
            if void_incinerations > 0 {
                break;
            }
        }

        assert_eq!(produced_payloads, 1);
        assert_eq!(constructor_transfers, 1);
        assert_eq!(conveyor_transfers, 1);
        assert_eq!(void_incinerations, 1);
        let Some(GameRuntimePayloadBlockState::Void(common)) =
            launcher.runtime.payload_runtime_states.get(&void_tile)
        else {
            panic!("payload void sidecar should remain present");
        };
        assert!(common.payload.is_none());
    }

    #[test]
    fn server_update_drives_owned_payload_loader_deconstructor_chain() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let loader_def = launcher
            .content_loader
            .block_by_name("payload-loader")
            .unwrap();
        let deconstructor_def = launcher
            .content_loader
            .block_by_name("small-deconstructor")
            .unwrap();
        let router_def = launcher.content_loader.block_by_name("router").unwrap();
        let router_id = router_def.base().id;
        let loader_tile = point2_pack(4, 4);
        let loader_trns = loader_def.base().size / 2 + 1;
        let deconstructor_x = 4 + loader_trns + (deconstructor_def.base().size - 1) / 2;
        let deconstructor_tile = point2_pack(deconstructor_x, 4);
        let mut loader_building =
            BuildingComp::new(loader_tile, loader_def.base().clone(), TeamId(6));
        loader_building.set_rotation(0);
        let mut build_bytes = Vec::new();
        BuildingComp::new(point2_pack(0, 0), router_def.base().clone(), TeamId(6))
            .write_base(&mut build_bytes, false)
            .unwrap();

        launcher.runtime.state.world.resize(14, 10);
        launcher.runtime.add_building(loader_building);
        launcher.runtime.add_building(BuildingComp::new(
            deconstructor_tile,
            deconstructor_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.payload_runtime_states.insert(
            loader_tile,
            GameRuntimePayloadBlockState::Loader {
                common: PayloadBlockBuildState {
                    payload: Some(PayloadRef::Block {
                        block: router_id,
                        version: 0,
                        build_bytes,
                    }),
                    ..PayloadBlockBuildState::default()
                },
                loader: PayloadLoaderState {
                    exporting: true,
                    ..PayloadLoaderState::default()
                },
            },
        );
        launcher.runtime.payload_runtime_states.insert(
            deconstructor_tile,
            GameRuntimePayloadBlockState::Deconstructor {
                common: PayloadBlockBuildState::default(),
                deconstructor: PayloadDeconstructorState::default(),
            },
        );
        launcher.runtime.state.set(GameStateState::Playing);

        let mut loader_transfers = 0;
        let mut deconstructor_moved_in = 0;
        let mut started_deconstructions = 0;
        for frame in 1..=240 {
            launcher.update();
            assert_eq!(launcher.runtime.state.update_id, frame);
            if let Some(report) = launcher.last_runtime_payload_report {
                loader_transfers += report.loader.transferred_payloads;
                deconstructor_moved_in += report.deconstructor.moved_in_payloads;
                started_deconstructions += report.deconstructor.started_deconstructions;
            }
            if started_deconstructions > 0 {
                break;
            }
        }

        assert_eq!(loader_transfers, 1);
        assert_eq!(deconstructor_moved_in, 1);
        assert_eq!(started_deconstructions, 1);
        let Some(GameRuntimePayloadBlockState::Deconstructor {
            common,
            deconstructor,
        }) = launcher
            .runtime
            .payload_runtime_states
            .get(&deconstructor_tile)
        else {
            panic!("payload deconstructor sidecar should remain present");
        };
        assert!(common.payload.is_none());
        assert!(matches!(
            deconstructor.deconstructing.as_ref(),
            Some(PayloadRef::Block { block, .. }) if *block == router_id
        ));
    }

    #[test]
    fn server_update_drives_owned_payload_source_router_void_chain() {
        let mut launcher = ServerLauncher::new(Vec::new());
        let source_def = launcher
            .content_loader
            .block_by_name("payload-source")
            .unwrap();
        let router_block_def = launcher
            .content_loader
            .block_by_name("payload-router")
            .unwrap();
        let void_def = launcher
            .content_loader
            .block_by_name("payload-void")
            .unwrap();
        let carried_block = launcher.content_loader.block_by_name("router").unwrap();
        let carried_block_id = carried_block.base().id;
        let source_tile = point2_pack(4, 4);
        let source_trns = source_def.base().size / 2 + 1;
        let router_x = 4 + source_trns + (router_block_def.base().size - 1) / 2;
        let router_tile = point2_pack(router_x, 4);
        let router_trns = router_block_def.base().size / 2 + 1;
        let void_x = router_x + router_trns + (void_def.base().size - 1) / 2;
        let void_tile = point2_pack(void_x, 4);
        let mut source_building =
            BuildingComp::new(source_tile, source_def.base().clone(), TeamId(6));
        source_building.set_rotation(0);
        let mut router_building =
            BuildingComp::new(router_tile, router_block_def.base().clone(), TeamId(6));
        router_building.set_rotation(1);

        launcher.runtime.state.world.resize(20, 10);
        launcher.runtime.add_building(source_building);
        launcher.runtime.add_building(router_building);
        launcher.runtime.add_building(BuildingComp::new(
            void_tile,
            void_def.base().clone(),
            TeamId(6),
        ));
        launcher.runtime.payload_runtime_states.insert(
            source_tile,
            GameRuntimePayloadBlockState::Source {
                common: PayloadBlockBuildState::default(),
                source: PayloadSourceState {
                    config_block: Some(carried_block_id),
                    ..PayloadSourceState::default()
                },
            },
        );
        launcher.runtime.payload_runtime_states.insert(
            router_tile,
            GameRuntimePayloadBlockState::Router {
                conveyor: PayloadConveyorState::default(),
                sorted: Some(PayloadSortKey {
                    content_type: ContentType::Block.ordinal() as i8,
                    id: carried_block_id,
                }),
                rec_dir: 0,
                matches: false,
                smooth_rot: 90.0,
                control_time: -1.0,
            },
        );
        launcher.runtime.payload_runtime_states.insert(
            void_tile,
            GameRuntimePayloadBlockState::Void(PayloadBlockBuildState::default()),
        );
        launcher.runtime.state.set(GameStateState::Playing);

        let mut spawned_payloads = 0;
        let mut source_transfers = 0;
        let mut router_transfers = 0;
        let mut void_incinerations = 0;
        for frame in 1..=360 {
            launcher.update();
            assert_eq!(launcher.runtime.state.update_id, frame);
            if let Some(report) = launcher.last_runtime_payload_report {
                spawned_payloads += report.source.spawned_block_payloads;
                source_transfers += report.source.transferred_payloads;
                router_transfers += report.conveyor.transferred_payloads;
                void_incinerations += report.void.incinerated_payloads;
            }
            if void_incinerations > 0 {
                break;
            }
        }

        assert!(spawned_payloads >= 1);
        assert!(source_transfers >= 1);
        assert!(router_transfers >= 1);
        assert!(void_incinerations >= 1);
        let Some(GameRuntimePayloadBlockState::Router { matches, .. }) =
            launcher.runtime.payload_runtime_states.get(&router_tile)
        else {
            panic!("payload router sidecar should remain present");
        };
        assert!(*matches);
        let Some(GameRuntimePayloadBlockState::Void(_common)) =
            launcher.runtime.payload_runtime_states.get(&void_tile)
        else {
            panic!("payload void sidecar should remain present");
        };
    }
}
