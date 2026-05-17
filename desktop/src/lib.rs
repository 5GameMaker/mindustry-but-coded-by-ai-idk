use mindustry_core::mindustry::client_launcher::ClientLauncher;
use mindustry_core::mindustry::vars::AppContext;
use mindustry_core::mindustry::UPSTREAM_BASELINE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopLauncher {
    pub client: ClientLauncher,
    pub args: Vec<String>,
}

impl DesktopLauncher {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            client: ClientLauncher::new(AppContext::new("data")),
            args,
        }
    }
}

pub fn run(args: Vec<String>) -> DesktopLauncher {
    let mut launcher = DesktopLauncher::new(args);
    launcher.client.setup();
    launcher
}

pub fn banner() -> String {
    format!("mindustry desktop bootstrap ({UPSTREAM_BASELINE})")
}

#[cfg(test)]
mod tests {
    use mindustry_core::mindustry::net::{
        packet_ids, ConnectPacket, PacketEnvelope, PacketKind, PacketSerializer,
    };

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
