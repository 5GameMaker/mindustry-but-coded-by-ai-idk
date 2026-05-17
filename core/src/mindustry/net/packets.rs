use base64::Engine;
use crc32fast::Hasher;
use std::io::{Read, Write};

use crate::mindustry::io::type_io::{
    read_i32, read_i64, read_string, read_u8, read_u32, write_i32, write_i64, write_string,
    write_u32,
};

use super::packet::{PacketCodec, PacketPriority, PacketRuntime};

pub type PacketId = u8;

/// Packet IDs assigned by `mindustry.net.Net.registerPacket(...)` in upstream v157.4.
pub mod packet_ids {
    use super::PacketId;

    pub const STREAM_BEGIN: PacketId = 0;
    pub const STREAM_CHUNK: PacketId = 1;
    pub const WORLD_STREAM: PacketId = 2;
    pub const CONNECT_PACKET: PacketId = 3;
    pub const ANNOUNCE_CALL_PACKET: PacketId = 5;
    pub const CLEAR_OBJECTIVES_CALL_PACKET: PacketId = 17;
    pub const CLIENT_BINARY_PACKET_RELIABLE_CALL_PACKET: PacketId = 18;
    pub const CLIENT_BINARY_PACKET_UNRELIABLE_CALL_PACKET: PacketId = 19;
    pub const CLIENT_PACKET_RELIABLE_CALL_PACKET: PacketId = 22;
    pub const CLIENT_PACKET_UNRELIABLE_CALL_PACKET: PacketId = 23;
    pub const COMPLETE_OBJECTIVE_CALL_PACKET: PacketId = 29;
    pub const CONNECT_CALL_PACKET: PacketId = 30;
    pub const COPY_TO_CLIPBOARD_CALL_PACKET: PacketId = 33;
    pub const DEBUG_STATUS_CLIENT_CALL_PACKET: PacketId = 37;
    pub const DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET: PacketId = 38;
    pub const HIDE_HUD_TEXT_CALL_PACKET: PacketId = 51;
    pub const INFO_MESSAGE_CALL_PACKET: PacketId = 52;
    pub const INFO_TOAST_CALL_PACKET: PacketId = 57;
    pub const KICK_CALL_PACKET: PacketId = 59;
    pub const OPEN_URI_CALL_PACKET: PacketId = 68;
    pub const PING_RESPONSE_CALL_PACKET: PacketId = 74;
    pub const PLAYER_DISCONNECT_CALL_PACKET: PacketId = 75;
    pub const REMOVE_MARKER_CALL_PACKET: PacketId = 77;
    pub const REMOVE_QUEUE_BLOCK_CALL_PACKET: PacketId = 78;
    pub const WORLD_DATA_BEGIN_CALL_PACKET: PacketId = 157;
}

/// Framework message IDs written after the outer `PacketSerializer` framework marker.
pub mod framework_message_ids {
    use super::PacketId;

    pub const PING: PacketId = 0;
    pub const DISCOVER_HOST: PacketId = 1;
    pub const KEEP_ALIVE: PacketId = 2;
    pub const REGISTER_UDP: PacketId = 3;
    pub const REGISTER_TCP: PacketId = 4;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketTransport {
    /// Normal `mindustry.net.Packet` serialized as `id + length + compression + payload`.
    NetPacket,
    /// Arc framework message serialized behind the outer `0xfe` marker.
    FrameworkMessage,
    /// UDP discovery response payload handled by `NetworkIO`, not by `Net.registerPacket`.
    DiscoveryPayload,
    /// ArcNet connection/disconnection event created locally by listeners.
    LocalEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketDirection {
    ClientToServer,
    ServerToClient,
    Bidirectional,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketCodecState {
    /// Read/write support exists in this Rust crate.
    Implemented,
    /// The packet is represented as stream bytes; it has no separate field codec.
    StreamPayload,
    /// The upstream type is not serialized as a normal packet payload.
    NotSerialized,
    /// Known upstream generated packet, but intentionally not covered in this phase.
    NotCovered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketManifestEntry {
    pub id: Option<PacketId>,
    pub name: &'static str,
    pub transport: PacketTransport,
    pub direction: PacketDirection,
    pub streamable: bool,
    pub priority: Option<PacketPriority>,
    pub allow_client_endpoint: bool,
    pub allow_server_endpoint: bool,
    pub force_uncompressed: bool,
    pub codec: PacketCodecState,
    pub upstream: &'static str,
    pub notes: &'static str,
}

pub const CONNECT_EVENT_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: None,
    name: "Connect",
    transport: PacketTransport::LocalEvent,
    direction: PacketDirection::Local,
    streamable: false,
    priority: Some(PacketPriority::High),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::NotSerialized,
    upstream: "Packets.Connect",
    notes: "ArcNet listener event; not registered by Net.registerPacket.",
};

pub const DISCONNECT_EVENT_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: None,
    name: "Disconnect",
    transport: PacketTransport::LocalEvent,
    direction: PacketDirection::Local,
    streamable: false,
    priority: Some(PacketPriority::High),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::NotSerialized,
    upstream: "Packets.Disconnect",
    notes: "ArcNet listener event; not registered by Net.registerPacket.",
};

pub const STREAM_BEGIN_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::STREAM_BEGIN),
    name: "StreamBegin",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "Packets.StreamBegin",
    notes: "First upstream Net.registerPacket entry; allow(server) returns !server.",
};

