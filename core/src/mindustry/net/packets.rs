use base64::Engine;
use crc32fast::Hasher;
use std::io::{Read, Write};

use crate::mindustry::core::content_loader::ContentLoader;
use crate::mindustry::ctype::ContentId;
use crate::mindustry::io::type_io::{
    read_block, read_bullet_type_id, read_bytes, read_client_plans, read_i32, read_i64, read_kick,
    read_object_safe, read_plans_queue, read_string, read_team, read_tile_pos, read_u32, read_u8,
    read_unit_ref, write_block, write_bullet_type_id, write_bytes, write_client_plans, write_i32,
    write_i64, write_kick, write_object, write_plans_queue_net, write_string, write_team,
    write_tile_pos, write_u32, write_unit_ref, BuildPlanWire, TeamId, TypeValue, UnitRef,
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
    pub const CLIENT_PLAN_SNAPSHOT_CALL_PACKET: PacketId = 24;
    pub const CLIENT_PLAN_SNAPSHOT_RECEIVED_CALL_PACKET: PacketId = 25;
    pub const CLIENT_SNAPSHOT_CALL_PACKET: PacketId = 26;
    pub const COMPLETE_OBJECTIVE_CALL_PACKET: PacketId = 29;
    pub const CONNECT_CALL_PACKET: PacketId = 30;
    pub const CONNECT_CONFIRM_CALL_PACKET: PacketId = 31;
    pub const CONSTRUCT_FINISH_CALL_PACKET: PacketId = 32;
    pub const COPY_TO_CLIPBOARD_CALL_PACKET: PacketId = 33;
    pub const CREATE_BULLET_CALL_PACKET: PacketId = 34;
    pub const DEBUG_STATUS_CLIENT_CALL_PACKET: PacketId = 37;
    pub const DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET: PacketId = 38;
    pub const HIDE_FOLLOW_UP_MENU_CALL_PACKET: PacketId = 50;
    pub const HIDE_HUD_TEXT_CALL_PACKET: PacketId = 51;
    pub const INFO_MESSAGE_CALL_PACKET: PacketId = 52;
    pub const INFO_POPUP_CALL_PACKET: PacketId = 53;
    pub const INFO_POPUP_CALL_PACKET2: PacketId = 54;
    pub const INFO_POPUP_RELIABLE_CALL_PACKET: PacketId = 55;
    pub const INFO_POPUP_RELIABLE_CALL_PACKET2: PacketId = 56;
    pub const INFO_TOAST_CALL_PACKET: PacketId = 57;
    pub const KICK_CALL_PACKET: PacketId = 58;
    pub const KICK_CALL_PACKET2: PacketId = 59;
    pub const LABEL_CALL_PACKET: PacketId = 60;
    pub const LABEL_CALL_PACKET2: PacketId = 61;
    pub const LABEL_RELIABLE_CALL_PACKET: PacketId = 62;
    pub const LABEL_RELIABLE_CALL_PACKET2: PacketId = 63;
    pub const OPEN_URI_CALL_PACKET: PacketId = 68;
    pub const PING_RESPONSE_CALL_PACKET: PacketId = 74;
    pub const PLAYER_DISCONNECT_CALL_PACKET: PacketId = 75;
    pub const REMOVE_MARKER_CALL_PACKET: PacketId = 77;
    pub const REMOVE_QUEUE_BLOCK_CALL_PACKET: PacketId = 78;
    pub const SET_CAMERA_POSITION_CALL_PACKET: PacketId = 97;
    pub const SET_FLAG_CALL_PACKET: PacketId = 98;
    pub const SET_HUD_TEXT_CALL_PACKET: PacketId = 100;
    pub const SET_HUD_TEXT_RELIABLE_CALL_PACKET: PacketId = 101;
    pub const SET_MAP_AREA_CALL_PACKET: PacketId = 106;
    pub const SET_RULE_CALL_PACKET: PacketId = 111;
    pub const TEXT_INPUT_CALL_PACKET: PacketId = 129;
    pub const TEXT_INPUT_CALL_PACKET2: PacketId = 130;
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

pub const CLIENT_PLAN_SNAPSHOT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::CLIENT_PLAN_SNAPSHOT_CALL_PACKET),
    name: "ClientPlanSnapshotCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ClientToServer,
    streamable: false,
    priority: Some(PacketPriority::Low),
    allow_client_endpoint: false,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.ClientPlanSnapshotCallPacket",
    notes: "Generated by Call.registerPackets(); client-to-server preview build plan snapshot using TypeIO ClientBuildPlans.",
};

pub const CLIENT_PLAN_SNAPSHOT_RECEIVED_CALL_PACKET_MANIFEST: PacketManifestEntry =
    PacketManifestEntry {
        id: Some(packet_ids::CLIENT_PLAN_SNAPSHOT_RECEIVED_CALL_PACKET),
        name: "ClientPlanSnapshotReceivedCallPacket",
        transport: PacketTransport::NetPacket,
        direction: PacketDirection::ServerToClient,
        streamable: false,
        priority: Some(PacketPriority::Low),
        allow_client_endpoint: true,
        allow_server_endpoint: false,
        force_uncompressed: false,
        codec: PacketCodecState::Implemented,
        upstream: "mindustry.gen.ClientPlanSnapshotReceivedCallPacket",
        notes: "Generated by Call.registerPackets(); server-to-client forwarded preview plans includes player entity id plus group id and ClientBuildPlans.",
    };

pub const CLIENT_SNAPSHOT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::CLIENT_SNAPSHOT_CALL_PACKET),
    name: "ClientSnapshotCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ClientToServer,
    streamable: false,
    priority: Some(PacketPriority::High),
    allow_client_endpoint: false,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.ClientSnapshotCallPacket",
    notes: "Generated by Call.registerPackets(); client movement/build-state snapshot, including Tile, Block and Queue<BuildPlan> TypeIO fields.",
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

pub const CONNECT_CONFIRM_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::CONNECT_CONFIRM_CALL_PACKET),
    name: "ConnectConfirmCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ClientToServer,
    streamable: false,
    priority: Some(PacketPriority::High),
    allow_client_endpoint: false,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.ConnectConfirmCallPacket",
    notes: "Generated by Call.registerPackets(); empty client-to-server connection confirmation packet.",
};

pub const CONSTRUCT_FINISH_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::CONSTRUCT_FINISH_CALL_PACKET),
    name: "ConstructFinishCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.ConstructFinishCallPacket",
    notes: "Generated by Call.registerPackets(); server-to-client construct completion with Tile, Block, Unit, Team and Object TypeIO fields.",
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

pub const CREATE_BULLET_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::CREATE_BULLET_CALL_PACKET),
    name: "CreateBulletCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.CreateBulletCallPacket",
    notes:
        "Generated by Call.registerPackets(); BulletType id, Team byte and six Java float fields.",
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

pub const HIDE_FOLLOW_UP_MENU_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::HIDE_FOLLOW_UP_MENU_CALL_PACKET),
    name: "HideFollowUpMenuCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.HideFollowUpMenuCallPacket",
    notes: "Generated by Call.registerPackets(); single Java int menu id.",
};

pub const HIDE_HUD_TEXT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::HIDE_HUD_TEXT_CALL_PACKET),
    name: "HideHudTextCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.HideHudTextCallPacket",
    notes: "Generated by Call.registerPackets(); no fields.",
};

pub const INFO_MESSAGE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::INFO_MESSAGE_CALL_PACKET),
    name: "InfoMessageCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.InfoMessageCallPacket",
    notes: "Generated by Call.registerPackets(); single TypeIO string message.",
};

pub const INFO_POPUP_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::INFO_POPUP_CALL_PACKET),
    name: "InfoPopupCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.InfoPopupCallPacket",
    notes:
        "Generated by Call.registerPackets(); nullable message plus float/int popup layout payload.",
};

pub const INFO_POPUP_CALL_PACKET2_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::INFO_POPUP_CALL_PACKET2),
    name: "InfoPopupCallPacket2",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.InfoPopupCallPacket2",
    notes: "Generated by Call.registerPackets(); nullable message/id plus float/int popup layout payload.",
};

