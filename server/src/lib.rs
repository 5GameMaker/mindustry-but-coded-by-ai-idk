pub mod server_control;

use mindustry_core::mindustry::core::{
    content_loader::ContentLoader, GameRuntime, GameRuntimeOwnedEffectResources, NetServer,
};
use mindustry_core::mindustry::ctype::ContentId;
use mindustry_core::mindustry::entities::{
    bullet::BulletType,
    comp::{BuildingComp, BulletComp, UnitComp},
};
use mindustry_core::mindustry::net::{write_minimal_world_data, ArcNetProvider, Net};
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
    pub net_server: NetServer,
    pub network_error: Option<String>,
}

impl ServerLauncher {
    pub fn new(args: Vec<String>) -> Self {
        let mut context = AppContext::server("config");
        if let Some(port) = parse_port_arg(&args) {
            context.port = port;
        }

        Self {
            context,
            control: ServerControl::new(args.clone()),
            runtime: GameRuntime::default(),
            content_loader: ContentLoader::create_base_content_or_panic(),
            last_runtime_effect_report: None,
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
        self.last_runtime_effect_report = self.update_runtime_effect_blocks(1.0 / 60.0);
    }

    pub fn flush_pending_world_data(&self) -> io::Result<usize> {
        self.net_server.send_pending_world_data(|connection_id| {
            write_minimal_world_data(connection_id)
                .expect("bootstrap world data payload should be encodable")
        })
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
    use mindustry_core::mindustry::core::{
        content_loader::ContentLoader, GameRuntime, GameStateState, NetServer,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::net::{
        packet_ids, Connect, ConnectFilter, ConnectPacket, DoneCallback, Host, HostCallback, Net,
        NetConnection, NetProvider, PacketKind, PacketSerializer, ProviderEvent,
    };
    use mindustry_core::mindustry::vars::{AppContext, RuntimeMode};
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
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[0].0, 71);
        assert_eq!(sent[1].0, 71);
        assert!(sent.iter().all(|(_, _, reliable)| *reliable));
        assert!(matches!(sent[0].1, PacketKind::StreamBegin(_)));
        assert!(matches!(sent[1].1, PacketKind::StreamChunk(_)));
        assert!(!sent
            .iter()
            .any(|(_, packet, _)| matches!(packet, PacketKind::WorldDataBeginCallPacket(_))));
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
}
