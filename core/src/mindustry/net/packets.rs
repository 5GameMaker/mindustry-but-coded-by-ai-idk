use base64::Engine;
use crc32fast::Hasher;
use std::io::{Read, Write};

use crate::mindustry::core::content_loader::ContentLoader;
use crate::mindustry::ctype::{ContentId, ContentType};
use crate::mindustry::io::type_io::{
    read_block, read_building_ref, read_bullet_type_id, read_bytes, read_client_plans, read_color,
    read_content_id, read_content_name, read_effect_id, read_entity_ref, read_i32, read_i64,
    read_int_seq, read_ints, read_item, read_item_stacks, read_kick, read_liquid,
    read_liquid_stacks, read_marker_control, read_object_safe, read_objective_marker_json,
    read_objectives_json, read_plans_queue, read_required_content_name, read_rules_json,
    read_sound_id, read_string, read_string_array, read_team, read_tile_pos, read_trace_info,
    read_u32, read_u64, read_u8, read_unit_container, read_unit_ref, read_unit_type, write_block,
    write_building_ref, write_bullet_type_id, write_bytes, write_client_plans, write_color,
    write_content_by_name, write_content_id, write_effect_id, write_entity_ref, write_i32,
    write_i64, write_int_seq, write_ints, write_item, write_item_stacks, write_kick, write_liquid,
    write_liquid_stacks, write_marker_control, write_object, write_objective_marker_json,
    write_objectives_json, write_plans_queue_net, write_required_content_ref, write_rules_json,
    write_sound_id, write_string, write_string_array, write_team, write_tile_pos, write_trace_info,
    write_u32, write_u64, write_u8, write_unit_container, write_unit_ref, write_unit_type,
    BuildPlanWire, BuildingRef, EntityRef, RgbaColor, TeamId, TypeValue, UnitRef,
};
use crate::mindustry::io::UnitSyncContainer;
use crate::mindustry::net::administration::TraceInfo as NetTraceInfo;
use crate::mindustry::r#type::{ItemStack, LiquidStack};

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
    pub const CREATE_MARKER_CALL_PACKET: PacketId = 35;
    pub const CREATE_WEATHER_CALL_PACKET: PacketId = 36;
    pub const DEBUG_STATUS_CLIENT_CALL_PACKET: PacketId = 37;
    pub const DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET: PacketId = 38;
    pub const DECONSTRUCT_FINISH_CALL_PACKET: PacketId = 39;
    pub const DELETE_PLANS_CALL_PACKET: PacketId = 40;
    pub const DESTROY_PAYLOAD_CALL_PACKET: PacketId = 41;
    pub const DROP_ITEM_CALL_PACKET: PacketId = 42;
    pub const EFFECT_CALL_PACKET: PacketId = 43;
    pub const EFFECT_CALL_PACKET2: PacketId = 44;
    pub const EFFECT_RELIABLE_CALL_PACKET: PacketId = 45;
    pub const ENTITY_SNAPSHOT_CALL_PACKET: PacketId = 46;
    pub const FOLLOW_UP_MENU_CALL_PACKET: PacketId = 47;
    pub const GAME_OVER_CALL_PACKET: PacketId = 48;
    pub const HIDDEN_SNAPSHOT_CALL_PACKET: PacketId = 49;
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
    pub const LANDING_PAD_LANDED_CALL_PACKET: PacketId = 64;
    pub const LOGIC_EXPLOSION_CALL_PACKET: PacketId = 65;
    pub const MENU_CALL_PACKET: PacketId = 66;
    pub const MENU_CHOOSE_CALL_PACKET: PacketId = 67;
    pub const OPEN_URI_CALL_PACKET: PacketId = 68;
    pub const PAYLOAD_DROPPED_CALL_PACKET: PacketId = 69;
    pub const PICKED_BUILD_PAYLOAD_CALL_PACKET: PacketId = 70;
    pub const PICKED_UNIT_PAYLOAD_CALL_PACKET: PacketId = 71;
    pub const PING_CALL_PACKET: PacketId = 72;
    pub const PING_LOCATION_CALL_PACKET: PacketId = 73;
    pub const PING_RESPONSE_CALL_PACKET: PacketId = 74;
    pub const PLAYER_DISCONNECT_CALL_PACKET: PacketId = 75;
    pub const PLAYER_SPAWN_CALL_PACKET: PacketId = 76;
    pub const REMOVE_MARKER_CALL_PACKET: PacketId = 77;
    pub const REMOVE_QUEUE_BLOCK_CALL_PACKET: PacketId = 78;
    pub const REMOVE_TILE_CALL_PACKET: PacketId = 79;
    pub const REMOVE_WORLD_LABEL_CALL_PACKET: PacketId = 80;
    pub const REQUEST_BLOCK_SNAPSHOT_CALL_PACKET: PacketId = 81;
    pub const REQUEST_BUILD_PAYLOAD_CALL_PACKET: PacketId = 82;
    pub const REQUEST_DEBUG_STATUS_CALL_PACKET: PacketId = 83;
    pub const REQUEST_DROP_PAYLOAD_CALL_PACKET: PacketId = 84;
    pub const REQUEST_ITEM_CALL_PACKET: PacketId = 85;
    pub const REQUEST_UNIT_PAYLOAD_CALL_PACKET: PacketId = 86;
    pub const RESEARCHED_CALL_PACKET: PacketId = 87;
    pub const ROTATE_BLOCK_CALL_PACKET: PacketId = 88;
    pub const SECTOR_CAPTURE_CALL_PACKET: PacketId = 89;
    pub const SEND_CHAT_MESSAGE_CALL_PACKET: PacketId = 90;
    pub const SEND_MESSAGE_CALL_PACKET: PacketId = 91;
    pub const SEND_MESSAGE_CALL_PACKET2: PacketId = 92;
    pub const SERVER_BINARY_PACKET_RELIABLE_CALL_PACKET: PacketId = 93;
    pub const SERVER_BINARY_PACKET_UNRELIABLE_CALL_PACKET: PacketId = 94;
    pub const SERVER_PACKET_RELIABLE_CALL_PACKET: PacketId = 95;
    pub const SERVER_PACKET_UNRELIABLE_CALL_PACKET: PacketId = 96;
    pub const SET_CAMERA_POSITION_CALL_PACKET: PacketId = 97;
    pub const SET_FLAG_CALL_PACKET: PacketId = 98;
    pub const SET_FLOOR_CALL_PACKET: PacketId = 99;
    pub const SET_HUD_TEXT_CALL_PACKET: PacketId = 100;
    pub const SET_HUD_TEXT_RELIABLE_CALL_PACKET: PacketId = 101;
    pub const SET_ITEM_CALL_PACKET: PacketId = 102;
    pub const SET_ITEMS_CALL_PACKET: PacketId = 103;
    pub const SET_LIQUID_CALL_PACKET: PacketId = 104;
    pub const SET_LIQUIDS_CALL_PACKET: PacketId = 105;
    pub const SET_MAP_AREA_CALL_PACKET: PacketId = 106;
    pub const SET_OBJECTIVES_CALL_PACKET: PacketId = 107;
    pub const SET_OVERLAY_CALL_PACKET: PacketId = 108;
    pub const SET_PLAYER_TEAM_EDITOR_CALL_PACKET: PacketId = 109;
    pub const SET_POSITION_CALL_PACKET: PacketId = 110;
    pub const SET_RULE_CALL_PACKET: PacketId = 111;
    pub const SET_RULES_CALL_PACKET: PacketId = 112;
    pub const SET_TEAM_CALL_PACKET: PacketId = 113;
    pub const SET_TEAMS_CALL_PACKET: PacketId = 114;
    pub const SET_TILE_CALL_PACKET: PacketId = 115;
    pub const SET_TILE_BLOCKS_CALL_PACKET: PacketId = 116;
    pub const SET_TILE_FLOORS_CALL_PACKET: PacketId = 117;
    pub const SET_TILE_ITEMS_CALL_PACKET: PacketId = 118;
    pub const SET_TILE_LIQUIDS_CALL_PACKET: PacketId = 119;
    pub const SET_TILE_OVERLAYS_CALL_PACKET: PacketId = 120;
    pub const SET_UNIT_COMMAND_CALL_PACKET: PacketId = 121;
    pub const SET_UNIT_STANCE_CALL_PACKET: PacketId = 122;
    pub const SOUND_CALL_PACKET: PacketId = 123;
    pub const SOUND_AT_CALL_PACKET: PacketId = 124;
    pub const SPAWN_EFFECT_CALL_PACKET: PacketId = 125;
    pub const STATE_SNAPSHOT_CALL_PACKET: PacketId = 126;
    pub const SYNC_VARIABLE_CALL_PACKET: PacketId = 127;
    pub const TAKE_ITEMS_CALL_PACKET: PacketId = 128;
    pub const TEXT_INPUT_CALL_PACKET: PacketId = 129;
    pub const TEXT_INPUT_CALL_PACKET2: PacketId = 130;
    pub const TEXT_INPUT_RESULT_CALL_PACKET: PacketId = 131;
    pub const TILE_CONFIG_CALL_PACKET: PacketId = 132;
    pub const TILE_TAP_CALL_PACKET: PacketId = 133;
    pub const TRACE_INFO_CALL_PACKET: PacketId = 134;
    pub const TRANSFER_INVENTORY_CALL_PACKET: PacketId = 135;
    pub const TRANSFER_ITEM_EFFECT_CALL_PACKET: PacketId = 136;
    pub const TRANSFER_ITEM_TO_CALL_PACKET: PacketId = 137;
    pub const TRANSFER_ITEM_TO_UNIT_CALL_PACKET: PacketId = 138;
    pub const UNIT_BLOCK_SPAWN_CALL_PACKET: PacketId = 139;
    pub const UNIT_BUILDING_CONTROL_SELECT_CALL_PACKET: PacketId = 140;
    pub const UNIT_CAP_DEATH_CALL_PACKET: PacketId = 141;
    pub const UNIT_CLEAR_CALL_PACKET: PacketId = 142;
    pub const UNIT_CONTROL_CALL_PACKET: PacketId = 143;
    pub const UNIT_DEATH_CALL_PACKET: PacketId = 144;
    pub const UNIT_DESPAWN_CALL_PACKET: PacketId = 145;
    pub const UNIT_DESTROY_CALL_PACKET: PacketId = 146;
    pub const UNIT_ENTERED_PAYLOAD_CALL_PACKET: PacketId = 147;
    pub const UNIT_ENV_DEATH_CALL_PACKET: PacketId = 148;
    pub const UNIT_SAFE_DEATH_CALL_PACKET: PacketId = 149;
    pub const UNIT_SPAWN_CALL_PACKET: PacketId = 150;
    pub const UNIT_TETHER_BLOCK_SPAWNED_CALL_PACKET: PacketId = 151;
    pub const UPDATE_GAME_OVER_CALL_PACKET: PacketId = 152;
    pub const UPDATE_MARKER_CALL_PACKET: PacketId = 153;
    pub const UPDATE_MARKER_TEXT_CALL_PACKET: PacketId = 154;
    pub const UPDATE_MARKER_TEXTURE_CALL_PACKET: PacketId = 155;
    pub const WARNING_TOAST_CALL_PACKET: PacketId = 156;
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

pub const CREATE_MARKER_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::CREATE_MARKER_CALL_PACKET),
    name: "CreateMarkerCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.CreateMarkerCallPacket",
    notes: "Generated by Call.registerPackets(); Java int id followed by TypeIO ObjectiveMarker JSON bytes.",
};

pub const CREATE_WEATHER_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::CREATE_WEATHER_CALL_PACKET),
    name: "CreateWeatherCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.CreateWeatherCallPacket",
    notes: "Generated by Call.registerPackets(); nullable Weather short id plus intensity, duration and wind floats.",
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

pub const DECONSTRUCT_FINISH_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::DECONSTRUCT_FINISH_CALL_PACKET),
    name: "DeconstructFinishCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.DeconstructFinishCallPacket",
    notes: "Generated by Call.registerPackets(); Tile pos, Block id and Unit ref for deconstruct completion.",
};

pub const DELETE_PLANS_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::DELETE_PLANS_CALL_PACKET),
    name: "DeletePlansCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.DeletePlansCallPacket",
    notes: "Generated by Call.registerPackets(); server-forwarded packets include player entity id, client-origin packets omit it, both carry TypeIO int array positions.",
};

pub const DESTROY_PAYLOAD_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::DESTROY_PAYLOAD_CALL_PACKET),
    name: "DestroyPayloadCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.DestroyPayloadCallPacket",
    notes: "Generated by Call.registerPackets(); single Building tile position as Java int.",
};

pub const DROP_ITEM_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::DROP_ITEM_CALL_PACKET),
    name: "DropItemCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ClientToServer,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: false,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.DropItemCallPacket",
    notes: "Generated by Call.registerPackets(); single Java float angle.",
};

pub const EFFECT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::EFFECT_CALL_PACKET),
    name: "EffectCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.EffectCallPacket",
    notes: "Generated by Call.registerPackets(); Effect id, x/y/rotation floats and Color rgba.",
};

pub const EFFECT_CALL_PACKET2_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::EFFECT_CALL_PACKET2),
    name: "EffectCallPacket2",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.EffectCallPacket2",
    notes: "Generated by Call.registerPackets(); EffectCallPacket payload followed by TypeIO Object data.",
};

pub const EFFECT_RELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::EFFECT_RELIABLE_CALL_PACKET),
    name: "EffectReliableCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.EffectReliableCallPacket",
    notes: "Generated by Call.registerPackets(); reliable twin of EffectCallPacket.",
};

pub const ENTITY_SNAPSHOT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::ENTITY_SNAPSHOT_CALL_PACKET),
    name: "EntitySnapshotCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Low),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.EntitySnapshotCallPacket",
    notes: "Generated by Call.registerPackets(); low-priority snapshot amount short plus TypeIO byte array.",
};

pub const FOLLOW_UP_MENU_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::FOLLOW_UP_MENU_CALL_PACKET),
    name: "FollowUpMenuCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.FollowUpMenuCallPacket",
    notes: "Generated by Call.registerPackets(); menu id, title/message strings and TypeIO string matrix options.",
};

pub const GAME_OVER_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::GAME_OVER_CALL_PACKET),
    name: "GameOverCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.GameOverCallPacket",
    notes: "Generated by Call.registerPackets(); single TypeIO Team byte winner.",
};

pub const HIDDEN_SNAPSHOT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::HIDDEN_SNAPSHOT_CALL_PACKET),
    name: "HiddenSnapshotCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Low),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.HiddenSnapshotCallPacket",
    notes: "Generated by Call.registerPackets(); low-priority TypeIO IntSeq of hidden entity ids.",
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

pub const LANDING_PAD_LANDED_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::LANDING_PAD_LANDED_CALL_PACKET),
    name: "LandingPadLandedCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.LandingPadLandedCallPacket",
    notes: "Generated by Call.registerPackets(); single TypeIO Tile position for landing pad completion.",
};

pub const LOGIC_EXPLOSION_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::LOGIC_EXPLOSION_CALL_PACKET),
    name: "LogicExplosionCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.LogicExplosionCallPacket",
    notes: "Generated by Call.registerPackets(); Team, four Java floats and four booleans.",
};

pub const MENU_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::MENU_CALL_PACKET),
    name: "MenuCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.MenuCallPacket",
    notes: "Generated by Call.registerPackets(); menu id, title/message strings and option string matrix.",
};

pub const MENU_CHOOSE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::MENU_CHOOSE_CALL_PACKET),
    name: "MenuChooseCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.MenuChooseCallPacket",
    notes: "Generated by Call.registerPackets(); server-forwarded packets include player entity id, client-origin packets omit it.",
};

pub const OPEN_URI_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::OPEN_URI_CALL_PACKET),
    name: "OpenURICallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.OpenURICallPacket",
    notes: "Generated by Call.registerPackets(); single TypeIO string uri.",
};

pub const PAYLOAD_DROPPED_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::PAYLOAD_DROPPED_CALL_PACKET),
    name: "PayloadDroppedCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.PayloadDroppedCallPacket",
    notes: "Generated by Call.registerPackets(); Unit ref and dropped payload world coordinates.",
};

pub const PICKED_BUILD_PAYLOAD_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::PICKED_BUILD_PAYLOAD_CALL_PACKET),
    name: "PickedBuildPayloadCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.PickedBuildPayloadCallPacket",
    notes:
        "Generated by Call.registerPackets(); Unit ref, Building tile position and onGround flag.",
};

pub const PICKED_UNIT_PAYLOAD_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::PICKED_UNIT_PAYLOAD_CALL_PACKET),
    name: "PickedUnitPayloadCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.PickedUnitPayloadCallPacket",
    notes: "Generated by Call.registerPackets(); source Unit ref and target Unit ref.",
};

pub const PING_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::PING_CALL_PACKET),
    name: "PingCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ClientToServer,
    streamable: false,
    priority: Some(PacketPriority::High),
    allow_client_endpoint: false,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.PingCallPacket",
    notes:
        "Generated by Call.registerPackets(); high priority Java long time from client to server.",
};

pub const PING_LOCATION_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::PING_LOCATION_CALL_PACKET),
    name: "PingLocationCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.PingLocationCallPacket",
    notes: "Generated by Call.registerPackets(); server-forwarded packets include player entity id, client-origin packets omit it.",
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

pub const PLAYER_SPAWN_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::PLAYER_SPAWN_CALL_PACKET),
    name: "PlayerSpawnCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.PlayerSpawnCallPacket",
    notes: "Generated by Call.registerPackets(); TypeIO tile position then player entity id.",
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

pub const REMOVE_TILE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::REMOVE_TILE_CALL_PACKET),
    name: "RemoveTileCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.RemoveTileCallPacket",
    notes: "Generated by Call.registerPackets(); TypeIO tile position.",
};

pub const REMOVE_WORLD_LABEL_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::REMOVE_WORLD_LABEL_CALL_PACKET),
    name: "RemoveWorldLabelCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.RemoveWorldLabelCallPacket",
    notes: "Generated by Call.registerPackets(); single Java int label id.",
};

pub const REQUEST_BLOCK_SNAPSHOT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::REQUEST_BLOCK_SNAPSHOT_CALL_PACKET),
    name: "RequestBlockSnapshotCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ClientToServer,
    streamable: false,
    priority: Some(PacketPriority::Low),
    allow_client_endpoint: false,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.RequestBlockSnapshotCallPacket",
    notes: "Generated by Call.registerPackets(); low priority Java int packed position.",
};