pub const STREAM_CHUNK_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::STREAM_CHUNK),
    name: "StreamChunk",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: true,
    codec: PacketCodecState::Implemented,
    upstream: "Packets.StreamChunk",
    notes: "Second upstream Net.registerPacket entry; serializer never compresses StreamChunk.",
};

pub const WORLD_STREAM_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::WORLD_STREAM),
    name: "WorldStream",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: true,
    priority: Some(PacketPriority::High),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::StreamPayload,
    upstream: "Packets.WorldStream",
    notes: "Extends Streamable; payload is carried by StreamBegin/StreamChunk.",
};

pub const CONNECT_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::CONNECT_PACKET),
    name: "ConnectPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ClientToServer,
    streamable: false,
    priority: Some(PacketPriority::High),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "Packets.ConnectPacket",
    notes: "Fourth upstream Net.registerPacket entry; keeps Java read/write asymmetry.",
};

pub const ANNOUNCE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::ANNOUNCE_CALL_PACKET),
    name: "AnnounceCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.AnnounceCallPacket",
    notes: "Generated by Call.registerPackets(); single TypeIO string payload.",
};

pub const CLEAR_OBJECTIVES_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::CLEAR_OBJECTIVES_CALL_PACKET),
    name: "ClearObjectivesCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.ClearObjectivesCallPacket",
    notes: "Generated by Call.registerPackets(); no fields.",
};

pub const CLIENT_BINARY_PACKET_RELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry =
    PacketManifestEntry {
        id: Some(packet_ids::CLIENT_BINARY_PACKET_RELIABLE_CALL_PACKET),
        name: "ClientBinaryPacketReliableCallPacket",
        transport: PacketTransport::NetPacket,
        direction: PacketDirection::ClientToServer,
        streamable: false,
        priority: Some(PacketPriority::Normal),
        allow_client_endpoint: false,
        allow_server_endpoint: true,
        force_uncompressed: false,
        codec: PacketCodecState::Implemented,
        upstream: "mindustry.gen.ClientBinaryPacketReliableCallPacket",
        notes: "Generated by Call.registerPackets(); TypeIO string plus TypeIO byte[] payload.",
    };

pub const CLIENT_BINARY_PACKET_UNRELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry =
    PacketManifestEntry {
        id: Some(packet_ids::CLIENT_BINARY_PACKET_UNRELIABLE_CALL_PACKET),
        name: "ClientBinaryPacketUnreliableCallPacket",
        transport: PacketTransport::NetPacket,
        direction: PacketDirection::ClientToServer,
        streamable: false,
        priority: Some(PacketPriority::Normal),
        allow_client_endpoint: false,
        allow_server_endpoint: true,
        force_uncompressed: false,
        codec: PacketCodecState::Implemented,
        upstream: "mindustry.gen.ClientBinaryPacketUnreliableCallPacket",
        notes: "Generated by Call.registerPackets(); unreliable variant of binary custom packet.",
    };

pub const CLIENT_PACKET_RELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::CLIENT_PACKET_RELIABLE_CALL_PACKET),
    name: "ClientPacketReliableCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ClientToServer,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: false,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.ClientPacketReliableCallPacket",
    notes: "Generated by Call.registerPackets(); two TypeIO string fields: type, contents.",
};

pub const CLIENT_PACKET_UNRELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry =
    PacketManifestEntry {
        id: Some(packet_ids::CLIENT_PACKET_UNRELIABLE_CALL_PACKET),
        name: "ClientPacketUnreliableCallPacket",
        transport: PacketTransport::NetPacket,
        direction: PacketDirection::ClientToServer,
        streamable: false,
        priority: Some(PacketPriority::Normal),
        allow_client_endpoint: false,
        allow_server_endpoint: true,
        force_uncompressed: false,
        codec: PacketCodecState::Implemented,
        upstream: "mindustry.gen.ClientPacketUnreliableCallPacket",
        notes: "Generated by Call.registerPackets(); unreliable variant of string custom packet.",
    };

pub const COMPLETE_OBJECTIVE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::COMPLETE_OBJECTIVE_CALL_PACKET),
    name: "CompleteObjectiveCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.CompleteObjectiveCallPacket",
    notes: "Generated by Call.registerPackets(); single Java int objective index.",
};