pub const INFO_POPUP_RELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::INFO_POPUP_RELIABLE_CALL_PACKET),
    name: "InfoPopupReliableCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.InfoPopupReliableCallPacket",
    notes: "Generated by Call.registerPackets(); reliable overload without popup id.",
};

pub const INFO_POPUP_RELIABLE_CALL_PACKET2_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::INFO_POPUP_RELIABLE_CALL_PACKET2),
    name: "InfoPopupReliableCallPacket2",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.InfoPopupReliableCallPacket2",
    notes: "Generated by Call.registerPackets(); reliable overload with nullable popup id.",
};

pub const INFO_TOAST_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::INFO_TOAST_CALL_PACKET),
    name: "InfoToastCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.InfoToastCallPacket",
    notes: "Generated by Call.registerPackets(); TypeIO string plus Java float duration.",
};

pub const KICK_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::KICK_CALL_PACKET),
    name: "KickCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::High),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.KickCallPacket",
    notes: "Generated by Call.registerPackets(); string kick reason overload is first after Java method sorting.",
};

pub const KICK_CALL_PACKET2_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::KICK_CALL_PACKET2),
    name: "KickCallPacket2",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::High),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.KickCallPacket2",
    notes: "Generated by Call.registerPackets(); KickReason enum overload serialized as one ordinal byte.",
};

pub const LABEL_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::LABEL_CALL_PACKET),
    name: "LabelCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.LabelCallPacket",
    notes: "Generated by Call.registerPackets(); nullable message plus duration/world coordinates.",
};

pub const LABEL_CALL_PACKET2_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::LABEL_CALL_PACKET2),
    name: "LabelCallPacket2",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.LabelCallPacket2",
    notes: "Generated by Call.registerPackets(); nullable message, Java int label id and duration/world coordinates.",
};

pub const LABEL_RELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::LABEL_RELIABLE_CALL_PACKET),
    name: "LabelReliableCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.LabelReliableCallPacket",
    notes: "Generated by Call.registerPackets(); reliable label overload without id.",
};

pub const LABEL_RELIABLE_CALL_PACKET2_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::LABEL_RELIABLE_CALL_PACKET2),
    name: "LabelReliableCallPacket2",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.LabelReliableCallPacket2",
    notes: "Generated by Call.registerPackets(); reliable label overload with id.",
};

pub const OPEN_URI_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::OPEN_URI_CALL_PACKET),
    name: "OpenURICallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.OpenURICallPacket",
    notes: "Generated by Call.registerPackets(); single TypeIO string uri.",
};

pub const PING_RESPONSE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::PING_RESPONSE_CALL_PACKET),
    name: "PingResponseCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.PingResponseCallPacket",
    notes: "Generated by Call.registerPackets(); single Java long time.",
};

pub const PLAYER_DISCONNECT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::PLAYER_DISCONNECT_CALL_PACKET),
    name: "PlayerDisconnectCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.PlayerDisconnectCallPacket",
    notes: "Generated by Call.registerPackets(); single Java int player id.",
};

pub const REMOVE_MARKER_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::REMOVE_MARKER_CALL_PACKET),
    name: "RemoveMarkerCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.RemoveMarkerCallPacket",
    notes: "Generated by Call.registerPackets(); single Java int marker id.",
};

pub const REMOVE_QUEUE_BLOCK_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::REMOVE_QUEUE_BLOCK_CALL_PACKET),
    name: "RemoveQueueBlockCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.RemoveQueueBlockCallPacket",
    notes: "Generated by Call.registerPackets(); Java int x/y plus boolean breaking.",
};

pub const SET_CAMERA_POSITION_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_CAMERA_POSITION_CALL_PACKET),
    name: "SetCameraPositionCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetCameraPositionCallPacket",
    notes: "Generated by Call.registerPackets(); two Java float fields.",
};

pub const SET_FLAG_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_FLAG_CALL_PACKET),
    name: "SetFlagCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetFlagCallPacket",
    notes: "Generated by Call.registerPackets(); TypeIO string flag plus Java boolean add.",
};

pub const SET_HUD_TEXT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_HUD_TEXT_CALL_PACKET),
    name: "SetHudTextCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetHudTextCallPacket",
    notes: "Generated by Call.registerPackets(); unreliable TypeIO string HUD message.",
};

pub const SET_HUD_TEXT_RELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_HUD_TEXT_RELIABLE_CALL_PACKET),
    name: "SetHudTextReliableCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetHudTextReliableCallPacket",
    notes: "Generated by Call.registerPackets(); reliable TypeIO string HUD message.",
};

pub const SET_MAP_AREA_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_MAP_AREA_CALL_PACKET),
    name: "SetMapAreaCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetMapAreaCallPacket",
    notes: "Generated by Call.registerPackets(); four Java int map-area fields.",
};

pub const SET_RULE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_RULE_CALL_PACKET),
    name: "SetRuleCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetRuleCallPacket",
    notes: "Generated by Call.registerPackets(); two TypeIO string fields: rule and jsonData.",
};

pub const TEXT_INPUT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::TEXT_INPUT_CALL_PACKET),
    name: "TextInputCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.TextInputCallPacket",
    notes: "Generated by Call.registerPackets(); text input prompt without allowEmpty flag.",
};

pub const TEXT_INPUT_CALL_PACKET2_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::TEXT_INPUT_CALL_PACKET2),
    name: "TextInputCallPacket2",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.TextInputCallPacket2",
    notes: "Generated by Call.registerPackets(); text input prompt plus allowEmpty flag.",
};

pub const WORLD_DATA_BEGIN_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::WORLD_DATA_BEGIN_CALL_PACKET),
    name: "WorldDataBeginCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.WorldDataBeginCallPacket",
    notes: "Generated by Call.registerPackets(); no fields.",
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

pub const REGISTERED_PACKET_MANIFEST: [PacketManifestEntry; 49] = [
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
    CLIENT_PLAN_SNAPSHOT_CALL_PACKET_MANIFEST,
    CLIENT_PLAN_SNAPSHOT_RECEIVED_CALL_PACKET_MANIFEST,
    CLIENT_SNAPSHOT_CALL_PACKET_MANIFEST,
    COMPLETE_OBJECTIVE_CALL_PACKET_MANIFEST,
    CONNECT_CALL_PACKET_MANIFEST,
    CONNECT_CONFIRM_CALL_PACKET_MANIFEST,
    CONSTRUCT_FINISH_CALL_PACKET_MANIFEST,
    COPY_TO_CLIPBOARD_CALL_PACKET_MANIFEST,
    CREATE_BULLET_CALL_PACKET_MANIFEST,
    DEBUG_STATUS_CLIENT_CALL_PACKET_MANIFEST,
    DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET_MANIFEST,
    HIDE_FOLLOW_UP_MENU_CALL_PACKET_MANIFEST,
    HIDE_HUD_TEXT_CALL_PACKET_MANIFEST,
    INFO_MESSAGE_CALL_PACKET_MANIFEST,
    INFO_POPUP_CALL_PACKET_MANIFEST,
    INFO_POPUP_CALL_PACKET2_MANIFEST,
    INFO_POPUP_RELIABLE_CALL_PACKET_MANIFEST,
    INFO_POPUP_RELIABLE_CALL_PACKET2_MANIFEST,
    INFO_TOAST_CALL_PACKET_MANIFEST,
    KICK_CALL_PACKET_MANIFEST,
    KICK_CALL_PACKET2_MANIFEST,
    LABEL_CALL_PACKET_MANIFEST,
    LABEL_CALL_PACKET2_MANIFEST,
    LABEL_RELIABLE_CALL_PACKET_MANIFEST,
    LABEL_RELIABLE_CALL_PACKET2_MANIFEST,
    OPEN_URI_CALL_PACKET_MANIFEST,
    PING_RESPONSE_CALL_PACKET_MANIFEST,
    PLAYER_DISCONNECT_CALL_PACKET_MANIFEST,
    REMOVE_MARKER_CALL_PACKET_MANIFEST,
    REMOVE_QUEUE_BLOCK_CALL_PACKET_MANIFEST,
    SET_CAMERA_POSITION_CALL_PACKET_MANIFEST,
    SET_FLAG_CALL_PACKET_MANIFEST,
    SET_HUD_TEXT_CALL_PACKET_MANIFEST,
    SET_HUD_TEXT_RELIABLE_CALL_PACKET_MANIFEST,
    SET_MAP_AREA_CALL_PACKET_MANIFEST,
    SET_RULE_CALL_PACKET_MANIFEST,
    TEXT_INPUT_CALL_PACKET_MANIFEST,
    TEXT_INPUT_CALL_PACKET2_MANIFEST,
    WORLD_DATA_BEGIN_CALL_PACKET_MANIFEST,
];

