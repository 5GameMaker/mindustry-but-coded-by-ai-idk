pub mod server_control;

use mindustry_core::mindustry::core::NetServer;
use mindustry_core::mindustry::net::{ArcNetProvider, Net};
use mindustry_core::mindustry::vars::{AppContext, RuntimeMode};
use mindustry_core::mindustry::UPSTREAM_BASELINE;
use server_control::ServerControl;

#[derive(Debug, Clone)]
pub struct ServerLauncher {
    pub context: AppContext,
    pub args: Vec<String>,
    pub control: ServerControl,
    pub net_server: NetServer,
    pub network_error: Option<String>,
}

impl ServerLauncher {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            context: AppContext::server("config"),
            control: ServerControl::new(args.clone()),
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

    pub fn update(&self) {
        self.net_server.update();
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

#[cfg(test)]
mod tests {
    use super::ServerLauncher;
    use mindustry_core::mindustry::net::{packet_ids, ConnectPacket, PacketKind, PacketSerializer};
    use mindustry_core::mindustry::vars::RuntimeMode;
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
}
