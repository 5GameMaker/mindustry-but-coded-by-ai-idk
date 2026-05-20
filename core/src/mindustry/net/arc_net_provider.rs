use super::{
    packet::PacketCodec,
    packets::{
        packet_ids, AnnounceCallPacket, ClearObjectivesCallPacket,
        ClientBinaryPacketReliableCallPacket, ClientBinaryPacketUnreliableCallPacket,
        ClientPacketReliableCallPacket, ClientPacketUnreliableCallPacket,
        ClientPlanSnapshotCallPacket, ClientPlanSnapshotReceivedCallPacket,
        ClientSnapshotCallPacket, CompleteObjectiveCallPacket, ConnectCallPacket,
        ConnectConfirmCallPacket, ConnectPacket, ConstructFinishCallPacket,
        CopyToClipboardCallPacket, DebugStatusClientCallPacket,
        DebugStatusClientUnreliableCallPacket, HideFollowUpMenuCallPacket, HideHudTextCallPacket,
        InfoMessageCallPacket, InfoPopupCallPacket, InfoPopupCallPacket2,
        InfoPopupReliableCallPacket, InfoPopupReliableCallPacket2, InfoToastCallPacket,
        KickCallPacket, KickCallPacket2, LabelCallPacket, LabelCallPacket2,
        LabelReliableCallPacket, LabelReliableCallPacket2, OpenUriCallPacket,
        PingResponseCallPacket, PlayerDisconnectCallPacket, RemoveMarkerCallPacket,
        RemoveQueueBlockCallPacket, SetCameraPositionCallPacket, SetFlagCallPacket,
        SetHudTextCallPacket, SetHudTextReliableCallPacket, SetMapAreaCallPacket,
        SetRuleCallPacket, StreamBegin, StreamChunk, TextInputCallPacket, TextInputCallPacket2,
        WorldDataBeginCallPacket,
    },
    PacketKind,
};
use crate::mindustry::core::content_loader::ContentLoader;

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

    pub fn write_packet_kind_with_loader(
        packet: &PacketKind,
        loader: &ContentLoader,
    ) -> Result<Vec<u8>, SerializerError> {
        let envelope = Self::packet_kind_to_envelope_with_loader(packet, loader)?;
        Ok(Self::write_envelope(&envelope))
    }

    pub fn read_packet_kind(bytes: &[u8]) -> Result<PacketKind, SerializerError> {
        let envelope = Self::read_envelope(bytes)?;
        Self::packet_kind_from_envelope(&envelope)
    }

    pub fn read_packet_kind_with_loader(
        bytes: &[u8],
        loader: &ContentLoader,
    ) -> Result<PacketKind, SerializerError> {
        let envelope = Self::read_envelope(bytes)?;
        Self::packet_kind_from_envelope_with_loader(&envelope, loader)
    }

    pub fn packet_kind_to_envelope(packet: &PacketKind) -> Result<PacketEnvelope, SerializerError> {
        if matches!(
            packet,
            PacketKind::ClientPlanSnapshotCallPacket(_)
                | PacketKind::ClientPlanSnapshotReceivedCallPacket(_)
                | PacketKind::ClientSnapshotCallPacket(_)
                | PacketKind::ConstructFinishCallPacket(_)
        ) {
            return Err(SerializerError::RequiresContentLoader);
        }

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
            PacketKind::ConnectConfirmCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::CONNECT_CONFIRM_CALL_PACKET
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
            PacketKind::InfoPopupCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::INFO_POPUP_CALL_PACKET
            }
            PacketKind::InfoPopupCallPacket2(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::INFO_POPUP_CALL_PACKET2
            }
            PacketKind::InfoPopupReliableCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::INFO_POPUP_RELIABLE_CALL_PACKET
            }
            PacketKind::InfoPopupReliableCallPacket2(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::INFO_POPUP_RELIABLE_CALL_PACKET2
            }
            PacketKind::InfoToastCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::INFO_TOAST_CALL_PACKET
            }
            PacketKind::KickCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::KICK_CALL_PACKET
            }
            PacketKind::KickCallPacket2(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::KICK_CALL_PACKET2
            }
            PacketKind::LabelCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::LABEL_CALL_PACKET
            }
            PacketKind::LabelCallPacket2(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::LABEL_CALL_PACKET2
            }
            PacketKind::LabelReliableCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::LABEL_RELIABLE_CALL_PACKET
            }
            PacketKind::LabelReliableCallPacket2(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::LABEL_RELIABLE_CALL_PACKET2
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
            PacketKind::SetCameraPositionCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::SET_CAMERA_POSITION_CALL_PACKET
            }
            PacketKind::SetFlagCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::SET_FLAG_CALL_PACKET
            }
            PacketKind::SetHudTextCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::SET_HUD_TEXT_CALL_PACKET
            }
            PacketKind::SetHudTextReliableCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::SET_HUD_TEXT_RELIABLE_CALL_PACKET
            }
            PacketKind::SetMapAreaCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::SET_MAP_AREA_CALL_PACKET
            }
            PacketKind::SetRuleCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::SET_RULE_CALL_PACKET
            }
            PacketKind::TextInputCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::TEXT_INPUT_CALL_PACKET
            }
            PacketKind::TextInputCallPacket2(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::TEXT_INPUT_CALL_PACKET2
            }
            PacketKind::WorldDataBeginCallPacket(packet) => {
                packet.write_to(&mut payload)?;
                packet_ids::WORLD_DATA_BEGIN_CALL_PACKET
            }
            PacketKind::Streamable(stream) => {
                payload.extend_from_slice(&stream.stream);
                packet_ids::WORLD_STREAM
            }
            PacketKind::ClientPlanSnapshotCallPacket(_)
            | PacketKind::ClientPlanSnapshotReceivedCallPacket(_)
            | PacketKind::ClientSnapshotCallPacket(_)
            | PacketKind::ConstructFinishCallPacket(_) => {
                return Err(SerializerError::RequiresContentLoader);
            }
            PacketKind::Other { id, .. } => return Err(SerializerError::UnsupportedPacketId(*id)),
        };

        Self::packet_payload_to_envelope(packet, id, payload)
    }

    pub fn packet_kind_to_envelope_with_loader(
        packet: &PacketKind,
        loader: &ContentLoader,
    ) -> Result<PacketEnvelope, SerializerError> {
        let mut payload = Vec::new();
        let id = match packet {
            PacketKind::ClientPlanSnapshotCallPacket(packet) => {
                packet.write_to_with_loader(&mut payload, loader)?;
                packet_ids::CLIENT_PLAN_SNAPSHOT_CALL_PACKET
            }
            PacketKind::ClientPlanSnapshotReceivedCallPacket(packet) => {
                packet.write_to_with_loader(&mut payload, loader)?;
                packet_ids::CLIENT_PLAN_SNAPSHOT_RECEIVED_CALL_PACKET
            }
            PacketKind::ClientSnapshotCallPacket(packet) => {
                packet.write_to_with_loader(&mut payload, loader)?;
                packet_ids::CLIENT_SNAPSHOT_CALL_PACKET
            }
            PacketKind::ConstructFinishCallPacket(packet) => {
                packet.write_to_with_loader(&mut payload, loader)?;
                packet_ids::CONSTRUCT_FINISH_CALL_PACKET
            }
            _ => return Self::packet_kind_to_envelope(packet),
        };

        Self::packet_payload_to_envelope(packet, id, payload)
    }

    fn packet_payload_to_envelope(
        packet: &PacketKind,
        id: u8,
        payload: Vec<u8>,
    ) -> Result<PacketEnvelope, SerializerError> {
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
        if let PacketEnvelope::Packet { id, .. } = envelope {
            if matches!(
                *id,
                packet_ids::CLIENT_PLAN_SNAPSHOT_CALL_PACKET
                    | packet_ids::CLIENT_PLAN_SNAPSHOT_RECEIVED_CALL_PACKET
                    | packet_ids::CLIENT_SNAPSHOT_CALL_PACKET
                    | packet_ids::CONSTRUCT_FINISH_CALL_PACKET
            ) {
                return Err(SerializerError::RequiresContentLoader);
            }
        }

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
                    packet_ids::CONNECT_CONFIRM_CALL_PACKET => {
                        Ok(PacketKind::ConnectConfirmCallPacket(
                            ConnectConfirmCallPacket::read_from(&mut cursor)?,
                        ))
                    }
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
                    packet_ids::INFO_POPUP_CALL_PACKET => Ok(PacketKind::InfoPopupCallPacket(
                        InfoPopupCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::INFO_POPUP_CALL_PACKET2 => Ok(PacketKind::InfoPopupCallPacket2(
                        InfoPopupCallPacket2::read_from(&mut cursor)?,
                    )),
                    packet_ids::INFO_POPUP_RELIABLE_CALL_PACKET => {
                        Ok(PacketKind::InfoPopupReliableCallPacket(
                            InfoPopupReliableCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::INFO_POPUP_RELIABLE_CALL_PACKET2 => {
                        Ok(PacketKind::InfoPopupReliableCallPacket2(
                            InfoPopupReliableCallPacket2::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::INFO_TOAST_CALL_PACKET => Ok(PacketKind::InfoToastCallPacket(
                        InfoToastCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::KICK_CALL_PACKET => Ok(PacketKind::KickCallPacket(
                        KickCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::KICK_CALL_PACKET2 => Ok(PacketKind::KickCallPacket2(
                        KickCallPacket2::read_from(&mut cursor)?,
                    )),
                    packet_ids::LABEL_CALL_PACKET => Ok(PacketKind::LabelCallPacket(
                        LabelCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::LABEL_CALL_PACKET2 => Ok(PacketKind::LabelCallPacket2(
                        LabelCallPacket2::read_from(&mut cursor)?,
                    )),
                    packet_ids::LABEL_RELIABLE_CALL_PACKET => {
                        Ok(PacketKind::LabelReliableCallPacket(
                            LabelReliableCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::LABEL_RELIABLE_CALL_PACKET2 => {
                        Ok(PacketKind::LabelReliableCallPacket2(
                            LabelReliableCallPacket2::read_from(&mut cursor)?,
                        ))
                    }
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
                    packet_ids::SET_CAMERA_POSITION_CALL_PACKET => {
                        Ok(PacketKind::SetCameraPositionCallPacket(
                            SetCameraPositionCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::SET_FLAG_CALL_PACKET => Ok(PacketKind::SetFlagCallPacket(
                        SetFlagCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::SET_HUD_TEXT_CALL_PACKET => Ok(PacketKind::SetHudTextCallPacket(
                        SetHudTextCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::SET_HUD_TEXT_RELIABLE_CALL_PACKET => {
                        Ok(PacketKind::SetHudTextReliableCallPacket(
                            SetHudTextReliableCallPacket::read_from(&mut cursor)?,
                        ))
                    }
                    packet_ids::SET_MAP_AREA_CALL_PACKET => Ok(PacketKind::SetMapAreaCallPacket(
                        SetMapAreaCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::SET_RULE_CALL_PACKET => Ok(PacketKind::SetRuleCallPacket(
                        SetRuleCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::TEXT_INPUT_CALL_PACKET => Ok(PacketKind::TextInputCallPacket(
                        TextInputCallPacket::read_from(&mut cursor)?,
                    )),
                    packet_ids::TEXT_INPUT_CALL_PACKET2 => Ok(PacketKind::TextInputCallPacket2(
                        TextInputCallPacket2::read_from(&mut cursor)?,
                    )),
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

    pub fn packet_kind_from_envelope_with_loader(
        envelope: &PacketEnvelope,
        loader: &ContentLoader,
    ) -> Result<PacketKind, SerializerError> {
        match envelope {
            PacketEnvelope::Packet { id, payload, .. } => {
                let mut cursor = payload.as_slice();
                match *id {
                    packet_ids::CLIENT_PLAN_SNAPSHOT_CALL_PACKET => {
                        Ok(PacketKind::ClientPlanSnapshotCallPacket(
                            ClientPlanSnapshotCallPacket::read_from_with_loader(
                                &mut cursor,
                                loader,
                            )?,
                        ))
                    }
                    packet_ids::CLIENT_PLAN_SNAPSHOT_RECEIVED_CALL_PACKET => {
                        Ok(PacketKind::ClientPlanSnapshotReceivedCallPacket(
                            ClientPlanSnapshotReceivedCallPacket::read_from_with_loader(
                                &mut cursor,
                                loader,
                            )?,
                        ))
                    }
                    packet_ids::CLIENT_SNAPSHOT_CALL_PACKET => {
                        Ok(PacketKind::ClientSnapshotCallPacket(
                            ClientSnapshotCallPacket::read_from_with_loader(&mut cursor, loader)?,
                        ))
                    }
                    packet_ids::CONSTRUCT_FINISH_CALL_PACKET => {
                        Ok(PacketKind::ConstructFinishCallPacket(
                            ConstructFinishCallPacket::read_from_with_loader(&mut cursor, loader)?,
                        ))
                    }
                    _ => Self::packet_kind_from_envelope(envelope),
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
    #[error("packet codec requires ContentLoader")]
    RequiresContentLoader,
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

    use crate::mindustry::{
        core::content_loader::ContentLoader,
        ctype::ContentType,
        io::{BuildPlanWire, ContentRef, TeamId, TypeValue, UnitRef},
        net::packets::{
            ClientBinaryPacketCallPacket, ClientPacketCallPacket, ClientPlanSnapshotCallPacket,
            ClientPlanSnapshotReceivedCallPacket, ClientSnapshotCallPacket,
            ConstructFinishCallPacket,
        },
    };

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
    fn client_plan_snapshot_packets_roundtrip_with_content_loader() {
        let loader = ContentLoader::create_base_content().unwrap();
        let plans = vec![BuildPlanWire::new_place_config(
            10,
            20,
            1,
            "duo",
            TypeValue::Content(ContentRef::new(ContentType::Item, 0)),
        )];
        let packet = PacketKind::ClientPlanSnapshotCallPacket(ClientPlanSnapshotCallPacket {
            group_id: 77,
            plans: Some(plans.clone()),
        });
        assert_eq!(
            PacketSerializer::write_packet_kind(&packet).unwrap_err(),
            SerializerError::RequiresContentLoader
        );
        let bytes = PacketSerializer::write_packet_kind_with_loader(&packet, &loader).unwrap();
        assert_eq!(bytes[0], packet_ids::CLIENT_PLAN_SNAPSHOT_CALL_PACKET);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap_err(),
            SerializerError::RequiresContentLoader
        );
        assert_eq!(
            PacketSerializer::read_packet_kind_with_loader(&bytes, &loader).unwrap(),
            packet
        );

        let received = PacketKind::ClientPlanSnapshotReceivedCallPacket(
            ClientPlanSnapshotReceivedCallPacket {
                player_id: 123,
                group_id: 78,
                plans: Some(plans),
            },
        );
        let bytes = PacketSerializer::write_packet_kind_with_loader(&received, &loader).unwrap();
        assert_eq!(
            bytes[0],
            packet_ids::CLIENT_PLAN_SNAPSHOT_RECEIVED_CALL_PACKET
        );
        assert_eq!(
            PacketSerializer::read_packet_kind_with_loader(&bytes, &loader).unwrap(),
            received
        );
    }

    #[test]
    fn client_snapshot_packet_roundtrips_with_content_loader() {
        let loader = ContentLoader::create_base_content().unwrap();
        let packet = PacketKind::ClientSnapshotCallPacket(ClientSnapshotCallPacket {
            snapshot_id: 11,
            unit_id: 22,
            dead: false,
            x: 1.0,
            y: 2.0,
            pointer_x: 3.0,
            pointer_y: 4.0,
            rotation: 5.0,
            base_rotation: 6.0,
            x_velocity: 7.0,
            y_velocity: 8.0,
            mining: Some(crate::mindustry::world::point2_pack(9, 10)),
            boosting: true,
            shooting: false,
            chatting: true,
            building: false,
            selected_block: Some("duo".into()),
            selected_rotation: 1,
            plans: Some(vec![BuildPlanWire::new_place(12, 13, 2, "router")]),
            view_x: 14.0,
            view_y: 15.0,
            view_width: 16.0,
            view_height: 17.0,
        });
        assert_eq!(
            PacketSerializer::write_packet_kind(&packet).unwrap_err(),
            SerializerError::RequiresContentLoader
        );
        let bytes = PacketSerializer::write_packet_kind_with_loader(&packet, &loader).unwrap();
        assert_eq!(bytes[0], packet_ids::CLIENT_SNAPSHOT_CALL_PACKET);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap_err(),
            SerializerError::RequiresContentLoader
        );
        assert_eq!(
            PacketSerializer::read_packet_kind_with_loader(&bytes, &loader).unwrap(),
            packet
        );
    }

    #[test]
    fn construct_finish_packet_roundtrips_with_content_loader() {
        let loader = ContentLoader::create_base_content().unwrap();
        let packet = PacketKind::ConstructFinishCallPacket(ConstructFinishCallPacket {
            tile: Some(crate::mindustry::world::point2_pack(1, 2)),
            block: Some("router".into()),
            builder: UnitRef::Block {
                tile_pos: crate::mindustry::world::point2_pack(3, 4),
            },
            rotation: 1,
            team: TeamId(6),
            config: TypeValue::Content(ContentRef::new(ContentType::Item, 0)),
        });
        assert_eq!(
            PacketSerializer::write_packet_kind(&packet).unwrap_err(),
            SerializerError::RequiresContentLoader
        );
        let bytes = PacketSerializer::write_packet_kind_with_loader(&packet, &loader).unwrap();
        assert_eq!(bytes[0], packet_ids::CONSTRUCT_FINISH_CALL_PACKET);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap_err(),
            SerializerError::RequiresContentLoader
        );
        assert_eq!(
            PacketSerializer::read_packet_kind_with_loader(&bytes, &loader).unwrap(),
            packet
        );
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

        let confirm = PacketKind::ConnectConfirmCallPacket(ConnectConfirmCallPacket);
        let bytes = PacketSerializer::write_packet_kind(&confirm).unwrap();
        assert_eq!(
            bytes,
            vec![packet_ids::CONNECT_CONFIRM_CALL_PACKET, 0, 0, 0]
        );
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), confirm);

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

        let popup = PacketKind::InfoPopupCallPacket(InfoPopupCallPacket {
            message: Some("popup".into()),
            duration: 2.5,
            align: 1,
            top: 2,
            left: 3,
            bottom: 4,
            right: 5,
        });
        let bytes = PacketSerializer::write_packet_kind(&popup).unwrap();
        assert_eq!(bytes[0], packet_ids::INFO_POPUP_CALL_PACKET);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), popup);

        let popup_with_id = PacketKind::InfoPopupCallPacket2(InfoPopupCallPacket2 {
            message: None,
            id: Some("objective".into()),
            duration: 3.0,
            align: 6,
            top: 7,
            left: 8,
            bottom: 9,
            right: 10,
        });
        let bytes = PacketSerializer::write_packet_kind(&popup_with_id).unwrap();
        assert_eq!(bytes[0], packet_ids::INFO_POPUP_CALL_PACKET2);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            popup_with_id
        );

        let reliable_popup = PacketKind::InfoPopupReliableCallPacket(InfoPopupReliableCallPacket(
            InfoPopupCallPacket {
                message: Some("reliable popup".into()),
                duration: 1.0,
                align: 11,
                top: 12,
                left: 13,
                bottom: 14,
                right: 15,
            },
        ));
        let bytes = PacketSerializer::write_packet_kind(&reliable_popup).unwrap();
        assert_eq!(bytes[0], packet_ids::INFO_POPUP_RELIABLE_CALL_PACKET);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            reliable_popup
        );

        let reliable_popup_with_id = PacketKind::InfoPopupReliableCallPacket2(
            InfoPopupReliableCallPacket2(InfoPopupCallPacket2 {
                message: Some("reliable popup id".into()),
                id: Some("id".into()),
                duration: 1.25,
                align: 16,
                top: 17,
                left: 18,
                bottom: 19,
                right: 20,
            }),
        );
        let bytes = PacketSerializer::write_packet_kind(&reliable_popup_with_id).unwrap();
        assert_eq!(bytes[0], packet_ids::INFO_POPUP_RELIABLE_CALL_PACKET2);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            reliable_popup_with_id
        );

        let kick_string = PacketKind::KickCallPacket(KickCallPacket {
            reason: "custom".into(),
        });
        let bytes = PacketSerializer::write_packet_kind(&kick_string).unwrap();
        assert_eq!(bytes[0], packet_ids::KICK_CALL_PACKET);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            kick_string
        );

        let kick_reason = PacketKind::KickCallPacket2(KickCallPacket2 {
            reason: crate::mindustry::net::KickReason::ServerClose,
        });
        let bytes = PacketSerializer::write_packet_kind(&kick_reason).unwrap();
        assert_eq!(bytes[0], packet_ids::KICK_CALL_PACKET2);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            kick_reason
        );

        let label = PacketKind::LabelCallPacket(LabelCallPacket {
            message: Some("label".into()),
            duration: 1.5,
            world_x: 10.0,
            world_y: -2.0,
        });
        let bytes = PacketSerializer::write_packet_kind(&label).unwrap();
        assert_eq!(bytes[0], packet_ids::LABEL_CALL_PACKET);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), label);

        let label_with_id = PacketKind::LabelCallPacket2(LabelCallPacket2 {
            message: None,
            id: 42,
            duration: 2.0,
            world_x: 4.0,
            world_y: 5.0,
        });
        let bytes = PacketSerializer::write_packet_kind(&label_with_id).unwrap();
        assert_eq!(bytes[0], packet_ids::LABEL_CALL_PACKET2);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            label_with_id
        );

        let reliable_label =
            PacketKind::LabelReliableCallPacket(LabelReliableCallPacket(LabelCallPacket {
                message: Some("reliable label".into()),
                duration: 3.0,
                world_x: 6.0,
                world_y: 7.0,
            }));
        let bytes = PacketSerializer::write_packet_kind(&reliable_label).unwrap();
        assert_eq!(bytes[0], packet_ids::LABEL_RELIABLE_CALL_PACKET);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            reliable_label
        );

        let reliable_label_with_id =
            PacketKind::LabelReliableCallPacket2(LabelReliableCallPacket2(LabelCallPacket2 {
                message: Some("reliable label id".into()),
                id: 43,
                duration: 4.0,
                world_x: 8.0,
                world_y: 9.0,
            }));
        let bytes = PacketSerializer::write_packet_kind(&reliable_label_with_id).unwrap();
        assert_eq!(bytes[0], packet_ids::LABEL_RELIABLE_CALL_PACKET2);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            reliable_label_with_id
        );

        let camera =
            PacketKind::SetCameraPositionCallPacket(SetCameraPositionCallPacket { x: 1.0, y: 2.0 });
        let bytes = PacketSerializer::write_packet_kind(&camera).unwrap();
        assert_eq!(bytes[0], packet_ids::SET_CAMERA_POSITION_CALL_PACKET);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), camera);

        let flag = PacketKind::SetFlagCallPacket(SetFlagCallPacket {
            flag: "objective".into(),
            add: true,
        });
        let bytes = PacketSerializer::write_packet_kind(&flag).unwrap();
        assert_eq!(bytes[0], packet_ids::SET_FLAG_CALL_PACKET);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), flag);

        let hud = PacketKind::SetHudTextCallPacket(SetHudTextCallPacket {
            message: "hud".into(),
        });
        let bytes = PacketSerializer::write_packet_kind(&hud).unwrap();
        assert_eq!(bytes[0], packet_ids::SET_HUD_TEXT_CALL_PACKET);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), hud);

        let reliable_hud = PacketKind::SetHudTextReliableCallPacket(SetHudTextReliableCallPacket(
            SetHudTextCallPacket {
                message: "hud reliable".into(),
            },
        ));
        let bytes = PacketSerializer::write_packet_kind(&reliable_hud).unwrap();
        assert_eq!(bytes[0], packet_ids::SET_HUD_TEXT_RELIABLE_CALL_PACKET);
        assert_eq!(
            PacketSerializer::read_packet_kind(&bytes).unwrap(),
            reliable_hud
        );

        let area = PacketKind::SetMapAreaCallPacket(SetMapAreaCallPacket {
            x: 1,
            y: 2,
            width: 3,
            height: 4,
        });
        let bytes = PacketSerializer::write_packet_kind(&area).unwrap();
        assert_eq!(bytes[0], packet_ids::SET_MAP_AREA_CALL_PACKET);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), area);

        let rule = PacketKind::SetRuleCallPacket(SetRuleCallPacket {
            rule: "unitCap".into(),
            json_data: "{\"value\":42}".into(),
        });
        let bytes = PacketSerializer::write_packet_kind(&rule).unwrap();
        assert_eq!(bytes[0], packet_ids::SET_RULE_CALL_PACKET);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), rule);

        let prompt = PacketKind::TextInputCallPacket(TextInputCallPacket {
            text_input_id: 1,
            title: "title".into(),
            message: "message".into(),
            text_length: 64,
            default_text: "default".into(),
            numeric: false,
        });
        let bytes = PacketSerializer::write_packet_kind(&prompt).unwrap();
        assert_eq!(bytes[0], packet_ids::TEXT_INPUT_CALL_PACKET);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), prompt);

        let prompt2 = PacketKind::TextInputCallPacket2(TextInputCallPacket2 {
            prompt: TextInputCallPacket {
                text_input_id: 2,
                title: "title2".into(),
                message: "message2".into(),
                text_length: 32,
                default_text: String::new(),
                numeric: true,
            },
            allow_empty: true,
        });
        let bytes = PacketSerializer::write_packet_kind(&prompt2).unwrap();
        assert_eq!(bytes[0], packet_ids::TEXT_INPUT_CALL_PACKET2);
        assert_eq!(PacketSerializer::read_packet_kind(&bytes).unwrap(), prompt2);
    }
}
