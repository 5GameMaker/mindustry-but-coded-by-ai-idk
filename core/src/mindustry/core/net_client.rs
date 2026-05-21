use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::mindustry::net::{
    Connect, ConnectConfirmCallPacket, ConnectPacket, Disconnect, Net, PacketKind, ProviderEvent,
    Streamable,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_CLIENT_VERSION: i32 = 157;
pub const DEFAULT_CLIENT_VERSION_TYPE: &str = "official";

pub type PacketHandler = Arc<dyn Fn(PacketKind) + Send + Sync + 'static>;
pub type BinaryPacketHandler = Arc<dyn Fn(&[u8]) + Send + Sync + 'static>;

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
        self.connect_packet_sent = false;
        self.last_sent_connect_packet = None;
        self.last_connect_packet_error = None;
        self.connect_confirm_sent = false;
        self.last_connect_confirm_error = None;
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
    }

    fn record_world_stream(&mut self, stream: &Streamable) {
        self.connecting = false;
        self.connected = true;
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
        let now = Instant::now();
        state.timeout_resets += 1;
        state.last_timeout_reset_at = Some(now);
        state.timeout_deadline = Some(now + DEFAULT_TIMEOUT);
    }

    pub fn set_connect_config(&self, config: Option<ClientConnectConfig>) {
        let mut state = self.state.lock().unwrap();
        state.connect_config = config;
        state.connect_packet_sent = false;
        state.last_sent_connect_packet = None;
        state.last_connect_packet_error = None;
        state.connect_confirm_sent = false;
        state.last_connect_confirm_error = None;
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
                if let PacketKind::Streamable(stream) = &packet {
                    state.last_binary_stream = Some(stream.stream.clone());
                    if state.auto_confirm_world_stream && !state.connect_confirm_sent {
                        state.connect_confirm_sent = true;
                        state.last_connect_confirm_error = None;
                        true
                    } else {
                        false
                    }
                } else {
                    false
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
    use std::io;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use crate::mindustry::net::{
        Connect, Disconnect, DoneCallback, Host, HostCallback, Net, NetConnection, NetProvider,
        PacketKind, StreamBegin, StreamChunk, Streamable, WorldDataBeginCallPacket,
    };

    use super::{ClientConnectConfig, NetClient};

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
