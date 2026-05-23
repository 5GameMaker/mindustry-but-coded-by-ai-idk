use super::Host;
use crate::mindustry::ctype::ContentId;
use crate::mindustry::game::Gamemode;
use crate::mindustry::io::type_io::{
    read_bool as read_io_bool, read_color, read_command_id, read_f32 as read_io_f32,
    read_i16 as read_io_i16, read_i32 as read_io_i32, read_string as read_io_string, read_team,
    read_unit_ref, TeamId, UnitRef,
};
use crate::mindustry::io::{
    read_chunk_map, read_content_header_snapshot, read_content_patches, read_legacy_team_blocks,
    ContentHeaderSnapshot, ContentPatchSet, LegacyShortChunkMap, LegacyTeamBlocks,
};
use crate::mindustry::vars::DEFAULT_PORT;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::collections::BTreeMap;
use std::io::{self, Read, Write};

#[derive(Debug, Clone, PartialEq)]
pub struct ServerData {
    pub name: String,
    pub map: String,
    pub players: i32,
    pub wave: i32,
    pub version: i32,
    pub version_type: String,
    pub mode: Gamemode,
    pub player_limit: i32,
    pub description: String,
    pub mode_name: Option<String>,
    pub port: u16,
}

/// Parsed wire image of generated Java `mindustry.gen.Player.write(...)`.
///
/// The network world stream writes `player.id` separately and then this
/// generated entity body. Keeping a lossy-but-typed mirror lets the Rust client
/// find the `SaveIO` tail boundary without needing a full generated entity
/// system yet.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkPlayerData {
    pub revision: i16,
    pub admin: bool,
    pub boosting: bool,
    pub color: i32,
    pub last_command_id: Option<ContentId>,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub name: Option<String>,
    pub selected_block_id: Option<ContentId>,
    pub selected_rotation: i32,
    pub shooting: bool,
    pub team: TeamId,
    pub typing: bool,
    pub unit: UnitRef,
    pub x: f32,
    pub y: f32,
}

/// Stage-1 Rust mirror of Java `NetworkIO.writeWorld(...)`.
///
/// Upstream writes this body through `DataOutputStream`, then wraps it in a
/// `DeflaterOutputStream` before sending it as `Packets.WorldStream`.
/// This struct covers the stable front matter now; the SaveIO-backed tail
/// (`writeContentHeader`, `writeContentPatches`, `writeMap`, `writeTeamBlocks`,
/// `writeMarkers`, `writeCustomChunks`) is represented as raw section bytes and
/// must be replaced by real SaveVersion codecs as the migration continues.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkWorldData {
    pub rules_json: String,
    pub map_locales_json: String,
    pub map_tags: BTreeMap<String, String>,
    pub wave: i32,
    pub wave_time: f32,
    pub tick: f64,
    pub rand_seed0: i64,
    pub rand_seed1: i64,
    pub player_id: i32,
    pub player: Option<NetworkPlayerData>,
    pub player_bytes: Vec<u8>,
    pub tail_parse_error: Option<String>,
    pub content_header_snapshot: Option<ContentHeaderSnapshot>,
    pub content_patches_snapshot: Option<ContentPatchSet>,
    pub map_snapshot: Option<LegacyShortChunkMap>,
    pub team_blocks_snapshot: Option<LegacyTeamBlocks>,
    pub content_header: Vec<u8>,
    pub content_patches: Vec<u8>,
    pub map_bytes: Vec<u8>,
    pub team_blocks: Vec<u8>,
    pub markers: Vec<u8>,
    pub custom_chunks: Vec<u8>,
}

impl Default for NetworkWorldData {
    fn default() -> Self {
        Self {
            rules_json: "{}".into(),
            map_locales_json: "{}".into(),
            map_tags: BTreeMap::new(),
            wave: 0,
            wave_time: 0.0,
            tick: 0.0,
            rand_seed0: 0,
            rand_seed1: 0,
            player_id: -1,
            player: None,
            player_bytes: Vec::new(),
            tail_parse_error: None,
            content_header_snapshot: None,
            content_patches_snapshot: None,
            map_snapshot: None,
            team_blocks_snapshot: None,
            content_header: Vec::new(),
            content_patches: Vec::new(),
            map_bytes: Vec::new(),
            team_blocks: Vec::new(),
            markers: Vec::new(),
            custom_chunks: Vec::new(),
        }
    }
}

