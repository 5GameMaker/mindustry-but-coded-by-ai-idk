use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::mindustry::net::{
    packet_ids, ClientPlanSnapshotCallPacket, ClientPlanSnapshotReceivedCallPacket,
    ClientSnapshotCallPacket, Connect, ConnectPacket, Disconnect, Net, NetConnection, PacketKind,
    ProviderEvent, SentPacket, Streamable, WorldDataBeginCallPacket,
};

pub type PacketHandler = Arc<Mutex<Box<dyn FnMut(&PacketKind) + Send + 'static>>>;
pub type BinaryPacketHandler = Arc<Mutex<Box<dyn FnMut(&[u8]) + Send + 'static>>>;

#[derive(Debug, Clone, Default)]
pub struct NetServerState {
    pub active: bool,
    pub server: bool,
    pub listen_port: Option<u16>,
    pub connections: Vec<NetConnection>,
    pub connection_states: HashMap<i32, NetConnection>,
    pub last_connection_id: Option<i32>,
    pub last_connect: Option<Connect>,
    pub last_handshake: Option<ConnectPacket>,
    pub last_connect_confirm_connection_id: Option<i32>,
    pub last_world_data_connection_id: Option<i32>,
    pub last_world_data_bytes: Option<usize>,
    pub world_data_begin_sent: u64,
    pub world_streams_sent: u64,
    pub last_world_data_error: Option<String>,
    pub last_ping_connection_id: Option<i32>,
    pub last_ping_time: Option<i64>,
    pub ping_requests_seen: u64,
    pub ping_responses_sent: u64,
    pub last_ping_error: Option<String>,
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
            }
            Err(error) => {
                state.last_client_plan_snapshot_forwarded_error = Some(error.to_string());
            }
        }
        result
    }

    pub fn update(&self) {
        let connections = {
            let net = self.net.lock().expect("Net mutex poisoned");
            net.get_connections()
        };

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        Self::sync_provider_connections(&mut state, connections);
        state.last_updated_at = Some(Instant::now());
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
                {
                    let mut state = state.lock().expect("NetServerState mutex poisoned");
                    state.last_connection_id = connection_id;
                    state.last_handshake = Some(connect_packet.clone());
                    if let Some(connection_id) = connection_id {
                        let connection = state
                            .connection_states
                            .entry(connection_id)
                            .or_insert_with(|| NetConnection::new(String::new()));
                        connection.uuid = connect_packet.uuid.clone();
                        connection.usid = connect_packet.usid.clone();
                        connection.mobile = connect_packet.mobile;
                        connection.has_begun_connecting = true;
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

    fn current_millis() -> i64 {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        millis.min(i64::MAX as u128) as i64
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

    use crate::mindustry::net::{
        packet_ids, ClientPlanSnapshotCallPacket, ClientPlanSnapshotReceivedCallPacket,
        ClientSnapshotCallPacket, Connect, ConnectConfirmCallPacket, ConnectPacket, Disconnect,
        DoneCallback, Host, HostCallback, Net, NetConnection, NetProvider, PacketKind,
        PingCallPacket, PingResponseCallPacket, SentPacket,
    };
    use crate::mindustry::vars::MAX_TCP_SIZE;

    use super::NetServer;

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
        assert_eq!(connection.uuid, "uuid");
        assert_eq!(connection.usid, "usid");
        assert!(!connection.mobile);
        assert!(connection.has_connected);
        assert!(connection.has_begun_connecting);
        assert!(connection.has_disconnected);
        assert_eq!(state.events.len(), 4);
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
}
