use std::collections::{HashMap, VecDeque};

use super::packets::{
    AnnounceCallPacket, ClearObjectivesCallPacket, ClientBinaryPacketReliableCallPacket,
    ClientBinaryPacketUnreliableCallPacket, ClientPacketReliableCallPacket,
    ClientPacketUnreliableCallPacket, CompleteObjectiveCallPacket, ConnectCallPacket,
    ConnectPacket, CopyToClipboardCallPacket, DebugStatusClientCallPacket,
    DebugStatusClientUnreliableCallPacket, StreamBegin, StreamChunk,
};
use super::streamable::{StreamBuilder, Streamable};

#[derive(Debug, Clone, PartialEq)]
pub enum PacketKind {
    StreamBegin(StreamBegin),
    StreamChunk(StreamChunk),
    Streamable(Streamable),
    ConnectPacket(ConnectPacket),
    AnnounceCallPacket(AnnounceCallPacket),
    ClearObjectivesCallPacket(ClearObjectivesCallPacket),
    ClientBinaryPacketReliableCallPacket(ClientBinaryPacketReliableCallPacket),
    ClientBinaryPacketUnreliableCallPacket(ClientBinaryPacketUnreliableCallPacket),
    ClientPacketReliableCallPacket(ClientPacketReliableCallPacket),
    ClientPacketUnreliableCallPacket(ClientPacketUnreliableCallPacket),
    CompleteObjectiveCallPacket(CompleteObjectiveCallPacket),
    ConnectCallPacket(ConnectCallPacket),
    CopyToClipboardCallPacket(CopyToClipboardCallPacket),
    DebugStatusClientCallPacket(DebugStatusClientCallPacket),
    DebugStatusClientUnreliableCallPacket(DebugStatusClientUnreliableCallPacket),
    Other {
        id: u8,
        priority: i32,
        allow_client: bool,
        allow_server: bool,
    },
}

impl PacketKind {
    pub fn priority(&self) -> i32 {
        match self {
            PacketKind::StreamBegin(_)
            | PacketKind::StreamChunk(_)
            | PacketKind::Streamable(_)
            | PacketKind::ConnectPacket(_) => 2,
            PacketKind::AnnounceCallPacket(_)
            | PacketKind::ClearObjectivesCallPacket(_)
            | PacketKind::ClientBinaryPacketReliableCallPacket(_)
            | PacketKind::ClientBinaryPacketUnreliableCallPacket(_)
            | PacketKind::ClientPacketReliableCallPacket(_)
            | PacketKind::ClientPacketUnreliableCallPacket(_)
            | PacketKind::CompleteObjectiveCallPacket(_)
            | PacketKind::ConnectCallPacket(_)
            | PacketKind::CopyToClipboardCallPacket(_) => 1,
            PacketKind::DebugStatusClientCallPacket(_)
            | PacketKind::DebugStatusClientUnreliableCallPacket(_) => 2,
            PacketKind::Other { priority, .. } => *priority,
        }
    }