impl NetworkWorldData {
    pub fn bootstrap_for_connection(connection_id: i32) -> Self {
        let mut map_tags = BTreeMap::new();
        map_tags.insert("name".into(), "Rust Bootstrap".into());
        map_tags.insert(
            "description".into(),
            "Rust network bootstrap world data".into(),
        );

        Self {
            player_id: connection_id,
            map_tags,
            ..Self::default()
        }
    }
}

impl ServerData {
    pub fn to_host(&self, ping: i32, host_address: impl Into<String>) -> Host {
        Host::new(
            ping,
            self.name.clone(),
            host_address,
            self.port as i32,
            self.map.clone(),
            self.wave,
            self.players,
            self.version,
            self.version_type.clone(),
            self.mode,
            self.player_limit,
            self.description.clone(),
            self.mode_name.clone(),
        )
    }
}

pub fn write_server_data(data: &ServerData) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(500);
    write_string(&mut buffer, &data.name, 100);
    write_string(&mut buffer, &data.map, 64);
    buffer.extend_from_slice(&data.players.to_be_bytes());
    buffer.extend_from_slice(&data.wave.to_be_bytes());
    buffer.extend_from_slice(&data.version.to_be_bytes());
    write_string(&mut buffer, &data.version_type, 32);
    buffer.push(gamemode_ordinal(data.mode));
    buffer.extend_from_slice(&data.player_limit.to_be_bytes());
    write_string(&mut buffer, &data.description, 100);
    write_string(&mut buffer, data.mode_name.as_deref().unwrap_or(""), 50);
    buffer.extend_from_slice(&(data.port as i16).to_be_bytes());
    buffer
}

pub fn read_server_data(
    ping: i32,
    host_address: impl Into<String>,
    bytes: &[u8],
) -> Result<Host, NetworkIoError> {
    let data = read_server_payload(bytes)?;
    Ok(data.to_host(ping, host_address))
}

pub fn read_server_payload(bytes: &[u8]) -> Result<ServerData, NetworkIoError> {
    let mut cursor = Cursor::new(bytes);
    let name = cursor.read_string()?;
    let map = cursor.read_string()?;
    let players = cursor.read_i32()?;
    let wave = cursor.read_i32()?;
    let version = cursor.read_i32()?;
    let version_type = cursor.read_string()?;
    let mode = gamemode_from_ordinal(cursor.read_u8()?);
    let player_limit = cursor.read_i32()?;
    let description = cursor.read_string()?;
    let raw_mode_name = cursor.read_string()?;
    let port = cursor.read_i16()?;
    let port = if port != 0 { port as u16 } else { DEFAULT_PORT };

    Ok(ServerData {
        name,
        map,
        players,
        wave,
        version,
        version_type,
        mode,
        player_limit,
        description,
        mode_name: if raw_mode_name.is_empty() {
            None
        } else {
            Some(raw_mode_name)
        },
        port,
    })
}

pub fn write_world_data_raw(data: &NetworkWorldData) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();

    // Java NetworkIO.writeWorld front matter:
    // rules JSON, map-locales JSON, map tags, wave, wavetime, tick, random seeds,
    // player id and serialized player bytes.
    write_java_utf(&mut buffer, &data.rules_json)?;
    write_java_utf(&mut buffer, &data.map_locales_json)?;
    write_java_string_map(&mut buffer, &data.map_tags)?;
    buffer.extend_from_slice(&data.wave.to_be_bytes());
    buffer.extend_from_slice(&data.wave_time.to_bits().to_be_bytes());
    buffer.extend_from_slice(&data.tick.to_bits().to_be_bytes());
    buffer.extend_from_slice(&data.rand_seed0.to_be_bytes());
    buffer.extend_from_slice(&data.rand_seed1.to_be_bytes());
    buffer.extend_from_slice(&data.player_id.to_be_bytes());
    buffer.extend_from_slice(&data.player_bytes);

    // SaveVersion-backed tail. These are placeholders until the real map/save
    // codecs are migrated; keeping them here preserves one explicit runtime
    // insertion point instead of scattering ad-hoc bytes through NetServer.
    buffer.extend_from_slice(&data.content_header);
    buffer.extend_from_slice(&data.content_patches);
    buffer.extend_from_slice(&data.map_bytes);
    buffer.extend_from_slice(&data.team_blocks);
    buffer.extend_from_slice(&data.markers);
    buffer.extend_from_slice(&data.custom_chunks);

    Ok(buffer)
}

