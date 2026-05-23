use mindustry_core::mindustry::client_launcher::ClientLauncher;
use mindustry_core::mindustry::core::{ClientConnectConfig, NetClient};
use mindustry_core::mindustry::net::{ArcNetProvider, Net};
use mindustry_core::mindustry::vars::AppContext;
use mindustry_core::mindustry::UPSTREAM_BASELINE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopConnectTarget {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct DesktopLauncher {
    pub client: ClientLauncher,
    pub net_client: NetClient,
    pub connect_target: Option<DesktopConnectTarget>,
    pub connect_error: Option<String>,
    pub args: Vec<String>,
}

impl DesktopLauncher {
    pub fn new(args: Vec<String>) -> Self {
        let connect_target = parse_connect_target(&args);
        Self {
            client: ClientLauncher::new(AppContext::new("data")),
            net_client: NetClient::with_net(Net::new(Box::new(ArcNetProvider::new()))),
            connect_target,
            connect_error: None,
            args,
        }
    }

    pub fn update(&mut self) {
        self.client.update();
        self.net_client.update();
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
    use mindustry_core::mindustry::net::{
        packet_ids, ConnectPacket, PacketEnvelope, PacketKind, PacketSerializer,
    };
    use mindustry_core::mindustry::net::{ArcNetProvider, NetProvider};
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