pub const PACKET_MANIFEST: [PacketManifestEntry; 53] = [
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
    CLIENT_PLAN_SNAPSHOT_CALL_PACKET_MANIFEST,
    CLIENT_PLAN_SNAPSHOT_RECEIVED_CALL_PACKET_MANIFEST,
    CLIENT_SNAPSHOT_CALL_PACKET_MANIFEST,
    COMPLETE_OBJECTIVE_CALL_PACKET_MANIFEST,
    CONNECT_CALL_PACKET_MANIFEST,
    CONNECT_CONFIRM_CALL_PACKET_MANIFEST,
    CONSTRUCT_FINISH_CALL_PACKET_MANIFEST,
    COPY_TO_CLIPBOARD_CALL_PACKET_MANIFEST,
    CREATE_BULLET_CALL_PACKET_MANIFEST,
    DEBUG_STATUS_CLIENT_CALL_PACKET_MANIFEST,
    DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET_MANIFEST,
    HIDE_FOLLOW_UP_MENU_CALL_PACKET_MANIFEST,
    HIDE_HUD_TEXT_CALL_PACKET_MANIFEST,
    INFO_MESSAGE_CALL_PACKET_MANIFEST,
    INFO_POPUP_CALL_PACKET_MANIFEST,
    INFO_POPUP_CALL_PACKET2_MANIFEST,
    INFO_POPUP_RELIABLE_CALL_PACKET_MANIFEST,
    INFO_POPUP_RELIABLE_CALL_PACKET2_MANIFEST,
    INFO_TOAST_CALL_PACKET_MANIFEST,
    KICK_CALL_PACKET_MANIFEST,
    KICK_CALL_PACKET2_MANIFEST,
    LABEL_CALL_PACKET_MANIFEST,
    LABEL_CALL_PACKET2_MANIFEST,
    LABEL_RELIABLE_CALL_PACKET_MANIFEST,
    LABEL_RELIABLE_CALL_PACKET2_MANIFEST,
    OPEN_URI_CALL_PACKET_MANIFEST,
    PING_RESPONSE_CALL_PACKET_MANIFEST,
    PLAYER_DISCONNECT_CALL_PACKET_MANIFEST,
    REMOVE_MARKER_CALL_PACKET_MANIFEST,
    REMOVE_QUEUE_BLOCK_CALL_PACKET_MANIFEST,
    SET_CAMERA_POSITION_CALL_PACKET_MANIFEST,
    SET_FLAG_CALL_PACKET_MANIFEST,
    SET_HUD_TEXT_CALL_PACKET_MANIFEST,
    SET_HUD_TEXT_RELIABLE_CALL_PACKET_MANIFEST,
    SET_MAP_AREA_CALL_PACKET_MANIFEST,
    SET_RULE_CALL_PACKET_MANIFEST,
    TEXT_INPUT_CALL_PACKET_MANIFEST,
    TEXT_INPUT_CALL_PACKET2_MANIFEST,
    WORLD_DATA_BEGIN_CALL_PACKET_MANIFEST,
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

    pub const fn ordinal(self) -> u8 {
        match self {
            KickReason::Kick => 0,
            KickReason::ClientOutdated => 1,
            KickReason::ServerOutdated => 2,
            KickReason::Banned => 3,
            KickReason::Gameover => 4,
            KickReason::RecentKick => 5,
            KickReason::NameInUse => 6,
            KickReason::IdInUse => 7,
            KickReason::NameEmpty => 8,
            KickReason::CustomClient => 9,
            KickReason::ServerClose => 10,
            KickReason::Vote => 11,
            KickReason::TypeMismatch => 12,
            KickReason::Whitelist => 13,
            KickReason::PlayerLimit => 14,
            KickReason::ServerRestarting => 15,
        }
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
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

    pub const fn ordinal(self) -> u8 {
        match self {
            AdminAction::Kick => 0,
            AdminAction::Ban => 1,
            AdminAction::Trace => 2,
            AdminAction::Wave => 3,
            AdminAction::SwitchTeam => 4,
        }
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub const fn wire_name(self) -> &'static str {
        match self {
            AdminAction::Kick => "kick",
            AdminAction::Ban => "ban",
            AdminAction::Trace => "trace",
            AdminAction::Wave => "wave",
            AdminAction::SwitchTeam => "switchTeam",
        }
    }
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
        let contents = read_bytes(read)?;
        Ok(Self {
            packet_type,
            contents,
        })
    }

    fn write_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.packet_type))?;
        write_bytes(write, &self.contents)
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

#[derive(Debug, Clone, PartialEq)]
pub struct ClientPlanSnapshotCallPacket {
    pub group_id: i32,
    pub plans: Option<Vec<BuildPlanWire>>,
}

impl ClientPlanSnapshotCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            group_id: read_i32(read)?,
            plans: read_client_plans(read, loader)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_i32(write, self.group_id)?;
        write_client_plans(write, loader, self.plans.as_deref())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientPlanSnapshotReceivedCallPacket {
    pub player_id: i32,
    pub group_id: i32,
    pub plans: Option<Vec<BuildPlanWire>>,
}

impl ClientPlanSnapshotReceivedCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            player_id: read_i32(read)?,
            group_id: read_i32(read)?,
            plans: read_client_plans(read, loader)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_i32(write, self.player_id)?;
        write_i32(write, self.group_id)?;
        write_client_plans(write, loader, self.plans.as_deref())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientSnapshotCallPacket {
    pub snapshot_id: i32,
    pub unit_id: i32,
    pub dead: bool,
    pub x: f32,
    pub y: f32,
    pub pointer_x: f32,
    pub pointer_y: f32,
    pub rotation: f32,
    pub base_rotation: f32,
    pub x_velocity: f32,
    pub y_velocity: f32,
    pub mining: Option<i32>,
    pub boosting: bool,
    pub shooting: bool,
    pub chatting: bool,
    pub building: bool,
    pub selected_block: Option<String>,
    pub selected_rotation: i32,
    pub plans: Option<Vec<BuildPlanWire>>,
    pub view_x: f32,
    pub view_y: f32,
    pub view_width: f32,
    pub view_height: f32,
}