pub fn write_world_data(data: &NetworkWorldData) -> io::Result<Vec<u8>> {
    let raw = write_world_data_raw(data)?;
    deflate_world_data(&raw)
}

pub fn read_world_data(bytes: &[u8]) -> Result<NetworkWorldData, NetworkIoError> {
    let raw = inflate_world_data(bytes)?;
    read_world_data_raw(&raw)
}

pub fn read_world_data_raw(bytes: &[u8]) -> Result<NetworkWorldData, NetworkIoError> {
    let mut cursor = Cursor::new(bytes);
    let rules_json = cursor.read_java_utf()?;
    let map_locales_json = cursor.read_java_utf()?;
    let map_tags = cursor.read_java_string_map()?;
    let wave = cursor.read_i32()?;
    let wave_time = f32::from_bits(cursor.read_u32()?);
    let tick = f64::from_bits(cursor.read_u64()?);
    let rand_seed0 = cursor.read_i64()?;
    let rand_seed1 = cursor.read_i64()?;
    let player_id = cursor.read_i32()?;
    let tail = parse_world_tail(cursor.remaining());

    Ok(NetworkWorldData {
        rules_json,
        map_locales_json,
        map_tags,
        wave,
        wave_time,
        tick,
        rand_seed0,
        rand_seed1,
        player_id,
        player: tail.player,
        player_bytes: tail.player_bytes,
        tail_parse_error: tail.tail_parse_error,
        content_header_snapshot: tail.content_header_snapshot,
        content_patches_snapshot: tail.content_patches_snapshot,
        map_snapshot: tail.map_snapshot,
        team_blocks_snapshot: tail.team_blocks_snapshot,
        content_header: tail.content_header,
        content_patches: tail.content_patches,
        map_bytes: tail.map_bytes,
        team_blocks: tail.team_blocks,
        markers: tail.markers,
        custom_chunks: tail.custom_chunks,
    })
}

pub fn write_minimal_world_data(connection_id: i32) -> io::Result<Vec<u8>> {
    write_world_data(&NetworkWorldData::bootstrap_for_connection(connection_id))
}

pub fn deflate_world_data(bytes: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(bytes)?;
    encoder.finish()
}

pub fn inflate_world_data(bytes: &[u8]) -> Result<Vec<u8>, NetworkIoError> {
    let mut decoder = ZlibDecoder::new(bytes);
    let mut raw = Vec::new();
    decoder
        .read_to_end(&mut raw)
        .map_err(|error| NetworkIoError::Io(error.to_string()))?;
    Ok(raw)
}

#[derive(Debug, Clone, Default)]
struct ParsedWorldTail {
    player: Option<NetworkPlayerData>,
    player_bytes: Vec<u8>,
    tail_parse_error: Option<String>,
    content_header_snapshot: Option<ContentHeaderSnapshot>,
    content_patches_snapshot: Option<ContentPatchSet>,
    map_snapshot: Option<LegacyShortChunkMap>,
    team_blocks_snapshot: Option<LegacyTeamBlocks>,
    content_header: Vec<u8>,
    content_patches: Vec<u8>,
    map_bytes: Vec<u8>,
    team_blocks: Vec<u8>,
    markers: Vec<u8>,
    custom_chunks: Vec<u8>,
}

