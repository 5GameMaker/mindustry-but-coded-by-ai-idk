pub mod server_control;

use mindustry_core::mindustry::core::{
    content_loader::ContentLoader, GameRuntime, GameRuntimeNetworkContext,
    GameRuntimeOwnedEffectResources, GameRuntimeOwnedFrameReport,
    GameRuntimeOwnedItemTransportFrameReport, GameRuntimeOwnedPayloadFrameReport, NetServer,
};
use mindustry_core::mindustry::ctype::ContentId;
use mindustry_core::mindustry::entities::{
    bullet::BulletType,
    comp::{BuildingComp, BulletComp, UnitComp},
};
use mindustry_core::mindustry::io::ContentPatchSet;
use mindustry_core::mindustry::net::{
    write_world_data, ArcNetProvider, Net, NetworkPlayerData, NetworkWorldData,
};
use mindustry_core::mindustry::vars::{AppContext, RuntimeMode};
use mindustry_core::mindustry::world::blocks::defense::EffectBlockFrameBatchReport;
use mindustry_core::mindustry::UPSTREAM_BASELINE;
use server_control::ServerControl;
use std::io;

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
        let _ = self.flush_pending_world_data();
        if let Some(report) = self.update_runtime_owned_blocks(1.0 / 60.0) {
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
    use super::ServerLauncher;
    use mindustry_core::mindustry::content::blocks::BlockDef;
    use mindustry_core::mindustry::core::game_runtime::GameRuntimePayloadBlockState;
    use mindustry_core::mindustry::core::{
        content_loader::ContentLoader, GameRuntime, GameRuntimeNetworkContext, GameStateState,
        NetServer,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::game::{BlockPlan, TEAM_SHARDED};
    use mindustry_core::mindustry::io::{TeamId, TypeValue};
    use mindustry_core::mindustry::net::{
        packet_ids, read_world_data, Connect, ConnectFilter, ConnectPacket, DoneCallback, Host,
        HostCallback, Net, NetConnection, NetProvider, NetworkWorldData, PacketKind,
        PacketSerializer, ProviderEvent,
    };
    use mindustry_core::mindustry::vars::{AppContext, RuntimeMode};
    use mindustry_core::mindustry::world::blocks::payloads::{
        BlockProducerState, PayloadBlockBuildState, PayloadConveyorState,
        PayloadDeconstructorState, PayloadRef, PayloadSourceState,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use std::io;
    use std::net::{TcpListener, UdpSocket};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

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
}