impl ClientSnapshotCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            snapshot_id: read_i32(read)?,
            unit_id: read_i32(read)?,
            dead: read_bool(read)?,
            x: read_f32(read)?,
            y: read_f32(read)?,
            pointer_x: read_f32(read)?,
            pointer_y: read_f32(read)?,
            rotation: read_f32(read)?,
            base_rotation: read_f32(read)?,
            x_velocity: read_f32(read)?,
            y_velocity: read_f32(read)?,
            mining: read_tile_pos(read)?,
            boosting: read_bool(read)?,
            shooting: read_bool(read)?,
            chatting: read_bool(read)?,
            building: read_bool(read)?,
            selected_block: read_block(read, loader)?,
            selected_rotation: read_i32(read)?,
            plans: read_plans_queue(read, loader)?,
            view_x: read_f32(read)?,
            view_y: read_f32(read)?,
            view_width: read_f32(read)?,
            view_height: read_f32(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_i32(write, self.snapshot_id)?;
        write_i32(write, self.unit_id)?;
        write_bool(write, self.dead)?;
        write_f32(write, self.x)?;
        write_f32(write, self.y)?;
        write_f32(write, self.pointer_x)?;
        write_f32(write, self.pointer_y)?;
        write_f32(write, self.rotation)?;
        write_f32(write, self.base_rotation)?;
        write_f32(write, self.x_velocity)?;
        write_f32(write, self.y_velocity)?;
        write_tile_pos(write, self.mining)?;
        write_bool(write, self.boosting)?;
        write_bool(write, self.shooting)?;
        write_bool(write, self.chatting)?;
        write_bool(write, self.building)?;
        write_block(write, loader, self.selected_block.as_deref())?;
        write_i32(write, self.selected_rotation)?;
        write_plans_queue_net(write, loader, self.plans.as_deref())?;
        write_f32(write, self.view_x)?;
        write_f32(write, self.view_y)?;
        write_f32(write, self.view_width)?;
        write_f32(write, self.view_height)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectConfirmCallPacket;

impl PacketCodec for ConnectConfirmCallPacket {
    fn read_from<R: Read>(_read: &mut R) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn write_to<W: Write>(&self, _write: &mut W) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstructFinishCallPacket {
    pub tile: Option<i32>,
    pub block: Option<String>,
    pub builder: UnitRef,
    pub rotation: u8,
    pub team: TeamId,
    pub config: TypeValue,
}

impl ConstructFinishCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            tile: read_tile_pos(read)?,
            block: read_block(read, loader)?,
            builder: read_unit_ref(read)?,
            rotation: read_u8(read)?,
            team: read_team(read)?,
            config: read_object_safe(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_tile_pos(write, self.tile)?;
        write_block(write, loader, self.block.as_deref())?;
        write_unit_ref(write, self.builder)?;
        write.write_all(&[self.rotation])?;
        write_team(write, Some(self.team))?;
        write_object(write, &self.config)
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreateBulletCallPacket {
    pub bullet_type_id: ContentId,
    pub team: TeamId,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub damage: f32,
    pub velocity_scl: f32,
    pub lifetime_scl: f32,
}

impl PacketCodec for CreateBulletCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            bullet_type_id: read_bullet_type_id(read)?,
            team: read_team(read)?,
            x: read_f32(read)?,
            y: read_f32(read)?,
            angle: read_f32(read)?,
            damage: read_f32(read)?,
            velocity_scl: read_f32(read)?,
            lifetime_scl: read_f32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_bullet_type_id(write, self.bullet_type_id)?;
        write_team(write, Some(self.team))?;
        write_f32(write, self.x)?;
        write_f32(write, self.y)?;
        write_f32(write, self.angle)?;
        write_f32(write, self.damage)?;
        write_f32(write, self.velocity_scl)?;
        write_f32(write, self.lifetime_scl)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HideFollowUpMenuCallPacket {
    pub menu_id: i32,
}

impl PacketCodec for HideFollowUpMenuCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            menu_id: read_i32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.menu_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HideHudTextCallPacket;

impl PacketCodec for HideHudTextCallPacket {
    fn read_from<R: Read>(_read: &mut R) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn write_to<W: Write>(&self, _write: &mut W) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoMessageCallPacket {
    pub message: String,
}

impl PacketCodec for InfoMessageCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            message: read_string(read)?.unwrap_or_default(),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.message))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfoPopupCallPacket {
    pub message: Option<String>,
    pub duration: f32,
    pub align: i32,
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}

impl InfoPopupCallPacket {
    fn read_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            message: read_string(read)?,
            duration: read_f32(read)?,
            align: read_i32(read)?,
            top: read_i32(read)?,
            left: read_i32(read)?,
            bottom: read_i32(read)?,
            right: read_i32(read)?,
        })
    }

    fn write_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, self.message.as_deref())?;
        write_f32(write, self.duration)?;
        write_i32(write, self.align)?;
        write_i32(write, self.top)?;
        write_i32(write, self.left)?;
        write_i32(write, self.bottom)?;
        write_i32(write, self.right)
    }
}

impl PacketCodec for InfoPopupCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfoPopupCallPacket2 {
    pub message: Option<String>,
    pub id: Option<String>,
    pub duration: f32,
    pub align: i32,
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}

impl InfoPopupCallPacket2 {
    fn read_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            message: read_string(read)?,
            id: read_string(read)?,
            duration: read_f32(read)?,
            align: read_i32(read)?,
            top: read_i32(read)?,
            left: read_i32(read)?,
            bottom: read_i32(read)?,
            right: read_i32(read)?,
        })
    }

    fn write_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, self.message.as_deref())?;
        write_string(write, self.id.as_deref())?;
        write_f32(write, self.duration)?;
        write_i32(write, self.align)?;
        write_i32(write, self.top)?;
        write_i32(write, self.left)?;
        write_i32(write, self.bottom)?;
        write_i32(write, self.right)
    }
}

impl PacketCodec for InfoPopupCallPacket2 {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfoPopupReliableCallPacket(pub InfoPopupCallPacket);

impl PacketCodec for InfoPopupReliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        InfoPopupCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfoPopupReliableCallPacket2(pub InfoPopupCallPacket2);

impl PacketCodec for InfoPopupReliableCallPacket2 {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        InfoPopupCallPacket2::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfoToastCallPacket {
    pub message: String,
    pub duration: f32,
}

impl PacketCodec for InfoToastCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            message: read_string(read)?.unwrap_or_default(),
            duration: f32::from_bits(read_u32(read)?),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.message))?;
        write_u32(write, self.duration.to_bits())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KickCallPacket {
    pub reason: String,
}

impl PacketCodec for KickCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            reason: read_string(read)?.unwrap_or_default(),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.reason))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KickCallPacket2 {
    pub reason: KickReason,
}

impl PacketCodec for KickCallPacket2 {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            reason: read_kick(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_kick(write, self.reason)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelCallPacket {
    pub message: Option<String>,
    pub duration: f32,
    pub world_x: f32,
    pub world_y: f32,
}

impl LabelCallPacket {
    fn read_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            message: read_string(read)?,
            duration: read_f32(read)?,
            world_x: read_f32(read)?,
            world_y: read_f32(read)?,
        })
    }

    fn write_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, self.message.as_deref())?;
        write_f32(write, self.duration)?;
        write_f32(write, self.world_x)?;
        write_f32(write, self.world_y)
    }
}

impl PacketCodec for LabelCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelCallPacket2 {
    pub message: Option<String>,
    pub id: i32,
    pub duration: f32,
    pub world_x: f32,
    pub world_y: f32,
}

impl LabelCallPacket2 {
    fn read_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            message: read_string(read)?,
            id: read_i32(read)?,
            duration: read_f32(read)?,
            world_x: read_f32(read)?,
            world_y: read_f32(read)?,
        })
    }

    fn write_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, self.message.as_deref())?;
        write_i32(write, self.id)?;
        write_f32(write, self.duration)?;
        write_f32(write, self.world_x)?;
        write_f32(write, self.world_y)
    }
}

impl PacketCodec for LabelCallPacket2 {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelReliableCallPacket(pub LabelCallPacket);

impl PacketCodec for LabelReliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        LabelCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelReliableCallPacket2(pub LabelCallPacket2);

impl PacketCodec for LabelReliableCallPacket2 {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        LabelCallPacket2::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenUriCallPacket {
    pub uri: String,
}

impl PacketCodec for OpenUriCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            uri: read_string(read)?.unwrap_or_default(),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.uri))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PingResponseCallPacket {
    pub time: i64,
}

impl PacketCodec for PingResponseCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            time: read_i64(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i64(write, self.time)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerDisconnectCallPacket {
    pub player_id: i32,
}

impl PacketCodec for PlayerDisconnectCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player_id: read_i32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.player_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveMarkerCallPacket {
    pub id: i32,
}

impl PacketCodec for RemoveMarkerCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            id: read_i32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveQueueBlockCallPacket {
    pub x: i32,
    pub y: i32,
    pub breaking: bool,
}

impl PacketCodec for RemoveQueueBlockCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            x: read_i32(read)?,
            y: read_i32(read)?,
            breaking: read_u8(read)? != 0,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.x)?;
        write_i32(write, self.y)?;
        write.write_all(&[self.breaking as u8])
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetCameraPositionCallPacket {
    pub x: f32,
    pub y: f32,
}

