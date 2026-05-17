use super::{
    packet::PacketCodec,
    packets::{
        packet_ids, AnnounceCallPacket, ClearObjectivesCallPacket,
        ClientBinaryPacketReliableCallPacket, ClientBinaryPacketUnreliableCallPacket,
        ClientPacketReliableCallPacket, ClientPacketUnreliableCallPacket,
        CompleteObjectiveCallPacket, ConnectCallPacket, ConnectPacket, CopyToClipboardCallPacket,
        DebugStatusClientCallPacket, DebugStatusClientUnreliableCallPacket,
        HideFollowUpMenuCallPacket, HideHudTextCallPacket, InfoMessageCallPacket,
        InfoToastCallPacket, KickCallPacket, OpenUriCallPacket, PingResponseCallPacket,
        PlayerDisconnectCallPacket, RemoveMarkerCallPacket, RemoveQueueBlockCallPacket,
        StreamBegin, StreamChunk, WorldDataBeginCallPacket,
    },
    PacketKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameworkMessage {
    Ping { id: i32, is_reply: bool },
    DiscoverHost,
    KeepAlive,
    RegisterUdp { connection_id: i32 },
    RegisterTcp { connection_id: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketEnvelope {
    Framework(FrameworkMessage),
    Packet {
        id: u8,
        length: u16,
        compression: u8,
        payload: Vec<u8>,
    },
    Raw(Vec<u8>),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PacketSerializer;

impl PacketSerializer {
    pub const FRAMEWORK_ID: u8 = 0xfe;
    pub const COMPRESSION_NONE: u8 = 0;
    pub const COMPRESSION_LZ4: u8 = 1;
    pub const COMPRESS_THRESHOLD: usize = 36;

    pub fn write_framework(message: &FrameworkMessage, out: &mut Vec<u8>) {
        match message {
            FrameworkMessage::Ping { id, is_reply } => {
                out.push(0);
                out.extend_from_slice(&id.to_be_bytes());
                out.push(if *is_reply { 1 } else { 0 });
            }
            FrameworkMessage::DiscoverHost => out.push(1),
            FrameworkMessage::KeepAlive => out.push(2),
            FrameworkMessage::RegisterUdp { connection_id } => {
                out.push(3);
                out.extend_from_slice(&connection_id.to_be_bytes());
            }
            FrameworkMessage::RegisterTcp { connection_id } => {
                out.push(4);
                out.extend_from_slice(&connection_id.to_be_bytes());
            }
        }
    }

    pub fn read_framework(bytes: &[u8]) -> Result<FrameworkMessage, SerializerError> {
        let mut cursor = Cursor::new(bytes);
        let id = cursor.u8()?;
        match id {
            0 => Ok(FrameworkMessage::Ping {
                id: cursor.i32()?,
                is_reply: cursor.u8()? == 1,
            }),
            1 => Ok(FrameworkMessage::DiscoverHost),
            2 => Ok(FrameworkMessage::KeepAlive),
            3 => Ok(FrameworkMessage::RegisterUdp {
                connection_id: cursor.i32()?,
            }),
            4 => Ok(FrameworkMessage::RegisterTcp {
                connection_id: cursor.i32()?,
            }),
            _ => Err(SerializerError::UnknownFrameworkMessage(id)),
        }
    }

    pub fn write_envelope(envelope: &PacketEnvelope) -> Vec<u8> {
        match envelope {
            PacketEnvelope::Raw(bytes) => bytes.clone(),
            PacketEnvelope::Framework(message) => {
                let mut out = vec![Self::FRAMEWORK_ID];
                Self::write_framework(message, &mut out);
                out
            }
            PacketEnvelope::Packet {
                id,
                length: _,
                compression,
                payload,
            } => {
                let mut out = vec![*id];
                out.extend_from_slice(&(payload.len() as u16).to_be_bytes());
                out.push(*compression);
                if *compression == Self::COMPRESSION_LZ4 {
                    let compressed = lz4_flex::block::compress(payload);
                    out.extend_from_slice(&compressed);
                } else {
                    out.extend_from_slice(payload);
                }
                out
            }
        }
    }

    pub fn read_envelope(bytes: &[u8]) -> Result<PacketEnvelope, SerializerError> {
        let mut cursor = Cursor::new(bytes);
        let id = cursor.u8()?;
        if id == Self::FRAMEWORK_ID {
            return Ok(PacketEnvelope::Framework(Self::read_framework(
                cursor.remaining(),
            )?));
        }
        let length = cursor.u16()?;
        let compression = cursor.u8()?;
        let payload = if compression == Self::COMPRESSION_NONE {
            cursor.take(length as usize)?.to_vec()
        } else if compression == Self::COMPRESSION_LZ4 {
            lz4_flex::block::decompress(cursor.remaining(), length as usize)
                .map_err(|err| SerializerError::Compression(err.to_string()))?
        } else {
            return Err(SerializerError::UnknownCompression(compression));
        };
        Ok(PacketEnvelope::Packet {
            id,
            length,
            compression,
            payload,
        })
    }

    pub fn write_packet_kind(packet: &PacketKind) -> Result<Vec<u8>, SerializerError> {
        let envelope = Self::packet_kind_to_envelope(packet)?;
        Ok(Self::write_envelope(&envelope))
    }

    pub fn read_packet_kind(bytes: &[u8]) -> Result<PacketKind, SerializerError> {
        let envelope = Self::read_envelope(bytes)?;
        Self::packet_kind_from_envelope(&envelope)
    }

    pub fn packet_kind_to_envelope(packet: &PacketKind) -> Result<PacketEnvelope, SerializerError> {
        let mut payload = Vec::new();
        let id = match packet {
            PacketKind::StreamBegin(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::STREAM_BEGIN
            }
            PacketKind::StreamChunk(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::STREAM_CHUNK
            }
            PacketKind::ConnectPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::CONNECT_PACKET
            }
            PacketKind::AnnounceCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::ANNOUNCE_CALL_PACKET
            }
            PacketKind::ClearObjectivesCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::CLEAR_OBJECTIVES_CALL_PACKET
            }
            PacketKind::ClientBinaryPacketReliableCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::CLIENT_BINARY_PACKET_RELIABLE_CALL_PACKET
            }
            PacketKind::ClientBinaryPacketUnreliableCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::CLIENT_BINARY_PACKET_UNRELIABLE_CALL_PACKET
            }
            PacketKind::ClientPacketReliableCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::CLIENT_PACKET_RELIABLE_CALL_PACKET
            }
            PacketKind::ClientPacketUnreliableCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::CLIENT_PACKET_UNRELIABLE_CALL_PACKET
            }
            PacketKind::CompleteObjectiveCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::COMPLETE_OBJECTIVE_CALL_PACKET
            }
            PacketKind::ConnectCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::CONNECT_CALL_PACKET
            }
            PacketKind::CopyToClipboardCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::COPY_TO_CLIPBOARD_CALL_PACKET
            }
            PacketKind::DebugStatusClientCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::DEBUG_STATUS_CLIENT_CALL_PACKET
            }
            PacketKind::DebugStatusClientUnreliableCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET
            }
            PacketKind::HideFollowUpMenuCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::HIDE_FOLLOW_UP_MENU_CALL_PACKET
            }
            PacketKind::HideHudTextCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::HIDE_HUD_TEXT_CALL_PACKET
            }
            PacketKind::InfoMessageCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::INFO_MESSAGE_CALL_PACKET
            }
            PacketKind::InfoToastCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::INFO_TOAST_CALL_PACKET
            }
            PacketKind::KickCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::KICK_CALL_PACKET
            }
            PacketKind::OpenUriCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::OPEN_URI_CALL_PACKET
            }
            PacketKind::PingResponseCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::PING_RESPONSE_CALL_PACKET
            }
            PacketKind::PlayerDisconnectCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::PLAYER_DISCONNECT_CALL_PACKET
            }
            PacketKind::RemoveMarkerCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::REMOVE_MARKER_CALL_PACKET
            }
            PacketKind::RemoveQueueBlockCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::REMOVE_QUEUE_BLOCK_CALL_PACKET
            }
            PacketKind::WorldDataBeginCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::WORLD_DATA_BEGIN_CALL_PACKET
            }
            PacketKind::Streamable(stream) => {
                payload.extend_from_slice(&stream.stream);
                packet_ids::WORLD_STREAM
            }
            PacketKind::Other { id, .. } => return Err(SerializerError::UnsupportedPacketId(*id)),
        };

        Ok(PacketEnvelope::Packet {
            id,
            length: payload
                .len()
                .try_into()
                .map_err(|_| SerializerError::PayloadTooLarge(payload.len()))?,
            compression: if payload.len() < Self::COMPRESS_THRESHOLD
                || matches!(packet, PacketKind::StreamChunk(_))
            {
                Self::COMPRESSION_NONE
            } else {
                Self::COMPRESSION_LZ4
            },
            payload,
        })
    }

    pub fn packet_kind_from_envelope(
        envelope: &PacketEnvelope,
    ) -> Result<PacketKind, SerializerError> {
        match envelope {
            PacketEnvelope::Packet { id, payload, .. } => {
                let mut cursor = payload.as_slice();
                match *id {
                    packet_ids::STREAM_BEGIN => Ok(PacketKind::StreamBegin(
                        StreamBegin::read_from(&mut cursor)?,
                    )),
                    packet_ids::STREAM_CHUNK => Ok(PacketKind::StreamChunk(
                        StreamChunk::read_from(&mut cursor)?,
                    )),
                    packet_ids::WORLD_STREAM => Ok(PacketKind::Streamable(
                        crate::mindustry::net::streamable::Streamable {
                            stream: payload.clone(),
                        },
                    )),
                    packet_ids::CONNECT_PACKET => Ok(PacketKind::ConnectPacket(
                        ConnectPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::ANNOUNCE_CALL_PACKET => Ok(PacketKind::AnnounceCallPacket(
                        AnnounceCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::CLEAR_OBJECTIVES_CALL_PACKET => {
                        Ok(PacketKind::ClearObjectivesCallPacket(
                            ClearObjectivesCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::CLIENT_BINARY_PACKET_RELIABLE_CALL_PACKET => {
                        Ok(PacketKind::ClientBinaryPacketReliableCallPacket(
                            ClientBinaryPacketReliableCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::CLIENT_BINARY_PACKET_UNRELIABLE_CALL_PACKET => {
                        Ok(PacketKind::ClientBinaryPacketUnreliableCallPacket(
                            ClientBinaryPacketUnreliableCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::CLIENT_PACKET_RELIABLE_CALL_PACKET => {
                        Ok(PacketKind::ClientPacketReliableCallPacket(
                            ClientPacketReliableCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::CLIENT_PACKET_UNRELIABLE_CALL_PACKET => {
                        Ok(PacketKind::ClientPacketUnreliableCallPacket(
                            ClientPacketUnreliableCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::COMPLETE_OBJECTIVE_CALL_PACKET => {
                        Ok(PacketKind::CompleteObjectiveCallPacket(
                            CompleteObjectiveCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::CONNECT_CALL_PACKET => Ok(PacketKind::ConnectCallPacket(
                        ConnectCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::COPY_TO_CLIPBOARD_CALL_PACKET => {
                        Ok(PacketKind::CopyToClipboardCallPacket(
                            CopyToClipboardCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::DEBUG_STATUS_CLIENT_CALL_PACKET => {
                        Ok(PacketKind::DebugStatusClientCallPacket(
                            DebugStatusClientCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::DEBUG_STATUS_CLIENT_UNRELIABLE_CALL_PACKET => {
                        Ok(PacketKind::DebugStatusClientUnreliableCallPacket(
                            DebugStatusClientUnreliableCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::HIDE_FOLLOW_UP_MENU_CALL_PACKET => {
                        Ok(PacketKind::HideFollowUpMenuCallPacket(
                            HideFollowUpMenuCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::HIDE_HUD_TEXT_CALL_PACKET => Ok(PacketKind::HideHudTextCallPacket(
                        HideHudTextCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::INFO_MESSAGE_CALL_PACKET => Ok(PacketKind::InfoMessageCallPacket(
                        InfoMessageCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::INFO_TOAST_CALL_PACKET => Ok(PacketKind::InfoToastCallPacket(
                        InfoToastCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::KICK_CALL_PACKET => Ok(PacketKind::KickCallPacket(
                        KickCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::OPEN_URI_CALL_PACKET => Ok(PacketKind::OpenUriCallPacket(
                        OpenUriCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::PING_RESPONSE_CALL_PACKET => {
                        Ok(PacketKind::PingResponseCallPacket(
                            PingResponseCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::PLAYER_DISCONNECT_CALL_PACKET => {
                        Ok(PacketKind::PlayerDisconnectCallPacket(
                            PlayerDisconnectCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::REMOVE_MARKER_CALL_PACKET => {
                        Ok(PacketKind::RemoveMarkerCallPacket(
                            RemoveMarkerCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::REMOVE_QUEUE_BLOCK_CALL_PACKET => {
                        Ok(PacketKind::RemoveQueueBlockCallPacket(
                            RemoveQueueBlockCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::WORLD_DATA_BEGIN_CALL_PACKET => {
                        Ok(PacketKind::WorldDataBeginCallPacket(
                            WorldDataBeginCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    other => Err(SerializerError::UnsupportedPacketId(other)),
                }
            }
            PacketEnvelope::Framework(_) => Err(SerializerError::ExpectedPacketEnvelope),
            PacketEnvelope::Raw(_) => Err(SerializerError::ExpectedPacketEnvelope),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SerializerError {
    #[error("buffer underflow while reading packet serializer bytes")]
    Underflow,
    #[error("unknown framework message id {0}")]
    UnknownFrameworkMessage(u8),
    #[error("unsupported packet id {0}")]
    UnsupportedPacketId(u8),
    #[error("unknown packet compression id {0}")]
    UnknownCompression(u8),
    #[error("packet compression error: {0}")]
    Compression(String),
    #[error("packet payload too large for Java unsigned short length: {0}")]
    PayloadTooLarge(usize),
    #[error("expected normal packet envelope")]
    ExpectedPacketEnvelope,
    #[error("packet codec IO error: {0}")]
    Io(String),
}

impl From<std::io::Error> for SerializerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.pos..]
    }
    fn take(&mut self, len: usize) -> Result<&'a [u8], SerializerError> {
        if self.pos + len > self.bytes.len() {
            return Err(SerializerError::Underflow);
        }
        let out = &self.bytes[self.pos..self.pos + len];
        self.pos += len;
        Ok(out)
    }
    fn u8(&mut self) -> Result<u8, SerializerError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, SerializerError> {
        let b = self.take(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }
    fn i32(&mut self) -> Result<i32, SerializerError> {
        let b = self.take(4)?;
        Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    use crate::mindustry::net::packets::{ClientBinaryPacketCallPacket, ClientPacketCallPacket};

    #[test]
    fn framework_ping_matches_java_layout() {
        let msg = FrameworkMessage::Ping {
            id: 42,
            is_reply: true,
        };
        let bytes = PacketSerializer::write_envelope(&PacketEnvelope::Framework(msg.clone()));
        assert_eq!(bytes, vec![0xfe, 0, 0, 0, 0, 42, 1]);
        assert_eq!(
            PacketSerializer::read_envelope(&bytes).unwrap(),
            PacketEnvelope::Framework(msg)
        );
    }

    #[test]
    fn packet_envelope_matches_java_header_layout() {
        let env = PacketEnvelope::Packet {
            id: 4,
            length: 0,
            compression: 0,
            payload: vec![9, 8],
        };
        let bytes = PacketSerializer::write_envelope(&env);
        assert_eq!(bytes, vec![4, 0, 2, 0, 9, 8]);
        assert_eq!(
            PacketSerializer::read_envelope(&bytes).unwrap(),
            PacketEnvelope::Packet {
                id: 4,
                length: 2,
                compression: 0,
                payload: vec![9, 8]
            }
        );
    }

    #[test]
    fn uncompressed_packet_envelope_consumes_declared_length_only() {
        let bytes = vec![4, 0, 2, 0, 9, 8, 7, 6];
        assert_eq!(
            PacketSerializer::read_envelope(&bytes).unwrap(),
            PacketEnvelope::Packet {
                id: 4,
                length: 2,
                compression: 0,
                payload: vec![9, 8]
            }
        );
    }

    #[test]
    fn uncompressed_packet_envelope_rejects_short_payloads() {
        let bytes = vec![4, 0, 3, 0, 9, 8];
        assert_eq!(
            PacketSerializer::read_envelope(&bytes).unwrap_err(),
            SerializerError::Underflow
        );
    }

    #[test]
    fn packet_kind_stream_begin_roundtrips_through_java_envelope() {
        let packet = PacketKind::StreamBegin(StreamBegin {
            id: 9,
            total: 12,
            packet_type: packet_ids::WORLD_STREAM,
        });
        let bytes = PacketSerializer::write_packet_kind(&packet).unwrap();
        assert_eq!(bytes, vec![0, 0, 9, 0, 0, 0, 0, 9, 0, 0, 0, 12, 2]);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), packet);
    }

    #[test]
    fn packet_kind_stream_chunk_roundtrips_through_java_envelope() {
        let packet = PacketKind::StreamChunk(StreamChunk {
            id: 7,
            data: vec![4, 5, 6],
        });
        let bytes = PacketSerializer::write_packet_kind(&packet).unwrap();
        assert_eq!(bytes, vec![1, 0, 9, 0, 0, 0, 0, 7, 0, 3, 4, 5, 6]);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), packet);
    }

    #[test]
    fn packet_kind_connect_packet_uses_manifest_id_without_changing_codec() {
        let packet = ConnectPacket {
            version: 157,
            version_type: "official".into(),
            mods: Vec::new(),
            name: "p".into(),
            locale: "en_US".into(),
            uuid: base64::engine::general_purpose::STANDARD.encode([1, 2, 3, 4, 5, 6, 7, 8]),
            usid: "usid".into(),
            mobile: false,
            color: 7,
            uuid_crc32: None,
        };
        let kind = PacketKind::ConnectPacket(packet);
        let bytes = PacketSerializer::write_packet_kind(&kind).unwrap();
        assert_eq!(bytes[0], packet_ids::CONNECT_PACKET);
        let envelope = PacketSerializer::read_envelope(&bytes).unwrap();
        match envelope {
            PacketEnvelope::Packet {
                id,
                compression,
                payload,
                ..
            } => {
                assert_eq!(id, packet_ids::CONNECT_PACKET);
                assert_eq!(compression, PacketSerializer::COMPRESSION_LZ4);
                assert!(!payload.is_empty());
            }
            other => panic!("unexpected envelope: {other:?}"),
        }
    }

    #[test]
    fn lz4_packet_envelope_uses_declared_uncompressed_length() {
        let payload: Vec<u8> = (0..96).map(|i| (i % 7) as u8).collect();
        let envelope = PacketEnvelope::Packet {
            id: packet_ids::WORLD_STREAM,
            length: payload.len() as u16,
            compression: PacketSerializer::COMPRESSION_LZ4,
            payload: payload.clone(),
        };
        let bytes = PacketSerializer::write_envelope(&envelope);
        assert_eq!(bytes[0], packet_ids::WORLD_STREAM);
        assert_eq!(
            u16::from_be_bytes([bytes[1], bytes[2]]) as usize,
            payload.len()
        );
        assert_eq!(bytes[3], PacketSerializer::COMPRESSION_LZ4);
        assert_ne!(&bytes[4..], payload.as_slice());

        assert_eq!(
            PacketSerializer::read_envelope(&bytes).unwrap(),
            PacketEnvelope::Packet {
                id: packet_ids::WORLD_STREAM,
                length: payload.len() as u16,
                compression: PacketSerializer::COMPRESSION_LZ4,
                payload,
            }
        );
    }

    #[test]
    fn packet_kind_world_stream_is_compressed_above_java_threshold() {
        let stream = crate::mindustry::net::streamable::Streamable {
            stream: vec![3; PacketSerializer::COMPRESS_THRESHOLD + 8],
        };
        let kind = PacketKind::Streamable(stream.clone());
        let bytes = PacketSerializer::write_packet_kind(&kind).unwrap();
        assert_eq!(bytes[0], packet_ids::WORLD_STREAM);
        assert_eq!(bytes[3], PacketSerializer::COMPRESSION_LZ4);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            PacketKind::Streamable(stream)
        );
    }

    #[test]
    fn packet_kind_stream_chunk_remains_uncompressed_above_threshold() {
        let packet = PacketKind::StreamChunk(StreamChunk {
            id: 99,
            data: vec![5; PacketSerializer::COMPRESS_THRESHOLD + 20],
        });
        let bytes = PacketSerializer::write_packet_kind(&packet).unwrap();
        assert_eq!(bytes[0], packet_ids::STREAM_CHUNK);
        assert_eq!(bytes[3], PacketSerializer::COMPRESSION_NONE);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), packet);
    }

    #[test]
    fn generated_call_packets_roundtrip_through_java_envelope() {
        let announce = PacketKind::AnnounceCallPacket(AnnounceCallPacket {
            message: "server announcement".into(),
        });
        let bytes = PacketSerializer::write_packet_kind(&announce).unwrap();
        assert_eq!(bytes[0], packet_ids::ANNOUNCE_CALL_PACKET);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            announce
        );

        let clear = PacketKind::ClearObjectivesCallPacket(ClearObjectivesCallPacket);
        let bytes = PacketSerializer::write_packet_kind(&clear).unwrap();
        assert_eq!(bytes[0], packet_ids::CLEAR_OBJECTIVES_CALL_PACKET);
        assert_eq!(bytes[1], 0);
        assert_eq!(bytes[2], 0);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), clear);

        let binary = PacketKind::ClientBinaryPacketReliableCallPacket(
            ClientBinaryPacketReliableCallPacket(ClientBinaryPacketCallPacket {
                packet_type: "mod".into(),
                contents: vec![9; 64],
            }),
        );
        let bytes = PacketSerializer::write_packet_kind(&binary).unwrap();
        assert_eq!(
            bytes[0],
            packet_ids::CLIENT_BINARY_PACKET_RELIABLE_CALL_PACKET
        );
        assert_eq!(bytes[3], PacketSerializer::COMPRESSION_LZ4);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), binary);

        let string_packet = PacketKind::ClientPacketUnreliableCallPacket(
            ClientPacketUnreliableCallPacket(ClientPacketCallPacket {
                packet_type: "chat".into(),
                contents: "hello".into(),
            }),
        );
        let bytes = PacketSerializer::write_packet_kind(&string_packet).unwrap();
        assert_eq!(bytes[0], packet_ids::CLIENT_PACKET_UNRELIABLE_CALL_PACKET);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            string_packet
        );

        let complete =
            PacketKind::CompleteObjectiveCallPacket(CompleteObjectiveCallPacket { index: 3 });
        let bytes = PacketSerializer::write_packet_kind(&complete).unwrap();
        assert_eq!(bytes[0], packet_ids::COMPLETE_OBJECTIVE_CALL_PACKET);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            complete
        );

        let connect = PacketKind::ConnectCallPacket(ConnectCallPacket {
            ip: "localhost".into(),
            port: 6567,
        });
        let bytes = PacketSerializer::write_packet_kind(&connect).unwrap();
        assert_eq!(bytes[0], packet_ids::CONNECT_CALL_PACKET);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), connect);

        let clipboard = PacketKind::CopyToClipboardCallPacket(CopyToClipboardCallPacket {
            text: "copied text".into(),
        });
        let bytes = PacketSerializer::write_packet_kind(&clipboard).unwrap();
        assert_eq!(bytes[0], packet_ids::COPY_TO_CLIPBOARD_CALL_PACKET);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            clipboard
        );

        let debug = PacketKind::DebugStatusClientCallPacket(DebugStatusClientCallPacket {
            value: 1,
            last_client_snapshot: 2,
            snapshots_sent: 3,
        });
        let bytes = PacketSerializer::write_packet_kind(&debug).unwrap();
        assert_eq!(bytes[0], packet_ids::DEBUG_STATUS_CLIENT_CALL_PACKET);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), debug);
    }
}