pub const CONNECT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::CONNECT_CALL_PACKET),
    name: "ConnectCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.ConnectCallPacket",
    notes: "Generated by Call.registerPackets(); TypeIO string ip plus Java int port.",
};

pub const COPY_TO_CLIPBOARD_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::COPY_TO_CLIPBOARD_CALL_PACKET),
    name: "CopyToClipboardCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.CopyToClipboardCallPacket",
    notes: "Generated by Call.registerPackets(); single TypeIO string text payload.",
};

pub const DEBUG_STATUS_CLIENT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::DEBUG_STATUS_CLIENT_CALL_PACKET),
    name: "DebugStatusClientCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::High),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.DebugStatusClientCallPacket",
    notes: "Generated by Call.registerPackets(); three Java int fields, high priority.",
};

pub const DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry =
    PacketManifestEntry {
        id: Some(packet_ids::DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET),
        name: "DebugStatusClientUnreliableCallPacket",
        transport: PacketTransport::NetPacket,
        direction: PacketDirection::Bidirectional,
        streamable: false,
        priority: Some(PacketPriority::High),
        allow_client_endpoint: true,
        allow_server_endpoint: true,
        force_uncompressed: false,
        codec: PacketCodecState::Implemented,
        upstream: "mindustry.gen.DebugStatusClientUnreliableCallPacket",
        notes: "Generated by Call.registerPackets(); unreliable variant of debug status packet.",
    };

pub const SERVER_DATA_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: None,
    name: "ServerData",
    transport: PacketTransport::DiscoveryPayload,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: None,
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "NetworkIO.writeServerData/readServerData",
    notes: "UDP discovery response payload; not a Net.registerPacket entry.",
};

pub const PING_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(framework_message_ids::PING),
    name: "Ping",
    transport: PacketTransport::FrameworkMessage,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: None,
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "ArcNetProvider.PacketSerializer.FrameworkMessage.Ping",
    notes: "Upstream uses one Ping framework message with an isReply flag; no separate Pong class.",
};

pub const REGISTERED_PACKET_MANIFEST: [PacketManifestEntry; 15] = [
    STREAM_BEGIN_MANIFEST,
    STREAM_CHUNK_MANIFEST,
    WORLD_STREAM_MANIFEST,
    CONNECT_PACKET_MANIFEST,
    ANNOUNCE_CALL_PACKET_MANIFEST,
    CLEAR_OBJECTIVES_CALL_PACKET_MANIFEST,
    CLIENT_BINARY_PACKET_RELIABLE_CALL_PACKET_MANIFEST,
    CLIENT_BINARY_PACKET_UNRELIABLE_CALL_PACKET_MANIFEST,
    CLIENT_PACKET_RELIABLE_CALL_PACKET_MANIFEST,
    CLIENT_PACKET_UNRELIABLE_CALL_PACKET_MANIFEST,
    COMPLETE_OBJECTIVE_CALL_PACKET_MANIFEST,
    CONNECT_CALL_PACKET_MANIFEST,
    COPY_TO_CLIPBOARD_CALL_PACKET_MANIFEST,
    DEBUG_STATUS_CLIENT_CALL_PACKET_MANIFEST,
    DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET_MANIFEST,
];

pub const PACKET_MANIFEST: [PacketManifestEntry; 19] = [
    CONNECT_EVENT_MANIFEST,
    DISCONNECT_EVENT_MANIFEST,
    STREAM_BEGIN_MANIFEST,
    STREAM_CHUNK_MANIFEST,
    WORLD_STREAM_MANIFEST,
    CONNECT_PACKET_MANIFEST,
    ANNOUNCE_CALL_PACKET_MANIFEST,
    CLEAR_OBJECTIVES_CALL_PACKET_MANIFEST,
    CLIENT_BINARY_PACKET_RELIABLE_CALL_PACKET_MANIFEST,
    CLIENT_BINARY_PACKET_UNRELIABLE_CALL_PACKET_MANIFEST,
    CLIENT_PACKET_RELIABLE_CALL_PACKET_MANIFEST,
    CLIENT_PACKET_UNRELIABLE_CALL_PACKET_MANIFEST,
    COMPLETE_OBJECTIVE_CALL_PACKET_MANIFEST,
    CONNECT_CALL_PACKET_MANIFEST,
    COPY_TO_CLIPBOARD_CALL_PACKET_MANIFEST,
    DEBUG_STATUS_CLIENT_CALL_PACKET_MANIFEST,
    DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET_MANIFEST,
    SERVER_DATA_MANIFEST,
    PING_MANIFEST,
];

pub const PACKET_MANIFEST_PHASE1_GAPS: [&str; 2] = [
    "Generated Call.registerPackets() entries, including any generated InvokePacket classes, are not covered because the generated Java output is not checked into upstream v157.4.",
    "Pong is not listed separately because upstream ArcNetProvider uses FrameworkMessage.Ping plus an isReply flag.",
];