impl PacketCodec for SetCameraPositionCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            x: read_f32(read)?,
            y: read_f32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_f32(write, self.x)?;
        write_f32(write, self.y)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetFlagCallPacket {
    pub flag: String,
    pub add: bool,
}

impl PacketCodec for SetFlagCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            flag: read_string(read)?.unwrap_or_default(),
            add: read_bool(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.flag))?;
        write_bool(write, self.add)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetHudTextCallPacket {
    pub message: String,
}

impl SetHudTextCallPacket {
    fn read_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            message: read_string(read)?.unwrap_or_default(),
        })
    }

    fn write_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.message))
    }
}

impl PacketCodec for SetHudTextCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetHudTextReliableCallPacket(pub SetHudTextCallPacket);

impl PacketCodec for SetHudTextReliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        SetHudTextCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetMapAreaCallPacket {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl PacketCodec for SetMapAreaCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            x: read_i32(read)?,
            y: read_i32(read)?,
            width: read_i32(read)?,
            height: read_i32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.x)?;
        write_i32(write, self.y)?;
        write_i32(write, self.width)?;
        write_i32(write, self.height)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetRuleCallPacket {
    pub rule: String,
    pub json_data: String,
}

impl PacketCodec for SetRuleCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            rule: read_string(read)?.unwrap_or_default(),
            json_data: read_string(read)?.unwrap_or_default(),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.rule))?;
        write_string(write, Some(&self.json_data))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputCallPacket {
    pub text_input_id: i32,
    pub title: String,
    pub message: String,
    pub text_length: i32,
    pub default_text: String,
    pub numeric: bool,
}

impl TextInputCallPacket {
    fn read_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            text_input_id: read_i32(read)?,
            title: read_string(read)?.unwrap_or_default(),
            message: read_string(read)?.unwrap_or_default(),
            text_length: read_i32(read)?,
            default_text: read_string(read)?.unwrap_or_default(),
            numeric: read_bool(read)?,
        })
    }

    fn write_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.text_input_id)?;
        write_string(write, Some(&self.title))?;
        write_string(write, Some(&self.message))?;
        write_i32(write, self.text_length)?;
        write_string(write, Some(&self.default_text))?;
        write_bool(write, self.numeric)
    }
}

impl PacketCodec for TextInputCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputCallPacket2 {
    pub prompt: TextInputCallPacket,
    pub allow_empty: bool,
}

