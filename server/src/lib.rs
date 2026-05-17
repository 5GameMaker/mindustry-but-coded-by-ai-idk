pub mod server_control;

use mindustry_core::mindustry::vars::{AppContext, RuntimeMode};
use mindustry_core::mindustry::UPSTREAM_BASELINE;
use server_control::ServerControl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLauncher {
    pub context: AppContext,
    pub args: Vec<String>,
    pub control: ServerControl,
}

impl ServerLauncher {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            context: AppContext::server("config"),
            control: ServerControl::new(args.clone()),
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
    use mindustry_core::mindustry::net::{packet_ids, ConnectPacket, PacketKind, PacketSerializer};

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
