use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::mindustry::ctype::ContentId;
use crate::mindustry::entities::comp::{PlayerComp, UnitComp};
use crate::mindustry::entities::units::BuildPlan;
use crate::mindustry::io::{BuildPlanWire, EntityRef, TeamId, TypeValue};
use crate::mindustry::logic::LMarkerControl;
use crate::mindustry::net::{
    read_world_data, BlockSnapshotCallPacket, BuildDestroyedCallPacket,
    BuildHealthUpdateCallPacket, BuildingControlSelectCallPacket, ClearItemsCallPacket,
    ClearLiquidsCallPacket, ClientPlanSnapshotCallPacket, ClientPlanSnapshotReceivedCallPacket,
    ClientSnapshotCallPacket, CommandBuildingCallPacket, CommandUnitsCallPacket,
    CompleteObjectiveCallPacket, Connect, ConnectCallPacket, ConnectConfirmCallPacket,
    ConnectPacket, ConstructFinishCallPacket, CreateBulletCallPacket, CreateMarkerCallPacket,
    CreateWeatherCallPacket, DebugStatusClientCallPacket, DebugStatusClientUnreliableCallPacket,
    DeconstructFinishCallPacket, DeletePlansCallPacket, Disconnect, EffectCallPacket,
    EffectCallPacket2, EffectReliableCallPacket, EntitySnapshotCallPacket,
    HiddenSnapshotCallPacket, KickCallPacket, KickCallPacket2, LandingPadLandedCallPacket,
    LogicExplosionCallPacket, MenuChooseCallPacket, Net, NetworkWorldData, PacketKind,
    PayloadDroppedCallPacket, PickedBuildPayloadCallPacket, PickedUnitPayloadCallPacket,
    PingCallPacket, PingLocationCallPacket, PlayerDisconnectCallPacket, PlayerSpawnCallPacket,
    ProviderEvent, RemoveMarkerCallPacket, RemoveQueueBlockCallPacket, RemoveTileCallPacket,
    RemoveWorldLabelCallPacket, RequestBuildPayloadCallPacket, RequestDropPayloadCallPacket,
    RequestItemCallPacket, RequestUnitPayloadCallPacket, RotateBlockCallPacket,
    SetCameraPositionCallPacket, SetFlagCallPacket, SetFloorCallPacket, SetItemCallPacket,
    SetItemsCallPacket, SetLiquidCallPacket, SetLiquidsCallPacket, SetMapAreaCallPacket,
    SetObjectivesCallPacket, SetOverlayCallPacket, SetPlayerTeamEditorCallPacket,
    SetPositionCallPacket, SetRuleCallPacket, SetRulesCallPacket, SetTeamCallPacket,
    SetTeamsCallPacket, SetTileBlocksCallPacket, SetTileCallPacket, SetTileFloorsCallPacket,
    SetTileItemsCallPacket, SetTileLiquidsCallPacket, SetTileOverlaysCallPacket,
    SetUnitCommandCallPacket, SetUnitStanceCallPacket, SoundAtCallPacket, SoundCallPacket,
    StateSnapshotCallPacket, StreamBuilder, Streamable, SyncVariableCallPacket,
    TakeItemsCallPacket, TextInputResultCallPacket, TileConfigCallPacket, TileTapCallPacket,
    TraceInfoCallPacket, TransferInventoryCallPacket, TransferItemEffectCallPacket,
    TransferItemToCallPacket, TransferItemToUnitCallPacket, UnitBuildingControlSelectCallPacket,
    UnitClearCallPacket, UnitControlCallPacket, UnitEnteredPayloadCallPacket,
    UpdateMarkerCallPacket, UpdateMarkerTextCallPacket, UpdateMarkerTextureCallPacket,
};
use crate::mindustry::vars::MAX_PLAYER_PREVIEW_PLANS;
use crate::mindustry::world::{BlockId, BuildingRef as WorldBuildingRef, Tile, Tiles};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const PING_INTERVAL: Duration = Duration::from_secs(1);
pub const CLIENT_PLAYER_SYNC_INTERVAL_MS: i64 = 66;
pub const CLIENT_PLAN_SYNC_INTERVAL_MS: i64 = 500;
pub const CLIENT_PLAN_PREVIEW_CHUNK_SIZE: usize = 900 / 12;
pub const DEFAULT_CLIENT_VERSION: i32 = 157;
pub const DEFAULT_CLIENT_VERSION_TYPE: &str = "official";

