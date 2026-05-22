use super::packets::{KickReason, StreamBegin, StreamChunk};
use super::streamable::Streamable;
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
}