pub fn registered_packet_manifest() -> &'static [PacketManifestEntry] {
    &REGISTERED_PACKET_MANIFEST
}

pub fn packet_manifest() -> &'static [PacketManifestEntry] {
    &PACKET_MANIFEST
}

pub fn packet_manifest_phase1_gaps() -> &'static [&'static str] {
    &PACKET_MANIFEST_PHASE1_GAPS
}

pub fn find_registered_packet_by_id(id: PacketId) -> Option<&'static PacketManifestEntry> {
    REGISTERED_PACKET_MANIFEST
        .iter()
        .find(|entry| entry.id == Some(id))
}

pub fn find_packet_by_transport_id(
    transport: PacketTransport,
    id: PacketId,
) -> Option<&'static PacketManifestEntry> {
    PACKET_MANIFEST
        .iter()
        .find(|entry| entry.transport == transport && entry.id == Some(id))
}

pub fn find_packet_by_name(name: &str) -> Option<&'static PacketManifestEntry> {
    PACKET_MANIFEST.iter().find(|entry| entry.name == name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KickReason {
    Kick,
    ClientOutdated,
    ServerOutdated,
    Banned,
    Gameover,
    RecentKick,
    NameInUse,
    IdInUse,
    NameEmpty,
    CustomClient,
    ServerClose,
    Vote,
    TypeMismatch,
    Whitelist,
    PlayerLimit,
    ServerRestarting,
}

impl KickReason {
    pub const ALL: [KickReason; 16] = [
        KickReason::Kick,
        KickReason::ClientOutdated,
        KickReason::ServerOutdated,
        KickReason::Banned,
        KickReason::Gameover,
        KickReason::RecentKick,
        KickReason::NameInUse,
        KickReason::IdInUse,
        KickReason::NameEmpty,
        KickReason::CustomClient,
        KickReason::ServerClose,
        KickReason::Vote,
        KickReason::TypeMismatch,
        KickReason::Whitelist,
        KickReason::PlayerLimit,
        KickReason::ServerRestarting,
    ];

    pub const fn quiet(self) -> bool {
        matches!(self, KickReason::Gameover)
    }

    pub const fn wire_name(self) -> &'static str {
        match self {
            KickReason::Kick => "kick",
            KickReason::ClientOutdated => "clientOutdated",
            KickReason::ServerOutdated => "serverOutdated",
            KickReason::Banned => "banned",
            KickReason::Gameover => "gameover",
            KickReason::RecentKick => "recentKick",
            KickReason::NameInUse => "nameInUse",
            KickReason::IdInUse => "idInUse",
            KickReason::NameEmpty => "nameEmpty",
            KickReason::CustomClient => "customClient",
            KickReason::ServerClose => "serverClose",
            KickReason::Vote => "vote",
            KickReason::TypeMismatch => "typeMismatch",
            KickReason::Whitelist => "whitelist",
            KickReason::PlayerLimit => "playerLimit",
            KickReason::ServerRestarting => "serverRestarting",
        }
    }

    pub fn bundle_key(self) -> String {
        format!("server.kicked.{}", self.wire_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdminAction {
    Kick,
    Ban,
    Trace,
    Wave,
    SwitchTeam,
}

impl AdminAction {
    pub const ALL: [AdminAction; 5] = [
        AdminAction::Kick,
        AdminAction::Ban,
        AdminAction::Trace,
        AdminAction::Wave,
        AdminAction::SwitchTeam,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connect {
    pub address_tcp: String,
}

impl PacketRuntime for Connect {
    fn priority(&self) -> PacketPriority {
        PacketPriority::High
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disconnect {
    pub reason: String,
}

impl PacketRuntime for Disconnect {
    fn priority(&self) -> PacketPriority {
        PacketPriority::High
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamBegin {
    pub id: i32,
    pub total: i32,
    pub packet_type: u8,
}

impl PacketRuntime for StreamBegin {
    fn allow(&self, server: bool) -> bool {
        !server
    }
}

impl PacketCodec for StreamBegin {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let id = read_i32(read)?;
        let total = read_i32(read)?;
        let packet_type = read_u8(read)?;
        Ok(Self {
            id,
            total,
            packet_type,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.id)?;
        write_i32(write, self.total)?;
        write.write_all(&[self.packet_type])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamChunk {
    pub id: i32,
    pub data: Vec<u8>,
}

impl PacketRuntime for StreamChunk {
    fn allow(&self, server: bool) -> bool {
        !server
    }
}

impl PacketCodec for StreamChunk {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let id = read_i32(read)?;
        let len = read_i16(read)? as usize;
        let mut data = vec![0; len];
        read.read_exact(&mut data)?;
        Ok(Self { id, data })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.id)?;
        write_i16(write, self.data.len() as i16)?;
        write.write_all(&self.data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectPacket {
    pub version: i32,
    pub version_type: String,
    pub mods: Vec<String>,
    pub name: String,
    pub locale: String,
    pub uuid: String,
    pub usid: String,
    pub mobile: bool,
    pub color: i32,
    pub uuid_crc32: Option<u32>,
}

impl PacketRuntime for ConnectPacket {
    fn priority(&self) -> PacketPriority {
        PacketPriority::High
    }
}

impl ConnectPacket {
    pub const TYPICAL_UUID_RAW_LEN: usize = 8;
    pub const READ_UUID_RAW_LEN: usize = 16;

    pub fn uuid_bytes(&self) -> Result<Vec<u8>, base64::DecodeError> {
        base64::engine::general_purpose::STANDARD.decode(self.uuid.as_bytes())
    }

    pub fn uuid_crc32(&self) -> Result<u32, base64::DecodeError> {
        let bytes = self.uuid_bytes()?;
        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        Ok(hasher.finalize())
    }
}

impl PacketCodec for ConnectPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let version = read_i32(read)?;
        let version_type = read_string(read)?.unwrap_or_default();
        let name = read_string(read)?.unwrap_or_default();
        let locale = read_string(read)?.unwrap_or_default();
        let usid = read_string(read)?.unwrap_or_default();
        let mut uuid_bytes = vec![0; Self::READ_UUID_RAW_LEN];
        read.read_exact(&mut uuid_bytes)?;
        let uuid = base64::engine::general_purpose::STANDARD.encode(uuid_bytes);
        let mobile = read_u8(read)? == 1;
        let color = read_i32(read)?;
        let total_mods = read_u8(read)? as usize;
        let mut mods = Vec::with_capacity(total_mods);
        for _ in 0..total_mods {
            mods.push(read_string(read)?.unwrap_or_default());
        }
        Ok(Self {
            version,
            version_type,
            mods,
            name,
            locale,
            uuid,
            usid,
            mobile,
            color,
            uuid_crc32: None,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.version)?;
        write_string(write, Some(&self.version_type))?;
        write_string(write, Some(&self.name))?;
        write_string(write, Some(&self.locale))?;
        write_string(write, Some(&self.usid))?;

        let uuid_bytes = self.uuid_bytes().map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("invalid base64 uuid: {err}"),
            )
        })?;
        write.write_all(&uuid_bytes)?;
        let mut hasher = Hasher::new();
        hasher.update(&uuid_bytes);
        write_i64(write, hasher.finalize() as i64)?;

        write.write_all(&[self.mobile as u8])?;
        write_i32(write, self.color)?;
        if self.mods.len() > u8::MAX as usize {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "too many mods in ConnectPacket",
            ));
        }
        write.write_all(&[self.mods.len() as u8])?;
        for module in &self.mods {
            write_string(write, Some(module))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnounceCallPacket {
    pub message: String,
}

impl PacketCodec for AnnounceCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            message: read_string(read)?.unwrap_or_default(),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.message))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClearObjectivesCallPacket;

impl PacketCodec for ClearObjectivesCallPacket {
    fn read_from<R: Read>(_read: &mut R) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn write_to<W: Write>(&self, _write: &mut W) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientBinaryPacketCallPacket {
    pub packet_type: String,
    pub contents: Vec<u8>,
}

impl ClientBinaryPacketCallPacket {
    fn read_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let packet_type = read_string(read)?.unwrap_or_default();
        let len = read_i16(read)?;
        if len < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "negative byte array length in client binary call packet",
            ));
        }
        let mut contents = vec![0; len as usize];
        read.read_exact(&mut contents)?;
        Ok(Self {
            packet_type,
            contents,
        })
    }

    fn write_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        if self.contents.len() > i16::MAX as usize {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "client binary call packet contents exceed Java short length",
            ));
        }
        write_string(write, Some(&self.packet_type))?;
        write_i16(write, self.contents.len() as i16)?;
        write.write_all(&self.contents)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientBinaryPacketReliableCallPacket(pub ClientBinaryPacketCallPacket);

impl PacketCodec for ClientBinaryPacketReliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        ClientBinaryPacketCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientBinaryPacketUnreliableCallPacket(pub ClientBinaryPacketCallPacket);

impl PacketCodec for ClientBinaryPacketUnreliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        ClientBinaryPacketCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPacketCallPacket {
    pub packet_type: String,
    pub contents: String,
}

impl ClientPacketCallPacket {
    fn read_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            packet_type: read_string(read)?.unwrap_or_default(),
            contents: read_string(read)?.unwrap_or_default(),
        })
    }

    fn write_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.packet_type))?;
        write_string(write, Some(&self.contents))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPacketReliableCallPacket(pub ClientPacketCallPacket);

