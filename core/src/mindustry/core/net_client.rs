use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::mindustry::net::{Connect, Disconnect, Net, PacketKind, ProviderEvent, Streamable};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

pub type PacketHandler = Arc<dyn Fn(PacketKind) + Send + Sync + 'static>;
pub type BinaryPacketHandler = Arc<dyn Fn(&[u8]) + Send + Sync + 'static>;

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
    pub last_binary_stream: Option<Vec<u8>>,
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
    fn record_connect(&mut self, connect: &Connect) {
        self.connecting = false;
        self.connected = true;
        self.connect_events += 1;
        self.last_connect = Some(connect.clone());
        self.last_packet = Some(PacketKind::Connect(connect.clone()));
    }

    fn record_disconnect(&mut self, disconnect: &Disconnect) {
        self.connecting = false;
        self.connected = false;
        self.disconnect_events += 1;
        self.last_disconnect = Some(disconnect.clone());
        self.last_packet = Some(PacketKind::Disconnect(disconnect.clone()));
    }

    fn record_world_stream(&mut self, stream: &Streamable) {
        self.world_stream_events += 1;
        self.last_world_stream = Some(stream.clone());
        self.last_binary_stream = Some(stream.stream.clone());
        self.last_packet = Some(PacketKind::Streamable(stream.clone()));
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
        let state = Arc::new(Mutex::new(NetClientState::default()));
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
        let now = Instant::now();
        state.timeout_resets += 1;
        state.last_timeout_reset_at = Some(now);
        state.timeout_deadline = Some(now + DEFAULT_TIMEOUT);
    }

    pub fn begin_connecting(&self) {
        let mut state = self.state.lock().unwrap();
        state.connecting = true;
        state.connected = false;
        state.connection_attempts += 1;
        state.last_update_at = Some(Instant::now());
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

    pub fn update(&self) {
        let cursor = {
            let state = self.state.lock().unwrap();
            state.handled_client_cursor
        };

        let (provider_events, handled_packets) = {
            let mut net = self.net.lock().unwrap();
            let provider_events = net.drain_provider_events();
            let handled_packets = net.handled_client_packets().to_vec();
            (provider_events, handled_packets)
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
            (
                state.quiet,
                state.packet_handlers.clone(),
                state.binary_packet_handlers.clone(),
            )
        };

        for packet in new_packets {
            {
                let mut state = self.state.lock().unwrap();
                state.last_packet = Some(packet.clone());
                if let PacketKind::Streamable(stream) = &packet {
                    state.last_binary_stream = Some(stream.stream.clone());
                }
            }

            if quiet {
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
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::mindustry::net::{Connect, Disconnect, PacketKind, Streamable};

    use super::NetClient;

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
}
