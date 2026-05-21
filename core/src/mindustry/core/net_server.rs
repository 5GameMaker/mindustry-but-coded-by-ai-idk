use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use crate::mindustry::net::{
    Connect, ConnectPacket, Disconnect, Net, NetConnection, PacketKind, ProviderEvent,
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
    pub fn new(mut net: Net) -> Self {
        let state = Arc::new(Mutex::new(NetServerState::default()));
        let packet_handlers = Arc::new(Mutex::new(Vec::new()));
        let binary_packet_handlers = Arc::new(Mutex::new(Vec::new()));

        Self::install_typed_listeners(&mut net, &state, &packet_handlers);

        Self {
            net: Arc::new(Mutex::new(net)),
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

    pub fn update(&self) {
        let connections = {
            let net = self.net.lock().expect("Net mutex poisoned");
            net.get_connections()
        };

        let mut state = self.state.lock().expect("NetServerState mutex poisoned");
        Self::sync_provider_connections(&mut state, connections);
        state.last_updated_at = Some(Instant::now());
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
                *connection = provider_connection.clone();
                connection.has_connected = has_connected;
                connection.has_begun_connecting = has_begun_connecting;
                connection.has_disconnected |= has_disconnected;
                connection.sent = sent;
            }
        }

        if !matched_any && state.connection_states.len() == 1 && snapshot.len() == 1 {
            if let Some((_, connection)) = state.connection_states.iter_mut().next() {
                let has_connected = connection.has_connected;
                let has_begun_connecting = connection.has_begun_connecting;
                let has_disconnected = connection.has_disconnected;
                let sent = connection.sent.clone();
                *connection = snapshot[0].clone();
                connection.has_connected = has_connected;
                connection.has_begun_connecting = has_begun_connecting;
                connection.has_disconnected |= has_disconnected;
                connection.sent = sent;
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
    use std::sync::{Arc, Mutex};

    use crate::mindustry::net::{Connect, ConnectPacket, Disconnect, PacketKind};

    use super::NetServer;

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
                true,
                PacketKind::Disconnect(Disconnect {
                    reason: "left".into(),
                }),
            );
        }

        assert_eq!(
            *seen.lock().unwrap(),
            vec!["connect", "connect-packet", "disconnect"]
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
        assert_eq!(state.last_disconnect_reason.as_deref(), Some("left"));
        let connection = state.connection_states.get(&12).unwrap();
        assert_eq!(connection.address, "10.0.0.2:6567");
        assert_eq!(connection.uuid, "uuid");
        assert_eq!(connection.usid, "usid");
        assert!(!connection.mobile);
        assert!(!connection.has_connected);
        assert!(connection.has_begun_connecting);
        assert!(connection.has_disconnected);
        assert_eq!(state.events.len(), 3);
    }
}