pub const REQUEST_BUILD_PAYLOAD_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::REQUEST_BUILD_PAYLOAD_CALL_PACKET),
    name: "RequestBuildPayloadCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.RequestBuildPayloadCallPacket",
    notes: "Generated by Call.registerPackets(); server-forwarded packets include player entity id, client-origin packets omit it.",
};

pub const REQUEST_DEBUG_STATUS_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::REQUEST_DEBUG_STATUS_CALL_PACKET),
    name: "RequestDebugStatusCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ClientToServer,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: false,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.RequestDebugStatusCallPacket",
    notes: "Generated by Call.registerPackets(); empty client-to-server payload.",
};

pub const REQUEST_DROP_PAYLOAD_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::REQUEST_DROP_PAYLOAD_CALL_PACKET),
    name: "RequestDropPayloadCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.RequestDropPayloadCallPacket",
    notes: "Generated by Call.registerPackets(); server-forwarded packets include player entity id before x/y.",
};

pub const REQUEST_ITEM_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::REQUEST_ITEM_CALL_PACKET),
    name: "RequestItemCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.RequestItemCallPacket",
    notes: "Generated by Call.registerPackets(); server-forwarded packets include player entity id before build/item/amount.",
};

pub const REQUEST_UNIT_PAYLOAD_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::REQUEST_UNIT_PAYLOAD_CALL_PACKET),
    name: "RequestUnitPayloadCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.RequestUnitPayloadCallPacket",
    notes: "Generated by Call.registerPackets(); server-forwarded packets include player entity id before TypeIO unit.",
};

pub const RESEARCHED_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::RESEARCHED_CALL_PACKET),
    name: "ResearchedCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.ResearchedCallPacket",
    notes:
        "Generated by Call.registerPackets(); TypeIO content ref byte content type plus short id.",
};

pub const ROTATE_BLOCK_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::ROTATE_BLOCK_CALL_PACKET),
    name: "RotateBlockCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.RotateBlockCallPacket",
    notes: "Generated by Call.registerPackets(); server-forwarded packets include player entity id before build/direction.",
};

pub const SECTOR_CAPTURE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SECTOR_CAPTURE_CALL_PACKET),
    name: "SectorCaptureCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SectorCaptureCallPacket",
    notes: "Generated by Call.registerPackets(); empty server-to-client call packet.",
};

pub const SEND_CHAT_MESSAGE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SEND_CHAT_MESSAGE_CALL_PACKET),
    name: "SendChatMessageCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ClientToServer,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: false,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SendChatMessageCallPacket",
    notes: "Generated by Call.registerPackets(); client chat message to server.",
};

pub const SEND_MESSAGE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SEND_MESSAGE_CALL_PACKET),
    name: "SendMessageCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SendMessageCallPacket",
    notes: "Generated by Call.registerPackets(); plain message string for clients.",
};

pub const SEND_MESSAGE_CALL_PACKET2_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SEND_MESSAGE_CALL_PACKET2),
    name: "SendMessageCallPacket2",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SendMessageCallPacket2",
    notes: "Generated by Call.registerPackets(); message plus unformatted text and sender entity.",
};

pub const SERVER_BINARY_PACKET_RELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry =
    PacketManifestEntry {
        id: Some(packet_ids::SERVER_BINARY_PACKET_RELIABLE_CALL_PACKET),
        name: "ServerBinaryPacketReliableCallPacket",
        transport: PacketTransport::NetPacket,
        direction: PacketDirection::ClientToServer,
        streamable: false,
        priority: Some(PacketPriority::Normal),
        allow_client_endpoint: false,
        allow_server_endpoint: true,
        force_uncompressed: false,
        codec: PacketCodecState::Implemented,
        upstream: "mindustry.gen.ServerBinaryPacketReliableCallPacket",
        notes: "Generated by Call.registerPackets(); reliable server-binary custom packet.",
    };

pub const SERVER_BINARY_PACKET_UNRELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry =
    PacketManifestEntry {
        id: Some(packet_ids::SERVER_BINARY_PACKET_UNRELIABLE_CALL_PACKET),
        name: "ServerBinaryPacketUnreliableCallPacket",
        transport: PacketTransport::NetPacket,
        direction: PacketDirection::ClientToServer,
        streamable: false,
        priority: Some(PacketPriority::Normal),
        allow_client_endpoint: false,
        allow_server_endpoint: true,
        force_uncompressed: false,
        codec: PacketCodecState::Implemented,
        upstream: "mindustry.gen.ServerBinaryPacketUnreliableCallPacket",
        notes: "Generated by Call.registerPackets(); unreliable server-binary custom packet.",
    };

pub const SERVER_PACKET_RELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SERVER_PACKET_RELIABLE_CALL_PACKET),
    name: "ServerPacketReliableCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ClientToServer,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: false,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.ServerPacketReliableCallPacket",
    notes: "Generated by Call.registerPackets(); reliable server string packet.",
};

pub const SERVER_PACKET_UNRELIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry =
    PacketManifestEntry {
        id: Some(packet_ids::SERVER_PACKET_UNRELIABLE_CALL_PACKET),
        name: "ServerPacketUnreliableCallPacket",
        transport: PacketTransport::NetPacket,
        direction: PacketDirection::ClientToServer,
        streamable: false,
        priority: Some(PacketPriority::Normal),
        allow_client_endpoint: false,
        allow_server_endpoint: true,
        force_uncompressed: false,
        codec: PacketCodecState::Implemented,
        upstream: "mindustry.gen.ServerPacketUnreliableCallPacket",
        notes: "Generated by Call.registerPackets(); unreliable server string packet.",
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

pub const SET_FLOOR_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_FLOOR_CALL_PACKET),
    name: "SetFloorCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetFloorCallPacket",
    notes: "Generated by Call.registerPackets(); tile plus floor/overlay block content names.",
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

pub const SET_ITEM_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_ITEM_CALL_PACKET),
    name: "SetItemCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetItemCallPacket",
    notes: "Generated by Call.registerPackets(); building item stack update.",
};

pub const SET_ITEMS_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_ITEMS_CALL_PACKET),
    name: "SetItemsCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetItemsCallPacket",
    notes: "Generated by Call.registerPackets(); building item stacks update.",
};

pub const SET_LIQUID_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_LIQUID_CALL_PACKET),
    name: "SetLiquidCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetLiquidCallPacket",
    notes: "Generated by Call.registerPackets(); building liquid amount update.",
};

pub const SET_LIQUIDS_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_LIQUIDS_CALL_PACKET),
    name: "SetLiquidsCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetLiquidsCallPacket",
    notes: "Generated by Call.registerPackets(); building liquid stacks update.",
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

pub const SET_OBJECTIVES_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_OBJECTIVES_CALL_PACKET),
    name: "SetObjectivesCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetObjectivesCallPacket",
    notes: "Generated by Call.registerPackets(); objectives JSON payload.",
};

pub const SET_OVERLAY_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_OVERLAY_CALL_PACKET),
    name: "SetOverlayCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetOverlayCallPacket",
    notes: "Generated by Call.registerPackets(); tile plus overlay block content name.",
};

pub const SET_PLAYER_TEAM_EDITOR_CALL_PACKET_MANIFEST: PacketManifestEntry =
    PacketManifestEntry {
        id: Some(packet_ids::SET_PLAYER_TEAM_EDITOR_CALL_PACKET),
        name: "SetPlayerTeamEditorCallPacket",
        transport: PacketTransport::NetPacket,
        direction: PacketDirection::Bidirectional,
        streamable: false,
        priority: Some(PacketPriority::Normal),
        allow_client_endpoint: true,
        allow_server_endpoint: true,
        force_uncompressed: false,
        codec: PacketCodecState::Implemented,
        upstream: "mindustry.gen.SetPlayerTeamEditorCallPacket",
        notes: "Generated by Call.registerPackets(); player entity only appears on server-forwarded packets.",
    };

pub const SET_POSITION_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_POSITION_CALL_PACKET),
    name: "SetPositionCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetPositionCallPacket",
    notes: "Generated by Call.registerPackets(); two Java float fields.",
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

pub const SET_RULES_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_RULES_CALL_PACKET),
    name: "SetRulesCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetRulesCallPacket",
    notes: "Generated by Call.registerPackets(); rules JSON payload.",
};

pub const SET_TEAM_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_TEAM_CALL_PACKET),
    name: "SetTeamCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetTeamCallPacket",
    notes: "Generated by Call.registerPackets(); building plus team.",
};

pub const SET_TEAMS_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_TEAMS_CALL_PACKET),
    name: "SetTeamsCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetTeamsCallPacket",
    notes: "Generated by Call.registerPackets(); packed positions plus team.",
};

pub const SET_TILE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_TILE_CALL_PACKET),
    name: "SetTileCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetTileCallPacket",
    notes: "Generated by Call.registerPackets(); tile plus block/team/rotation fields.",
};

pub const SET_TILE_BLOCKS_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_TILE_BLOCKS_CALL_PACKET),
    name: "SetTileBlocksCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetTileBlocksCallPacket",
    notes: "Generated by Call.registerPackets(); block/team plus packed positions.",
};

pub const SET_TILE_FLOORS_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_TILE_FLOORS_CALL_PACKET),
    name: "SetTileFloorsCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetTileFloorsCallPacket",
    notes: "Generated by Call.registerPackets(); floor block plus packed positions.",
};

pub const SET_TILE_ITEMS_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_TILE_ITEMS_CALL_PACKET),
    name: "SetTileItemsCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetTileItemsCallPacket",
    notes: "Generated by Call.registerPackets(); item amount plus packed positions.",
};

pub const SET_TILE_LIQUIDS_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_TILE_LIQUIDS_CALL_PACKET),
    name: "SetTileLiquidsCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetTileLiquidsCallPacket",
    notes: "Generated by Call.registerPackets(); liquid amount plus packed positions.",
};

pub const SET_TILE_OVERLAYS_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_TILE_OVERLAYS_CALL_PACKET),
    name: "SetTileOverlaysCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetTileOverlaysCallPacket",
    notes: "Generated by Call.registerPackets(); overlay block plus packed positions.",
};

pub const SET_UNIT_COMMAND_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_UNIT_COMMAND_CALL_PACKET),
    name: "SetUnitCommandCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetUnitCommandCallPacket",
    notes: "Generated by Call.registerPackets(); selected unit ids plus command content.",
};

pub const SET_UNIT_STANCE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SET_UNIT_STANCE_CALL_PACKET),
    name: "SetUnitStanceCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SetUnitStanceCallPacket",
    notes: "Generated by Call.registerPackets(); selected unit ids plus stance content.",
};

pub const SOUND_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SOUND_CALL_PACKET),
    name: "SoundCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SoundCallPacket",
    notes: "Generated by Call.registerPackets(); global sound id/volume/pitch/pan.",
};

pub const SOUND_AT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SOUND_AT_CALL_PACKET),
    name: "SoundAtCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SoundAtCallPacket",
    notes: "Generated by Call.registerPackets(); positional sound id/volume/pitch.",
};

pub const SPAWN_EFFECT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SPAWN_EFFECT_CALL_PACKET),
    name: "SpawnEffectCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SpawnEffectCallPacket",
    notes: "Generated by Call.registerPackets(); spawn effect coordinates plus unit type.",
};

pub const STATE_SNAPSHOT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::STATE_SNAPSHOT_CALL_PACKET),
    name: "StateSnapshotCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Low),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.StateSnapshotCallPacket",
    notes: "Generated by Call.registerPackets(); low-priority game state snapshot.",
};

pub const SYNC_VARIABLE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::SYNC_VARIABLE_CALL_PACKET),
    name: "SyncVariableCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.SyncVariableCallPacket",
    notes: "Generated by Call.registerPackets(); logic variable sync payload.",
};

pub const TAKE_ITEMS_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::TAKE_ITEMS_CALL_PACKET),
    name: "TakeItemsCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.TakeItemsCallPacket",
    notes: "Generated by Call.registerPackets(); take items from building to unit.",
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

pub const TEXT_INPUT_RESULT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::TEXT_INPUT_RESULT_CALL_PACKET),
    name: "TextInputResultCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.TextInputResultCallPacket",
    notes: "Generated by Call.registerPackets(); text input result echo.",
};

pub const TILE_CONFIG_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::TILE_CONFIG_CALL_PACKET),
    name: "TileConfigCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.TileConfigCallPacket",
    notes: "Generated by Call.registerPackets(); tile configuration object payload.",
};

pub const TILE_TAP_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::TILE_TAP_CALL_PACKET),
    name: "TileTapCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.TileTapCallPacket",
    notes: "Generated by Call.registerPackets(); tile tap notification.",
};

pub const TRACE_INFO_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::TRACE_INFO_CALL_PACKET),
    name: "TraceInfoCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.TraceInfoCallPacket",
    notes: "Generated by Call.registerPackets(); administration trace info.",
};

pub const TRANSFER_INVENTORY_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::TRANSFER_INVENTORY_CALL_PACKET),
    name: "TransferInventoryCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.TransferInventoryCallPacket",
    notes: "Generated by Call.registerPackets(); transfer inventory request.",
};

pub const TRANSFER_ITEM_EFFECT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::TRANSFER_ITEM_EFFECT_CALL_PACKET),
    name: "TransferItemEffectCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.TransferItemEffectCallPacket",
    notes: "Generated by Call.registerPackets(); transfer item effect.",
};

pub const TRANSFER_ITEM_TO_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::TRANSFER_ITEM_TO_CALL_PACKET),
    name: "TransferItemToCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.TransferItemToCallPacket",
    notes: "Generated by Call.registerPackets(); transfer item to unit/building.",
};

pub const TRANSFER_ITEM_TO_UNIT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::TRANSFER_ITEM_TO_UNIT_CALL_PACKET),
    name: "TransferItemToUnitCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.TransferItemToUnitCallPacket",
    notes: "Generated by Call.registerPackets(); transfer item to unit.",
};

pub const UNIT_BLOCK_SPAWN_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UNIT_BLOCK_SPAWN_CALL_PACKET),
    name: "UnitBlockSpawnCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UnitBlockSpawnCallPacket",
    notes: "Generated by Call.registerPackets(); unit block spawn notification.",
};

pub const UNIT_BUILDING_CONTROL_SELECT_CALL_PACKET_MANIFEST: PacketManifestEntry =
    PacketManifestEntry {
        id: Some(packet_ids::UNIT_BUILDING_CONTROL_SELECT_CALL_PACKET),
        name: "UnitBuildingControlSelectCallPacket",
        transport: PacketTransport::NetPacket,
        direction: PacketDirection::ServerToClient,
        streamable: false,
        priority: Some(PacketPriority::Normal),
        allow_client_endpoint: true,
        allow_server_endpoint: false,
        force_uncompressed: false,
        codec: PacketCodecState::Implemented,
        upstream: "mindustry.gen.UnitBuildingControlSelectCallPacket",
        notes: "Generated by Call.registerPackets(); unit building control select.",
    };

pub const UNIT_CAP_DEATH_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UNIT_CAP_DEATH_CALL_PACKET),
    name: "UnitCapDeathCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UnitCapDeathCallPacket",
    notes: "Generated by Call.registerPackets(); unit cap death notification.",
};

pub const UNIT_CLEAR_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UNIT_CLEAR_CALL_PACKET),
    name: "UnitClearCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UnitClearCallPacket",
    notes: "Generated by Call.registerPackets(); clear unit control.",
};

pub const UNIT_CONTROL_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UNIT_CONTROL_CALL_PACKET),
    name: "UnitControlCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::Bidirectional,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: true,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UnitControlCallPacket",
    notes: "Generated by Call.registerPackets(); unit control selection.",
};

pub const UNIT_DEATH_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UNIT_DEATH_CALL_PACKET),
    name: "UnitDeathCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UnitDeathCallPacket",
    notes: "Generated by Call.registerPackets(); unit death notification.",
};

pub const UNIT_DESPAWN_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UNIT_DESPAWN_CALL_PACKET),
    name: "UnitDespawnCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UnitDespawnCallPacket",
    notes: "Generated by Call.registerPackets(); unit despawn notification.",
};

pub const UNIT_DESTROY_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UNIT_DESTROY_CALL_PACKET),
    name: "UnitDestroyCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UnitDestroyCallPacket",
    notes: "Generated by Call.registerPackets(); unit destroy notification.",
};

pub const UNIT_ENTERED_PAYLOAD_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UNIT_ENTERED_PAYLOAD_CALL_PACKET),
    name: "UnitEnteredPayloadCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UnitEnteredPayloadCallPacket",
    notes: "Generated by Call.registerPackets(); unit entered payload notification.",
};

pub const UNIT_ENV_DEATH_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UNIT_ENV_DEATH_CALL_PACKET),
    name: "UnitEnvDeathCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UnitEnvDeathCallPacket",
    notes: "Generated by Call.registerPackets(); unit environment death notification.",
};

pub const UNIT_SAFE_DEATH_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UNIT_SAFE_DEATH_CALL_PACKET),
    name: "UnitSafeDeathCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UnitSafeDeathCallPacket",
    notes: "Generated by Call.registerPackets(); unit safe death notification.",
};

pub const UNIT_SPAWN_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UNIT_SPAWN_CALL_PACKET),
    name: "UnitSpawnCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Low),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UnitSpawnCallPacket",
    notes: "Generated by Call.registerPackets(); unit spawn sync container.",
};

pub const UNIT_TETHER_BLOCK_SPAWNED_CALL_PACKET_MANIFEST: PacketManifestEntry =
    PacketManifestEntry {
        id: Some(packet_ids::UNIT_TETHER_BLOCK_SPAWNED_CALL_PACKET),
        name: "UnitTetherBlockSpawnedCallPacket",
        transport: PacketTransport::NetPacket,
        direction: PacketDirection::ServerToClient,
        streamable: false,
        priority: Some(PacketPriority::Normal),
        allow_client_endpoint: true,
        allow_server_endpoint: false,
        force_uncompressed: false,
        codec: PacketCodecState::Implemented,
        upstream: "mindustry.gen.UnitTetherBlockSpawnedCallPacket",
        notes: "Generated by Call.registerPackets(); unit tether block spawned notification.",
    };

