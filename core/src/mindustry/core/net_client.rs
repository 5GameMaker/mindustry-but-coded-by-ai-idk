use std::fmt;
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::mindustry::entities::comp::{PlayerComp, UnitComp};
use crate::mindustry::entities::units::BuildPlan;
use crate::mindustry::io::BuildPlanWire;
use crate::mindustry::net::{
    BuildingControlSelectCallPacket, ClearItemsCallPacket, ClearLiquidsCallPacket,
    ClientPlanSnapshotCallPacket, ClientPlanSnapshotReceivedCallPacket, ClientSnapshotCallPacket,
    CommandUnitsCallPacket, Connect, ConnectConfirmCallPacket, ConnectPacket,
    DeletePlansCallPacket, Disconnect, EntitySnapshotCallPacket, HiddenSnapshotCallPacket, Net,
    PacketKind, PayloadDroppedCallPacket, PickedBuildPayloadCallPacket,
    PickedUnitPayloadCallPacket, PingCallPacket, PingLocationCallPacket, ProviderEvent,
    RemoveQueueBlockCallPacket, RequestBuildPayloadCallPacket, RequestDropPayloadCallPacket,
    RequestItemCallPacket, RequestUnitPayloadCallPacket, RotateBlockCallPacket, SetItemCallPacket,
    SetItemsCallPacket, SetLiquidCallPacket, SetLiquidsCallPacket, SetUnitCommandCallPacket,
    SetUnitStanceCallPacket, StateSnapshotCallPacket, StreamBuilder, Streamable,
    TileConfigCallPacket, TileTapCallPacket, TransferInventoryCallPacket,
    UnitBuildingControlSelectCallPacket, UnitClearCallPacket, UnitControlCallPacket,
    UnitEnteredPayloadCallPacket,
};
use crate::mindustry::vars::MAX_PLAYER_PREVIEW_PLANS;

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
    pub last_packet: Option<PacketKind>,
    pub connect_config: Option<ClientConnectConfig>,
    pub connect_packet_sent: bool,
    pub last_sent_connect_packet: Option<ConnectPacket>,
    pub last_connect_packet_error: Option<String>,
    pub auto_confirm_world_stream: bool,
    pub connect_confirm_sent: bool,
    pub last_connect_confirm_error: Option<String>,
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
            .field("last_packet", &self.last_packet)
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
        self.connect_events += 1;
        self.last_connect = Some(connect.clone());
        self.last_packet = Some(PacketKind::Connect(connect.clone()));
        self.connect_packet_sent = false;
        self.last_sent_connect_packet = None;
        self.last_connect_packet_error = None;
        self.connect_confirm_sent = false;
        self.last_connect_confirm_error = None;
        self.reset_ping_state();
        self.reset_client_gameplay_sync_state();
        self.clear_loading_stream_tracking();
    }

    fn record_disconnect(&mut self, disconnect: &Disconnect) {
        self.connecting = false;
        self.connected = false;
        self.disconnect_events += 1;
        self.last_disconnect = Some(disconnect.clone());
        self.last_packet = Some(PacketKind::Disconnect(disconnect.clone()));
        self.connect_packet_sent = false;
        self.connect_confirm_sent = false;
        self.last_connect_confirm_error = None;
        self.reset_ping_state();
        self.reset_client_gameplay_sync_state();
        self.clear_loading_stream_tracking();
        self.clear_timeout_clock();
    }

    fn record_world_stream(&mut self, stream: &Streamable) {
        self.connecting = false;
        self.connected = true;
        self.world_stream_events += 1;
        self.last_world_stream = Some(stream.clone());
        self.last_binary_stream = Some(stream.stream.clone());
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
        state.connection_attempts += 1;
        state.last_update_at = Some(Instant::now());
        state.connect_packet_sent = false;
        state.last_sent_connect_packet = None;
        state.last_connect_packet_error = None;
        state.connect_confirm_sent = false;
        state.last_connect_confirm_error = None;
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
        state.manual_disconnects += 1;
        state.last_update_at = Some(Instant::now());
        state.connect_packet_sent = false;
        state.connect_confirm_sent = false;
        state.last_connect_confirm_error = None;
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

            let connect_confirm_to_send = {
                let mut state = self.state.lock().unwrap();
                state.last_packet = Some(packet.clone());
                match &packet {
                    PacketKind::Streamable(stream) => {
                        state.last_binary_stream = Some(stream.stream.clone());
                        if state.auto_confirm_world_stream && !state.connect_confirm_sent {
                            state.connect_confirm_sent = true;
                            state.last_connect_confirm_error = None;
                            true
                        } else {
                            false
                        }
                    }
                    PacketKind::PingResponseCallPacket(response) => {
                        let now = Self::current_millis();
                        state.ping_responses_received += 1;
                        state.last_ping_response_time = Some(response.time);
                        state.last_ping_response_at = Some(Instant::now());
                        state.ping_ms = now.saturating_sub(response.time).max(0) as u32;
                        false
                    }
                    PacketKind::EntitySnapshotCallPacket(snapshot) => {
                        let now = Instant::now();
                        state.entity_snapshot_packets_seen += 1;
                        state.last_entity_snapshot = Some(snapshot.clone());
                        state.last_entity_snapshot_at = Some(now);
                        state.last_server_snapshot_at = Some(now);
                        false
                    }
                    PacketKind::HiddenSnapshotCallPacket(snapshot) => {
                        let now = Instant::now();
                        state.hidden_snapshot_packets_seen += 1;
                        state.last_hidden_snapshot = Some(snapshot.clone());
                        state.last_hidden_snapshot_at = Some(now);
                        state.last_server_snapshot_at = Some(now);
                        false
                    }
                    PacketKind::StateSnapshotCallPacket(snapshot) => {
                        let now = Instant::now();
                        state.state_snapshot_packets_seen += 1;
                        state.last_state_snapshot = Some(snapshot.clone());
                        state.last_state_snapshot_at = Some(now);
                        state.last_server_snapshot_at = Some(now);
                        false
                    }
                    PacketKind::ClientPlanSnapshotReceivedCallPacket(snapshot) => {
                        let now = Instant::now();
                        state.client_plan_snapshot_received_packets_seen += 1;
                        state.last_client_plan_snapshot_received = Some(snapshot.clone());
                        state.last_client_plan_snapshot_received_at = Some(now);
                        false
                    }
                    PacketKind::SetItemCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_item_packets_seen += 1;
                        state.last_set_item = Some(packet.clone());
                        state.last_set_item_at = Some(now);
                        false
                    }
                    PacketKind::SetItemsCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_items_packets_seen += 1;
                        state.last_set_items = Some(packet.clone());
                        state.last_set_items_at = Some(now);
                        false
                    }
                    PacketKind::ClearItemsCallPacket(packet) => {
                        let now = Instant::now();
                        state.clear_items_packets_seen += 1;
                        state.last_clear_items = Some(*packet);
                        state.last_clear_items_at = Some(now);
                        false
                    }
                    PacketKind::SetLiquidCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_liquid_packets_seen += 1;
                        state.last_set_liquid = Some(packet.clone());
                        state.last_set_liquid_at = Some(now);
                        false
                    }
                    PacketKind::SetLiquidsCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_liquids_packets_seen += 1;
                        state.last_set_liquids = Some(packet.clone());
                        state.last_set_liquids_at = Some(now);
                        false
                    }
                    PacketKind::ClearLiquidsCallPacket(packet) => {
                        let now = Instant::now();
                        state.clear_liquids_packets_seen += 1;
                        state.last_clear_liquids = Some(*packet);
                        state.last_clear_liquids_at = Some(now);
                        false
                    }
                    PacketKind::RequestItemCallPacket(packet) => {
                        let now = Instant::now();
                        state.request_item_packets_seen += 1;
                        state.last_request_item = Some(packet.clone());
                        state.last_request_item_at = Some(now);
                        false
                    }
                    PacketKind::TransferInventoryCallPacket(packet) => {
                        let now = Instant::now();
                        state.transfer_inventory_packets_seen += 1;
                        state.last_transfer_inventory = Some(packet.clone());
                        state.last_transfer_inventory_at = Some(now);
                        false
                    }
                    PacketKind::RequestBuildPayloadCallPacket(packet) => {
                        let now = Instant::now();
                        state.request_build_payload_packets_seen += 1;
                        state.last_request_build_payload = Some(packet.clone());
                        state.last_request_build_payload_at = Some(now);
                        false
                    }
                    PacketKind::RequestUnitPayloadCallPacket(packet) => {
                        let now = Instant::now();
                        state.request_unit_payload_packets_seen += 1;
                        state.last_request_unit_payload = Some(packet.clone());
                        state.last_request_unit_payload_at = Some(now);
                        false
                    }
                    PacketKind::PickedBuildPayloadCallPacket(packet) => {
                        let now = Instant::now();
                        state.picked_build_payload_packets_seen += 1;
                        state.last_picked_build_payload = Some(packet.clone());
                        state.last_picked_build_payload_at = Some(now);
                        false
                    }
                    PacketKind::PickedUnitPayloadCallPacket(packet) => {
                        let now = Instant::now();
                        state.picked_unit_payload_packets_seen += 1;
                        state.last_picked_unit_payload = Some(packet.clone());
                        state.last_picked_unit_payload_at = Some(now);
                        false
                    }
                    PacketKind::RequestDropPayloadCallPacket(packet) => {
                        let now = Instant::now();
                        state.request_drop_payload_packets_seen += 1;
                        state.last_request_drop_payload = Some(packet.clone());
                        state.last_request_drop_payload_at = Some(now);
                        false
                    }
                    PacketKind::PayloadDroppedCallPacket(packet) => {
                        let now = Instant::now();
                        state.payload_dropped_packets_seen += 1;
                        state.last_payload_dropped = Some(packet.clone());
                        state.last_payload_dropped_at = Some(now);
                        false
                    }
                    PacketKind::UnitEnteredPayloadCallPacket(packet) => {
                        let now = Instant::now();
                        state.unit_entered_payload_packets_seen += 1;
                        state.last_unit_entered_payload = Some(packet.clone());
                        state.last_unit_entered_payload_at = Some(now);
                        false
                    }
                    PacketKind::PingLocationCallPacket(packet) => {
                        let now = Instant::now();
                        state.ping_location_packets_seen += 1;
                        state.last_ping_location = Some(packet.clone());
                        state.last_ping_location_at = Some(now);
                        false
                    }
                    PacketKind::DeletePlansCallPacket(packet) => {
                        let now = Instant::now();
                        state.delete_plans_packets_seen += 1;
                        state.last_delete_plans = Some(packet.clone());
                        state.last_delete_plans_at = Some(now);
                        false
                    }
                    PacketKind::CommandUnitsCallPacket(packet) => {
                        let now = Instant::now();
                        state.command_units_packets_seen += 1;
                        state.last_command_units = Some(packet.clone());
                        state.last_command_units_at = Some(now);
                        false
                    }
                    PacketKind::SetUnitCommandCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_unit_command_packets_seen += 1;
                        state.last_set_unit_command = Some(packet.clone());
                        state.last_set_unit_command_at = Some(now);
                        false
                    }
                    PacketKind::SetUnitStanceCallPacket(packet) => {
                        let now = Instant::now();
                        state.set_unit_stance_packets_seen += 1;
                        state.last_set_unit_stance = Some(packet.clone());
                        state.last_set_unit_stance_at = Some(now);
                        false
                    }
                    PacketKind::BuildingControlSelectCallPacket(packet) => {
                        let now = Instant::now();
                        state.building_control_select_packets_seen += 1;
                        state.last_building_control_select = Some(packet.clone());
                        state.last_building_control_select_at = Some(now);
                        false
                    }
                    PacketKind::UnitBuildingControlSelectCallPacket(packet) => {
                        let now = Instant::now();
                        state.unit_building_control_select_packets_seen += 1;
                        state.last_unit_building_control_select = Some(packet.clone());
                        state.last_unit_building_control_select_at = Some(now);
                        false
                    }
                    PacketKind::UnitControlCallPacket(packet) => {
                        let now = Instant::now();
                        state.unit_control_packets_seen += 1;
                        state.last_unit_control = Some(packet.clone());
                        state.last_unit_control_at = Some(now);
                        false
                    }
                    PacketKind::UnitClearCallPacket(packet) => {
                        let now = Instant::now();
                        state.unit_clear_packets_seen += 1;
                        state.last_unit_clear = Some(packet.clone());
                        state.last_unit_clear_at = Some(now);
                        false
                    }
                    PacketKind::RemoveQueueBlockCallPacket(packet) => {
                        let now = Instant::now();
                        state.remove_queue_block_packets_seen += 1;
                        state.last_remove_queue_block = Some(*packet);
                        state.last_remove_queue_block_at = Some(now);
                        false
                    }
                    PacketKind::TileConfigCallPacket(packet) => {
                        let now = Instant::now();
                        state.tile_config_packets_seen += 1;
                        state.last_tile_config = Some(packet.clone());
                        state.last_tile_config_at = Some(now);
                        false
                    }
                    PacketKind::RotateBlockCallPacket(packet) => {
                        let now = Instant::now();
                        state.rotate_block_packets_seen += 1;
                        state.last_rotate_block = Some(packet.clone());
                        state.last_rotate_block_at = Some(now);
                        false
                    }
                    PacketKind::TileTapCallPacket(packet) => {
                        let now = Instant::now();
                        state.tile_tap_packets_seen += 1;
                        state.last_tile_tap = Some(packet.clone());
                        state.last_tile_tap_at = Some(now);
                        false
                    }
                    _ => false,
                }
            };

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
        state.timeout_disconnects += 1;
        state.last_update_at = Some(Instant::now());
        state.connect_packet_sent = false;
        state.connect_confirm_sent = false;
        state.last_connect_confirm_error = None;
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
    use std::io;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use crate::mindustry::entities::comp::{PlayerComp, PlayerUnitState, UnitComp};
    use crate::mindustry::entities::units::BuildPlan;
    use crate::mindustry::io::UnitRef;
    use crate::mindustry::io::{BuildPlanWire, BuildingRef, EntityRef, TeamId, TypeValue, Vec2};
    use crate::mindustry::net::{
        BuildingControlSelectCallPacket, ClearItemsCallPacket, ClearLiquidsCallPacket,
        ClientPlanSnapshotCallPacket, ClientPlanSnapshotReceivedCallPacket,
        ClientSnapshotCallPacket, CommandUnitsCallPacket, Connect, DeletePlansCallPacket,
        Disconnect, DoneCallback, EntitySnapshotCallPacket, HiddenSnapshotCallPacket, Host,
        HostCallback, Net, NetConnection, NetProvider, PacketKind, PayloadDroppedCallPacket,
        PickedBuildPayloadCallPacket, PickedUnitPayloadCallPacket, PingLocationCallPacket,
        PingResponseCallPacket, RemoveQueueBlockCallPacket, RequestBuildPayloadCallPacket,
        RequestDropPayloadCallPacket, RequestItemCallPacket, RequestUnitPayloadCallPacket,
        RotateBlockCallPacket, SetItemCallPacket, SetItemsCallPacket, SetLiquidCallPacket,
        SetLiquidsCallPacket, SetUnitCommandCallPacket, SetUnitStanceCallPacket,
        StateSnapshotCallPacket, StreamBegin, StreamChunk, Streamable, TileConfigCallPacket,
        TileTapCallPacket, TransferInventoryCallPacket, UnitBuildingControlSelectCallPacket,
        UnitClearCallPacket, UnitControlCallPacket, UnitEnteredPayloadCallPacket,
        WorldDataBeginCallPacket,
    };
    use crate::mindustry::r#type::{ItemStack, LiquidStack, UnitType};
    use crate::mindustry::world::block::Block;

    use super::{
        ClientCameraView, ClientConnectConfig, ClientInputSnapshot, NetClient,
        CLIENT_PLAN_PREVIEW_CHUNK_SIZE,
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
        assert_eq!(state.entity_snapshot_packets_seen, 0);
        assert!(state.last_entity_snapshot.is_none());
        assert_eq!(state.hidden_snapshot_packets_seen, 0);
        assert!(state.last_hidden_snapshot.is_none());
        assert!(state.last_server_snapshot_at.is_none());
        assert!(state.last_packet.is_none());
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

        let queued_normal_packets = Arc::new(Mutex::new(0));
        let queued_normal_packets_handler = Arc::clone(&queued_normal_packets);
        client.add_packet_handler(move |packet| {
            if matches!(packet, PacketKind::WorldDataBeginCallPacket(_)) {
                *queued_normal_packets_handler.lock().unwrap() += 1;
            }
        });

        {
            let mut net = client.net_mut();
            net.handle_client_received(PacketKind::WorldDataBeginCallPacket(
                WorldDataBeginCallPacket,
            ));
            net.handle_client_received(PacketKind::Streamable(Streamable::new(vec![1, 2, 3])));
            net.handle_client_received(PacketKind::Streamable(Streamable::new(vec![4, 5, 6])));
        }

        client.update();
        client.update();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert!(sent[0].1);
        assert!(matches!(sent[0].0, PacketKind::ConnectConfirmCallPacket(_)));
        assert_eq!(*queued_normal_packets.lock().unwrap(), 1);

        let state = client.state();
        let state = state.lock().unwrap();
        assert!(!state.connecting);
        assert!(state.connected);
        assert!(state.connect_confirm_sent);
        assert!(state.last_connect_confirm_error.is_none());
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