impl PacketCodec for TextInputCallPacket2 {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            prompt: TextInputCallPacket::read_payload(read)?,
            allow_empty: read_bool(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.prompt.write_payload(write)?;
        write_bool(write, self.allow_empty)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorldDataBeginCallPacket;

impl PacketCodec for WorldDataBeginCallPacket {
    fn read_from<R: Read>(_read: &mut R) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn write_to<W: Write>(&self, _write: &mut W) -> std::io::Result<()> {
        Ok(())
    }
}

fn read_bool<R: Read>(read: &mut R) -> std::io::Result<bool> {
    Ok(read_u8(read)? != 0)
}

fn write_bool<W: Write>(write: &mut W, value: bool) -> std::io::Result<()> {
    write.write_all(&[value as u8])
}

fn read_f32<R: Read>(read: &mut R) -> std::io::Result<f32> {
    Ok(f32::from_bits(read_u32(read)?))
}

fn write_f32<W: Write>(write: &mut W, value: f32) -> std::io::Result<()> {
    write_u32(write, value.to_bits())
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
    use crate::mindustry::io::TypeValue;

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

        let confirm = ConnectConfirmCallPacket;
        let mut bytes = Vec::new();
        confirm.write_to(&mut bytes).unwrap();
        assert!(bytes.is_empty());
        assert_eq!(
            ConnectConfirmCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            confirm
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

        let bullet = CreateBulletCallPacket {
            bullet_type_id: 12,
            team: TeamId(6),
            x: 10.0,
            y: -20.5,
            angle: 90.0,
            damage: 35.25,
            velocity_scl: 1.5,
            lifetime_scl: 0.75,
        };
        let mut bytes = Vec::new();
        bullet.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&12i16.to_be_bytes());
        expected.push(6);
        for value in [10.0f32, -20.5, 90.0, 35.25, 1.5, 0.75] {
            expected.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        assert_eq!(bytes, expected);
        assert_eq!(
            CreateBulletCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            bullet
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
    fn generated_client_plan_snapshot_packets_use_client_build_plans_layout() {
        let loader = ContentLoader::create_base_content().unwrap();
        let router_id = loader
            .get_by_name(crate::mindustry::ctype::ContentType::Block, "router")
            .unwrap()
            .id
            .to_be_bytes();
        let duo_id = loader
            .get_by_name(crate::mindustry::ctype::ContentType::Block, "duo")
            .unwrap()
            .id
            .to_be_bytes();
        let plans = vec![
            BuildPlanWire::new_place_config(3, 4, 2, "router", TypeValue::String("ignored".into())),
            BuildPlanWire::new_place_config(
                5,
                6,
                1,
                "duo",
                TypeValue::Content(crate::mindustry::io::ContentRef::new(
                    crate::mindustry::ctype::ContentType::Item,
                    0,
                )),
            ),
        ];
        let packet = ClientPlanSnapshotCallPacket {
            group_id: 7,
            plans: Some(plans.clone()),
        };
        let mut bytes = Vec::new();
        packet.write_to_with_loader(&mut bytes, &loader).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&7i32.to_be_bytes());
        expected.extend_from_slice(&2i16.to_be_bytes());
        expected.extend_from_slice(&crate::mindustry::world::point2_pack(3, 4).to_be_bytes());
        expected.extend_from_slice(&router_id);
        expected.push(0);
        expected.extend_from_slice(&crate::mindustry::world::point2_pack(5, 6).to_be_bytes());
        expected.extend_from_slice(&duo_id);
        expected.push(1);
        expected.extend_from_slice(&[
            5,
            crate::mindustry::ctype::ContentType::Item.ordinal(),
            0,
            0,
        ]);
        assert_eq!(bytes, expected);

        let decoded =
            ClientPlanSnapshotCallPacket::read_from_with_loader(&mut bytes.as_slice(), &loader)
                .unwrap();
        assert_eq!(decoded.group_id, 7);
        let decoded_plans = decoded.plans.unwrap();
        assert_eq!(decoded_plans[0].rotation, 0);
        assert_eq!(decoded_plans[0].config, TypeValue::Null);
        assert_eq!(decoded_plans[1], plans[1]);

        let null_packet = ClientPlanSnapshotCallPacket {
            group_id: 8,
            plans: None,
        };
        bytes.clear();
        null_packet
            .write_to_with_loader(&mut bytes, &loader)
            .unwrap();
        assert_eq!(bytes, vec![0, 0, 0, 8, 0, 0]);
        assert_eq!(
            ClientPlanSnapshotCallPacket::read_from_with_loader(&mut bytes.as_slice(), &loader)
                .unwrap(),
            null_packet
        );

        let received = ClientPlanSnapshotReceivedCallPacket {
            player_id: 1234,
            group_id: 9,
            plans: Some(vec![plans[1].clone()]),
        };
        bytes.clear();
        received.write_to_with_loader(&mut bytes, &loader).unwrap();
        assert_eq!(&bytes[..4], &1234i32.to_be_bytes());
        assert_eq!(&bytes[4..8], &9i32.to_be_bytes());
        assert_eq!(
            ClientPlanSnapshotReceivedCallPacket::read_from_with_loader(
                &mut bytes.as_slice(),
                &loader
            )
            .unwrap(),
            received
        );
    }

    #[test]
    fn generated_client_snapshot_packet_uses_java_typeio_field_order() {
        let loader = ContentLoader::create_base_content().unwrap();
        let router_id = loader
            .get_by_name(crate::mindustry::ctype::ContentType::Block, "router")
            .unwrap()
            .id
            .to_be_bytes();
        let conveyor_id = loader
            .get_by_name(crate::mindustry::ctype::ContentType::Block, "conveyor")
            .unwrap()
            .id
            .to_be_bytes();
        let mining = crate::mindustry::world::point2_pack(12, 34);
        let plans = vec![BuildPlanWire::new_place(5, 6, 1, "conveyor")];
        let packet = ClientSnapshotCallPacket {
            snapshot_id: 101,
            unit_id: 202,
            dead: true,
            x: 1.25,
            y: -2.5,
            pointer_x: 3.75,
            pointer_y: -4.125,
            rotation: 90.0,
            base_rotation: 45.0,
            x_velocity: 0.5,
            y_velocity: -0.75,
            mining: Some(mining),
            boosting: true,
            shooting: false,
            chatting: true,
            building: false,
            selected_block: Some("router".into()),
            selected_rotation: 3,
            plans: Some(plans.clone()),
            view_x: 10.0,
            view_y: 20.0,
            view_width: 30.0,
            view_height: 40.0,
        };

        let mut bytes = Vec::new();
        packet.write_to_with_loader(&mut bytes, &loader).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&101i32.to_be_bytes());
        expected.extend_from_slice(&202i32.to_be_bytes());
        expected.push(1);
        for value in [1.25f32, -2.5, 3.75, -4.125, 90.0, 45.0, 0.5, -0.75] {
            expected.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        expected.extend_from_slice(&mining.to_be_bytes());
        expected.push(1);
        expected.push(0);
        expected.push(1);
        expected.push(0);
        expected.extend_from_slice(&router_id);
        expected.extend_from_slice(&3i32.to_be_bytes());
        expected.extend_from_slice(&1i32.to_be_bytes());
        expected.push(0);
        expected.extend_from_slice(&crate::mindustry::world::point2_pack(5, 6).to_be_bytes());
        expected.extend_from_slice(&conveyor_id);
        expected.push(1);
        expected.push(1);
        expected.push(0);
        for value in [10.0f32, 20.0, 30.0, 40.0] {
            expected.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        assert_eq!(bytes, expected);
        assert_eq!(
            ClientSnapshotCallPacket::read_from_with_loader(&mut bytes.as_slice(), &loader)
                .unwrap(),
            packet
        );

        let null_packet = ClientSnapshotCallPacket {
            mining: None,
            selected_block: None,
            plans: None,
            ..packet
        };
        bytes.clear();
        null_packet
            .write_to_with_loader(&mut bytes, &loader)
            .unwrap();
        let mut expected_prefix = Vec::new();
        expected_prefix.extend_from_slice(&101i32.to_be_bytes());
        expected_prefix.extend_from_slice(&202i32.to_be_bytes());
        expected_prefix.push(1);
        for value in [1.25f32, -2.5, 3.75, -4.125, 90.0, 45.0, 0.5, -0.75] {
            expected_prefix.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        expected_prefix
            .extend_from_slice(&crate::mindustry::world::point2_pack(-1, -1).to_be_bytes());
        expected_prefix.push(1);
        expected_prefix.push(0);
        expected_prefix.push(1);
        expected_prefix.push(0);
        expected_prefix.extend_from_slice(&(-1i16).to_be_bytes());
        expected_prefix.extend_from_slice(&3i32.to_be_bytes());
        expected_prefix.extend_from_slice(&(-1i32).to_be_bytes());
        for value in [10.0f32, 20.0, 30.0, 40.0] {
            expected_prefix.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        assert_eq!(bytes, expected_prefix);
        assert_eq!(
            ClientSnapshotCallPacket::read_from_with_loader(&mut bytes.as_slice(), &loader)
                .unwrap(),
            null_packet
        );
    }

    #[test]
    fn generated_construct_finish_packet_uses_java_typeio_field_order() {
        let loader = ContentLoader::create_base_content().unwrap();
        let router_id = loader
            .get_by_name(crate::mindustry::ctype::ContentType::Block, "router")
            .unwrap()
            .id
            .to_be_bytes();
        let tile = crate::mindustry::world::point2_pack(7, 8);
        let packet = ConstructFinishCallPacket {
            tile: Some(tile),
            block: Some("router".into()),
            builder: UnitRef::Unit { id: 1234 },
            rotation: 2,
            team: TeamId(6),
            config: TypeValue::Int(42),
        };

        let mut bytes = Vec::new();
        packet.write_to_with_loader(&mut bytes, &loader).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&tile.to_be_bytes());
        expected.extend_from_slice(&router_id);
        expected.push(2);
        expected.extend_from_slice(&1234i32.to_be_bytes());
        expected.push(2);
        expected.push(6);
        expected.push(1);
        expected.extend_from_slice(&42i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            ConstructFinishCallPacket::read_from_with_loader(&mut bytes.as_slice(), &loader)
                .unwrap(),
            packet
        );

        let null_packet = ConstructFinishCallPacket {
            tile: None,
            block: None,
            builder: UnitRef::Null,
            rotation: 0,
            team: TeamId(0),
            config: TypeValue::Null,
        };
        bytes.clear();
        null_packet
            .write_to_with_loader(&mut bytes, &loader)
            .unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&crate::mindustry::world::point2_pack(-1, -1).to_be_bytes());
        expected.extend_from_slice(&(-1i16).to_be_bytes());
        expected.push(0);
        expected.extend_from_slice(&0i32.to_be_bytes());
        expected.push(0);
        expected.push(0);
        expected.push(0);
        assert_eq!(bytes, expected);
        assert_eq!(
            ConstructFinishCallPacket::read_from_with_loader(&mut bytes.as_slice(), &loader)
                .unwrap(),
            null_packet
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
    fn generated_ui_and_misc_call_packets_use_typeio_field_order() {
        let hide_follow = HideFollowUpMenuCallPacket { menu_id: 42 };
        let mut bytes = Vec::new();
        hide_follow.write_to(&mut bytes).unwrap();
        assert_eq!(bytes, 42i32.to_be_bytes());
        assert_eq!(
            HideFollowUpMenuCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            hide_follow
        );

        let hide_hud = HideHudTextCallPacket;
        let mut bytes = Vec::new();
        hide_hud.write_to(&mut bytes).unwrap();
        assert!(bytes.is_empty());
        assert_eq!(
            HideHudTextCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            hide_hud
        );

        let info = InfoMessageCallPacket {
            message: "hello".into(),
        };
        let mut bytes = Vec::new();
        info.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, Some("hello")).unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(
            InfoMessageCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            info
        );

        let toast = InfoToastCallPacket {
            message: "toast".into(),
            duration: 1.5,
        };
        let mut bytes = Vec::new();
        toast.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, Some("toast")).unwrap();
        expected.extend_from_slice(&1.5f32.to_bits().to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            InfoToastCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            toast
        );

        let kick = KickCallPacket {
            reason: "bye".into(),
        };
        let mut bytes = Vec::new();
        kick.write_to(&mut bytes).unwrap();
        assert_eq!(
            KickCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            kick
        );

        let kick_reason = KickCallPacket2 {
            reason: KickReason::ServerClose,
        };
        let mut bytes = Vec::new();
        kick_reason.write_to(&mut bytes).unwrap();
        assert_eq!(bytes, vec![KickReason::ServerClose.ordinal()]);
        assert_eq!(
            KickCallPacket2::read_from(&mut bytes.as_slice()).unwrap(),
            kick_reason
        );

        let uri = OpenUriCallPacket {
            uri: "https://example.invalid".into(),
        };
        let mut bytes = Vec::new();
        uri.write_to(&mut bytes).unwrap();
        assert_eq!(
            OpenUriCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            uri
        );

        let ping = PingResponseCallPacket { time: 123456789 };
        let mut bytes = Vec::new();
        ping.write_to(&mut bytes).unwrap();
        assert_eq!(bytes, 123456789i64.to_be_bytes());
        assert_eq!(
            PingResponseCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            ping
        );

        let disconnect = PlayerDisconnectCallPacket { player_id: 7 };
        let mut bytes = Vec::new();
        disconnect.write_to(&mut bytes).unwrap();
        assert_eq!(
            PlayerDisconnectCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            disconnect
        );

        let marker = RemoveMarkerCallPacket { id: 99 };
        let mut bytes = Vec::new();
        marker.write_to(&mut bytes).unwrap();
        assert_eq!(
            RemoveMarkerCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            marker
        );

        let remove_queue = RemoveQueueBlockCallPacket {
            x: 1,
            y: 2,
            breaking: true,
        };
        let mut bytes = Vec::new();
        remove_queue.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1i32.to_be_bytes());
        expected.extend_from_slice(&2i32.to_be_bytes());
        expected.push(1);
        assert_eq!(bytes, expected);
        assert_eq!(
            RemoveQueueBlockCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            remove_queue
        );

        let world = WorldDataBeginCallPacket;
        let mut bytes = Vec::new();
        world.write_to(&mut bytes).unwrap();
        assert!(bytes.is_empty());
        assert_eq!(
            WorldDataBeginCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            world
        );
    }

    #[test]
    fn generated_popup_label_hud_rule_and_text_input_packets_use_typeio_field_order() {
        let popup = InfoPopupCallPacket {
            message: Some("popup".into()),
            duration: 2.5,
            align: 1,
            top: 2,
            left: 3,
            bottom: 4,
            right: 5,
        };
        let mut bytes = Vec::new();
        popup.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, Some("popup")).unwrap();
        expected.extend_from_slice(&2.5f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&1i32.to_be_bytes());
        expected.extend_from_slice(&2i32.to_be_bytes());
        expected.extend_from_slice(&3i32.to_be_bytes());
        expected.extend_from_slice(&4i32.to_be_bytes());
        expected.extend_from_slice(&5i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            InfoPopupCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            popup
        );

        let popup_with_id = InfoPopupCallPacket2 {
            message: None,
            id: Some("objective".into()),
            duration: 3.25,
            align: 6,
            top: 7,
            left: 8,
            bottom: 9,
            right: 10,
        };
        let mut bytes = Vec::new();
        popup_with_id.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, None).unwrap();
        crate::mindustry::io::write_string(&mut expected, Some("objective")).unwrap();
        expected.extend_from_slice(&3.25f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&6i32.to_be_bytes());
        expected.extend_from_slice(&7i32.to_be_bytes());
        expected.extend_from_slice(&8i32.to_be_bytes());
        expected.extend_from_slice(&9i32.to_be_bytes());
        expected.extend_from_slice(&10i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            InfoPopupCallPacket2::read_from(&mut bytes.as_slice()).unwrap(),
            popup_with_id
        );

        let reliable_popup = InfoPopupReliableCallPacket(popup.clone());
        let mut reliable_bytes = Vec::new();
        reliable_popup.write_to(&mut reliable_bytes).unwrap();
        assert_eq!(reliable_bytes, {
            let mut expected = Vec::new();
            popup.write_to(&mut expected).unwrap();
            expected
        });
        assert_eq!(
            InfoPopupReliableCallPacket::read_from(&mut reliable_bytes.as_slice()).unwrap(),
            reliable_popup
        );

        let reliable_popup_with_id = InfoPopupReliableCallPacket2(popup_with_id.clone());
        let mut reliable_bytes = Vec::new();
        reliable_popup_with_id
            .write_to(&mut reliable_bytes)
            .unwrap();
        assert_eq!(
            InfoPopupReliableCallPacket2::read_from(&mut reliable_bytes.as_slice()).unwrap(),
            reliable_popup_with_id
        );

        let label = LabelCallPacket {
            message: Some("label".into()),
            duration: 4.5,
            world_x: 12.0,
            world_y: -7.25,
        };
        let mut bytes = Vec::new();
        label.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, Some("label")).unwrap();
        expected.extend_from_slice(&4.5f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&12.0f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&(-7.25f32).to_bits().to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            LabelCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            label
        );

        let label_with_id = LabelCallPacket2 {
            message: None,
            id: 99,
            duration: 1.25,
            world_x: 8.0,
            world_y: 9.0,
        };
        let mut bytes = Vec::new();
        label_with_id.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, None).unwrap();
        expected.extend_from_slice(&99i32.to_be_bytes());
        expected.extend_from_slice(&1.25f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&8.0f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&9.0f32.to_bits().to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            LabelCallPacket2::read_from(&mut bytes.as_slice()).unwrap(),
            label_with_id
        );

