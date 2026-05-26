use mindustry_core::mindustry::client_launcher::ClientLauncher;
use mindustry_core::mindustry::core::{
    content_loader::ContentLoader, ClientConnectConfig, GameRuntime, GameRuntimeMapLoadReport,
    GameRuntimeNetworkContext, GameState, GameStateState, NetClient,
};
use mindustry_core::mindustry::ctype::{ContentId, ContentType};
use mindustry_core::mindustry::entities::PlayerComp;
use mindustry_core::mindustry::io::{ContentHeaderSnapshot, LegacyTeamBlocks, TeamId};
use mindustry_core::mindustry::net::{
    ArcNetProvider, Net, NetworkPlayerData, NetworkWorldData, StateSnapshotCallPacket,
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
    last_runtime_map_load_report: Option<GameRuntimeMapLoadReport>,
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
            last_runtime_map_load_report: None,
        }
    }

    pub fn update(&mut self) {
        self.client.update();
        self.net_client.update();
        self.sync_loaded_world_data();
        self.sync_client_loaded_state();
        self.sync_state_snapshot();
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
                }
            }
        }
        self.last_applied_world_data = loaded_world_data;
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
    use mindustry_core::mindustry::core::{GameRuntimeNetworkContext, WorldLoadEventKind};
    use mindustry_core::mindustry::ctype::ContentId;
    use mindustry_core::mindustry::ctype::ContentType;
    use mindustry_core::mindustry::io::{
        ContentHeaderEntry, ContentHeaderSnapshot, LegacyMapBlockRecord, LegacyMapFloorRecord,
        LegacyShortChunkMap,
    };
    use mindustry_core::mindustry::net::{
        packet_ids, ConnectPacket, PacketEnvelope, PacketKind, PacketSerializer,
    };
    use mindustry_core::mindustry::net::{ArcNetProvider, NetProvider};
    use mindustry_core::mindustry::net::{
        NetworkPlayerData, NetworkWorldData, StateSnapshotCallPacket,
    };
    use mindustry_core::mindustry::{
        entities::{comp::BuildingComp, PlayerComp},
        game::{BlockPlan, TEAM_CRUX, TEAM_SHARDED},
        io::{
            LegacyTeamBlockGroup, LegacyTeamBlockPlan, LegacyTeamBlocks, TeamId, TypeValue, UnitRef,
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