pub const UPDATE_GAME_OVER_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UPDATE_GAME_OVER_CALL_PACKET),
    name: "UpdateGameOverCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UpdateGameOverCallPacket",
    notes: "Generated by Call.registerPackets(); update game over winner notification.",
};

pub const UPDATE_MARKER_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UPDATE_MARKER_CALL_PACKET),
    name: "UpdateMarkerCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UpdateMarkerCallPacket",
    notes: "Generated by Call.registerPackets(); marker update payload.",
};

pub const UPDATE_MARKER_TEXT_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UPDATE_MARKER_TEXT_CALL_PACKET),
    name: "UpdateMarkerTextCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UpdateMarkerTextCallPacket",
    notes: "Generated by Call.registerPackets(); marker text update payload.",
};

pub const UPDATE_MARKER_TEXTURE_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::UPDATE_MARKER_TEXTURE_CALL_PACKET),
    name: "UpdateMarkerTextureCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.UpdateMarkerTextureCallPacket",
    notes: "Generated by Call.registerPackets(); marker texture update payload.",
};

pub const WARNING_TOAST_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::WARNING_TOAST_CALL_PACKET),
    name: "WarningToastCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
    force_uncompressed: false,
    codec: PacketCodecState::Implemented,
    upstream: "mindustry.gen.WarningToastCallPacket",
    notes: "Generated by Call.registerPackets(); warning toast notification.",
};

pub const WORLD_DATA_BEGIN_CALL_PACKET_MANIFEST: PacketManifestEntry = PacketManifestEntry {
    id: Some(packet_ids::WORLD_DATA_BEGIN_CALL_PACKET),
    name: "WorldDataBeginCallPacket",
    transport: PacketTransport::NetPacket,
    direction: PacketDirection::ServerToClient,
    streamable: false,
    priority: Some(PacketPriority::Normal),
    allow_client_endpoint: true,
    allow_server_endpoint: false,
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

pub const REGISTERED_PACKET_MANIFEST: [PacketManifestEntry; 142] = [
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
    CREATE_MARKER_CALL_PACKET_MANIFEST,
    CREATE_WEATHER_CALL_PACKET_MANIFEST,
    DEBUG_STATUS_CLIENT_CALL_PACKET_MANIFEST,
    DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET_MANIFEST,
    DECONSTRUCT_FINISH_CALL_PACKET_MANIFEST,
    DELETE_PLANS_CALL_PACKET_MANIFEST,
    DESTROY_PAYLOAD_CALL_PACKET_MANIFEST,
    DROP_ITEM_CALL_PACKET_MANIFEST,
    EFFECT_CALL_PACKET_MANIFEST,
    EFFECT_CALL_PACKET2_MANIFEST,
    EFFECT_RELIABLE_CALL_PACKET_MANIFEST,
    ENTITY_SNAPSHOT_CALL_PACKET_MANIFEST,
    FOLLOW_UP_MENU_CALL_PACKET_MANIFEST,
    GAME_OVER_CALL_PACKET_MANIFEST,
    HIDDEN_SNAPSHOT_CALL_PACKET_MANIFEST,
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
    LANDING_PAD_LANDED_CALL_PACKET_MANIFEST,
    LOGIC_EXPLOSION_CALL_PACKET_MANIFEST,
    MENU_CALL_PACKET_MANIFEST,
    MENU_CHOOSE_CALL_PACKET_MANIFEST,
    OPEN_URI_CALL_PACKET_MANIFEST,
    PAYLOAD_DROPPED_CALL_PACKET_MANIFEST,
    PICKED_BUILD_PAYLOAD_CALL_PACKET_MANIFEST,
    PICKED_UNIT_PAYLOAD_CALL_PACKET_MANIFEST,
    PING_CALL_PACKET_MANIFEST,
    PING_LOCATION_CALL_PACKET_MANIFEST,
    PING_RESPONSE_CALL_PACKET_MANIFEST,
    PLAYER_DISCONNECT_CALL_PACKET_MANIFEST,
    PLAYER_SPAWN_CALL_PACKET_MANIFEST,
    REMOVE_MARKER_CALL_PACKET_MANIFEST,
    REMOVE_QUEUE_BLOCK_CALL_PACKET_MANIFEST,
    REMOVE_TILE_CALL_PACKET_MANIFEST,
    REMOVE_WORLD_LABEL_CALL_PACKET_MANIFEST,
    REQUEST_BLOCK_SNAPSHOT_CALL_PACKET_MANIFEST,
    REQUEST_BUILD_PAYLOAD_CALL_PACKET_MANIFEST,
    REQUEST_DEBUG_STATUS_CALL_PACKET_MANIFEST,
    REQUEST_DROP_PAYLOAD_CALL_PACKET_MANIFEST,
    REQUEST_ITEM_CALL_PACKET_MANIFEST,
    REQUEST_UNIT_PAYLOAD_CALL_PACKET_MANIFEST,
    RESEARCHED_CALL_PACKET_MANIFEST,
    ROTATE_BLOCK_CALL_PACKET_MANIFEST,
    SECTOR_CAPTURE_CALL_PACKET_MANIFEST,
    SEND_CHAT_MESSAGE_CALL_PACKET_MANIFEST,
    SEND_MESSAGE_CALL_PACKET_MANIFEST,
    SEND_MESSAGE_CALL_PACKET2_MANIFEST,
    SERVER_BINARY_PACKET_RELIABLE_CALL_PACKET_MANIFEST,
    SERVER_BINARY_PACKET_UNRELIABLE_CALL_PACKET_MANIFEST,
    SERVER_PACKET_RELIABLE_CALL_PACKET_MANIFEST,
    SERVER_PACKET_UNRELIABLE_CALL_PACKET_MANIFEST,
    SET_CAMERA_POSITION_CALL_PACKET_MANIFEST,
    SET_FLAG_CALL_PACKET_MANIFEST,
    SET_FLOOR_CALL_PACKET_MANIFEST,
    SET_HUD_TEXT_CALL_PACKET_MANIFEST,
    SET_HUD_TEXT_RELIABLE_CALL_PACKET_MANIFEST,
    SET_ITEM_CALL_PACKET_MANIFEST,
    SET_ITEMS_CALL_PACKET_MANIFEST,
    SET_LIQUID_CALL_PACKET_MANIFEST,
    SET_LIQUIDS_CALL_PACKET_MANIFEST,
    SET_MAP_AREA_CALL_PACKET_MANIFEST,
    SET_OBJECTIVES_CALL_PACKET_MANIFEST,
    SET_OVERLAY_CALL_PACKET_MANIFEST,
    SET_PLAYER_TEAM_EDITOR_CALL_PACKET_MANIFEST,
    SET_POSITION_CALL_PACKET_MANIFEST,
    SET_RULE_CALL_PACKET_MANIFEST,
    SET_RULES_CALL_PACKET_MANIFEST,
    SET_TEAM_CALL_PACKET_MANIFEST,
    SET_TEAMS_CALL_PACKET_MANIFEST,
    SET_TILE_CALL_PACKET_MANIFEST,
    SET_TILE_BLOCKS_CALL_PACKET_MANIFEST,
    SET_TILE_FLOORS_CALL_PACKET_MANIFEST,
    SET_TILE_ITEMS_CALL_PACKET_MANIFEST,
    SET_TILE_LIQUIDS_CALL_PACKET_MANIFEST,
    SET_TILE_OVERLAYS_CALL_PACKET_MANIFEST,
    SET_UNIT_COMMAND_CALL_PACKET_MANIFEST,
    SET_UNIT_STANCE_CALL_PACKET_MANIFEST,
    SOUND_CALL_PACKET_MANIFEST,
    SOUND_AT_CALL_PACKET_MANIFEST,
    SPAWN_EFFECT_CALL_PACKET_MANIFEST,
    STATE_SNAPSHOT_CALL_PACKET_MANIFEST,
    SYNC_VARIABLE_CALL_PACKET_MANIFEST,
    TAKE_ITEMS_CALL_PACKET_MANIFEST,
    TEXT_INPUT_CALL_PACKET_MANIFEST,
    TEXT_INPUT_CALL_PACKET2_MANIFEST,
    TEXT_INPUT_RESULT_CALL_PACKET_MANIFEST,
    TILE_CONFIG_CALL_PACKET_MANIFEST,
    TILE_TAP_CALL_PACKET_MANIFEST,
    TRACE_INFO_CALL_PACKET_MANIFEST,
    TRANSFER_INVENTORY_CALL_PACKET_MANIFEST,
    TRANSFER_ITEM_EFFECT_CALL_PACKET_MANIFEST,
    TRANSFER_ITEM_TO_CALL_PACKET_MANIFEST,
    TRANSFER_ITEM_TO_UNIT_CALL_PACKET_MANIFEST,
    UNIT_BLOCK_SPAWN_CALL_PACKET_MANIFEST,
    UNIT_BUILDING_CONTROL_SELECT_CALL_PACKET_MANIFEST,
    UNIT_CAP_DEATH_CALL_PACKET_MANIFEST,
    UNIT_CLEAR_CALL_PACKET_MANIFEST,
    UNIT_CONTROL_CALL_PACKET_MANIFEST,
    UNIT_DEATH_CALL_PACKET_MANIFEST,
    UNIT_DESPAWN_CALL_PACKET_MANIFEST,
    UNIT_DESTROY_CALL_PACKET_MANIFEST,
    UNIT_ENTERED_PAYLOAD_CALL_PACKET_MANIFEST,
    UNIT_ENV_DEATH_CALL_PACKET_MANIFEST,
    UNIT_SAFE_DEATH_CALL_PACKET_MANIFEST,
    UNIT_SPAWN_CALL_PACKET_MANIFEST,
    UNIT_TETHER_BLOCK_SPAWNED_CALL_PACKET_MANIFEST,
    UPDATE_GAME_OVER_CALL_PACKET_MANIFEST,
    UPDATE_MARKER_CALL_PACKET_MANIFEST,
    UPDATE_MARKER_TEXT_CALL_PACKET_MANIFEST,
    UPDATE_MARKER_TEXTURE_CALL_PACKET_MANIFEST,
    WARNING_TOAST_CALL_PACKET_MANIFEST,
    WORLD_DATA_BEGIN_CALL_PACKET_MANIFEST,
];

pub const PACKET_MANIFEST: [PacketManifestEntry; 146] = [
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
    CREATE_MARKER_CALL_PACKET_MANIFEST,
    CREATE_WEATHER_CALL_PACKET_MANIFEST,
    DEBUG_STATUS_CLIENT_CALL_PACKET_MANIFEST,
    DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET_MANIFEST,
    DECONSTRUCT_FINISH_CALL_PACKET_MANIFEST,
    DELETE_PLANS_CALL_PACKET_MANIFEST,
    DESTROY_PAYLOAD_CALL_PACKET_MANIFEST,
    DROP_ITEM_CALL_PACKET_MANIFEST,
    EFFECT_CALL_PACKET_MANIFEST,
    EFFECT_CALL_PACKET2_MANIFEST,
    EFFECT_RELIABLE_CALL_PACKET_MANIFEST,
    ENTITY_SNAPSHOT_CALL_PACKET_MANIFEST,
    FOLLOW_UP_MENU_CALL_PACKET_MANIFEST,
    GAME_OVER_CALL_PACKET_MANIFEST,
    HIDDEN_SNAPSHOT_CALL_PACKET_MANIFEST,
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
    LANDING_PAD_LANDED_CALL_PACKET_MANIFEST,
    LOGIC_EXPLOSION_CALL_PACKET_MANIFEST,
    MENU_CALL_PACKET_MANIFEST,
    MENU_CHOOSE_CALL_PACKET_MANIFEST,
    OPEN_URI_CALL_PACKET_MANIFEST,
    PAYLOAD_DROPPED_CALL_PACKET_MANIFEST,
    PICKED_BUILD_PAYLOAD_CALL_PACKET_MANIFEST,
    PICKED_UNIT_PAYLOAD_CALL_PACKET_MANIFEST,
    PING_CALL_PACKET_MANIFEST,
    PING_LOCATION_CALL_PACKET_MANIFEST,
    PING_RESPONSE_CALL_PACKET_MANIFEST,
    PLAYER_DISCONNECT_CALL_PACKET_MANIFEST,
    PLAYER_SPAWN_CALL_PACKET_MANIFEST,
    REMOVE_MARKER_CALL_PACKET_MANIFEST,
    REMOVE_QUEUE_BLOCK_CALL_PACKET_MANIFEST,
    REMOVE_TILE_CALL_PACKET_MANIFEST,
    REMOVE_WORLD_LABEL_CALL_PACKET_MANIFEST,
    REQUEST_BLOCK_SNAPSHOT_CALL_PACKET_MANIFEST,
    REQUEST_BUILD_PAYLOAD_CALL_PACKET_MANIFEST,
    REQUEST_DEBUG_STATUS_CALL_PACKET_MANIFEST,
    REQUEST_DROP_PAYLOAD_CALL_PACKET_MANIFEST,
    REQUEST_ITEM_CALL_PACKET_MANIFEST,
    REQUEST_UNIT_PAYLOAD_CALL_PACKET_MANIFEST,
    RESEARCHED_CALL_PACKET_MANIFEST,
    ROTATE_BLOCK_CALL_PACKET_MANIFEST,
    SECTOR_CAPTURE_CALL_PACKET_MANIFEST,
    SEND_CHAT_MESSAGE_CALL_PACKET_MANIFEST,
    SEND_MESSAGE_CALL_PACKET_MANIFEST,
    SEND_MESSAGE_CALL_PACKET2_MANIFEST,
    SERVER_BINARY_PACKET_RELIABLE_CALL_PACKET_MANIFEST,
    SERVER_BINARY_PACKET_UNRELIABLE_CALL_PACKET_MANIFEST,
    SERVER_PACKET_RELIABLE_CALL_PACKET_MANIFEST,
    SERVER_PACKET_UNRELIABLE_CALL_PACKET_MANIFEST,
    SET_CAMERA_POSITION_CALL_PACKET_MANIFEST,
    SET_FLAG_CALL_PACKET_MANIFEST,
    SET_FLOOR_CALL_PACKET_MANIFEST,
    SET_HUD_TEXT_CALL_PACKET_MANIFEST,
    SET_HUD_TEXT_RELIABLE_CALL_PACKET_MANIFEST,
    SET_ITEM_CALL_PACKET_MANIFEST,
    SET_ITEMS_CALL_PACKET_MANIFEST,
    SET_LIQUID_CALL_PACKET_MANIFEST,
    SET_LIQUIDS_CALL_PACKET_MANIFEST,
    SET_MAP_AREA_CALL_PACKET_MANIFEST,
    SET_OBJECTIVES_CALL_PACKET_MANIFEST,
    SET_OVERLAY_CALL_PACKET_MANIFEST,
    SET_PLAYER_TEAM_EDITOR_CALL_PACKET_MANIFEST,
    SET_POSITION_CALL_PACKET_MANIFEST,
    SET_RULE_CALL_PACKET_MANIFEST,
    SET_RULES_CALL_PACKET_MANIFEST,
    SET_TEAM_CALL_PACKET_MANIFEST,
    SET_TEAMS_CALL_PACKET_MANIFEST,
    SET_TILE_CALL_PACKET_MANIFEST,
    SET_TILE_BLOCKS_CALL_PACKET_MANIFEST,
    SET_TILE_FLOORS_CALL_PACKET_MANIFEST,
    SET_TILE_ITEMS_CALL_PACKET_MANIFEST,
    SET_TILE_LIQUIDS_CALL_PACKET_MANIFEST,
    SET_TILE_OVERLAYS_CALL_PACKET_MANIFEST,
    SET_UNIT_COMMAND_CALL_PACKET_MANIFEST,
    SET_UNIT_STANCE_CALL_PACKET_MANIFEST,
    SOUND_CALL_PACKET_MANIFEST,
    SOUND_AT_CALL_PACKET_MANIFEST,
    SPAWN_EFFECT_CALL_PACKET_MANIFEST,
    STATE_SNAPSHOT_CALL_PACKET_MANIFEST,
    SYNC_VARIABLE_CALL_PACKET_MANIFEST,
    TAKE_ITEMS_CALL_PACKET_MANIFEST,
    TEXT_INPUT_CALL_PACKET_MANIFEST,
    TEXT_INPUT_CALL_PACKET2_MANIFEST,
    TEXT_INPUT_RESULT_CALL_PACKET_MANIFEST,
    TILE_CONFIG_CALL_PACKET_MANIFEST,
    TILE_TAP_CALL_PACKET_MANIFEST,
    TRACE_INFO_CALL_PACKET_MANIFEST,
    TRANSFER_INVENTORY_CALL_PACKET_MANIFEST,
    TRANSFER_ITEM_EFFECT_CALL_PACKET_MANIFEST,
    TRANSFER_ITEM_TO_CALL_PACKET_MANIFEST,
    TRANSFER_ITEM_TO_UNIT_CALL_PACKET_MANIFEST,
    UNIT_BLOCK_SPAWN_CALL_PACKET_MANIFEST,
    UNIT_BUILDING_CONTROL_SELECT_CALL_PACKET_MANIFEST,
    UNIT_CAP_DEATH_CALL_PACKET_MANIFEST,
    UNIT_CLEAR_CALL_PACKET_MANIFEST,
    UNIT_CONTROL_CALL_PACKET_MANIFEST,
    UNIT_DEATH_CALL_PACKET_MANIFEST,
    UNIT_DESPAWN_CALL_PACKET_MANIFEST,
    UNIT_DESTROY_CALL_PACKET_MANIFEST,
    UNIT_ENTERED_PAYLOAD_CALL_PACKET_MANIFEST,
    UNIT_ENV_DEATH_CALL_PACKET_MANIFEST,
    UNIT_SAFE_DEATH_CALL_PACKET_MANIFEST,
    UNIT_SPAWN_CALL_PACKET_MANIFEST,
    UNIT_TETHER_BLOCK_SPAWNED_CALL_PACKET_MANIFEST,
    UPDATE_GAME_OVER_CALL_PACKET_MANIFEST,
    UPDATE_MARKER_CALL_PACKET_MANIFEST,
    UPDATE_MARKER_TEXT_CALL_PACKET_MANIFEST,
    UPDATE_MARKER_TEXTURE_CALL_PACKET_MANIFEST,
    WARNING_TOAST_CALL_PACKET_MANIFEST,
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

#[derive(Debug, Clone, PartialEq)]
pub struct DeconstructFinishCallPacket {
    pub tile: Option<i32>,
    pub block: Option<String>,
    pub builder: UnitRef,
}

impl DeconstructFinishCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            tile: read_tile_pos(read)?,
            block: read_block(read, loader)?,
            builder: read_unit_ref(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_tile_pos(write, self.tile)?;
        write_block(write, loader, self.block.as_deref())?;
        write_unit_ref(write, self.builder)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletePlansCallPacket {
    /// Present on server-forwarded packets read by clients; omitted on
    /// client-origin packets read by the server, matching upstream's
    /// `Vars.net.server()`/`Vars.net.client()` branch in generated code.
    pub player_id: Option<i32>,
    pub positions: Vec<i32>,
}

impl DeletePlansCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player_id: None,
            positions: read_ints(read)?,
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_ints(write, &self.positions)
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let raw_player_id = read_i32(read)?;
        Ok(Self {
            player_id: (raw_player_id >= 0).then_some(raw_player_id),
            positions: read_ints(read)?,
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.player_id.unwrap_or(-1))?;
        write_ints(write, &self.positions)
    }
}

impl PacketCodec for DeletePlansCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DestroyPayloadCallPacket {
    pub build_pos: Option<i32>,
}

impl PacketCodec for DestroyPayloadCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let raw = read_i32(read)?;
        Ok(Self {
            build_pos: (raw >= 0).then_some(raw),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.build_pos.unwrap_or(-1))
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateMarkerCallPacket {
    pub id: i32,
    pub marker_json: String,
}

impl PacketCodec for CreateMarkerCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            id: read_i32(read)?,
            marker_json: read_objective_marker_json(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.id)?;
        write_objective_marker_json(write, &self.marker_json)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreateWeatherCallPacket {
    pub weather_id: Option<ContentId>,
    pub intensity: f32,
    pub duration: f32,
    pub wind_x: f32,
    pub wind_y: f32,
}

impl PacketCodec for CreateWeatherCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            weather_id: read_content_id(read)?,
            intensity: read_f32(read)?,
            duration: read_f32(read)?,
            wind_x: read_f32(read)?,
            wind_y: read_f32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_content_id(write, ContentType::Weather, self.weather_id)?;
        write_f32(write, self.intensity)?;
        write_f32(write, self.duration)?;
        write_f32(write, self.wind_x)?;
        write_f32(write, self.wind_y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropItemCallPacket {
    pub angle: f32,
}

impl PacketCodec for DropItemCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            angle: read_f32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_f32(write, self.angle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectCallPacket {
    pub effect_id: u16,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub color: RgbaColor,
}

impl EffectCallPacket {
    fn read_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            effect_id: read_effect_id(read)?,
            x: read_f32(read)?,
            y: read_f32(read)?,
            rotation: read_f32(read)?,
            color: read_color(read)?,
        })
    }

    fn write_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_effect_id(write, self.effect_id as i16)?;
        write_f32(write, self.x)?;
        write_f32(write, self.y)?;
        write_f32(write, self.rotation)?;
        write_color(write, self.color)
    }
}

impl PacketCodec for EffectCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectCallPacket2 {
    pub effect: EffectCallPacket,
    pub data: TypeValue,
}

impl PacketCodec for EffectCallPacket2 {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            effect: EffectCallPacket::read_payload(read)?,
            data: read_object_safe(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.effect.write_payload(write)?;
        write_object(write, &self.data)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectReliableCallPacket(pub EffectCallPacket);

impl PacketCodec for EffectReliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        EffectCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitySnapshotCallPacket {
    pub amount: i16,
    pub data: Vec<u8>,
}

impl PacketCodec for EntitySnapshotCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            amount: read_i16(read)?,
            data: read_bytes(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i16(write, self.amount)?;
        write_bytes(write, &self.data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FollowUpMenuCallPacket {
    pub menu_id: i32,
    pub title: Option<String>,
    pub message: Option<String>,
    pub options: Vec<Vec<Option<String>>>,
}

impl PacketCodec for FollowUpMenuCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            menu_id: read_i32(read)?,
            title: read_string(read)?,
            message: read_string(read)?,
            options: read_string_array(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.menu_id)?;
        write_string(write, self.title.as_deref())?;
        write_string(write, self.message.as_deref())?;
        let rows: Vec<Vec<Option<&str>>> = self
            .options
            .iter()
            .map(|row| row.iter().map(|value| value.as_deref()).collect())
            .collect();
        write_string_array(write, &rows)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameOverCallPacket {
    pub winner: TeamId,
}

impl PacketCodec for GameOverCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            winner: read_team(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_team(write, Some(self.winner))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HiddenSnapshotCallPacket {
    pub ids: Vec<i32>,
}

impl PacketCodec for HiddenSnapshotCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            ids: read_int_seq(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_int_seq(write, &self.ids)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandingPadLandedCallPacket {
    pub tile: Option<i32>,
}

impl PacketCodec for LandingPadLandedCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            tile: read_tile_pos(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_tile_pos(write, self.tile)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicExplosionCallPacket {
    pub team: TeamId,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub damage: f32,
    pub air: bool,
    pub ground: bool,
    pub pierce: bool,
    pub effect: bool,
}

impl PacketCodec for LogicExplosionCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            team: read_team(read)?,
            x: read_f32(read)?,
            y: read_f32(read)?,
            radius: read_f32(read)?,
            damage: read_f32(read)?,
            air: read_bool(read)?,
            ground: read_bool(read)?,
            pierce: read_bool(read)?,
            effect: read_bool(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_team(write, Some(self.team))?;
        write_f32(write, self.x)?;
        write_f32(write, self.y)?;
        write_f32(write, self.radius)?;
        write_f32(write, self.damage)?;
        write_bool(write, self.air)?;
        write_bool(write, self.ground)?;
        write_bool(write, self.pierce)?;
        write_bool(write, self.effect)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuCallPacket {
    pub menu_id: i32,
    pub title: String,
    pub message: String,
    pub options: Vec<Vec<Option<String>>>,
}

impl PacketCodec for MenuCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            menu_id: read_i32(read)?,
            title: read_string(read)?.unwrap_or_default(),
            message: read_string(read)?.unwrap_or_default(),
            options: read_string_array(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.menu_id)?;
        write_string(write, Some(&self.title))?;
        write_string(write, Some(&self.message))?;
        let rows: Vec<Vec<Option<&str>>> = self
            .options
            .iter()
            .map(|row| row.iter().map(|value| value.as_deref()).collect())
            .collect();
        write_string_array(write, &rows)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuChooseCallPacket {
    /// Present on server-forwarded packets read by clients; omitted on
    /// client-origin packets read by the server.
    pub player_id: Option<i32>,
    pub menu_id: i32,
    pub option: i32,
}

impl MenuChooseCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player_id: None,
            menu_id: read_i32(read)?,
            option: read_i32(read)?,
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.menu_id)?;
        write_i32(write, self.option)
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let raw_player_id = read_i32(read)?;
        Ok(Self {
            player_id: (raw_player_id >= 0).then_some(raw_player_id),
            menu_id: read_i32(read)?,
            option: read_i32(read)?,
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.player_id.unwrap_or(-1))?;
        write_i32(write, self.menu_id)?;
        write_i32(write, self.option)
    }
}

impl PacketCodec for MenuChooseCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PayloadDroppedCallPacket {
    pub unit: UnitRef,
    pub x: f32,
    pub y: f32,
}

impl PacketCodec for PayloadDroppedCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            unit: read_unit_ref(read)?,
            x: read_f32(read)?,
            y: read_f32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_unit_ref(write, self.unit)?;
        write_f32(write, self.x)?;
        write_f32(write, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickedBuildPayloadCallPacket {
    pub unit: UnitRef,
    pub build_pos: Option<i32>,
    pub on_ground: bool,
}

impl PacketCodec for PickedBuildPayloadCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let unit = read_unit_ref(read)?;
        let raw_build_pos = read_i32(read)?;
        Ok(Self {
            unit,
            build_pos: (raw_build_pos >= 0).then_some(raw_build_pos),
            on_ground: read_bool(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_unit_ref(write, self.unit)?;
        write_i32(write, self.build_pos.unwrap_or(-1))?;
        write_bool(write, self.on_ground)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickedUnitPayloadCallPacket {
    pub unit: UnitRef,
    pub target: UnitRef,
}

impl PacketCodec for PickedUnitPayloadCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            unit: read_unit_ref(read)?,
            target: read_unit_ref(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_unit_ref(write, self.unit)?;
        write_unit_ref(write, self.target)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PingCallPacket {
    pub time: i64,
}

impl PacketCodec for PingCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            time: read_i64(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i64(write, self.time)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingLocationCallPacket {
    /// Present on server-forwarded packets read by clients; omitted on
    /// client-origin packets read by the server.
    pub player_id: Option<i32>,
    pub x: f32,
    pub y: f32,
    pub text: String,
}

impl PingLocationCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player_id: None,
            x: read_f32(read)?,
            y: read_f32(read)?,
            text: read_string(read)?.unwrap_or_default(),
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_f32(write, self.x)?;
        write_f32(write, self.y)?;
        write_string(write, Some(&self.text))
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let raw_player_id = read_i32(read)?;
        Ok(Self {
            player_id: (raw_player_id >= 0).then_some(raw_player_id),
            x: read_f32(read)?,
            y: read_f32(read)?,
            text: read_string(read)?.unwrap_or_default(),
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.player_id.unwrap_or(-1))?;
        write_f32(write, self.x)?;
        write_f32(write, self.y)?;
        write_string(write, Some(&self.text))
    }
}

impl PacketCodec for PingLocationCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
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
pub struct PlayerSpawnCallPacket {
    pub tile: Option<i32>,
    pub player: EntityRef,
}

impl PacketCodec for PlayerSpawnCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            tile: read_tile_pos(read)?,
            player: read_entity_ref(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_tile_pos(write, self.tile)?;
        write_entity_ref(write, self.player)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveTileCallPacket {
    pub tile: Option<i32>,
}

impl PacketCodec for RemoveTileCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            tile: read_tile_pos(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_tile_pos(write, self.tile)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveWorldLabelCallPacket {
    pub id: i32,
}

impl PacketCodec for RemoveWorldLabelCallPacket {
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
pub struct RequestBlockSnapshotCallPacket {
    pub pos: i32,
}

impl PacketRuntime for RequestBlockSnapshotCallPacket {
    fn priority(&self) -> PacketPriority {
        PacketPriority::Low
    }

    fn allow(&self, server: bool) -> bool {
        server
    }
}

impl PacketCodec for RequestBlockSnapshotCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            pos: read_i32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.pos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestBuildPayloadCallPacket {
    /// Present on server-forwarded packets read by clients; omitted on
    /// client-origin packets read by the server.
    pub player: EntityRef,
    pub build: BuildingRef,
}

impl RequestBuildPayloadCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            build: read_building_ref(read)?,
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_building_ref(write, self.build)
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            build: read_building_ref(read)?,
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_building_ref(write, self.build)
    }
}

impl PacketCodec for RequestBuildPayloadCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestDebugStatusCallPacket;

impl PacketRuntime for RequestDebugStatusCallPacket {
    fn allow(&self, server: bool) -> bool {
        server
    }
}

impl PacketCodec for RequestDebugStatusCallPacket {
    fn read_from<R: Read>(_read: &mut R) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn write_to<W: Write>(&self, _write: &mut W) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RequestDropPayloadCallPacket {
    /// Present on server-forwarded packets read by clients; omitted on
    /// client-origin packets read by the server.
    pub player: EntityRef,
    pub x: f32,
    pub y: f32,
}

impl RequestDropPayloadCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            x: read_f32(read)?,
            y: read_f32(read)?,
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_f32(write, self.x)?;
        write_f32(write, self.y)
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            x: read_f32(read)?,
            y: read_f32(read)?,
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_f32(write, self.x)?;
        write_f32(write, self.y)
    }
}

impl PacketCodec for RequestDropPayloadCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestItemCallPacket {
    /// Present on server-forwarded packets read by clients; omitted on
    /// client-origin packets read by the server.
    pub player: EntityRef,
    pub build: BuildingRef,
    pub item: Option<String>,
    pub amount: i32,
}

impl RequestItemCallPacket {
    pub fn read_from_client_payload_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            build: read_building_ref(read)?,
            item: crate::mindustry::io::type_io::read_item(read, loader)?,
            amount: read_i32(read)?,
        })
    }

    pub fn write_client_payload_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_building_ref(write, self.build)?;
        crate::mindustry::io::type_io::write_item(write, loader, self.item.as_deref())?;
        write_i32(write, self.amount)
    }

    pub fn read_from_server_payload_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            build: read_building_ref(read)?,
            item: crate::mindustry::io::type_io::read_item(read, loader)?,
            amount: read_i32(read)?,
        })
    }

    pub fn write_server_payload_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_building_ref(write, self.build)?;
        crate::mindustry::io::type_io::write_item(write, loader, self.item.as_deref())?;
        write_i32(write, self.amount)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestUnitPayloadCallPacket {
    /// Present on server-forwarded packets read by clients; omitted on
    /// client-origin packets read by the server.
    pub player: EntityRef,
    pub target: UnitRef,
}

impl RequestUnitPayloadCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            target: read_unit_ref(read)?,
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_unit_ref(write, self.target)
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            target: read_unit_ref(read)?,
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_unit_ref(write, self.target)
    }
}

impl PacketCodec for RequestUnitPayloadCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchedCallPacket {
    pub content_type: ContentType,
    pub content: String,
}

impl ResearchedCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        let (content_type, content) = read_required_content_name(read, loader)?;
        Ok(Self {
            content_type,
            content,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_required_content_ref(write, loader, self.content_type, &self.content)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotateBlockCallPacket {
    /// Present on server-forwarded packets read by clients; omitted on
    /// client-origin packets read by the server.
    pub player: EntityRef,
    pub build: BuildingRef,
    pub direction: bool,
}

impl RotateBlockCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            build: read_building_ref(read)?,
            direction: read_bool(read)?,
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_building_ref(write, self.build)?;
        write_bool(write, self.direction)
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            build: read_building_ref(read)?,
            direction: read_bool(read)?,
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_building_ref(write, self.build)?;
        write_bool(write, self.direction)
    }
}

impl PacketCodec for RotateBlockCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SectorCaptureCallPacket;

impl PacketCodec for SectorCaptureCallPacket {
    fn read_from<R: Read>(_read: &mut R) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn write_to<W: Write>(&self, _write: &mut W) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendChatMessageCallPacket {
    pub message: String,
}

impl PacketCodec for SendChatMessageCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            message: read_string(read)?.unwrap_or_default(),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.message))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendMessageCallPacket {
    pub message: String,
}

impl PacketCodec for SendMessageCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            message: read_string(read)?.unwrap_or_default(),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.message))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendMessageCallPacket2 {
    pub message: String,
    pub unformatted: String,
    pub player_sender: EntityRef,
}

impl PacketCodec for SendMessageCallPacket2 {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            message: read_string(read)?.unwrap_or_default(),
            unformatted: read_string(read)?.unwrap_or_default(),
            player_sender: read_entity_ref(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.message))?;
        write_string(write, Some(&self.unformatted))?;
        write_entity_ref(write, self.player_sender)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerBinaryPacketCallPacket {
    pub packet_type: String,
    pub contents: Vec<u8>,
}

impl ServerBinaryPacketCallPacket {
    fn read_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            packet_type: read_string(read)?.unwrap_or_default(),
            contents: read_bytes(read)?,
        })
    }

    fn write_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_string(write, Some(&self.packet_type))?;
        write_bytes(write, &self.contents)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerBinaryPacketReliableCallPacket(pub ServerBinaryPacketCallPacket);

