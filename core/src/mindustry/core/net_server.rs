use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::mindustry::entities::comp::building::{BuildingComp, BuildingConfigChange};
use crate::mindustry::io::{BuildPlanWire, EntityRef, TeamId, TypeValue};
use crate::mindustry::net::{
    packet_ids, ActionType, Administration, ClientPlanSnapshotCallPacket,
    ClientPlanSnapshotReceivedCallPacket, ClientSnapshotCallPacket, Connect, ConnectPacket,
    DebugStatusClientCallPacket, DebugStatusClientUnreliableCallPacket, Disconnect,
    EntitySnapshotCallPacket, HiddenSnapshotCallPacket, KickCallPacket, KickCallPacket2,
    KickReason, Net, NetConnection, PacketKind, PlayerAction, ProviderEvent, RotateBlockCallPacket,
    SentPacket, StateSnapshotCallPacket, SteamAdminData, Streamable, TileConfigCallPacket,
    TileTapCallPacket, WorldDataBeginCallPacket,
};

pub type PacketHandler = Arc<Mutex<Box<dyn FnMut(&PacketKind) + Send + 'static>>>;
pub type BinaryPacketHandler = Arc<Mutex<Box<dyn FnMut(&[u8]) + Send + 'static>>>;

pub const PLAN_PREVIEW_SYNC_INTERVAL_MS: i64 = 500;
pub const PLAN_PREVIEW_CHUNK_SIZE: usize = 900 / 12;
pub const JAVA_MAX_NAME_BYTES: usize = 40;

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerPreviewPlanSource {
    pub player_id: i32,
    pub team: TeamId,
    pub connection_id: Option<i32>,
    pub local: bool,
    pub connected: bool,
    pub last_preview_plan_group_server: i32,
    pub plans: Vec<BuildPlanWire>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreviewPlanBroadcast {
    pub target_connection_id: i32,
    pub packet: ClientPlanSnapshotReceivedCallPacket,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectPacketValidationContext {
    pub connection_address: String,
    pub connection_kicked: bool,
    pub connection_connected: bool,
    pub has_begun_connecting: bool,
    pub ip_banned: bool,
    pub subnet_banned: bool,
    pub id_banned: bool,
    pub name_banned: bool,
    pub recent_kick_active: bool,
    pub player_limit_reached: bool,
    pub whitelisted: bool,
    pub server_version: i32,
    pub server_version_type: String,
    pub allows_custom_clients: bool,
    pub prevent_duplicates: bool,
    pub duplicate_name: bool,
    pub duplicate_id: bool,
    pub duplicate_connection_uuid: bool,
}

impl Default for ConnectPacketValidationContext {
    fn default() -> Self {
        Self {
            connection_address: String::new(),
            connection_kicked: false,
            connection_connected: true,
            has_begun_connecting: false,
            ip_banned: false,
            subnet_banned: false,
            id_banned: false,
            name_banned: false,
            recent_kick_active: false,
            player_limit_reached: false,
            whitelisted: true,
            server_version: 157,
            server_version_type: "official".into(),
            allows_custom_clients: false,
            prevent_duplicates: false,
            duplicate_name: false,
            duplicate_id: false,
            duplicate_connection_uuid: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectPacketDecision {
    Ignore,
    Kick(KickReason),
    Accept,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectPacketValidationPlan {
    pub decision: ConnectPacketDecision,
    pub normalized_uuid: String,
    pub normalized_name: String,
    pub locale: String,
    pub mobile: bool,
    pub mark_begun_connecting: bool,
    pub mod_client: bool,
}

impl ConnectPacketValidationPlan {
    pub fn accepted(&self) -> bool {
        self.decision == ConnectPacketDecision::Accept
    }

    pub fn kick_reason(&self) -> Option<KickReason> {
        match self.decision {
            ConnectPacketDecision::Kick(reason) => Some(reason),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NetServerState {
    pub active: bool,
    pub server: bool,
    pub administration: Administration,
    pub steam_admin_data: SteamAdminData,
    pub player_limit: i32,
    pub listen_port: Option<u16>,
    pub connections: Vec<NetConnection>,
    pub connection_states: HashMap<i32, NetConnection>,
    pub last_connection_id: Option<i32>,
    pub last_connect: Option<Connect>,
    pub last_handshake: Option<ConnectPacket>,
    pub last_connect_validation_connection_id: Option<i32>,
    pub last_connect_validation_plan: Option<ConnectPacketValidationPlan>,
    pub connect_packets_accepted: u64,
    pub connect_packets_rejected: u64,
    pub pending_connect_kicks: Vec<(i32, KickReason)>,
    pub pending_world_data_connections: Vec<i32>,
    pub last_connect_confirm_connection_id: Option<i32>,
    pub last_world_data_connection_id: Option<i32>,
    pub last_world_data_bytes: Option<usize>,
    pub world_data_begin_sent: u64,
    pub world_streams_sent: u64,
    pub last_world_data_error: Option<String>,
    pub last_kick_connection_id: Option<i32>,
    pub last_kick_reason: Option<KickReason>,
    pub last_kick_message: Option<String>,
    pub kick_packets_sent: u64,
    pub last_kick_error: Option<String>,
    pub last_ping_connection_id: Option<i32>,
    pub last_ping_time: Option<i64>,
    pub ping_requests_seen: u64,
    pub ping_responses_sent: u64,
    pub last_ping_error: Option<String>,
    pub last_debug_status_connection_id: Option<i32>,
    pub last_debug_status: Option<DebugStatusClientCallPacket>,
    pub debug_status_requests_seen: u64,
    pub debug_status_responses_sent: u64,
    pub pending_debug_status_connections: Vec<i32>,
    pub last_debug_status_error: Option<String>,
    pub last_client_snapshot_connection_id: Option<i32>,
    pub last_client_snapshot: Option<ClientSnapshotCallPacket>,
    pub last_client_snapshot_received_at: Option<Instant>,
    pub client_snapshot_packets_seen: u64,
    pub last_client_plan_snapshot_connection_id: Option<i32>,
    pub last_client_plan_snapshot: Option<ClientPlanSnapshotCallPacket>,
    pub last_client_plan_snapshot_received_at: Option<Instant>,
    pub client_plan_snapshot_packets_seen: u64,
    pub last_client_plan_snapshot_forwarded_connection_id: Option<i32>,
    pub last_client_plan_snapshot_forwarded_at: Option<Instant>,
    pub client_plan_snapshots_forwarded: u64,
    pub last_client_plan_snapshot_forwarded_error: Option<String>,
    pub last_tile_config_connection_id: Option<i32>,
    pub last_tile_config: Option<TileConfigCallPacket>,
    pub last_tile_config_received_at: Option<Instant>,
    pub tile_config_packets_seen: u64,
    pub tile_config_rejections: u64,
    pub last_tile_config_rollback_connection_id: Option<i32>,
    pub last_tile_config_rollback: Option<TileConfigCallPacket>,
    pub last_tile_config_rollback_at: Option<Instant>,
    pub tile_config_rollbacks_sent: u64,
    pub last_tile_config_rollback_error: Option<String>,
    pub last_rotate_block_connection_id: Option<i32>,
    pub last_rotate_block: Option<RotateBlockCallPacket>,
    pub last_rotate_block_received_at: Option<Instant>,
    pub rotate_block_packets_seen: u64,
    pub last_tile_tap_connection_id: Option<i32>,
    pub last_tile_tap: Option<TileTapCallPacket>,
    pub last_tile_tap_received_at: Option<Instant>,
    pub tile_tap_packets_seen: u64,
    pub last_state_snapshot_connection_id: Option<i32>,
    pub last_state_snapshot: Option<StateSnapshotCallPacket>,
    pub last_state_snapshot_sent_at: Option<Instant>,
    pub state_snapshot_packets_sent: u64,
    pub last_state_snapshot_error: Option<String>,
    pub last_entity_snapshot_connection_id: Option<i32>,
    pub last_entity_snapshot: Option<EntitySnapshotCallPacket>,
    pub last_entity_snapshot_sent_at: Option<Instant>,
    pub entity_snapshot_packets_sent: u64,
    pub last_entity_snapshot_error: Option<String>,
    pub last_hidden_snapshot_connection_id: Option<i32>,
    pub last_hidden_snapshot: Option<HiddenSnapshotCallPacket>,
    pub last_hidden_snapshot_sent_at: Option<Instant>,
    pub hidden_snapshot_packets_sent: u64,
    pub last_hidden_snapshot_error: Option<String>,
    pub last_disconnect: Option<Disconnect>,
    pub last_disconnect_reason: Option<String>,
    pub events: Vec<ProviderEvent>,
    pub last_updated_at: Option<Instant>,
}

#[derive(Clone)]
pub struct NetServer {
    net: Arc<Mutex<Net>>,
    state: Arc<Mutex<NetServerState>>,
    packet_handlers: Arc<Mutex<Vec<PacketHandler>>>,
    binary_packet_handlers: Arc<Mutex<Vec<BinaryPacketHandler>>>,
}

impl std::fmt::Debug for NetServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.lock().expect("NetServerState mutex poisoned");
        f.debug_struct("NetServer")
            .field("active", &state.active)
            .field("server", &state.server)
            .field("listen_port", &state.listen_port)
            .field("connections", &state.connections.len())
            .field(
                "packet_handlers",
                &self.packet_handlers.lock().map(|v| v.len()).unwrap_or(0),
            )
            .field(
                "binary_packet_handlers",
                &self
                    .binary_packet_handlers
                    .lock()
                    .map(|v| v.len())
                    .unwrap_or(0),
            )
            .finish()
    }
}

impl Default for NetServer {
    fn default() -> Self {
        Self::new(Net::default())
    }
}

impl NetServer {
    pub fn new(net: Net) -> Self {
        let state = Arc::new(Mutex::new(NetServerState::default()));
        let packet_handlers = Arc::new(Mutex::new(Vec::new()));
        let binary_packet_handlers = Arc::new(Mutex::new(Vec::new()));

        let net = Arc::new(Mutex::new(net));
        {
            let mut net = net.lock().expect("Net mutex poisoned");
            Self::install_typed_listeners(&mut net, &state, &packet_handlers);
        }

        Self {
            net,
            state,
            packet_handlers,
            binary_packet_handlers,
        }
    }

    pub fn net(&self) -> Arc<Mutex<Net>> {
        Arc::clone(&self.net)
    }

    pub fn net_mut(&self) -> MutexGuard<'_, Net> {
        self.net.lock().expect("Net mutex poisoned")
    }

    pub fn state(&self) -> Arc<Mutex<NetServerState>> {
        Arc::clone(&self.state)
    }

    pub fn open(&self, port: u16) -> io::Result<()> {
        if self.is_active() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "server already active",
            ));
        }

        let mut net = self.net.lock().expect("Net mutex poisoned");
        net.host(port)?;
        let connections = net.get_connections();
        drop(net);

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        state.active = true;
        state.server = true;
        state.listen_port = Some(port);
        state.connections = connections;
        state.last_updated_at = Some(Instant::now());
        Ok(())
    }

    pub fn close(&self) {
        if self.is_active() {
            let mut net = self.net.lock().expect("Net mutex poisoned");
            net.close_server();
        }

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        state.active = false;
        state.server = false;
        state.listen_port = None;
        state.connections.clear();
        state.connection_states.clear();
        state.last_updated_at = Some(Instant::now());
    }

    pub fn is_active(&self) -> bool {
        self.state
            .lock()
            .expect("NetServerState mutex poisoned")
            .active
    }

    pub fn is_server(&self) -> bool {
        self.state
            .lock()
            .expect("NetServerState mutex poisoned")
            .server
    }

    pub fn add_packet_handler<F>(&self, handler: F)
    where
        F: FnMut(&PacketKind) + Send + 'static,
    {
        let handler = Arc::new(Mutex::new(
            Box::new(handler) as Box<dyn FnMut(&PacketKind) + Send + 'static>
        ));
        self.packet_handlers
            .lock()
            .expect("packet handlers mutex poisoned")
            .push(handler);
    }

    pub fn get_packet_handlers(&self) -> Vec<PacketHandler> {
        self.packet_handlers
            .lock()
            .expect("packet handlers mutex poisoned")
            .clone()
    }

    pub fn add_binary_packet_handler<F>(&self, handler: F)
    where
        F: FnMut(&[u8]) + Send + 'static,
    {
        let handler = Arc::new(Mutex::new(
            Box::new(handler) as Box<dyn FnMut(&[u8]) + Send + 'static>
        ));
        self.binary_packet_handlers
            .lock()
            .expect("binary packet handlers mutex poisoned")
            .push(handler);
    }

    pub fn get_binary_packet_handlers(&self) -> Vec<BinaryPacketHandler> {
        self.binary_packet_handlers
            .lock()
            .expect("binary packet handlers mutex poisoned")
            .clone()
    }

    pub fn send_world_data_begin(&self, connection_id: i32) -> io::Result<()> {
        let packet = PacketKind::WorldDataBeginCallPacket(WorldDataBeginCallPacket);
        let result = {
            let mut net = self.net.lock().expect("Net mutex poisoned");
            net.send_to(connection_id, &packet, true)
        };

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        state.last_world_data_connection_id = Some(connection_id);
        state.last_updated_at = Some(Instant::now());
        match &result {
            Ok(()) => {
                state.world_data_begin_sent += 1;
                state.last_world_data_error = None;
                state
                    .connection_states
                    .entry(connection_id)
                    .or_insert_with(|| NetConnection::new(String::new()))
                    .send(SentPacket::Other("WorldDataBeginCallPacket".into()), true);
            }
            Err(error) => {
                state.last_world_data_error = Some(error.to_string());
            }
        }
        result
    }

    pub fn send_world_data(&self, connection_id: i32, bytes: impl Into<Vec<u8>>) -> io::Result<()> {
        let bytes = bytes.into();
        let byte_len = bytes.len();
        let send_plan = Self::world_stream_send_plan(bytes);

        let mut first_error = None;
        {
            let mut net = self.net.lock().expect("Net mutex poisoned");
            for (packet, _, reliable) in &send_plan {
                if let Err(error) = net.send_to(connection_id, packet, *reliable) {
                    first_error = Some(error);
                    break;
                }
            }
        }

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        state.last_world_data_connection_id = Some(connection_id);
        state.last_world_data_bytes = Some(byte_len);
        state.last_updated_at = Some(Instant::now());

        if let Some(error) = first_error {
            state.last_world_data_error = Some(error.to_string());
            return Err(error);
        }

        state.world_streams_sent += 1;
        state.last_world_data_error = None;
        let connection = state
            .connection_states
            .entry(connection_id)
            .or_insert_with(|| NetConnection::new(String::new()));
        connection.sent.extend(
            send_plan
                .into_iter()
                .map(|(_, sent_packet, reliable)| (sent_packet, reliable)),
        );
        Ok(())
    }

    pub fn kick_connection_reason(&self, connection_id: i32, reason: KickReason) -> io::Result<()> {
        let packet = PacketKind::KickCallPacket2(KickCallPacket2 { reason });
        let result = {
            let mut net = self.net.lock().expect("Net mutex poisoned");
            net.send_to(connection_id, &packet, true)
        };

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        state.last_kick_connection_id = Some(connection_id);
        state.last_kick_reason = Some(reason);
        state.last_kick_message = None;
        state.last_updated_at = Some(Instant::now());
        match &result {
            Ok(()) => {
                state.kick_packets_sent += 1;
                state.last_kick_error = None;
                state
                    .connection_states
                    .entry(connection_id)
                    .or_insert_with(|| NetConnection::new(String::new()))
                    .kick_reason(reason);
            }
            Err(error) => {
                state.last_kick_error = Some(error.to_string());
            }
        }
        result
    }

    pub fn kick_connection_message(
        &self,
        connection_id: i32,
        reason: impl Into<String>,
    ) -> io::Result<()> {
        let reason = reason.into();
        let packet = PacketKind::KickCallPacket(KickCallPacket {
            reason: reason.clone(),
        });
        let result = {
            let mut net = self.net.lock().expect("Net mutex poisoned");
            net.send_to(connection_id, &packet, true)
        };

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        state.last_kick_connection_id = Some(connection_id);
        state.last_kick_reason = None;
        state.last_kick_message = Some(reason.clone());
        state.last_updated_at = Some(Instant::now());
        match &result {
            Ok(()) => {
                state.kick_packets_sent += 1;
                state.last_kick_error = None;
                state
                    .connection_states
                    .entry(connection_id)
                    .or_insert_with(|| NetConnection::new(String::new()))
                    .kick_message(reason);
            }
            Err(error) => {
                state.last_kick_error = Some(error.to_string());
            }
        }
        result
    }

    pub fn resend_world_data(
        &self,
        connection_id: i32,
        bytes: impl Into<Vec<u8>>,
    ) -> io::Result<()> {
        self.send_world_data_begin(connection_id)?;
        self.send_world_data(connection_id, bytes)
    }

    pub fn send_client_plan_snapshot_received(
        &self,
        connection_id: i32,
        packet: ClientPlanSnapshotReceivedCallPacket,
    ) -> io::Result<()> {
        let packet = PacketKind::ClientPlanSnapshotReceivedCallPacket(packet);
        let result = {
            let mut net = self.net.lock().expect("Net mutex poisoned");
            net.send_to(connection_id, &packet, false)
        };

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        state.last_client_plan_snapshot_forwarded_connection_id = Some(connection_id);
        state.last_client_plan_snapshot_forwarded_at = Some(Instant::now());
        match &result {
            Ok(()) => {
                state.client_plan_snapshots_forwarded += 1;
                state.last_client_plan_snapshot_forwarded_error = None;
                Self::record_connection_sent(
                    &mut state,
                    connection_id,
                    "ClientPlanSnapshotReceivedCallPacket",
                    false,
                );
            }
            Err(error) => {
                state.last_client_plan_snapshot_forwarded_error = Some(error.to_string());
            }
        }
        result
    }

    pub fn send_client_plan_snapshot_received_to_many(
        &self,
        connection_ids: impl IntoIterator<Item = i32>,
        packet: ClientPlanSnapshotReceivedCallPacket,
    ) -> io::Result<usize> {
        let mut sent = 0;
        let mut first_error = None;

        for connection_id in connection_ids {
            match self.send_client_plan_snapshot_received(connection_id, packet.clone()) {
                Ok(()) => sent += 1,
                Err(error) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }

        if let Some(error) = first_error {
            let mut state = self.state.lock().expect("NetServerState mutex poisoned");
            state.last_client_plan_snapshot_forwarded_error = Some(error.to_string());
            Err(error)
        } else {
            Ok(sent)
        }
    }

    pub fn broadcast_client_plan_previews(
        &self,
        players: &mut [PlayerPreviewPlanSource],
    ) -> io::Result<usize> {
        let broadcasts = Self::client_plan_preview_broadcasts(players);
        let mut sent = 0;
        let mut first_error = None;

        for broadcast in broadcasts {
            match self.send_client_plan_snapshot_received(
                broadcast.target_connection_id,
                broadcast.packet,
            ) {
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

    pub fn send_tile_config_rollback(
        &self,
        connection_id: i32,
        packet: TileConfigCallPacket,
    ) -> io::Result<()> {
        let packet_kind = PacketKind::TileConfigCallPacket(packet.clone());
        let result = {
            let mut net = self.net.lock().expect("Net mutex poisoned");
            net.send_to(connection_id, &packet_kind, true)
        };

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        let now = Instant::now();
        state.last_tile_config_rollback_connection_id = Some(connection_id);
        state.last_tile_config_rollback = Some(packet);
        state.last_tile_config_rollback_at = Some(now);
        state.last_updated_at = Some(now);
        match &result {
            Ok(()) => {
                state.tile_config_rollbacks_sent += 1;
                state.last_tile_config_rollback_error = None;
                Self::record_connection_sent(
                    &mut state,
                    connection_id,
                    "TileConfigCallPacket",
                    true,
                );
            }
            Err(error) => {
                state.last_tile_config_rollback_error = Some(error.to_string());
            }
        }
        result
    }

    pub fn handle_tile_config_authority<F>(
        &self,
        connection_id: Option<i32>,
        player: EntityRef,
        build: &mut BuildingComp,
        value: TypeValue,
        accepts: F,
    ) -> io::Result<BuildingConfigChange>
    where
        F: FnOnce(&TypeValue) -> bool,
    {
        let change = build.configure_any_checked(value, accepts);

        if !change.accepted {
            let mut state = self.state.lock().expect("NetServerState mutex poisoned");
            state.tile_config_rejections += 1;
            state.last_updated_at = Some(Instant::now());
            drop(state);

            if let (Some(connection_id), Some(rollback)) = (connection_id, change.rollback.clone())
            {
                let packet = TileConfigCallPacket::rollback_for(player, rollback);
                self.send_tile_config_rollback(connection_id, packet)?;
            }
        }

        Ok(change)
    }

    pub fn send_state_snapshot(
        &self,
        connection_id: i32,
        packet: StateSnapshotCallPacket,
    ) -> io::Result<()> {
        let packet_kind = PacketKind::StateSnapshotCallPacket(packet.clone());
        let result = {
            let mut net = self.net.lock().expect("Net mutex poisoned");
            net.send_to(connection_id, &packet_kind, false)
        };

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        let now = Instant::now();
        state.last_state_snapshot_connection_id = Some(connection_id);
        state.last_state_snapshot = Some(packet);
        state.last_state_snapshot_sent_at = Some(now);
        state.last_updated_at = Some(now);
        match &result {
            Ok(()) => {
                state.state_snapshot_packets_sent += 1;
                state.last_state_snapshot_error = None;
                Self::record_connection_sent(
                    &mut state,
                    connection_id,
                    "StateSnapshotCallPacket",
                    false,
                );
            }
            Err(error) => {
                state.last_state_snapshot_error = Some(error.to_string());
            }
        }
        result
    }

    pub fn send_entity_snapshot(
        &self,
        connection_id: i32,
        packet: EntitySnapshotCallPacket,
    ) -> io::Result<()> {
        let packet_kind = PacketKind::EntitySnapshotCallPacket(packet.clone());
        let result = {
            let mut net = self.net.lock().expect("Net mutex poisoned");
            net.send_to(connection_id, &packet_kind, false)
        };

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        let now = Instant::now();
        state.last_entity_snapshot_connection_id = Some(connection_id);
        state.last_entity_snapshot = Some(packet);
        state.last_entity_snapshot_sent_at = Some(now);
        state.last_updated_at = Some(now);
        match &result {
            Ok(()) => {
                state.entity_snapshot_packets_sent += 1;
                state.last_entity_snapshot_error = None;
                Self::record_connection_sent(
                    &mut state,
                    connection_id,
                    "EntitySnapshotCallPacket",
                    false,
                );
            }
            Err(error) => {
                state.last_entity_snapshot_error = Some(error.to_string());
            }
        }
        result
    }

    pub fn send_hidden_snapshot(
        &self,
        connection_id: i32,
        packet: HiddenSnapshotCallPacket,
    ) -> io::Result<()> {
        let packet_kind = PacketKind::HiddenSnapshotCallPacket(packet.clone());
        let result = {
            let mut net = self.net.lock().expect("Net mutex poisoned");
            net.send_to(connection_id, &packet_kind, false)
        };

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        let now = Instant::now();
        state.last_hidden_snapshot_connection_id = Some(connection_id);
        state.last_hidden_snapshot = Some(packet);
        state.last_hidden_snapshot_sent_at = Some(now);
        state.last_updated_at = Some(now);
        match &result {
            Ok(()) => {
                state.hidden_snapshot_packets_sent += 1;
                state.last_hidden_snapshot_error = None;
                Self::record_connection_sent(
                    &mut state,
                    connection_id,
                    "HiddenSnapshotCallPacket",
                    false,
                );
            }
            Err(error) => {
                state.last_hidden_snapshot_error = Some(error.to_string());
            }
        }
        result
    }

    pub fn send_entity_sync_snapshot(
        &self,
        connection_id: i32,
        state_snapshot: StateSnapshotCallPacket,
        entity_snapshots: impl IntoIterator<Item = EntitySnapshotCallPacket>,
        hidden_snapshot: Option<HiddenSnapshotCallPacket>,
    ) -> io::Result<()> {
        self.send_state_snapshot(connection_id, state_snapshot)?;
        for snapshot in entity_snapshots {
            self.send_entity_snapshot(connection_id, snapshot)?;
        }
        if let Some(snapshot) = hidden_snapshot {
            self.send_hidden_snapshot(connection_id, snapshot)?;
        }

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        let connection = state
            .connection_states
            .entry(connection_id)
            .or_insert_with(|| NetConnection::new(String::new()));
        connection.snapshots_sent = connection.snapshots_sent.saturating_add(1);
        state.last_updated_at = Some(Instant::now());
        Ok(())
    }

    pub fn update(&self) {
        self.flush_pending_connect_kicks();
        self.flush_pending_debug_status();

        let connections = {
            let net = self.net.lock().expect("Net mutex poisoned");
            net.get_connections()
        };

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        Self::sync_provider_connections(&mut state, connections);
        state.last_updated_at = Some(Instant::now());
    }

    pub fn take_pending_world_data_connections(&self) -> Vec<i32> {
        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        std::mem::take(&mut state.pending_world_data_connections)
    }

    pub fn send_pending_world_data<F>(&self, mut world_data: F) -> io::Result<usize>
    where
        F: FnMut(i32) -> Vec<u8>,
    {
        let pending = self.take_pending_world_data_connections();
        let mut sent = 0;
        let mut first_error = None;
        let mut unsent = Vec::new();

        for connection_id in pending {
            if first_error.is_some() {
                unsent.push(connection_id);
                continue;
            }

            match self.send_world_data(connection_id, world_data(connection_id)) {
                Ok(()) => sent += 1,
                Err(error) => {
                    unsent.push(connection_id);
                    first_error = Some(error);
                }
            }
        }

        if !unsent.is_empty() {
            let mut state = self.state.lock().expect("NetServerState mutex poisoned");
            let mut restored = unsent;
            restored.extend(std::mem::take(&mut state.pending_world_data_connections));
            state.pending_world_data_connections = restored;
        }

        if let Some(error) = first_error {
            Err(error)
        } else {
            Ok(sent)
        }
    }

    pub fn debug_status_for_connection(
        state: &NetServerState,
        connection_id: i32,
    ) -> DebugStatusClientCallPacket {
        let Some(connection) = state.connection_states.get(&connection_id) else {
            return DebugStatusClientCallPacket {
                value: 0,
                last_client_snapshot: -1,
                snapshots_sent: 0,
            };
        };

        let value = (if connection.has_disconnected { 1 } else { 0 })
            | (if connection.has_connected { 2 } else { 0 })
            | (if connection.player_added { 4 } else { 0 })
            | (if connection.has_begun_connecting {
                8
            } else {
                0
            });

        DebugStatusClientCallPacket {
            value,
            last_client_snapshot: connection.last_received_client_snapshot,
            snapshots_sent: connection.snapshots_sent,
        }
    }

    pub fn send_debug_status(&self, connection_id: i32) -> io::Result<()> {
        let packet = {
            let state = self.state.lock().expect("NetServerState mutex poisoned");
            Self::debug_status_for_connection(&state, connection_id)
        };

        let reliable = PacketKind::DebugStatusClientCallPacket(packet);
        let unreliable = PacketKind::DebugStatusClientUnreliableCallPacket(
            DebugStatusClientUnreliableCallPacket(packet),
        );

        let mut first_error = None;
        {
            let mut net = self.net.lock().expect("Net mutex poisoned");
            if let Err(error) = net.send_to(connection_id, &reliable, true) {
                first_error = Some(error);
            } else if let Err(error) = net.send_to(connection_id, &unreliable, false) {
                first_error = Some(error);
            }
        }

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        state.last_debug_status_connection_id = Some(connection_id);
        state.last_debug_status = Some(packet);
        state.last_updated_at = Some(Instant::now());

        if let Some(error) = first_error {
            state.last_debug_status_error = Some(error.to_string());
            return Err(error);
        }

        state.debug_status_responses_sent = state.debug_status_responses_sent.saturating_add(2);
        state.last_debug_status_error = None;
        Self::record_connection_sent(
            &mut state,
            connection_id,
            "DebugStatusClientCallPacket",
            true,
        );
        Self::record_connection_sent(
            &mut state,
            connection_id,
            "DebugStatusClientUnreliableCallPacket",
            false,
        );
        Ok(())
    }

    fn flush_pending_debug_status(&self) {
        let pending = {
            let mut state = self.state.lock().expect("NetServerState mutex poisoned");
            std::mem::take(&mut state.pending_debug_status_connections)
        };

        for connection_id in pending {
            let _ = self.send_debug_status(connection_id);
        }
    }

    fn flush_pending_connect_kicks(&self) {
        let pending = {
            let mut state = self.state.lock().expect("NetServerState mutex poisoned");
            std::mem::take(&mut state.pending_connect_kicks)
        };

        for (connection_id, reason) in pending {
            let _ = self.kick_connection_reason(connection_id, reason);
        }
    }

    fn world_stream_send_plan(bytes: Vec<u8>) -> Vec<(PacketKind, SentPacket, bool)> {
        let mut connection = NetConnection::new(String::new());
        connection.send_stream(Streamable::new(bytes), packet_ids::WORLD_STREAM);
        connection
            .sent
            .into_iter()
            .filter_map(|(sent_packet, reliable)| {
                let packet = match &sent_packet {
                    SentPacket::StreamBegin(begin) => PacketKind::StreamBegin(begin.clone()),
                    SentPacket::StreamChunk(chunk) => PacketKind::StreamChunk(chunk.clone()),
                    _ => return None,
                };
                Some((packet, sent_packet, reliable))
            })
            .collect()
    }

    fn connect_packet_validation_context(
        state: &NetServerState,
        connection_id: i32,
        packet: &ConnectPacket,
    ) -> ConnectPacketValidationContext {
        let connection = state.connection_states.get(&connection_id);
        let connection_address = connection
            .map(|connection| connection.address.clone())
            .unwrap_or_default();
        let normalized_uuid = if connection_address.starts_with("steam:") {
            connection_address
                .strip_prefix("steam:")
                .unwrap_or(&connection_address)
                .to_string()
        } else {
            packet.uuid.clone()
        };
        let normalized_name = Self::fix_name_like_java(&packet.name);
        let duplicate_name_key = normalized_name.trim().to_ascii_lowercase();

        let mut context = ConnectPacketValidationContext {
            connection_address,
            connection_kicked: connection
                .map(|connection| connection.kicked)
                .unwrap_or_default(),
            connection_connected: connection
                .map(|connection| connection.is_connected())
                .unwrap_or(true),
            has_begun_connecting: connection
                .map(|connection| connection.has_begun_connecting)
                .unwrap_or_default(),
            duplicate_name: false,
            duplicate_id: false,
            duplicate_connection_uuid: false,
            ..Default::default()
        };

        context.ip_banned = state
            .administration
            .is_ip_banned(&context.connection_address);
        context.subnet_banned = state
            .administration
            .is_subnet_banned(&context.connection_address);
        context.id_banned = state.administration.is_id_banned(&normalized_uuid)
            || state
                .steam_admin_data
                .is_banned(&context.connection_address);
        context.name_banned = state.administration.is_name_banned(&packet.name);
        context.recent_kick_active = state.administration.is_recently_kicked(
            &normalized_uuid,
            &context.connection_address,
            Self::current_millis(),
        );
        context.whitelisted = state
            .administration
            .is_whitelisted(&normalized_uuid, &packet.usid);
        let is_admin = state
            .administration
            .is_admin(&normalized_uuid, &packet.usid)
            || state.steam_admin_data.is_admin(&context.connection_address);
        let connected_players = state
            .connection_states
            .values()
            .filter(|connection| {
                connection.player_added && !connection.kicked && !connection.has_disconnected
            })
            .count() as i32;
        context.player_limit_reached =
            state.player_limit > 0 && connected_players >= state.player_limit && !is_admin;

        for (other_connection_id, other) in &state.connection_states {
            if *other_connection_id == connection_id || other.kicked || other.has_disconnected {
                continue;
            }

            if !duplicate_name_key.is_empty()
                && !other.name.trim().is_empty()
                && other.name.trim().to_ascii_lowercase() == duplicate_name_key
            {
                context.duplicate_name = true;
            }

            if (!normalized_uuid.is_empty() && other.uuid == normalized_uuid)
                || (!packet.usid.is_empty() && other.usid == packet.usid)
            {
                context.duplicate_id = true;
            }

            if !normalized_uuid.is_empty() && other.uuid == normalized_uuid {
                context.duplicate_connection_uuid = true;
            }
        }

        context
    }

    fn record_server_connect_packet(
        state: &mut NetServerState,
        connection_id: Option<i32>,
        connect_packet: &ConnectPacket,
    ) -> bool {
        state.last_connection_id = connection_id;
        state.last_handshake = Some(connect_packet.clone());

        let Some(connection_id) = connection_id else {
            state.last_connect_validation_connection_id = None;
            state.last_connect_validation_plan = None;
            state.events.push(ProviderEvent::ServerPacket {
                connection_id: -1,
                packet: PacketKind::ConnectPacket(connect_packet.clone()),
            });
            state.last_updated_at = Some(Instant::now());
            return true;
        };

        let context = Self::connect_packet_validation_context(state, connection_id, connect_packet);
        let plan = Self::validate_connect_packet(connect_packet, &context);
        let decision = plan.decision;
        let kick_reason = plan.kick_reason();

        state.last_connect_validation_connection_id = Some(connection_id);
        state.last_connect_validation_plan = Some(plan.clone());

        if decision == ConnectPacketDecision::Ignore {
            state.last_updated_at = Some(Instant::now());
            return false;
        }

        {
            let connection = state
                .connection_states
                .entry(connection_id)
                .or_insert_with(|| NetConnection::new(context.connection_address.clone()));
            connection.uuid = plan.normalized_uuid.clone();
            connection.usid = connect_packet.usid.clone();
            connection.name = plan.normalized_name.clone();
            connection.locale = plan.locale.clone();
            connection.color = connect_packet.color;
            connection.mobile = plan.mobile;
            connection.modclient = plan.mod_client;
            connection.connect_time = Self::current_millis();
            if plan.mark_begun_connecting {
                connection.has_begun_connecting = true;
            }
        }

        if let Some(reason) = kick_reason {
            state.connect_packets_rejected = state.connect_packets_rejected.saturating_add(1);
            state.pending_connect_kicks.push((connection_id, reason));
        } else {
            state.connect_packets_accepted = state.connect_packets_accepted.saturating_add(1);
            state.pending_world_data_connections.push(connection_id);
        }

        state.events.push(ProviderEvent::ServerPacket {
            connection_id,
            packet: PacketKind::ConnectPacket(connect_packet.clone()),
        });
        state.last_updated_at = Some(Instant::now());
        true
    }

    pub fn validate_connect_packet(
        packet: &ConnectPacket,
        context: &ConnectPacketValidationContext,
    ) -> ConnectPacketValidationPlan {
        let normalized_uuid = if context.connection_address.starts_with("steam:") {
            context
                .connection_address
                .strip_prefix("steam:")
                .unwrap_or(&context.connection_address)
                .to_string()
        } else {
            packet.uuid.clone()
        };
        let normalized_name = Self::fix_name_like_java(&packet.name);
        let locale = if packet.locale.is_empty() {
            "en".into()
        } else {
            packet.locale.clone()
        };

        let mut plan = ConnectPacketValidationPlan {
            decision: ConnectPacketDecision::Accept,
            normalized_uuid,
            normalized_name,
            locale,
            mobile: packet.mobile,
            mark_begun_connecting: false,
            mod_client: false,
        };

        if context.connection_kicked
            || !context.connection_connected
            || context.ip_banned
            || context.subnet_banned
        {
            plan.decision = ConnectPacketDecision::Ignore;
            return plan;
        }

        if context.has_begun_connecting {
            plan.decision = ConnectPacketDecision::Kick(KickReason::IdInUse);
            return plan;
        }

        plan.mark_begun_connecting = true;

        if plan.normalized_uuid.is_empty() || packet.usid.is_empty() {
            plan.decision = ConnectPacketDecision::Kick(KickReason::IdInUse);
            return plan;
        }

        if context.id_banned || context.name_banned {
            plan.decision = ConnectPacketDecision::Kick(KickReason::Banned);
            return plan;
        }

        if context.recent_kick_active {
            plan.decision = ConnectPacketDecision::Kick(KickReason::RecentKick);
            return plan;
        }

        if context.player_limit_reached {
            plan.decision = ConnectPacketDecision::Kick(KickReason::PlayerLimit);
            return plan;
        }

        if !context.whitelisted {
            plan.decision = ConnectPacketDecision::Kick(KickReason::Whitelist);
            return plan;
        }

        if packet.version_type.is_empty()
            || ((packet.version == -1 || packet.version_type != context.server_version_type)
                && context.server_version != -1
                && !context.allows_custom_clients)
        {
            plan.decision = if packet.version_type != context.server_version_type {
                ConnectPacketDecision::Kick(KickReason::TypeMismatch)
            } else {
                ConnectPacketDecision::Kick(KickReason::CustomClient)
            };
            return plan;
        }

        if context.prevent_duplicates {
            if context.duplicate_name {
                plan.decision = ConnectPacketDecision::Kick(KickReason::NameInUse);
                return plan;
            }

            if context.duplicate_id || context.duplicate_connection_uuid {
                plan.decision = ConnectPacketDecision::Kick(KickReason::IdInUse);
                return plan;
            }
        }

        if plan.normalized_name.trim().is_empty() {
            plan.decision = ConnectPacketDecision::Kick(KickReason::NameEmpty);
            return plan;
        }

        if packet.version != context.server_version
            && context.server_version != -1
            && packet.version != -1
        {
            plan.decision = if packet.version > context.server_version {
                ConnectPacketDecision::Kick(KickReason::ServerOutdated)
            } else {
                ConnectPacketDecision::Kick(KickReason::ClientOutdated)
            };
            return plan;
        }

        if packet.version == -1 {
            plan.mod_client = true;
        }

        plan
    }

    pub fn fix_name_like_java(name: &str) -> String {
        let name = name.trim().replace(['\n', '\t'], "");
        if name == "[" || name == "]" {
            return String::new();
        }

        let mut result = String::new();
        for ch in name.chars() {
            let mut candidate = result.clone();
            candidate.push(ch);
            if candidate.len() > JAVA_MAX_NAME_BYTES {
                break;
            }
            result.push(ch);
        }
        result
    }

    pub fn client_plan_preview_broadcasts(
        players: &mut [PlayerPreviewPlanSource],
    ) -> Vec<PreviewPlanBroadcast> {
        let mut broadcasts = Vec::new();

        for source_index in 0..players.len() {
            players[source_index].last_preview_plan_group_server = players[source_index]
                .last_preview_plan_group_server
                .saturating_add(1);
            let group_id = players[source_index].last_preview_plan_group_server;
            let player_id = players[source_index].player_id;
            let team = players[source_index].team;
            let plans = players[source_index].plans.clone();

            let payloads = if plans.is_empty() {
                vec![None]
            } else {
                plans
                    .chunks(PLAN_PREVIEW_CHUNK_SIZE)
                    .map(|chunk| Some(chunk.to_vec()))
                    .collect()
            };

            for (target_index, target) in players.iter().enumerate() {
                if target_index == source_index
                    || target.team != team
                    || target.local
                    || !target.connected
                {
                    continue;
                }

                let Some(target_connection_id) = target.connection_id else {
                    continue;
                };

                broadcasts.extend(payloads.iter().cloned().map(|plans| PreviewPlanBroadcast {
                    target_connection_id,
                    packet: ClientPlanSnapshotReceivedCallPacket {
                        player_id,
                        group_id,
                        plans,
                    },
                }));
            }
        }

        broadcasts
    }

    fn install_typed_listeners(
        net: &mut Net,
        state: &Arc<Mutex<NetServerState>>,
        packet_handlers: &Arc<Mutex<Vec<PacketHandler>>>,
    ) {
        {
            let state = Arc::clone(state);
            let packet_handlers = Arc::clone(packet_handlers);
            net.handle_server_connect(move |connection_id, connect| {
                let packet = PacketKind::Connect(connect.clone());
                {
                    let mut state = state.lock().expect("NetServerState mutex poisoned");
                    state.active = true;
                    state.server = true;
                    state.last_connection_id = connection_id;
                    state.last_connect = Some(connect.clone());
                    if let Some(connection_id) = connection_id {
                        let connection = state
                            .connection_states
                            .entry(connection_id)
                            .or_insert_with(|| NetConnection::new(connect.address_tcp.clone()));
                        connection.address = connect.address_tcp.clone();
                        connection.has_connected = false;
                        connection.has_begun_connecting = false;
                    }
                    state.events.push(ProviderEvent::ServerConnected {
                        connection_id: connection_id.unwrap_or(-1),
                        address: connect.address_tcp.clone(),
                    });
                    state.last_updated_at = Some(Instant::now());
                }
                Self::dispatch_packet_handlers(&packet_handlers, &packet);
            });
        }

        {
            let state = Arc::clone(state);
            let packet_handlers = Arc::clone(packet_handlers);
            net.handle_server_disconnect(move |connection_id, disconnect| {
                let packet = PacketKind::Disconnect(disconnect.clone());
                {
                    let mut state = state.lock().expect("NetServerState mutex poisoned");
                    state.last_connection_id = connection_id;
                    state.last_disconnect = Some(disconnect.clone());
                    state.last_disconnect_reason = Some(disconnect.reason.clone());
                    if let Some(connection_id) = connection_id {
                        let connection = state
                            .connection_states
                            .entry(connection_id)
                            .or_insert_with(|| NetConnection::new(String::new()));
                        connection.has_disconnected = true;
                        connection.player_added = false;
                    }
                    state.events.push(ProviderEvent::ServerDisconnected {
                        connection_id: connection_id.unwrap_or(-1),
                        reason: disconnect.reason.clone(),
                    });
                    state.last_updated_at = Some(Instant::now());
                }
                Self::dispatch_packet_handlers(&packet_handlers, &packet);
            });
        }

        {
            let state = Arc::clone(state);
            let packet_handlers = Arc::clone(packet_handlers);
            net.handle_server_connect_packet(move |connection_id, connect_packet| {
                let packet = PacketKind::ConnectPacket(connect_packet.clone());
                let dispatch = {
                    let mut state = state.lock().expect("NetServerState mutex poisoned");
                    Self::record_server_connect_packet(&mut state, connection_id, connect_packet)
                };

                if dispatch {
                    Self::dispatch_packet_handlers(&packet_handlers, &packet);
                }
            });
        }

        {
            let state = Arc::clone(state);
            let packet_handlers = Arc::clone(packet_handlers);
            net.handle_server_connect_confirm(move |connection_id, connect_confirm| {
                let packet = PacketKind::ConnectConfirmCallPacket(*connect_confirm);
                {
                    let mut state = state.lock().expect("NetServerState mutex poisoned");
                    state.last_connection_id = connection_id;
                    state.last_connect_confirm_connection_id = connection_id;
                    if let Some(connection_id) = connection_id {
                        let connection = state
                            .connection_states
                            .entry(connection_id)
                            .or_insert_with(|| NetConnection::new(String::new()));
                        connection.has_begun_connecting = true;
                        connection.player_added = true;
                        connection.has_connected = true;
                    }
                    state.events.push(ProviderEvent::ServerPacket {
                        connection_id: connection_id.unwrap_or(-1),
                        packet: packet.clone(),
                    });
                    state.last_updated_at = Some(Instant::now());
                }
                Self::dispatch_packet_handlers(&packet_handlers, &packet);
            });
        }

        {
            let state = Arc::clone(state);
            let packet_handlers = Arc::clone(packet_handlers);
            net.handle_server(move |connection_id, packet| match packet {
                PacketKind::PingCallPacket(ping) => {
                    {
                        let mut state = state.lock().expect("NetServerState mutex poisoned");
                        state.last_connection_id = connection_id;
                        state.last_ping_connection_id = connection_id;
                        state.last_ping_time = Some(ping.time);
                        state.ping_requests_seen += 1;
                        state.last_updated_at = Some(Instant::now());
                        state.ping_responses_sent += 1;
                        state.last_ping_error = None;
                        state.events.push(ProviderEvent::ServerPacket {
                            connection_id: connection_id.unwrap_or(-1),
                            packet: PacketKind::PingCallPacket(*ping),
                        });
                    }

                    Self::dispatch_packet_handlers(
                        &packet_handlers,
                        &PacketKind::PingCallPacket(*ping),
                    );
                }
                PacketKind::RequestDebugStatusCallPacket(request) => {
                    let packet = PacketKind::RequestDebugStatusCallPacket(*request);
                    {
                        let mut state = state.lock().expect("NetServerState mutex poisoned");
                        state.last_connection_id = connection_id;
                        state.debug_status_requests_seen =
                            state.debug_status_requests_seen.saturating_add(1);
                        if let Some(connection_id) = connection_id {
                            state.pending_debug_status_connections.push(connection_id);
                        }
                        state.events.push(ProviderEvent::ServerPacket {
                            connection_id: connection_id.unwrap_or(-1),
                            packet: packet.clone(),
                        });
                        state.last_updated_at = Some(Instant::now());
                    }

                    Self::dispatch_packet_handlers(&packet_handlers, &packet);
                }
                PacketKind::ClientSnapshotCallPacket(snapshot) => {
                    let packet = PacketKind::ClientSnapshotCallPacket(snapshot.clone());
                    let accepted = {
                        let mut state = state.lock().expect("NetServerState mutex poisoned");
                        Self::record_client_snapshot(&mut state, connection_id, snapshot)
                    };

                    if !accepted {
                        return;
                    }

                    Self::dispatch_packet_handlers(&packet_handlers, &packet);
                }
                PacketKind::ClientPlanSnapshotCallPacket(snapshot) => {
                    let packet = PacketKind::ClientPlanSnapshotCallPacket(snapshot.clone());
                    {
                        let mut state = state.lock().expect("NetServerState mutex poisoned");
                        Self::record_client_plan_snapshot(&mut state, connection_id, snapshot);
                    }

                    Self::dispatch_packet_handlers(&packet_handlers, &packet);
                }
                PacketKind::TileConfigCallPacket(tile_config) => {
                    let packet = PacketKind::TileConfigCallPacket(tile_config.clone());
                    let accepted = {
                        let mut state = state.lock().expect("NetServerState mutex poisoned");
                        Self::record_tile_config(&mut state, connection_id, tile_config)
                    };

                    if !accepted {
                        return;
                    }

                    Self::dispatch_packet_handlers(&packet_handlers, &packet);
                }
                PacketKind::RotateBlockCallPacket(rotate) => {
                    let packet = PacketKind::RotateBlockCallPacket(rotate.clone());
                    let accepted = {
                        let mut state = state.lock().expect("NetServerState mutex poisoned");
                        Self::record_rotate_block(&mut state, connection_id, rotate)
                    };

                    if !accepted {
                        return;
                    }

                    Self::dispatch_packet_handlers(&packet_handlers, &packet);
                }
                PacketKind::TileTapCallPacket(tile_tap) => {
                    let packet = PacketKind::TileTapCallPacket(tile_tap.clone());
                    {
                        let mut state = state.lock().expect("NetServerState mutex poisoned");
                        Self::record_tile_tap(&mut state, connection_id, tile_tap);
                    }

                    Self::dispatch_packet_handlers(&packet_handlers, &packet);
                }
                _ => {}
            });
        }
    }

    fn dispatch_packet_handlers(
        packet_handlers: &Arc<Mutex<Vec<PacketHandler>>>,
        packet: &PacketKind,
    ) {
        let handlers = packet_handlers
            .lock()
            .expect("packet handlers mutex poisoned")
            .clone();

        for handler in handlers {
            let mut handler = handler.lock().expect("packet handler mutex poisoned");
            let callback = handler.as_mut();
            callback(packet);
        }
    }

    fn record_client_snapshot(
        state: &mut NetServerState,
        connection_id: Option<i32>,
        snapshot: &ClientSnapshotCallPacket,
    ) -> bool {
        if let Some(connection_id) = connection_id {
            if let Some(connection) = state.connection_states.get(&connection_id) {
                if snapshot.snapshot_id < connection.last_received_client_snapshot {
                    return false;
                }
            }
        }

        let now = Instant::now();
        state.last_connection_id = connection_id;
        state.last_client_snapshot_connection_id = connection_id;
        state.last_client_snapshot = Some(snapshot.clone());
        state.last_client_snapshot_received_at = Some(now);
        state.client_snapshot_packets_seen += 1;
        state.last_updated_at = Some(now);

        if let Some(connection_id) = connection_id {
            let connection = state
                .connection_states
                .entry(connection_id)
                .or_insert_with(|| NetConnection::new(String::new()));
            connection.last_received_client_snapshot = snapshot.snapshot_id;
            connection.last_received_client_time = Self::current_millis();
            connection.view_x = snapshot.view_x;
            connection.view_y = snapshot.view_y;
            connection.view_width = snapshot.view_width;
            connection.view_height = snapshot.view_height;
        }

        state.events.push(ProviderEvent::ServerPacket {
            connection_id: connection_id.unwrap_or(-1),
            packet: PacketKind::ClientSnapshotCallPacket(snapshot.clone()),
        });
        true
    }

    fn record_client_plan_snapshot(
        state: &mut NetServerState,
        connection_id: Option<i32>,
        snapshot: &ClientPlanSnapshotCallPacket,
    ) {
        let now = Instant::now();
        state.last_connection_id = connection_id;
        state.last_client_plan_snapshot_connection_id = connection_id;
        state.last_client_plan_snapshot = Some(snapshot.clone());
        state.last_client_plan_snapshot_received_at = Some(now);
        state.client_plan_snapshot_packets_seen += 1;
        state.last_updated_at = Some(now);

        state.events.push(ProviderEvent::ServerPacket {
            connection_id: connection_id.unwrap_or(-1),
            packet: PacketKind::ClientPlanSnapshotCallPacket(snapshot.clone()),
        });
    }

    fn record_tile_config(
        state: &mut NetServerState,
        connection_id: Option<i32>,
        packet: &TileConfigCallPacket,
    ) -> bool {
        let mut action =
            Self::player_action_for_connection(state, connection_id, ActionType::Configure);
        action.tile = packet.build.tile_pos;
        action.config = Some(format!("{:?}", packet.value));

        if !state.administration.allow_action(&action) {
            return false;
        }

        let now = Instant::now();
        state.last_connection_id = connection_id;
        state.last_tile_config_connection_id = connection_id;
        state.last_tile_config = Some(packet.clone());
        state.last_tile_config_received_at = Some(now);
        state.tile_config_packets_seen += 1;
        state.last_updated_at = Some(now);

        state.events.push(ProviderEvent::ServerPacket {
            connection_id: connection_id.unwrap_or(-1),
            packet: PacketKind::TileConfigCallPacket(packet.clone()),
        });
        true
    }

    fn record_rotate_block(
        state: &mut NetServerState,
        connection_id: Option<i32>,
        packet: &RotateBlockCallPacket,
    ) -> bool {
        let mut action =
            Self::player_action_for_connection(state, connection_id, ActionType::Rotate);
        action.tile = packet.build.tile_pos;
        action.rotation = i32::from(packet.direction);

        if !state.administration.allow_action(&action) {
            return false;
        }

        let now = Instant::now();
        state.last_connection_id = connection_id;
        state.last_rotate_block_connection_id = connection_id;
        state.last_rotate_block = Some(packet.clone());
        state.last_rotate_block_received_at = Some(now);
        state.rotate_block_packets_seen += 1;
        state.last_updated_at = Some(now);

        state.events.push(ProviderEvent::ServerPacket {
            connection_id: connection_id.unwrap_or(-1),
            packet: PacketKind::RotateBlockCallPacket(packet.clone()),
        });
        true
    }

    fn player_action_for_connection(
        state: &NetServerState,
        connection_id: Option<i32>,
        action_type: ActionType,
    ) -> PlayerAction {
        let player = connection_id.and_then(|connection_id| {
            state
                .connection_states
                .get(&connection_id)
                .map(|connection| {
                    if !connection.name.is_empty() {
                        connection.name.clone()
                    } else if !connection.uuid.is_empty() {
                        connection.uuid.clone()
                    } else {
                        connection_id.to_string()
                    }
                })
        });

        PlayerAction::new(player, action_type)
    }

    fn record_tile_tap(
        state: &mut NetServerState,
        connection_id: Option<i32>,
        packet: &TileTapCallPacket,
    ) {
        let now = Instant::now();
        state.last_connection_id = connection_id;
        state.last_tile_tap_connection_id = connection_id;
        state.last_tile_tap = Some(packet.clone());
        state.last_tile_tap_received_at = Some(now);
        state.tile_tap_packets_seen += 1;
        state.last_updated_at = Some(now);

        state.events.push(ProviderEvent::ServerPacket {
            connection_id: connection_id.unwrap_or(-1),
            packet: PacketKind::TileTapCallPacket(packet.clone()),
        });
    }

    fn current_millis() -> i64 {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        millis.min(i64::MAX as u128) as i64
    }

    fn record_connection_sent(
        state: &mut NetServerState,
        connection_id: i32,
        packet_name: &str,
        reliable: bool,
    ) {
        state
            .connection_states
            .entry(connection_id)
            .or_insert_with(|| NetConnection::new(String::new()))
            .send(SentPacket::Other(packet_name.into()), reliable);
    }

    fn sync_provider_connections(state: &mut NetServerState, connections: Vec<NetConnection>) {
        let snapshot = connections;
        state.connections = snapshot.clone();
        let mut matched_any = false;

        for connection in state.connection_states.values_mut() {
            if let Some(provider_connection) = snapshot.iter().find(|provider_connection| {
                (!provider_connection.address.is_empty()
                    && provider_connection.address == connection.address)
                    || (!provider_connection.uuid.is_empty()
                        && provider_connection.uuid == connection.uuid)
                    || (!provider_connection.usid.is_empty()
                        && provider_connection.usid == connection.usid)
            }) {
                matched_any = true;
                let has_connected = connection.has_connected;
                let has_begun_connecting = connection.has_begun_connecting;
                let has_disconnected = connection.has_disconnected;
                let player_added = connection.player_added;
                let name = connection.name.clone();
                let locale = connection.locale.clone();
                let color = connection.color;
                let modclient = connection.modclient;
                let kicked = connection.kicked;
                let sent = connection.sent.clone();
                let last_received_client_snapshot = connection.last_received_client_snapshot;
                let snapshots_sent = connection.snapshots_sent;
                let last_received_client_time = connection.last_received_client_time;
                let view_width = connection.view_width;
                let view_height = connection.view_height;
                let view_x = connection.view_x;
                let view_y = connection.view_y;
                *connection = provider_connection.clone();
                connection.has_connected = has_connected;
                connection.has_begun_connecting = has_begun_connecting;
                connection.has_disconnected |= has_disconnected;
                connection.player_added = player_added;
                connection.name = name;
                connection.locale = locale;
                connection.color = color;
                connection.modclient |= modclient;
                connection.kicked |= kicked;
                connection.sent = sent;
                connection.last_received_client_snapshot = last_received_client_snapshot;
                connection.snapshots_sent = snapshots_sent;
                connection.last_received_client_time = last_received_client_time;
                connection.view_width = view_width;
                connection.view_height = view_height;
                connection.view_x = view_x;
                connection.view_y = view_y;
            }
        }

        if !matched_any && state.connection_states.len() == 1 && snapshot.len() == 1 {
            if let Some((_, connection)) = state.connection_states.iter_mut().next() {
                let has_connected = connection.has_connected;
                let has_begun_connecting = connection.has_begun_connecting;
                let has_disconnected = connection.has_disconnected;
                let player_added = connection.player_added;
                let name = connection.name.clone();
                let locale = connection.locale.clone();
                let color = connection.color;
                let modclient = connection.modclient;
                let kicked = connection.kicked;
                let sent = connection.sent.clone();
                let last_received_client_snapshot = connection.last_received_client_snapshot;
                let snapshots_sent = connection.snapshots_sent;
                let last_received_client_time = connection.last_received_client_time;
                let view_width = connection.view_width;
                let view_height = connection.view_height;
                let view_x = connection.view_x;
                let view_y = connection.view_y;
                *connection = snapshot[0].clone();
                connection.has_connected = has_connected;
                connection.has_begun_connecting = has_begun_connecting;
                connection.has_disconnected |= has_disconnected;
                connection.player_added = player_added;
                connection.name = name;
                connection.locale = locale;
                connection.color = color;
                connection.modclient |= modclient;
                connection.kicked |= kicked;
                connection.sent = sent;
                connection.last_received_client_snapshot = last_received_client_snapshot;
                connection.snapshots_sent = snapshots_sent;
                connection.last_received_client_time = last_received_client_time;
                connection.view_width = view_width;
                connection.view_height = view_height;
                connection.view_x = view_x;
                connection.view_y = view_y;
            }
        } else if state.connection_states.is_empty() && snapshot.len() == 1 {
            if let Some(connection_id) = state.last_connection_id {
                let mut connection = snapshot[0].clone();
                connection.has_connected = false;
                connection.has_begun_connecting = false;
                state.connection_states.insert(connection_id, connection);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use crate::mindustry::entities::comp::BuildingComp;
    use crate::mindustry::io::{BuildPlanWire, BuildingRef, EntityRef, TeamId, TypeValue};
    use crate::mindustry::net::{
        packet_ids, ActionType, ClientPlanSnapshotCallPacket, ClientPlanSnapshotReceivedCallPacket,
        ClientSnapshotCallPacket, Connect, ConnectConfirmCallPacket, ConnectPacket, Disconnect,
        DoneCallback, EntitySnapshotCallPacket, HiddenSnapshotCallPacket, Host, HostCallback,
        KickReason, Net, NetConnection, NetProvider, PacketKind, PingCallPacket,
        PingResponseCallPacket, ProviderEvent, RequestDebugStatusCallPacket, RotateBlockCallPacket,
        SentPacket, StateSnapshotCallPacket, SteamAdminData, TileConfigCallPacket,
        TileTapCallPacket,
    };
    use crate::mindustry::vars::MAX_TCP_SIZE;
    use crate::mindustry::world::block::Block;

    use super::{
        ConnectPacketValidationContext, NetServer, NetServerState, PlayerPreviewPlanSource,
        PLAN_PREVIEW_CHUNK_SIZE,
    };

    #[derive(Clone, Default)]
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
                "not implemented",
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
    }

    fn connect_packet(name: &str) -> ConnectPacket {
        ConnectPacket {
            version: 157,
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

    fn config_block() -> Block {
        let mut block = Block::new(5, "router");
        block.health = 100;
        block
    }

    #[test]
    fn connect_packet_validation_accepts_and_normalizes_java_handshake_fields() {
        let mut packet = connect_packet("\n player\t");
        packet.locale.clear();
        let context = ConnectPacketValidationContext {
            connection_address: "steam:steam-uuid".into(),
            ..Default::default()
        };

        let plan = NetServer::validate_connect_packet(&packet, &context);

        assert!(plan.accepted());
        assert_eq!(plan.normalized_uuid, "steam-uuid");
        assert_eq!(plan.normalized_name, "player");
        assert_eq!(plan.locale, "en");
        assert!(plan.mark_begun_connecting);
        assert!(!plan.mod_client);
        assert_eq!(plan.kick_reason(), None);
    }

    #[test]
    fn connect_packet_validation_matches_java_rejection_order() {
        let packet = connect_packet("player");

        let plan = NetServer::validate_connect_packet(
            &packet,
            &ConnectPacketValidationContext {
                has_begun_connecting: true,
                ..Default::default()
            },
        );
        assert_eq!(plan.kick_reason(), Some(KickReason::IdInUse));

        let plan = NetServer::validate_connect_packet(
            &packet,
            &ConnectPacketValidationContext {
                id_banned: true,
                ..Default::default()
            },
        );
        assert_eq!(plan.kick_reason(), Some(KickReason::Banned));

        let plan = NetServer::validate_connect_packet(
            &packet,
            &ConnectPacketValidationContext {
                whitelisted: false,
                ..Default::default()
            },
        );
        assert_eq!(plan.kick_reason(), Some(KickReason::Whitelist));

        let plan = NetServer::validate_connect_packet(
            &packet,
            &ConnectPacketValidationContext {
                prevent_duplicates: true,
                duplicate_name: true,
                ..Default::default()
            },
        );
        assert_eq!(plan.kick_reason(), Some(KickReason::NameInUse));
    }

    #[test]
    fn connect_packet_context_uses_administration_bans_and_whitelist() {
        let packet = connect_packet("player");
        let mut state = NetServerState::default();
        state
            .connection_states
            .insert(7, NetConnection::new("1.2.3.4"));
        state.administration.ban_player_ip("1.2.3.4");
        state.administration.ban_player_id("uuid");

        let context = NetServer::connect_packet_validation_context(&state, 7, &packet);
        assert!(context.ip_banned);
        assert!(context.id_banned);

        let mut state = NetServerState::default();
        state
            .connection_states
            .insert(8, NetConnection::new("10.0.0.2"));
        state.administration.set_whitelist_enabled(true);
        let context = NetServer::connect_packet_validation_context(&state, 8, &packet);
        assert!(!context.whitelisted);

        state.administration.admin_player("uuid", "usid");
        state.administration.whitelist("uuid");
        let context = NetServer::connect_packet_validation_context(&state, 8, &packet);
        assert!(context.whitelisted);
    }

    #[test]
    fn connect_packet_context_uses_steam_ban_data_for_steam_connections() {
        let packet = connect_packet("player");
        let mut state = NetServerState {
            steam_admin_data: SteamAdminData::from_lists(["12345"], ["99999"]),
            ..Default::default()
        };
        state
            .connection_states
            .insert(9, NetConnection::new("steam:12345"));

        let context = NetServer::connect_packet_validation_context(&state, 9, &packet);

        assert_eq!(context.connection_address, "steam:12345");
        assert!(context.id_banned);

        let plan = NetServer::validate_connect_packet(&packet, &context);
        assert_eq!(plan.kick_reason(), Some(KickReason::Banned));
    }

    #[test]
    fn connect_packet_context_fills_name_recent_kick_and_player_limit_rejections() {
        let mut packet = connect_packet("blocked-name");
        packet.uuid = "limited".into();
        packet.usid = "limited-usid".into();

        let mut state = NetServerState {
            player_limit: 1,
            ..Default::default()
        };
        state
            .connection_states
            .insert(10, NetConnection::new("5.5.5.5"));
        let mut existing = NetConnection::new("6.6.6.6");
        existing.player_added = true;
        state.connection_states.insert(11, existing);
        state.administration.ban_name_pattern("blocked");
        state.administration.handle_kicked(
            "limited",
            "5.5.5.5",
            NetServer::current_millis() + 60_000,
        );

        let context = NetServer::connect_packet_validation_context(&state, 10, &packet);

        assert!(context.name_banned);
        assert!(context.recent_kick_active);
        assert!(context.player_limit_reached);
        assert_eq!(
            NetServer::validate_connect_packet(&packet, &context).kick_reason(),
            Some(KickReason::Banned)
        );

        state
            .administration
            .admin_player(packet.uuid.clone(), packet.usid.clone());
        let context = NetServer::connect_packet_validation_context(&state, 10, &packet);
        assert!(!context.player_limit_reached);
    }

    #[test]
    fn connect_packet_validation_matches_java_version_and_name_rejections() {
        let mut packet = connect_packet("player");

        packet.version_type = "custom".into();
        let plan = NetServer::validate_connect_packet(&packet, &Default::default());
        assert_eq!(plan.kick_reason(), Some(KickReason::TypeMismatch));

        packet.version_type = "official".into();
        packet.version = -1;
        let plan = NetServer::validate_connect_packet(&packet, &Default::default());
        assert_eq!(plan.kick_reason(), Some(KickReason::CustomClient));

        let plan = NetServer::validate_connect_packet(
            &packet,
            &ConnectPacketValidationContext {
                allows_custom_clients: true,
                ..Default::default()
            },
        );
        assert!(plan.accepted());
        assert!(plan.mod_client);

        packet.version = 158;
        let plan = NetServer::validate_connect_packet(&packet, &Default::default());
        assert_eq!(plan.kick_reason(), Some(KickReason::ServerOutdated));

        packet.version = 156;
        let plan = NetServer::validate_connect_packet(&packet, &Default::default());
        assert_eq!(plan.kick_reason(), Some(KickReason::ClientOutdated));

        packet.version = 157;
        packet.name = "[\n\t".into();
        let plan = NetServer::validate_connect_packet(&packet, &Default::default());
        assert_eq!(plan.kick_reason(), Some(KickReason::NameEmpty));
    }

    #[test]
    fn core_typed_listeners_update_server_state_and_handlers() {
        let server = NetServer::default();
        let seen = Arc::new(Mutex::new(Vec::new()));

        let seen_handler = Arc::clone(&seen);
        server.add_packet_handler(move |packet| {
            let label = match packet {
                PacketKind::Connect(_) => "connect",
                PacketKind::ConnectPacket(_) => "connect-packet",
                PacketKind::ConnectConfirmCallPacket(_) => "connect-confirm",
                PacketKind::Disconnect(_) => "disconnect",
                _ => return,
            };
            seen_handler.lock().unwrap().push(label.to_string());
        });

        {
            let mut net = server.net_mut();
            net.handle_server_received_from_connection(
                Some(12),
                false,
                PacketKind::Connect(Connect {
                    address_tcp: "10.0.0.2:6567".into(),
                }),
            );
            net.handle_server_received_from_connection(
                Some(12),
                false,
                PacketKind::ConnectPacket(connect_packet("player")),
            );
            net.handle_server_received_from_connection(
                Some(12),
                false,
                PacketKind::ConnectConfirmCallPacket(ConnectConfirmCallPacket),
            );
            net.handle_server_received_from_connection(
                Some(12),
                true,
                PacketKind::Disconnect(Disconnect {
                    reason: "left".into(),
                }),
            );
        }

        assert_eq!(
            *seen.lock().unwrap(),
            vec!["connect", "connect-packet", "connect-confirm", "disconnect"]
        );

        let state = server.state();
        let state = state.lock().unwrap();
        assert!(state.active);
        assert!(state.server);
        assert_eq!(state.last_connection_id, Some(12));
        assert_eq!(
            state.last_connect.as_ref().unwrap().address_tcp,
            "10.0.0.2:6567"
        );
        assert_eq!(state.last_handshake.as_ref().unwrap().name, "player");
        assert_eq!(state.last_connect_confirm_connection_id, Some(12));
        assert_eq!(state.last_disconnect_reason.as_deref(), Some("left"));
        let connection = state.connection_states.get(&12).unwrap();
        assert_eq!(connection.address, "10.0.0.2:6567");
        assert_eq!(connection.name, "player");
        assert_eq!(connection.locale, "en_US");
        assert_eq!(connection.uuid, "uuid");
        assert_eq!(connection.usid, "usid");
        assert!(!connection.mobile);
        assert!(!connection.modclient);
        assert!(connection.has_connected);
        assert!(connection.has_begun_connecting);
        assert!(connection.has_disconnected);
        assert!(!connection.player_added);
        assert_eq!(state.last_connect_validation_connection_id, Some(12));
        assert!(state
            .last_connect_validation_plan
            .as_ref()
            .unwrap()
            .accepted());
        assert_eq!(state.connect_packets_accepted, 1);
        assert_eq!(state.connect_packets_rejected, 0);
        assert_eq!(state.pending_world_data_connections, vec![12]);
        assert_eq!(state.events.len(), 4);
    }

    #[test]
    fn accepted_connect_packet_queues_world_data_connection_for_runtime_adapter() {
        let server = NetServer::default();

        {
            let mut net = server.net_mut();
            net.handle_server_received_from_connection(
                Some(51),
                false,
                PacketKind::Connect(Connect {
                    address_tcp: "10.0.0.51:6567".into(),
                }),
            );
            net.handle_server_received_from_connection(
                Some(51),
                false,
                PacketKind::ConnectPacket(connect_packet("player")),
            );
        }

        {
            let state = server.state();
            let state = state.lock().unwrap();
            assert_eq!(state.connect_packets_accepted, 1);
            assert_eq!(state.connect_packets_rejected, 0);
            assert!(state.pending_connect_kicks.is_empty());
            assert_eq!(state.pending_world_data_connections, vec![51]);
        }

        assert_eq!(server.take_pending_world_data_connections(), vec![51]);
        assert!(server.take_pending_world_data_connections().is_empty());
    }

    #[test]
    fn send_pending_world_data_streams_accepted_connections_without_begin_packet() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));

        {
            let mut net = server.net_mut();
            net.handle_server_received_from_connection(
                Some(52),
                false,
                PacketKind::Connect(Connect {
                    address_tcp: "10.0.0.52:6567".into(),
                }),
            );
            net.handle_server_received_from_connection(
                Some(52),
                false,
                PacketKind::ConnectPacket(connect_packet("player")),
            );
        }

        let sent_count = server
            .send_pending_world_data(|connection_id| vec![connection_id as u8, 1, 2])
            .unwrap();
        assert_eq!(sent_count, 1);
        assert!(server.take_pending_world_data_connections().is_empty());

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[0].0, 52);
        assert_eq!(sent[1].0, 52);
        assert!(sent.iter().all(|(_, _, reliable)| *reliable));
        assert!(matches!(sent[0].1, PacketKind::StreamBegin(_)));
        assert!(matches!(sent[1].1, PacketKind::StreamChunk(_)));
        assert!(!sent
            .iter()
            .any(|(_, packet, _)| matches!(packet, PacketKind::WorldDataBeginCallPacket(_))));
        drop(sent);

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.last_world_data_connection_id, Some(52));
        assert_eq!(state.last_world_data_bytes, Some(3));
        assert_eq!(state.world_streams_sent, 1);
    }

    #[test]
    fn accepted_connect_then_world_data_then_connect_confirm_completes_join_flags() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));

        {
            let mut net = server.net_mut();
            net.handle_server_received_from_connection(
                Some(53),
                false,
                PacketKind::Connect(Connect {
                    address_tcp: "10.0.0.53:6567".into(),
                }),
            );
            net.handle_server_received_from_connection(
                Some(53),
                false,
                PacketKind::ConnectPacket(connect_packet("player")),
            );
        }

        {
            let state = server.state();
            let state = state.lock().unwrap();
            assert_eq!(state.pending_world_data_connections, vec![53]);
            let connection = state.connection_states.get(&53).unwrap();
            assert!(connection.has_begun_connecting);
            assert!(!connection.has_connected);
            assert!(!connection.player_added);
        }

        let sent_count = server
            .send_pending_world_data(|connection_id| vec![connection_id as u8, 1, 2, 3])
            .unwrap();
        assert_eq!(sent_count, 1);

        {
            let sent = sent.lock().unwrap();
            assert_eq!(sent.len(), 2);
            assert_eq!(sent[0].0, 53);
            assert_eq!(sent[1].0, 53);
            assert!(sent.iter().all(|(_, _, reliable)| *reliable));
            match &sent[0].1 {
                PacketKind::StreamBegin(begin) => {
                    assert_eq!(begin.id, 0);
                    assert_eq!(begin.total, 4);
                    assert_eq!(begin.packet_type, packet_ids::WORLD_STREAM);
                }
                other => panic!("unexpected first world stream packet: {other:?}"),
            }
            match &sent[1].1 {
                PacketKind::StreamChunk(chunk) => assert_eq!(chunk.data, vec![53, 1, 2, 3]),
                other => panic!("unexpected second world stream packet: {other:?}"),
            }
        }

        {
            let mut net = server.net_mut();
            net.handle_server_received_from_connection(
                Some(53),
                false,
                PacketKind::ConnectConfirmCallPacket(ConnectConfirmCallPacket),
            );
        }

        let state = server.state();
        let state = state.lock().unwrap();
        assert!(state.pending_world_data_connections.is_empty());
        assert_eq!(state.last_world_data_connection_id, Some(53));
        assert_eq!(state.last_world_data_bytes, Some(4));
        assert_eq!(state.world_streams_sent, 1);
        assert_eq!(state.last_connect_confirm_connection_id, Some(53));
        let connection = state.connection_states.get(&53).unwrap();
        assert!(connection.has_begun_connecting);
        assert!(connection.has_connected);
        assert!(connection.player_added);
        assert_eq!(connection.sent.len(), 2);
        assert!(matches!(connection.sent[0].0, SentPacket::StreamBegin(_)));
        assert!(matches!(connection.sent[1].0, SentPacket::StreamChunk(_)));
    }

    #[test]
    fn connect_packet_listener_validates_and_flushes_kick_without_recursive_net_lock() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));
        let seen = Arc::new(Mutex::new(Vec::new()));

        let seen_handler = Arc::clone(&seen);
        server.add_packet_handler(move |packet| {
            if matches!(packet, PacketKind::ConnectPacket(_)) {
                seen_handler.lock().unwrap().push("connect-packet");
            }
        });

        let mut rejected = connect_packet("player");
        rejected.version_type = "custom".into();

        {
            let mut net = server.net_mut();
            net.handle_server_received_from_connection(
                Some(41),
                false,
                PacketKind::Connect(Connect {
                    address_tcp: "10.0.0.41:6567".into(),
                }),
            );
            net.handle_server_received_from_connection(
                Some(41),
                false,
                PacketKind::ConnectPacket(rejected),
            );
        }

        {
            let state = server.state();
            let state = state.lock().unwrap();
            assert_eq!(state.connect_packets_accepted, 0);
            assert_eq!(state.connect_packets_rejected, 1);
            assert_eq!(
                state.pending_connect_kicks,
                vec![(41, KickReason::TypeMismatch)]
            );
            assert_eq!(state.last_kick_connection_id, None);
            assert_eq!(
                state
                    .last_connect_validation_plan
                    .as_ref()
                    .unwrap()
                    .kick_reason(),
                Some(KickReason::TypeMismatch)
            );
            let connection = state.connection_states.get(&41).unwrap();
            assert!(connection.has_begun_connecting);
            assert!(!connection.kicked);
        }

        server.update();

        assert_eq!(*seen.lock().unwrap(), vec!["connect-packet"]);
        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, 41);
        assert!(sent[0].2);
        assert!(matches!(
            sent[0].1,
            PacketKind::KickCallPacket2(packet)
                if packet.reason == KickReason::TypeMismatch
        ));
        drop(sent);

        let state = server.state();
        let state = state.lock().unwrap();
        assert!(state.pending_connect_kicks.is_empty());
        assert_eq!(state.last_kick_connection_id, Some(41));
        assert_eq!(state.last_kick_reason, Some(KickReason::TypeMismatch));
        let connection = state.connection_states.get(&41).unwrap();
        assert!(connection.kicked);
        assert!(connection.has_disconnected);
    }

    #[test]
    fn client_snapshot_and_plan_snapshot_packets_update_server_state() {
        let server = NetServer::default();
        let seen = Arc::new(Mutex::new(Vec::new()));

        let seen_handler = Arc::clone(&seen);
        server.add_packet_handler(move |packet| {
            let label = match packet {
                PacketKind::ClientSnapshotCallPacket(_) => "snapshot",
                PacketKind::ClientPlanSnapshotCallPacket(_) => "plan",
                _ => return,
            };
            seen_handler.lock().unwrap().push(label.to_string());
        });

        {
            let mut net = server.net_mut();
            net.handle_server_received_from_connection(
                Some(7),
                true,
                PacketKind::ClientSnapshotCallPacket(ClientSnapshotCallPacket {
                    snapshot_id: 15,
                    unit_id: 23,
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
                }),
            );
            net.handle_server_received_from_connection(
                Some(7),
                true,
                PacketKind::ClientSnapshotCallPacket(ClientSnapshotCallPacket {
                    snapshot_id: 14,
                    unit_id: 99,
                    dead: true,
                    x: 0.0,
                    y: 0.0,
                    pointer_x: 0.0,
                    pointer_y: 0.0,
                    rotation: 0.0,
                    base_rotation: 0.0,
                    x_velocity: 0.0,
                    y_velocity: 0.0,
                    mining: None,
                    boosting: false,
                    shooting: false,
                    chatting: false,
                    building: false,
                    selected_block: None,
                    selected_rotation: 0,
                    plans: None,
                    view_x: 0.0,
                    view_y: 0.0,
                    view_width: 0.0,
                    view_height: 0.0,
                }),
            );
            net.handle_server_received_from_connection(
                Some(7),
                true,
                PacketKind::ClientPlanSnapshotCallPacket(ClientPlanSnapshotCallPacket {
                    group_id: 88,
                    plans: None,
                }),
            );
        }

        assert_eq!(*seen.lock().unwrap(), vec!["snapshot", "plan"]);

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.last_client_snapshot_connection_id, Some(7));
        assert_eq!(
            state
                .last_client_snapshot
                .as_ref()
                .map(|snapshot| snapshot.snapshot_id),
            Some(15)
        );
        assert_eq!(state.client_snapshot_packets_seen, 1);
        assert_eq!(state.last_client_plan_snapshot_connection_id, Some(7));
        assert_eq!(
            state
                .last_client_plan_snapshot
                .as_ref()
                .map(|snapshot| snapshot.group_id),
            Some(88)
        );
        assert_eq!(state.client_plan_snapshot_packets_seen, 1);

        let connection = state.connection_states.get(&7).unwrap();
        assert_eq!(connection.last_received_client_snapshot, 15);
        assert_eq!(connection.view_x, 9.0);
        assert_eq!(connection.view_y, 10.0);
        assert_eq!(connection.view_width, 11.0);
        assert_eq!(connection.view_height, 12.0);
        assert!(connection.last_received_client_time > 0);
        assert_eq!(state.events.len(), 2);
    }

    #[test]
    fn tile_config_packets_are_recorded_and_dispatched() {
        let server = NetServer::default();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let tile_pos = crate::mindustry::world::point2_pack(2, 3);
        let packet = TileConfigCallPacket::client(
            BuildingRef::new(tile_pos),
            TypeValue::String("next".into()),
        );

        let seen_handler = Arc::clone(&seen);
        server.add_packet_handler(move |packet| {
            if let PacketKind::TileConfigCallPacket(packet) = packet {
                seen_handler.lock().unwrap().push(packet.value.clone());
            }
        });

        {
            let mut net = server.net_mut();
            net.handle_server_received_from_connection(
                Some(9),
                true,
                PacketKind::TileConfigCallPacket(packet.clone()),
            );
        }

        assert_eq!(
            *seen.lock().unwrap(),
            vec![TypeValue::String("next".into())]
        );

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.last_tile_config_connection_id, Some(9));
        assert_eq!(state.last_tile_config.as_ref(), Some(&packet));
        assert_eq!(state.tile_config_packets_seen, 1);
        assert!(state.last_tile_config_received_at.is_some());
        assert!(matches!(
            state.events.last(),
            Some(ProviderEvent::ServerPacket {
                connection_id: 9,
                packet: PacketKind::TileConfigCallPacket(_)
            })
        ));
    }

    #[test]
    fn action_filters_gate_tile_config_and_rotate_player_packets() {
        let server = NetServer::default();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let tile_pos = crate::mindustry::world::point2_pack(2, 3);
        let blocked_config = TileConfigCallPacket::client(
            BuildingRef::new(tile_pos),
            TypeValue::String("forbidden".into()),
        );
        let allowed_server_config = blocked_config.clone();
        let rotate = RotateBlockCallPacket::client(BuildingRef::new(tile_pos), true);

        {
            let state = server.state();
            let mut state = state.lock().unwrap();
            let mut connection = NetConnection::new("1.2.3.4");
            connection.name = "builder".into();
            state.connection_states.insert(9, connection);
            state.administration.add_action_filter(|action| {
                action.player.is_none()
                    || (action.action_type != ActionType::Configure
                        && action.action_type != ActionType::Rotate)
            });
        }

        let seen_handler = Arc::clone(&seen);
        server.add_packet_handler(move |packet| {
            let label = match packet {
                PacketKind::TileConfigCallPacket(_) => "config",
                PacketKind::RotateBlockCallPacket(_) => "rotate",
                _ => return,
            };
            seen_handler.lock().unwrap().push(label.to_string());
        });

        {
            let mut net = server.net_mut();
            net.handle_server_received_from_connection(
                Some(9),
                true,
                PacketKind::TileConfigCallPacket(blocked_config),
            );
            net.handle_server_received_from_connection(
                Some(9),
                true,
                PacketKind::RotateBlockCallPacket(rotate),
            );
            net.handle_server_received_from_connection(
                None,
                true,
                PacketKind::TileConfigCallPacket(allowed_server_config),
            );
        }

        assert_eq!(*seen.lock().unwrap(), vec!["config"]);

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.tile_config_packets_seen, 1);
        assert_eq!(state.rotate_block_packets_seen, 0);
        assert_eq!(state.last_tile_config_connection_id, None);
        assert!(state.last_rotate_block.is_none());
    }

    #[test]
    fn rotate_and_tile_tap_packets_are_recorded_and_dispatched() {
        let server = NetServer::default();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let build_pos = crate::mindustry::world::point2_pack(4, 5);
        let tap_pos = crate::mindustry::world::point2_pack(6, 7);
        let rotate = RotateBlockCallPacket {
            player: EntityRef::null(),
            build: BuildingRef::new(build_pos),
            direction: true,
        };
        let tap = TileTapCallPacket {
            player: EntityRef::null(),
            tile: Some(tap_pos),
        };

        let seen_handler = Arc::clone(&seen);
        server.add_packet_handler(move |packet| {
            let label = match packet {
                PacketKind::RotateBlockCallPacket(_) => "rotate",
                PacketKind::TileTapCallPacket(_) => "tap",
                _ => return,
            };
            seen_handler.lock().unwrap().push(label.to_string());
        });

        {
            let mut net = server.net_mut();
            net.handle_server_received_from_connection(
                Some(11),
                true,
                PacketKind::RotateBlockCallPacket(rotate.clone()),
            );
            net.handle_server_received_from_connection(
                Some(11),
                true,
                PacketKind::TileTapCallPacket(tap.clone()),
            );
        }

        assert_eq!(*seen.lock().unwrap(), vec!["rotate", "tap"]);

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.last_rotate_block_connection_id, Some(11));
        assert_eq!(state.last_rotate_block.as_ref(), Some(&rotate));
        assert_eq!(state.rotate_block_packets_seen, 1);
        assert!(state.last_rotate_block_received_at.is_some());
        assert_eq!(state.last_tile_tap_connection_id, Some(11));
        assert_eq!(state.last_tile_tap.as_ref(), Some(&tap));
        assert_eq!(state.tile_tap_packets_seen, 1);
        assert!(state.last_tile_tap_received_at.is_some());
        assert_eq!(state.events.len(), 2);
        assert!(matches!(
            state.events[0],
            ProviderEvent::ServerPacket {
                connection_id: 11,
                packet: PacketKind::RotateBlockCallPacket(_)
            }
        ));
        assert!(matches!(
            state.events[1],
            ProviderEvent::ServerPacket {
                connection_id: 11,
                packet: PacketKind::TileTapCallPacket(_)
            }
        ));
    }

    #[test]
    fn tile_config_authority_rejects_and_sends_reliable_rollback() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));
        let tile_pos = crate::mindustry::world::point2_pack(4, 5);
        let mut building = BuildingComp::new(tile_pos, config_block(), TeamId(2));
        building.set_config_value(TypeValue::String("old".into()));

        let change = server
            .handle_tile_config_authority(
                Some(17),
                EntityRef::new(33),
                &mut building,
                TypeValue::Int(9),
                |value| matches!(value, TypeValue::String(_)),
            )
            .unwrap();

        assert!(!change.accepted);
        assert_eq!(building.config_value(), TypeValue::String("old".into()));

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, 17);
        assert!(sent[0].2);
        match &sent[0].1 {
            PacketKind::TileConfigCallPacket(packet) => {
                assert_eq!(packet.player, EntityRef::new(33));
                assert_eq!(packet.build, BuildingRef::new(tile_pos));
                assert_eq!(packet.value, TypeValue::String("old".into()));
            }
            other => panic!("unexpected rollback packet: {other:?}"),
        }
        drop(sent);

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.tile_config_rejections, 1);
        assert_eq!(state.last_tile_config_rollback_connection_id, Some(17));
        assert_eq!(state.tile_config_rollbacks_sent, 1);
        assert!(state.last_tile_config_rollback_error.is_none());
        let connection = state.connection_states.get(&17).unwrap();
        assert!(matches!(
            connection.sent[0].0,
            SentPacket::Other(ref name) if name == "TileConfigCallPacket"
        ));
        assert!(connection.sent[0].1);
    }

    #[test]
    fn send_client_plan_snapshot_received_targets_connection_unreliably_and_records_state() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));
        let packet = ClientPlanSnapshotReceivedCallPacket {
            player_id: 42,
            group_id: 88,
            plans: None,
        };

        server
            .send_client_plan_snapshot_received(12, packet.clone())
            .unwrap();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, 12);
        assert!(!sent[0].2);
        assert!(matches!(
            &sent[0].1,
            PacketKind::ClientPlanSnapshotReceivedCallPacket(sent_packet) if sent_packet == &packet
        ));
        drop(sent);

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(
            state.last_client_plan_snapshot_forwarded_connection_id,
            Some(12)
        );
        assert_eq!(state.client_plan_snapshots_forwarded, 1);
        assert!(state.last_client_plan_snapshot_forwarded_at.is_some());
        assert!(state.last_client_plan_snapshot_forwarded_error.is_none());

        let connection = state.connection_states.get(&12).unwrap();
        assert!(matches!(
            connection.sent[0].0,
            SentPacket::Other(ref name) if name == "ClientPlanSnapshotReceivedCallPacket"
        ));
        assert!(!connection.sent[0].1);
    }

    #[test]
    fn send_client_plan_snapshot_received_to_many_reuses_payload_for_selected_connections() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));
        let packet = ClientPlanSnapshotReceivedCallPacket {
            player_id: 42,
            group_id: 89,
            plans: None,
        };

        let sent_count = server
            .send_client_plan_snapshot_received_to_many(vec![3, 5, 8], packet.clone())
            .unwrap();

        assert_eq!(sent_count, 3);
        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 3);
        let target_ids: Vec<i32> = sent
            .iter()
            .map(|(connection_id, _, _)| *connection_id)
            .collect();
        assert_eq!(target_ids, vec![3, 5, 8]);
        assert!(sent.iter().all(|(_, packet_kind, reliable)| {
            !*reliable
                && matches!(
                    packet_kind,
                    PacketKind::ClientPlanSnapshotReceivedCallPacket(sent_packet)
                        if sent_packet == &packet
                )
        }));
        drop(sent);

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(
            state.last_client_plan_snapshot_forwarded_connection_id,
            Some(8)
        );
        assert_eq!(state.client_plan_snapshots_forwarded, 3);
        assert!(state.last_client_plan_snapshot_forwarded_error.is_none());

        for connection_id in [3, 5, 8] {
            let connection = state.connection_states.get(&connection_id).unwrap();
            assert_eq!(connection.sent.len(), 1);
            assert!(matches!(
                connection.sent[0].0,
                SentPacket::Other(ref name) if name == "ClientPlanSnapshotReceivedCallPacket"
            ));
            assert!(!connection.sent[0].1);
        }
    }

    #[test]
    fn broadcast_client_plan_previews_chunks_by_player_group_and_team_targets() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));
        let many_plans: Vec<_> = (0..PLAN_PREVIEW_CHUNK_SIZE + 2)
            .map(|index| BuildPlanWire::new_place(index as i32, index as i32 + 1, 0, "router"))
            .collect();
        let mut players = vec![
            PlayerPreviewPlanSource {
                player_id: 1,
                team: TeamId(1),
                connection_id: Some(10),
                local: false,
                connected: true,
                last_preview_plan_group_server: 4,
                plans: many_plans,
            },
            PlayerPreviewPlanSource {
                player_id: 2,
                team: TeamId(1),
                connection_id: Some(20),
                local: false,
                connected: true,
                last_preview_plan_group_server: 0,
                plans: Vec::new(),
            },
        ];

        let planned = NetServer::client_plan_preview_broadcasts(&mut players);
        assert_eq!(planned.len(), 3);
        assert_eq!(players[0].last_preview_plan_group_server, 5);
        assert_eq!(players[1].last_preview_plan_group_server, 1);
        assert_eq!(planned[0].target_connection_id, 20);
        assert_eq!(planned[0].packet.player_id, 1);
        assert_eq!(planned[0].packet.group_id, 5);
        assert_eq!(
            planned[0].packet.plans.as_ref().unwrap().len(),
            PLAN_PREVIEW_CHUNK_SIZE
        );
        assert_eq!(planned[1].target_connection_id, 20);
        assert_eq!(planned[1].packet.plans.as_ref().unwrap().len(), 2);
        assert_eq!(planned[2].target_connection_id, 10);
        assert_eq!(planned[2].packet.player_id, 2);
        assert_eq!(planned[2].packet.group_id, 1);
        assert!(planned[2].packet.plans.is_none());

        let sent_count = server.broadcast_client_plan_previews(&mut players).unwrap();
        assert_eq!(sent_count, 3);
        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 3);
        assert!(sent.iter().all(|(_, _, reliable)| !*reliable));
        assert!(matches!(
            sent[0].1,
            PacketKind::ClientPlanSnapshotReceivedCallPacket(_)
        ));
    }

    #[test]
    fn send_entity_sync_snapshot_uses_java_snapshot_order_unreliably_and_records_state() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));
        let state_packet = StateSnapshotCallPacket {
            wave_time: 1.5,
            wave: 2,
            enemies: 3,
            paused: false,
            game_over: false,
            time_data: 123,
            tps: 60,
            rand0: 11,
            rand1: 22,
            core_data: vec![1, 2, 3],
        };
        let first_entity_packet = EntitySnapshotCallPacket {
            amount: 1,
            data: vec![7, 8],
        };
        let second_entity_packet = EntitySnapshotCallPacket {
            amount: 2,
            data: vec![9, 10, 11],
        };
        let hidden_packet = HiddenSnapshotCallPacket { ids: vec![4, 5] };

        server
            .send_entity_sync_snapshot(
                21,
                state_packet.clone(),
                vec![first_entity_packet.clone(), second_entity_packet.clone()],
                Some(hidden_packet.clone()),
            )
            .unwrap();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 4);
        assert!(sent
            .iter()
            .all(|(connection_id, _, reliable)| *connection_id == 21 && !*reliable));
        assert!(matches!(
            &sent[0].1,
            PacketKind::StateSnapshotCallPacket(packet) if packet == &state_packet
        ));
        assert!(matches!(
            &sent[1].1,
            PacketKind::EntitySnapshotCallPacket(packet) if packet == &first_entity_packet
        ));
        assert!(matches!(
            &sent[2].1,
            PacketKind::EntitySnapshotCallPacket(packet) if packet == &second_entity_packet
        ));
        assert!(matches!(
            &sent[3].1,
            PacketKind::HiddenSnapshotCallPacket(packet) if packet == &hidden_packet
        ));
        drop(sent);

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.last_state_snapshot_connection_id, Some(21));
        assert_eq!(state.last_state_snapshot.as_ref(), Some(&state_packet));
        assert_eq!(state.state_snapshot_packets_sent, 1);
        assert!(state.last_state_snapshot_sent_at.is_some());
        assert!(state.last_state_snapshot_error.is_none());
        assert_eq!(state.last_entity_snapshot_connection_id, Some(21));
        assert_eq!(
            state.last_entity_snapshot.as_ref(),
            Some(&second_entity_packet)
        );
        assert_eq!(state.entity_snapshot_packets_sent, 2);
        assert!(state.last_entity_snapshot_sent_at.is_some());
        assert!(state.last_entity_snapshot_error.is_none());
        assert_eq!(state.last_hidden_snapshot_connection_id, Some(21));
        assert_eq!(state.last_hidden_snapshot.as_ref(), Some(&hidden_packet));
        assert_eq!(state.hidden_snapshot_packets_sent, 1);
        assert!(state.last_hidden_snapshot_sent_at.is_some());
        assert!(state.last_hidden_snapshot_error.is_none());

        let connection = state.connection_states.get(&21).unwrap();
        assert_eq!(connection.snapshots_sent, 1);
        assert_eq!(connection.sent.len(), 4);
        assert!(connection.sent.iter().all(|(_, reliable)| !*reliable));
        assert!(matches!(
            connection.sent[0].0,
            SentPacket::Other(ref name) if name == "StateSnapshotCallPacket"
        ));
        assert!(matches!(
            connection.sent[1].0,
            SentPacket::Other(ref name) if name == "EntitySnapshotCallPacket"
        ));
        assert!(matches!(
            connection.sent[2].0,
            SentPacket::Other(ref name) if name == "EntitySnapshotCallPacket"
        ));
        assert!(matches!(
            connection.sent[3].0,
            SentPacket::Other(ref name) if name == "HiddenSnapshotCallPacket"
        ));
    }

    #[test]
    fn send_world_data_targets_connection_and_records_java_stream_shape() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));

        server
            .resend_world_data(12, vec![7; MAX_TCP_SIZE + 3])
            .unwrap();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 4);
        assert!(matches!(sent[0].1, PacketKind::WorldDataBeginCallPacket(_)));
        assert!(sent
            .iter()
            .all(|(connection_id, _, reliable)| { *connection_id == 12 && *reliable }));

        match &sent[1].1 {
            PacketKind::StreamBegin(begin) => {
                assert_eq!(begin.id, 0);
                assert_eq!(begin.total, (MAX_TCP_SIZE + 3) as i32);
                assert_eq!(begin.packet_type, packet_ids::WORLD_STREAM);
            }
            other => panic!("unexpected packet: {other:?}"),
        }
        match &sent[2].1 {
            PacketKind::StreamChunk(chunk) => assert_eq!(chunk.data.len(), MAX_TCP_SIZE),
            other => panic!("unexpected packet: {other:?}"),
        }
        match &sent[3].1 {
            PacketKind::StreamChunk(chunk) => assert_eq!(chunk.data.len(), 3),
            other => panic!("unexpected packet: {other:?}"),
        }
        drop(sent);

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.last_world_data_connection_id, Some(12));
        assert_eq!(state.last_world_data_bytes, Some(MAX_TCP_SIZE + 3));
        assert_eq!(state.world_data_begin_sent, 1);
        assert_eq!(state.world_streams_sent, 1);
        assert!(state.last_world_data_error.is_none());

        let connection = state.connection_states.get(&12).unwrap();
        assert!(matches!(connection.sent[0].0, SentPacket::Other(_)));
        assert!(matches!(connection.sent[1].0, SentPacket::StreamBegin(_)));
        assert!(matches!(connection.sent[2].0, SentPacket::StreamChunk(_)));
        assert!(matches!(connection.sent[3].0, SentPacket::StreamChunk(_)));
    }

    #[test]
    fn kick_connection_reason_sends_java_reason_packet_and_closes_connection_state() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));

        server
            .kick_connection_reason(31, KickReason::TypeMismatch)
            .unwrap();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, 31);
        assert!(sent[0].2);
        assert!(matches!(
            sent[0].1,
            PacketKind::KickCallPacket2(packet) if packet.reason == KickReason::TypeMismatch
        ));
        drop(sent);

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.last_kick_connection_id, Some(31));
        assert_eq!(state.last_kick_reason, Some(KickReason::TypeMismatch));
        assert_eq!(state.last_kick_message, None);
        assert_eq!(state.kick_packets_sent, 1);
        assert!(state.last_kick_error.is_none());

        let connection = state.connection_states.get(&31).unwrap();
        assert!(connection.kicked);
        assert!(connection.has_disconnected);
        assert!(matches!(
            connection.sent[0].0,
            SentPacket::KickReason(KickReason::TypeMismatch)
        ));
        assert!(connection.sent[0].1);
    }

    #[test]
    fn kick_connection_message_sends_java_string_packet_and_records_message() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));

        server
            .kick_connection_message(32, "[accent]Incompatible mods![]")
            .unwrap();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, 32);
        assert!(sent[0].2);
        assert!(matches!(
            &sent[0].1,
            PacketKind::KickCallPacket(packet)
                if packet.reason == "[accent]Incompatible mods![]"
        ));
        drop(sent);

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.last_kick_connection_id, Some(32));
        assert_eq!(state.last_kick_reason, None);
        assert_eq!(
            state.last_kick_message.as_deref(),
            Some("[accent]Incompatible mods![]")
        );
        assert_eq!(state.kick_packets_sent, 1);
        assert!(state.last_kick_error.is_none());

        let connection = state.connection_states.get(&32).unwrap();
        assert!(connection.kicked);
        assert!(connection.has_disconnected);
        assert!(matches!(
            &connection.sent[0].0,
            SentPacket::KickMessage(message) if message == "[accent]Incompatible mods![]"
        ));
        assert!(connection.sent[0].1);
    }

    #[test]
    fn ping_call_returns_ping_response_to_same_connection() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));

        {
            let mut net = server.net_mut();
            net.handle_server_received_from_connection(
                Some(7),
                true,
                PacketKind::PingCallPacket(PingCallPacket { time: 1234 }),
            );
        }

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, 7);
        assert!(matches!(
            sent[0].1,
            PacketKind::PingResponseCallPacket(PingResponseCallPacket { time: 1234 })
        ));
        assert!(sent[0].2);
        drop(sent);

        let state = server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.last_ping_connection_id, Some(7));
        assert_eq!(state.last_ping_time, Some(1234));
        assert_eq!(state.ping_requests_seen, 1);
        assert_eq!(state.ping_responses_sent, 1);
        assert!(state.last_ping_error.is_none());
    }

    #[test]
    fn request_debug_status_sends_reliable_and_unreliable_java_flags_on_update() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let provider = CaptureProvider {
            sent: Arc::clone(&sent),
        };
        let server = NetServer::new(Net::new(Box::new(provider)));
        let seen = Arc::new(Mutex::new(Vec::new()));

        let seen_handler = Arc::clone(&seen);
        server.add_packet_handler(move |packet| {
            if matches!(packet, PacketKind::RequestDebugStatusCallPacket(_)) {
                seen_handler.lock().unwrap().push("debug");
            }
        });

        {
            let mut net = server.net_mut();
            net.handle_server_received_from_connection(
                Some(61),
                false,
                PacketKind::Connect(Connect {
                    address_tcp: "10.0.0.61:6567".into(),
                }),
            );
            net.handle_server_received_from_connection(
                Some(61),
                false,
                PacketKind::ConnectPacket(connect_packet("player")),
            );
            net.handle_server_received_from_connection(
                Some(61),
                false,
                PacketKind::ConnectConfirmCallPacket(ConnectConfirmCallPacket),
            );
            net.handle_server_received_from_connection(
                Some(61),
                true,
                PacketKind::RequestDebugStatusCallPacket(RequestDebugStatusCallPacket),
            );
        }

        {
            let state = server.state();
            let state = state.lock().unwrap();
            assert_eq!(state.debug_status_requests_seen, 1);
            assert_eq!(state.pending_debug_status_connections, vec![61]);
            let connection = state.connection_states.get(&61).unwrap();
            assert!(connection.has_connected);
            assert!(connection.has_begun_connecting);
            assert!(connection.player_added);
        }

        server.update();

        assert_eq!(*seen.lock().unwrap(), vec!["debug"]);
        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[0].0, 61);
        assert_eq!(sent[1].0, 61);
        assert!(sent[0].2);
        assert!(!sent[1].2);

        let expected_value = 2 | 4 | 8;
        assert!(matches!(
            sent[0].1,
            PacketKind::DebugStatusClientCallPacket(packet)
                if packet.value == expected_value
                    && packet.last_client_snapshot == -1
                    && packet.snapshots_sent == 0
        ));
        assert!(matches!(
            sent[1].1,
            PacketKind::DebugStatusClientUnreliableCallPacket(packet)
                if packet.0.value == expected_value
                    && packet.0.last_client_snapshot == -1
                    && packet.0.snapshots_sent == 0
        ));
        drop(sent);

        let state = server.state();
        let state = state.lock().unwrap();
        assert!(state.pending_debug_status_connections.is_empty());
        assert_eq!(state.last_debug_status_connection_id, Some(61));
        assert_eq!(state.last_debug_status.unwrap().value, expected_value);
        assert_eq!(state.debug_status_responses_sent, 2);
        assert!(state.last_debug_status_error.is_none());
    }
}