fn parse_world_tail(bytes: &[u8]) -> ParsedWorldTail {
    let mut out = ParsedWorldTail {
        player_bytes: bytes.to_vec(),
        ..ParsedWorldTail::default()
    };

    if bytes.is_empty() {
        return out;
    }

    let mut remaining = bytes;
    let player_start = remaining;
    match read_network_player(&mut remaining) {
        Ok(player) => {
            let consumed = player_start.len() - remaining.len();
            out.player = Some(player);
            out.player_bytes = player_start[..consumed].to_vec();

            if let Err(error) = parse_save_tail(&mut remaining, &mut out) {
                out.tail_parse_error = Some(error.to_string());
                if out.content_header.is_empty()
                    && out.content_patches.is_empty()
                    && out.map_bytes.is_empty()
                    && out.team_blocks.is_empty()
                    && out.markers.is_empty()
                    && out.custom_chunks.is_empty()
                {
                    out.markers = remaining.to_vec();
                }
            }
        }
        Err(error) => {
            // Stage compatibility: older Rust bootstrap payloads and tests may
            // still store opaque player/tail bytes here. Preserve them and
            // report the optional tail parse failure without rejecting the
            // already-valid NetworkIO front matter.
            out.tail_parse_error = Some(error.to_string());
        }
    }

    out
}

fn parse_save_tail(remaining: &mut &[u8], out: &mut ParsedWorldTail) -> io::Result<()> {
    if remaining.is_empty() {
        return Ok(());
    }

    let (bytes, header) = read_section(remaining, |input| read_content_header_snapshot(input))?;
    out.content_header = bytes;
    out.content_header_snapshot = Some(header);

    if remaining.is_empty() {
        return Ok(());
    }

    let (bytes, patches) = read_section(remaining, |input| read_content_patches(input))?;
    out.content_patches = bytes;
    out.content_patches_snapshot = Some(patches);

    if remaining.is_empty() {
        return Ok(());
    }

    let (bytes, map) = read_section(remaining, |input| read_chunk_map(input))?;
    out.map_bytes = bytes;
    out.map_snapshot = Some(map);

    if remaining.is_empty() {
        return Ok(());
    }

    let (bytes, team_blocks) = read_section(remaining, |input| read_legacy_team_blocks(input))?;
    out.team_blocks = bytes;
    out.team_blocks_snapshot = Some(team_blocks);

    // Java follows with `MapMarkers` UBJSON bytes and then custom chunks.
    // The marker codec is not migrated yet, so preserve the remaining tail in
    // one explicit slot instead of losing it or guessing the custom chunk
    // boundary.
    out.markers = remaining.to_vec();
    *remaining = &[];

    Ok(())
}

fn read_section<T, F>(remaining: &mut &[u8], reader: F) -> io::Result<(Vec<u8>, T)>
where
    F: FnOnce(&mut &[u8]) -> io::Result<T>,
{
    let start = *remaining;
    let value = reader(remaining)?;
    let consumed = start.len() - remaining.len();
    Ok((start[..consumed].to_vec(), value))
}

fn read_network_player<R: Read>(read: &mut R) -> io::Result<NetworkPlayerData> {
    let revision = read_io_i16(read)?;
    let admin = read_io_bool(read)?;
    let boosting = read_io_bool(read)?;
    let color = read_color(read)?.rgba();

    let last_command_id = if revision >= 1 {
        read_command_id(read)?
    } else {
        None
    };

    let mouse_x = read_io_f32(read)?;
    let mouse_y = read_io_f32(read)?;
    let name = read_io_string(read)?;

    let (selected_block_id, selected_rotation) = if revision >= 2 {
        let raw = read_io_i16(read)?;
        let selected_block_id = if raw < 0 { None } else { Some(raw) };
        (selected_block_id, read_io_i32(read)?)
    } else {
        (None, 0)
    };

    let shooting = read_io_bool(read)?;
    let team = read_team(read)?;
    let typing = read_io_bool(read)?;
    let unit = read_unit_ref(read)?;
    let x = read_io_f32(read)?;
    let y = read_io_f32(read)?;

    match revision {
        0..=2 => Ok(NetworkPlayerData {
            revision,
            admin,
            boosting,
            color,
            last_command_id,
            mouse_x,
            mouse_y,
            name,
            selected_block_id,
            selected_rotation,
            shooting,
            team,
            typing,
            unit,
            x,
            y,
        }),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown PlayerComp revision {revision}"),
        )),
    }
}