impl PacketCodec for ServerBinaryPacketReliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        ServerBinaryPacketCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerBinaryPacketUnreliableCallPacket(pub ServerBinaryPacketCallPacket);

impl PacketCodec for ServerBinaryPacketUnreliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        ServerBinaryPacketCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPacketCallPacket {
    pub packet_type: String,
    pub contents: String,
}

impl ServerPacketCallPacket {
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
pub struct ServerPacketReliableCallPacket(pub ServerPacketCallPacket);

impl PacketCodec for ServerPacketReliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        ServerPacketCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPacketUnreliableCallPacket(pub ServerPacketCallPacket);

impl PacketCodec for ServerPacketUnreliableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        ServerPacketCallPacket::read_payload(read).map(Self)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.0.write_payload(write)
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
pub struct SetFloorCallPacket {
    pub tile: Option<i32>,
    pub floor: Option<String>,
    pub overlay: Option<String>,
}

impl SetFloorCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            tile: read_tile_pos(read)?,
            floor: read_block(read, loader)?,
            overlay: read_block(read, loader)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_tile_pos(write, self.tile)?;
        write_block(write, loader, self.floor.as_deref())?;
        write_block(write, loader, self.overlay.as_deref())
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetItemCallPacket {
    pub build: BuildingRef,
    pub item: Option<String>,
    pub amount: i32,
}

impl SetItemCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            build: read_building_ref(read)?,
            item: read_item(read, loader)?,
            amount: read_i32(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_building_ref(write, self.build)?;
        write_item(write, loader, self.item.as_deref())?;
        write_i32(write, self.amount)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetItemsCallPacket {
    pub build: BuildingRef,
    pub items: Vec<ItemStack>,
}

impl SetItemsCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            build: read_building_ref(read)?,
            items: read_item_stacks(read, loader)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_building_ref(write, self.build)?;
        write_item_stacks(write, loader, &self.items)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetLiquidCallPacket {
    pub build: BuildingRef,
    pub liquid: Option<String>,
    pub amount: f32,
}

impl SetLiquidCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            build: read_building_ref(read)?,
            liquid: read_liquid(read, loader)?,
            amount: read_f32(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_building_ref(write, self.build)?;
        write_liquid(write, loader, self.liquid.as_deref())?;
        write_f32(write, self.amount)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetLiquidsCallPacket {
    pub build: BuildingRef,
    pub liquids: Vec<LiquidStack>,
}

impl SetLiquidsCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            build: read_building_ref(read)?,
            liquids: read_liquid_stacks(read, loader)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_building_ref(write, self.build)?;
        write_liquid_stacks(write, loader, &self.liquids)
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
pub struct SetObjectivesCallPacket {
    pub objectives_json: String,
}

impl PacketCodec for SetObjectivesCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            objectives_json: read_objectives_json(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_objectives_json(write, &self.objectives_json)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetOverlayCallPacket {
    pub tile: Option<i32>,
    pub overlay: Option<String>,
}

impl SetOverlayCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            tile: read_tile_pos(read)?,
            overlay: read_block(read, loader)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_tile_pos(write, self.tile)?;
        write_block(write, loader, self.overlay.as_deref())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetPlayerTeamEditorCallPacket {
    /// Present on server-forwarded packets read by clients; omitted on
    /// client-origin packets read by the server.
    pub player: EntityRef,
    pub team: TeamId,
}

impl SetPlayerTeamEditorCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            team: read_team(read)?,
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_team(write, Some(self.team))
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            team: read_team(read)?,
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_team(write, Some(self.team))
    }
}

impl PacketCodec for SetPlayerTeamEditorCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetPositionCallPacket {
    pub x: f32,
    pub y: f32,
}

impl PacketCodec for SetPositionCallPacket {
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
pub struct SetRulesCallPacket {
    pub rules_json: String,
}

impl PacketCodec for SetRulesCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            rules_json: read_rules_json(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_rules_json(write, &self.rules_json)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetTeamCallPacket {
    pub build: BuildingRef,
    pub team: TeamId,
}

impl PacketCodec for SetTeamCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            build: read_building_ref(read)?,
            team: read_team(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_building_ref(write, self.build)?;
        write_team(write, Some(self.team))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetTeamsCallPacket {
    pub positions: Vec<i32>,
    pub team: TeamId,
}

impl PacketCodec for SetTeamsCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            positions: read_ints(read)?,
            team: read_team(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_ints(write, &self.positions)?;
        write_team(write, Some(self.team))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetTileCallPacket {
    pub tile: Option<i32>,
    pub block: Option<String>,
    pub team: TeamId,
    pub rotation: i32,
}

impl SetTileCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            tile: read_tile_pos(read)?,
            block: read_block(read, loader)?,
            team: read_team(read)?,
            rotation: read_i32(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_tile_pos(write, self.tile)?;
        write_block(write, loader, self.block.as_deref())?;
        write_team(write, Some(self.team))?;
        write_i32(write, self.rotation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetTileBlocksCallPacket {
    pub block: Option<String>,
    pub team: TeamId,
    pub positions: Vec<i32>,
}

impl SetTileBlocksCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            block: read_block(read, loader)?,
            team: read_team(read)?,
            positions: read_ints(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_block(write, loader, self.block.as_deref())?;
        write_team(write, Some(self.team))?;
        write_ints(write, &self.positions)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetTileFloorsCallPacket {
    pub block: Option<String>,
    pub positions: Vec<i32>,
}

impl SetTileFloorsCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            block: read_block(read, loader)?,
            positions: read_ints(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_block(write, loader, self.block.as_deref())?;
        write_ints(write, &self.positions)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetTileItemsCallPacket {
    pub item: Option<String>,
    pub amount: i32,
    pub positions: Vec<i32>,
}

impl SetTileItemsCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            item: read_item(read, loader)?,
            amount: read_i32(read)?,
            positions: read_ints(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_item(write, loader, self.item.as_deref())?;
        write_i32(write, self.amount)?;
        write_ints(write, &self.positions)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetTileLiquidsCallPacket {
    pub liquid: Option<String>,
    pub amount: f32,
    pub positions: Vec<i32>,
}

impl SetTileLiquidsCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            liquid: read_liquid(read, loader)?,
            amount: read_f32(read)?,
            positions: read_ints(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_liquid(write, loader, self.liquid.as_deref())?;
        write_f32(write, self.amount)?;
        write_ints(write, &self.positions)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetTileOverlaysCallPacket {
    pub block: Option<String>,
    pub positions: Vec<i32>,
}

impl SetTileOverlaysCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            block: read_block(read, loader)?,
            positions: read_ints(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_block(write, loader, self.block.as_deref())?;
        write_ints(write, &self.positions)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetUnitCommandCallPacket {
    pub player: EntityRef,
    pub unit_ids: Vec<i32>,
    pub command: String,
}

impl SetUnitCommandCallPacket {
    pub fn read_from_client_payload_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            unit_ids: read_ints(read)?,
            command: read_content_name(read, loader, ContentType::UnitCommand)?.ok_or_else(
                || std::io::Error::new(std::io::ErrorKind::InvalidData, "null unit command id"),
            )?,
        })
    }

    pub fn write_client_payload_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_ints(write, &self.unit_ids)?;
        write_content_by_name(write, loader, ContentType::UnitCommand, Some(&self.command))
    }

    pub fn read_from_server_payload_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            unit_ids: read_ints(read)?,
            command: read_content_name(read, loader, ContentType::UnitCommand)?.ok_or_else(
                || std::io::Error::new(std::io::ErrorKind::InvalidData, "null unit command id"),
            )?,
        })
    }

    pub fn write_server_payload_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_ints(write, &self.unit_ids)?;
        write_content_by_name(write, loader, ContentType::UnitCommand, Some(&self.command))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetUnitStanceCallPacket {
    pub player: EntityRef,
    pub unit_ids: Vec<i32>,
    pub stance: String,
}

impl SetUnitStanceCallPacket {
    pub fn read_from_client_payload_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            unit_ids: read_ints(read)?,
            stance: read_content_name(read, loader, ContentType::UnitStance)?.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "null unit stance id")
            })?,
        })
    }

    pub fn write_client_payload_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_ints(write, &self.unit_ids)?;
        write_content_by_name(write, loader, ContentType::UnitStance, Some(&self.stance))
    }

