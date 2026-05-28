use mindustry_core::mindustry::client_launcher::ClientLauncher;
use mindustry_core::mindustry::content::blocks::{BlockDef, DistributionBlockKind};
use mindustry_core::mindustry::core::game_runtime::{
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
use mindustry_core::mindustry::ctype::{ContentId, ContentType};
use mindustry_core::mindustry::entities::{
    entity_class_kind, standard_effect,
    standard_effect_draw_plans_with_data_value_and_resolved_context,
    standard_effect_render_lifetime, EffectRenderInput, EntityClassKind, PlayerComp,
    PlayerUnitSwitchContext, ShieldArcAbility, StandardEffectCircleRenderPrimitive,
    StandardEffectDrawPlan, StandardEffectLightRenderPrimitive, StandardEffectLineRenderPrimitive,
    StandardEffectRectRenderPrimitive, StandardEffectShieldArcBreak,
    StandardEffectSquareRenderPrimitive, StandardEffectTriangleRenderPrimitive, PLAYER_CLASS_ID,
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
use mindustry_core::mindustry::net::{
    ArcNetProvider, EffectCallPacket2, Net, NetworkPlayerData, NetworkPlayerSyncData,
    NetworkWorldData, PacketKind, StateSnapshotCallPacket,
};
use mindustry_core::mindustry::service::{
    AchievementContext, GameServiceApplySummary, GameServiceTriggerSnapshot,
};
use mindustry_core::mindustry::vars::{AppContext, MAX_PLAYER_PREVIEW_PLANS};
use mindustry_core::mindustry::UPSTREAM_BASELINE;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub connect_target: Option<DesktopConnectTarget>,
    pub connect_error: Option<String>,
    pub args: Vec<String>,
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

impl DesktopLauncher {
    pub fn new(args: Vec<String>) -> Self {
        let connect_target = parse_connect_target(&args);
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
            connect_target,
            connect_error: None,
            args,
            content_loader: ContentLoader::create_base_content_or_panic(),
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
                    self.runtime.apply_client_unit_destroy_packet(&packet)
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
    use super::{run, DesktopEffectRenderStats, DesktopLauncher, HeadlessDesktopEffectRenderer};
    use mindustry_core::mindustry::core::game_runtime::{
        GameRuntimeCampaignBlockState, GameRuntimeDistributionBlockState,
        GameRuntimePayloadBlockState, GameRuntimeReconstructorConfigureResult,
        GameRuntimeUnitBlockState, GameRuntimeUnitCargoUnloadConfigureResult,
        GameRuntimeUnitFactoryConfigureResult,
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
    use mindustry_core::mindustry::io::{
        ContentHeaderEntry, ContentHeaderSnapshot, LegacyMapBlockRecord, LegacyMapFloorRecord,
        LegacyShortChunkMap,
    };
    use mindustry_core::mindustry::net::{
        packet_ids, ConnectPacket, PacketEnvelope, PacketKind, PacketSerializer,
    };
    use mindustry_core::mindustry::net::{ArcNetProvider, NetProvider};
    use mindustry_core::mindustry::net::{
        AssemblerDroneSpawnedCallPacket, AssemblerUnitSpawnedCallPacket,
        ClientPlanSnapshotReceivedCallPacket, CommandBuildingCallPacket, EffectCallPacket,
        EffectCallPacket2, LandingPadLandedCallPacket, NetworkPlayerData, NetworkPlayerSyncData,
        NetworkWorldData, StateSnapshotCallPacket, TileConfigCallPacket, UnitBlockSpawnCallPacket,
        UnitCapDeathCallPacket, UnitDeathCallPacket, UnitDespawnCallPacket, UnitDestroyCallPacket,
        UnitEnteredPayloadCallPacket, UnitEnvDeathCallPacket, UnitSafeDeathCallPacket,
        UnitSpawnCallPacket, UnitTetherBlockSpawnedCallPacket,
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
        r#type::{ItemStack, PayloadKey, PayloadSeq, Sector, UnitType},
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
        assert_eq!(launcher.runtime.client_local_sound_at_events.len(), 1);
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
        assert_eq!(launcher.runtime.client_local_sound_at_events.len(), 1);
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

        let mut unit_type = UnitType::new(9903, "crawler");
        unit_type.allow_leg_step = true;
        unit_type.leg_count = 2;
        unit_type.leg_length = 10.0;
        unit_type.leg_extension = 3.0;
        unit_type.leg_region = TextureRegionRef::with_size("crawler-leg", 16, 8);
        unit_type.leg_base_region = TextureRegionRef::with_size("crawler-leg-base", 12, 6);
        let mut unit = UnitComp::new(9903, unit_type, TeamId(4));
        unit.add();
        unit.set_pos(10.0, 20.0);
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
        assert_eq!(launcher.runtime.client_local_effect_entities.len(), 4);
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 4);
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
        assert_eq!(
            launcher.standard_local_effect_line_primitives.len(),
            leg_primitives + leg_base_primitives
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
        assert_eq!(launcher.runtime.client_local_effect_entities.len(), 8);
        assert_eq!(launcher.standard_local_effect_line_primitives.len(), 8);
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
        unit_type.death_explosion_effect = "despawn".into();
        unit_type.hit_size = 16.0;
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