fn write_string(buffer: &mut Vec<u8>, string: &str, max_len: usize) {
    let bytes = string.as_bytes();
    let len = bytes.len().min(max_len).min(u8::MAX as usize);
    buffer.push(len as u8);
    buffer.extend_from_slice(&bytes[..len]);
}

fn write_java_string_map(buffer: &mut Vec<u8>, map: &BTreeMap<String, String>) -> io::Result<()> {
    let len = i16::try_from(map.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Java StringMap cannot contain more than i16::MAX entries",
        )
    })?;
    buffer.extend_from_slice(&len.to_be_bytes());
    for (key, value) in map {
        write_java_utf(buffer, key)?;
        write_java_utf(buffer, value)?;
    }
    Ok(())
}

fn write_java_utf(buffer: &mut Vec<u8>, value: &str) -> io::Result<()> {
    let encoded = java_modified_utf8(value);
    let len = u16::try_from(encoded.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Java DataOutput.writeUTF payload exceeds u16::MAX bytes",
        )
    })?;
    buffer.extend_from_slice(&len.to_be_bytes());
    buffer.extend_from_slice(&encoded);
    Ok(())
}

fn java_modified_utf8(value: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for unit in value.encode_utf16() {
        match unit {
            0x0001..=0x007f => out.push(unit as u8),
            0x0000 | 0x0080..=0x07ff => {
                out.push((0xc0 | ((unit >> 6) & 0x1f)) as u8);
                out.push((0x80 | (unit & 0x3f)) as u8);
            }
            _ => {
                out.push((0xe0 | ((unit >> 12) & 0x0f)) as u8);
                out.push((0x80 | ((unit >> 6) & 0x3f)) as u8);
                out.push((0x80 | (unit & 0x3f)) as u8);
            }
        }
    }
    out
}

fn gamemode_ordinal(mode: Gamemode) -> u8 {
    match mode {
        Gamemode::Survival => 0,
        Gamemode::Sandbox => 1,
        Gamemode::Attack => 2,
        Gamemode::Pvp => 3,
        Gamemode::Editor => 4,
    }
}

