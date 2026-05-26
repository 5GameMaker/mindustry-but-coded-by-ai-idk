use mindustry_core::mindustry::client_launcher::ClientLauncher;
use mindustry_core::mindustry::core::game_runtime::GameRuntimeClientSnapshotApplyReport;
use mindustry_core::mindustry::core::net_client::{
    ClientBlockSnapshotMirror, ClientHiddenSnapshotMirror,
};
use mindustry_core::mindustry::core::{
    content_loader::ContentLoader, ClientConnectConfig, GameRuntime, GameRuntimeMapLoadReport,
    GameRuntimeNetworkContext, GameState, GameStateState, NetClient,
};
use mindustry_core::mindustry::ctype::{ContentId, ContentType};
use mindustry_core::mindustry::entities::{
    entity_class_kind, EntityClassKind, PlayerComp, PlayerUnitSwitchContext, PLAYER_CLASS_ID,
};
use mindustry_core::mindustry::io::{
    read_unit_sync, ContentHeaderSnapshot, LegacyTeamBlocks, TeamId,
};
use mindustry_core::mindustry::net::{
    ArcNetProvider, Net, NetworkPlayerData, NetworkPlayerSyncData, NetworkWorldData,
    StateSnapshotCallPacket,
};
use mindustry_core::mindustry::vars::AppContext;
use mindustry_core::mindustry::UPSTREAM_BASELINE;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopConnectTarget {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct DesktopLauncher {
    pub client: ClientLauncher,
    pub net_client: NetClient,
    pub game_state: GameState,
    pub runtime: GameRuntime,
    pub player: PlayerComp,
    pub connect_target: Option<DesktopConnectTarget>,
    pub connect_error: Option<String>,
    pub args: Vec<String>,
    content_loader: ContentLoader,
    last_applied_world_data: Option<mindustry_core::mindustry::net::NetworkWorldData>,
    last_applied_state_snapshot: Option<StateSnapshotCallPacket>,
    last_applied_block_snapshot_mirror: Option<ClientBlockSnapshotMirror>,
    last_applied_entity_snapshot_mirror_count: usize,
    last_applied_hidden_snapshot_mirror: Option<ClientHiddenSnapshotMirror>,
    last_runtime_map_load_report: Option<GameRuntimeMapLoadReport>,
    last_client_snapshot_apply_report: Option<GameRuntimeClientSnapshotApplyReport>,
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
            connect_target,
            connect_error: None,
            args,
            content_loader: ContentLoader::create_base_content_or_panic(),
            last_applied_world_data: None,
            last_applied_state_snapshot: None,
            last_applied_block_snapshot_mirror: None,
            last_applied_entity_snapshot_mirror_count: 0,
            last_applied_hidden_snapshot_mirror: None,
            last_runtime_map_load_report: None,
            last_client_snapshot_apply_report: None,
        }
    }

    pub fn update(&mut self) {
        self.client.update();
        self.net_client.update();
        self.sync_loaded_world_data();
        self.sync_client_loaded_state();
        self.sync_state_snapshot();
        self.sync_snapshot_mirrors();
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
                report.merge(
                    self.runtime
                        .apply_client_entity_snapshot_record_with_content(
                            &self.content_loader,
                            record.entity_id,
                            record.type_id,
                            record.sync_bytes.clone(),
                        ),
                );
                if record.type_id == PLAYER_CLASS_ID
                    && self
                        .apply_client_player_entity_snapshot(record.entity_id, &record.sync_bytes)
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

    fn reset_snapshot_apply_cursors_to_current_net_state(&mut self) {
        let (block_mirror, entity_count, hidden_mirror) = {
            let state = self.net_client.state();
            let state = state.lock().unwrap();
            (
                state.last_block_snapshot_mirror.clone(),
                state.entity_snapshot_mirrors.len(),
                state.last_hidden_snapshot_mirror.clone(),
            )
        };
        self.last_applied_block_snapshot_mirror = block_mirror;
        self.last_applied_entity_snapshot_mirror_count = entity_count;
        self.last_applied_hidden_snapshot_mirror = hidden_mirror;
        self.last_client_snapshot_apply_report = None;
    }

    fn clear_snapshot_apply_cursors(&mut self) {
        self.last_applied_block_snapshot_mirror = None;
        self.last_applied_entity_snapshot_mirror_count = 0;
        self.last_applied_hidden_snapshot_mirror = None;
        self.last_client_snapshot_apply_report = None;
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

            if entity_class_kind(type_id) != Some(EntityClassKind::Unit) {
                report.entity_parse_errors += 1;
                return report;
            }
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
    use super::{run, DesktopLauncher};
    use mindustry_core::mindustry::core::game_runtime::GameRuntimePayloadBlockState;
    use mindustry_core::mindustry::core::net_client::{
        ClientBlockSnapshotMirror, ClientBlockSnapshotRecordMirror, ClientEntitySnapshotMirror,
        ClientEntitySnapshotRecordMirror, ClientHiddenSnapshotMirror,
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
        NetworkPlayerData, NetworkPlayerSyncData, NetworkWorldData, StateSnapshotCallPacket,
    };
    use mindustry_core::mindustry::{
        entities::{comp::BuildingComp, PlayerComp, PLAYER_CLASS_ID},
        game::{BlockPlan, TEAM_CRUX, TEAM_SHARDED},
        io::type_io::ControllerWire,
        io::{
            type_io, LegacyTeamBlockGroup, LegacyTeamBlockPlan, LegacyTeamBlocks, TeamId,
            TypeValue, UnitRef, Vec2 as IoVec2,
        },
        r#type::ItemStack,
        world::blocks::payloads::{PayloadBlockBuildState, PayloadLoaderState, PayloadRef},
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

        let mut data = Vec::new();
        data.extend_from_slice(&launcher.player.id.to_be_bytes());
        data.push(PLAYER_CLASS_ID);
        data.extend_from_slice(&player_bytes);
        data.extend_from_slice(&8801i32.to_be_bytes());
        data.push(2);
        data.extend_from_slice(&unit_bytes);

        {
            let state = launcher.net_client.state();
            let mut state = state.lock().unwrap();
            state
                .entity_snapshot_mirrors
                .push(ClientEntitySnapshotMirror {
                    amount: 2,
                    data,
                    records: Vec::new(),
                    parse_error: Some(
                        "multi-record entity snapshot with opaque sync bytes is not splittable yet"
                            .into(),
                    ),
                });
        }

        launcher.update();

        let report = launcher
            .last_client_snapshot_apply_report
            .expect("mixed fallback should apply player and unit records");
        assert_eq!(report.entity_records_applied, 2);
        assert_eq!(report.entity_typed_records_applied, 2);
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