    pub fn read_from_server_payload_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            unit_ids: read_ints(read)?,
            stance: read_content_name(read, loader, ContentType::UnitStance)?.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "null unit stance id")
            })?,
        })
    }

    pub fn write_server_payload_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_ints(write, &self.unit_ids)?;
        write_content_by_name(write, loader, ContentType::UnitStance, Some(&self.stance))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundCallPacket {
    pub sound_id: i16,
    pub volume: f32,
    pub pitch: f32,
    pub pan: f32,
}

impl PacketCodec for SoundCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            sound_id: read_sound_id(read)?,
            volume: read_f32(read)?,
            pitch: read_f32(read)?,
            pan: read_f32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_sound_id(write, self.sound_id)?;
        write_f32(write, self.volume)?;
        write_f32(write, self.pitch)?;
        write_f32(write, self.pan)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundAtCallPacket {
    pub sound_id: i16,
    pub x: f32,
    pub y: f32,
    pub volume: f32,
    pub pitch: f32,
}

impl PacketCodec for SoundAtCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            sound_id: read_sound_id(read)?,
            x: read_f32(read)?,
            y: read_f32(read)?,
            volume: read_f32(read)?,
            pitch: read_f32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_sound_id(write, self.sound_id)?;
        write_f32(write, self.x)?;
        write_f32(write, self.y)?;
        write_f32(write, self.volume)?;
        write_f32(write, self.pitch)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnEffectCallPacket {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub unit_type: String,
}

impl SpawnEffectCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            x: read_f32(read)?,
            y: read_f32(read)?,
            rotation: read_f32(read)?,
            unit_type: read_unit_type(read, loader)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_f32(write, self.x)?;
        write_f32(write, self.y)?;
        write_f32(write, self.rotation)?;
        write_unit_type(write, loader, &self.unit_type)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateSnapshotCallPacket {
    pub wave_time: f32,
    pub wave: i32,
    pub enemies: i32,
    pub paused: bool,
    pub game_over: bool,
    pub time_data: i32,
    pub tps: u8,
    pub rand0: i64,
    pub rand1: i64,
    pub core_data: Vec<u8>,
}

impl PacketCodec for StateSnapshotCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            wave_time: read_f32(read)?,
            wave: read_i32(read)?,
            enemies: read_i32(read)?,
            paused: read_bool(read)?,
            game_over: read_bool(read)?,
            time_data: read_i32(read)?,
            tps: read_u8(read)?,
            rand0: read_i64(read)?,
            rand1: read_i64(read)?,
            core_data: read_bytes(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_f32(write, self.wave_time)?;
        write_i32(write, self.wave)?;
        write_i32(write, self.enemies)?;
        write_bool(write, self.paused)?;
        write_bool(write, self.game_over)?;
        write_i32(write, self.time_data)?;
        write_u8(write, self.tps)?;
        write_i64(write, self.rand0)?;
        write_i64(write, self.rand1)?;
        write_bytes(write, &self.core_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyncVariableCallPacket {
    pub building: BuildingRef,
    pub variable: i32,
    pub value: TypeValue,
}

impl PacketCodec for SyncVariableCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            building: read_building_ref(read)?,
            variable: read_i32(read)?,
            value: read_object_safe(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_building_ref(write, self.building)?;
        write_i32(write, self.variable)?;
        write_object(write, &self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TakeItemsCallPacket {
    pub build: BuildingRef,
    pub item: Option<String>,
    pub amount: i32,
    pub to: UnitRef,
}

impl TakeItemsCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            build: read_building_ref(read)?,
            item: read_item(read, loader)?,
            amount: read_i32(read)?,
            to: read_unit_ref(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_building_ref(write, self.build)?;
        write_item(write, loader, self.item.as_deref())?;
        write_i32(write, self.amount)?;
        write_unit_ref(write, self.to)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputResultCallPacket {
    pub player: EntityRef,
    pub text_input_id: i32,
    pub text: String,
}

impl TextInputResultCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            text_input_id: read_i32(read)?,
            text: read_string(read)?.unwrap_or_default(),
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.text_input_id)?;
        write_string(write, Some(&self.text))
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            text_input_id: read_i32(read)?,
            text: read_string(read)?.unwrap_or_default(),
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_i32(write, self.text_input_id)?;
        write_string(write, Some(&self.text))
    }
}

impl PacketCodec for TextInputResultCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileConfigCallPacket {
    pub player: EntityRef,
    pub build: BuildingRef,
    pub value: TypeValue,
}

impl TileConfigCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            build: read_building_ref(read)?,
            value: read_object_safe(read)?,
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_building_ref(write, self.build)?;
        write_object(write, &self.value)
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            build: read_building_ref(read)?,
            value: read_object_safe(read)?,
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_building_ref(write, self.build)?;
        write_object(write, &self.value)
    }
}

impl PacketCodec for TileConfigCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileTapCallPacket {
    pub player: EntityRef,
    pub tile: Option<i32>,
}

impl TileTapCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            tile: read_tile_pos(read)?,
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_tile_pos(write, self.tile)
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            tile: read_tile_pos(read)?,
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_tile_pos(write, self.tile)
    }
}

impl PacketCodec for TileTapCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceInfoCallPacket {
    pub player: EntityRef,
    pub info: NetTraceInfo,
}

impl PacketCodec for TraceInfoCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            info: read_trace_info(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_trace_info(write, &self.info)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferInventoryCallPacket {
    pub player: EntityRef,
    pub build: BuildingRef,
}

impl TransferInventoryCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            build: read_building_ref(read)?,
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_building_ref(write, self.build)
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            build: read_building_ref(read)?,
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_building_ref(write, self.build)
    }
}

impl PacketCodec for TransferInventoryCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransferItemEffectCallPacket {
    pub item: Option<String>,
    pub x: f32,
    pub y: f32,
    pub to: EntityRef,
}

impl TransferItemEffectCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            item: read_item(read, loader)?,
            x: read_f32(read)?,
            y: read_f32(read)?,
            to: read_entity_ref(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_item(write, loader, self.item.as_deref())?;
        write_f32(write, self.x)?;
        write_f32(write, self.y)?;
        write_entity_ref(write, self.to)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransferItemToCallPacket {
    pub unit: UnitRef,
    pub item: Option<String>,
    pub amount: i32,
    pub x: f32,
    pub y: f32,
    pub build: BuildingRef,
}

impl TransferItemToCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            unit: read_unit_ref(read)?,
            item: read_item(read, loader)?,
            amount: read_i32(read)?,
            x: read_f32(read)?,
            y: read_f32(read)?,
            build: read_building_ref(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_unit_ref(write, self.unit)?;
        write_item(write, loader, self.item.as_deref())?;
        write_i32(write, self.amount)?;
        write_f32(write, self.x)?;
        write_f32(write, self.y)?;
        write_building_ref(write, self.build)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransferItemToUnitCallPacket {
    pub item: Option<String>,
    pub x: f32,
    pub y: f32,
    pub to: EntityRef,
}

impl TransferItemToUnitCallPacket {
    pub fn read_from_with_loader<R: Read>(
        read: &mut R,
        loader: &ContentLoader,
    ) -> std::io::Result<Self> {
        Ok(Self {
            item: read_item(read, loader)?,
            x: read_f32(read)?,
            y: read_f32(read)?,
            to: read_entity_ref(read)?,
        })
    }

    pub fn write_to_with_loader<W: Write>(
        &self,
        write: &mut W,
        loader: &ContentLoader,
    ) -> std::io::Result<()> {
        write_item(write, loader, self.item.as_deref())?;
        write_f32(write, self.x)?;
        write_f32(write, self.y)?;
        write_entity_ref(write, self.to)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitBlockSpawnCallPacket {
    pub tile: Option<i32>,
}

impl PacketCodec for UnitBlockSpawnCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            tile: read_tile_pos(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_tile_pos(write, self.tile)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitBuildingControlSelectCallPacket {
    pub unit: UnitRef,
    pub build: BuildingRef,
}

impl PacketCodec for UnitBuildingControlSelectCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            unit: read_unit_ref(read)?,
            build: read_building_ref(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_unit_ref(write, self.unit)?;
        write_building_ref(write, self.build)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitCapDeathCallPacket {
    pub unit: UnitRef,
}

impl PacketCodec for UnitCapDeathCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            unit: read_unit_ref(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_unit_ref(write, self.unit)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitClearCallPacket {
    pub player: EntityRef,
}

impl UnitClearCallPacket {
    pub fn read_from_client_payload<R: Read>(_read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
        })
    }

    pub fn write_client_payload<W: Write>(&self, _write: &mut W) -> std::io::Result<()> {
        Ok(())
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_entity_ref(write, self.player)
    }
}

impl PacketCodec for UnitClearCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitControlCallPacket {
    pub player: EntityRef,
    pub unit: UnitRef,
}

impl UnitControlCallPacket {
    pub fn read_from_client_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: EntityRef::null(),
            unit: read_unit_ref(read)?,
        })
    }

    pub fn write_client_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_unit_ref(write, self.unit)
    }

    pub fn read_from_server_payload<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            player: read_entity_ref(read)?,
            unit: read_unit_ref(read)?,
        })
    }

    pub fn write_server_payload<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_entity_ref(write, self.player)?;
        write_unit_ref(write, self.unit)
    }
}

impl PacketCodec for UnitControlCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Self::read_from_client_payload(read)
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.write_client_payload(write)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitDeathCallPacket {
    pub uid: i32,
}

impl PacketCodec for UnitDeathCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            uid: read_i32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.uid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitDespawnCallPacket {
    pub unit: UnitRef,
}