impl PacketCodec for ClientPacketReliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        ClientPacketCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPacketUnreliableCallPacket(pub ClientPacketCallPacket);

impl PacketCodec for ClientPacketUnreliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        ClientPacketCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompleteObjectiveCallPacket {
    pub index: i32,
}

impl PacketCodec for CompleteObjectiveCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            index: read_i32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.index)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectCallPacket {
    pub ip: String,
    pub port: i32,
}

impl PacketCodec for ConnectCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            ip: read_string(read)?.unwrap_or_default(),
            port: read_i32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.ip))?;
        write_i32(write, self.port)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyToClipboardCallPacket {
    pub text: String,
}

impl PacketCodec for CopyToClipboardCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            text: read_string(read)?.unwrap_or_default(),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.text))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugStatusClientCallPacket {
    pub value: i32,
    pub last_client_snapshot: i32,
    pub snapshots_sent: i32,
}

impl DebugStatusClientCallPacket {
    fn read_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            value: read_i32(read)?,
            last_client_snapshot: read_i32(read)?,
            snapshots_sent: read_i32(read)?,
        })
    }

    fn write_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.value)?;
        write_i32(write, self.last_client_snapshot)?;
        write_i32(write, self.snapshots_sent)
    }
}

impl PacketCodec for DebugStatusClientCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_payload(write)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugStatusClientUnreliableCallPacket(pub DebugStatusClientCallPacket);

