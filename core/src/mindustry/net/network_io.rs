use super::Host;
use crate::mindustry::game::Gamemode;
use crate::mindustry::vars::DEFAULT_PORT;
use flate2::{write::ZlibEncoder, Compression};
use std::collections::BTreeMap;
use std::io::{self, Write};

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
    pub player_bytes: Vec<u8>,
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
            player_bytes: Vec::new(),
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

pub fn write_minimal_world_data(connection_id: i32) -> io::Result<Vec<u8>> {
    write_world_data(&NetworkWorldData::bootstrap_for_connection(connection_id))
}

pub fn deflate_world_data(bytes: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(bytes)?;
    encoder.finish()
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

    fn read_string(&mut self) -> Result<String, NetworkIoError> {
        let len = self.read_u8()? as usize;
        let b = self.take(len)?;
        std::str::from_utf8(b)
            .map(str::to_string)
            .map_err(|_| NetworkIoError::InvalidUtf8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