    pub fn allow(&self, server: bool) -> bool {
        match self {
            PacketKind::StreamBegin(_) | PacketKind::StreamChunk(_) | PacketKind::Streamable(_) => {
                !server
            }
            PacketKind::ConnectPacket(_)
            | PacketKind::AnnounceCallPacket(_)
            | PacketKind::ClearObjectivesCallPacket(_)
            | PacketKind::CompleteObjectiveCallPacket(_)
            | PacketKind::CopyToClipboardCallPacket(_)
            | PacketKind::DebugStatusClientCallPacket(_)
            | PacketKind::DebugStatusClientUnreliableCallPacket(_) => true,
            PacketKind::ConnectCallPacket(_) => !server,
            PacketKind::ClientBinaryPacketReliableCallPacket(_)
            | PacketKind::ClientBinaryPacketUnreliableCallPacket(_)
            | PacketKind::ClientPacketReliableCallPacket(_)
            | PacketKind::ClientPacketUnreliableCallPacket(_) => server,
            PacketKind::Other {
                allow_client,
                allow_server,
                ..
            } => {
                if server {
                    *allow_server
                } else {
                    *allow_client
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Net {
    server: bool,
    active: bool,
    client_loaded: bool,
    current_stream: Option<StreamBuilder>,
    packet_queue: VecDeque<PacketKind>,
    streams: HashMap<i32, StreamBuilder>,
    handled_client_packets: Vec<PacketKind>,
    handled_server_packets: Vec<PacketKind>,
}

impl Net {
    pub fn set_client_loaded(&mut self, loaded: bool) {
        self.client_loaded = loaded;
        if loaded {
            let queued: Vec<_> = self.packet_queue.drain(..).collect();
            for packet in queued {
                self.handle_client_received(packet);
            }
        } else {
            self.packet_queue.clear();
        }
    }

    pub fn set_client_connected(&mut self) {
        self.active = true;
        self.server = false;
    }

    pub fn mark_server_active(&mut self) {
        self.active = true;
        self.server = true;
    }

    pub fn active(&self) -> bool {
        self.active
    }
    pub fn server(&self) -> bool {
        self.server && self.active
    }
    pub fn client(&self) -> bool {
        !self.server && self.active
    }
    pub fn current_stream(&self) -> Option<&StreamBuilder> {
        self.current_stream.as_ref()
    }
    pub fn queued_len(&self) -> usize {
        self.packet_queue.len()
    }
    pub fn handled_client_packets(&self) -> &[PacketKind] {
        &self.handled_client_packets
    }
    pub fn handled_server_packets(&self) -> &[PacketKind] {
        &self.handled_server_packets
    }

    pub fn handle_client_received(&mut self, packet: PacketKind) {
        if !packet.allow(false) {
            return;
        }

        match packet {
            PacketKind::StreamBegin(begin) => {
                let builder = StreamBuilder::new(begin.id, begin.packet_type, begin.total);
                self.current_stream = Some(builder.clone());
                self.streams.insert(begin.id, builder);
            }
            PacketKind::StreamChunk(chunk) => {
                let mut completed = None;
                if let Some(builder) = self.streams.get_mut(&chunk.id) {
                    builder.add(&chunk.data);
                    self.current_stream = Some(builder.clone());
                    if builder.is_done() {
                        completed = Some(builder.clone());
                    }
                } else {
                    panic!("Received stream chunk without a StreamBegin beforehand!");
                }
                if let Some(builder) = completed {
                    self.streams.remove(&builder.id);
                    self.current_stream = None;
                    self.handle_client_received(PacketKind::Streamable(builder.build()));
                }
            }
            packet => {
                let priority = packet.priority();
                if self.client_loaded || priority == 2 {
                    self.handled_client_packets.push(packet);
                } else if priority != 0 {
                    self.packet_queue.push_back(packet);
                }
            }
        }
    }

    pub fn handle_server_received(&mut self, connection_has_connected: bool, packet: PacketKind) {
        if !packet.allow(true) {
            return;
        }
        if connection_has_connected || packet.priority() == 2 {
            self.handled_server_packets.push(packet);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_client_packet_queues_until_loaded() {
        let mut net = Net::default();
        net.handle_client_received(PacketKind::Other {
            id: 9,
            priority: 1,
            allow_client: true,
            allow_server: true,
        });
        assert_eq!(net.queued_len(), 1);
        assert!(net.handled_client_packets().is_empty());
        net.set_client_loaded(true);
        assert_eq!(net.queued_len(), 0);
        assert_eq!(net.handled_client_packets().len(), 1);
    }

    #[test]
    fn client_stream_reassembles_to_streamable() {
        let mut net = Net::default();
        net.handle_client_received(PacketKind::StreamBegin(StreamBegin {
            id: 1,
            total: 3,
            packet_type: 2,
        }));
        net.handle_client_received(PacketKind::StreamChunk(StreamChunk {
            id: 1,
            data: vec![1, 2],
        }));
        assert!(net.handled_client_packets().is_empty());
        net.handle_client_received(PacketKind::StreamChunk(StreamChunk {
            id: 1,
            data: vec![3],
        }));
        assert_eq!(net.handled_client_packets().len(), 1);
        match &net.handled_client_packets()[0] {
            PacketKind::Streamable(stream) => assert_eq!(stream.stream, vec![1, 2, 3]),
            other => panic!("unexpected packet: {other:?}"),
        }
    }
}