impl PacketCodec for DebugStatusClientUnreliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        DebugStatusClientCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

fn read_i16<R: Read>(read: &mut R) -> std::io::Result<i16> {
    let mut b = [0; 2];
    read.read_exact(&mut b)?;
    Ok(i16::from_be_bytes(b))
}

fn write_i16<W: Write>(write: &mut W, value: i16) -> std::io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_begin_uses_java_field_order() {
        let packet = StreamBegin {
            id: 7,
            total: 300,
            packet_type: 2,
        };
        let mut bytes = Vec::new();
        packet.write_to(&mut bytes).unwrap();
        assert_eq!(bytes, vec![0, 0, 0, 7, 0, 0, 1, 44, 2]);
        let decoded = StreamBegin::read_from(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn stream_chunk_uses_java_field_order() {
        let packet = StreamChunk {
            id: 3,
            data: vec![1, 2, 3],
        };
        let mut bytes = Vec::new();
        packet.write_to(&mut bytes).unwrap();
        assert_eq!(bytes, vec![0, 0, 0, 3, 0, 3, 1, 2, 3]);
        let decoded = StreamChunk::read_from(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn connect_packet_write_matches_upstream_field_order_and_crc_slot() {
        let packet = ConnectPacket {
            version: 157,
            version_type: "official".into(),
            mods: vec!["mod-a".into(), "mod-b".into()],
            name: "p".into(),
            locale: "en_US".into(),
            uuid: base64::engine::general_purpose::STANDARD.encode([1, 2, 3, 4, 5, 6, 7, 8]),
            usid: "usid".into(),
            mobile: true,
            color: 0x11223344,
            uuid_crc32: None,
        };

        let mut bytes = Vec::new();
        packet.write_to(&mut bytes).unwrap();

        let expected_crc = packet.uuid_crc32().unwrap() as i64;
        let mut expected = Vec::new();
        expected.extend_from_slice(&157i32.to_be_bytes());
        crate::mindustry::io::write_string(&mut expected, Some("official")).unwrap();
        crate::mindustry::io::write_string(&mut expected, Some("p")).unwrap();
        crate::mindustry::io::write_string(&mut expected, Some("en_US")).unwrap();
        crate::mindustry::io::write_string(&mut expected, Some("usid")).unwrap();
        expected.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        expected.extend_from_slice(&expected_crc.to_be_bytes());
        expected.push(1);
        expected.extend_from_slice(&0x11223344i32.to_be_bytes());
        expected.push(2);
        crate::mindustry::io::write_string(&mut expected, Some("mod-a")).unwrap();
        crate::mindustry::io::write_string(&mut expected, Some("mod-b")).unwrap();

        assert_eq!(bytes, expected);
    }

    #[test]
    fn connect_packet_read_consumes_sixteen_uuid_bytes_like_java() {
        let raw_uuid = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let crc = {
            let mut hasher = Hasher::new();
            hasher.update(&raw_uuid);
            hasher.finalize()
        };
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&157i32.to_be_bytes());
        crate::mindustry::io::write_string(&mut bytes, Some("official")).unwrap();
        crate::mindustry::io::write_string(&mut bytes, Some("p")).unwrap();
        crate::mindustry::io::write_string(&mut bytes, Some("en_US")).unwrap();
        crate::mindustry::io::write_string(&mut bytes, Some("usid")).unwrap();
        bytes.extend_from_slice(&raw_uuid);
        bytes.extend_from_slice(&(crc as i64).to_be_bytes());
        bytes.push(1);
        bytes.extend_from_slice(&5i32.to_be_bytes());
        bytes.push(0);

        let decoded = ConnectPacket::read_from(&mut bytes.as_slice()).unwrap();
        let mut uuid_and_crc = Vec::from(raw_uuid);
        uuid_and_crc.extend_from_slice(&(crc as i64).to_be_bytes());
        assert_eq!(
            decoded.uuid,
            base64::engine::general_purpose::STANDARD.encode(uuid_and_crc)
        );
        assert!(decoded.mobile);
        assert_eq!(decoded.color, 5);
    }

    #[test]
    fn generated_simple_call_packets_use_typeio_field_order() {
        let announce = AnnounceCallPacket {
            message: "hello".into(),
        };
        let mut bytes = Vec::new();
        announce.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, Some("hello")).unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(
            AnnounceCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            announce
        );

        let clear = ClearObjectivesCallPacket;
        let mut bytes = Vec::new();
        clear.write_to(&mut bytes).unwrap();
        assert!(bytes.is_empty());
        assert_eq!(
            ClearObjectivesCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            clear
        );

        let complete = CompleteObjectiveCallPacket { index: 7 };
        let mut bytes = Vec::new();
        complete.write_to(&mut bytes).unwrap();
        assert_eq!(bytes, 7i32.to_be_bytes());
        assert_eq!(
            CompleteObjectiveCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            complete
        );

        let connect = ConnectCallPacket {
            ip: "127.0.0.1".into(),
            port: 6567,
        };
        let mut bytes = Vec::new();
        connect.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, Some("127.0.0.1")).unwrap();
        expected.extend_from_slice(&6567i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            ConnectCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            connect
        );

        let clipboard = CopyToClipboardCallPacket {
            text: "copy me".into(),
        };
        let mut bytes = Vec::new();
        clipboard.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, Some("copy me")).unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(
            CopyToClipboardCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            clipboard
        );
    }

    #[test]
    fn generated_client_binary_call_packets_use_typeio_bytes_order() {
        let payload = ClientBinaryPacketCallPacket {
            packet_type: "mod-channel".into(),
            contents: vec![1, 2, 3, 4],
        };
        let reliable = ClientBinaryPacketReliableCallPacket(payload.clone());
        let mut bytes = Vec::new();
        reliable.write_to(&mut bytes).unwrap();

        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, Some("mod-channel")).unwrap();
        expected.extend_from_slice(&4i16.to_be_bytes());
        expected.extend_from_slice(&[1, 2, 3, 4]);
        assert_eq!(bytes, expected);
        assert_eq!(
            ClientBinaryPacketReliableCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            reliable
        );

        let unreliable = ClientBinaryPacketUnreliableCallPacket(payload);
        let mut bytes = Vec::new();
        unreliable.write_to(&mut bytes).unwrap();
        assert_eq!(
            ClientBinaryPacketUnreliableCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            unreliable
        );
    }

    #[test]
    fn generated_client_string_call_packets_use_typeio_string_order() {
        let payload = ClientPacketCallPacket {
            packet_type: "mod-channel".into(),
            contents: "{\"ok\":true}".into(),
        };
        let reliable = ClientPacketReliableCallPacket(payload.clone());
        let mut bytes = Vec::new();
        reliable.write_to(&mut bytes).unwrap();

        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, Some("mod-channel")).unwrap();
        crate::mindustry::io::write_string(&mut expected, Some("{\"ok\":true}")).unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(
            ClientPacketReliableCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            reliable
        );

        let unreliable = ClientPacketUnreliableCallPacket(payload);
        let mut bytes = Vec::new();
        unreliable.write_to(&mut bytes).unwrap();
        assert_eq!(
            ClientPacketUnreliableCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            unreliable
        );
    }

    #[test]
    fn generated_debug_status_call_packets_use_java_int_order() {
        let packet = DebugStatusClientCallPacket {
            value: 1,
            last_client_snapshot: 2,
            snapshots_sent: 3,
        };
        let mut bytes = Vec::new();
        packet.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1i32.to_be_bytes());
        expected.extend_from_slice(&2i32.to_be_bytes());
        expected.extend_from_slice(&3i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            DebugStatusClientCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            packet
        );

        let unreliable = DebugStatusClientUnreliableCallPacket(packet);
        let mut bytes = Vec::new();
        unreliable.write_to(&mut bytes).unwrap();
        assert_eq!(
            DebugStatusClientUnreliableCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            unreliable
        );
    }

    #[test]
    fn registered_manifest_keeps_upstream_base_packet_ids_stable() {
        let ids: Vec<_> = registered_packet_manifest()
            .iter()
            .map(|entry| (entry.id.unwrap(), entry.name))
            .collect();

        assert_eq!(
            ids,
            vec![
                (packet_ids::STREAM_BEGIN, "StreamBegin"),
                (packet_ids::STREAM_CHUNK, "StreamChunk"),
                (packet_ids::WORLD_STREAM, "WorldStream"),
                (packet_ids::CONNECT_PACKET, "ConnectPacket"),
                (packet_ids::ANNOUNCE_CALL_PACKET, "AnnounceCallPacket"),
                (
                    packet_ids::CLEAR_OBJECTIVES_CALL_PACKET,
                    "ClearObjectivesCallPacket",
                ),
                (
                    packet_ids::CLIENT_BINARY_PACKET_RELIABLE_CALL_PACKET,
                    "ClientBinaryPacketReliableCallPacket",
                ),
                (
                    packet_ids::CLIENT_BINARY_PACKET_UNRELIABLE_CALL_PACKET,
                    "ClientBinaryPacketUnreliableCallPacket",
                ),
                (
                    packet_ids::CLIENT_PACKET_RELIABLE_CALL_PACKET,
                    "ClientPacketReliableCallPacket",
                ),
                (
                    packet_ids::CLIENT_PACKET_UNRELIABLE_CALL_PACKET,
                    "ClientPacketUnreliableCallPacket",
                ),
                (
                    packet_ids::COMPLETE_OBJECTIVE_CALL_PACKET,
                    "CompleteObjectiveCallPacket",
                ),
                (packet_ids::CONNECT_CALL_PACKET, "ConnectCallPacket"),
                (
                    packet_ids::COPY_TO_CLIPBOARD_CALL_PACKET,
                    "CopyToClipboardCallPacket",
                ),
                (
                    packet_ids::DEBUG_STATUS_CLIENT_CALL_PACKET,
                    "DebugStatusClientCallPacket",
                ),
                (
                    packet_ids::DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET,
                    "DebugStatusClientUnreliableCallPacket",
                ),
            ]
        );
        assert_eq!(find_registered_packet_by_id(0).unwrap().name, "StreamBegin");
        assert_eq!(find_registered_packet_by_id(1).unwrap().name, "StreamChunk");
        assert_eq!(find_registered_packet_by_id(2).unwrap().name, "WorldStream");
        assert_eq!(
            find_registered_packet_by_id(3).unwrap().name,
            "ConnectPacket"
        );
        assert!(find_registered_packet_by_id(4).is_none());
        assert_eq!(
            find_registered_packet_by_id(packet_ids::ANNOUNCE_CALL_PACKET)
                .unwrap()
                .name,
            "AnnounceCallPacket"
        );
    }

    #[test]
    fn manifest_supports_lookup_by_name_and_transport_id() {
        let connect = find_packet_by_name("ConnectPacket").unwrap();
        assert_eq!(connect.id, Some(packet_ids::CONNECT_PACKET));
        assert_eq!(connect.direction, PacketDirection::ClientToServer);
        assert_eq!(connect.priority, Some(PacketPriority::High));
        assert_eq!(connect.codec, PacketCodecState::Implemented);

        let world_stream = find_packet_by_name("WorldStream").unwrap();
        assert_eq!(world_stream.id, Some(packet_ids::WORLD_STREAM));
        assert!(world_stream.streamable);
        assert_eq!(world_stream.codec, PacketCodecState::StreamPayload);

        let ping = find_packet_by_transport_id(
            PacketTransport::FrameworkMessage,
            framework_message_ids::PING,
        )
        .unwrap();
        assert_eq!(ping.name, "Ping");
        assert_eq!(ping.direction, PacketDirection::Bidirectional);

        assert!(find_packet_by_name("Pong").is_none());
        assert!(packet_manifest_phase1_gaps()
            .iter()
            .any(|gap| gap.contains("Pong")));
    }

    #[test]
    fn implemented_codecs_are_present_in_manifest() {
        for name in [
            "StreamBegin",
            "StreamChunk",
            "ConnectPacket",
            "AnnounceCallPacket",
            "ClearObjectivesCallPacket",
            "ClientBinaryPacketReliableCallPacket",
            "ClientBinaryPacketUnreliableCallPacket",
            "ClientPacketReliableCallPacket",
            "ClientPacketUnreliableCallPacket",
            "CompleteObjectiveCallPacket",
            "ConnectCallPacket",
            "CopyToClipboardCallPacket",
            "DebugStatusClientCallPacket",
            "DebugStatusClientUnreliableCallPacket",
            "ServerData",
            "Ping",
        ] {
            let entry = find_packet_by_name(name).unwrap_or_else(|| panic!("{name} missing"));
            assert_eq!(
                entry.codec,
                PacketCodecState::Implemented,
                "{name} should be marked as implemented"
            );
        }

        assert!(find_packet_by_name("Connect").is_some());
        assert!(find_packet_by_name("Disconnect").is_some());
        assert!(find_packet_by_name("InvokePacket").is_none());
        assert!(packet_manifest_phase1_gaps()
            .iter()
            .any(|gap| gap.contains("Call.registerPackets()")));
    }
}