pub type PacketHandler = Arc<dyn Fn(PacketKind) + Send + Sync + 'static>;
pub type BinaryPacketHandler = Arc<dyn Fn(&[u8]) + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientCameraView {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for ClientCameraView {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ClientInputSnapshot {
    pub chatting: bool,
    pub building: bool,
    /// Java sends `Mechc.baseRotation()` here and `0` for non-mechs.
    pub base_rotation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientTileBlockKind {
    Block,
    Floor,
    Overlay,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ClientTileStorageMirror {
    pub items: BTreeMap<String, i32>,
    pub liquids: BTreeMap<String, f32>,
    pub team: Option<i32>,
    pub health: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ClientMarkerTextMirror {
    pub fetch: bool,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ClientMarkerMirror {
    pub marker_json: Option<String>,
    pub controls: BTreeMap<LMarkerControl, (f64, f64, f64)>,
    pub text_controls: BTreeMap<LMarkerControl, ClientMarkerTextMirror>,
    pub texture: Option<TypeValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientMapAreaMirror {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl From<&SetMapAreaCallPacket> for ClientMapAreaMirror {
    fn from(packet: &SetMapAreaCallPacket) -> Self {
        Self {
            x: packet.x,
            y: packet.y,
            width: packet.width,
            height: packet.height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientPlayerTeamEditorMirror {
    pub player: EntityRef,
    pub team: TeamId,
}

impl From<&SetPlayerTeamEditorCallPacket> for ClientPlayerTeamEditorMirror {
    fn from(packet: &SetPlayerTeamEditorCallPacket) -> Self {
        Self {
            player: packet.player,
            team: packet.team,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientWeatherMirror {
    pub weather_id: Option<ContentId>,
    pub intensity: f32,
    pub duration: f32,
    pub wind_x: f32,
    pub wind_y: f32,
}

impl From<&CreateWeatherCallPacket> for ClientWeatherMirror {
    fn from(packet: &CreateWeatherCallPacket) -> Self {
        Self {
            weather_id: packet.weather_id,
            intensity: packet.intensity,
            duration: packet.duration,
            wind_x: packet.wind_x,
            wind_y: packet.wind_y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientLogicExplosionMirror {
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

impl From<&LogicExplosionCallPacket> for ClientLogicExplosionMirror {
    fn from(packet: &LogicExplosionCallPacket) -> Self {
        Self {
            team: packet.team,
            x: packet.x,
            y: packet.y,
            radius: packet.radius,
            damage: packet.damage,
            air: packet.air,
            ground: packet.ground,
            pierce: packet.pierce,
            effect: packet.effect,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientBulletMirror {
    pub bullet_type_id: ContentId,
    pub team: TeamId,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub damage: f32,
    pub velocity_scl: f32,
    pub lifetime_scl: f32,
}

impl From<&CreateBulletCallPacket> for ClientBulletMirror {
    fn from(packet: &CreateBulletCallPacket) -> Self {
        Self {
            bullet_type_id: packet.bullet_type_id,
            team: packet.team,
            x: packet.x,
            y: packet.y,
            angle: packet.angle,
            damage: packet.damage,
            velocity_scl: packet.velocity_scl,
            lifetime_scl: packet.lifetime_scl,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientPlayerSpawnMirror {
    pub tile: Option<i32>,
    pub player: EntityRef,
}

impl From<&PlayerSpawnCallPacket> for ClientPlayerSpawnMirror {
    fn from(packet: &PlayerSpawnCallPacket) -> Self {
        Self {
            tile: packet.tile,
            player: packet.player,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientGameStateMirror {
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

impl From<&StateSnapshotCallPacket> for ClientGameStateMirror {
    fn from(snapshot: &StateSnapshotCallPacket) -> Self {
        Self {
            wave_time: snapshot.wave_time,
            wave: snapshot.wave,
            enemies: snapshot.enemies,
            paused: snapshot.paused,
            game_over: snapshot.game_over,
            time_data: snapshot.time_data,
            tps: snapshot.tps,
            rand0: snapshot.rand0,
            rand1: snapshot.rand1,
            core_data: snapshot.core_data.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientConnectConfig {
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

impl Default for ClientConnectConfig {
    fn default() -> Self {
        Self {
            version: DEFAULT_CLIENT_VERSION,
            version_type: DEFAULT_CLIENT_VERSION_TYPE.into(),
            mods: Vec::new(),
            name: "player".into(),
            locale: "en_US".into(),
            uuid: String::new(),
            usid: String::new(),
            mobile: false,
            color: 0,
            uuid_crc32: None,
        }
    }
}

impl ClientConnectConfig {
    pub fn to_connect_packet(&self) -> ConnectPacket {
        ConnectPacket {
            version: self.version,
            version_type: self.version_type.clone(),
            mods: self.mods.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            uuid: self.uuid.clone(),
            usid: self.usid.clone(),
            mobile: self.mobile,
            color: self.color,
            uuid_crc32: self.uuid_crc32,
        }
    }
}

#[derive(Clone, Default)]
pub struct NetClientState {
    pub quiet: bool,
    pub connecting: bool,
    pub connected: bool,
    pub ping_ms: u32,
    pub connection_attempts: u64,
    pub manual_disconnects: u64,
    pub connect_events: u64,
    pub disconnect_events: u64,
    pub world_stream_events: u64,
    pub update_count: u64,
    pub total_provider_event_count: u64,
    pub last_provider_event_count: usize,
    pub timeout_resets: u64,
    pub timeout_deadline: Option<Instant>,
    pub last_timeout_reset_at: Option<Instant>,
    pub last_update_at: Option<Instant>,
    pub last_connect: Option<Connect>,
    pub last_disconnect: Option<Disconnect>,
    pub last_world_stream: Option<Streamable>,
    pub last_loaded_world_data: Option<NetworkWorldData>,
    pub last_world_data_error: Option<String>,
    pub last_packet: Option<PacketKind>,
    pub last_message: Option<String>,
    pub last_message_unformatted: Option<String>,
    pub last_message_sender: Option<EntityRef>,
    pub message_packets_seen: u64,
    pub last_client_ui_packet: Option<PacketKind>,
    pub last_client_ui_packet_at: Option<Instant>,
    pub client_ui_packets_seen: u64,
    pub last_announcement: Option<String>,
    pub last_info_message: Option<String>,
    pub last_toast_message: Option<String>,
    pub last_hud_text: Option<String>,
    pub last_open_uri: Option<String>,
    pub last_clipboard_text: Option<String>,
    pub last_debug_status: Option<DebugStatusClientCallPacket>,
    pub last_debug_status_at: Option<Instant>,
    pub debug_status_packets_seen: u64,
    pub last_debug_status_unreliable: Option<DebugStatusClientUnreliableCallPacket>,
    pub last_debug_status_unreliable_at: Option<Instant>,
    pub debug_status_unreliable_packets_seen: u64,
    pub last_menu_choose: Option<MenuChooseCallPacket>,
    pub last_menu_choose_at: Option<Instant>,
    pub menu_choose_packets_seen: u64,
    pub last_text_input_result: Option<TextInputResultCallPacket>,
    pub last_text_input_result_at: Option<Instant>,
    pub text_input_result_packets_seen: u64,
    pub last_world_update_packet: Option<PacketKind>,
    pub last_world_update_packet_at: Option<Instant>,
    pub world_update_packets_seen: u64,
    pub logic_flags: BTreeSet<String>,
    pub map_area_mirror: Option<ClientMapAreaMirror>,
    pub player_team_editor_mirrors: BTreeMap<Option<i32>, ClientPlayerTeamEditorMirror>,
    pub weather_mirrors: Vec<ClientWeatherMirror>,
    pub logic_explosion_mirrors: Vec<ClientLogicExplosionMirror>,
    pub bullet_mirrors: Vec<ClientBulletMirror>,
    pub landed_pad_tiles: Vec<Option<i32>>,
    pub player_spawn_mirrors: Vec<ClientPlayerSpawnMirror>,
    pub logic_variable_mirrors: BTreeMap<(Option<i32>, i32), TypeValue>,
    pub last_unit_lifecycle_packet: Option<PacketKind>,
    pub last_unit_lifecycle_packet_at: Option<Instant>,
    pub unit_lifecycle_packets_seen: u64,
    pub last_marker_packet: Option<PacketKind>,
    pub last_marker_packet_at: Option<Instant>,
    pub marker_packets_seen: u64,
    pub marker_mirrors: BTreeMap<i32, ClientMarkerMirror>,
    pub removed_marker_ids: BTreeSet<i32>,
    pub removed_world_label_ids: BTreeSet<i32>,
    pub last_campaign_event_packet: Option<PacketKind>,
    pub last_campaign_event_packet_at: Option<Instant>,
    pub campaign_event_packets_seen: u64,
    pub last_player_disconnect: Option<PlayerDisconnectCallPacket>,
    pub last_player_disconnect_at: Option<Instant>,
    pub player_disconnect_packets_seen: u64,
    pub removed_entity_ids: BTreeSet<i32>,
    pub last_trace_info: Option<TraceInfoCallPacket>,
    pub last_trace_info_at: Option<Instant>,
    pub trace_info_packets_seen: u64,
    pub last_sound: Option<SoundCallPacket>,
    pub last_sound_at: Option<Instant>,
    pub sound_packets_seen: u64,
    pub last_sound_at_packet: Option<SoundAtCallPacket>,
    pub last_sound_at_packet_at: Option<Instant>,
    pub sound_at_packets_seen: u64,
    pub last_effect: Option<EffectCallPacket>,
    pub last_effect_at: Option<Instant>,
    pub effect_packets_seen: u64,
    pub last_effect_with_data: Option<EffectCallPacket2>,
    pub last_effect_with_data_at: Option<Instant>,
    pub effect_with_data_packets_seen: u64,
    pub last_reliable_effect: Option<EffectReliableCallPacket>,
    pub last_reliable_effect_at: Option<Instant>,
    pub reliable_effect_packets_seen: u64,
    pub last_camera_position: Option<SetCameraPositionCallPacket>,
    pub last_camera_position_at: Option<Instant>,
    pub camera_position_packets_seen: u64,
    pub last_set_position: Option<SetPositionCallPacket>,
    pub last_set_position_at: Option<Instant>,
    pub set_position_packets_seen: u64,
    pub kicked: bool,
    pub last_kick: Option<KickCallPacket>,
    pub last_kick_at: Option<Instant>,
    pub last_kick_reason: Option<KickCallPacket2>,
    pub last_kick_reason_at: Option<Instant>,
    pub kick_packets_seen: u64,
    pub last_block_snapshot: Option<BlockSnapshotCallPacket>,
    pub last_block_snapshot_at: Option<Instant>,
    pub block_snapshot_packets_seen: u64,
    pub last_connect_call: Option<ConnectCallPacket>,
    pub last_connect_call_at: Option<Instant>,
    pub connect_call_packets_seen: u64,
    pub connect_config: Option<ClientConnectConfig>,
    pub connect_packet_sent: bool,
    pub last_sent_connect_packet: Option<ConnectPacket>,
    pub last_connect_packet_error: Option<String>,
    pub auto_confirm_world_stream: bool,
    pub connect_confirm_sent: bool,
    pub last_connect_confirm_error: Option<String>,
    pub world_data_loading: bool,
    pub world_data_begin_packets_seen: u64,
    pub last_world_data_begin_at: Option<Instant>,
    pub next_ping_at: Option<Instant>,
    pub next_client_snapshot_at: Option<Instant>,
    pub next_client_plan_snapshot_at: Option<Instant>,
    pub ping_requests_sent: u64,
    pub ping_responses_received: u64,
    pub last_ping_request_time: Option<i64>,
    pub last_ping_request_at: Option<Instant>,
    pub last_ping_response_time: Option<i64>,
    pub last_ping_response_at: Option<Instant>,
    pub last_ping_request_error: Option<String>,
    pub timeout_disconnects: u64,
    pub last_stream_id: Option<i32>,
    pub last_stream_len: usize,
    pub last_binary_stream: Option<Vec<u8>>,
    pub last_entity_snapshot: Option<EntitySnapshotCallPacket>,
    pub last_entity_snapshot_at: Option<Instant>,
    pub entity_snapshot_packets_seen: u64,
    pub last_hidden_snapshot: Option<HiddenSnapshotCallPacket>,
    pub last_hidden_snapshot_at: Option<Instant>,
    pub hidden_snapshot_packets_seen: u64,
    pub last_state_snapshot: Option<StateSnapshotCallPacket>,
    pub last_state_snapshot_at: Option<Instant>,
    pub state_snapshot_packets_seen: u64,
    pub last_state_snapshot_mirror: Option<ClientGameStateMirror>,
    pub last_set_rules: Option<SetRulesCallPacket>,
    pub last_set_rules_at: Option<Instant>,
    pub set_rules_packets_seen: u64,
    pub last_rules_json: Option<String>,
    pub last_set_rule: Option<SetRuleCallPacket>,
    pub last_set_rule_at: Option<Instant>,
    pub set_rule_packets_seen: u64,
    pub rule_json_patches: BTreeMap<String, String>,
    pub last_set_objectives: Option<SetObjectivesCallPacket>,
    pub last_set_objectives_at: Option<Instant>,
    pub set_objectives_packets_seen: u64,
    pub last_objectives_json: Option<String>,
    pub last_clear_objectives_at: Option<Instant>,
    pub clear_objectives_packets_seen: u64,
    pub last_complete_objective: Option<CompleteObjectiveCallPacket>,
    pub last_complete_objective_at: Option<Instant>,
    pub complete_objective_packets_seen: u64,
    pub completed_objective_indices: Vec<i32>,
    pub last_client_plan_snapshot_received: Option<ClientPlanSnapshotReceivedCallPacket>,
    pub last_client_plan_snapshot_received_at: Option<Instant>,
    pub client_plan_snapshot_received_packets_seen: u64,
    pub last_set_item: Option<SetItemCallPacket>,
    pub last_set_item_at: Option<Instant>,
    pub set_item_packets_seen: u64,
    pub last_set_items: Option<SetItemsCallPacket>,
    pub last_set_items_at: Option<Instant>,
    pub set_items_packets_seen: u64,
    pub last_clear_items: Option<ClearItemsCallPacket>,
    pub last_clear_items_at: Option<Instant>,
    pub clear_items_packets_seen: u64,
    pub last_set_liquid: Option<SetLiquidCallPacket>,
    pub last_set_liquid_at: Option<Instant>,
    pub set_liquid_packets_seen: u64,
    pub last_set_liquids: Option<SetLiquidsCallPacket>,
    pub last_set_liquids_at: Option<Instant>,
    pub set_liquids_packets_seen: u64,
    pub last_clear_liquids: Option<ClearLiquidsCallPacket>,
    pub last_clear_liquids_at: Option<Instant>,
    pub clear_liquids_packets_seen: u64,
    pub building_storage_mirrors: BTreeMap<i32, ClientTileStorageMirror>,
    pub last_take_items: Option<TakeItemsCallPacket>,
    pub last_take_items_at: Option<Instant>,
    pub take_items_packets_seen: u64,
    pub last_transfer_item_effect: Option<TransferItemEffectCallPacket>,
    pub last_transfer_item_effect_at: Option<Instant>,
    pub transfer_item_effect_packets_seen: u64,
    pub last_transfer_item_to: Option<TransferItemToCallPacket>,
    pub last_transfer_item_to_at: Option<Instant>,
    pub transfer_item_to_packets_seen: u64,
    pub last_transfer_item_to_unit: Option<TransferItemToUnitCallPacket>,
    pub last_transfer_item_to_unit_at: Option<Instant>,
    pub transfer_item_to_unit_packets_seen: u64,
    pub last_request_item: Option<RequestItemCallPacket>,
    pub last_request_item_at: Option<Instant>,
    pub request_item_packets_seen: u64,
    pub last_transfer_inventory: Option<TransferInventoryCallPacket>,
    pub last_transfer_inventory_at: Option<Instant>,
    pub transfer_inventory_packets_seen: u64,
    pub last_request_build_payload: Option<RequestBuildPayloadCallPacket>,
    pub last_request_build_payload_at: Option<Instant>,
    pub request_build_payload_packets_seen: u64,
    pub last_request_unit_payload: Option<RequestUnitPayloadCallPacket>,
    pub last_request_unit_payload_at: Option<Instant>,
    pub request_unit_payload_packets_seen: u64,
    pub last_picked_build_payload: Option<PickedBuildPayloadCallPacket>,
    pub last_picked_build_payload_at: Option<Instant>,
    pub picked_build_payload_packets_seen: u64,
    pub last_picked_unit_payload: Option<PickedUnitPayloadCallPacket>,
    pub last_picked_unit_payload_at: Option<Instant>,
    pub picked_unit_payload_packets_seen: u64,
    pub last_request_drop_payload: Option<RequestDropPayloadCallPacket>,
    pub last_request_drop_payload_at: Option<Instant>,
    pub request_drop_payload_packets_seen: u64,
    pub last_payload_dropped: Option<PayloadDroppedCallPacket>,
    pub last_payload_dropped_at: Option<Instant>,
    pub payload_dropped_packets_seen: u64,
    pub last_unit_entered_payload: Option<UnitEnteredPayloadCallPacket>,
    pub last_unit_entered_payload_at: Option<Instant>,
    pub unit_entered_payload_packets_seen: u64,
    pub last_ping_location: Option<PingLocationCallPacket>,
    pub last_ping_location_at: Option<Instant>,
    pub ping_location_packets_seen: u64,
    pub last_delete_plans: Option<DeletePlansCallPacket>,
    pub last_delete_plans_at: Option<Instant>,
    pub delete_plans_packets_seen: u64,
    pub last_command_building: Option<CommandBuildingCallPacket>,
    pub last_command_building_at: Option<Instant>,
    pub command_building_packets_seen: u64,
    pub last_command_units: Option<CommandUnitsCallPacket>,
    pub last_command_units_at: Option<Instant>,
    pub command_units_packets_seen: u64,
    pub last_set_unit_command: Option<SetUnitCommandCallPacket>,
    pub last_set_unit_command_at: Option<Instant>,
    pub set_unit_command_packets_seen: u64,
    pub last_set_unit_stance: Option<SetUnitStanceCallPacket>,
    pub last_set_unit_stance_at: Option<Instant>,
    pub set_unit_stance_packets_seen: u64,
    pub last_building_control_select: Option<BuildingControlSelectCallPacket>,
    pub last_building_control_select_at: Option<Instant>,
    pub building_control_select_packets_seen: u64,
    pub last_unit_building_control_select: Option<UnitBuildingControlSelectCallPacket>,
    pub last_unit_building_control_select_at: Option<Instant>,
    pub unit_building_control_select_packets_seen: u64,
    pub last_unit_control: Option<UnitControlCallPacket>,
    pub last_unit_control_at: Option<Instant>,
    pub unit_control_packets_seen: u64,
    pub last_unit_clear: Option<UnitClearCallPacket>,
    pub last_unit_clear_at: Option<Instant>,
    pub unit_clear_packets_seen: u64,
    pub last_remove_queue_block: Option<RemoveQueueBlockCallPacket>,
    pub last_remove_queue_block_at: Option<Instant>,
    pub remove_queue_block_packets_seen: u64,
    pub last_tile_config: Option<TileConfigCallPacket>,
    pub last_tile_config_at: Option<Instant>,
    pub tile_config_packets_seen: u64,
    pub last_rotate_block: Option<RotateBlockCallPacket>,
    pub last_rotate_block_at: Option<Instant>,
    pub rotate_block_packets_seen: u64,
    pub last_tile_tap: Option<TileTapCallPacket>,
    pub last_tile_tap_at: Option<Instant>,
    pub tile_tap_packets_seen: u64,
    pub last_server_snapshot_at: Option<Instant>,
    pub last_sent_client_snapshot_id: i32,
    pub last_sent_client_snapshot: Option<ClientSnapshotCallPacket>,
    pub client_snapshot_packets_sent: u64,
    pub last_client_snapshot_error: Option<String>,
    pub last_sent_client_plan_snapshot: Option<ClientPlanSnapshotCallPacket>,
    pub client_plan_snapshot_packets_sent: u64,
    pub last_client_plan_snapshot_error: Option<String>,
    pub last_provider_events: Vec<ProviderEvent>,
    packet_handlers: Vec<PacketHandler>,
    binary_packet_handlers: Vec<BinaryPacketHandler>,
    handled_client_cursor: usize,
}

impl fmt::Debug for NetClientState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NetClientState")
            .field("quiet", &self.quiet)
            .field("connecting", &self.connecting)
            .field("connected", &self.connected)
            .field("ping_ms", &self.ping_ms)
            .field("connection_attempts", &self.connection_attempts)
            .field("manual_disconnects", &self.manual_disconnects)
            .field("connect_events", &self.connect_events)
            .field("disconnect_events", &self.disconnect_events)
            .field("world_stream_events", &self.world_stream_events)
            .field("update_count", &self.update_count)
            .field(
                "total_provider_event_count",
                &self.total_provider_event_count,
            )
            .field("last_provider_event_count", &self.last_provider_event_count)
            .field("timeout_resets", &self.timeout_resets)
            .field("timeout_deadline", &self.timeout_deadline)
            .field("last_timeout_reset_at", &self.last_timeout_reset_at)
            .field("last_update_at", &self.last_update_at)
            .field("last_connect", &self.last_connect)
            .field("last_disconnect", &self.last_disconnect)
            .field("last_world_stream", &self.last_world_stream)
            .field("last_loaded_world_data", &self.last_loaded_world_data)
            .field("last_world_data_error", &self.last_world_data_error)
            .field("last_packet", &self.last_packet)
            .field("last_message", &self.last_message)
            .field("message_packets_seen", &self.message_packets_seen)
            .field("client_ui_packets_seen", &self.client_ui_packets_seen)
            .field("last_announcement", &self.last_announcement)
            .field("last_info_message", &self.last_info_message)
            .field("last_toast_message", &self.last_toast_message)
            .field("last_hud_text", &self.last_hud_text)
            .field("last_open_uri", &self.last_open_uri)
            .field("debug_status_packets_seen", &self.debug_status_packets_seen)
            .field(
                "debug_status_unreliable_packets_seen",
                &self.debug_status_unreliable_packets_seen,
            )
            .field("menu_choose_packets_seen", &self.menu_choose_packets_seen)
            .field(
                "text_input_result_packets_seen",
                &self.text_input_result_packets_seen,
            )
            .field("world_update_packets_seen", &self.world_update_packets_seen)
            .field("logic_flags", &self.logic_flags)
            .field("map_area_mirror", &self.map_area_mirror)
            .field(
                "player_team_editor_mirrors",
                &self.player_team_editor_mirrors,
            )
            .field("weather_mirrors_len", &self.weather_mirrors.len())
            .field(
                "logic_explosion_mirrors_len",
                &self.logic_explosion_mirrors.len(),
            )
            .field("bullet_mirrors_len", &self.bullet_mirrors.len())
            .field("landed_pad_tiles_len", &self.landed_pad_tiles.len())
            .field("player_spawn_mirrors_len", &self.player_spawn_mirrors.len())
            .field("logic_variable_mirrors", &self.logic_variable_mirrors)
            .field(
                "unit_lifecycle_packets_seen",
                &self.unit_lifecycle_packets_seen,
            )
            .field("marker_packets_seen", &self.marker_packets_seen)
            .field("marker_mirrors", &self.marker_mirrors)
            .field("removed_marker_ids", &self.removed_marker_ids)
            .field("removed_world_label_ids", &self.removed_world_label_ids)
            .field(
                "campaign_event_packets_seen",
                &self.campaign_event_packets_seen,
            )
            .field(
                "player_disconnect_packets_seen",
                &self.player_disconnect_packets_seen,
            )
            .field("removed_entity_ids", &self.removed_entity_ids)
            .field("trace_info_packets_seen", &self.trace_info_packets_seen)
            .field("sound_packets_seen", &self.sound_packets_seen)
            .field("sound_at_packets_seen", &self.sound_at_packets_seen)
            .field("effect_packets_seen", &self.effect_packets_seen)
            .field(
                "effect_with_data_packets_seen",
                &self.effect_with_data_packets_seen,
            )
            .field(
                "reliable_effect_packets_seen",
                &self.reliable_effect_packets_seen,
            )
            .field(
                "camera_position_packets_seen",
                &self.camera_position_packets_seen,
            )
            .field("set_position_packets_seen", &self.set_position_packets_seen)
            .field("kicked", &self.kicked)
            .field("kick_packets_seen", &self.kick_packets_seen)
            .field(
                "block_snapshot_packets_seen",
                &self.block_snapshot_packets_seen,
            )
            .field("connect_call_packets_seen", &self.connect_call_packets_seen)
            .field("connect_config", &self.connect_config)
            .field("connect_packet_sent", &self.connect_packet_sent)
            .field("last_sent_connect_packet", &self.last_sent_connect_packet)
            .field("last_connect_packet_error", &self.last_connect_packet_error)
            .field("auto_confirm_world_stream", &self.auto_confirm_world_stream)
            .field("connect_confirm_sent", &self.connect_confirm_sent)
            .field(
                "last_connect_confirm_error",
                &self.last_connect_confirm_error,
            )
            .field("world_data_loading", &self.world_data_loading)
            .field(
                "world_data_begin_packets_seen",
                &self.world_data_begin_packets_seen,
            )
            .field("next_ping_at", &self.next_ping_at)
            .field("next_client_snapshot_at", &self.next_client_snapshot_at)
            .field(
                "next_client_plan_snapshot_at",
                &self.next_client_plan_snapshot_at,
            )
            .field("ping_requests_sent", &self.ping_requests_sent)
            .field("ping_responses_received", &self.ping_responses_received)
            .field("last_ping_request_time", &self.last_ping_request_time)
            .field("last_ping_response_time", &self.last_ping_response_time)
            .field("timeout_disconnects", &self.timeout_disconnects)
            .field("last_stream_id", &self.last_stream_id)
            .field("last_stream_len", &self.last_stream_len)
            .field(
                "entity_snapshot_packets_seen",
                &self.entity_snapshot_packets_seen,
            )
            .field(
                "hidden_snapshot_packets_seen",
                &self.hidden_snapshot_packets_seen,
            )
            .field(
                "state_snapshot_packets_seen",
                &self.state_snapshot_packets_seen,
            )
            .field(
                "last_state_snapshot_mirror",
                &self.last_state_snapshot_mirror,
            )
            .field("set_rules_packets_seen", &self.set_rules_packets_seen)
            .field("last_rules_json", &self.last_rules_json)
            .field("set_rule_packets_seen", &self.set_rule_packets_seen)
            .field("rule_json_patches", &self.rule_json_patches)
            .field(
                "set_objectives_packets_seen",
                &self.set_objectives_packets_seen,
            )
            .field(
                "clear_objectives_packets_seen",
                &self.clear_objectives_packets_seen,
            )
            .field(
                "complete_objective_packets_seen",
                &self.complete_objective_packets_seen,
            )
            .field(
                "completed_objective_indices",
                &self.completed_objective_indices,
            )
            .field(
                "client_plan_snapshot_received_packets_seen",
                &self.client_plan_snapshot_received_packets_seen,
            )
            .field("set_item_packets_seen", &self.set_item_packets_seen)
            .field("set_items_packets_seen", &self.set_items_packets_seen)
            .field("clear_items_packets_seen", &self.clear_items_packets_seen)
            .field("set_liquid_packets_seen", &self.set_liquid_packets_seen)
            .field("set_liquids_packets_seen", &self.set_liquids_packets_seen)
            .field(
                "clear_liquids_packets_seen",
                &self.clear_liquids_packets_seen,
            )
            .field("building_storage_mirrors", &self.building_storage_mirrors)
            .field("take_items_packets_seen", &self.take_items_packets_seen)
            .field(
                "transfer_item_effect_packets_seen",
                &self.transfer_item_effect_packets_seen,
            )
            .field(
                "transfer_item_to_packets_seen",
                &self.transfer_item_to_packets_seen,
            )
            .field(
                "transfer_item_to_unit_packets_seen",
                &self.transfer_item_to_unit_packets_seen,
            )
            .field("request_item_packets_seen", &self.request_item_packets_seen)
            .field(
                "transfer_inventory_packets_seen",
                &self.transfer_inventory_packets_seen,
            )
            .field(
                "request_build_payload_packets_seen",
                &self.request_build_payload_packets_seen,
            )
            .field(
                "request_unit_payload_packets_seen",
                &self.request_unit_payload_packets_seen,
            )
            .field(
                "picked_build_payload_packets_seen",
                &self.picked_build_payload_packets_seen,
            )
            .field(
                "picked_unit_payload_packets_seen",
                &self.picked_unit_payload_packets_seen,
            )
            .field(
                "request_drop_payload_packets_seen",
                &self.request_drop_payload_packets_seen,
            )
            .field(
                "payload_dropped_packets_seen",
                &self.payload_dropped_packets_seen,
            )
            .field(
                "unit_entered_payload_packets_seen",
                &self.unit_entered_payload_packets_seen,
            )
            .field(
                "ping_location_packets_seen",
                &self.ping_location_packets_seen,
            )
            .field("delete_plans_packets_seen", &self.delete_plans_packets_seen)
            .field(
                "command_building_packets_seen",
                &self.command_building_packets_seen,
            )
            .field(
                "command_units_packets_seen",
                &self.command_units_packets_seen,
            )
            .field(
                "set_unit_command_packets_seen",
                &self.set_unit_command_packets_seen,
            )
            .field(
                "set_unit_stance_packets_seen",
                &self.set_unit_stance_packets_seen,
            )
            .field(
                "building_control_select_packets_seen",
                &self.building_control_select_packets_seen,
            )
            .field(
                "unit_building_control_select_packets_seen",
                &self.unit_building_control_select_packets_seen,
            )
            .field("unit_control_packets_seen", &self.unit_control_packets_seen)
            .field("unit_clear_packets_seen", &self.unit_clear_packets_seen)
            .field(
                "remove_queue_block_packets_seen",
                &self.remove_queue_block_packets_seen,
            )
            .field("tile_config_packets_seen", &self.tile_config_packets_seen)
            .field("rotate_block_packets_seen", &self.rotate_block_packets_seen)
            .field("tile_tap_packets_seen", &self.tile_tap_packets_seen)
            .field("last_server_snapshot_at", &self.last_server_snapshot_at)
            .field(
                "last_sent_client_snapshot_id",
                &self.last_sent_client_snapshot_id,
            )
            .field(
                "client_snapshot_packets_sent",
                &self.client_snapshot_packets_sent,
            )
            .field(
                "client_plan_snapshot_packets_sent",
                &self.client_plan_snapshot_packets_sent,
            )
            .field(
                "last_binary_stream_len",
                &self.last_binary_stream.as_ref().map(|s| s.len()),
            )
            .field("last_provider_events_len", &self.last_provider_events.len())
            .field("packet_handler_count", &self.packet_handlers.len())
            .field(
                "binary_packet_handler_count",
                &self.binary_packet_handlers.len(),
            )
            .finish()
    }
}

impl NetClientState {
    fn reset_ping_state(&mut self) {
        self.next_ping_at = None;
        self.ping_requests_sent = 0;
        self.ping_responses_received = 0;
        self.last_ping_request_time = None;
        self.last_ping_request_at = None;
        self.last_ping_response_time = None;
        self.last_ping_response_at = None;
        self.last_ping_request_error = None;
        self.ping_ms = 0;
    }

    fn reset_client_gameplay_sync_state(&mut self) {
        self.next_client_snapshot_at = None;
        self.next_client_plan_snapshot_at = None;
    }

    fn clear_loading_stream_tracking(&mut self) {
        self.last_stream_id = None;
        self.last_stream_len = 0;
    }

    fn clear_timeout_clock(&mut self) {
        self.timeout_deadline = None;
    }

    fn record_client_ui_packet(&mut self, packet: &PacketKind) {
        self.client_ui_packets_seen = self.client_ui_packets_seen.saturating_add(1);
        self.last_client_ui_packet = Some(packet.clone());
        self.last_client_ui_packet_at = Some(Instant::now());
    }

    fn record_world_update_packet(&mut self, packet: &PacketKind) {
        self.world_update_packets_seen = self.world_update_packets_seen.saturating_add(1);
        self.last_world_update_packet = Some(packet.clone());
        self.last_world_update_packet_at = Some(Instant::now());
    }

    fn record_unit_lifecycle_packet(&mut self, packet: &PacketKind) {
        self.unit_lifecycle_packets_seen = self.unit_lifecycle_packets_seen.saturating_add(1);
        self.last_unit_lifecycle_packet = Some(packet.clone());
        self.last_unit_lifecycle_packet_at = Some(Instant::now());
    }

    fn record_marker_packet(&mut self, packet: &PacketKind) {
        self.marker_packets_seen = self.marker_packets_seen.saturating_add(1);
        self.last_marker_packet = Some(packet.clone());
        self.last_marker_packet_at = Some(Instant::now());
    }

    fn record_campaign_event_packet(&mut self, packet: &PacketKind) {
        self.campaign_event_packets_seen = self.campaign_event_packets_seen.saturating_add(1);
        self.last_campaign_event_packet = Some(packet.clone());
        self.last_campaign_event_packet_at = Some(Instant::now());
    }

    fn record_server_kick_disconnect(&mut self) {
        self.quiet = true;
        self.connecting = false;
        self.connected = false;
        self.world_data_loading = false;
        self.kicked = true;
        self.connect_packet_sent = false;
        self.connect_confirm_sent = false;
        self.last_connect_confirm_error = None;
        self.last_loaded_world_data = None;
        self.last_world_data_error = None;
        self.removed_entity_ids.clear();
        self.reset_ping_state();
        self.reset_client_gameplay_sync_state();
        self.clear_loading_stream_tracking();
        self.clear_timeout_clock();
    }

    fn record_server_redirect_disconnect(&mut self) {
        self.quiet = true;
        self.connecting = false;
        self.connected = false;
        self.world_data_loading = false;
        self.kicked = false;
        self.connect_packet_sent = false;
        self.connect_confirm_sent = false;
        self.last_connect_confirm_error = None;
        self.last_loaded_world_data = None;
        self.last_world_data_error = None;
        self.removed_entity_ids.clear();
        self.reset_ping_state();
        self.reset_client_gameplay_sync_state();
        self.clear_loading_stream_tracking();
        self.clear_timeout_clock();
    }

    fn record_world_data_begin(&mut self) {
        self.connecting = true;
        self.connected = false;
        self.world_data_loading = true;
        self.kicked = false;
        self.world_data_begin_packets_seen = self.world_data_begin_packets_seen.saturating_add(1);
        self.last_world_data_begin_at = Some(Instant::now());
        self.connect_confirm_sent = false;
        self.last_connect_confirm_error = None;
        self.last_loaded_world_data = None;
        self.last_world_data_error = None;
        self.removed_entity_ids.clear();
        self.reset_client_gameplay_sync_state();
        self.clear_loading_stream_tracking();
        self.reset_loading_timeout();
    }

    fn reset_loading_timeout(&mut self) {
        let now = Instant::now();
        self.timeout_resets += 1;
        self.last_timeout_reset_at = Some(now);
        self.timeout_deadline = Some(now + DEFAULT_TIMEOUT);
    }

    fn record_loading_stream(&mut self, stream: &StreamBuilder) {
        let stream_len = stream.len();
        match self.last_stream_id {
            Some(id) if id == stream.id => {
                if stream_len > self.last_stream_len {
                    self.last_stream_len = stream_len;
                    self.reset_loading_timeout();
                }
            }
            _ => {
                self.last_stream_id = Some(stream.id);
                self.last_stream_len = stream_len;
                if stream_len > 0 {
                    self.reset_loading_timeout();
                }
            }
        }
    }

    fn record_connect(&mut self, connect: &Connect) {
        self.connecting = false;
        self.connected = true;
        self.world_data_loading = false;
        self.kicked = false;
        self.connect_events += 1;
        self.last_connect = Some(connect.clone());
        self.last_packet = Some(PacketKind::Connect(connect.clone()));
        self.connect_packet_sent = false;
        self.last_sent_connect_packet = None;
        self.last_connect_packet_error = None;
        self.connect_confirm_sent = false;
        self.last_connect_confirm_error = None;
        self.last_loaded_world_data = None;
        self.last_world_data_error = None;
        self.removed_entity_ids.clear();
        self.reset_ping_state();
        self.reset_client_gameplay_sync_state();
        self.clear_loading_stream_tracking();
    }

    fn record_disconnect(&mut self, disconnect: &Disconnect) {
        self.connecting = false;
        self.connected = false;
        self.world_data_loading = false;
        self.disconnect_events += 1;
        self.last_disconnect = Some(disconnect.clone());
        self.last_packet = Some(PacketKind::Disconnect(disconnect.clone()));
        self.connect_packet_sent = false;
        self.connect_confirm_sent = false;
        self.last_connect_confirm_error = None;
        self.last_loaded_world_data = None;
        self.last_world_data_error = None;
        self.removed_entity_ids.clear();
        self.reset_ping_state();
        self.reset_client_gameplay_sync_state();
        self.clear_loading_stream_tracking();
        self.clear_timeout_clock();
    }

    fn record_world_stream(&mut self, stream: &Streamable) {
        self.connecting = false;
        self.connected = false;
        self.world_data_loading = true;
        self.world_stream_events += 1;
        self.last_world_stream = Some(stream.clone());
        self.last_binary_stream = Some(stream.stream.clone());
        match read_world_data(&stream.stream) {
            Ok(world_data) => {
                self.connected = true;
                self.world_data_loading = false;
                self.last_loaded_world_data = Some(world_data);
                self.last_world_data_error = None;
            }
            Err(error) => {
                self.connected = false;
                self.world_data_loading = true;
                self.last_world_data_error = Some(error.to_string());
                self.last_loaded_world_data = None;
            }
        }
        self.last_packet = Some(PacketKind::Streamable(stream.clone()));
        self.next_ping_at = Some(Instant::now() + PING_INTERVAL);
        self.reset_client_gameplay_sync_state();
        self.clear_loading_stream_tracking();
        self.clear_timeout_clock();
    }
}

#[derive(Clone, Debug)]
pub struct NetClient {
    net: Arc<Mutex<Net>>,
    state: Arc<Mutex<NetClientState>>,
}

impl Default for NetClient {
    fn default() -> Self {
        Self::new()
    }
}

impl NetClient {
    pub fn new() -> Self {
        Self::with_net(Net::default())
    }

    pub fn with_net(mut net: Net) -> Self {
        let state = Arc::new(Mutex::new(NetClientState {
            auto_confirm_world_stream: true,
            ..NetClientState::default()
        }));
        Self::install_client_listeners(&mut net, &state);
        Self {
            net: Arc::new(Mutex::new(net)),
            state,
        }
    }

    pub fn state(&self) -> Arc<Mutex<NetClientState>> {
        Arc::clone(&self.state)
    }

    pub fn net(&self) -> Arc<Mutex<Net>> {
        Arc::clone(&self.net)
    }

    pub fn net_mut(&self) -> MutexGuard<'_, Net> {
        self.net.lock().unwrap()
    }

    pub fn set_quiet(&self, quiet: bool) {
        self.state.lock().unwrap().quiet = quiet;
    }

    pub fn is_connecting(&self) -> bool {
        self.state.lock().unwrap().connecting
    }

    pub fn get_ping(&self) -> u32 {
        self.state.lock().unwrap().ping_ms
    }

    pub fn reset_timeout(&self) {
        let mut state = self.state.lock().unwrap();
        state.reset_loading_timeout();
    }

    pub fn set_connect_config(&self, config: Option<ClientConnectConfig>) {
        let mut state = self.state.lock().unwrap();
        state.connect_config = config;
        state.connect_packet_sent = false;
        state.last_sent_connect_packet = None;
        state.last_connect_packet_error = None;
        state.connect_confirm_sent = false;
        state.last_connect_confirm_error = None;
        state.kicked = false;
        state.removed_entity_ids.clear();
        state.reset_ping_state();
        state.reset_client_gameplay_sync_state();
        state.clear_loading_stream_tracking();
        state.clear_timeout_clock();
    }

    pub fn get_connect_config(&self) -> Option<ClientConnectConfig> {
        self.state.lock().unwrap().connect_config.clone()
    }

    pub fn set_auto_confirm_world_stream(&self, enabled: bool) {
        self.state.lock().unwrap().auto_confirm_world_stream = enabled;
    }

    pub fn begin_connecting(&self) {
        let mut state = self.state.lock().unwrap();
        state.connecting = true;
        state.connected = false;
        state.world_data_loading = false;
        state.connection_attempts += 1;
        state.last_update_at = Some(Instant::now());
        state.connect_packet_sent = false;
        state.last_sent_connect_packet = None;
        state.last_connect_packet_error = None;
        state.connect_confirm_sent = false;
        state.last_connect_confirm_error = None;
        state.kicked = false;
        state.removed_entity_ids.clear();
        state.reset_ping_state();
        state.reset_client_gameplay_sync_state();
        state.clear_loading_stream_tracking();
        drop(state);
        self.reset_timeout();
    }

    pub fn disconnect_quietly(&self) {
        self.set_quiet(true);
        self.disconnect_no_reset();
    }

    pub fn disconnect_no_reset(&self) {
        {
            let mut net = self.net.lock().unwrap();
            net.disconnect();
        }

        let mut state = self.state.lock().unwrap();
        state.connecting = false;
        state.connected = false;
        state.world_data_loading = false;
        state.manual_disconnects += 1;
        state.last_update_at = Some(Instant::now());
        state.connect_packet_sent = false;
        state.connect_confirm_sent = false;
        state.last_connect_confirm_error = None;
        state.kicked = false;
        state.reset_ping_state();
        state.reset_client_gameplay_sync_state();
        state.clear_loading_stream_tracking();
        state.clear_timeout_clock();
    }

    pub fn add_packet_handler<F>(&self, handler: F)
    where
        F: Fn(PacketKind) + Send + Sync + 'static,
    {
        self.state
            .lock()
            .unwrap()
            .packet_handlers
            .push(Arc::new(handler));
    }

    pub fn get_packet_handlers(&self) -> Vec<PacketHandler> {
        self.state.lock().unwrap().packet_handlers.clone()
    }

    pub fn add_binary_packet_handler<F>(&self, handler: F)
    where
        F: Fn(&[u8]) + Send + Sync + 'static,
    {
        self.state
            .lock()
            .unwrap()
            .binary_packet_handlers
            .push(Arc::new(handler));
    }

    pub fn get_binary_packet_handlers(&self) -> Vec<BinaryPacketHandler> {
        self.state.lock().unwrap().binary_packet_handlers.clone()
    }

    pub fn send_client_snapshot(&self, snapshot: ClientSnapshotCallPacket) -> io::Result<()> {
        let result = {
            let mut net = self.net.lock().unwrap();
            net.send(
                &PacketKind::ClientSnapshotCallPacket(snapshot.clone()),
                false,
            )
        };

        let mut state = self.state.lock().unwrap();
        match &result {
            Ok(()) => {
                state.client_snapshot_packets_sent += 1;
                state.last_sent_client_snapshot = Some(snapshot);
                state.last_client_snapshot_error = None;
            }
            Err(error) => {
                state.last_client_snapshot_error = Some(error.to_string());
            }
        }
        result
    }

    pub fn send_client_plan_snapshot(
        &self,
        snapshot: ClientPlanSnapshotCallPacket,
    ) -> io::Result<()> {
        let result = {
            let mut net = self.net.lock().unwrap();
            net.send(
                &PacketKind::ClientPlanSnapshotCallPacket(snapshot.clone()),
                false,
            )
        };

        let mut state = self.state.lock().unwrap();
        match &result {
            Ok(()) => {
                state.client_plan_snapshot_packets_sent += 1;
                state.last_sent_client_plan_snapshot = Some(snapshot);
                state.last_client_plan_snapshot_error = None;
            }
            Err(error) => {
                state.last_client_plan_snapshot_error = Some(error.to_string());
            }
        }
        result
    }

    pub fn next_client_snapshot_packet(
        &self,
        player: &PlayerComp,
        unit: Option<&UnitComp>,
        input: ClientInputSnapshot,
        camera: ClientCameraView,
    ) -> ClientSnapshotCallPacket {
        let snapshot_id = {
            let mut state = self.state.lock().unwrap();
            let snapshot_id = state.last_sent_client_snapshot_id;
            state.last_sent_client_snapshot_id = state.last_sent_client_snapshot_id.wrapping_add(1);
            snapshot_id
        };

        Self::client_snapshot_packet(snapshot_id, player, unit, input, camera)
    }

    pub fn send_next_client_snapshot(
        &self,
        player: &PlayerComp,
        unit: Option<&UnitComp>,
        input: ClientInputSnapshot,
        camera: ClientCameraView,
    ) -> io::Result<ClientSnapshotCallPacket> {
        let snapshot = self.next_client_snapshot_packet(player, unit, input, camera);
        self.send_client_snapshot(snapshot.clone())?;
        Ok(snapshot)
    }

    pub fn tick_client_gameplay_sync(
        &self,
        player: &mut PlayerComp,
        unit: Option<&UnitComp>,
        input: ClientInputSnapshot,
        camera: ClientCameraView,
        preview_plans: &[BuildPlan],
    ) -> io::Result<()> {
        let mut first_error = None;

        if let Err(error) = self.tick_client_snapshot_sync(&*player, unit, input, camera) {
            first_error = Some(error);
        }

        if let Err(error) = self.tick_client_plan_sync(player, preview_plans) {
            if first_error.is_none() {
                first_error = Some(error);
            }
        }

        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn tick_client_snapshot_sync(
        &self,
        player: &PlayerComp,
        unit: Option<&UnitComp>,
        input: ClientInputSnapshot,
        camera: ClientCameraView,
    ) -> io::Result<()> {
        let now = Instant::now();
        let should_send = {
            let mut state = self.state.lock().unwrap();
            if !state.connected || !state.connect_confirm_sent {
                return Ok(());
            }

            match state.next_client_snapshot_at {
                Some(deadline) if now < deadline => false,
                _ => {
                    state.next_client_snapshot_at =
                        Some(now + Duration::from_millis(CLIENT_PLAYER_SYNC_INTERVAL_MS as u64));
                    true
                }
            }
        };

        if should_send {
            self.send_next_client_snapshot(player, unit, input, camera)?;
        }

        Ok(())
    }

    fn tick_client_plan_sync(
        &self,
        player: &mut PlayerComp,
        preview_plans: &[BuildPlan],
    ) -> io::Result<()> {
        let now = Instant::now();
        let should_send = {
            let mut state = self.state.lock().unwrap();
            if !state.connected || !state.connect_confirm_sent {
                return Ok(());
            }

            match state.next_client_plan_snapshot_at {
                Some(deadline) if now < deadline => false,
                _ => {
                    state.next_client_plan_snapshot_at =
                        Some(now + Duration::from_millis(CLIENT_PLAN_SYNC_INTERVAL_MS as u64));
                    true
                }
            }
        };

        if should_send {
            self.send_client_plan_snapshots_with_limit(
                player,
                preview_plans,
                MAX_PLAYER_PREVIEW_PLANS,
            )?;
        }

        Ok(())
    }

    pub fn client_snapshot_packet(
        snapshot_id: i32,
        player: &PlayerComp,
        unit: Option<&UnitComp>,
        input: ClientInputSnapshot,
        camera: ClientCameraView,
    ) -> ClientSnapshotCallPacket {
        let dead = player.dead();
        let active_unit = if dead { None } else { unit };
        let selected_block = player
            .selected_block
            .as_ref()
            .map(|block| block.name.clone());
        let plans = if player.is_builder() {
            active_unit.and_then(|unit| {
                if unit.builder.plans.is_empty() {
                    None
                } else {
                    Some(
                        unit.builder
                            .plans
                            .iter()
                            .map(BuildPlanWire::from_build_plan)
                            .collect(),
                    )
                }
            })
        } else {
            None
        };

        ClientSnapshotCallPacket {
            snapshot_id,
            unit_id: active_unit.map_or(-1, UnitComp::id),
            dead,
            x: active_unit.map_or(player.x, UnitComp::x),
            y: active_unit.map_or(player.y, UnitComp::y),
            pointer_x: active_unit.map_or(0.0, |unit| unit.weapons.aim_x),
            pointer_y: active_unit.map_or(0.0, |unit| unit.weapons.aim_y),
            rotation: active_unit.map_or(0.0, UnitComp::rotation),
            base_rotation: active_unit.map_or(0.0, |_| input.base_rotation),
            x_velocity: active_unit.map_or(0.0, |unit| unit.vel.vel.x),
            y_velocity: active_unit.map_or(0.0, |unit| unit.vel.vel.y),
            mining: active_unit.and_then(|unit| unit.to_sync_wire().mine_tile),
            boosting: player.boosting,
            shooting: player.shooting,
            chatting: input.chatting,
            building: input.building,
            selected_block,
            selected_rotation: player.selected_rotation,
            plans,
            view_x: camera.x,
            view_y: camera.y,
            view_width: camera.width,
            view_height: camera.height,
        }
    }

    pub fn client_plan_snapshot_packets(
        player: &mut PlayerComp,
        plans: &[BuildPlan],
    ) -> Vec<ClientPlanSnapshotCallPacket> {
        Self::client_plan_snapshot_packets_with_limit(player, plans, MAX_PLAYER_PREVIEW_PLANS)
    }

    pub fn client_plan_snapshot_packets_with_limit(
        player: &mut PlayerComp,
        plans: &[BuildPlan],
        max_preview_plans: usize,
    ) -> Vec<ClientPlanSnapshotCallPacket> {
        player.last_preview_plan_group = player.last_preview_plan_group.saturating_add(1);
        let group_id = player.last_preview_plan_group;
        let max_preview_plans = max_preview_plans.min(MAX_PLAYER_PREVIEW_PLANS);
        let plans: Vec<_> = plans
            .iter()
            .take(max_preview_plans)
            .map(BuildPlanWire::from_build_plan)
            .collect();

        if plans.is_empty() {
            return vec![ClientPlanSnapshotCallPacket {
                group_id,
                plans: None,
            }];
        }

        plans
            .chunks(CLIENT_PLAN_PREVIEW_CHUNK_SIZE)
            .map(|chunk| ClientPlanSnapshotCallPacket {
                group_id,
                plans: Some(chunk.to_vec()),
            })
            .collect()
    }

    pub fn send_client_plan_snapshots(
        &self,
        player: &mut PlayerComp,
        plans: &[BuildPlan],
    ) -> io::Result<usize> {
        self.send_client_plan_snapshots_with_limit(player, plans, MAX_PLAYER_PREVIEW_PLANS)
    }

    pub fn send_client_plan_snapshots_with_limit(
        &self,
        player: &mut PlayerComp,
        plans: &[BuildPlan],
        max_preview_plans: usize,
    ) -> io::Result<usize> {
        let packets =
            Self::client_plan_snapshot_packets_with_limit(player, plans, max_preview_plans);
        let mut sent = 0;
        let mut first_error = None;

        for packet in packets {
            match self.send_client_plan_snapshot(packet) {
                Ok(()) => sent += 1,
                Err(error) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }

        if let Some(error) = first_error {
            Err(error)
        } else {
            Ok(sent)
        }
    }

    pub fn apply_set_position_packet(
        player: &mut PlayerComp,
        unit: Option<&mut UnitComp>,
        packet: &SetPositionCallPacket,
    ) -> bool {
        if player.dead() {
            return false;
        }

        if let Some(unit) = unit {
            unit.set_pos(packet.x, packet.y);
        }
        player.x = packet.x;
        player.y = packet.y;
        true
    }

    pub fn apply_set_camera_position_packet(
        camera: &mut ClientCameraView,
        packet: &SetCameraPositionCallPacket,
    ) {
        camera.x = packet.x;
        camera.y = packet.y;
    }

    fn tile_mut_by_packed_pos(tiles: &mut Tiles, pos: i32) -> Option<&mut Tile> {
        tiles.get_mut(
            crate::mindustry::world::point2_x(pos) as i32,
            crate::mindustry::world::point2_y(pos) as i32,
        )
    }

    fn build_pos_by_packed_pos(tiles: &Tiles, pos: i32) -> Option<i32> {
        tiles
            .get(
                crate::mindustry::world::point2_x(pos) as i32,
                crate::mindustry::world::point2_y(pos) as i32,
            )
            .and_then(|tile| tile.build.map(|build| build.tile_pos))
    }

    fn apply_team_to_build_refs(tiles: &mut Tiles, build_pos: i32, team: i32) -> usize {
        let mut updated = 0;
        for tile in tiles.iter_mut() {
            if let Some(mut build) = tile.build {
                if build.tile_pos == build_pos {
                    build.team = team;
                    tile.build = Some(build);
                    updated += 1;
                }
            }
        }
        updated
    }

    fn clear_tile_block(tile: &mut Tile) {
        tile.block = Tile::AIR;
        tile.build = None;
        tile.changing = false;
    }

    fn set_tile_block(tile: &mut Tile, pos: i32, block: BlockId, team: i32, rotation: i32) {
        tile.block = block;
        tile.build = Some(WorldBuildingRef {
            tile_pos: pos,
            block,
            team,
            rotation,
        });
        tile.changing = false;
    }

    pub fn apply_remove_tile_packet(
        tiles: &mut Tiles,
        packet: &RemoveTileCallPacket,
    ) -> Result<(), String> {
        let pos = packet
            .tile
            .ok_or_else(|| "removeTile missing tile".to_string())?;
        let tile = Self::tile_mut_by_packed_pos(tiles, pos)
            .ok_or_else(|| format!("removeTile tile out of bounds: {pos}"))?;
        Self::clear_tile_block(tile);
        Ok(())
    }

    pub fn apply_build_destroyed_packet(
        tiles: &mut Tiles,
        packet: &BuildDestroyedCallPacket,
    ) -> bool {
        let Some(pos) = packet.build.tile_pos else {
            return false;
        };
        let Some(tile) = Self::tile_mut_by_packed_pos(tiles, pos) else {
            return false;
        };
        Self::clear_tile_block(tile);
        true
    }

    pub fn apply_set_tile_packet<F>(
        tiles: &mut Tiles,
        packet: &SetTileCallPacket,
        mut block_id_by_name: F,
    ) -> Result<(), String>
    where
        F: FnMut(&str) -> Option<BlockId>,
    {
        let pos = packet
            .tile
            .ok_or_else(|| "setTile missing tile".to_string())?;
        let tile = Self::tile_mut_by_packed_pos(tiles, pos)
            .ok_or_else(|| format!("setTile tile out of bounds: {pos}"))?;

        match packet.block.as_deref() {
            Some(block_name) => {
                let block_id = block_id_by_name(block_name)
                    .ok_or_else(|| format!("unknown block: {block_name}"))?;
                Self::set_tile_block(
                    tile,
                    pos,
                    block_id,
                    i32::from(packet.team.0),
                    packet.rotation,
                );
            }
            None => {
                Self::clear_tile_block(tile);
            }
        }
        Ok(())
    }

    pub fn apply_set_tile_blocks_packet<F>(
        tiles: &mut Tiles,
        packet: &SetTileBlocksCallPacket,
        mut block_id_by_name: F,
    ) -> Result<usize, String>
    where
        F: FnMut(&str) -> Option<BlockId>,
    {
        let Some(block_name) = packet.block.as_deref() else {
            return Ok(0);
        };
        let block_id =
            block_id_by_name(block_name).ok_or_else(|| format!("unknown block: {block_name}"))?;
        let mut applied = 0;

        for &pos in &packet.positions {
            if let Some(tile) = Self::tile_mut_by_packed_pos(tiles, pos) {
                Self::set_tile_block(tile, pos, block_id, i32::from(packet.team.0), 0);
                applied += 1;
            }
        }

        Ok(applied)
    }

    pub fn apply_construct_finish_packet<F>(
        tiles: &mut Tiles,
        packet: &ConstructFinishCallPacket,
        mut block_by_name: F,
    ) -> Result<bool, String>
    where
        F: FnMut(&str) -> Option<(BlockId, ClientTileBlockKind)>,
    {
        let Some(pos) = packet.tile else {
            return Ok(false);
        };
        let Some(block_name) = packet.block.as_deref() else {
            return Ok(false);
        };
        let (block_id, kind) =
            block_by_name(block_name).ok_or_else(|| format!("unknown block: {block_name}"))?;
        let Some(tile) = Self::tile_mut_by_packed_pos(tiles, pos) else {
            return Ok(false);
        };

        match kind {
            ClientTileBlockKind::Block => Self::set_tile_block(
                tile,
                pos,
                block_id,
                i32::from(packet.team.0),
                i32::from(packet.rotation),
            ),
            ClientTileBlockKind::Floor => {
                tile.floor = block_id;
                tile.changing = false;
            }
            ClientTileBlockKind::Overlay => {
                tile.overlay = block_id;
                tile.changing = false;
            }
        }

        Ok(true)
    }

    pub fn apply_deconstruct_finish_packet(
        tiles: &mut Tiles,
        packet: &DeconstructFinishCallPacket,
    ) -> bool {
        let Some(pos) = packet.tile else {
            return false;
        };
        let Some(tile) = Self::tile_mut_by_packed_pos(tiles, pos) else {
            return false;
        };
        Self::clear_tile_block(tile);
        true
    }

    pub fn apply_set_tile_items_packet(
        tiles: &Tiles,
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &SetTileItemsCallPacket,
    ) -> usize {
        let Some(item) = packet.item.as_deref() else {
            return 0;
        };
        let mut applied = 0;

        for &pos in &packet.positions {
            if let Some(build_pos) = Self::build_pos_by_packed_pos(tiles, pos) {
                storage
                    .entry(build_pos)
                    .or_default()
                    .items
                    .insert(item.to_string(), packet.amount);
                applied += 1;
            }
        }

        applied
    }

    pub fn apply_set_tile_liquids_packet(
        tiles: &Tiles,
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &SetTileLiquidsCallPacket,
    ) -> usize {
        let Some(liquid) = packet.liquid.as_deref() else {
            return 0;
        };
        let mut applied = 0;

        for &pos in &packet.positions {
            if let Some(build_pos) = Self::build_pos_by_packed_pos(tiles, pos) {
                storage
                    .entry(build_pos)
                    .or_default()
                    .liquids
                    .insert(liquid.to_string(), packet.amount);
                applied += 1;
            }
        }

        applied
    }

    pub fn apply_set_item_packet(
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &SetItemCallPacket,
    ) -> bool {
        let (Some(build_pos), Some(item)) = (packet.build.tile_pos, packet.item.as_deref()) else {
            return false;
        };
        storage
            .entry(build_pos)
            .or_default()
            .items
            .insert(item.to_string(), packet.amount);
        true
    }

    pub fn apply_set_items_packet(
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &SetItemsCallPacket,
    ) -> usize {
        let Some(build_pos) = packet.build.tile_pos else {
            return 0;
        };
        let mirror = storage.entry(build_pos).or_default();
        for stack in &packet.items {
            mirror.items.insert(stack.item.clone(), stack.amount);
        }
        packet.items.len()
    }

    pub fn apply_clear_items_packet(
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &ClearItemsCallPacket,
    ) -> bool {
        let Some(build_pos) = packet.build.tile_pos else {
            return false;
        };
        storage.entry(build_pos).or_default().items.clear();
        true
    }

    pub fn apply_set_liquid_packet(
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &SetLiquidCallPacket,
    ) -> bool {
        let (Some(build_pos), Some(liquid)) = (packet.build.tile_pos, packet.liquid.as_deref())
        else {
            return false;
        };
        storage
            .entry(build_pos)
            .or_default()
            .liquids
            .insert(liquid.to_string(), packet.amount);
        true
    }

    pub fn apply_set_liquids_packet(
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &SetLiquidsCallPacket,
    ) -> usize {
        let Some(build_pos) = packet.build.tile_pos else {
            return 0;
        };
        let mirror = storage.entry(build_pos).or_default();
        for stack in &packet.liquids {
            mirror.liquids.insert(stack.liquid.clone(), stack.amount);
        }
        packet.liquids.len()
    }

    pub fn apply_clear_liquids_packet(
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &ClearLiquidsCallPacket,
    ) -> bool {
        let Some(build_pos) = packet.build.tile_pos else {
            return false;
        };
        storage.entry(build_pos).or_default().liquids.clear();
        true
    }

    pub fn apply_set_team_packet(
        tiles: &mut Tiles,
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &SetTeamCallPacket,
    ) -> bool {
        let Some(build_pos) = packet.build.tile_pos else {
            return false;
        };
        let team = i32::from(packet.team.0);
        if Self::apply_team_to_build_refs(tiles, build_pos, team) == 0 {
            return false;
        }
        storage.entry(build_pos).or_default().team = Some(team);
        true
    }

    pub fn apply_set_team_mirror_packet(
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &SetTeamCallPacket,
    ) -> bool {
        let Some(build_pos) = packet.build.tile_pos else {
            return false;
        };
        storage.entry(build_pos).or_default().team = Some(i32::from(packet.team.0));
        true
    }

    pub fn apply_set_teams_packet(
        tiles: &mut Tiles,
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &SetTeamsCallPacket,
    ) -> usize {
        let team = i32::from(packet.team.0);
        let mut seen = BTreeSet::new();
        let mut applied = 0;

        for &pos in &packet.positions {
            let Some(build_pos) = Self::build_pos_by_packed_pos(tiles, pos) else {
                continue;
            };
            if !seen.insert(build_pos) {
                continue;
            }
            if Self::apply_team_to_build_refs(tiles, build_pos, team) > 0 {
                storage.entry(build_pos).or_default().team = Some(team);
                applied += 1;
            }
        }

        applied
    }

    pub fn apply_build_health_update_packet(
        tiles: &Tiles,
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &BuildHealthUpdateCallPacket,
    ) -> usize {
        let mut applied = 0;
        for pair in packet.buildings.chunks_exact(2) {
            let pos = pair[0];
            let Some(build_pos) = Self::build_pos_by_packed_pos(tiles, pos) else {
                continue;
            };
            let health = f32::from_bits(pair[1] as u32);
            storage.entry(build_pos).or_default().health = Some(health);
            applied += 1;
        }
        applied
    }

    pub fn apply_build_health_update_mirror_packet(
        storage: &mut BTreeMap<i32, ClientTileStorageMirror>,
        packet: &BuildHealthUpdateCallPacket,
    ) -> usize {
        let mut applied = 0;
        for pair in packet.buildings.chunks_exact(2) {
            let pos = pair[0];
            let health = f32::from_bits(pair[1] as u32);
            storage.entry(pos).or_default().health = Some(health);
            applied += 1;
        }
        applied
    }

    pub fn apply_create_marker_packet(state: &mut NetClientState, packet: &CreateMarkerCallPacket) {
        state.removed_marker_ids.remove(&packet.id);
        state
            .marker_mirrors
            .entry(packet.id)
            .or_default()
            .marker_json = Some(packet.marker_json.clone());
    }

    pub fn apply_update_marker_packet(state: &mut NetClientState, packet: &UpdateMarkerCallPacket) {
        if packet.control == LMarkerControl::Remove {
            state.marker_mirrors.remove(&packet.id);
            state.removed_marker_ids.insert(packet.id);
            return;
        }
        state
            .marker_mirrors
            .entry(packet.id)
            .or_default()
            .controls
            .insert(packet.control, (packet.p1, packet.p2, packet.p3));
    }

    pub fn apply_update_marker_text_packet(
        state: &mut NetClientState,
        packet: &UpdateMarkerTextCallPacket,
    ) {
        state
            .marker_mirrors
            .entry(packet.id)
            .or_default()
            .text_controls
            .insert(
                packet.r#type,
                ClientMarkerTextMirror {
                    fetch: packet.fetch,
                    text: packet.text.clone(),
                },
            );
    }

    pub fn apply_update_marker_texture_packet(
        state: &mut NetClientState,
        packet: &UpdateMarkerTextureCallPacket,
    ) {
        state.marker_mirrors.entry(packet.id).or_default().texture = Some(packet.texture.clone());
    }

    pub fn apply_remove_marker_packet(state: &mut NetClientState, packet: &RemoveMarkerCallPacket) {
        state.marker_mirrors.remove(&packet.id);
        state.removed_marker_ids.insert(packet.id);
    }

    pub fn apply_remove_world_label_packet(
        state: &mut NetClientState,
        packet: &RemoveWorldLabelCallPacket,
    ) {
        state.removed_world_label_ids.insert(packet.id);
    }

    pub fn apply_set_flag_packet(state: &mut NetClientState, packet: &SetFlagCallPacket) {
        if packet.add {
            state.logic_flags.insert(packet.flag.clone());
        } else {
            state.logic_flags.remove(&packet.flag);
        }
    }

    pub fn apply_set_map_area_packet(state: &mut NetClientState, packet: &SetMapAreaCallPacket) {
        state.map_area_mirror = Some(ClientMapAreaMirror::from(packet));
    }

    pub fn apply_set_player_team_editor_packet(
        state: &mut NetClientState,
        packet: &SetPlayerTeamEditorCallPacket,
    ) {
        let mirror = ClientPlayerTeamEditorMirror::from(packet);
        state
            .player_team_editor_mirrors
            .insert(packet.player.id, mirror);
    }

    pub fn apply_create_weather_packet(
        state: &mut NetClientState,
        packet: &CreateWeatherCallPacket,
    ) {
        state
            .weather_mirrors
            .push(ClientWeatherMirror::from(packet));
    }

    pub fn apply_logic_explosion_packet(
        state: &mut NetClientState,
        packet: &LogicExplosionCallPacket,
    ) {
        state
            .logic_explosion_mirrors
            .push(ClientLogicExplosionMirror::from(packet));
    }

    pub fn apply_create_bullet_packet(state: &mut NetClientState, packet: &CreateBulletCallPacket) {
        state.bullet_mirrors.push(ClientBulletMirror::from(packet));
    }

    pub fn apply_landing_pad_landed_packet(
        state: &mut NetClientState,
        packet: &LandingPadLandedCallPacket,
    ) {
        state.landed_pad_tiles.push(packet.tile);
    }

    pub fn apply_player_spawn_packet(state: &mut NetClientState, packet: &PlayerSpawnCallPacket) {
        state
            .player_spawn_mirrors
            .push(ClientPlayerSpawnMirror::from(packet));
    }

    pub fn apply_sync_variable_packet(state: &mut NetClientState, packet: &SyncVariableCallPacket) {
        state.logic_variable_mirrors.insert(
            (packet.building.tile_pos, packet.variable),
            packet.value.clone(),
        );
    }

    pub fn apply_set_floor_packet<F>(
        tiles: &mut Tiles,
        packet: &SetFloorCallPacket,
        mut block_id_by_name: F,
    ) -> Result<(), String>
    where
        F: FnMut(&str) -> Option<BlockId>,
    {
        let pos = packet
            .tile
            .ok_or_else(|| "setFloor missing tile".to_string())?;
        let tile = Self::tile_mut_by_packed_pos(tiles, pos)
            .ok_or_else(|| format!("setFloor tile out of bounds: {pos}"))?;

        if let Some(floor_name) = packet.floor.as_deref() {
            tile.floor = block_id_by_name(floor_name)
                .ok_or_else(|| format!("unknown floor: {floor_name}"))?;
        }
        tile.overlay = match packet.overlay.as_deref() {
            Some(overlay_name) => block_id_by_name(overlay_name)
                .ok_or_else(|| format!("unknown overlay: {overlay_name}"))?,
            None => Tile::AIR,
        };
        Ok(())
    }

    pub fn apply_set_tile_floors_packet<F>(
        tiles: &mut Tiles,
        packet: &SetTileFloorsCallPacket,
        mut block_id_by_name: F,
    ) -> Result<usize, String>
    where
        F: FnMut(&str) -> Option<BlockId>,
    {
        let Some(floor_name) = packet.block.as_deref() else {
            return Ok(0);
        };
        let floor_id =
            block_id_by_name(floor_name).ok_or_else(|| format!("unknown floor: {floor_name}"))?;
        let mut applied = 0;

        for &pos in &packet.positions {
            if let Some(tile) = Self::tile_mut_by_packed_pos(tiles, pos) {
                tile.floor = floor_id;
                applied += 1;
            }
        }

        Ok(applied)
    }

    pub fn apply_set_overlay_packet<F>(
        tiles: &mut Tiles,
        packet: &SetOverlayCallPacket,
        mut block_id_by_name: F,
    ) -> Result<(), String>
    where
        F: FnMut(&str) -> Option<BlockId>,
    {
        let pos = packet
            .tile
            .ok_or_else(|| "setOverlay missing tile".to_string())?;
        let tile = Self::tile_mut_by_packed_pos(tiles, pos)
            .ok_or_else(|| format!("setOverlay tile out of bounds: {pos}"))?;
        tile.overlay = match packet.overlay.as_deref() {
            Some(overlay_name) => block_id_by_name(overlay_name)
                .ok_or_else(|| format!("unknown overlay: {overlay_name}"))?,
            None => Tile::AIR,
        };
        Ok(())
    }

    pub fn apply_set_tile_overlays_packet<F>(
        tiles: &mut Tiles,
        packet: &SetTileOverlaysCallPacket,
        mut block_id_by_name: F,
    ) -> Result<usize, String>
    where
        F: FnMut(&str) -> Option<BlockId>,
    {
        let Some(overlay_name) = packet.block.as_deref() else {
            return Ok(0);
        };
        let overlay_id = block_id_by_name(overlay_name)
            .ok_or_else(|| format!("unknown overlay: {overlay_name}"))?;
        let mut applied = 0;

        for &pos in &packet.positions {
            if let Some(tile) = Self::tile_mut_by_packed_pos(tiles, pos) {
                tile.overlay = overlay_id;
                applied += 1;
            }
        }

        Ok(applied)
    }

    pub fn apply_received_preview_plans_to_player(
        player: &mut PlayerComp,
        packet: &ClientPlanSnapshotReceivedCallPacket,
        now_millis: i64,
        max_preview_plans: usize,
    ) -> io::Result<usize> {
        let plans = packet
            .plans
            .as_ref()
            .map(|plans| {
                plans
                    .iter()
                    .map(BuildPlanWire::to_build_plan)
                    .collect::<io::Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();
        let count_base = if packet.group_id > player.last_preview_plan_group {
            0
        } else {
            player.preview_plans_assembling.len()
        };
        player.handle_preview_plans(packet.group_id, &plans, now_millis, max_preview_plans);
        Ok(player
            .preview_plans_assembling
            .len()
            .saturating_sub(count_base))
    }

    pub fn update(&self) {
        let cursor = {
            let state = self.state.lock().unwrap();
            state.handled_client_cursor
        };

        let (provider_events, handled_packets, current_stream) = {
            let mut net = self.net.lock().unwrap();
            let provider_events = net.drain_provider_events();
            let handled_packets = net.handled_client_packets().to_vec();
            let current_stream = net.current_stream().cloned();
            (provider_events, handled_packets, current_stream)
        };

        let start = cursor.min(handled_packets.len());
        let new_packets = handled_packets[start..].to_vec();

        let (quiet, packet_handlers, binary_handlers) = {
            let mut state = self.state.lock().unwrap();
            state.handled_client_cursor = handled_packets.len();
            state.update_count += 1;
            state.total_provider_event_count += provider_events.len() as u64;
            state.last_provider_event_count = provider_events.len();
            state.last_provider_events = provider_events;
            state.last_update_at = Some(Instant::now());
            if !state.connect_confirm_sent {
                if let Some(stream) = current_stream.as_ref() {
                    state.record_loading_stream(stream);
                } else {
                    state.clear_loading_stream_tracking();
                }
            } else {
                state.clear_loading_stream_tracking();
                state.clear_timeout_clock();
            }
            (
                state.quiet,
                state.packet_handlers.clone(),
                state.binary_packet_handlers.clone(),
            )
        };

        for packet in new_packets {
            let disconnect_due_to_remote_control = matches!(
                &packet,
                PacketKind::KickCallPacket(_)
                    | PacketKind::KickCallPacket2(_)
                    | PacketKind::ConnectCallPacket(_)
            );

            let connect_packet_to_send = {
                let mut state = self.state.lock().unwrap();
                if let PacketKind::Connect(_) = &packet {
                    if !state.connect_packet_sent {
                        let connect_packet = state
                            .connect_config
                            .as_ref()
                            .map(ClientConnectConfig::to_connect_packet);
                        if let Some(connect_packet) = connect_packet {
                            state.connect_packet_sent = true;
                            state.last_sent_connect_packet = Some(connect_packet.clone());
                            state.last_connect_packet_error = None;
                            Some(connect_packet)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(connect_packet) = connect_packet_to_send {
                let result = {
                    let mut net = self.net.lock().unwrap();
                    net.send(&PacketKind::ConnectPacket(connect_packet), true)
                };
                if let Err(error) = result {
                    self.state.lock().unwrap().last_connect_packet_error = Some(error.to_string());
                }
            }

            let (connect_confirm_to_send, mark_client_unloaded) = {
                let mut state = self.state.lock().unwrap();
                state.last_packet = Some(packet.clone());
                match &packet {
                    PacketKind::Streamable(stream) => {
                        let world_data_ready = state.last_loaded_world_data.is_some()
                            && state.last_world_data_error.is_none();
                        state.connecting = false;
                        state.connected = world_data_ready;
                        state.world_data_loading = !world_data_ready;
                        state.last_binary_stream = Some(stream.stream.clone());
                        if state.auto_confirm_world_stream
                            && !state.connect_confirm_sent
                            && world_data_ready
                        {
                            state.connect_confirm_sent = true;
                            state.last_connect_confirm_error = None;
                            (true, false)
                        } else {
                            (false, false)
                        }
                    }
                    PacketKind::WorldDataBeginCallPacket(_) => {
                        state.record_world_data_begin();
                        (false, true)
                    }
                    PacketKind::SendMessageCallPacket(packet) => {
                        state.message_packets_seen += 1;
                        state.last_message = Some(packet.message.clone());
                        state.last_message_unformatted = None;
                        state.last_message_sender = None;
                        (false, false)
                    }
                    PacketKind::SendMessageCallPacket2(packet) => {
                        state.message_packets_seen += 1;
                        state.last_message = Some(packet.message.clone());
                        state.last_message_unformatted = Some(packet.unformatted.clone());
                        state.last_message_sender = Some(packet.player_sender);
                        (false, false)
                    }
                    PacketKind::AnnounceCallPacket(ui_packet) => {
                        state.record_client_ui_packet(&packet);
                        state.last_announcement = Some(ui_packet.message.clone());
                        (false, false)
                    }
                    PacketKind::InfoMessageCallPacket(ui_packet) => {
                        state.record_client_ui_packet(&packet);
                        state.last_info_message = Some(ui_packet.message.clone());
                        (false, false)
                    }
                    PacketKind::InfoToastCallPacket(ui_packet) => {
                        state.record_client_ui_packet(&packet);
                        state.last_toast_message = Some(ui_packet.message.clone());
                        (false, false)
                    }
                    PacketKind::WarningToastCallPacket(ui_packet) => {
                        state.record_client_ui_packet(&packet);
                        state.last_toast_message = Some(ui_packet.text.clone());
                        (false, false)
                    }
                    PacketKind::SetHudTextCallPacket(ui_packet) => {
                        state.record_client_ui_packet(&packet);
                        state.last_hud_text = Some(ui_packet.message.clone());
                        (false, false)
                    }
                    PacketKind::SetHudTextReliableCallPacket(ui_packet) => {
                        state.record_client_ui_packet(&packet);
                        state.last_hud_text = Some(ui_packet.0.message.clone());
                        (false, false)
                    }
                    PacketKind::HideHudTextCallPacket(_) => {
                        state.record_client_ui_packet(&packet);
                        state.last_hud_text = None;
                        (false, false)
                    }
                    PacketKind::OpenUriCallPacket(ui_packet) => {
                        state.record_client_ui_packet(&packet);
                        state.last_open_uri = Some(ui_packet.uri.clone());
                        (false, false)
                    }
                    PacketKind::CopyToClipboardCallPacket(ui_packet) => {
                        state.record_client_ui_packet(&packet);
                        state.last_clipboard_text = Some(ui_packet.text.clone());
                        (false, false)
                    }
                    PacketKind::InfoPopupCallPacket(_)
                    | PacketKind::InfoPopupCallPacket2(_)
                    | PacketKind::InfoPopupReliableCallPacket(_)
                    | PacketKind::InfoPopupReliableCallPacket2(_)
                    | PacketKind::MenuCallPacket(_)
                    | PacketKind::FollowUpMenuCallPacket(_)
                    | PacketKind::HideFollowUpMenuCallPacket(_)
                    | PacketKind::TextInputCallPacket(_)
                    | PacketKind::TextInputCallPacket2(_)
                    | PacketKind::LabelCallPacket(_)
                    | PacketKind::LabelCallPacket2(_)
                    | PacketKind::LabelReliableCallPacket(_)
                    | PacketKind::LabelReliableCallPacket2(_) => {
                        state.record_client_ui_packet(&packet);
                        (false, false)
                    }
                    PacketKind::DebugStatusClientCallPacket(debug) => {
                        let now = Instant::now();
                        state.debug_status_packets_seen += 1;
                        state.last_debug_status = Some(*debug);
                        state.last_debug_status_at = Some(now);
                        (false, false)
                    }
                    PacketKind::DebugStatusClientUnreliableCallPacket(debug) => {
                        let now = Instant::now();
                        state.debug_status_unreliable_packets_seen += 1;
                        state.last_debug_status_unreliable = Some(*debug);
                        state.last_debug_status_unreliable_at = Some(now);
                        (false, false)
                    }
                    PacketKind::MenuChooseCallPacket(menu) => {
                        let now = Instant::now();
                        state.menu_choose_packets_seen += 1;
                        state.last_menu_choose = Some(*menu);
                        state.last_menu_choose_at = Some(now);
                        (false, false)
                    }
                    PacketKind::TextInputResultCallPacket(result) => {
                        let now = Instant::now();
                        state.text_input_result_packets_seen += 1;
                        state.last_text_input_result = Some(result.clone());
                        state.last_text_input_result_at = Some(now);
                        (false, false)
                    }
                    PacketKind::PlayerDisconnectCallPacket(packet) => {
                        let now = Instant::now();
                        state.player_disconnect_packets_seen += 1;
                        state.removed_entity_ids.insert(packet.player_id);
                        state.last_player_disconnect = Some(*packet);
                        state.last_player_disconnect_at = Some(now);
                        (false, false)
                    }
                    PacketKind::TraceInfoCallPacket(packet) => {
                        let now = Instant::now();
                        state.trace_info_packets_seen += 1;
                        state.last_trace_info = Some(packet.clone());
                        state.last_trace_info_at = Some(now);
                        (false, false)
                    }
                    PacketKind::ConnectCallPacket(packet) => {
                        let now = Instant::now();
                        state.connect_call_packets_seen += 1;
                        state.last_connect_call = Some(packet.clone());
                        state.last_connect_call_at = Some(now);
                        state.record_server_redirect_disconnect();
                        (false, false)
                    }
                    PacketKind::SoundCallPacket(packet) => {
                        let now = Instant::now();
                        state.sound_packets_seen += 1;
                        state.last_sound = Some(packet.clone());
                        state.last_sound_at = Some(now);
                        (false, false)
                    }
                    PacketKind::SoundAtCallPacket(packet) => {
                        let now = Instant::now();
                        state.sound_at_packets_seen += 1;
                        state.last_sound_at_packet = Some(packet.clone());
                        state.last_sound_at_packet_at = Some(now);
                        (false, false)
                    }
                    PacketKind::EffectCallPacket(packet) => {
                        let now = Instant::now();
                        state.effect_packets_seen += 1;
                        state.last_effect = Some(*packet);
                        state.last_effect_at = Some(now);
                        (false, false)
                    }
                    PacketKind::EffectCallPacket2(packet) => {
                        let now = Instant::now();
                        state.effect_with_data_packets_seen += 1;
                        state.last_effect_with_data = Some(packet.clone());
                        state.last_effect_with_data_at = Some(now);
                        (false, false)
                    }
                    PacketKind::EffectReliableCallPacket(packet) => {
                        let now = Instant::now();
                        state.reliable_effect_packets_seen += 1;
                        state.last_reliable_effect = Some(*packet);
                        state.last_reliable_effect_at = Some(now);
                        (false, false)
                    }
                    PacketKind::SetCameraPositionCallPacket(packet) => {
                        let now = Instant::now();
                        state.camera_position_packets_seen += 1;
                        state.last_camera_position = Some(*packet);
                        state.last_camera_position_at = Some(now);
                        (false, false)
                    }
                    PacketKind::SetPositionCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_position_packets_seen += 1;
                        state.last_set_position = Some(*packet);
                        state.last_set_position_at = Some(now);
                        (false, false)
                    }
                    PacketKind::KickCallPacket(packet) => {
                        let now = Instant::now();
                        state.kick_packets_seen += 1;
                        state.last_kick = Some(packet.clone());
                        state.last_kick_at = Some(now);
                        state.record_server_kick_disconnect();
                        (false, false)
                    }
                    PacketKind::KickCallPacket2(packet) => {
                        let now = Instant::now();
                        state.kick_packets_seen += 1;
                        state.last_kick_reason = Some(*packet);
                        state.last_kick_reason_at = Some(now);
                        state.record_server_kick_disconnect();
                        (false, false)
                    }
                    PacketKind::PingResponseCallPacket(response) => {
                        let now = Self::current_millis();
                        state.ping_responses_received += 1;
                        state.last_ping_response_time = Some(response.time);
                        state.last_ping_response_at = Some(Instant::now());
                        state.ping_ms = now.saturating_sub(response.time).max(0) as u32;
                        (false, false)
                    }
                    PacketKind::EntitySnapshotCallPacket(snapshot) => {
                        let now = Instant::now();
                        state.entity_snapshot_packets_seen += 1;
                        state.last_entity_snapshot = Some(snapshot.clone());
                        state.last_entity_snapshot_at = Some(now);
                        state.last_server_snapshot_at = Some(now);
                        (false, false)
                    }
                    PacketKind::HiddenSnapshotCallPacket(snapshot) => {
                        let now = Instant::now();
                        state.hidden_snapshot_packets_seen += 1;
                        state.last_hidden_snapshot = Some(snapshot.clone());
                        state.last_hidden_snapshot_at = Some(now);
                        state.last_server_snapshot_at = Some(now);
                        (false, false)
                    }
                    PacketKind::StateSnapshotCallPacket(snapshot) => {
                        let now = Instant::now();
                        state.state_snapshot_packets_seen += 1;
                        state.last_state_snapshot = Some(snapshot.clone());
                        state.last_state_snapshot_at = Some(now);
                        state.last_state_snapshot_mirror =
                            Some(ClientGameStateMirror::from(snapshot));
                        state.last_server_snapshot_at = Some(now);
                        (false, false)
                    }
                    PacketKind::BlockSnapshotCallPacket(snapshot) => {
                        let now = Instant::now();
                        state.block_snapshot_packets_seen += 1;
                        state.last_block_snapshot = Some(snapshot.clone());
                        state.last_block_snapshot_at = Some(now);
                        state.last_server_snapshot_at = Some(now);
                        (false, false)
                    }
                    PacketKind::ConstructFinishCallPacket(_)
                    | PacketKind::DeconstructFinishCallPacket(_)
                    | PacketKind::DestroyPayloadCallPacket(_)
                    | PacketKind::AutoDoorToggleCallPacket(_)
                    | PacketKind::BeginBreakCallPacket(_)
                    | PacketKind::BeginPlaceCallPacket(_)
                    | PacketKind::BuildDestroyedCallPacket(_)
                    | PacketKind::BuildHealthUpdateCallPacket(_)
                    | PacketKind::RemoveTileCallPacket(_)
                    | PacketKind::SetFlagCallPacket(_)
                    | PacketKind::SetFloorCallPacket(_)
                    | PacketKind::SetMapAreaCallPacket(_)
                    | PacketKind::SetOverlayCallPacket(_)
                    | PacketKind::SetPlayerTeamEditorCallPacket(_)
                    | PacketKind::SetTeamCallPacket(_)
                    | PacketKind::SetTeamsCallPacket(_)
                    | PacketKind::SetTileCallPacket(_)
                    | PacketKind::SetTileBlocksCallPacket(_)
                    | PacketKind::SetTileFloorsCallPacket(_)
                    | PacketKind::SetTileItemsCallPacket(_)
                    | PacketKind::SetTileLiquidsCallPacket(_)
                    | PacketKind::SetTileOverlaysCallPacket(_)
                    | PacketKind::SpawnEffectCallPacket(_)
                    | PacketKind::LogicExplosionCallPacket(_)
                    | PacketKind::CreateBulletCallPacket(_)
                    | PacketKind::CreateWeatherCallPacket(_)
                    | PacketKind::LandingPadLandedCallPacket(_)
                    | PacketKind::PlayerSpawnCallPacket(_)
                    | PacketKind::SyncVariableCallPacket(_) => {
                        match &packet {
                            PacketKind::SetFlagCallPacket(packet) => {
                                Self::apply_set_flag_packet(&mut state, packet);
                            }
                            PacketKind::SetMapAreaCallPacket(packet) => {
                                Self::apply_set_map_area_packet(&mut state, packet);
                            }
                            PacketKind::SetPlayerTeamEditorCallPacket(packet) => {
                                Self::apply_set_player_team_editor_packet(&mut state, packet);
                            }
                            PacketKind::SetTeamCallPacket(packet) => {
                                Self::apply_set_team_mirror_packet(
                                    &mut state.building_storage_mirrors,
                                    packet,
                                );
                            }
                            PacketKind::BuildHealthUpdateCallPacket(packet) => {
                                Self::apply_build_health_update_mirror_packet(
                                    &mut state.building_storage_mirrors,
                                    packet,
                                );
                            }
                            PacketKind::LogicExplosionCallPacket(packet) => {
                                Self::apply_logic_explosion_packet(&mut state, packet);
                            }
                            PacketKind::CreateBulletCallPacket(packet) => {
                                Self::apply_create_bullet_packet(&mut state, packet);
                            }
                            PacketKind::CreateWeatherCallPacket(packet) => {
                                Self::apply_create_weather_packet(&mut state, packet);
                            }
                            PacketKind::LandingPadLandedCallPacket(packet) => {
                                Self::apply_landing_pad_landed_packet(&mut state, packet);
                            }
                            PacketKind::PlayerSpawnCallPacket(packet) => {
                                Self::apply_player_spawn_packet(&mut state, packet);
                            }
                            PacketKind::SyncVariableCallPacket(packet) => {
                                Self::apply_sync_variable_packet(&mut state, packet);
                            }
                            _ => {}
                        }
                        state.record_world_update_packet(&packet);
                        (false, false)
                    }
                    PacketKind::UnitBlockSpawnCallPacket(_)
                    | PacketKind::UnitCapDeathCallPacket(_)
                    | PacketKind::UnitDeathCallPacket(_)
                    | PacketKind::UnitDespawnCallPacket(_)
                    | PacketKind::UnitDestroyCallPacket(_)
                    | PacketKind::UnitEnvDeathCallPacket(_)
                    | PacketKind::UnitSafeDeathCallPacket(_)
                    | PacketKind::UnitSpawnCallPacket(_)
                    | PacketKind::UnitTetherBlockSpawnedCallPacket(_)
                    | PacketKind::AssemblerDroneSpawnedCallPacket(_)
                    | PacketKind::AssemblerUnitSpawnedCallPacket(_) => {
                        state.record_unit_lifecycle_packet(&packet);
                        (false, false)
                    }
                    PacketKind::CreateMarkerCallPacket(_)
                    | PacketKind::UpdateMarkerCallPacket(_)
                    | PacketKind::UpdateMarkerTextCallPacket(_)
                    | PacketKind::UpdateMarkerTextureCallPacket(_)
                    | PacketKind::RemoveMarkerCallPacket(_)
                    | PacketKind::RemoveWorldLabelCallPacket(_) => {
                        match &packet {
                            PacketKind::CreateMarkerCallPacket(packet) => {
                                Self::apply_create_marker_packet(&mut state, packet);
                            }
                            PacketKind::UpdateMarkerCallPacket(packet) => {
                                Self::apply_update_marker_packet(&mut state, packet);
                            }
                            PacketKind::UpdateMarkerTextCallPacket(packet) => {
                                Self::apply_update_marker_text_packet(&mut state, packet);
                            }
                            PacketKind::UpdateMarkerTextureCallPacket(packet) => {
                                Self::apply_update_marker_texture_packet(&mut state, packet);
                            }
                            PacketKind::RemoveMarkerCallPacket(packet) => {
                                Self::apply_remove_marker_packet(&mut state, packet);
                            }
                            PacketKind::RemoveWorldLabelCallPacket(packet) => {
                                Self::apply_remove_world_label_packet(&mut state, packet);
                            }
                            _ => {}
                        }
                        state.record_marker_packet(&packet);
                        (false, false)
                    }
                    PacketKind::GameOverCallPacket(_)
                    | PacketKind::UpdateGameOverCallPacket(_)
                    | PacketKind::ResearchedCallPacket(_)
                    | PacketKind::SectorCaptureCallPacket(_) => {
                        state.record_campaign_event_packet(&packet);
                        (false, false)
                    }
                    PacketKind::SetRulesCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_rules_packets_seen += 1;
                        state.last_rules_json = Some(packet.rules_json.clone());
                        state.rule_json_patches.clear();
                        state.last_set_rules = Some(packet.clone());
                        state.last_set_rules_at = Some(now);
                        (false, false)
                    }
                    PacketKind::SetRuleCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_rule_packets_seen += 1;
                        state
                            .rule_json_patches
                            .insert(packet.rule.clone(), packet.json_data.clone());
                        state.last_set_rule = Some(packet.clone());
                        state.last_set_rule_at = Some(now);
                        (false, false)
                    }
                    PacketKind::SetObjectivesCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_objectives_packets_seen += 1;
                        state.last_objectives_json = Some(packet.objectives_json.clone());
                        state.completed_objective_indices.clear();
                        state.last_set_objectives = Some(packet.clone());
                        state.last_set_objectives_at = Some(now);
                        (false, false)
                    }
                    PacketKind::ClearObjectivesCallPacket(_) => {
                        let now = Instant::now();
                        state.clear_objectives_packets_seen += 1;
                        state.last_objectives_json = None;
                        state.completed_objective_indices.clear();
                        state.last_clear_objectives_at = Some(now);
                        (false, false)
                    }
                    PacketKind::CompleteObjectiveCallPacket(packet) => {
                        let now = Instant::now();
                        state.complete_objective_packets_seen += 1;
                        state.completed_objective_indices.push(packet.index);
                        state.last_complete_objective = Some(*packet);
                        state.last_complete_objective_at = Some(now);
                        (false, false)
                    }
                    PacketKind::ClientPlanSnapshotReceivedCallPacket(snapshot) => {
                        let now = Instant::now();
                        state.client_plan_snapshot_received_packets_seen += 1;
                        state.last_client_plan_snapshot_received = Some(snapshot.clone());
                        state.last_client_plan_snapshot_received_at = Some(now);
                        (false, false)
                    }
                    PacketKind::SetItemCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_item_packets_seen += 1;
                        Self::apply_set_item_packet(&mut state.building_storage_mirrors, packet);
                        state.last_set_item = Some(packet.clone());
                        state.last_set_item_at = Some(now);
                        (false, false)
                    }
                    PacketKind::SetItemsCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_items_packets_seen += 1;
                        Self::apply_set_items_packet(&mut state.building_storage_mirrors, packet);
                        state.last_set_items = Some(packet.clone());
                        state.last_set_items_at = Some(now);
                        (false, false)
                    }
                    PacketKind::ClearItemsCallPacket(packet) => {
                        let now = Instant::now();
                        state.clear_items_packets_seen += 1;
                        Self::apply_clear_items_packet(&mut state.building_storage_mirrors, packet);
                        state.last_clear_items = Some(*packet);
                        state.last_clear_items_at = Some(now);
                        (false, false)
                    }
                    PacketKind::SetLiquidCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_liquid_packets_seen += 1;
                        Self::apply_set_liquid_packet(&mut state.building_storage_mirrors, packet);
                        state.last_set_liquid = Some(packet.clone());
                        state.last_set_liquid_at = Some(now);
                        (false, false)
                    }
                    PacketKind::SetLiquidsCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_liquids_packets_seen += 1;
                        Self::apply_set_liquids_packet(&mut state.building_storage_mirrors, packet);
                        state.last_set_liquids = Some(packet.clone());
                        state.last_set_liquids_at = Some(now);
                        (false, false)
                    }
                    PacketKind::ClearLiquidsCallPacket(packet) => {
                        let now = Instant::now();
                        state.clear_liquids_packets_seen += 1;
                        Self::apply_clear_liquids_packet(
                            &mut state.building_storage_mirrors,
                            packet,
                        );
                        state.last_clear_liquids = Some(*packet);
                        state.last_clear_liquids_at = Some(now);
                        (false, false)
                    }
                    PacketKind::TakeItemsCallPacket(packet) => {
                        let now = Instant::now();
                        state.take_items_packets_seen += 1;
                        state.last_take_items = Some(packet.clone());
                        state.last_take_items_at = Some(now);
                        (false, false)
                    }
                    PacketKind::TransferItemEffectCallPacket(packet) => {
                        let now = Instant::now();
                        state.transfer_item_effect_packets_seen += 1;
                        state.last_transfer_item_effect = Some(packet.clone());
                        state.last_transfer_item_effect_at = Some(now);
                        (false, false)
                    }
                    PacketKind::TransferItemToCallPacket(packet) => {
                        let now = Instant::now();
                        state.transfer_item_to_packets_seen += 1;
                        state.last_transfer_item_to = Some(packet.clone());
                        state.last_transfer_item_to_at = Some(now);
                        (false, false)
                    }
                    PacketKind::TransferItemToUnitCallPacket(packet) => {
                        let now = Instant::now();
                        state.transfer_item_to_unit_packets_seen += 1;
                        state.last_transfer_item_to_unit = Some(packet.clone());
                        state.last_transfer_item_to_unit_at = Some(now);
                        (false, false)
                    }
                    PacketKind::RequestItemCallPacket(packet) => {
                        let now = Instant::now();
                        state.request_item_packets_seen += 1;
                        state.last_request_item = Some(packet.clone());
                        state.last_request_item_at = Some(now);
                        (false, false)
                    }
                    PacketKind::TransferInventoryCallPacket(packet) => {
                        let now = Instant::now();
                        state.transfer_inventory_packets_seen += 1;
                        state.last_transfer_inventory = Some(packet.clone());
                        state.last_transfer_inventory_at = Some(now);
                        (false, false)
                    }
                    PacketKind::RequestBuildPayloadCallPacket(packet) => {
                        let now = Instant::now();
                        state.request_build_payload_packets_seen += 1;
                        state.last_request_build_payload = Some(packet.clone());
                        state.last_request_build_payload_at = Some(now);
                        (false, false)
                    }
                    PacketKind::RequestUnitPayloadCallPacket(packet) => {
                        let now = Instant::now();
                        state.request_unit_payload_packets_seen += 1;
                        state.last_request_unit_payload = Some(packet.clone());
                        state.last_request_unit_payload_at = Some(now);
                        (false, false)
                    }
                    PacketKind::PickedBuildPayloadCallPacket(packet) => {
                        let now = Instant::now();
                        state.picked_build_payload_packets_seen += 1;
                        state.last_picked_build_payload = Some(packet.clone());
                        state.last_picked_build_payload_at = Some(now);
                        (false, false)
                    }
                    PacketKind::PickedUnitPayloadCallPacket(packet) => {
                        let now = Instant::now();
                        state.picked_unit_payload_packets_seen += 1;
                        state.last_picked_unit_payload = Some(packet.clone());
                        state.last_picked_unit_payload_at = Some(now);
                        (false, false)
                    }
                    PacketKind::RequestDropPayloadCallPacket(packet) => {
                        let now = Instant::now();
                        state.request_drop_payload_packets_seen += 1;
                        state.last_request_drop_payload = Some(packet.clone());
                        state.last_request_drop_payload_at = Some(now);
                        (false, false)
                    }
                    PacketKind::PayloadDroppedCallPacket(packet) => {
                        let now = Instant::now();
                        state.payload_dropped_packets_seen += 1;
                        state.last_payload_dropped = Some(packet.clone());
                        state.last_payload_dropped_at = Some(now);
                        (false, false)
                    }
                    PacketKind::UnitEnteredPayloadCallPacket(packet) => {
                        let now = Instant::now();
                        state.unit_entered_payload_packets_seen += 1;
                        state.last_unit_entered_payload = Some(packet.clone());
                        state.last_unit_entered_payload_at = Some(now);
                        (false, false)
                    }
                    PacketKind::PingLocationCallPacket(packet) => {
                        let now = Instant::now();
                        state.ping_location_packets_seen += 1;
                        state.last_ping_location = Some(packet.clone());
                        state.last_ping_location_at = Some(now);
                        (false, false)
                    }
                    PacketKind::DeletePlansCallPacket(packet) => {
                        let now = Instant::now();
                        state.delete_plans_packets_seen += 1;
                        state.last_delete_plans = Some(packet.clone());
                        state.last_delete_plans_at = Some(now);
                        (false, false)
                    }
                    PacketKind::CommandBuildingCallPacket(packet) => {
                        let now = Instant::now();
                        state.command_building_packets_seen += 1;
                        state.last_command_building = Some(packet.clone());
                        state.last_command_building_at = Some(now);
                        (false, false)
                    }
                    PacketKind::CommandUnitsCallPacket(packet) => {
                        let now = Instant::now();
                        state.command_units_packets_seen += 1;
                        state.last_command_units = Some(packet.clone());
                        state.last_command_units_at = Some(now);
                        (false, false)
                    }
                    PacketKind::SetUnitCommandCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_unit_command_packets_seen += 1;
                        state.last_set_unit_command = Some(packet.clone());
                        state.last_set_unit_command_at = Some(now);
                        (false, false)
                    }
                    PacketKind::SetUnitStanceCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_unit_stance_packets_seen += 1;
                        state.last_set_unit_stance = Some(packet.clone());
                        state.last_set_unit_stance_at = Some(now);
                        (false, false)
                    }
                    PacketKind::BuildingControlSelectCallPacket(packet) => {
                        let now = Instant::now();
                        state.building_control_select_packets_seen += 1;
                        state.last_building_control_select = Some(packet.clone());
                        state.last_building_control_select_at = Some(now);
                        (false, false)
                    }
                    PacketKind::UnitBuildingControlSelectCallPacket(packet) => {
                        let now = Instant::now();
                        state.unit_building_control_select_packets_seen += 1;
                        state.last_unit_building_control_select = Some(packet.clone());
                        state.last_unit_building_control_select_at = Some(now);
                        (false, false)
                    }
                    PacketKind::UnitControlCallPacket(packet) => {
                        let now = Instant::now();
                        state.unit_control_packets_seen += 1;
                        state.last_unit_control = Some(packet.clone());
                        state.last_unit_control_at = Some(now);
                        (false, false)
                    }
                    PacketKind::UnitClearCallPacket(packet) => {
                        let now = Instant::now();
                        state.unit_clear_packets_seen += 1;
                        state.last_unit_clear = Some(packet.clone());
                        state.last_unit_clear_at = Some(now);
                        (false, false)
                    }
                    PacketKind::RemoveQueueBlockCallPacket(packet) => {
                        let now = Instant::now();
                        state.remove_queue_block_packets_seen += 1;
                        state.last_remove_queue_block = Some(*packet);
                        state.last_remove_queue_block_at = Some(now);
                        (false, false)
                    }
                    PacketKind::TileConfigCallPacket(packet) => {
                        let now = Instant::now();
                        state.tile_config_packets_seen += 1;
                        state.last_tile_config = Some(packet.clone());
                        state.last_tile_config_at = Some(now);
                        (false, false)
                    }
                    PacketKind::RotateBlockCallPacket(packet) => {
                        let now = Instant::now();
                        state.rotate_block_packets_seen += 1;
                        state.last_rotate_block = Some(packet.clone());
                        state.last_rotate_block_at = Some(now);
                        (false, false)
                    }
                    PacketKind::TileTapCallPacket(packet) => {
                        let now = Instant::now();
                        state.tile_tap_packets_seen += 1;
                        state.last_tile_tap = Some(packet.clone());
                        state.last_tile_tap_at = Some(now);
                        (false, false)
                    }
                    _ => (false, false),
                }
            };

            if mark_client_unloaded {
                self.net.lock().unwrap().set_client_loaded(false);
            }

            if disconnect_due_to_remote_control {
                self.net.lock().unwrap().disconnect();
            }

            if quiet {
                if connect_confirm_to_send {
                    self.finish_connecting_and_send_confirm();
                }
                continue;
            }

            let binary_payload = match &packet {
                PacketKind::Streamable(stream) => Some(stream.stream.clone()),
                _ => None,
            };

            for handler in &packet_handlers {
                handler(packet.clone());
            }

            if let Some(binary_payload) = binary_payload {
                for handler in &binary_handlers {
                    handler(&binary_payload);
                }
            }

            if connect_confirm_to_send {
                self.finish_connecting_and_send_confirm();
            }
        }

        self.maybe_disconnect_due_to_timeout();
        self.maybe_send_ping();
    }

    fn finish_connecting_and_send_confirm(&self) {
        let result = {
            let mut net = self.net.lock().unwrap();
            net.set_client_loaded(true);
            net.send(
                &PacketKind::ConnectConfirmCallPacket(ConnectConfirmCallPacket),
                true,
            )
        };
        if let Err(error) = result {
            self.state.lock().unwrap().last_connect_confirm_error = Some(error.to_string());
        }
    }

    fn maybe_disconnect_due_to_timeout(&self) {
        let timed_out = {
            let state = self.state.lock().unwrap();
            if state.connect_confirm_sent {
                false
            } else {
                state
                    .timeout_deadline
                    .is_some_and(|deadline| Instant::now() >= deadline)
            }
        };

        if !timed_out {
            return;
        }

        {
            let mut net = self.net.lock().unwrap();
            net.disconnect();
        }

        let mut state = self.state.lock().unwrap();
        state.connecting = false;
        state.connected = false;
        state.world_data_loading = false;
        state.timeout_disconnects += 1;
        state.last_update_at = Some(Instant::now());
        state.connect_packet_sent = false;
        state.connect_confirm_sent = false;
        state.last_connect_confirm_error = None;
        state.removed_entity_ids.clear();
        state.reset_ping_state();
        state.clear_loading_stream_tracking();
        state.clear_timeout_clock();
    }

    fn maybe_send_ping(&self) {
        let ping_time = {
            let mut state = self.state.lock().unwrap();
            if !state.connected || !state.connect_confirm_sent {
                return;
            }

            let now = Instant::now();
            match state.next_ping_at {
                Some(deadline) if now < deadline => return,
                _ => {}
            }

            let time = Self::current_millis();
            state.ping_requests_sent += 1;
            state.last_ping_request_time = Some(time);
            state.last_ping_request_at = Some(now);
            state.last_ping_request_error = None;
            state.next_ping_at = Some(now + PING_INTERVAL);
            Some(time)
        };

        if let Some(time) = ping_time {
            let result = {
                let mut net = self.net.lock().unwrap();
                net.send(&PacketKind::PingCallPacket(PingCallPacket { time }), true)
            };

            if let Err(error) = result {
                self.state.lock().unwrap().last_ping_request_error = Some(error.to_string());
            }
        }
    }

    fn install_client_listeners(net: &mut Net, state: &Arc<Mutex<NetClientState>>) {
        {
            let state = Arc::clone(state);
            net.handle_client_connect(move |connect| {
                let mut state = state.lock().unwrap();
                state.record_connect(connect);
            });
        }

        {
            let state = Arc::clone(state);
            net.handle_client_disconnect(move |disconnect| {
                let mut state = state.lock().unwrap();
                state.record_disconnect(disconnect);
            });
        }

        {
            let state = Arc::clone(state);
            net.handle_client_world_stream(move |stream| {
                let mut state = state.lock().unwrap();
                state.record_world_stream(stream);
            });
        }
    }

    fn current_millis() -> i64 {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        millis.min(i64::MAX as u128) as i64
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use crate::mindustry::entities::comp::{PlayerComp, PlayerUnitState, UnitComp};
    use crate::mindustry::entities::units::BuildPlan;
    use crate::mindustry::io::type_io::RgbaColor;
    use crate::mindustry::io::UnitRef;
    use crate::mindustry::io::{BuildPlanWire, BuildingRef, EntityRef, TeamId, TypeValue, Vec2};
    use crate::mindustry::logic::LMarkerControl;
    use crate::mindustry::net::{
        write_minimal_world_data, AnnounceCallPacket, BlockSnapshotCallPacket,
        BuildDestroyedCallPacket, BuildHealthUpdateCallPacket, BuildingControlSelectCallPacket,
        ClearItemsCallPacket, ClearLiquidsCallPacket, ClearObjectivesCallPacket,
        ClientPlanSnapshotCallPacket, ClientPlanSnapshotReceivedCallPacket,
        ClientSnapshotCallPacket, CommandBuildingCallPacket, CommandUnitsCallPacket,
        CompleteObjectiveCallPacket, Connect, ConnectCallPacket, ConstructFinishCallPacket,
        CopyToClipboardCallPacket, CreateBulletCallPacket, CreateMarkerCallPacket,
        CreateWeatherCallPacket, DebugStatusClientCallPacket,
        DebugStatusClientUnreliableCallPacket, DeconstructFinishCallPacket, DeletePlansCallPacket,
        Disconnect, DoneCallback, EffectCallPacket, EffectCallPacket2, EffectReliableCallPacket,
        EntitySnapshotCallPacket, GameOverCallPacket, HiddenSnapshotCallPacket,
        HideHudTextCallPacket, Host, HostCallback, InfoMessageCallPacket, InfoToastCallPacket,
        KickCallPacket, KickCallPacket2, KickReason, LandingPadLandedCallPacket,
        LogicExplosionCallPacket, MenuCallPacket, MenuChooseCallPacket, Net, NetConnection,
        NetProvider, OpenUriCallPacket, PacketKind, PayloadDroppedCallPacket,
        PickedBuildPayloadCallPacket, PickedUnitPayloadCallPacket, PingLocationCallPacket,
        PingResponseCallPacket, PlayerDisconnectCallPacket, PlayerSpawnCallPacket,
        RemoveMarkerCallPacket, RemoveQueueBlockCallPacket, RemoveTileCallPacket,
        RemoveWorldLabelCallPacket, RequestBuildPayloadCallPacket, RequestDropPayloadCallPacket,
        RequestItemCallPacket, RequestUnitPayloadCallPacket, RotateBlockCallPacket,
        SendMessageCallPacket, SendMessageCallPacket2, SetCameraPositionCallPacket,
        SetFlagCallPacket, SetFloorCallPacket, SetHudTextCallPacket, SetItemCallPacket,
        SetItemsCallPacket, SetLiquidCallPacket, SetLiquidsCallPacket, SetMapAreaCallPacket,
        SetObjectivesCallPacket, SetOverlayCallPacket, SetPlayerTeamEditorCallPacket,
        SetPositionCallPacket, SetRuleCallPacket, SetRulesCallPacket, SetTeamCallPacket,
        SetTeamsCallPacket, SetTileBlocksCallPacket, SetTileCallPacket, SetTileFloorsCallPacket,
        SetTileItemsCallPacket, SetTileLiquidsCallPacket, SetTileOverlaysCallPacket,
        SetUnitCommandCallPacket, SetUnitStanceCallPacket, SoundAtCallPacket, SoundCallPacket,
        StateSnapshotCallPacket, StreamBegin, StreamChunk, Streamable, SyncVariableCallPacket,
        TakeItemsCallPacket, TextInputResultCallPacket, TileConfigCallPacket, TileTapCallPacket,
        TraceInfoCallPacket, TransferInventoryCallPacket, TransferItemEffectCallPacket,
        TransferItemToCallPacket, TransferItemToUnitCallPacket,
        UnitBuildingControlSelectCallPacket, UnitClearCallPacket, UnitControlCallPacket,
        UnitDeathCallPacket, UnitEnteredPayloadCallPacket, UpdateMarkerCallPacket,
        UpdateMarkerTextCallPacket, UpdateMarkerTextureCallPacket, WarningToastCallPacket,
        WorldDataBeginCallPacket,
    };
    use crate::mindustry::r#type::{ItemStack, LiquidStack, UnitType};
    use crate::mindustry::world::block::Block;
    use crate::mindustry::world::{point2_pack, Tile, Tiles};

    use super::{
        ClientBulletMirror, ClientCameraView, ClientConnectConfig, ClientInputSnapshot,
        ClientLogicExplosionMirror, ClientMapAreaMirror, ClientMarkerTextMirror,
        ClientPlayerSpawnMirror, ClientPlayerTeamEditorMirror, ClientTileBlockKind,
        ClientTileStorageMirror, ClientWeatherMirror, NetClient, CLIENT_PLAN_PREVIEW_CHUNK_SIZE,
    };

    #[derive(Clone, Default)]
    struct CaptureProvider {
        sent: Arc<Mutex<Vec<(PacketKind, bool)>>>,
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

        fn send_client(&mut self, object: &PacketKind, reliable: bool) -> io::Result<()> {
            self.sent.lock().unwrap().push((object.clone(), reliable));
            Ok(())
        }

        fn disconnect_client(&mut self) {}

        fn discover_servers(&self, _callback: HostCallback, done: DoneCallback) {
            done();
        }

        fn ping_host(&self, _address: &str, _port: u16, _timeout: Duration) -> io::Result<Host> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "not implemented",
            ))
        }

        fn host_server(&mut self, _port: u16) -> io::Result<()> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "not implemented",
            ))
        }

        fn get_connections(&self) -> Vec<NetConnection> {
            Vec::new()
        }

        fn close_server(&mut self) {}
    }

    fn unit_type() -> UnitType {
        let mut unit = UnitType::new(1, "alpha");
        unit.aim_dst = 12.0;
        unit.build_speed = 1.0;
        unit
    }

    #[test]
    fn core_typed_listeners_update_client_state() {
        let client = NetClient::default();
        client.begin_connecting();
        assert!(client.is_connecting());

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::Connect(Connect {
                address_tcp: "127.0.0.1:6567".into(),
            }));
            net.handle_client_received(PacketKind::Streamable(Streamable::new(vec![1, 2, 3])));
            net.handle_client_received(PacketKind::Disconnect(Disconnect {
                reason: "closed".into(),
            }));
        }

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.connect_events, 1);
        assert_eq!(state.world_stream_events, 1);
        assert_eq!(state.disconnect_events, 1);
        assert_eq!(
            state.last_connect.as_ref().unwrap().address_tcp,
            "127.0.0.1:6567"
        );
        assert_eq!(
            state.last_world_stream.as_ref().unwrap().stream,
            vec![1, 2, 3]
        );
        assert_eq!(state.last_disconnect.as_ref().unwrap().reason, "closed");
        assert!(!state.connected);
        assert!(!state.connecting);
    }

    #[test]
    fn update_dispatches_new_packets_and_stream_bytes_once() {
        let client = NetClient::default();
        let packet_count = Arc::new(Mutex::new(0));
        let binary_payloads = Arc::new(Mutex::new(Vec::new()));

        let packet_count_handler = Arc::clone(&packet_count);
        client.add_packet_handler(move |packet| {
            if matches!(packet, PacketKind::Streamable(_)) {
                *packet_count_handler.lock().unwrap() += 1;
            }
        });

        let binary_payload_handler = Arc::clone(&binary_payloads);
        client.add_binary_packet_handler(move |bytes| {
            binary_payload_handler.lock().unwrap().push(bytes.to_vec());
        });

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::Streamable(Streamable::new(vec![7, 8, 9])));
        }

        client.update();
        client.update();

        assert_eq!(*packet_count.lock().unwrap(), 1);
        assert_eq!(*binary_payloads.lock().unwrap(), vec![vec![7, 8, 9]]);
    }

    #[test]
    fn update_records_server_messages_like_java_client_chatfrag_inputs() {
        let client = NetClient::default();

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::SendMessageCallPacket(SendMessageCallPacket {
                message: "[scarlet]Server warning".into(),
            }));
            net.handle_client_received(PacketKind::SendMessageCallPacket2(
                SendMessageCallPacket2 {
                    message: "[coral][[#ff00ffff]player[coral]]:[white] hello".into(),
                    unformatted: "hello".into(),
                    player_sender: EntityRef::new(77),
                },
            ));
        }

        client.update();
        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.message_packets_seen, 2);
        assert_eq!(
            state.last_message.as_deref(),
            Some("[coral][[#ff00ffff]player[coral]]:[white] hello")
        );
        assert_eq!(state.last_message_unformatted.as_deref(), Some("hello"));
        assert_eq!(state.last_message_sender, Some(EntityRef::new(77)));
    }

    #[test]
    fn update_records_client_ui_packets_for_later_frontend_application() {
        let client = NetClient::default();
        let announce = AnnounceCallPacket {
            message: "incoming wave".into(),
        };
        let info = InfoMessageCallPacket {
            message: "server info".into(),
        };
        let toast = InfoToastCallPacket {
            message: "toast".into(),
            duration: 2.5,
        };
        let warning = WarningToastCallPacket {
            unicode: 0xf071,
            text: "warning".into(),
        };
        let hud = SetHudTextCallPacket {
            message: "capture the core".into(),
        };
        let uri = OpenUriCallPacket {
            uri: "https://example.invalid".into(),
        };
        let clipboard = CopyToClipboardCallPacket {
            text: "copy me".into(),
        };
        let menu = MenuCallPacket {
            menu_id: 9,
            title: "title".into(),
            message: "choose".into(),
            options: vec![vec![Some("ok".into()), Some("cancel".into())]],
        };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::AnnounceCallPacket(announce.clone()));
            net.handle_client_received(PacketKind::InfoMessageCallPacket(info.clone()));
            net.handle_client_received(PacketKind::InfoToastCallPacket(toast.clone()));
            net.handle_client_received(PacketKind::WarningToastCallPacket(warning.clone()));
            net.handle_client_received(PacketKind::SetHudTextCallPacket(hud.clone()));
            net.handle_client_received(PacketKind::OpenUriCallPacket(uri.clone()));
            net.handle_client_received(PacketKind::CopyToClipboardCallPacket(clipboard.clone()));
            net.handle_client_received(PacketKind::MenuCallPacket(menu));
            net.handle_client_received(PacketKind::HideHudTextCallPacket(HideHudTextCallPacket));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.client_ui_packets_seen, 9);
        assert!(matches!(
            state.last_client_ui_packet.as_ref(),
            Some(PacketKind::HideHudTextCallPacket(_))
        ));
        assert!(state.last_client_ui_packet_at.is_some());
        assert_eq!(state.last_announcement.as_deref(), Some("incoming wave"));
        assert_eq!(state.last_info_message.as_deref(), Some("server info"));
        assert_eq!(state.last_toast_message.as_deref(), Some("warning"));
        assert!(state.last_hud_text.is_none());
        assert_eq!(
            state.last_open_uri.as_deref(),
            Some("https://example.invalid")
        );
        assert_eq!(state.last_clipboard_text.as_deref(), Some("copy me"));
    }

    #[test]
    fn update_records_debug_menu_and_text_input_result_packets() {
        let client = NetClient::default();
        let debug = DebugStatusClientCallPacket {
            value: 5,
            last_client_snapshot: 6,
            snapshots_sent: 7,
        };
        let debug_unreliable = DebugStatusClientUnreliableCallPacket(DebugStatusClientCallPacket {
            value: 8,
            last_client_snapshot: 9,
            snapshots_sent: 10,
        });
        let menu_choose = MenuChooseCallPacket {
            player_id: Some(44),
            menu_id: 3,
            option: 2,
        };
        let text_result = TextInputResultCallPacket {
            player: EntityRef::new(44),
            text_input_id: 11,
            text: "typed".into(),
        };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::DebugStatusClientCallPacket(debug));
            net.handle_client_received(PacketKind::DebugStatusClientUnreliableCallPacket(
                debug_unreliable,
            ));
            net.handle_client_received(PacketKind::MenuChooseCallPacket(menu_choose));
            net.handle_client_received(PacketKind::TextInputResultCallPacket(text_result.clone()));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.debug_status_packets_seen, 1);
        assert_eq!(state.last_debug_status, Some(debug));
        assert!(state.last_debug_status_at.is_some());
        assert_eq!(state.debug_status_unreliable_packets_seen, 1);
        assert_eq!(state.last_debug_status_unreliable, Some(debug_unreliable));
        assert!(state.last_debug_status_unreliable_at.is_some());
        assert_eq!(state.menu_choose_packets_seen, 1);
        assert_eq!(state.last_menu_choose, Some(menu_choose));
        assert!(state.last_menu_choose_at.is_some());
        assert_eq!(state.text_input_result_packets_seen, 1);
        assert_eq!(state.last_text_input_result.as_ref(), Some(&text_result));
        assert!(state.last_text_input_result_at.is_some());
    }

    #[test]
    fn update_records_marker_packets_and_updates_marker_mirrors() {
        let client = NetClient::default();
        let create = CreateMarkerCallPacket {
            id: 10,
            marker_json: r#"{"type":"shape"}"#.into(),
        };
        let update = UpdateMarkerCallPacket {
            id: 10,
            control: LMarkerControl::Pos,
            p1: 1.0,
            p2: 2.0,
            p3: 3.0,
        };
        let text = UpdateMarkerTextCallPacket {
            id: 10,
            r#type: LMarkerControl::FlushText,
            fetch: true,
            text: "目标".into(),
        };
        let texture = UpdateMarkerTextureCallPacket {
            id: 10,
            texture: TypeValue::String("router".into()),
        };
        let transient = CreateMarkerCallPacket {
            id: 11,
            marker_json: "{}".into(),
        };
        let remove = RemoveMarkerCallPacket { id: 11 };
        let remove_label = RemoveWorldLabelCallPacket { id: 12 };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::CreateMarkerCallPacket(create.clone()));
            net.handle_client_received(PacketKind::UpdateMarkerCallPacket(update.clone()));
            net.handle_client_received(PacketKind::UpdateMarkerTextCallPacket(text.clone()));
            net.handle_client_received(PacketKind::UpdateMarkerTextureCallPacket(texture.clone()));
            net.handle_client_received(PacketKind::CreateMarkerCallPacket(transient));
            net.handle_client_received(PacketKind::RemoveMarkerCallPacket(remove));
            net.handle_client_received(PacketKind::RemoveWorldLabelCallPacket(remove_label));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.marker_packets_seen, 7);
        assert!(matches!(
            state.last_marker_packet.as_ref(),
            Some(PacketKind::RemoveWorldLabelCallPacket(_))
        ));
        let marker = state.marker_mirrors.get(&10).unwrap();
        assert_eq!(
            marker.marker_json.as_deref(),
            Some(create.marker_json.as_str())
        );
        assert_eq!(
            marker.controls.get(&LMarkerControl::Pos),
            Some(&(update.p1, update.p2, update.p3))
        );
        assert_eq!(
            marker.text_controls.get(&LMarkerControl::FlushText),
            Some(&ClientMarkerTextMirror {
                fetch: text.fetch,
                text: text.text.clone(),
            })
        );
        assert_eq!(marker.texture.as_ref(), Some(&texture.texture));
        assert!(!state.marker_mirrors.contains_key(&11));
        assert!(state.removed_marker_ids.contains(&11));
        assert!(state.removed_world_label_ids.contains(&12));
    }

    #[test]
    fn update_records_logic_and_world_event_packets_for_later_application() {
        let client = NetClient::default();
        let add_flag = SetFlagCallPacket {
            flag: "open".into(),
            add: true,
        };
        let remove_flag = SetFlagCallPacket {
            flag: "legacy".into(),
            add: false,
        };
        let map_area = SetMapAreaCallPacket {
            x: 1,
            y: 2,
            width: 3,
            height: 4,
        };
        let team_editor = SetPlayerTeamEditorCallPacket {
            player: EntityRef::new(42),
            team: TeamId(6),
        };
        let weather = CreateWeatherCallPacket {
            weather_id: Some(3),
            intensity: 0.5,
            duration: 20.0,
            wind_x: 1.25,
            wind_y: -2.5,
        };
        let logic = LogicExplosionCallPacket {
            team: TeamId(2),
            x: 8.0,
            y: 9.0,
            radius: 10.0,
            damage: 11.0,
            air: true,
            ground: false,
            pierce: true,
            effect: false,
        };
        let bullet = CreateBulletCallPacket {
            bullet_type_id: 12,
            team: TeamId(4),
            x: 13.0,
            y: 14.0,
            angle: 15.0,
            damage: 16.0,
            velocity_scl: 1.5,
            lifetime_scl: 0.75,
        };
        let landing = LandingPadLandedCallPacket { tile: Some(77) };
        let spawn = PlayerSpawnCallPacket {
            tile: Some(88),
            player: EntityRef::new(99),
        };
        let sync = SyncVariableCallPacket {
            building: BuildingRef::new(1234),
            variable: 5,
            value: TypeValue::String("hello".into()),
        };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::SetFlagCallPacket(add_flag.clone()));
            net.handle_client_received(PacketKind::SetFlagCallPacket(remove_flag.clone()));
            net.handle_client_received(PacketKind::SetMapAreaCallPacket(map_area));
            net.handle_client_received(PacketKind::SetPlayerTeamEditorCallPacket(team_editor));
            net.handle_client_received(PacketKind::CreateWeatherCallPacket(weather));
            net.handle_client_received(PacketKind::LogicExplosionCallPacket(logic));
            net.handle_client_received(PacketKind::CreateBulletCallPacket(bullet));
            net.handle_client_received(PacketKind::LandingPadLandedCallPacket(landing));
            net.handle_client_received(PacketKind::PlayerSpawnCallPacket(spawn));
            net.handle_client_received(PacketKind::SyncVariableCallPacket(sync.clone()));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.world_update_packets_seen, 10);
        assert!(state.logic_flags.contains("open"));
        assert!(!state.logic_flags.contains("legacy"));
        assert_eq!(
            state.map_area_mirror,
            Some(ClientMapAreaMirror {
                x: 1,
                y: 2,
                width: 3,
                height: 4,
            })
        );
        assert_eq!(
            state.player_team_editor_mirrors.get(&Some(42)),
            Some(&ClientPlayerTeamEditorMirror {
                player: team_editor.player,
                team: team_editor.team,
            })
        );
        assert_eq!(
            state.weather_mirrors.last().copied(),
            Some(ClientWeatherMirror::from(&weather))
        );
        assert_eq!(
            state.logic_explosion_mirrors.last().copied(),
            Some(ClientLogicExplosionMirror::from(&logic))
        );
        assert_eq!(
            state.bullet_mirrors.last().copied(),
            Some(ClientBulletMirror::from(&bullet))
        );
        assert_eq!(state.landed_pad_tiles.last().copied(), Some(Some(77)));
        assert_eq!(
            state.player_spawn_mirrors.last().copied(),
            Some(ClientPlayerSpawnMirror::from(&spawn))
        );
        assert_eq!(
            state.logic_variable_mirrors.get(&(Some(1234), 5)),
            Some(&sync.value)
        );
        assert!(matches!(
            state.last_world_update_packet.as_ref(),
            Some(PacketKind::SyncVariableCallPacket(_))
        ));
    }

    #[test]
    fn update_records_world_unit_marker_and_campaign_packets_for_later_application() {
        let client = NetClient::default();
        let set_tile = SetTileCallPacket {
            tile: Some(12),
            block: Some("router".into()),
            team: TeamId(1),
            rotation: 2,
        };
        let health = 34.5f32;
        let build_health = BuildHealthUpdateCallPacket {
            buildings: vec![12, health.to_bits() as i32],
        };
        let set_team = SetTeamCallPacket {
            build: BuildingRef::new(12),
            team: TeamId(3),
        };
        let remove_tile = RemoveTileCallPacket { tile: Some(12) };
        let unit_death = UnitDeathCallPacket { uid: 77 };
        let create_marker = CreateMarkerCallPacket {
            id: 5,
            marker_json: "{}".into(),
        };
        let remove_marker = RemoveMarkerCallPacket { id: 5 };
        let game_over = GameOverCallPacket { winner: TeamId(2) };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::SetTileCallPacket(set_tile));
            net.handle_client_received(PacketKind::BuildHealthUpdateCallPacket(build_health));
            net.handle_client_received(PacketKind::SetTeamCallPacket(set_team));
            net.handle_client_received(PacketKind::RemoveTileCallPacket(remove_tile));
            net.handle_client_received(PacketKind::UnitDeathCallPacket(unit_death));
            net.handle_client_received(PacketKind::CreateMarkerCallPacket(create_marker));
            net.handle_client_received(PacketKind::RemoveMarkerCallPacket(remove_marker));
            net.handle_client_received(PacketKind::GameOverCallPacket(game_over));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.world_update_packets_seen, 4);
        assert!(matches!(
            state.last_world_update_packet.as_ref(),
            Some(PacketKind::RemoveTileCallPacket(_))
        ));
        assert!(state.last_world_update_packet_at.is_some());
        let mirror = state.building_storage_mirrors.get(&12).unwrap();
        assert_eq!(mirror.health, Some(health));
        assert_eq!(mirror.team, Some(3));
        assert_eq!(state.unit_lifecycle_packets_seen, 1);
        assert!(matches!(
            state.last_unit_lifecycle_packet.as_ref(),
            Some(PacketKind::UnitDeathCallPacket(_))
        ));
        assert!(state.last_unit_lifecycle_packet_at.is_some());
        assert_eq!(state.marker_packets_seen, 2);
        assert!(matches!(
            state.last_marker_packet.as_ref(),
            Some(PacketKind::RemoveMarkerCallPacket(_))
        ));
        assert!(state.last_marker_packet_at.is_some());
        assert_eq!(state.campaign_event_packets_seen, 1);
        assert!(matches!(
            state.last_campaign_event_packet.as_ref(),
            Some(PacketKind::GameOverCallPacket(_))
        ));
        assert!(state.last_campaign_event_packet_at.is_some());
    }

    #[test]
    fn update_records_player_disconnect_as_removed_entity_like_java_client() {
        let client = NetClient::default();

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::PlayerDisconnectCallPacket(
                PlayerDisconnectCallPacket { player_id: 41 },
            ));
            net.handle_client_received(PacketKind::PlayerDisconnectCallPacket(
                PlayerDisconnectCallPacket { player_id: 42 },
            ));
        }

        client.update();

        {
            let state = client.state();
            let state = state.lock().unwrap();
            assert_eq!(state.player_disconnect_packets_seen, 2);
            assert_eq!(
                state.last_player_disconnect,
                Some(PlayerDisconnectCallPacket { player_id: 42 })
            );
            assert!(state.last_player_disconnect_at.is_some());
            assert!(state.removed_entity_ids.contains(&41));
            assert!(state.removed_entity_ids.contains(&42));
        }

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::WorldDataBeginCallPacket(
                WorldDataBeginCallPacket,
            ));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert!(state.removed_entity_ids.is_empty());
        assert!(state.world_data_loading);
    }

    #[test]
    fn update_records_trace_info_packet_for_admin_ui_mirror() {
        let client = NetClient::default();
        let info = crate::mindustry::net::administration::TraceInfo::new(
            Some("127.0.0.1".into()),
            Some("uuid".into()),
            Some("en_US".into()),
            true,
            false,
            3,
            1,
            vec![Some("127.0.0.1".into())],
            vec![Some("player".into())],
        );
        let packet = TraceInfoCallPacket {
            player: EntityRef::new(7),
            info: info.clone(),
        };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::TraceInfoCallPacket(packet.clone()));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.trace_info_packets_seen, 1);
        assert_eq!(state.last_trace_info.as_ref(), Some(&packet));
        assert!(state.last_trace_info_at.is_some());
    }

    #[test]
    fn update_records_sound_and_effect_packets_for_client_mirrors() {
        let client = NetClient::default();
        let sound = SoundCallPacket {
            sound_id: 3,
            volume: 0.75,
            pitch: 1.25,
            pan: -0.5,
        };
        let sound_at = SoundAtCallPacket {
            sound_id: 4,
            x: 10.0,
            y: -20.0,
            volume: 0.5,
            pitch: 2.0,
        };
        let effect = EffectCallPacket {
            effect_id: 7,
            x: 1.0,
            y: 2.0,
            rotation: 90.0,
            color: RgbaColor::new(0x11223344),
        };
        let effect_with_data = EffectCallPacket2 {
            effect,
            data: TypeValue::String("payload".into()),
        };
        let reliable_effect = EffectReliableCallPacket(effect);

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::SoundCallPacket(sound.clone()));
            net.handle_client_received(PacketKind::SoundAtCallPacket(sound_at.clone()));
            net.handle_client_received(PacketKind::EffectCallPacket(effect));
            net.handle_client_received(PacketKind::EffectCallPacket2(effect_with_data.clone()));
            net.handle_client_received(PacketKind::EffectReliableCallPacket(reliable_effect));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.sound_packets_seen, 1);
        assert_eq!(state.last_sound.as_ref(), Some(&sound));
        assert!(state.last_sound_at.is_some());
        assert_eq!(state.sound_at_packets_seen, 1);
        assert_eq!(state.last_sound_at_packet.as_ref(), Some(&sound_at));
        assert!(state.last_sound_at_packet_at.is_some());
        assert_eq!(state.effect_packets_seen, 1);
        assert_eq!(state.last_effect.as_ref(), Some(&effect));
        assert!(state.last_effect_at.is_some());
        assert_eq!(state.effect_with_data_packets_seen, 1);
        assert_eq!(
            state.last_effect_with_data.as_ref(),
            Some(&effect_with_data)
        );
        assert!(state.last_effect_with_data_at.is_some());
        assert_eq!(state.reliable_effect_packets_seen, 1);
        assert_eq!(state.last_reliable_effect.as_ref(), Some(&reliable_effect));
        assert!(state.last_reliable_effect_at.is_some());
    }

    #[test]
    fn update_records_position_packets_and_applies_lightweight_helpers() {
        let client = NetClient::default();
        let camera_packet = SetCameraPositionCallPacket { x: 30.0, y: 40.0 };
        let position_packet = SetPositionCallPacket { x: -5.0, y: 9.0 };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::SetCameraPositionCallPacket(camera_packet));
            net.handle_client_received(PacketKind::SetPositionCallPacket(position_packet));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.camera_position_packets_seen, 1);
        assert_eq!(state.last_camera_position, Some(camera_packet));
        assert!(state.last_camera_position_at.is_some());
        assert_eq!(state.set_position_packets_seen, 1);
        assert_eq!(state.last_set_position, Some(position_packet));
        assert!(state.last_set_position_at.is_some());
        drop(state);

        let mut camera = ClientCameraView {
            width: 80.0,
            height: 45.0,
            ..ClientCameraView::default()
        };
        NetClient::apply_set_camera_position_packet(&mut camera, &camera_packet);
        assert_eq!(camera.x, camera_packet.x);
        assert_eq!(camera.y, camera_packet.y);
        assert_eq!(camera.width, 80.0);
        assert_eq!(camera.height, 45.0);

        let mut player = PlayerComp::default();
        player.set_unit_state(PlayerUnitState::unit(34).with_valid(true));
        let mut unit = UnitComp::new(34, unit_type(), TeamId(2));
        assert!(NetClient::apply_set_position_packet(
            &mut player,
            Some(&mut unit),
            &position_packet
        ));
        assert_eq!((player.x, player.y), (position_packet.x, position_packet.y));
        assert_eq!((unit.x(), unit.y()), (position_packet.x, position_packet.y));

        let mut dead_player = PlayerComp::default();
        let mut dead_unit = UnitComp::new(35, unit_type(), TeamId(2));
        assert!(!NetClient::apply_set_position_packet(
            &mut dead_player,
            Some(&mut dead_unit),
            &position_packet
        ));
        assert_eq!((dead_player.x, dead_player.y), (0.0, 0.0));
        assert_eq!((dead_unit.x(), dead_unit.y()), (0.0, 0.0));
    }

    #[test]
    fn apply_tile_topology_packets_updates_lightweight_tiles() {
        let mut tiles = Tiles::new(4, 4);
        let resolve = |name: &str| match name {
            "router" => Some(10),
            "stone" => Some(20),
            "ore-copper" => Some(30),
            _ => None,
        };
        let pos = point2_pack(1, 2);

        NetClient::apply_set_tile_packet(
            &mut tiles,
            &SetTileCallPacket {
                tile: Some(pos),
                block: Some("router".into()),
                team: TeamId(2),
                rotation: 1,
            },
            resolve,
        )
        .unwrap();

        let tile = tiles.get(1, 2).unwrap();
        assert_eq!(tile.block, 10);
        let build = tile.build.as_ref().unwrap();
        assert_eq!(build.tile_pos, pos);
        assert_eq!(build.block, 10);
        assert_eq!(build.team, 2);
        assert_eq!(build.rotation, 1);

        NetClient::apply_set_floor_packet(
            &mut tiles,
            &SetFloorCallPacket {
                tile: Some(pos),
                floor: Some("stone".into()),
                overlay: Some("ore-copper".into()),
            },
            resolve,
        )
        .unwrap();

        let tile = tiles.get(1, 2).unwrap();
        assert_eq!(tile.floor, 20);
        assert_eq!(tile.overlay, 30);
        assert_eq!(tile.block, 10);
        assert!(tile.build.is_some());

        NetClient::apply_set_overlay_packet(
            &mut tiles,
            &SetOverlayCallPacket {
                tile: Some(pos),
                overlay: None,
            },
            resolve,
        )
        .unwrap();
        assert_eq!(tiles.get(1, 2).unwrap().overlay, Tile::AIR);

        NetClient::apply_remove_tile_packet(&mut tiles, &RemoveTileCallPacket { tile: Some(pos) })
            .unwrap();
        let tile = tiles.get(1, 2).unwrap();
        assert_eq!(tile.block, Tile::AIR);
        assert!(tile.build.is_none());
        assert_eq!(tile.floor, 20);

        assert!(NetClient::apply_set_tile_packet(
            &mut tiles,
            &SetTileCallPacket {
                tile: Some(pos),
                block: Some("missing".into()),
                team: TeamId(1),
                rotation: 0,
            },
            resolve,
        )
        .unwrap_err()
        .contains("unknown block"));
        assert!(NetClient::apply_remove_tile_packet(
            &mut tiles,
            &RemoveTileCallPacket {
                tile: Some(point2_pack(9, 9)),
            },
        )
        .unwrap_err()
        .contains("out of bounds"));
        assert!(NetClient::apply_set_overlay_packet(
            &mut tiles,
            &SetOverlayCallPacket {
                tile: None,
                overlay: Some("ore-copper".into()),
            },
            resolve,
        )
        .unwrap_err()
        .contains("missing tile"));
    }

    #[test]
    fn apply_batch_tile_topology_packets_mirrors_java_remote_helpers() {
        let mut tiles = Tiles::new(4, 4);
        let resolve = |name: &str| match name {
            "router" => Some(10),
            "stone" => Some(20),
            "ore-copper" => Some(30),
            _ => None,
        };
        let a = point2_pack(0, 0);
        let b = point2_pack(3, 2);
        let out_of_bounds = point2_pack(9, 9);

        assert_eq!(
            NetClient::apply_set_tile_blocks_packet(
                &mut tiles,
                &SetTileBlocksCallPacket {
                    block: Some("router".into()),
                    team: TeamId(4),
                    positions: vec![a, out_of_bounds, b],
                },
                resolve,
            )
            .unwrap(),
            2
        );
        for (x, y, pos) in [(0, 0, a), (3, 2, b)] {
            let tile = tiles.get(x, y).unwrap();
            assert_eq!(tile.block, 10);
            let build = tile.build.as_ref().unwrap();
            assert_eq!(build.tile_pos, pos);
            assert_eq!(build.block, 10);
            assert_eq!(build.team, 4);
            assert_eq!(build.rotation, 0);
        }

        assert_eq!(
            NetClient::apply_set_tile_floors_packet(
                &mut tiles,
                &SetTileFloorsCallPacket {
                    block: Some("stone".into()),
                    positions: vec![a, b],
                },
                resolve,
            )
            .unwrap(),
            2
        );
        assert_eq!(tiles.get(0, 0).unwrap().floor, 20);
        assert_eq!(tiles.get(3, 2).unwrap().floor, 20);
        assert_eq!(tiles.get(0, 0).unwrap().overlay, Tile::AIR);

        assert_eq!(
            NetClient::apply_set_tile_overlays_packet(
                &mut tiles,
                &SetTileOverlaysCallPacket {
                    block: Some("ore-copper".into()),
                    positions: vec![a, b],
                },
                resolve,
            )
            .unwrap(),
            2
        );
        assert_eq!(tiles.get(0, 0).unwrap().overlay, 30);
        assert_eq!(tiles.get(3, 2).unwrap().overlay, 30);

        assert_eq!(
            NetClient::apply_set_tile_blocks_packet(
                &mut tiles,
                &SetTileBlocksCallPacket {
                    block: None,
                    team: TeamId(1),
                    positions: vec![a],
                },
                resolve,
            )
            .unwrap(),
            0
        );
        assert_eq!(tiles.get(0, 0).unwrap().block, 10);

        assert!(NetClient::apply_set_tile_floors_packet(
            &mut tiles,
            &SetTileFloorsCallPacket {
                block: Some("missing".into()),
                positions: vec![a],
            },
            resolve,
        )
        .unwrap_err()
        .contains("unknown floor"));
        assert_eq!(tiles.get(0, 0).unwrap().floor, 20);
    }

    #[test]
    fn apply_build_lifecycle_packets_updates_lightweight_tiles() {
        let mut tiles = Tiles::new(4, 4);
        let resolve = |name: &str| match name {
            "router" => Some((10, ClientTileBlockKind::Block)),
            "stone" => Some((20, ClientTileBlockKind::Floor)),
            "ore-copper" => Some((30, ClientTileBlockKind::Overlay)),
            _ => None,
        };
        let build_pos = point2_pack(1, 1);
        let floor_pos = point2_pack(2, 1);
        let overlay_pos = point2_pack(3, 1);

        assert!(NetClient::apply_construct_finish_packet(
            &mut tiles,
            &ConstructFinishCallPacket {
                tile: Some(build_pos),
                block: Some("router".into()),
                builder: UnitRef::Null,
                rotation: 2,
                team: TeamId(3),
                config: TypeValue::Null,
            },
            resolve,
        )
        .unwrap());
        let tile = tiles.get(1, 1).unwrap();
        assert_eq!(tile.block, 10);
        let build = tile.build.as_ref().unwrap();
        assert_eq!(build.tile_pos, build_pos);
        assert_eq!(build.block, 10);
        assert_eq!(build.team, 3);
        assert_eq!(build.rotation, 2);

        assert!(NetClient::apply_construct_finish_packet(
            &mut tiles,
            &ConstructFinishCallPacket {
                tile: Some(floor_pos),
                block: Some("stone".into()),
                builder: UnitRef::Null,
                rotation: 0,
                team: TeamId(0),
                config: TypeValue::Null,
            },
            resolve,
        )
        .unwrap());
        assert_eq!(tiles.get(2, 1).unwrap().floor, 20);
        assert_eq!(tiles.get(2, 1).unwrap().block, Tile::AIR);

        assert!(NetClient::apply_construct_finish_packet(
            &mut tiles,
            &ConstructFinishCallPacket {
                tile: Some(overlay_pos),
                block: Some("ore-copper".into()),
                builder: UnitRef::Null,
                rotation: 0,
                team: TeamId(0),
                config: TypeValue::Null,
            },
            resolve,
        )
        .unwrap());
        assert_eq!(tiles.get(3, 1).unwrap().overlay, 30);

        assert!(NetClient::apply_deconstruct_finish_packet(
            &mut tiles,
            &DeconstructFinishCallPacket {
                tile: Some(build_pos),
                block: Some("router".into()),
                builder: UnitRef::Null,
            },
        ));
        let tile = tiles.get(1, 1).unwrap();
        assert_eq!(tile.block, Tile::AIR);
        assert!(tile.build.is_none());

        NetClient::apply_set_tile_packet(
            &mut tiles,
            &SetTileCallPacket {
                tile: Some(build_pos),
                block: Some("router".into()),
                team: TeamId(3),
                rotation: 1,
            },
            |name| match name {
                "router" => Some(10),
                _ => None,
            },
        )
        .unwrap();
        assert!(NetClient::apply_build_destroyed_packet(
            &mut tiles,
            &BuildDestroyedCallPacket {
                build: BuildingRef::new(build_pos),
            },
        ));
        assert_eq!(tiles.get(1, 1).unwrap().block, Tile::AIR);
        assert!(tiles.get(1, 1).unwrap().build.is_none());

        assert!(!NetClient::apply_build_destroyed_packet(
            &mut tiles,
            &BuildDestroyedCallPacket {
                build: BuildingRef::null(),
            },
        ));
        assert!(!NetClient::apply_deconstruct_finish_packet(
            &mut tiles,
            &DeconstructFinishCallPacket {
                tile: Some(point2_pack(9, 9)),
                block: Some("router".into()),
                builder: UnitRef::Null,
            },
        ));
        assert!(NetClient::apply_construct_finish_packet(
            &mut tiles,
            &ConstructFinishCallPacket {
                tile: Some(build_pos),
                block: Some("missing".into()),
                builder: UnitRef::Null,
                rotation: 0,
                team: TeamId(0),
                config: TypeValue::Null,
            },
            resolve,
        )
        .unwrap_err()
        .contains("unknown block"));
    }

    #[test]
    fn apply_set_tile_items_and_liquids_packets_updates_storage_mirror() {
        let mut tiles = Tiles::new(4, 4);
        let center = point2_pack(1, 1);
        let proxy = point2_pack(2, 1);
        let empty = point2_pack(0, 0);
        let out_of_bounds = point2_pack(9, 9);

        NetClient::apply_set_tile_packet(
            &mut tiles,
            &SetTileCallPacket {
                tile: Some(center),
                block: Some("router".into()),
                team: TeamId(2),
                rotation: 0,
            },
            |name| match name {
                "router" => Some(10),
                _ => None,
            },
        )
        .unwrap();
        let build = tiles.get(1, 1).unwrap().build.unwrap();
        tiles.get_mut(2, 1).unwrap().build = Some(build);

        let mut storage = BTreeMap::<i32, ClientTileStorageMirror>::new();
        assert_eq!(
            NetClient::apply_set_tile_items_packet(
                &tiles,
                &mut storage,
                &SetTileItemsCallPacket {
                    item: Some("copper".into()),
                    amount: 12,
                    positions: vec![center, proxy, empty, out_of_bounds],
                },
            ),
            2
        );
        assert_eq!(storage.get(&center).unwrap().items.get("copper"), Some(&12));

        assert_eq!(
            NetClient::apply_set_tile_liquids_packet(
                &tiles,
                &mut storage,
                &SetTileLiquidsCallPacket {
                    liquid: Some("water".into()),
                    amount: 6.5,
                    positions: vec![proxy, empty],
                },
            ),
            1
        );
        assert_eq!(
            storage.get(&center).unwrap().liquids.get("water"),
            Some(&6.5)
        );

        assert_eq!(
            NetClient::apply_set_tile_items_packet(
                &tiles,
                &mut storage,
                &SetTileItemsCallPacket {
                    item: None,
                    amount: 99,
                    positions: vec![center],
                },
            ),
            0
        );
        assert_eq!(storage.get(&center).unwrap().items.get("copper"), Some(&12));
        assert_eq!(
            NetClient::apply_set_tile_liquids_packet(
                &tiles,
                &mut storage,
                &SetTileLiquidsCallPacket {
                    liquid: None,
                    amount: 99.0,
                    positions: vec![center],
                },
            ),
            0
        );
        assert_eq!(
            storage.get(&center).unwrap().liquids.get("water"),
            Some(&6.5)
        );
    }

    #[test]
    fn apply_building_item_and_liquid_packets_updates_storage_mirror() {
        let build = BuildingRef::new(point2_pack(1, 1));
        let mut storage = BTreeMap::<i32, ClientTileStorageMirror>::new();

        assert!(NetClient::apply_set_item_packet(
            &mut storage,
            &SetItemCallPacket {
                build,
                item: Some("copper".into()),
                amount: 7,
            },
        ));
        assert_eq!(
            storage
                .get(&build.tile_pos.unwrap())
                .unwrap()
                .items
                .get("copper"),
            Some(&7)
        );

        assert_eq!(
            NetClient::apply_set_items_packet(
                &mut storage,
                &SetItemsCallPacket {
                    build,
                    items: vec![ItemStack::new("lead", 3), ItemStack::new("scrap", 4)],
                },
            ),
            2
        );
        let mirror = storage.get(&build.tile_pos.unwrap()).unwrap();
        assert_eq!(mirror.items.get("copper"), Some(&7));
        assert_eq!(mirror.items.get("lead"), Some(&3));
        assert_eq!(mirror.items.get("scrap"), Some(&4));

        assert!(NetClient::apply_set_liquid_packet(
            &mut storage,
            &SetLiquidCallPacket {
                build,
                liquid: Some("water".into()),
                amount: 2.5,
            },
        ));
        assert_eq!(
            NetClient::apply_set_liquids_packet(
                &mut storage,
                &SetLiquidsCallPacket {
                    build,
                    liquids: vec![LiquidStack::new("slag", 5.0)],
                },
            ),
            1
        );
        let mirror = storage.get(&build.tile_pos.unwrap()).unwrap();
        assert_eq!(mirror.liquids.get("water"), Some(&2.5));
        assert_eq!(mirror.liquids.get("slag"), Some(&5.0));

        assert!(NetClient::apply_clear_items_packet(
            &mut storage,
            &ClearItemsCallPacket { build },
        ));
        assert!(storage
            .get(&build.tile_pos.unwrap())
            .unwrap()
            .items
            .is_empty());
        assert!(NetClient::apply_clear_liquids_packet(
            &mut storage,
            &ClearLiquidsCallPacket { build },
        ));
        assert!(storage
            .get(&build.tile_pos.unwrap())
            .unwrap()
            .liquids
            .is_empty());

        assert!(!NetClient::apply_set_item_packet(
            &mut storage,
            &SetItemCallPacket {
                build: BuildingRef::null(),
                item: Some("copper".into()),
                amount: 1,
            },
        ));
        assert!(!NetClient::apply_set_liquid_packet(
            &mut storage,
            &SetLiquidCallPacket {
                build,
                liquid: None,
                amount: 1.0,
            },
        ));
    }

    #[test]
    fn apply_building_team_and_health_packets_updates_lightweight_mirror() {
        let mut tiles = Tiles::new(4, 4);
        let center = point2_pack(1, 1);
        let proxy = point2_pack(2, 1);
        let empty = point2_pack(0, 0);
        NetClient::apply_set_tile_packet(
            &mut tiles,
            &SetTileCallPacket {
                tile: Some(center),
                block: Some("router".into()),
                team: TeamId(2),
                rotation: 0,
            },
            |name| match name {
                "router" => Some(10),
                _ => None,
            },
        )
        .unwrap();
        let build = tiles.get(1, 1).unwrap().build.unwrap();
        tiles.get_mut(2, 1).unwrap().build = Some(build);

        let mut storage = BTreeMap::<i32, ClientTileStorageMirror>::new();
        assert!(NetClient::apply_set_team_packet(
            &mut tiles,
            &mut storage,
            &SetTeamCallPacket {
                build: BuildingRef::new(center),
                team: TeamId(5),
            },
        ));
        assert_eq!(tiles.get(1, 1).unwrap().build.unwrap().team, 5);
        assert_eq!(tiles.get(2, 1).unwrap().build.unwrap().team, 5);
        assert_eq!(storage.get(&center).unwrap().team, Some(5));

        assert_eq!(
            NetClient::apply_set_teams_packet(
                &mut tiles,
                &mut storage,
                &SetTeamsCallPacket {
                    positions: vec![proxy, empty, point2_pack(9, 9)],
                    team: TeamId(6),
                },
            ),
            1
        );
        assert_eq!(tiles.get(1, 1).unwrap().build.unwrap().team, 6);
        assert_eq!(tiles.get(2, 1).unwrap().build.unwrap().team, 6);
        assert_eq!(storage.get(&center).unwrap().team, Some(6));

        let health = 123.5f32;
        assert_eq!(
            NetClient::apply_build_health_update_packet(
                &tiles,
                &mut storage,
                &BuildHealthUpdateCallPacket {
                    buildings: vec![
                        center,
                        health.to_bits() as i32,
                        empty,
                        999.0f32.to_bits() as i32,
                        center,
                    ],
                },
            ),
            1
        );
        assert_eq!(storage.get(&center).unwrap().health, Some(health));

        assert!(!NetClient::apply_set_team_packet(
            &mut tiles,
            &mut storage,
            &SetTeamCallPacket {
                build: BuildingRef::null(),
                team: TeamId(1),
            },
        ));
    }

    #[test]
    fn update_records_kick_packets_and_marks_client_disconnected_quietly() {
        let client = NetClient::default();
        client.begin_connecting();
        let kick_reason = KickCallPacket2 {
            reason: KickReason::ServerRestarting,
        };

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::KickCallPacket2(kick_reason));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.kick_packets_seen, 1);
        assert_eq!(state.last_kick_reason, Some(kick_reason));
        assert!(state.last_kick_reason_at.is_some());
        assert!(state.kicked);
        assert!(state.quiet);
        assert!(!state.connecting);
        assert!(!state.connected);
        assert!(state.timeout_deadline.is_none());
        assert_eq!(state.manual_disconnects, 0);
        drop(state);

        let kick = KickCallPacket {
            reason: "custom reason".into(),
        };
        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::KickCallPacket(kick.clone()));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.kick_packets_seen, 2);
        assert_eq!(state.last_kick.as_ref(), Some(&kick));
        assert!(state.last_kick_at.is_some());
        assert!(state.kicked);
    }

    #[test]
    fn update_records_block_snapshot_metadata_for_later_world_application() {
        let client = NetClient::default();
        let packet = BlockSnapshotCallPacket {
            amount: 2,
            data: vec![1, 2, 3, 4, 5, 6],
        };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::BlockSnapshotCallPacket(packet.clone()));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.block_snapshot_packets_seen, 1);
        assert_eq!(state.last_block_snapshot.as_ref(), Some(&packet));
        assert!(state.last_block_snapshot_at.is_some());
        assert!(state.last_server_snapshot_at.is_some());
    }

    #[test]
    fn update_records_objective_packets_like_java_rules_objectives_calls() {
        let client = NetClient::default();
        let objectives = SetObjectivesCallPacket {
            objectives_json: r#"{"objectives":[]}"#.into(),
        };
        let complete = CompleteObjectiveCallPacket { index: 3 };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::SetObjectivesCallPacket(objectives.clone()));
            net.handle_client_received(PacketKind::CompleteObjectiveCallPacket(complete));
            net.handle_client_received(PacketKind::ClearObjectivesCallPacket(
                ClearObjectivesCallPacket,
            ));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.set_objectives_packets_seen, 1);
        assert_eq!(state.last_set_objectives.as_ref(), Some(&objectives));
        assert!(state.last_set_objectives_at.is_some());
        assert_eq!(state.complete_objective_packets_seen, 1);
        assert_eq!(state.last_complete_objective, Some(complete));
        assert!(state.last_complete_objective_at.is_some());
        assert_eq!(state.clear_objectives_packets_seen, 1);
        assert!(state.last_clear_objectives_at.is_some());
        assert!(state.last_objectives_json.is_none());
        assert!(state.completed_objective_indices.is_empty());
    }

    #[test]
    fn update_records_connect_redirect_packet_and_marks_remote_disconnect() {
        let client = NetClient::default();
        client.begin_connecting();
        let packet = ConnectCallPacket {
            ip: "127.0.0.1".into(),
            port: 6567,
        };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::ConnectCallPacket(packet.clone()));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.connect_call_packets_seen, 1);
        assert_eq!(state.last_connect_call.as_ref(), Some(&packet));
        assert!(state.last_connect_call_at.is_some());
        assert!(state.quiet);
        assert!(!state.kicked);
        assert!(!state.connecting);
        assert!(!state.connected);
        assert!(state.timeout_deadline.is_none());
        assert_eq!(state.manual_disconnects, 0);
    }

    #[test]
    fn update_sends_configured_connect_packet_once_after_connect_event() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let client = NetClient::with_net(Net::new(Box::new(provider)));
        client.set_connect_config(Some(ClientConnectConfig {
            name: "rust-player".into(),
            locale: "zh_CN".into(),
            usid: "usid".into(),
            uuid: "uuid".into(),
            color: 42,
            ..ClientConnectConfig::default()
        }));

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::Connect(Connect {
                address_tcp: "127.0.0.1:6567".into(),
            }));
        }

        client.update();
        client.update();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert!(sent[0].1);
        match &sent[0].0 {
            PacketKind::ConnectPacket(packet) => {
                assert_eq!(packet.version, 157);
                assert_eq!(packet.version_type, "official");
                assert_eq!(packet.name, "rust-player");
                assert_eq!(packet.locale, "zh_CN");
                assert_eq!(packet.usid, "usid");
                assert_eq!(packet.uuid, "uuid");
                assert_eq!(packet.color, 42);
            }
            other => panic!("unexpected packet: {other:?}"),
        }

        let state = client.state();
        let state = state.lock().unwrap();
        assert!(state.connect_packet_sent);
        assert_eq!(
            state.last_sent_connect_packet.as_ref().unwrap().name,
            "rust-player"
        );
        assert!(state.last_connect_packet_error.is_none());
    }

    #[test]
    fn update_sends_ping_and_updates_rtt_from_response() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let client = NetClient::with_net(Net::new(Box::new(provider)));

        {
            let state = client.state();
            let mut state = state.lock().unwrap();
            state.connected = true;
            state.connect_confirm_sent = true;
            state.next_ping_at = Some(Instant::now() - Duration::from_secs(1));
        }

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
        }

        client.update();

        let request_time = {
            let sent = sent.lock().unwrap();
            assert_eq!(sent.len(), 1);
            match &sent[0].0 {
                PacketKind::PingCallPacket(packet) => packet.time,
                other => panic!("unexpected packet: {other:?}"),
            }
        };

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::PingResponseCallPacket(
                PingResponseCallPacket {
                    time: request_time - 37,
                },
            ));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.ping_requests_sent, 1);
        assert_eq!(state.ping_responses_received, 1);
        assert!(state.ping_ms >= 37);
        assert!(state.ping_ms <= 1000);
        assert_eq!(state.last_ping_request_time, Some(request_time));
        assert_eq!(state.last_ping_response_time, Some(request_time - 37));
        assert!(state.last_ping_request_error.is_none());
    }

    #[test]
    fn update_records_server_snapshots_when_client_loaded() {
        let client = NetClient::default();
        let state_snapshot = StateSnapshotCallPacket {
            wave_time: 1.25,
            wave: 2,
            enemies: 3,
            paused: false,
            game_over: false,
            time_data: 456,
            tps: 60,
            rand0: 11,
            rand1: 22,
            core_data: vec![1, 2, 3],
        };
        let entity_snapshot = EntitySnapshotCallPacket {
            amount: 2,
            data: vec![7, 8, 9],
        };
        let hidden_snapshot = HiddenSnapshotCallPacket { ids: vec![4, 5] };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::StateSnapshotCallPacket(state_snapshot.clone()));
            net.handle_client_received(PacketKind::EntitySnapshotCallPacket(
                entity_snapshot.clone(),
            ));
            net.handle_client_received(PacketKind::HiddenSnapshotCallPacket(
                hidden_snapshot.clone(),
            ));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.state_snapshot_packets_seen, 1);
        assert_eq!(state.last_state_snapshot.as_ref(), Some(&state_snapshot));
        assert!(state.last_state_snapshot_at.is_some());
        let mirror = state
            .last_state_snapshot_mirror
            .as_ref()
            .expect("state snapshot should mirror into lightweight game state");
        assert_eq!(mirror.wave_time, state_snapshot.wave_time);
        assert_eq!(mirror.wave, state_snapshot.wave);
        assert_eq!(mirror.enemies, state_snapshot.enemies);
        assert_eq!(mirror.paused, state_snapshot.paused);
        assert_eq!(mirror.game_over, state_snapshot.game_over);
        assert_eq!(mirror.time_data, state_snapshot.time_data);
        assert_eq!(mirror.tps, state_snapshot.tps);
        assert_eq!(mirror.rand0, state_snapshot.rand0);
        assert_eq!(mirror.rand1, state_snapshot.rand1);
        assert_eq!(mirror.core_data, state_snapshot.core_data);
        assert_eq!(state.entity_snapshot_packets_seen, 1);
        assert_eq!(state.last_entity_snapshot.as_ref(), Some(&entity_snapshot));
        assert!(state.last_entity_snapshot_at.is_some());
        assert_eq!(state.hidden_snapshot_packets_seen, 1);
        assert_eq!(state.last_hidden_snapshot.as_ref(), Some(&hidden_snapshot));
        assert!(state.last_hidden_snapshot_at.is_some());
        assert!(state.last_server_snapshot_at.is_some());
        assert!(matches!(
            state.last_packet,
            Some(PacketKind::HiddenSnapshotCallPacket(_))
        ));
    }

    #[test]
    fn priority_zero_server_snapshots_are_ignored_before_client_loaded() {
        let client = NetClient::default();

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::StateSnapshotCallPacket(
                StateSnapshotCallPacket {
                    wave_time: 1.25,
                    wave: 2,
                    enemies: 3,
                    paused: false,
                    game_over: false,
                    time_data: 456,
                    tps: 60,
                    rand0: 11,
                    rand1: 22,
                    core_data: vec![1, 2, 3],
                },
            ));
            net.handle_client_received(PacketKind::EntitySnapshotCallPacket(
                EntitySnapshotCallPacket {
                    amount: 2,
                    data: vec![7, 8, 9],
                },
            ));
            net.handle_client_received(PacketKind::HiddenSnapshotCallPacket(
                HiddenSnapshotCallPacket { ids: vec![4, 5] },
            ));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.state_snapshot_packets_seen, 0);
        assert!(state.last_state_snapshot.is_none());
        assert!(state.last_state_snapshot_mirror.is_none());
        assert_eq!(state.entity_snapshot_packets_seen, 0);
        assert!(state.last_entity_snapshot.is_none());
        assert_eq!(state.hidden_snapshot_packets_seen, 0);
        assert!(state.last_hidden_snapshot.is_none());
        assert!(state.last_server_snapshot_at.is_none());
        assert!(state.last_packet.is_none());
    }

    #[test]
    fn update_records_server_rule_packets_like_java_set_rules_calls() {
        let client = NetClient::default();
        let rules = SetRulesCallPacket {
            rules_json: r#"{"waves":true,"waveSpacing":7200}"#.into(),
        };
        let rule = SetRuleCallPacket {
            rule: "pvp".into(),
            json_data: "true".into(),
        };
        let second_rule = SetRuleCallPacket {
            rule: "unitCap".into(),
            json_data: "42".into(),
        };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::SetRulesCallPacket(rules.clone()));
            net.handle_client_received(PacketKind::SetRuleCallPacket(rule.clone()));
            net.handle_client_received(PacketKind::SetRuleCallPacket(second_rule.clone()));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.set_rules_packets_seen, 1);
        assert_eq!(state.last_set_rules.as_ref(), Some(&rules));
        assert!(state.last_set_rules_at.is_some());
        assert_eq!(
            state.last_rules_json.as_deref(),
            Some(rules.rules_json.as_str())
        );
        assert_eq!(state.set_rule_packets_seen, 2);
        assert_eq!(state.last_set_rule.as_ref(), Some(&second_rule));
        assert!(state.last_set_rule_at.is_some());
        assert_eq!(
            state.rule_json_patches.get("pvp").map(String::as_str),
            Some(rule.json_data.as_str())
        );
        assert_eq!(
            state.rule_json_patches.get("unitCap").map(String::as_str),
            Some(second_rule.json_data.as_str())
        );
    }

    #[test]
    fn set_rules_replaces_recorded_rule_patch_mirror() {
        let client = NetClient::default();

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::SetRuleCallPacket(SetRuleCallPacket {
                rule: "pvp".into(),
                json_data: "true".into(),
            }));
            net.handle_client_received(PacketKind::SetRulesCallPacket(SetRulesCallPacket {
                rules_json: r#"{"pvp":false}"#.into(),
            }));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.set_rule_packets_seen, 1);
        assert_eq!(state.set_rules_packets_seen, 1);
        assert!(state.rule_json_patches.is_empty());
        assert_eq!(state.last_rules_json.as_deref(), Some(r#"{"pvp":false}"#));
    }

    #[test]
    fn send_client_snapshot_helpers_emit_expected_packets() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let client = NetClient::with_net(Net::new(Box::new(provider)));

        client
            .send_client_snapshot(ClientSnapshotCallPacket {
                snapshot_id: 12,
                unit_id: 34,
                dead: false,
                x: 1.0,
                y: 2.0,
                pointer_x: 3.0,
                pointer_y: 4.0,
                rotation: 5.0,
                base_rotation: 6.0,
                x_velocity: 7.0,
                y_velocity: 8.0,
                mining: None,
                boosting: true,
                shooting: false,
                chatting: true,
                building: false,
                selected_block: Some("duo".into()),
                selected_rotation: 1,
                plans: None,
                view_x: 9.0,
                view_y: 10.0,
                view_width: 11.0,
                view_height: 12.0,
            })
            .unwrap();

        client
            .send_client_plan_snapshot(ClientPlanSnapshotCallPacket {
                group_id: 77,
                plans: None,
            })
            .unwrap();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 2);
        assert!(matches!(
            sent[0].0,
            PacketKind::ClientSnapshotCallPacket(ClientSnapshotCallPacket {
                snapshot_id: 12,
                ..
            })
        ));
        assert!(matches!(
            sent[1].0,
            PacketKind::ClientPlanSnapshotCallPacket(ClientPlanSnapshotCallPacket {
                group_id: 77,
                ..
            })
        ));
        assert!(sent.iter().all(|(_, reliable)| !*reliable));
    }

    #[test]
    fn tick_client_gameplay_sync_sends_due_snapshot_and_preview_plan_packets() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let client = NetClient::with_net(Net::new(Box::new(provider)));
        let mut player = PlayerComp::default();
        player.set_unit_state(
            PlayerUnitState::unit(34)
                .with_valid(true)
                .with_can_build(true),
        );

        let mut unit = UnitComp::new(34, unit_type(), TeamId(2));
        unit.set_pos(10.0, 20.0);
        unit.set_rotation(90.0);
        unit.vel.vel = Vec2::new(1.5, -2.5);
        unit.weapons.aim_x = 30.0;
        unit.weapons.aim_y = 40.0;

        let preview_plans = vec![BuildPlan::new_place(4, 5, 0, "router")];
        let start = Instant::now();

        {
            let state = client.state();
            let mut state = state.lock().unwrap();
            state.connected = true;
            state.connect_confirm_sent = true;
            state.next_client_snapshot_at = Some(start - Duration::from_millis(1));
            state.next_client_plan_snapshot_at = Some(start - Duration::from_millis(1));
        }

        client
            .tick_client_gameplay_sync(
                &mut player,
                Some(&unit),
                ClientInputSnapshot {
                    chatting: true,
                    building: true,
                    base_rotation: 15.0,
                },
                ClientCameraView {
                    x: 7.0,
                    y: 8.0,
                    width: 9.0,
                    height: 10.0,
                },
                &preview_plans,
            )
            .unwrap();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 2);
        assert!(matches!(
            sent[0].0,
            PacketKind::ClientSnapshotCallPacket(ClientSnapshotCallPacket { snapshot_id: 0, .. })
        ));
        assert!(matches!(
            sent[1].0,
            PacketKind::ClientPlanSnapshotCallPacket(ClientPlanSnapshotCallPacket {
                group_id: 0,
                ..
            })
        ));
        assert!(sent.iter().all(|(_, reliable)| !*reliable));

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.client_snapshot_packets_sent, 1);
        assert_eq!(state.client_plan_snapshot_packets_sent, 1);
        assert!(state.next_client_snapshot_at.unwrap() > start);
        assert!(state.next_client_plan_snapshot_at.unwrap() > start);
    }

    #[test]
    fn tick_client_gameplay_sync_skips_when_deadlines_are_in_the_future() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let client = NetClient::with_net(Net::new(Box::new(provider)));
        let mut player = PlayerComp::default();
        let preview_plans = vec![BuildPlan::new_place(4, 5, 0, "router")];
        let future = Instant::now() + Duration::from_secs(1);

        {
            let state = client.state();
            let mut state = state.lock().unwrap();
            state.connected = true;
            state.connect_confirm_sent = true;
            state.next_client_snapshot_at = Some(future);
            state.next_client_plan_snapshot_at = Some(future);
        }

        client
            .tick_client_gameplay_sync(
                &mut player,
                None,
                ClientInputSnapshot::default(),
                ClientCameraView::default(),
                &preview_plans,
            )
            .unwrap();

        let sent = sent.lock().unwrap();
        assert!(sent.is_empty());

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.client_snapshot_packets_sent, 0);
        assert_eq!(state.client_plan_snapshot_packets_sent, 0);
        assert_eq!(state.next_client_snapshot_at, Some(future));
        assert_eq!(state.next_client_plan_snapshot_at, Some(future));
    }

    #[test]
    fn next_client_snapshot_packet_mirrors_java_player_sync_shape() {
        let client = NetClient::default();
        let mut player = PlayerComp::default();
        player.x = 100.0;
        player.y = 200.0;
        player.boosting = true;
        player.shooting = true;
        player.selected_rotation = 2;
        player.selected_block = Some(Block::new(3, "router"));
        player.set_unit_state(
            PlayerUnitState::unit(34)
                .with_valid(true)
                .with_can_build(true),
        );

        let mut unit = UnitComp::new(34, unit_type(), TeamId(2));
        unit.set_pos(10.0, 20.0);
        unit.set_rotation(90.0);
        unit.vel.vel = Vec2::new(1.5, -2.5);
        unit.weapons.aim_x = 30.0;
        unit.weapons.aim_y = 40.0;
        unit.builder.plans.push_back(BuildPlan::new_config(
            4,
            5,
            1,
            "duo",
            TypeValue::String("cfg".into()),
        ));

        let snapshot = client.next_client_snapshot_packet(
            &player,
            Some(&unit),
            ClientInputSnapshot {
                chatting: true,
                building: true,
                base_rotation: 15.0,
            },
            ClientCameraView {
                x: 7.0,
                y: 8.0,
                width: 9.0,
                height: 10.0,
            },
        );

        assert_eq!(snapshot.snapshot_id, 0);
        assert_eq!(snapshot.unit_id, 34);
        assert!(!snapshot.dead);
        assert_eq!((snapshot.x, snapshot.y), (10.0, 20.0));
        assert_eq!((snapshot.pointer_x, snapshot.pointer_y), (30.0, 40.0));
        assert_eq!(snapshot.rotation, 90.0);
        assert_eq!(snapshot.base_rotation, 15.0);
        assert_eq!((snapshot.x_velocity, snapshot.y_velocity), (1.5, -2.5));
        assert!(snapshot.boosting);
        assert!(snapshot.shooting);
        assert!(snapshot.chatting);
        assert!(snapshot.building);
        assert_eq!(snapshot.selected_block.as_deref(), Some("router"));
        assert_eq!(snapshot.selected_rotation, 2);
        assert_eq!(snapshot.plans.as_ref().unwrap().len(), 1);
        assert_eq!(
            snapshot.plans.as_ref().unwrap()[0].config,
            TypeValue::String("cfg".into())
        );
        assert_eq!(
            (
                snapshot.view_x,
                snapshot.view_y,
                snapshot.view_width,
                snapshot.view_height
            ),
            (7.0, 8.0, 9.0, 10.0)
        );

        let next = client.next_client_snapshot_packet(
            &player,
            Some(&unit),
            ClientInputSnapshot::default(),
            ClientCameraView::default(),
        );
        assert_eq!(next.snapshot_id, 1);
    }

    #[test]
    fn client_plan_snapshot_packets_increment_group_and_chunk_preview_plans() {
        let mut player = PlayerComp::default();
        let plans: Vec<_> = (0..CLIENT_PLAN_PREVIEW_CHUNK_SIZE + 2)
            .map(|index| BuildPlan::new_place(index as i32, index as i32 + 1, 0, "router"))
            .collect();

        let packets = NetClient::client_plan_snapshot_packets_with_limit(
            &mut player,
            &plans,
            CLIENT_PLAN_PREVIEW_CHUNK_SIZE + 2,
        );

        assert_eq!(packets.len(), 2);
        assert_eq!(player.last_preview_plan_group, 0);
        assert_eq!(packets[0].group_id, 0);
        assert_eq!(
            packets[0].plans.as_ref().unwrap().len(),
            CLIENT_PLAN_PREVIEW_CHUNK_SIZE
        );
        assert_eq!(packets[1].group_id, 0);
        assert_eq!(packets[1].plans.as_ref().unwrap().len(), 2);

        let empty = NetClient::client_plan_snapshot_packets(&mut player, &[]);
        assert_eq!(player.last_preview_plan_group, 1);
        assert_eq!(
            empty,
            vec![ClientPlanSnapshotCallPacket {
                group_id: 1,
                plans: None
            }]
        );
    }

    #[test]
    fn update_records_received_preview_plan_packets_and_applies_to_player() {
        let client = NetClient::default();
        let packet = ClientPlanSnapshotReceivedCallPacket {
            player_id: 42,
            group_id: 3,
            plans: Some(vec![BuildPlanWire::new_place_config(
                1,
                2,
                0,
                "router",
                TypeValue::String("cfg".into()),
            )]),
        };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::ClientPlanSnapshotReceivedCallPacket(
                packet.clone(),
            ));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.client_plan_snapshot_received_packets_seen, 1);
        assert_eq!(
            state.last_client_plan_snapshot_received.as_ref(),
            Some(&packet)
        );
        assert!(state.last_client_plan_snapshot_received_at.is_some());
        drop(state);

        let mut player = PlayerComp::default();
        let added = NetClient::apply_received_preview_plans_to_player(&mut player, &packet, 10, 10)
            .unwrap();
        assert_eq!(added, 1);
        assert_eq!(player.preview_plans_assembling.len(), 1);
        assert_eq!(player.last_preview_plan_group, 3);
        assert!(player.receiving_new_plan_group);

        let current = player.get_preview_plans(110);
        assert_eq!(current.len(), 1);
        assert_eq!(current[0].config, TypeValue::String("cfg".into()));
    }

    #[test]
    fn update_records_server_forwarded_input_packets() {
        let client = NetClient::default();
        let build_pos = 12_345;
        let tap_pos = 54_321;
        let tile_config = TileConfigCallPacket::server(
            EntityRef::new(7),
            BuildingRef::new(build_pos),
            TypeValue::String("cfg".into()),
        );
        let rotate_block =
            RotateBlockCallPacket::server(EntityRef::new(7), BuildingRef::new(build_pos), true);
        let tile_tap = TileTapCallPacket::server(EntityRef::new(7), Some(tap_pos));
        let remove_queue_block = RemoveQueueBlockCallPacket {
            x: 3,
            y: 4,
            breaking: true,
        };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::TileConfigCallPacket(tile_config.clone()));
            net.handle_client_received(PacketKind::RotateBlockCallPacket(rotate_block));
            net.handle_client_received(PacketKind::TileTapCallPacket(tile_tap.clone()));
            net.handle_client_received(PacketKind::RemoveQueueBlockCallPacket(remove_queue_block));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.tile_config_packets_seen, 1);
        assert_eq!(state.last_tile_config.as_ref(), Some(&tile_config));
        assert!(state.last_tile_config_at.is_some());
        assert_eq!(state.rotate_block_packets_seen, 1);
        assert_eq!(state.last_rotate_block.as_ref(), Some(&rotate_block));
        assert!(state.last_rotate_block_at.is_some());
        assert_eq!(state.tile_tap_packets_seen, 1);
        assert_eq!(state.last_tile_tap.as_ref(), Some(&tile_tap));
        assert!(state.last_tile_tap_at.is_some());
        assert_eq!(state.remove_queue_block_packets_seen, 1);
        assert_eq!(
            state.last_remove_queue_block.as_ref(),
            Some(&remove_queue_block)
        );
        assert!(state.last_remove_queue_block_at.is_some());
        assert!(matches!(
            state.last_packet,
            Some(PacketKind::RemoveQueueBlockCallPacket(_))
        ));
    }

    #[test]
    fn update_records_server_forwarded_inventory_payload_and_unit_packets() {
        let client = NetClient::default();
        let primary_build = BuildingRef::new(22_001);
        let secondary_build = BuildingRef::new(22_002);
        let set_item = SetItemCallPacket {
            build: primary_build,
            item: Some("copper".into()),
            amount: 42,
        };
        let set_items = SetItemsCallPacket {
            build: secondary_build,
            items: vec![ItemStack::new("lead", 3), ItemStack::new("scrap", 4)],
        };
        let clear_items = ClearItemsCallPacket {
            build: secondary_build,
        };
        let set_liquid = SetLiquidCallPacket {
            build: primary_build,
            liquid: Some("water".into()),
            amount: 6.5,
        };
        let set_liquids = SetLiquidsCallPacket {
            build: secondary_build,
            liquids: vec![
                LiquidStack::new("water", 1.25),
                LiquidStack::new("slag", 2.5),
            ],
        };
        let clear_liquids = ClearLiquidsCallPacket {
            build: secondary_build,
        };
        let take_items = TakeItemsCallPacket {
            build: primary_build,
            item: Some("copper".into()),
            amount: 5,
            to: UnitRef::Unit { id: 410 },
        };
        let transfer_item_effect = TransferItemEffectCallPacket {
            item: Some("lead".into()),
            x: 10.5,
            y: 20.25,
            to: EntityRef::new(315),
        };
        let transfer_item_to = TransferItemToCallPacket {
            unit: UnitRef::Unit { id: 411 },
            item: Some("scrap".into()),
            amount: 9,
            x: 30.5,
            y: 40.25,
            build: secondary_build,
        };
        let transfer_item_to_unit = TransferItemToUnitCallPacket {
            item: Some("titanium".into()),
            x: 50.5,
            y: 60.25,
            to: EntityRef::new(316),
        };
        let request_item = RequestItemCallPacket {
            player: EntityRef::new(301),
            build: primary_build,
            item: Some("copper".into()),
            amount: 77,
        };
        let transfer_inventory = TransferInventoryCallPacket {
            player: EntityRef::new(302),
            build: secondary_build,
        };
        let request_build_payload = RequestBuildPayloadCallPacket {
            player: EntityRef::new(303),
            build: primary_build,
        };
        let request_unit_payload = RequestUnitPayloadCallPacket {
            player: EntityRef::new(307),
            target: UnitRef::Unit { id: 404 },
        };
        let picked_build_payload = PickedBuildPayloadCallPacket {
            unit: UnitRef::Unit { id: 401 },
            build_pos: Some(22_003),
            on_ground: true,
        };
        let picked_unit_payload = PickedUnitPayloadCallPacket {
            unit: UnitRef::Unit { id: 405 },
            target: UnitRef::Unit { id: 406 },
        };
        let request_drop_payload = RequestDropPayloadCallPacket {
            player: EntityRef::new(308),
            x: 123.25,
            y: 456.5,
        };
        let payload_dropped = PayloadDroppedCallPacket {
            unit: UnitRef::Unit { id: 407 },
            x: 321.75,
            y: 654.25,
        };
        let unit_entered_payload = UnitEnteredPayloadCallPacket {
            unit: UnitRef::Unit { id: 408 },
            build: secondary_build,
        };
        let ping_location = PingLocationCallPacket {
            player_id: Some(309),
            x: 11.5,
            y: 22.5,
            text: "go".into(),
        };
        let delete_plans = DeletePlansCallPacket {
            player_id: Some(310),
            positions: vec![1, 2, 3],
        };
        let command_building = CommandBuildingCallPacket {
            player: EntityRef::new(314),
            buildings: vec![22_004, 22_005],
            target: Vec2::new(7.0, 8.0),
        };
        let command_units = CommandUnitsCallPacket {
            player: EntityRef::new(311),
            unit_ids: vec![77, 78],
            build_target: secondary_build,
            unit_target: UnitRef::Unit { id: 409 },
            pos_target: Vec2::new(9.0, 10.0),
            queue_command: true,
            final_batch: false,
        };
        let set_unit_command = SetUnitCommandCallPacket {
            player: EntityRef::new(312),
            unit_ids: vec![77, 78],
            command: "move".into(),
        };
        let set_unit_stance = SetUnitStanceCallPacket {
            player: EntityRef::new(313),
            unit_ids: vec![77, 78],
            stance: "holdfire".into(),
            enable: false,
        };
        let building_control_select = BuildingControlSelectCallPacket {
            player: EntityRef::new(306),
            build: primary_build,
        };
        let unit_building_control_select = UnitBuildingControlSelectCallPacket {
            unit: UnitRef::Unit { id: 402 },
            build: secondary_build,
        };
        let unit_control = UnitControlCallPacket {
            player: EntityRef::new(304),
            unit: UnitRef::Unit { id: 403 },
        };
        let unit_clear = UnitClearCallPacket {
            player: EntityRef::new(305),
        };

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
            net.handle_client_received(PacketKind::SetItemCallPacket(set_item.clone()));
            net.handle_client_received(PacketKind::SetItemsCallPacket(set_items.clone()));
            net.handle_client_received(PacketKind::ClearItemsCallPacket(clear_items));
            net.handle_client_received(PacketKind::SetLiquidCallPacket(set_liquid.clone()));
            net.handle_client_received(PacketKind::SetLiquidsCallPacket(set_liquids.clone()));
            net.handle_client_received(PacketKind::ClearLiquidsCallPacket(clear_liquids));
            net.handle_client_received(PacketKind::TakeItemsCallPacket(take_items.clone()));
            net.handle_client_received(PacketKind::TransferItemEffectCallPacket(
                transfer_item_effect.clone(),
            ));
            net.handle_client_received(PacketKind::TransferItemToCallPacket(
                transfer_item_to.clone(),
            ));
            net.handle_client_received(PacketKind::TransferItemToUnitCallPacket(
                transfer_item_to_unit.clone(),
            ));
            net.handle_client_received(PacketKind::RequestItemCallPacket(request_item.clone()));
            net.handle_client_received(PacketKind::TransferInventoryCallPacket(
                transfer_inventory.clone(),
            ));
            net.handle_client_received(PacketKind::RequestBuildPayloadCallPacket(
                request_build_payload.clone(),
            ));
            net.handle_client_received(PacketKind::RequestUnitPayloadCallPacket(
                request_unit_payload.clone(),
            ));
            net.handle_client_received(PacketKind::PickedBuildPayloadCallPacket(
                picked_build_payload,
            ));
            net.handle_client_received(PacketKind::PickedUnitPayloadCallPacket(
                picked_unit_payload.clone(),
            ));
            net.handle_client_received(PacketKind::RequestDropPayloadCallPacket(
                request_drop_payload.clone(),
            ));
            net.handle_client_received(PacketKind::PayloadDroppedCallPacket(
                payload_dropped.clone(),
            ));
            net.handle_client_received(PacketKind::UnitEnteredPayloadCallPacket(
                unit_entered_payload.clone(),
            ));
            net.handle_client_received(PacketKind::PingLocationCallPacket(ping_location.clone()));
            net.handle_client_received(PacketKind::DeletePlansCallPacket(delete_plans.clone()));
            net.handle_client_received(PacketKind::CommandBuildingCallPacket(
                command_building.clone(),
            ));
            net.handle_client_received(PacketKind::CommandUnitsCallPacket(command_units.clone()));
            net.handle_client_received(PacketKind::SetUnitCommandCallPacket(
                set_unit_command.clone(),
            ));
            net.handle_client_received(PacketKind::SetUnitStanceCallPacket(
                set_unit_stance.clone(),
            ));
            net.handle_client_received(PacketKind::UnitBuildingControlSelectCallPacket(
                unit_building_control_select.clone(),
            ));
            net.handle_client_received(PacketKind::UnitControlCallPacket(unit_control.clone()));
            net.handle_client_received(PacketKind::UnitClearCallPacket(unit_clear.clone()));
            net.handle_client_received(PacketKind::BuildingControlSelectCallPacket(
                building_control_select.clone(),
            ));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.set_item_packets_seen, 1);
        assert_eq!(state.last_set_item.as_ref(), Some(&set_item));
        assert!(state.last_set_item_at.is_some());
        assert_eq!(state.set_items_packets_seen, 1);
        assert_eq!(state.last_set_items.as_ref(), Some(&set_items));
        assert!(state.last_set_items_at.is_some());
        assert_eq!(state.clear_items_packets_seen, 1);
        assert_eq!(state.last_clear_items.as_ref(), Some(&clear_items));
        assert!(state.last_clear_items_at.is_some());
        assert_eq!(state.set_liquid_packets_seen, 1);
        assert_eq!(state.last_set_liquid.as_ref(), Some(&set_liquid));
        assert!(state.last_set_liquid_at.is_some());
        assert_eq!(state.set_liquids_packets_seen, 1);
        assert_eq!(state.last_set_liquids.as_ref(), Some(&set_liquids));
        assert!(state.last_set_liquids_at.is_some());
        assert_eq!(state.clear_liquids_packets_seen, 1);
        assert_eq!(state.last_clear_liquids.as_ref(), Some(&clear_liquids));
        assert!(state.last_clear_liquids_at.is_some());
        let primary_storage = state
            .building_storage_mirrors
            .get(&primary_build.tile_pos.unwrap())
            .unwrap();
        assert_eq!(primary_storage.items.get("copper"), Some(&42));
        assert_eq!(primary_storage.liquids.get("water"), Some(&6.5));
        let secondary_storage = state
            .building_storage_mirrors
            .get(&secondary_build.tile_pos.unwrap())
            .unwrap();
        assert!(secondary_storage.items.is_empty());
        assert!(secondary_storage.liquids.is_empty());
        assert_eq!(state.take_items_packets_seen, 1);
        assert_eq!(state.last_take_items.as_ref(), Some(&take_items));
        assert!(state.last_take_items_at.is_some());
        assert_eq!(state.transfer_item_effect_packets_seen, 1);
        assert_eq!(
            state.last_transfer_item_effect.as_ref(),
            Some(&transfer_item_effect)
        );
        assert!(state.last_transfer_item_effect_at.is_some());
        assert_eq!(state.transfer_item_to_packets_seen, 1);
        assert_eq!(
            state.last_transfer_item_to.as_ref(),
            Some(&transfer_item_to)
        );
        assert!(state.last_transfer_item_to_at.is_some());
        assert_eq!(state.transfer_item_to_unit_packets_seen, 1);
        assert_eq!(
            state.last_transfer_item_to_unit.as_ref(),
            Some(&transfer_item_to_unit)
        );
        assert!(state.last_transfer_item_to_unit_at.is_some());
        assert_eq!(state.request_item_packets_seen, 1);
        assert_eq!(state.last_request_item.as_ref(), Some(&request_item));
        assert!(state.last_request_item_at.is_some());
        assert_eq!(state.transfer_inventory_packets_seen, 1);
        assert_eq!(
            state.last_transfer_inventory.as_ref(),
            Some(&transfer_inventory)
        );
        assert!(state.last_transfer_inventory_at.is_some());
        assert_eq!(state.request_build_payload_packets_seen, 1);
        assert_eq!(
            state.last_request_build_payload.as_ref(),
            Some(&request_build_payload)
        );
        assert!(state.last_request_build_payload_at.is_some());
        assert_eq!(state.request_unit_payload_packets_seen, 1);
        assert_eq!(
            state.last_request_unit_payload.as_ref(),
            Some(&request_unit_payload)
        );
        assert!(state.last_request_unit_payload_at.is_some());
        assert_eq!(state.picked_build_payload_packets_seen, 1);
        assert_eq!(
            state.last_picked_build_payload.as_ref(),
            Some(&picked_build_payload)
        );
        assert!(state.last_picked_build_payload_at.is_some());
        assert_eq!(state.picked_unit_payload_packets_seen, 1);
        assert_eq!(
            state.last_picked_unit_payload.as_ref(),
            Some(&picked_unit_payload)
        );
        assert!(state.last_picked_unit_payload_at.is_some());
        assert_eq!(state.request_drop_payload_packets_seen, 1);
        assert_eq!(
            state.last_request_drop_payload.as_ref(),
            Some(&request_drop_payload)
        );
        assert!(state.last_request_drop_payload_at.is_some());
        assert_eq!(state.payload_dropped_packets_seen, 1);
        assert_eq!(state.last_payload_dropped.as_ref(), Some(&payload_dropped));
        assert!(state.last_payload_dropped_at.is_some());
        assert_eq!(state.unit_entered_payload_packets_seen, 1);
        assert_eq!(
            state.last_unit_entered_payload.as_ref(),
            Some(&unit_entered_payload)
        );
        assert!(state.last_unit_entered_payload_at.is_some());
        assert_eq!(state.ping_location_packets_seen, 1);
        assert_eq!(state.last_ping_location.as_ref(), Some(&ping_location));
        assert!(state.last_ping_location_at.is_some());
        assert_eq!(state.delete_plans_packets_seen, 1);
        assert_eq!(state.last_delete_plans.as_ref(), Some(&delete_plans));
        assert!(state.last_delete_plans_at.is_some());
        assert_eq!(state.command_building_packets_seen, 1);
        assert_eq!(
            state.last_command_building.as_ref(),
            Some(&command_building)
        );
        assert!(state.last_command_building_at.is_some());
        assert_eq!(state.command_units_packets_seen, 1);
        assert_eq!(state.last_command_units.as_ref(), Some(&command_units));
        assert!(state.last_command_units_at.is_some());
        assert_eq!(state.set_unit_command_packets_seen, 1);
        assert_eq!(
            state.last_set_unit_command.as_ref(),
            Some(&set_unit_command)
        );
        assert!(state.last_set_unit_command_at.is_some());
        assert_eq!(state.set_unit_stance_packets_seen, 1);
        assert_eq!(state.last_set_unit_stance.as_ref(), Some(&set_unit_stance));
        assert!(state.last_set_unit_stance_at.is_some());
        assert_eq!(state.building_control_select_packets_seen, 1);
        assert_eq!(
            state.last_building_control_select.as_ref(),
            Some(&building_control_select)
        );
        assert!(state.last_building_control_select_at.is_some());
        assert_eq!(state.unit_building_control_select_packets_seen, 1);
        assert_eq!(
            state.last_unit_building_control_select.as_ref(),
            Some(&unit_building_control_select)
        );
        assert!(state.last_unit_building_control_select_at.is_some());
        assert_eq!(state.unit_control_packets_seen, 1);
        assert_eq!(state.last_unit_control.as_ref(), Some(&unit_control));
        assert!(state.last_unit_control_at.is_some());
        assert_eq!(state.unit_clear_packets_seen, 1);
        assert_eq!(state.last_unit_clear.as_ref(), Some(&unit_clear));
        assert!(state.last_unit_clear_at.is_some());
        assert!(matches!(
            state.last_packet,
            Some(PacketKind::BuildingControlSelectCallPacket(_))
        ));
    }

    #[test]
    fn update_resets_loading_timeout_when_stream_chunk_progresses() {
        let client = NetClient::default();
        client.begin_connecting();

        let initial_deadline = {
            let state = client.state();
            let state = state.lock().unwrap();
            state.timeout_deadline.unwrap()
        };

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::StreamBegin(StreamBegin {
                id: 7,
                total: 4,
                packet_type: 2,
            }));
        }

        client.update();

        {
            let state = client.state();
            let state = state.lock().unwrap();
            assert_eq!(state.timeout_resets, 1);
            assert_eq!(state.last_stream_id, Some(7));
            assert_eq!(state.last_stream_len, 0);
            assert_eq!(state.timeout_deadline.unwrap(), initial_deadline);
        }

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::StreamChunk(StreamChunk {
                id: 7,
                data: vec![1, 2],
            }));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.timeout_resets, 2);
        assert_eq!(state.last_stream_id, Some(7));
        assert_eq!(state.last_stream_len, 2);
        assert!(state.timeout_deadline.unwrap() > initial_deadline);
    }

    #[test]
    fn update_disconnects_when_loading_timeout_expires() {
        let client = NetClient::default();
        client.begin_connecting();

        {
            let state = client.state();
            let mut state = state.lock().unwrap();
            state.timeout_deadline = Some(Instant::now() - Duration::from_secs(1));
        }

        client.update();

        let state = client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.timeout_disconnects, 1);
        assert_eq!(state.manual_disconnects, 0);
        assert!(!state.connecting);
        assert!(!state.connected);
        assert!(state.timeout_deadline.is_none());
    }

    #[test]
    fn update_sends_connect_confirm_once_after_world_stream_and_marks_loaded() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let client = NetClient::with_net(Net::new(Box::new(provider)));
        client.begin_connecting();

        let payload = write_minimal_world_data(17).unwrap();
        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::Streamable(Streamable::new(payload)));
        }

        client.update();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert!(sent[0].1);
        assert!(matches!(sent[0].0, PacketKind::ConnectConfirmCallPacket(_)));

        let state = client.state();
        let state = state.lock().unwrap();
        assert!(!state.connecting);
        assert!(state.connected);
        assert!(!state.world_data_loading);
        assert!(state.connect_confirm_sent);
        assert!(state.last_connect_confirm_error.is_none());
    }

    #[test]
    fn invalid_world_stream_does_not_confirm_or_finish_loading() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let client = NetClient::with_net(Net::new(Box::new(provider)));
        client.begin_connecting();

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::Streamable(Streamable::new(vec![1, 2, 3])));
        }

        client.update();

        assert!(sent.lock().unwrap().is_empty());
        let state = client.state();
        let state = state.lock().unwrap();
        assert!(!state.connect_confirm_sent);
        assert!(!state.connected);
        assert!(state.world_data_loading);
        assert!(state.last_loaded_world_data.is_none());
        assert!(state.last_world_data_error.is_some());
    }

    #[test]
    fn world_data_begin_marks_client_loading_until_world_stream_confirms() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let client = NetClient::with_net(Net::new(Box::new(provider)));

        {
            let mut net = client.net_mut();
            net.set_client_loaded(true);
        }

        {
            let state = client.state();
            let mut state = state.lock().unwrap();
            state.connected = true;
            state.connect_confirm_sent = true;
        }

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::WorldDataBeginCallPacket(
                WorldDataBeginCallPacket,
            ));
        }

        client.update();

        {
            let state = client.state();
            let state = state.lock().unwrap();
            assert!(state.connecting);
            assert!(!state.connected);
            assert!(state.world_data_loading);
            assert_eq!(state.world_data_begin_packets_seen, 1);
            assert!(state.last_world_data_begin_at.is_some());
            assert!(!state.connect_confirm_sent);
            assert!(state.last_connect_confirm_error.is_none());
            assert!(state.timeout_deadline.is_some());
        }

        let payload = write_minimal_world_data(23).unwrap();
        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::Streamable(Streamable::new(payload)));
        }

        client.update();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert!(sent[0].1);
        assert!(matches!(sent[0].0, PacketKind::ConnectConfirmCallPacket(_)));

        let state = client.state();
        let state = state.lock().unwrap();
        assert!(!state.connecting);
        assert!(state.connected);
        assert!(!state.world_data_loading);
        assert!(state.connect_confirm_sent);
        assert_eq!(state.world_data_begin_packets_seen, 1);
        assert!(state.timeout_deadline.is_none());
    }

    #[test]
    fn world_stream_with_java_like_payload_is_parsed_and_confirmed() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let client = NetClient::with_net(Net::new(Box::new(provider)));
        client.begin_connecting();

        let payload = write_minimal_world_data(91).unwrap();
        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::Streamable(Streamable::new(payload)));
        }

        client.update();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert!(sent[0].1);
        assert!(matches!(sent[0].0, PacketKind::ConnectConfirmCallPacket(_)));

        let state = client.state();
        let state = state.lock().unwrap();
        assert!(state.connect_confirm_sent);
        assert_eq!(state.world_stream_events, 1);
        assert!(state.last_world_data_error.is_none());
        let world = state
            .last_loaded_world_data
            .as_ref()
            .expect("world data should be recorded");
        assert_eq!(world.player_id, 91);
        assert_eq!(world.rules_json, "{}");
        assert_eq!(world.map_locales_json, "{}");
        assert!(world.map_tags.contains_key("name"));
        assert!(world.player_bytes.is_empty());
    }

    #[test]
    fn partial_world_stream_does_not_send_connect_confirm() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let client = NetClient::with_net(Net::new(Box::new(provider)));

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::StreamBegin(StreamBegin {
                id: 7,
                total: 3,
                packet_type: 2,
            }));
            net.handle_client_received(PacketKind::StreamChunk(StreamChunk {
                id: 7,
                data: vec![1, 2],
            }));
        }

        client.update();

        assert!(sent.lock().unwrap().is_empty());
        let state = client.state();
        let state = state.lock().unwrap();
        assert!(!state.connect_confirm_sent);
    }
}
