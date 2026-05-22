use super::packets::{KickReason, StreamBegin, StreamChunk};
use super::streamable::Streamable;
use crate::mindustry::entities::units::BuildPlan;
use crate::mindustry::vars::MAX_TCP_SIZE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SentPacket {
    KickReason(KickReason),
    KickMessage(String),
    StreamBegin(StreamBegin),
    StreamChunk(StreamChunk),
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetConnection {
    pub address: String,
    pub name: String,
    pub locale: String,
    pub uuid: String,
    pub usid: String,
    pub color: i32,
    pub mobile: bool,
    pub modclient: bool,
    pub kicked: bool,
    pub sync_time: i64,
    pub connect_time: i64,
    pub last_received_client_snapshot: i32,
    pub snapshots_sent: i32,
    pub last_received_client_time: i64,
    pub rejected_requests: Vec<BuildPlan>,
    pub chat_rate: Ratekeeper,
    pub packet_rate: Ratekeeper,
    pub player_added: bool,
    pub has_connected: bool,
    pub has_begun_connecting: bool,
    pub has_disconnected: bool,
    pub view_width: f32,
    pub view_height: f32,
    pub view_x: f32,
    pub view_y: f32,
    pub sent: Vec<(SentPacket, bool)>,
}

impl NetConnection {
    pub fn new(address: impl Into<String>) -> Self {
        let uuid = "AAAAAAAA".to_string();
        Self {
            address: address.into(),
            name: String::new(),
            locale: "en".into(),
            usid: uuid.clone(),
            uuid,
            color: 0,
            mobile: false,
            modclient: false,
            kicked: false,
            sync_time: 0,
            connect_time: 0,
            last_received_client_snapshot: -1,
            snapshots_sent: 0,
            last_received_client_time: 0,
            rejected_requests: Vec::new(),
            chat_rate: Ratekeeper::new(),
            packet_rate: Ratekeeper::new(),
            player_added: false,
            has_connected: false,
            has_begun_connecting: false,
            has_disconnected: false,
            view_width: 0.0,
            view_height: 0.0,
            view_x: 0.0,
            view_y: 0.0,
            sent: Vec::new(),
        }
    }

    pub fn kick(&mut self) {
        self.kick_reason(KickReason::Kick);
    }

    pub fn kick_reason(&mut self, reason: KickReason) {
        if self.kicked {
            return;
        }
        self.send(SentPacket::KickReason(reason), true);
        self.kick_disconnect();
        self.kicked = true;
    }

    pub fn kick_message(&mut self, reason: impl Into<String>) {
        if self.kicked {
            return;
        }
        self.send(SentPacket::KickMessage(reason.into()), true);
        self.kick_disconnect();
        self.kicked = true;
    }

    pub fn kick_disconnect(&mut self) {
        self.close();
    }

    pub fn is_connected(&self) -> bool {
        !self.has_disconnected
    }

    pub fn send_stream(&mut self, stream: Streamable, packet_type: u8) {
        let begin = StreamBegin {
            id: self.next_stream_id(),
            total: stream.stream.len() as i32,
            packet_type,
        };
        let id = begin.id;
        self.send(SentPacket::StreamBegin(begin), true);

        for chunk in stream.stream.chunks(MAX_TCP_SIZE) {
            self.send(
                SentPacket::StreamChunk(StreamChunk {
                    id,
                    data: chunk.to_vec(),
                }),
                true,
            );
        }
    }

    pub fn send(&mut self, packet: SentPacket, reliable: bool) {
        self.sent.push((packet, reliable));
    }

    pub fn close(&mut self) {
        self.has_disconnected = true;
    }

    fn next_stream_id(&self) -> i32 {
        self.sent
            .iter()
            .filter(|(packet, _)| matches!(packet, SentPacket::StreamBegin(_)))
            .count() as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ratekeeper {
    pub occurences: i32,
    pub last_millis: i64,
}

impl Ratekeeper {
    pub const fn new() -> Self {
        Self {
            occurences: 0,
            last_millis: 0,
        }
    }

    pub fn allow(&mut self, window_millis: i64, limit: i32, now_millis: i64) -> bool {
        if window_millis <= 0 || limit <= 0 {
            return false;
        }

        if self.last_millis == 0 || now_millis - self.last_millis > window_millis {
            self.last_millis = now_millis;
            self.occurences = 0;
        }

        self.occurences += 1;
        self.occurences <= limit
    }

    pub fn reset(&mut self) {
        self.occurences = 0;
        self.last_millis = 0;
    }
}

impl Default for Ratekeeper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_stream_splits_by_java_max_tcp_size() {
        let mut con = NetConnection::new("127.0.0.1");
        con.send_stream(Streamable::new(vec![7; MAX_TCP_SIZE + 3]), 2);
        assert!(matches!(con.sent[0].0, SentPacket::StreamBegin(_)));
        assert!(matches!(con.sent[1].0, SentPacket::StreamChunk(_)));
        assert!(matches!(con.sent[2].0, SentPacket::StreamChunk(_)));
        if let SentPacket::StreamChunk(chunk) = &con.sent[1].0 {
            assert_eq!(chunk.data.len(), MAX_TCP_SIZE);
        }
        if let SentPacket::StreamChunk(chunk) = &con.sent[2].0 {
            assert_eq!(chunk.data.len(), 3);
        }
    }

    #[test]
    fn net_connection_initializes_java_rate_limit_state() {
        let con = NetConnection::new("127.0.0.1");
        assert!(con.rejected_requests.is_empty());
        assert_eq!(con.chat_rate, Ratekeeper::new());
        assert_eq!(con.packet_rate, Ratekeeper::new());
    }

    #[test]
    fn ratekeeper_allows_limited_occurrences_per_window() {
        let mut rate = Ratekeeper::new();
        assert!(rate.allow(1_000, 2, 100));
        assert!(rate.allow(1_000, 2, 200));
        assert!(!rate.allow(1_000, 2, 300));
        assert_eq!(rate.occurences, 3);

        assert!(rate.allow(1_000, 2, 1_301));
        assert_eq!(rate.occurences, 1);
        rate.reset();
        assert_eq!(rate, Ratekeeper::new());
    }
}
