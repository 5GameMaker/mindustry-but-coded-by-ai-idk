use super::Host;
use crate::mindustry::game::Gamemode;
use crate::mindustry::vars::DEFAULT_PORT;

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

fn write_string(buffer: &mut Vec<u8>, string: &str, max_len: usize) {
    let bytes = string.as_bytes();
    let len = bytes.len().min(max_len).min(u8::MAX as usize);
    buffer.push(len as u8);
    buffer.extend_from_slice(&bytes[..len]);
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
}