impl PacketCodec for UnitDespawnCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            unit: read_unit_ref(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_unit_ref(write, self.unit)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitDestroyCallPacket {
    pub uid: i32,
}

impl PacketCodec for UnitDestroyCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            uid: read_i32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.uid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitEnteredPayloadCallPacket {
    pub unit: UnitRef,
    pub build: BuildingRef,
}

impl PacketCodec for UnitEnteredPayloadCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            unit: read_unit_ref(read)?,
            build: read_building_ref(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_unit_ref(write, self.unit)?;
        write_building_ref(write, self.build)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitEnvDeathCallPacket {
    pub unit: UnitRef,
}

impl PacketCodec for UnitEnvDeathCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            unit: read_unit_ref(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_unit_ref(write, self.unit)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitSafeDeathCallPacket {
    pub unit: UnitRef,
}

impl PacketCodec for UnitSafeDeathCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            unit: read_unit_ref(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_unit_ref(write, self.unit)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitSpawnCallPacket {
    pub container: UnitSyncContainer,
}

impl PacketCodec for UnitSpawnCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            container: read_unit_container(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_unit_container(write, &self.container)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitTetherBlockSpawnedCallPacket {
    pub tile: Option<i32>,
    pub id: i32,
}

impl PacketCodec for UnitTetherBlockSpawnedCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            tile: read_tile_pos(read)?,
            id: read_i32(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_tile_pos(write, self.tile)?;
        write_i32(write, self.id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateGameOverCallPacket {
    pub winner: TeamId,
}

impl PacketCodec for UpdateGameOverCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            winner: read_team(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_team(write, Some(self.winner))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateMarkerCallPacket {
    pub id: i32,
    pub control: crate::mindustry::logic::LMarkerControl,
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
}

impl PacketCodec for UpdateMarkerCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            id: read_i32(read)?,
            control: read_marker_control(read)?,
            p1: f64::from_bits(read_u64(read)?),
            p2: f64::from_bits(read_u64(read)?),
            p3: f64::from_bits(read_u64(read)?),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.id)?;
        write_marker_control(write, self.control)?;
        write_u64(write, self.p1.to_bits())?;
        write_u64(write, self.p2.to_bits())?;
        write_u64(write, self.p3.to_bits())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMarkerTextCallPacket {
    pub id: i32,
    pub r#type: crate::mindustry::logic::LMarkerControl,
    pub fetch: bool,
    pub text: String,
}

impl PacketCodec for UpdateMarkerTextCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            id: read_i32(read)?,
            r#type: read_marker_control(read)?,
            fetch: read_u8(read)? != 0,
            text: read_string(read)?.unwrap_or_default(),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.id)?;
        write_marker_control(write, self.r#type)?;
        write_u8(write, self.fetch as u8)?;
        write_string(write, Some(&self.text))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateMarkerTextureCallPacket {
    pub id: i32,
    pub texture: TypeValue,
}

impl PacketCodec for UpdateMarkerTextureCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            id: read_i32(read)?,
            texture: read_object_safe(read)?,
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.id)?;
        write_object(write, &self.texture)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarningToastCallPacket {
    pub unicode: i32,
    pub text: String,
}

impl PacketCodec for WarningToastCallPacket {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            unicode: read_i32(read)?,
            text: read_string(read)?.unwrap_or_default(),
        })
    }

    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        write_i32(write, self.unicode)?;
        write_string(write, Some(&self.text))
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

        let marker = CreateMarkerCallPacket {
            id: 99,
            marker_json: r#"{"type":"Point","x":4,"y":5}"#.into(),
        };
        let mut bytes = Vec::new();
        marker.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&99i32.to_be_bytes());
        let marker_json = marker.marker_json.as_bytes();
        expected.extend_from_slice(&(marker_json.len() as i32).to_be_bytes());
        expected.extend_from_slice(marker_json);
        assert_eq!(bytes, expected);
        assert_eq!(
            CreateMarkerCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            marker
        );

        let weather = CreateWeatherCallPacket {
            weather_id: Some(1),
            intensity: 0.8,
            duration: 120.0,
            wind_x: -0.25,
            wind_y: 0.5,
        };
        let mut bytes = Vec::new();
        weather.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1i16.to_be_bytes());
        for value in [0.8f32, 120.0, -0.25, 0.5] {
            expected.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        assert_eq!(bytes, expected);
        assert_eq!(
            CreateWeatherCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            weather
        );

        let null_weather = CreateWeatherCallPacket {
            weather_id: None,
            intensity: 0.0,
            duration: 0.0,
            wind_x: 0.0,
            wind_y: 0.0,
        };
        bytes.clear();
        null_weather.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&(-1i16).to_be_bytes());
        expected.extend_from_slice(&0.0f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&0.0f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&0.0f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&0.0f32.to_bits().to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            CreateWeatherCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            null_weather
        );

        let effect = EffectCallPacket {
            effect_id: 0x1234,
            x: 1.25,
            y: -2.5,
            rotation: 90.0,
            color: RgbaColor::new(0x11223344),
        };
        let mut bytes = Vec::new();
        effect.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&0x1234i16.to_be_bytes());
        for value in [1.25f32, -2.5, 90.0] {
            expected.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        expected.extend_from_slice(&0x11223344i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            EffectCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            effect
        );

        let effect_with_data = EffectCallPacket2 {
            effect,
            data: TypeValue::Int(42),
        };
        let mut bytes = Vec::new();
        effect_with_data.write_to(&mut bytes).unwrap();
        let mut expected_with_data = expected;
        expected_with_data.push(1);
        expected_with_data.extend_from_slice(&42i32.to_be_bytes());
        assert_eq!(bytes, expected_with_data);
        assert_eq!(
            EffectCallPacket2::read_from(&mut bytes.as_slice()).unwrap(),
            effect_with_data
        );

        let reliable_effect = EffectReliableCallPacket(effect);
        let mut bytes = Vec::new();
        reliable_effect.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        effect.write_to(&mut expected).unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(
            EffectReliableCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            reliable_effect
        );

        let drop_item = DropItemCallPacket { angle: -45.5 };
        let mut bytes = Vec::new();
        drop_item.write_to(&mut bytes).unwrap();
        assert_eq!(bytes, (-45.5f32).to_bits().to_be_bytes());
        assert_eq!(
            DropItemCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            drop_item
        );

        let delete_plans = DeletePlansCallPacket {
            player_id: Some(123),
            positions: vec![
                crate::mindustry::world::point2_pack(1, 2),
                crate::mindustry::world::point2_pack(3, 4),
            ],
        };
        let mut bytes = Vec::new();
        delete_plans.write_client_payload(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&2i16.to_be_bytes());
        expected.extend_from_slice(&crate::mindustry::world::point2_pack(1, 2).to_be_bytes());
        expected.extend_from_slice(&crate::mindustry::world::point2_pack(3, 4).to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            DeletePlansCallPacket::read_from_client_payload(&mut bytes.as_slice()).unwrap(),
            DeletePlansCallPacket {
                player_id: None,
                positions: delete_plans.positions.clone()
            }
        );

        let mut bytes = Vec::new();
        delete_plans.write_server_payload(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&123i32.to_be_bytes());
        expected.extend_from_slice(&2i16.to_be_bytes());
        expected.extend_from_slice(&crate::mindustry::world::point2_pack(1, 2).to_be_bytes());
        expected.extend_from_slice(&crate::mindustry::world::point2_pack(3, 4).to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            DeletePlansCallPacket::read_from_server_payload(&mut bytes.as_slice()).unwrap(),
            delete_plans
        );

        let no_player = DeletePlansCallPacket {
            player_id: None,
            positions: Vec::new(),
        };
        bytes.clear();
        no_player.write_server_payload(&mut bytes).unwrap();
        assert_eq!(bytes, vec![255, 255, 255, 255, 0, 0]);
        assert_eq!(
            DeletePlansCallPacket::read_from_server_payload(&mut bytes.as_slice()).unwrap(),
            no_player
        );

        let destroy_payload = DestroyPayloadCallPacket {
            build_pos: Some(crate::mindustry::world::point2_pack(5, 6)),
        };
        let mut bytes = Vec::new();
        destroy_payload.write_to(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            crate::mindustry::world::point2_pack(5, 6).to_be_bytes()
        );
        assert_eq!(
            DestroyPayloadCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            destroy_payload
        );

        let null_destroy_payload = DestroyPayloadCallPacket { build_pos: None };
        bytes.clear();
        null_destroy_payload.write_to(&mut bytes).unwrap();
        assert_eq!(bytes, (-1i32).to_be_bytes());
        assert_eq!(
            DestroyPayloadCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            null_destroy_payload
        );

        let entity_snapshot = EntitySnapshotCallPacket {
            amount: 2,
            data: vec![9, 8, 7],
        };
        let mut bytes = Vec::new();
        entity_snapshot.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&2i16.to_be_bytes());
        expected.extend_from_slice(&3i16.to_be_bytes());
        expected.extend_from_slice(&[9, 8, 7]);
        assert_eq!(bytes, expected);
        assert_eq!(
            EntitySnapshotCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            entity_snapshot
        );

        let follow_up = FollowUpMenuCallPacket {
            menu_id: 77,
            title: Some("title".into()),
            message: None,
            options: vec![
                vec![Some("A".into()), Some("B".into())],
                vec![None, Some("C".into())],
            ],
        };
        let mut bytes = Vec::new();
        follow_up.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&77i32.to_be_bytes());
        crate::mindustry::io::write_string(&mut expected, Some("title")).unwrap();
        crate::mindustry::io::write_string(&mut expected, None).unwrap();
        crate::mindustry::io::type_io::write_string_array(
            &mut expected,
            &[vec![Some("A"), Some("B")], vec![None, Some("C")]],
        )
        .unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(
            FollowUpMenuCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            follow_up
        );

        let game_over = GameOverCallPacket { winner: TeamId(6) };
        let mut bytes = Vec::new();
        game_over.write_to(&mut bytes).unwrap();
        assert_eq!(bytes, vec![6]);
        assert_eq!(
            GameOverCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            game_over
        );

        let hidden_snapshot = HiddenSnapshotCallPacket {
            ids: vec![3, -4, 5],
        };
        let mut bytes = Vec::new();
        hidden_snapshot.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&3i32.to_be_bytes());
        expected.extend_from_slice(&3i32.to_be_bytes());
        expected.extend_from_slice(&(-4i32).to_be_bytes());
        expected.extend_from_slice(&5i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            HiddenSnapshotCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            hidden_snapshot
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
    fn generated_deconstruct_finish_packet_uses_java_typeio_field_order() {
        let loader = ContentLoader::create_base_content().unwrap();
        let router_id = loader
            .get_by_name(crate::mindustry::ctype::ContentType::Block, "router")
            .unwrap()
            .id
            .to_be_bytes();
        let tile = crate::mindustry::world::point2_pack(7, 8);
        let packet = DeconstructFinishCallPacket {
            tile: Some(tile),
            block: Some("router".into()),
            builder: UnitRef::Unit { id: 1234 },
        };

        let mut bytes = Vec::new();
        packet.write_to_with_loader(&mut bytes, &loader).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&tile.to_be_bytes());
        expected.extend_from_slice(&router_id);
        expected.push(2);
        expected.extend_from_slice(&1234i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            DeconstructFinishCallPacket::read_from_with_loader(&mut bytes.as_slice(), &loader)
                .unwrap(),
            packet
        );

        let null_packet = DeconstructFinishCallPacket {
            tile: None,
            block: None,
            builder: UnitRef::Null,
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
        assert_eq!(bytes, expected);
        assert_eq!(
            DeconstructFinishCallPacket::read_from_with_loader(&mut bytes.as_slice(), &loader)
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

        let spawn = PlayerSpawnCallPacket {
            tile: Some(crate::mindustry::world::point2_pack(8, 9)),
            player: EntityRef::new(10),
        };
        let mut bytes = Vec::new();
        spawn.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&crate::mindustry::world::point2_pack(8, 9).to_be_bytes());
        expected.extend_from_slice(&10i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            PlayerSpawnCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            spawn
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

        let remove_tile = RemoveTileCallPacket {
            tile: Some(crate::mindustry::world::point2_pack(2, 3)),
        };
        let mut bytes = Vec::new();
        remove_tile.write_to(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            crate::mindustry::world::point2_pack(2, 3).to_be_bytes()
        );
        assert_eq!(
            RemoveTileCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            remove_tile
        );

        let remove_label = RemoveWorldLabelCallPacket { id: 1234 };
        let mut bytes = Vec::new();
        remove_label.write_to(&mut bytes).unwrap();
        assert_eq!(bytes, 1234i32.to_be_bytes());
        assert_eq!(
            RemoveWorldLabelCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            remove_label
        );

        let block_snapshot = RequestBlockSnapshotCallPacket {
            pos: crate::mindustry::world::point2_pack(4, 5),
        };
        let mut bytes = Vec::new();
        block_snapshot.write_to(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            crate::mindustry::world::point2_pack(4, 5).to_be_bytes()
        );
        assert_eq!(
            RequestBlockSnapshotCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            block_snapshot
        );

        let request_build = RequestBuildPayloadCallPacket {
            player: EntityRef::new(11),
            build: BuildingRef::new(crate::mindustry::world::point2_pack(6, 7)),
        };
        let mut bytes = Vec::new();
        request_build.write_client_payload(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            crate::mindustry::world::point2_pack(6, 7).to_be_bytes()
        );
        assert_eq!(
            RequestBuildPayloadCallPacket::read_from_client_payload(&mut bytes.as_slice()).unwrap(),
            RequestBuildPayloadCallPacket {
                player: EntityRef::null(),
                build: request_build.build
            }
        );
        bytes.clear();
        request_build.write_server_payload(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            [
                11i32.to_be_bytes(),
                crate::mindustry::world::point2_pack(6, 7).to_be_bytes()
            ]
            .concat()
        );
        assert_eq!(
            RequestBuildPayloadCallPacket::read_from_server_payload(&mut bytes.as_slice()).unwrap(),
            request_build
        );

        let request_debug = RequestDebugStatusCallPacket;
        let mut bytes = Vec::new();
        request_debug.write_to(&mut bytes).unwrap();
        assert!(bytes.is_empty());
        assert_eq!(
            RequestDebugStatusCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            request_debug
        );

        let request_drop = RequestDropPayloadCallPacket {
            player: EntityRef::new(12),
            x: 13.5,
            y: -14.25,
        };
        let mut bytes = Vec::new();
        request_drop.write_client_payload(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&13.5f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&(-14.25f32).to_bits().to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            RequestDropPayloadCallPacket::read_from_client_payload(&mut bytes.as_slice()).unwrap(),
            RequestDropPayloadCallPacket {
                player: EntityRef::null(),
                x: 13.5,
                y: -14.25
            }
        );
        bytes.clear();
        request_drop.write_server_payload(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&12i32.to_be_bytes());
        expected.extend_from_slice(&13.5f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&(-14.25f32).to_bits().to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            RequestDropPayloadCallPacket::read_from_server_payload(&mut bytes.as_slice()).unwrap(),
            request_drop
        );

        let request_unit = RequestUnitPayloadCallPacket {
            player: EntityRef::new(13),
            target: UnitRef::Unit { id: 14 },
        };
        let mut bytes = Vec::new();
        request_unit.write_client_payload(&mut bytes).unwrap();
        assert_eq!(bytes, [vec![2], 14i32.to_be_bytes().to_vec()].concat());
        assert_eq!(
            RequestUnitPayloadCallPacket::read_from_client_payload(&mut bytes.as_slice()).unwrap(),
            RequestUnitPayloadCallPacket {
                player: EntityRef::null(),
                target: UnitRef::Unit { id: 14 }
            }
        );
        bytes.clear();
        request_unit.write_server_payload(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            [
                13i32.to_be_bytes().to_vec(),
                vec![2],
                14i32.to_be_bytes().to_vec()
            ]
            .concat()
        );
        assert_eq!(
            RequestUnitPayloadCallPacket::read_from_server_payload(&mut bytes.as_slice()).unwrap(),
            request_unit
        );

        let rotate = RotateBlockCallPacket {
            player: EntityRef::new(15),
            build: BuildingRef::new(crate::mindustry::world::point2_pack(8, 9)),
            direction: true,
        };
        let mut bytes = Vec::new();
        rotate.write_client_payload(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            [
                crate::mindustry::world::point2_pack(8, 9)
                    .to_be_bytes()
                    .to_vec(),
                vec![1]
            ]
            .concat()
        );
        assert_eq!(
            RotateBlockCallPacket::read_from_client_payload(&mut bytes.as_slice()).unwrap(),
            RotateBlockCallPacket {
                player: EntityRef::null(),
                build: rotate.build,
                direction: true
            }
        );
        bytes.clear();
        rotate.write_server_payload(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            [
                15i32.to_be_bytes().to_vec(),
                crate::mindustry::world::point2_pack(8, 9)
                    .to_be_bytes()
                    .to_vec(),
                vec![1]
            ]
            .concat()
        );
        assert_eq!(
            RotateBlockCallPacket::read_from_server_payload(&mut bytes.as_slice()).unwrap(),
            rotate
        );

        let loader = ContentLoader::create_base_content().unwrap();
        let request_item = RequestItemCallPacket {
            player: EntityRef::new(16),
            build: BuildingRef::new(crate::mindustry::world::point2_pack(10, 11)),
            item: Some("copper".into()),
            amount: 99,
        };
        let mut bytes = Vec::new();
        request_item
            .write_client_payload_with_loader(&mut bytes, &loader)
            .unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&crate::mindustry::world::point2_pack(10, 11).to_be_bytes());
        expected.extend_from_slice(&[0, 0]);
        expected.extend_from_slice(&99i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            RequestItemCallPacket::read_from_client_payload_with_loader(
                &mut bytes.as_slice(),
                &loader
            )
            .unwrap(),
            RequestItemCallPacket {
                player: EntityRef::null(),
                build: request_item.build,
                item: Some("copper".into()),
                amount: 99
            }
        );
        bytes.clear();
        request_item
            .write_server_payload_with_loader(&mut bytes, &loader)
            .unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&16i32.to_be_bytes());
        expected.extend_from_slice(&crate::mindustry::world::point2_pack(10, 11).to_be_bytes());
        expected.extend_from_slice(&[0, 0]);
        expected.extend_from_slice(&99i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            RequestItemCallPacket::read_from_server_payload_with_loader(
                &mut bytes.as_slice(),
                &loader
            )
            .unwrap(),
            request_item
        );

        let researched = ResearchedCallPacket {
            content_type: ContentType::Item,
            content: "copper".into(),
        };
        let mut bytes = Vec::new();
        researched
            .write_to_with_loader(&mut bytes, &loader)
            .unwrap();
        assert_eq!(bytes, vec![ContentType::Item.ordinal(), 0, 0]);
        assert_eq!(
            ResearchedCallPacket::read_from_with_loader(&mut bytes.as_slice(), &loader).unwrap(),
            researched
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

        let landing = LandingPadLandedCallPacket {
            tile: Some(crate::mindustry::world::point2_pack(11, 12)),
        };
        let mut bytes = Vec::new();
        landing.write_to(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            crate::mindustry::world::point2_pack(11, 12).to_be_bytes()
        );
        assert_eq!(
            LandingPadLandedCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            landing
        );

        let logic = LogicExplosionCallPacket {
            team: TeamId(3),
            x: 1.25,
            y: -2.5,
            radius: 12.0,
            damage: 45.5,
            air: true,
            ground: false,
            pierce: true,
            effect: false,
        };
        let mut bytes = Vec::new();
        logic.write_to(&mut bytes).unwrap();
        let mut expected = vec![3];
        for value in [1.25f32, -2.5, 12.0, 45.5] {
            expected.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        expected.extend_from_slice(&[1, 0, 1, 0]);
        assert_eq!(bytes, expected);
        assert_eq!(
            LogicExplosionCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            logic
        );

        let menu = MenuCallPacket {
            menu_id: 77,
            title: "title".into(),
            message: "message".into(),
            options: vec![vec![Some("A".into()), None], vec![Some("B".into())]],
        };
        let mut bytes = Vec::new();
        menu.write_to(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&77i32.to_be_bytes());
        crate::mindustry::io::write_string(&mut expected, Some("title")).unwrap();
        crate::mindustry::io::write_string(&mut expected, Some("message")).unwrap();
        crate::mindustry::io::type_io::write_string_array(
            &mut expected,
            &[vec![Some("A"), None], vec![Some("B")]],
        )
        .unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(
            MenuCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            menu
        );

        let menu_choose = MenuChooseCallPacket {
            player_id: Some(1234),
            menu_id: 5,
            option: 2,
        };
        let mut bytes = Vec::new();
        menu_choose.write_client_payload(&mut bytes).unwrap();
        assert_eq!(bytes, [5i32.to_be_bytes(), 2i32.to_be_bytes()].concat());
        assert_eq!(
            MenuChooseCallPacket::read_from_client_payload(&mut bytes.as_slice()).unwrap(),
            MenuChooseCallPacket {
                player_id: None,
                menu_id: 5,
                option: 2
            }
        );
        bytes.clear();
        menu_choose.write_server_payload(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            [
                1234i32.to_be_bytes(),
                5i32.to_be_bytes(),
                2i32.to_be_bytes()
            ]
            .concat()
        );
        assert_eq!(
            MenuChooseCallPacket::read_from_server_payload(&mut bytes.as_slice()).unwrap(),
            menu_choose
        );

        let payload_drop = PayloadDroppedCallPacket {
            unit: UnitRef::Unit { id: 88 },
            x: 4.0,
            y: -6.5,
        };
        let mut bytes = Vec::new();
        payload_drop.write_to(&mut bytes).unwrap();
        let mut expected = vec![2];
        expected.extend_from_slice(&88i32.to_be_bytes());
        expected.extend_from_slice(&4.0f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&(-6.5f32).to_bits().to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            PayloadDroppedCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            payload_drop
        );

        let picked_build = PickedBuildPayloadCallPacket {
            unit: UnitRef::Unit { id: 89 },
            build_pos: Some(crate::mindustry::world::point2_pack(3, 4)),
            on_ground: true,
        };
        let mut bytes = Vec::new();
        picked_build.write_to(&mut bytes).unwrap();
        let mut expected = vec![2];
        expected.extend_from_slice(&89i32.to_be_bytes());
        expected.extend_from_slice(&crate::mindustry::world::point2_pack(3, 4).to_be_bytes());
        expected.push(1);
        assert_eq!(bytes, expected);
        assert_eq!(
            PickedBuildPayloadCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            picked_build
        );

        let picked_unit = PickedUnitPayloadCallPacket {
            unit: UnitRef::Unit { id: 90 },
            target: UnitRef::Unit { id: 91 },
        };
        let mut bytes = Vec::new();
        picked_unit.write_to(&mut bytes).unwrap();
        let mut expected = vec![2];
        expected.extend_from_slice(&90i32.to_be_bytes());
        expected.push(2);
        expected.extend_from_slice(&91i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            PickedUnitPayloadCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            picked_unit
        );

        let ping = PingCallPacket { time: 987654321 };
        let mut bytes = Vec::new();
        ping.write_to(&mut bytes).unwrap();
        assert_eq!(bytes, 987654321i64.to_be_bytes());
        assert_eq!(
            PingCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            ping
        );

        let ping_location = PingLocationCallPacket {
            player_id: Some(7),
            x: 10.0,
            y: 20.5,
            text: "look".into(),
        };
        let mut bytes = Vec::new();
        ping_location.write_client_payload(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&10.0f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&20.5f32.to_bits().to_be_bytes());
        crate::mindustry::io::write_string(&mut expected, Some("look")).unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(
            PingLocationCallPacket::read_from_client_payload(&mut bytes.as_slice()).unwrap(),
            PingLocationCallPacket {
                player_id: None,
                x: 10.0,
                y: 20.5,
                text: "look".into()
            }
        );
        bytes.clear();
        ping_location.write_server_payload(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&7i32.to_be_bytes());
        expected.extend_from_slice(&10.0f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&20.5f32.to_bits().to_be_bytes());
        crate::mindustry::io::write_string(&mut expected, Some("look")).unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(
            PingLocationCallPacket::read_from_server_payload(&mut bytes.as_slice()).unwrap(),
            ping_location
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
                    packet_ids::CREATE_MARKER_CALL_PACKET,
                    "CreateMarkerCallPacket",
                ),
                (
                    packet_ids::CREATE_WEATHER_CALL_PACKET,
                    "CreateWeatherCallPacket",
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
                    packet_ids::DECONSTRUCT_FINISH_CALL_PACKET,
                    "DeconstructFinishCallPacket",
                ),
                (
                    packet_ids::DELETE_PLANS_CALL_PACKET,
                    "DeletePlansCallPacket"
                ),
                (
                    packet_ids::DESTROY_PAYLOAD_CALL_PACKET,
                    "DestroyPayloadCallPacket",
                ),
                (packet_ids::DROP_ITEM_CALL_PACKET, "DropItemCallPacket"),
                (packet_ids::EFFECT_CALL_PACKET, "EffectCallPacket"),
                (packet_ids::EFFECT_CALL_PACKET2, "EffectCallPacket2"),
                (
                    packet_ids::EFFECT_RELIABLE_CALL_PACKET,
                    "EffectReliableCallPacket",
                ),
                (
                    packet_ids::ENTITY_SNAPSHOT_CALL_PACKET,
                    "EntitySnapshotCallPacket",
                ),
                (
                    packet_ids::FOLLOW_UP_MENU_CALL_PACKET,
                    "FollowUpMenuCallPacket",
                ),
                (packet_ids::GAME_OVER_CALL_PACKET, "GameOverCallPacket"),
                (
                    packet_ids::HIDDEN_SNAPSHOT_CALL_PACKET,
                    "HiddenSnapshotCallPacket",
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
                (
                    packet_ids::LANDING_PAD_LANDED_CALL_PACKET,
                    "LandingPadLandedCallPacket",
                ),
                (
                    packet_ids::LOGIC_EXPLOSION_CALL_PACKET,
                    "LogicExplosionCallPacket",
                ),
                (packet_ids::MENU_CALL_PACKET, "MenuCallPacket"),
                (packet_ids::MENU_CHOOSE_CALL_PACKET, "MenuChooseCallPacket"),
                (packet_ids::OPEN_URI_CALL_PACKET, "OpenURICallPacket"),
                (
                    packet_ids::PAYLOAD_DROPPED_CALL_PACKET,
                    "PayloadDroppedCallPacket",
                ),
                (
                    packet_ids::PICKED_BUILD_PAYLOAD_CALL_PACKET,
                    "PickedBuildPayloadCallPacket",
                ),
                (
                    packet_ids::PICKED_UNIT_PAYLOAD_CALL_PACKET,
                    "PickedUnitPayloadCallPacket",
                ),
                (packet_ids::PING_CALL_PACKET, "PingCallPacket"),
                (
                    packet_ids::PING_LOCATION_CALL_PACKET,
                    "PingLocationCallPacket",
                ),
                (
                    packet_ids::PING_RESPONSE_CALL_PACKET,
                    "PingResponseCallPacket",
                ),
                (
                    packet_ids::PLAYER_DISCONNECT_CALL_PACKET,
                    "PlayerDisconnectCallPacket",
                ),
                (
                    packet_ids::PLAYER_SPAWN_CALL_PACKET,
                    "PlayerSpawnCallPacket"
                ),
                (
                    packet_ids::REMOVE_MARKER_CALL_PACKET,
                    "RemoveMarkerCallPacket",
                ),
                (
                    packet_ids::REMOVE_QUEUE_BLOCK_CALL_PACKET,
                    "RemoveQueueBlockCallPacket",
                ),
                (packet_ids::REMOVE_TILE_CALL_PACKET, "RemoveTileCallPacket"),
                (
                    packet_ids::REMOVE_WORLD_LABEL_CALL_PACKET,
                    "RemoveWorldLabelCallPacket",
                ),
                (
                    packet_ids::REQUEST_BLOCK_SNAPSHOT_CALL_PACKET,
                    "RequestBlockSnapshotCallPacket",
                ),
                (
                    packet_ids::REQUEST_BUILD_PAYLOAD_CALL_PACKET,
                    "RequestBuildPayloadCallPacket",
                ),
                (
                    packet_ids::REQUEST_DEBUG_STATUS_CALL_PACKET,
                    "RequestDebugStatusCallPacket",
                ),
                (
                    packet_ids::REQUEST_DROP_PAYLOAD_CALL_PACKET,
                    "RequestDropPayloadCallPacket",
                ),
                (
                    packet_ids::REQUEST_ITEM_CALL_PACKET,
                    "RequestItemCallPacket"
                ),
                (
                    packet_ids::REQUEST_UNIT_PAYLOAD_CALL_PACKET,
                    "RequestUnitPayloadCallPacket",
                ),
                (packet_ids::RESEARCHED_CALL_PACKET, "ResearchedCallPacket"),
                (
                    packet_ids::ROTATE_BLOCK_CALL_PACKET,
                    "RotateBlockCallPacket"
                ),
                (
                    packet_ids::SECTOR_CAPTURE_CALL_PACKET,
                    "SectorCaptureCallPacket"
                ),
                (
                    packet_ids::SEND_CHAT_MESSAGE_CALL_PACKET,
                    "SendChatMessageCallPacket",
                ),
                (
                    packet_ids::SEND_MESSAGE_CALL_PACKET,
                    "SendMessageCallPacket"
                ),
                (
                    packet_ids::SEND_MESSAGE_CALL_PACKET2,
                    "SendMessageCallPacket2",
                ),
                (
                    packet_ids::SERVER_BINARY_PACKET_RELIABLE_CALL_PACKET,
                    "ServerBinaryPacketReliableCallPacket",
                ),
                (
                    packet_ids::SERVER_BINARY_PACKET_UNRELIABLE_CALL_PACKET,
                    "ServerBinaryPacketUnreliableCallPacket",
                ),
                (
                    packet_ids::SERVER_PACKET_RELIABLE_CALL_PACKET,
                    "ServerPacketReliableCallPacket",
                ),
                (
                    packet_ids::SERVER_PACKET_UNRELIABLE_CALL_PACKET,
                    "ServerPacketUnreliableCallPacket",
                ),
                (
                    packet_ids::SET_CAMERA_POSITION_CALL_PACKET,
                    "SetCameraPositionCallPacket",
                ),
                (packet_ids::SET_FLAG_CALL_PACKET, "SetFlagCallPacket"),
                (packet_ids::SET_FLOOR_CALL_PACKET, "SetFloorCallPacket"),
                (packet_ids::SET_HUD_TEXT_CALL_PACKET, "SetHudTextCallPacket",),
                (
                    packet_ids::SET_HUD_TEXT_RELIABLE_CALL_PACKET,
                    "SetHudTextReliableCallPacket",
                ),
                (packet_ids::SET_ITEM_CALL_PACKET, "SetItemCallPacket"),
                (packet_ids::SET_ITEMS_CALL_PACKET, "SetItemsCallPacket"),
                (packet_ids::SET_LIQUID_CALL_PACKET, "SetLiquidCallPacket"),
                (packet_ids::SET_LIQUIDS_CALL_PACKET, "SetLiquidsCallPacket"),
                (packet_ids::SET_MAP_AREA_CALL_PACKET, "SetMapAreaCallPacket",),
                (
                    packet_ids::SET_OBJECTIVES_CALL_PACKET,
                    "SetObjectivesCallPacket"
                ),
                (packet_ids::SET_OVERLAY_CALL_PACKET, "SetOverlayCallPacket"),
                (
                    packet_ids::SET_PLAYER_TEAM_EDITOR_CALL_PACKET,
                    "SetPlayerTeamEditorCallPacket",
                ),
                (
                    packet_ids::SET_POSITION_CALL_PACKET,
                    "SetPositionCallPacket"
                ),
                (packet_ids::SET_RULE_CALL_PACKET, "SetRuleCallPacket"),
                (packet_ids::SET_RULES_CALL_PACKET, "SetRulesCallPacket"),
                (packet_ids::SET_TEAM_CALL_PACKET, "SetTeamCallPacket"),
                (packet_ids::SET_TEAMS_CALL_PACKET, "SetTeamsCallPacket"),
                (packet_ids::SET_TILE_CALL_PACKET, "SetTileCallPacket"),
                (
                    packet_ids::SET_TILE_BLOCKS_CALL_PACKET,
                    "SetTileBlocksCallPacket"
                ),
                (
                    packet_ids::SET_TILE_FLOORS_CALL_PACKET,
                    "SetTileFloorsCallPacket"
                ),
                (
                    packet_ids::SET_TILE_ITEMS_CALL_PACKET,
                    "SetTileItemsCallPacket"
                ),
                (
                    packet_ids::SET_TILE_LIQUIDS_CALL_PACKET,
                    "SetTileLiquidsCallPacket",
                ),
                (
                    packet_ids::SET_TILE_OVERLAYS_CALL_PACKET,
                    "SetTileOverlaysCallPacket",
                ),
                (
                    packet_ids::SET_UNIT_COMMAND_CALL_PACKET,
                    "SetUnitCommandCallPacket",
                ),
                (
                    packet_ids::SET_UNIT_STANCE_CALL_PACKET,
                    "SetUnitStanceCallPacket",
                ),
                (packet_ids::SOUND_CALL_PACKET, "SoundCallPacket"),
                (packet_ids::SOUND_AT_CALL_PACKET, "SoundAtCallPacket"),
                (
                    packet_ids::SPAWN_EFFECT_CALL_PACKET,
                    "SpawnEffectCallPacket"
                ),
                (
                    packet_ids::STATE_SNAPSHOT_CALL_PACKET,
                    "StateSnapshotCallPacket",
                ),
                (
                    packet_ids::SYNC_VARIABLE_CALL_PACKET,
                    "SyncVariableCallPacket",
                ),
                (packet_ids::TAKE_ITEMS_CALL_PACKET, "TakeItemsCallPacket"),
                (packet_ids::TEXT_INPUT_CALL_PACKET, "TextInputCallPacket"),
                (packet_ids::TEXT_INPUT_CALL_PACKET2, "TextInputCallPacket2"),
                (
                    packet_ids::TEXT_INPUT_RESULT_CALL_PACKET,
                    "TextInputResultCallPacket"
                ),
                (packet_ids::TILE_CONFIG_CALL_PACKET, "TileConfigCallPacket"),
                (packet_ids::TILE_TAP_CALL_PACKET, "TileTapCallPacket"),
                (packet_ids::TRACE_INFO_CALL_PACKET, "TraceInfoCallPacket"),
                (
                    packet_ids::TRANSFER_INVENTORY_CALL_PACKET,
                    "TransferInventoryCallPacket",
                ),
                (
                    packet_ids::TRANSFER_ITEM_EFFECT_CALL_PACKET,
                    "TransferItemEffectCallPacket",
                ),
                (
                    packet_ids::TRANSFER_ITEM_TO_CALL_PACKET,
                    "TransferItemToCallPacket",
                ),
                (
                    packet_ids::TRANSFER_ITEM_TO_UNIT_CALL_PACKET,
                    "TransferItemToUnitCallPacket",
                ),
                (
                    packet_ids::UNIT_BLOCK_SPAWN_CALL_PACKET,
                    "UnitBlockSpawnCallPacket",
                ),
                (
                    packet_ids::UNIT_BUILDING_CONTROL_SELECT_CALL_PACKET,
                    "UnitBuildingControlSelectCallPacket",
                ),
                (
                    packet_ids::UNIT_CAP_DEATH_CALL_PACKET,
                    "UnitCapDeathCallPacket"
                ),
                (packet_ids::UNIT_CLEAR_CALL_PACKET, "UnitClearCallPacket"),
                (
                    packet_ids::UNIT_CONTROL_CALL_PACKET,
                    "UnitControlCallPacket"
                ),
                (packet_ids::UNIT_DEATH_CALL_PACKET, "UnitDeathCallPacket"),
                (
                    packet_ids::UNIT_DESPAWN_CALL_PACKET,
                    "UnitDespawnCallPacket",
                ),
                (
                    packet_ids::UNIT_DESTROY_CALL_PACKET,
                    "UnitDestroyCallPacket",
                ),
                (
                    packet_ids::UNIT_ENTERED_PAYLOAD_CALL_PACKET,
                    "UnitEnteredPayloadCallPacket",
                ),
                (
                    packet_ids::UNIT_ENV_DEATH_CALL_PACKET,
                    "UnitEnvDeathCallPacket",
                ),
                (
                    packet_ids::UNIT_SAFE_DEATH_CALL_PACKET,
                    "UnitSafeDeathCallPacket",
                ),
                (packet_ids::UNIT_SPAWN_CALL_PACKET, "UnitSpawnCallPacket"),
                (
                    packet_ids::UNIT_TETHER_BLOCK_SPAWNED_CALL_PACKET,
                    "UnitTetherBlockSpawnedCallPacket",
                ),
                (
                    packet_ids::UPDATE_GAME_OVER_CALL_PACKET,
                    "UpdateGameOverCallPacket",
                ),
                (
                    packet_ids::UPDATE_MARKER_CALL_PACKET,
                    "UpdateMarkerCallPacket",
                ),
                (
                    packet_ids::UPDATE_MARKER_TEXT_CALL_PACKET,
                    "UpdateMarkerTextCallPacket",
                ),
                (
                    packet_ids::UPDATE_MARKER_TEXTURE_CALL_PACKET,
                    "UpdateMarkerTextureCallPacket",
                ),
                (
                    packet_ids::WARNING_TOAST_CALL_PACKET,
                    "WarningToastCallPacket",
                ),
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

        let world_data = find_packet_by_name("WorldDataBeginCallPacket").unwrap();
        assert_eq!(
            world_data.id,
            Some(packet_ids::WORLD_DATA_BEGIN_CALL_PACKET)
        );
        assert_eq!(world_data.direction, PacketDirection::ServerToClient);
        assert_eq!(world_data.priority, Some(PacketPriority::Normal));
        assert!(world_data.allow_client_endpoint);
        assert!(!world_data.allow_server_endpoint);

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

        let deconstruct = find_packet_by_name("DeconstructFinishCallPacket").unwrap();
        assert_eq!(
            deconstruct.id,
            Some(packet_ids::DECONSTRUCT_FINISH_CALL_PACKET)
        );
        assert_eq!(deconstruct.direction, PacketDirection::ServerToClient);
        assert_eq!(deconstruct.priority, Some(PacketPriority::Normal));
        assert!(deconstruct.allow_client_endpoint);
        assert!(!deconstruct.allow_server_endpoint);

        let delete_plans = find_packet_by_name("DeletePlansCallPacket").unwrap();
        assert_eq!(delete_plans.id, Some(packet_ids::DELETE_PLANS_CALL_PACKET));
        assert_eq!(delete_plans.direction, PacketDirection::Bidirectional);
        assert_eq!(delete_plans.priority, Some(PacketPriority::Normal));
        assert!(delete_plans.allow_client_endpoint);
        assert!(delete_plans.allow_server_endpoint);

        let destroy_payload = find_packet_by_name("DestroyPayloadCallPacket").unwrap();
        assert_eq!(
            destroy_payload.id,
            Some(packet_ids::DESTROY_PAYLOAD_CALL_PACKET)
        );
        assert_eq!(destroy_payload.direction, PacketDirection::ServerToClient);
        assert_eq!(destroy_payload.priority, Some(PacketPriority::Normal));
        assert!(destroy_payload.allow_client_endpoint);
        assert!(!destroy_payload.allow_server_endpoint);

        let landing = find_packet_by_name("LandingPadLandedCallPacket").unwrap();
        assert_eq!(landing.id, Some(packet_ids::LANDING_PAD_LANDED_CALL_PACKET));
        assert_eq!(landing.direction, PacketDirection::ServerToClient);
        assert_eq!(landing.priority, Some(PacketPriority::Normal));
        assert!(landing.allow_client_endpoint);
        assert!(!landing.allow_server_endpoint);

        let logic = find_packet_by_name("LogicExplosionCallPacket").unwrap();
        assert_eq!(logic.id, Some(packet_ids::LOGIC_EXPLOSION_CALL_PACKET));
        assert_eq!(logic.direction, PacketDirection::ServerToClient);
        assert_eq!(logic.priority, Some(PacketPriority::Normal));
        assert!(logic.allow_client_endpoint);
        assert!(!logic.allow_server_endpoint);

        let menu = find_packet_by_name("MenuCallPacket").unwrap();
        assert_eq!(menu.id, Some(packet_ids::MENU_CALL_PACKET));
        assert_eq!(menu.direction, PacketDirection::ServerToClient);
        assert_eq!(menu.priority, Some(PacketPriority::Normal));
        assert!(menu.allow_client_endpoint);
        assert!(!menu.allow_server_endpoint);

        let menu_choose = find_packet_by_name("MenuChooseCallPacket").unwrap();
        assert_eq!(menu_choose.id, Some(packet_ids::MENU_CHOOSE_CALL_PACKET));
        assert_eq!(menu_choose.direction, PacketDirection::Bidirectional);
        assert_eq!(menu_choose.priority, Some(PacketPriority::Normal));
        assert!(menu_choose.allow_client_endpoint);
        assert!(menu_choose.allow_server_endpoint);

        let uri = find_packet_by_name("OpenURICallPacket").unwrap();
        assert_eq!(uri.id, Some(packet_ids::OPEN_URI_CALL_PACKET));
        assert_eq!(uri.direction, PacketDirection::ServerToClient);
        assert_eq!(uri.priority, Some(PacketPriority::Normal));
        assert!(uri.allow_client_endpoint);
        assert!(!uri.allow_server_endpoint);

        let ping = find_packet_by_name("PingCallPacket").unwrap();
        assert_eq!(ping.id, Some(packet_ids::PING_CALL_PACKET));
        assert_eq!(ping.direction, PacketDirection::ClientToServer);
        assert_eq!(ping.priority, Some(PacketPriority::High));
        assert!(!ping.allow_client_endpoint);
        assert!(ping.allow_server_endpoint);

        let spawn = find_packet_by_name("PlayerSpawnCallPacket").unwrap();
        assert_eq!(spawn.id, Some(packet_ids::PLAYER_SPAWN_CALL_PACKET));
        assert_eq!(spawn.direction, PacketDirection::ServerToClient);
        assert!(spawn.allow_client_endpoint);
        assert!(!spawn.allow_server_endpoint);

        let request_snapshot = find_packet_by_name("RequestBlockSnapshotCallPacket").unwrap();
        assert_eq!(
            request_snapshot.id,
            Some(packet_ids::REQUEST_BLOCK_SNAPSHOT_CALL_PACKET)
        );
        assert_eq!(request_snapshot.direction, PacketDirection::ClientToServer);
        assert_eq!(request_snapshot.priority, Some(PacketPriority::Low));
        assert!(!request_snapshot.allow_client_endpoint);
        assert!(request_snapshot.allow_server_endpoint);

        let request_item = find_packet_by_name("RequestItemCallPacket").unwrap();
        assert_eq!(request_item.id, Some(packet_ids::REQUEST_ITEM_CALL_PACKET));
        assert_eq!(request_item.direction, PacketDirection::Bidirectional);
        assert!(request_item.allow_client_endpoint);
        assert!(request_item.allow_server_endpoint);

        let debug_status = find_packet_by_name("RequestDebugStatusCallPacket").unwrap();
        assert_eq!(
            debug_status.id,
            Some(packet_ids::REQUEST_DEBUG_STATUS_CALL_PACKET)
        );
        assert_eq!(debug_status.direction, PacketDirection::ClientToServer);
        assert!(!debug_status.allow_client_endpoint);
        assert!(debug_status.allow_server_endpoint);

        let researched = find_packet_by_name("ResearchedCallPacket").unwrap();
        assert_eq!(researched.id, Some(packet_ids::RESEARCHED_CALL_PACKET));
        assert_eq!(researched.direction, PacketDirection::ServerToClient);
        assert!(researched.allow_client_endpoint);
        assert!(!researched.allow_server_endpoint);

        let bullet = find_packet_by_name("CreateBulletCallPacket").unwrap();
        assert_eq!(bullet.id, Some(packet_ids::CREATE_BULLET_CALL_PACKET));
        assert_eq!(bullet.direction, PacketDirection::ServerToClient);
        assert_eq!(bullet.priority, Some(PacketPriority::Normal));
        assert!(bullet.allow_client_endpoint);
        assert!(!bullet.allow_server_endpoint);

        let marker = find_packet_by_name("CreateMarkerCallPacket").unwrap();
        assert_eq!(marker.id, Some(packet_ids::CREATE_MARKER_CALL_PACKET));
        assert_eq!(marker.direction, PacketDirection::ServerToClient);
        assert_eq!(marker.priority, Some(PacketPriority::Normal));
        assert!(marker.allow_client_endpoint);
        assert!(!marker.allow_server_endpoint);

        let weather = find_packet_by_name("CreateWeatherCallPacket").unwrap();
        assert_eq!(weather.id, Some(packet_ids::CREATE_WEATHER_CALL_PACKET));
        assert_eq!(weather.direction, PacketDirection::ServerToClient);
        assert_eq!(weather.priority, Some(PacketPriority::Normal));
        assert!(weather.allow_client_endpoint);
        assert!(!weather.allow_server_endpoint);

        let effect = find_packet_by_name("EffectCallPacket").unwrap();
        assert_eq!(effect.id, Some(packet_ids::EFFECT_CALL_PACKET));
        assert_eq!(effect.direction, PacketDirection::ServerToClient);
        assert_eq!(effect.priority, Some(PacketPriority::Normal));
        assert!(effect.allow_client_endpoint);
        assert!(!effect.allow_server_endpoint);

        let effect_with_data = find_packet_by_name("EffectCallPacket2").unwrap();
        assert_eq!(effect_with_data.id, Some(packet_ids::EFFECT_CALL_PACKET2));
        assert_eq!(effect_with_data.direction, PacketDirection::ServerToClient);
        assert_eq!(effect_with_data.priority, Some(PacketPriority::Normal));
        assert!(effect_with_data.allow_client_endpoint);
        assert!(!effect_with_data.allow_server_endpoint);

        let reliable_effect = find_packet_by_name("EffectReliableCallPacket").unwrap();
        assert_eq!(
            reliable_effect.id,
            Some(packet_ids::EFFECT_RELIABLE_CALL_PACKET)
        );
        assert_eq!(reliable_effect.direction, PacketDirection::ServerToClient);
        assert_eq!(reliable_effect.priority, Some(PacketPriority::Normal));
        assert!(reliable_effect.allow_client_endpoint);
        assert!(!reliable_effect.allow_server_endpoint);

        let drop_item = find_packet_by_name("DropItemCallPacket").unwrap();
        assert_eq!(drop_item.id, Some(packet_ids::DROP_ITEM_CALL_PACKET));
        assert_eq!(drop_item.direction, PacketDirection::ClientToServer);
        assert_eq!(drop_item.priority, Some(PacketPriority::Normal));
        assert!(!drop_item.allow_client_endpoint);
        assert!(drop_item.allow_server_endpoint);

        let entity_snapshot = find_packet_by_name("EntitySnapshotCallPacket").unwrap();
        assert_eq!(
            entity_snapshot.id,
            Some(packet_ids::ENTITY_SNAPSHOT_CALL_PACKET)
        );
        assert_eq!(entity_snapshot.direction, PacketDirection::ServerToClient);
        assert_eq!(entity_snapshot.priority, Some(PacketPriority::Low));
        assert!(entity_snapshot.allow_client_endpoint);
        assert!(!entity_snapshot.allow_server_endpoint);

        let follow_up = find_packet_by_name("FollowUpMenuCallPacket").unwrap();
        assert_eq!(follow_up.id, Some(packet_ids::FOLLOW_UP_MENU_CALL_PACKET));
        assert_eq!(follow_up.direction, PacketDirection::ServerToClient);
        assert_eq!(follow_up.priority, Some(PacketPriority::Normal));
        assert!(follow_up.allow_client_endpoint);
        assert!(!follow_up.allow_server_endpoint);

        let game_over = find_packet_by_name("GameOverCallPacket").unwrap();
        assert_eq!(game_over.id, Some(packet_ids::GAME_OVER_CALL_PACKET));
        assert_eq!(game_over.direction, PacketDirection::ServerToClient);
        assert_eq!(game_over.priority, Some(PacketPriority::Normal));
        assert!(game_over.allow_client_endpoint);
        assert!(!game_over.allow_server_endpoint);

        let hidden_snapshot = find_packet_by_name("HiddenSnapshotCallPacket").unwrap();
        assert_eq!(
            hidden_snapshot.id,
            Some(packet_ids::HIDDEN_SNAPSHOT_CALL_PACKET)
        );
        assert_eq!(hidden_snapshot.direction, PacketDirection::ServerToClient);
        assert_eq!(hidden_snapshot.priority, Some(PacketPriority::Low));
        assert!(hidden_snapshot.allow_client_endpoint);
        assert!(!hidden_snapshot.allow_server_endpoint);

        let unit_spawn = find_packet_by_name("UnitSpawnCallPacket").unwrap();
        assert_eq!(unit_spawn.id, Some(packet_ids::UNIT_SPAWN_CALL_PACKET));
        assert_eq!(unit_spawn.direction, PacketDirection::ServerToClient);
        assert_eq!(unit_spawn.priority, Some(PacketPriority::Low));
        assert!(unit_spawn.allow_client_endpoint);
        assert!(!unit_spawn.allow_server_endpoint);

        let update_marker_text = find_packet_by_name("UpdateMarkerTextCallPacket").unwrap();
        assert_eq!(
            update_marker_text.id,
            Some(packet_ids::UPDATE_MARKER_TEXT_CALL_PACKET)
        );
        assert_eq!(
            update_marker_text.direction,
            PacketDirection::ServerToClient
        );
        assert_eq!(update_marker_text.priority, Some(PacketPriority::Normal));

        let warning_toast = find_packet_by_name("WarningToastCallPacket").unwrap();
        assert_eq!(
            warning_toast.id,
            Some(packet_ids::WARNING_TOAST_CALL_PACKET)
        );
        assert_eq!(warning_toast.direction, PacketDirection::ServerToClient);
        assert_eq!(warning_toast.priority, Some(PacketPriority::Normal));

        assert_eq!(
            find_registered_packet_by_id(packet_ids::UNIT_SPAWN_CALL_PACKET)
                .unwrap()
                .name,
            "UnitSpawnCallPacket"
        );
        assert_eq!(
            find_packet_by_transport_id(
                PacketTransport::NetPacket,
                packet_ids::UPDATE_MARKER_TEXTURE_CALL_PACKET,
            )
            .unwrap()
            .name,
            "UpdateMarkerTextureCallPacket"
        );
    }

    #[test]
    fn packet_manifest_keeps_local_and_framework_entries_in_upstream_order() {
        let names: Vec<_> = packet_manifest().iter().map(|entry| entry.name).collect();

        assert_eq!(
            &names[..5],
            [
                "Connect",
                "Disconnect",
                "StreamBegin",
                "StreamChunk",
                "WorldStream",
            ]
        );
        assert!(names
            .windows(2)
            .any(|window| window == ["UnitDeathCallPacket", "UnitDespawnCallPacket"]));
        assert!(names
            .windows(2)
            .any(|window| window == ["WarningToastCallPacket", "WorldDataBeginCallPacket"]));
        assert!(names
            .windows(2)
            .any(|window| window == ["ServerData", "Ping"]));
    }

    #[test]
    fn generated_v157_4_new_call_packets_roundtrip() {
        let unit_spawn = UnitSpawnCallPacket {
            container: UnitSyncContainer::new(42, 7, vec![0xaa, 0xbb, 0xcc]),
        };
        let mut bytes = Vec::new();
        unit_spawn.write_to(&mut bytes).unwrap();
        assert_eq!(bytes, vec![0, 0, 0, 42, 7, 0xaa, 0xbb, 0xcc]);
        assert_eq!(
            UnitSpawnCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            unit_spawn
        );

        let update_game_over = UpdateGameOverCallPacket { winner: TeamId(3) };
        let mut bytes = Vec::new();
        update_game_over.write_to(&mut bytes).unwrap();
        assert_eq!(
            UpdateGameOverCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            update_game_over
        );

        let update_marker = UpdateMarkerCallPacket {
            id: 7,
            control: crate::mindustry::logic::LMarkerControl::Texture,
            p1: 1.25,
            p2: -2.5,
            p3: 3.75,
        };
        let mut bytes = Vec::new();
        update_marker.write_to(&mut bytes).unwrap();
        assert_eq!(
            UpdateMarkerCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            update_marker
        );

        let update_marker_text = UpdateMarkerTextCallPacket {
            id: 11,
            r#type: crate::mindustry::logic::LMarkerControl::LabelFlags,
            fetch: true,
            text: "hello marker".into(),
        };
        let mut bytes = Vec::new();
        update_marker_text.write_to(&mut bytes).unwrap();
        assert_eq!(
            UpdateMarkerTextCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            update_marker_text
        );

        let update_marker_texture = UpdateMarkerTextureCallPacket {
            id: 13,
            texture: TypeValue::String("ui/marker".into()),
        };
        let mut bytes = Vec::new();
        update_marker_texture.write_to(&mut bytes).unwrap();
        assert_eq!(
            UpdateMarkerTextureCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            update_marker_texture
        );

        let warning_toast = WarningToastCallPacket {
            unicode: 0x26a0,
            text: "注意".into(),
        };
        let mut bytes = Vec::new();
        warning_toast.write_to(&mut bytes).unwrap();
        assert_eq!(
            WarningToastCallPacket::read_from(&mut bytes.as_slice()).unwrap(),
            warning_toast
        );
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
            "CreateMarkerCallPacket",
            "CreateWeatherCallPacket",
            "DebugStatusClientCallPacket",
            "DebugStatusClientUnreliableCallPacket",
            "DeconstructFinishCallPacket",
            "DeletePlansCallPacket",
            "DestroyPayloadCallPacket",
            "DropItemCallPacket",
            "EffectCallPacket",
            "EffectCallPacket2",
            "EffectReliableCallPacket",
            "EntitySnapshotCallPacket",
            "FollowUpMenuCallPacket",
            "GameOverCallPacket",
            "HiddenSnapshotCallPacket",
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
            "LandingPadLandedCallPacket",
            "LogicExplosionCallPacket",
            "MenuCallPacket",
            "MenuChooseCallPacket",
            "OpenURICallPacket",
            "PayloadDroppedCallPacket",
            "PickedBuildPayloadCallPacket",
            "PickedUnitPayloadCallPacket",
            "PingCallPacket",
            "PingLocationCallPacket",
            "PingResponseCallPacket",
            "PlayerDisconnectCallPacket",
            "PlayerSpawnCallPacket",
            "RemoveMarkerCallPacket",
            "RemoveQueueBlockCallPacket",
            "RemoveTileCallPacket",
            "RemoveWorldLabelCallPacket",
            "RequestBlockSnapshotCallPacket",
            "RequestBuildPayloadCallPacket",
            "RequestDebugStatusCallPacket",
            "RequestDropPayloadCallPacket",
            "RequestItemCallPacket",
            "RequestUnitPayloadCallPacket",
            "ResearchedCallPacket",
            "RotateBlockCallPacket",
            "SectorCaptureCallPacket",
            "SendChatMessageCallPacket",
            "SendMessageCallPacket",
            "SendMessageCallPacket2",
            "ServerBinaryPacketReliableCallPacket",
            "ServerBinaryPacketUnreliableCallPacket",
            "ServerPacketReliableCallPacket",
            "ServerPacketUnreliableCallPacket",
            "SetCameraPositionCallPacket",
            "SetFlagCallPacket",
            "SetFloorCallPacket",
            "SetHudTextCallPacket",
            "SetHudTextReliableCallPacket",
            "SetItemCallPacket",
            "SetItemsCallPacket",
            "SetLiquidCallPacket",
            "SetLiquidsCallPacket",
            "SetMapAreaCallPacket",
            "SetObjectivesCallPacket",
            "SetOverlayCallPacket",
            "SetPlayerTeamEditorCallPacket",
            "SetPositionCallPacket",
            "SetRuleCallPacket",
            "SetRulesCallPacket",
            "SetTeamCallPacket",
            "SetTeamsCallPacket",
            "SetTileCallPacket",
            "SetTileBlocksCallPacket",
            "SetTileFloorsCallPacket",
            "SetTileItemsCallPacket",
            "SetTileLiquidsCallPacket",
            "SetTileOverlaysCallPacket",
            "SetUnitCommandCallPacket",
            "SetUnitStanceCallPacket",
            "SoundCallPacket",
            "SoundAtCallPacket",
            "SpawnEffectCallPacket",
            "StateSnapshotCallPacket",
            "SyncVariableCallPacket",
            "TakeItemsCallPacket",
            "TextInputCallPacket",
            "TextInputCallPacket2",
            "TextInputResultCallPacket",
            "TileConfigCallPacket",
            "TileTapCallPacket",
            "TraceInfoCallPacket",
            "TransferInventoryCallPacket",
            "TransferItemEffectCallPacket",
            "TransferItemToCallPacket",
            "TransferItemToUnitCallPacket",
            "UnitBlockSpawnCallPacket",
            "UnitBuildingControlSelectCallPacket",
            "UnitCapDeathCallPacket",
            "UnitClearCallPacket",
            "UnitControlCallPacket",
            "UnitDeathCallPacket",
            "UnitDespawnCallPacket",
            "UnitDestroyCallPacket",
            "UnitEnteredPayloadCallPacket",
            "UnitEnvDeathCallPacket",
            "UnitSafeDeathCallPacket",
            "UnitSpawnCallPacket",
            "UnitTetherBlockSpawnedCallPacket",
            "UpdateGameOverCallPacket",
            "UpdateMarkerCallPacket",
            "UpdateMarkerTextCallPacket",
            "UpdateMarkerTextureCallPacket",
            "WarningToastCallPacket",
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