        let reliable_label = LabelReliableCallPacket(label.clone());
        let mut bytes = Vec::new();
        reliable_label.write_to(&mut bytes).unwrap();
        assert_eq!(
            LabelReliableCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            reliable_label
        );

        let reliable_label_with_id = LabelReliableCallPacket2(label_with_id.clone());
        let mut bytes = Vec::new();
        reliable_label_with_id.write_to(&mut bytes).unwrap();
        assert_eq!(
            LabelReliableCallPacket2::read_from(&mut bytes.as_slice()).unwrap(),
            reliable_label_with_id
        );

        let camera = SetCameraPositionCallPacket { x: 10.0, y: -4.0 };
        let mut bytes = Vec::new();
        camera.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&10.0f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&(-4.0f32).to_bits().to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            SetCameraPositionCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            camera
        );

        let flag = SetFlagCallPacket {
            flag: "alpha".into(),
            add: true,
        };
        let mut bytes = Vec::new();
        flag.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, Some("alpha")).unwrap();
        expected.push(1);
        assert_eq!(bytes, expected);
        assert_eq!(
            SetFlagCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            flag
        );

        let hud = SetHudTextCallPacket {
            message: "hud".into(),
        };
        let reliable_hud = SetHudTextReliableCallPacket(hud.clone());
        let mut bytes = Vec::new();
        hud.write_to(&mut bytes).unwrap();
        let mut reliable_bytes = Vec::new();
        reliable_hud.write_to(&mut reliable_bytes).unwrap();
        assert_eq!(bytes, reliable_bytes);
        assert_eq!(
            SetHudTextCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            hud
        );
        assert_eq!(
            SetHudTextReliableCallPacket::read_from(&mut reliable_bytes.as_slice()).unwrap(),
            reliable_hud
        );

        let area = SetMapAreaCallPacket {
            x: 1,
            y: 2,
            width: 3,
            height: 4,
        };
        let mut bytes = Vec::new();
        area.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        for value in [1i32, 2, 3, 4] {
            expected.extend_from_slice(&value.to_be_bytes());
        }
        assert_eq!(bytes, expected);
        assert_eq!(
            SetMapAreaCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            area
        );

        let rule = SetRuleCallPacket {
            rule: "unitCap".into(),
            json_data: "{\"value\":42}".into(),
        };
        let mut bytes = Vec::new();
        rule.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        crate::mindustry::io::write_string(&mut expected, Some("unitCap")).unwrap();
        crate::mindustry::io::write_string(&mut expected, Some("{\"value\":42}")).unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(
            SetRuleCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            rule
        );

        let prompt = TextInputCallPacket {
            text_input_id: 12,
            title: "title".into(),
            message: "message".into(),
            text_length: 64,
            default_text: "default".into(),
            numeric: false,
        };
        let mut bytes = Vec::new();
        prompt.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&12i32.to_be_bytes());
        crate::mindustry::io::write_string(&mut expected, Some("title")).unwrap();
        crate::mindustry::io::write_string(&mut expected, Some("message")).unwrap();
        expected.extend_from_slice(&64i32.to_be_bytes());
        crate::mindustry::io::write_string(&mut expected, Some("default")).unwrap();
        expected.push(0);
        assert_eq!(bytes, expected);
        assert_eq!(
            TextInputCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            prompt
        );

        let prompt2 = TextInputCallPacket2 {
            prompt: prompt.clone(),
            allow_empty: true,
        };
        let mut bytes = Vec::new();
        prompt2.write_to(&mut bytes).unwrap();
        let mut expected = expected;
        expected.push(1);
        assert_eq!(bytes, expected);
        assert_eq!(
            TextInputCallPacket2::read_from(&mut bytes.as_slice()).unwrap(),
            prompt2
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
                    packet_ids::CLIENT_PLAN_SNAPSHOT_CALL_PACKET,
                    "ClientPlanSnapshotCallPacket",
                ),
                (
                    packet_ids::CLIENT_PLAN_SNAPSHOT_RECEIVED_CALL_PACKET,
                    "ClientPlanSnapshotReceivedCallPacket",
                ),
                (
                    packet_ids::CLIENT_SNAPSHOT_CALL_PACKET,
                    "ClientSnapshotCallPacket",
                ),
                (
                    packet_ids::COMPLETE_OBJECTIVE_CALL_PACKET,
                    "CompleteObjectiveCallPacket",
                ),
                (packet_ids::CONNECT_CALL_PACKET, "ConnectCallPacket"),
                (
                    packet_ids::CONNECT_CONFIRM_CALL_PACKET,
                    "ConnectConfirmCallPacket",
                ),
                (
                    packet_ids::CONSTRUCT_FINISH_CALL_PACKET,
                    "ConstructFinishCallPacket",
                ),
                (
                    packet_ids::COPY_TO_CLIPBOARD_CALL_PACKET,
                    "CopyToClipboardCallPacket",
                ),
                (
                    packet_ids::CREATE_BULLET_CALL_PACKET,
                    "CreateBulletCallPacket",
                ),
                (
                    packet_ids::DEBUG_STATUS_CLIENT_CALL_PACKET,
                    "DebugStatusClientCallPacket",
                ),
                (
                    packet_ids::DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET,
                    "DebugStatusClientUnreliableCallPacket",
                ),
                (
                    packet_ids::HIDE_FOLLOW_UP_MENU_CALL_PACKET,
                    "HideFollowUpMenuCallPacket",
                ),
                (
                    packet_ids::HIDE_HUD_TEXT_CALL_PACKET,
                    "HideHudTextCallPacket",
                ),
                (
                    packet_ids::INFO_MESSAGE_CALL_PACKET,
                    "InfoMessageCallPacket",
                ),
                (packet_ids::INFO_POPUP_CALL_PACKET, "InfoPopupCallPacket",),
                (packet_ids::INFO_POPUP_CALL_PACKET2, "InfoPopupCallPacket2",),
                (
                    packet_ids::INFO_POPUP_RELIABLE_CALL_PACKET,
                    "InfoPopupReliableCallPacket",
                ),
                (
                    packet_ids::INFO_POPUP_RELIABLE_CALL_PACKET2,
                    "InfoPopupReliableCallPacket2",
                ),
                (packet_ids::INFO_TOAST_CALL_PACKET, "InfoToastCallPacket"),
                (packet_ids::KICK_CALL_PACKET, "KickCallPacket"),
                (packet_ids::KICK_CALL_PACKET2, "KickCallPacket2"),
                (packet_ids::LABEL_CALL_PACKET, "LabelCallPacket"),
                (packet_ids::LABEL_CALL_PACKET2, "LabelCallPacket2"),
                (
                    packet_ids::LABEL_RELIABLE_CALL_PACKET,
                    "LabelReliableCallPacket",
                ),
                (
                    packet_ids::LABEL_RELIABLE_CALL_PACKET2,
                    "LabelReliableCallPacket2",
                ),
                (packet_ids::OPEN_URI_CALL_PACKET, "OpenURICallPacket"),
                (
                    packet_ids::PING_RESPONSE_CALL_PACKET,
                    "PingResponseCallPacket",
                ),
                (
                    packet_ids::PLAYER_DISCONNECT_CALL_PACKET,
                    "PlayerDisconnectCallPacket",
                ),
                (
                    packet_ids::REMOVE_MARKER_CALL_PACKET,
                    "RemoveMarkerCallPacket",
                ),
                (
                    packet_ids::REMOVE_QUEUE_BLOCK_CALL_PACKET,
                    "RemoveQueueBlockCallPacket",
                ),
                (
                    packet_ids::SET_CAMERA_POSITION_CALL_PACKET,
                    "SetCameraPositionCallPacket",
                ),
                (packet_ids::SET_FLAG_CALL_PACKET, "SetFlagCallPacket"),
                (packet_ids::SET_HUD_TEXT_CALL_PACKET, "SetHudTextCallPacket",),
                (
                    packet_ids::SET_HUD_TEXT_RELIABLE_CALL_PACKET,
                    "SetHudTextReliableCallPacket",
                ),
                (packet_ids::SET_MAP_AREA_CALL_PACKET, "SetMapAreaCallPacket",),
                (packet_ids::SET_RULE_CALL_PACKET, "SetRuleCallPacket"),
                (packet_ids::TEXT_INPUT_CALL_PACKET, "TextInputCallPacket"),
                (packet_ids::TEXT_INPUT_CALL_PACKET2, "TextInputCallPacket2"),
                (
                    packet_ids::WORLD_DATA_BEGIN_CALL_PACKET,
                    "WorldDataBeginCallPacket",
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

        let confirm = find_packet_by_name("ConnectConfirmCallPacket").unwrap();
        assert_eq!(confirm.id, Some(packet_ids::CONNECT_CONFIRM_CALL_PACKET));
        assert_eq!(confirm.direction, PacketDirection::ClientToServer);
        assert_eq!(confirm.priority, Some(PacketPriority::High));
        assert!(!confirm.allow_client_endpoint);
        assert!(confirm.allow_server_endpoint);

        let construct = find_packet_by_name("ConstructFinishCallPacket").unwrap();
        assert_eq!(construct.id, Some(packet_ids::CONSTRUCT_FINISH_CALL_PACKET));
        assert_eq!(construct.direction, PacketDirection::ServerToClient);
        assert_eq!(construct.priority, Some(PacketPriority::Normal));
        assert!(construct.allow_client_endpoint);
        assert!(!construct.allow_server_endpoint);

        let bullet = find_packet_by_name("CreateBulletCallPacket").unwrap();
        assert_eq!(bullet.id, Some(packet_ids::CREATE_BULLET_CALL_PACKET));
        assert_eq!(bullet.direction, PacketDirection::ServerToClient);
        assert_eq!(bullet.priority, Some(PacketPriority::Normal));
        assert!(bullet.allow_client_endpoint);
        assert!(!bullet.allow_server_endpoint);
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
            "ClientPlanSnapshotCallPacket",
            "ClientPlanSnapshotReceivedCallPacket",
            "ClientSnapshotCallPacket",
            "CompleteObjectiveCallPacket",
            "ConnectCallPacket",
            "ConnectConfirmCallPacket",
            "ConstructFinishCallPacket",
            "CopyToClipboardCallPacket",
            "CreateBulletCallPacket",
            "DebugStatusClientCallPacket",
            "DebugStatusClientUnreliableCallPacket",
            "HideFollowUpMenuCallPacket",
            "HideHudTextCallPacket",
            "InfoMessageCallPacket",
            "InfoPopupCallPacket",
            "InfoPopupCallPacket2",
            "InfoPopupReliableCallPacket",
            "InfoPopupReliableCallPacket2",
            "InfoToastCallPacket",
            "KickCallPacket",
            "KickCallPacket2",
            "LabelCallPacket",
            "LabelCallPacket2",
            "LabelReliableCallPacket",
            "LabelReliableCallPacket2",
            "OpenURICallPacket",
            "PingResponseCallPacket",
            "PlayerDisconnectCallPacket",
            "RemoveMarkerCallPacket",
            "RemoveQueueBlockCallPacket",
            "SetCameraPositionCallPacket",
            "SetFlagCallPacket",
            "SetHudTextCallPacket",
            "SetHudTextReliableCallPacket",
            "SetMapAreaCallPacket",
            "SetRuleCallPacket",
            "TextInputCallPacket",
            "TextInputCallPacket2",
            "WorldDataBeginCallPacket",
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
