//! Compatibility helpers for upstream `mindustry.io.versions.LegacyIO`.

use std::io::{self, Read};

use crate::mindustry::io::type_io::{read_i32, read_java_utf};

pub const LEGACY_SERVER_LIST_SETTING: &str = "server-list";

/// Upstream `LegacyIO.unitMap`, mapping removed pre-v6 unit names to their
/// modern content names.
pub const LEGACY_UNIT_MAP: &[(&str, &str)] = &[
    ("titan", "mace"),
    ("chaos-array", "scepter"),
    ("eradicator", "reign"),
    ("eruptor", "atrax"),
    ("wraith", "flare"),
    ("ghoul", "horizon"),
    ("revenant", "zenith"),
    ("lich", "antumbra"),
    ("reaper", "eclipse"),
    ("draug", "mono"),
    ("phantom", "poly"),
    ("spirit", "poly"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyServer {
    pub ip: String,
    pub port: i32,
}

impl LegacyServer {
    pub fn new(ip: impl Into<String>, port: i32) -> Self {
        Self {
            ip: ip.into(),
            port,
        }
    }
}

pub fn legacy_unit_name(name: &str) -> &str {
    LEGACY_UNIT_MAP
        .iter()
        .find_map(|(old, new)| (*old == name).then_some(*new))
        .unwrap_or(name)
}

pub fn read_legacy_servers(bytes: &[u8]) -> Vec<LegacyServer> {
    read_legacy_servers_from(&mut bytes.as_ref())
}

pub fn read_legacy_servers_result<R: Read>(read: &mut R) -> io::Result<Vec<LegacyServer>> {
    let length = read_i32(read)?;
    let mut servers = Vec::new();
    if length <= 0 {
        return Ok(servers);
    }

    // Java reads and ignores the serialized element type name before entries.
    let _type_name = read_java_utf(read)?;

    for _ in 0..length {
        servers.push(LegacyServer {
            ip: read_java_utf(read)?,
            port: read_i32(read)?,
        });
    }

    Ok(servers)
}

pub fn read_legacy_servers_from<R: Read>(read: &mut R) -> Vec<LegacyServer> {
    let Ok(length) = read_i32(read) else {
        return Vec::new();
    };
    let mut servers = Vec::new();
    if length <= 0 {
        return servers;
    }

    if read_java_utf(read).is_err() {
        return servers;
    }

    for _ in 0..length {
        let Ok(ip) = read_java_utf(read) else {
            return servers;
        };
        let Ok(port) = read_i32(read) else {
            return servers;
        };
        servers.push(LegacyServer { ip, port });
    }

    servers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::io::type_io::{write_i32, write_java_utf};

    #[test]
    fn legacy_unit_map_matches_upstream_renames() {
        assert_eq!(legacy_unit_name("titan"), "mace");
        assert_eq!(legacy_unit_name("chaos-array"), "scepter");
        assert_eq!(legacy_unit_name("spirit"), "poly");
        assert_eq!(legacy_unit_name("dagger"), "dagger");
        assert_eq!(LEGACY_UNIT_MAP.len(), 12);
    }

    #[test]
    fn read_legacy_servers_matches_java_data_input_shape() {
        let mut bytes = Vec::new();
        write_i32(&mut bytes, 2).unwrap();
        write_java_utf(&mut bytes, "mindustry.ui.dialogs.JoinDialog$Server").unwrap();
        write_java_utf(&mut bytes, "127.0.0.1").unwrap();
        write_i32(&mut bytes, 6567).unwrap();
        write_java_utf(&mut bytes, "example.org").unwrap();
        write_i32(&mut bytes, 1234).unwrap();

        assert_eq!(
            read_legacy_servers(&bytes),
            vec![
                LegacyServer::new("127.0.0.1", 6567),
                LegacyServer::new("example.org", 1234),
            ]
        );
    }

    #[test]
    fn read_legacy_servers_ignores_non_positive_lengths_like_java() {
        for length in [0, -1] {
            let mut bytes = Vec::new();
            write_i32(&mut bytes, length).unwrap();
            assert!(read_legacy_servers(&bytes).is_empty());
        }
    }

    #[test]
    fn read_legacy_servers_tolerates_corrupt_settings_and_keeps_prefix() {
        let mut bytes = Vec::new();
        write_i32(&mut bytes, 2).unwrap();
        write_java_utf(&mut bytes, "Server").unwrap();
        write_java_utf(&mut bytes, "ok").unwrap();
        write_i32(&mut bytes, 10).unwrap();
        write_java_utf(&mut bytes, "missing-port").unwrap();

        assert_eq!(
            read_legacy_servers(&bytes),
            vec![LegacyServer::new("ok", 10)]
        );
        assert!(read_legacy_servers(&[1, 2, 3]).is_empty());
    }

    #[test]
    fn strict_reader_reports_truncated_server_entries() {
        let mut bytes = Vec::new();
        write_i32(&mut bytes, 1).unwrap();
        write_java_utf(&mut bytes, "Server").unwrap();
        write_java_utf(&mut bytes, "missing-port").unwrap();

        assert!(read_legacy_servers_result(&mut bytes.as_slice()).is_err());
    }
}