fn gamemode_from_ordinal(id: u8) -> Gamemode {
    match id {
        1 => Gamemode::Sandbox,
        2 => Gamemode::Attack,
        3 => Gamemode::Pvp,
        4 => Gamemode::Editor,
        _ => Gamemode::Survival,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NetworkIoError {
    #[error("buffer underflow while reading server data")]
    Underflow,
    #[error("server data contains invalid UTF-8")]
    InvalidUtf8,
    #[error("world data contains invalid Java modified UTF-8")]
    InvalidModifiedUtf8,
    #[error("network IO failed: {0}")]
    Io(String),
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], NetworkIoError> {
        if self.pos + len > self.bytes.len() {
            return Err(NetworkIoError::Underflow);
        }
        let out = &self.bytes[self.pos..self.pos + len];
        self.pos += len;
        Ok(out)
    }

    fn read_u8(&mut self) -> Result<u8, NetworkIoError> {
        Ok(self.take(1)?[0])
    }

    fn read_i16(&mut self) -> Result<i16, NetworkIoError> {
        let b = self.take(2)?;
        Ok(i16::from_be_bytes([b[0], b[1]]))
    }

    fn read_i32(&mut self) -> Result<i32, NetworkIoError> {
        let b = self.take(4)?;
        Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_u32(&mut self) -> Result<u32, NetworkIoError> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_i64(&mut self) -> Result<i64, NetworkIoError> {
        let b = self.take(8)?;
        Ok(i64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn read_u64(&mut self) -> Result<u64, NetworkIoError> {
        let b = self.take(8)?;
        Ok(u64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn read_string(&mut self) -> Result<String, NetworkIoError> {
        let len = self.read_u8()? as usize;
        let b = self.take(len)?;
        std::str::from_utf8(b)
            .map(str::to_string)
            .map_err(|_| NetworkIoError::InvalidUtf8)
    }

    fn read_java_utf(&mut self) -> Result<String, NetworkIoError> {
        let len = self.read_i16()? as u16 as usize;
        let bytes = self.take(len)?;
        decode_java_modified_utf8(bytes)
    }

    fn read_java_string_map(&mut self) -> Result<BTreeMap<String, String>, NetworkIoError> {
        let len = self.read_i16()? as u16 as usize;
        let mut map = BTreeMap::new();
        for _ in 0..len {
            let key = self.read_java_utf()?;
            let value = self.read_java_utf()?;
            map.insert(key, value);
        }
        Ok(map)
    }

    fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.pos..]
    }
}

fn decode_java_modified_utf8(bytes: &[u8]) -> Result<String, NetworkIoError> {
    let mut units = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        let b0 = bytes[index];
        if b0 & 0x80 == 0 {
            units.push(b0 as u16);
            index += 1;
        } else if b0 & 0xe0 == 0xc0 {
            let b1 = *bytes
                .get(index + 1)
                .ok_or(NetworkIoError::InvalidModifiedUtf8)?;
            if b1 & 0xc0 != 0x80 {
                return Err(NetworkIoError::InvalidModifiedUtf8);
            }
            units.push((((b0 & 0x1f) as u16) << 6) | ((b1 & 0x3f) as u16));
            index += 2;
        } else if b0 & 0xf0 == 0xe0 {
            let b1 = *bytes
                .get(index + 1)
                .ok_or(NetworkIoError::InvalidModifiedUtf8)?;
            let b2 = *bytes
                .get(index + 2)
                .ok_or(NetworkIoError::InvalidModifiedUtf8)?;
            if b1 & 0xc0 != 0x80 || b2 & 0xc0 != 0x80 {
                return Err(NetworkIoError::InvalidModifiedUtf8);
            }
            units.push(
                (((b0 & 0x0f) as u16) << 12) | (((b1 & 0x3f) as u16) << 6) | ((b2 & 0x3f) as u16),
            );
            index += 3;
        } else {
            return Err(NetworkIoError::InvalidModifiedUtf8);
        }
    }

    String::from_utf16(&units).map_err(|_| NetworkIoError::InvalidModifiedUtf8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::io::type_io::{
        write_bool as write_io_bool, write_command_id, write_f32 as write_io_f32,
        write_i16 as write_io_i16, write_i32 as write_io_i32, write_string as write_io_string,
        write_team, write_u16 as write_io_u16, write_u8 as write_io_u8, write_unit_ref,
    };
    use crate::mindustry::io::{
        write_content_header_snapshot, write_content_patches, ContentHeaderEntry,
    };
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    #[test]
    fn server_data_roundtrips_java_order() {
        let data = ServerData {
            name: "Server".into(),
            map: "Map".into(),
            players: 5,
            wave: 12,
            version: 157,
            version_type: "release".into(),
            mode: Gamemode::Attack,
            player_limit: 16,
            description: "desc".into(),
            mode_name: Some("custom".into()),
            port: 6567,
        };
        let bytes = write_server_data(&data);
        let host = read_server_data(42, "127.0.0.1", &bytes).unwrap();
        assert_eq!(host.name, "Server");
        assert_eq!(host.mapname, "Map");
        assert_eq!(host.players, 5);
        assert_eq!(host.wave, 12);
        assert_eq!(host.version, 157);
        assert_eq!(host.version_type, "release");
        assert_eq!(host.mode, Gamemode::Attack);
        assert_eq!(host.player_limit, 16);
        assert_eq!(host.description, "desc");
        assert_eq!(host.mode_name.as_deref(), Some("custom"));
        assert_eq!(host.port, 6567);
        assert_eq!(host.ping, 42);
    }

    #[test]
    fn world_data_raw_writes_java_networkio_front_matter_order() {
        let mut map_tags = BTreeMap::new();
        map_tags.insert("name".into(), "Crater".into());
        map_tags.insert("wave".into(), "3".into());
        let data = NetworkWorldData {
            rules_json: "{}".into(),
            map_locales_json: "{\"en\":\"Name\"}".into(),
            map_tags,
            wave: 3,
            wave_time: 12.5,
            tick: 99.25,
            rand_seed0: 7,
            rand_seed1: 8,
            player_id: 42,
            player_bytes: vec![0xaa, 0xbb],
            content_header: vec![0x01],
            content_patches: vec![0x02],
            map_bytes: vec![0x03],
            team_blocks: vec![0x04],
            markers: vec![0x05],
            custom_chunks: vec![0x06],
            ..NetworkWorldData::default()
        };

        let raw = write_world_data_raw(&data).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&[0, 2]);
        expected.extend_from_slice(b"{}");
        expected.extend_from_slice(&[0, 13]);
        expected.extend_from_slice(br#"{"en":"Name"}"#);
        expected.extend_from_slice(&2i16.to_be_bytes());
        expected.extend_from_slice(&[0, 4]);
        expected.extend_from_slice(b"name");
        expected.extend_from_slice(&[0, 6]);
        expected.extend_from_slice(b"Crater");
        expected.extend_from_slice(&[0, 4]);
        expected.extend_from_slice(b"wave");
        expected.extend_from_slice(&[0, 1]);
        expected.extend_from_slice(b"3");
        expected.extend_from_slice(&3i32.to_be_bytes());
        expected.extend_from_slice(&12.5f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&99.25f64.to_bits().to_be_bytes());
        expected.extend_from_slice(&7i64.to_be_bytes());
        expected.extend_from_slice(&8i64.to_be_bytes());
        expected.extend_from_slice(&42i32.to_be_bytes());
        expected.extend_from_slice(&[0xaa, 0xbb, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);

        assert_eq!(raw, expected);
    }

    #[test]
    fn java_modified_utf8_matches_write_utf_edge_cases() {
        let mut bytes = Vec::new();
        write_java_utf(&mut bytes, "a\0é𝄞").unwrap();

        assert_eq!(
            bytes,
            vec![0x00, 0x0b, b'a', 0xc0, 0x80, 0xc3, 0xa9, 0xed, 0xa0, 0xb4, 0xed, 0xb4, 0x9e,]
        );
    }

    #[test]
    fn world_data_is_zlib_deflated_for_java_inflater() {
        let compressed = write_minimal_world_data(17).unwrap();
        assert!(compressed.len() > 2);
        assert_eq!(compressed[0] & 0x0f, 8);

        let mut decoder = ZlibDecoder::new(compressed.as_slice());
        let mut raw = Vec::new();
        decoder.read_to_end(&mut raw).unwrap();

        let expected =
            write_world_data_raw(&NetworkWorldData::bootstrap_for_connection(17)).unwrap();
        assert_eq!(raw, expected);
        assert!(raw.ends_with(&17i32.to_be_bytes()));
    }

    #[test]
    fn world_data_reader_roundtrips_front_matter_and_preserves_unparsed_tail() {
        let mut map_tags = BTreeMap::new();
        map_tags.insert("emoji".into(), "a\0é𝄞".into());
        map_tags.insert("name".into(), "Rust Map".into());
        let data = NetworkWorldData {
            rules_json: "{\"mode\":\"survival\"}".into(),
            map_locales_json: "{}".into(),
            map_tags,
            wave: 9,
            wave_time: 33.5,
            tick: 44.25,
            rand_seed0: 123,
            rand_seed1: 456,
            player_id: 77,
            player_bytes: vec![0x10, 0x20, 0x30],
            ..NetworkWorldData::default()
        };

        let compressed = write_world_data(&data).unwrap();
        let decoded = read_world_data(&compressed).unwrap();

        assert_eq!(decoded.rules_json, data.rules_json);
        assert_eq!(decoded.map_locales_json, data.map_locales_json);
        assert_eq!(decoded.map_tags, data.map_tags);
        assert_eq!(decoded.wave, 9);
        assert_eq!(decoded.wave_time, 33.5);
        assert_eq!(decoded.tick, 44.25);
        assert_eq!(decoded.rand_seed0, 123);
        assert_eq!(decoded.rand_seed1, 456);
        assert_eq!(decoded.player_id, 77);
        assert_eq!(decoded.player_bytes, vec![0x10, 0x20, 0x30]);
    }

    #[test]
    fn world_data_reader_splits_generated_player_and_save_tail_prefix() {
        let mut player = Vec::new();
        write_io_i16(&mut player, 2).unwrap();
        write_io_bool(&mut player, true).unwrap();
        write_io_bool(&mut player, false).unwrap();
        write_io_i32(&mut player, 0x11223344).unwrap();
        write_command_id(&mut player, Some(7)).unwrap();
        write_io_f32(&mut player, 12.0).unwrap();
        write_io_f32(&mut player, 13.0).unwrap();
        write_io_string(&mut player, Some("frog")).unwrap();
        write_io_i16(&mut player, 99).unwrap();
        write_io_i32(&mut player, 3).unwrap();
        write_io_bool(&mut player, true).unwrap();
        write_team(&mut player, Some(TeamId(6))).unwrap();
        write_io_bool(&mut player, false).unwrap();
        write_unit_ref(&mut player, UnitRef::Unit { id: 123 }).unwrap();
        write_io_f32(&mut player, 40.0).unwrap();
        write_io_f32(&mut player, 41.0).unwrap();

        let content_header_snapshot = ContentHeaderSnapshot {
            entries: vec![ContentHeaderEntry {
                content_type: 1,
                names: vec!["copper".into()],
            }],
        };
        let mut content_header = Vec::new();
        write_content_header_snapshot(&mut content_header, &content_header_snapshot).unwrap();

        let content_patches_snapshot = ContentPatchSet {
            patches: vec![b"patch".to_vec()],
        };
        let mut content_patches = Vec::new();
        write_content_patches(&mut content_patches, &content_patches_snapshot).unwrap();

        let mut map_bytes = Vec::new();
        write_io_u16(&mut map_bytes, 1).unwrap();
        write_io_u16(&mut map_bytes, 1).unwrap();
        write_io_i16(&mut map_bytes, 2).unwrap();
        write_io_i16(&mut map_bytes, 0).unwrap();
        write_io_u8(&mut map_bytes, 0).unwrap();
        write_io_i16(&mut map_bytes, 0).unwrap();
        write_io_u8(&mut map_bytes, 0).unwrap();
        write_io_u8(&mut map_bytes, 0).unwrap();

        let mut team_blocks = Vec::new();
        write_io_i32(&mut team_blocks, 0).unwrap();

        let marker_and_custom_tail = vec![0xaa, 0xbb, 0xcc];
        let data = NetworkWorldData {
            player_id: 42,
            player_bytes: player.clone(),
            content_header: content_header.clone(),
            content_patches: content_patches.clone(),
            map_bytes: map_bytes.clone(),
            team_blocks: team_blocks.clone(),
            markers: marker_and_custom_tail.clone(),
            ..NetworkWorldData::default()
        };

        let decoded = read_world_data(&write_world_data(&data).unwrap()).unwrap();
        let parsed_player = decoded.player.as_ref().expect("player body parsed");

        assert_eq!(decoded.player_bytes, player);
        assert_eq!(parsed_player.revision, 2);
        assert!(parsed_player.admin);
        assert_eq!(parsed_player.color, 0x11223344);
        assert_eq!(parsed_player.last_command_id, Some(7));
        assert_eq!(parsed_player.name.as_deref(), Some("frog"));
        assert_eq!(parsed_player.selected_block_id, Some(99));
        assert_eq!(parsed_player.selected_rotation, 3);
        assert_eq!(parsed_player.team, TeamId(6));
        assert_eq!(parsed_player.unit, UnitRef::Unit { id: 123 });

        assert!(decoded.tail_parse_error.is_none());
        assert_eq!(decoded.content_header, content_header);
        assert_eq!(
            decoded.content_header_snapshot.as_ref(),
            Some(&content_header_snapshot)
        );
        assert_eq!(decoded.content_patches, content_patches);
        assert_eq!(
            decoded.content_patches_snapshot.as_ref(),
            Some(&content_patches_snapshot)
        );
        assert_eq!(decoded.map_bytes, map_bytes);
        let map = decoded.map_snapshot.as_ref().expect("map parsed");
        assert_eq!(map.width, 1);
        assert_eq!(map.height, 1);
        assert_eq!(map.tile_count(), 1);
        assert_eq!(decoded.team_blocks, team_blocks);
        assert_eq!(
            decoded
                .team_blocks_snapshot
                .as_ref()
                .expect("team blocks parsed")
                .total_plans(),
            0
        );
        assert_eq!(decoded.markers, marker_and_custom_tail);
    }
}
