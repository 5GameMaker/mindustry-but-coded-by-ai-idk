use std::{
    collections::BTreeMap,
    io::{self, Read, Write},
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

use crate::mindustry::{
    game::Rules,
    io::type_io::{
        read_i32, read_i64, read_java_utf, read_u16, read_u8, write_i32, write_i64, write_java_utf,
        write_u16,
    },
};

pub const SAVE_HEADER: &[u8; 4] = b"MSAV";
pub const LATEST_SAVE_VERSION: i32 = 11;
pub const SAVE_REGION_META: &str = "meta";
pub const SAVE_REGION_CONTENT: &str = "content";
pub const SAVE_REGION_PATCHES: &str = "patches";
pub const SAVE_REGION_MAP: &str = "map";
pub const SAVE_REGION_ENTITIES: &str = "entities";
pub const SAVE_REGION_MARKERS: &str = "markers";
pub const SAVE_REGION_CUSTOM: &str = "custom";
pub const CUSTOM_CHUNK_STATIC_FOG_DATA: &str = "static-fog-data";

pub const SAVE_REGION_MANIFEST: &[&str] = &[
    SAVE_REGION_META,
    SAVE_REGION_CONTENT,
    SAVE_REGION_PATCHES,
    SAVE_REGION_MAP,
    SAVE_REGION_ENTITIES,
    SAVE_REGION_MARKERS,
    SAVE_REGION_CUSTOM,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveRegion {
    Meta,
    Content,
    Patches,
    Map,
    Entities,
    Markers,
    Custom,
}

impl SaveRegion {
    pub const fn as_str(self) -> &'static str {
        match self {
            SaveRegion::Meta => SAVE_REGION_META,
            SaveRegion::Content => SAVE_REGION_CONTENT,
            SaveRegion::Patches => SAVE_REGION_PATCHES,
            SaveRegion::Map => SAVE_REGION_MAP,
            SaveRegion::Entities => SAVE_REGION_ENTITIES,
            SaveRegion::Markers => SAVE_REGION_MARKERS,
            SaveRegion::Custom => SAVE_REGION_CUSTOM,
        }
    }

    pub const fn manifest() -> &'static [SaveRegion] {
        &[
            SaveRegion::Meta,
            SaveRegion::Content,
            SaveRegion::Patches,
            SaveRegion::Map,
            SaveRegion::Entities,
            SaveRegion::Markers,
            SaveRegion::Custom,
        ]
    }

    pub const fn manifest_for_version(version: i32) -> &'static [SaveRegion] {
        if version >= 11 {
            &[
                SaveRegion::Meta,
                SaveRegion::Content,
                SaveRegion::Patches,
                SaveRegion::Map,
                SaveRegion::Entities,
                SaveRegion::Markers,
                SaveRegion::Custom,
            ]
        } else if version >= 8 {
            &[
                SaveRegion::Meta,
                SaveRegion::Content,
                SaveRegion::Map,
                SaveRegion::Entities,
                SaveRegion::Markers,
                SaveRegion::Custom,
            ]
        } else {
            &[
                SaveRegion::Meta,
                SaveRegion::Content,
                SaveRegion::Map,
                SaveRegion::Entities,
                SaveRegion::Custom,
            ]
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSaveRegion {
    pub region: SaveRegion,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSaveEnvelope {
    pub version: i32,
    pub regions: Vec<RawSaveRegion>,
}

impl RawSaveEnvelope {
    pub fn new(version: i32) -> Self {
        Self {
            version,
            regions: SaveRegion::manifest_for_version(version)
                .iter()
                .copied()
                .map(|region| RawSaveRegion {
                    region,
                    payload: Vec::new(),
                })
                .collect(),
        }
    }

    pub fn get(&self, region: SaveRegion) -> Option<&[u8]> {
        self.regions
            .iter()
            .find(|entry| entry.region == region)
            .map(|entry| entry.payload.as_slice())
    }

    pub fn set(&mut self, region: SaveRegion, payload: Vec<u8>) -> io::Result<()> {
        let entry = self
            .regions
            .iter_mut()
            .find(|entry| entry.region == region)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "region {} is not present for save version {}",
                        region.as_str(),
                        self.version
                    ),
                )
            })?;
        entry.payload = payload;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentHeaderEntry {
    pub content_type: u8,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentHeaderSnapshot {
    pub entries: Vec<ContentHeaderEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SaveMeta {
    pub version: i32,
    pub build: i32,
    pub timestamp: i64,
    pub time_played: i64,
    pub map_name: Option<String>,
    pub wave: i32,
    pub rules: Rules,
    pub rules_json: String,
    pub tags: BTreeMap<String, String>,
    pub mods: Vec<String>,
}

impl SaveMeta {
    pub fn from_tags(tags: BTreeMap<String, String>) -> Self {
        let rules_json = tags.get("rules").cloned().unwrap_or_else(|| "{}".into());
        let mods = parse_json_string_array(tags.get("mods").map(String::as_str).unwrap_or("[]"));
        Self {
            version: parse_i32(&tags, "version"),
            build: parse_i32(&tags, "build"),
            timestamp: parse_i64(&tags, "saved"),
            time_played: parse_i64(&tags, "playtime"),
            map_name: tags.get("mapname").cloned(),
            wave: parse_i32(&tags, "wave"),
            rules: Rules::default(),
            rules_json,
            tags,
            mods,
        }
    }
}

pub fn write_header<W: Write>(write: &mut W, version: i32) -> io::Result<()> {
    write.write_all(SAVE_HEADER)?;
    write_i32(write, version)
}

pub fn read_header<R: Read>(read: &mut R) -> io::Result<i32> {
    let mut header = [0; 4];
    read.read_exact(&mut header)?;
    if &header != SAVE_HEADER {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "incorrect Mindustry save header",
        ));
    }
    read_i32(read)
}

pub fn write_raw_save_envelope<W: Write>(
    write: &mut W,
    envelope: &RawSaveEnvelope,
) -> io::Result<()> {
    write_header(write, envelope.version)?;
    let expected = SaveRegion::manifest_for_version(envelope.version);
    if envelope.regions.len() != expected.len()
        || envelope
            .regions
            .iter()
            .map(|entry| entry.region)
            .ne(expected.iter().copied())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "raw save envelope regions do not match version manifest",
        ));
    }
    for region in &envelope.regions {
        write_region(write, |payload| {
            payload.write_all(&region.payload)?;
            Ok(())
        })?;
    }
    Ok(())
}

pub fn read_raw_save_envelope<R: Read>(read: &mut R) -> io::Result<RawSaveEnvelope> {
    let version = read_header(read)?;
    let mut regions = Vec::new();
    for region in SaveRegion::manifest_for_version(version) {
        regions.push(RawSaveRegion {
            region: *region,
            payload: read_chunk(read)?,
        });
    }
    Ok(RawSaveEnvelope { version, regions })
}

pub fn write_deflated_raw_save_envelope<W: Write>(
    write: W,
    envelope: &RawSaveEnvelope,
) -> io::Result<()> {
    let mut encoder = ZlibEncoder::new(write, Compression::default());
    write_raw_save_envelope(&mut encoder, envelope)?;
    encoder.finish().map(|_| ())
}

pub fn read_deflated_raw_save_envelope<R: Read>(read: R) -> io::Result<RawSaveEnvelope> {
    let mut decoder = ZlibDecoder::new(read);
    read_raw_save_envelope(&mut decoder)
}

/// Reads the Java save metadata prefix from an already-inflated stream.
///
/// This mirrors `SaveIO.getMeta(DataInputStream)`: it reads `MSAV`, the save
/// version integer, then only the first versioned region/chunk (`meta`). It does
/// not require the map, entities, markers, or custom chunks to be present.
pub fn read_save_meta<R: Read>(read: &mut R) -> io::Result<SaveMeta> {
    let _save_version = read_header(read)?;
    read_meta_region(read)
}

/// Writes the minimum inflated prefix consumed by Java `SaveIO.getMeta`.
pub fn write_save_meta_prefix<W: Write>(
    write: &mut W,
    version: i32,
    tags: &BTreeMap<String, String>,
) -> io::Result<()> {
    write_header(write, version)?;
    write_meta_region(write, tags)
}

pub fn write_deflated_save_meta_prefix<W: Write>(
    write: W,
    version: i32,
    tags: &BTreeMap<String, String>,
) -> io::Result<()> {
    let mut encoder = ZlibEncoder::new(write, Compression::default());
    write_save_meta_prefix(&mut encoder, version, tags)?;
    encoder.finish().map(|_| ())
}

pub fn read_meta_payload<R: Read>(read: &mut R) -> io::Result<SaveMeta> {
    read_string_map(read).map(SaveMeta::from_tags)
}

pub fn read_deflated_save_meta<R: Read>(read: R) -> io::Result<SaveMeta> {
    let mut decoder = ZlibDecoder::new(read);
    read_save_meta(&mut decoder)
}

pub fn read_deflated_save_meta_with_backup(
    primary: &[u8],
    backup: Option<&[u8]>,
) -> io::Result<SaveMeta> {
    match read_deflated_save_meta(primary) {
        Ok(meta) => Ok(meta),
        Err(primary_error) => {
            if let Some(backup) = backup {
                read_deflated_save_meta(backup)
            } else {
                Err(primary_error)
            }
        }
    }
}

pub fn is_deflated_save_valid<R: Read>(read: R) -> bool {
    read_deflated_save_meta(read).is_ok()
}

pub fn write_chunk<W, F>(write: &mut W, f: F) -> io::Result<()>
where
    W: Write,
    F: FnOnce(&mut Vec<u8>) -> io::Result<()>,
{
    let mut payload = Vec::new();
    f(&mut payload)?;
    write_i32(write, payload.len() as i32)?;
    write.write_all(&payload)
}

pub fn read_chunk<R: Read>(read: &mut R) -> io::Result<Vec<u8>> {
    let len = read_i32(read)?;
    if len < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative chunk length",
        ));
    }
    let mut payload = vec![0; len as usize];
    read.read_exact(&mut payload)?;
    Ok(payload)
}

pub fn write_region<W, F>(write: &mut W, f: F) -> io::Result<()>
where
    W: Write,
    F: FnOnce(&mut Vec<u8>) -> io::Result<()>,
{
    write_chunk(write, f)
}

pub fn read_region<R, T, F>(read: &mut R, f: F) -> io::Result<T>
where
    R: Read,
    F: FnOnce(&mut &[u8]) -> io::Result<T>,
{
    let payload = read_chunk(read)?;
    let mut slice = payload.as_slice();
    let out = f(&mut slice)?;
    if !slice.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "region reader did not consume entire payload",
        ));
    }
    Ok(out)
}

pub fn write_string_map<W: Write>(write: &mut W, map: &BTreeMap<String, String>) -> io::Result<()> {
    if map.len() > u16::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "StringMap too large",
        ));
    }
    write_u16(write, map.len() as u16)?;
    for (key, value) in map {
        write_java_utf(write, key)?;
        write_java_utf(write, value)?;
    }
    Ok(())
}

pub fn read_string_map<R: Read>(read: &mut R) -> io::Result<BTreeMap<String, String>> {
    let len = read_u16(read)? as usize;
    let mut out = BTreeMap::new();
    for _ in 0..len {
        let key = read_java_utf(read)?;
        let value = read_java_utf(read)?;
        out.insert(key, value);
    }
    Ok(out)
}

pub fn write_meta_region<W: Write>(
    write: &mut W,
    tags: &BTreeMap<String, String>,
) -> io::Result<()> {
    write_region(write, |payload| write_string_map(payload, tags))
}

pub fn read_meta_region<R: Read>(read: &mut R) -> io::Result<SaveMeta> {
    read_region(read, |payload| read_meta_payload(payload))
}

pub fn write_content_header_snapshot<W: Write>(
    write: &mut W,
    snapshot: &ContentHeaderSnapshot,
) -> io::Result<()> {
    if snapshot.entries.len() > u8::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "content header entry count exceeds Java byte range",
        ));
    }
    write.write_all(&[snapshot.entries.len() as u8])?;
    for entry in &snapshot.entries {
        if entry.names.len() > u16::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "content header name list too large",
            ));
        }
        write.write_all(&[entry.content_type])?;
        write_u16(write, entry.names.len() as u16)?;
        for name in &entry.names {
            write_java_utf(write, name)?;
        }
    }
    Ok(())
}

pub fn read_content_header_snapshot<R: Read>(read: &mut R) -> io::Result<ContentHeaderSnapshot> {
    let mapped = read_u8(read)? as usize;
    let mut entries = Vec::with_capacity(mapped);
    for _ in 0..mapped {
        let content_type = read_u8(read)?;
        let total = read_u16(read)? as usize;
        let mut names = Vec::with_capacity(total);
        for _ in 0..total {
            names.push(read_java_utf(read)?);
        }
        entries.push(ContentHeaderEntry {
            content_type,
            names,
        });
    }
    Ok(ContentHeaderSnapshot { entries })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentPatchSet {
    pub patches: Vec<Vec<u8>>,
}

pub fn write_content_patches<W: Write>(write: &mut W, patches: &ContentPatchSet) -> io::Result<()> {
    if patches.patches.len() > u8::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "too many content patches",
        ));
    }
    write.write_all(&[patches.patches.len() as u8])?;
    for patch in &patches.patches {
        write_i32(write, patch.len() as i32)?;
        write.write_all(patch)?;
    }
    Ok(())
}

pub fn read_content_patches<R: Read>(read: &mut R) -> io::Result<ContentPatchSet> {
    let total = read_u8(read)? as usize;
    let mut patches = Vec::with_capacity(total);
    for _ in 0..total {
        patches.push(read_chunk(read)?);
    }
    Ok(ContentPatchSet { patches })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MarkerRegionBytes {
    pub bytes: Vec<u8>,
}

pub fn write_marker_region_bytes<W: Write>(
    write: &mut W,
    markers: &MarkerRegionBytes,
) -> io::Result<()> {
    write.write_all(&markers.bytes)
}

pub fn read_marker_region_bytes<R: Read>(read: &mut R) -> io::Result<MarkerRegionBytes> {
    let mut bytes = Vec::new();
    read.read_to_end(&mut bytes)?;
    Ok(MarkerRegionBytes { bytes })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomChunk {
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CustomChunkSet {
    pub chunks: Vec<CustomChunk>,
}

impl CustomChunkSet {
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.chunks
            .iter()
            .find(|chunk| chunk.name == name)
            .map(|chunk| chunk.bytes.as_slice())
    }

    pub fn insert_or_replace(&mut self, name: impl Into<String>, bytes: Vec<u8>) {
        let name = name.into();
        if let Some(chunk) = self.chunks.iter_mut().find(|chunk| chunk.name == name) {
            chunk.bytes = bytes;
        } else {
            self.chunks.push(CustomChunk { name, bytes });
        }
    }
}

pub fn write_custom_chunks<W: Write>(write: &mut W, chunks: &CustomChunkSet) -> io::Result<()> {
    write_i32(write, chunks.chunks.len() as i32)?;
    for chunk in &chunks.chunks {
        write_java_utf(write, &chunk.name)?;
        write_i32(write, chunk.bytes.len() as i32)?;
        write.write_all(&chunk.bytes)?;
    }
    Ok(())
}

pub fn read_custom_chunks<R: Read>(read: &mut R) -> io::Result<CustomChunkSet> {
    let total = read_i32(read)?;
    if total < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative custom chunk count",
        ));
    }
    let mut chunks = Vec::with_capacity(total as usize);
    for _ in 0..total {
        let name = read_java_utf(read)?;
        let bytes = read_chunk(read)?;
        chunks.push(CustomChunk { name, bytes });
    }
    Ok(CustomChunkSet { chunks })
}

pub fn write_legacy_short_chunk<W: Write>(write: &mut W, bytes: &[u8]) -> io::Result<()> {
    write_u16(write, bytes.len() as u16)?;
    write.write_all(bytes)
}

pub fn read_legacy_short_chunk<R: Read>(read: &mut R) -> io::Result<Vec<u8>> {
    let len = read_u16(read)? as usize;
    let mut payload = vec![0; len];
    read.read_exact(&mut payload)?;
    Ok(payload)
}

fn parse_i32(map: &BTreeMap<String, String>, key: &str) -> i32 {
    map.get(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or_default()
}

fn parse_i64(map: &BTreeMap<String, String>, key: &str) -> i64 {
    map.get(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or_default()
}

fn parse_json_string_array(input: &str) -> Vec<String> {
    let input = input.trim();
    if input.len() < 2 || !input.starts_with('[') || !input.ends_with(']') {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut chars = input[1..input.len() - 1].chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_whitespace() || ch == ',' {
            continue;
        }
        if ch != '"' {
            return Vec::new();
        }
        let mut value = String::new();
        while let Some(ch) = chars.next() {
            match ch {
                '"' => break,
                '\\' => match chars.next() {
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some('/') => value.push('/'),
                    Some('b') => value.push('\u{0008}'),
                    Some('f') => value.push('\u{000c}'),
                    Some('n') => value.push('\n'),
                    Some('r') => value.push('\r'),
                    Some('t') => value.push('\t'),
                    Some(other) => value.push(other),
                    None => return Vec::new(),
                },
                other => value.push(other),
            }
        }
        out.push(value);
    }
    out
}

#[allow(dead_code)]
fn _keep_imports_used_for_future_save_versions(_: fn(&mut &[u8]) -> io::Result<i64>) {
    let _ = read_i64::<&mut &[u8]>;
    let _ = write_i64::<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_header_roundtrips_msav_and_version() {
        let mut bytes = Vec::new();
        write_header(&mut bytes, 11).unwrap();
        assert_eq!(&bytes[..4], b"MSAV");
        assert_eq!(read_header(&mut bytes.as_slice()).unwrap(), 11);
    }

    #[test]
    fn region_and_string_map_roundtrip() {
        let mut tags = BTreeMap::new();
        tags.insert("version".into(), "157".into());
        tags.insert("build".into(), "157".into());
        tags.insert("saved".into(), "42".into());
        tags.insert("playtime".into(), "99".into());
        tags.insert("mapname".into(), "test-map".into());
        tags.insert("wave".into(), "12".into());
        tags.insert("mods".into(), r#"["a","b"]"#.into());

        let mut bytes = Vec::new();
        write_meta_region(&mut bytes, &tags).unwrap();
        let meta = read_meta_region(&mut bytes.as_slice()).unwrap();

        assert_eq!(meta.version, 157);
        assert_eq!(meta.build, 157);
        assert_eq!(meta.timestamp, 42);
        assert_eq!(meta.time_played, 99);
        assert_eq!(meta.map_name.as_deref(), Some("test-map"));
        assert_eq!(meta.wave, 12);
        assert_eq!(meta.mods, vec!["a", "b"]);
    }

    #[test]
    fn save_region_manifest_matches_java_order() {
        let names: Vec<&str> = SaveRegion::manifest()
            .iter()
            .map(|region| region.as_str())
            .collect();
        assert_eq!(
            names,
            vec![
                SAVE_REGION_META,
                SAVE_REGION_CONTENT,
                SAVE_REGION_PATCHES,
                SAVE_REGION_MAP,
                SAVE_REGION_ENTITIES,
                SAVE_REGION_MARKERS,
                SAVE_REGION_CUSTOM,
            ]
        );
        assert_eq!(SAVE_REGION_MANIFEST, names.as_slice());
    }

    #[test]
    fn save_region_manifest_respects_version_gates() {
        assert_eq!(SaveRegion::manifest_for_version(11), SaveRegion::manifest());
        assert!(!SaveRegion::manifest_for_version(10).contains(&SaveRegion::Patches));
        assert!(SaveRegion::manifest_for_version(10).contains(&SaveRegion::Markers));
        assert!(!SaveRegion::manifest_for_version(7).contains(&SaveRegion::Patches));
        assert!(!SaveRegion::manifest_for_version(7).contains(&SaveRegion::Markers));
    }

    #[test]
    fn raw_save_envelope_roundtrips_regions_in_version_order() {
        let mut envelope = RawSaveEnvelope::new(LATEST_SAVE_VERSION);
        envelope.set(SaveRegion::Meta, vec![1, 2, 3]).unwrap();
        envelope.set(SaveRegion::Content, vec![4]).unwrap();
        envelope.set(SaveRegion::Patches, vec![5, 6]).unwrap();

        let mut bytes = Vec::new();
        write_raw_save_envelope(&mut bytes, &envelope).unwrap();
        assert_eq!(&bytes[..4], SAVE_HEADER);

        let decoded = read_raw_save_envelope(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded.version, LATEST_SAVE_VERSION);
        assert_eq!(decoded.get(SaveRegion::Meta), Some(&[1, 2, 3][..]));
        assert_eq!(decoded.get(SaveRegion::Content), Some(&[4][..]));
        assert_eq!(decoded.get(SaveRegion::Patches), Some(&[5, 6][..]));
        assert_eq!(
            decoded
                .regions
                .iter()
                .map(|entry| entry.region)
                .collect::<Vec<_>>(),
            SaveRegion::manifest().to_vec()
        );
    }

    #[test]
    fn deflated_raw_save_envelope_matches_java_stream_wrapper_shape() {
        let mut envelope = RawSaveEnvelope::new(LATEST_SAVE_VERSION);
        envelope.set(SaveRegion::Meta, vec![1, 2, 3]).unwrap();
        envelope
            .set(SaveRegion::Content, b"content".to_vec())
            .unwrap();
        envelope.set(SaveRegion::Map, vec![9, 8, 7, 6]).unwrap();

        let mut deflated = Vec::new();
        write_deflated_raw_save_envelope(&mut deflated, &envelope).unwrap();
        assert_ne!(&deflated[..4], SAVE_HEADER);

        let decoded = read_deflated_raw_save_envelope(deflated.as_slice()).unwrap();
        assert_eq!(decoded, envelope);

        let mut raw = Vec::new();
        write_raw_save_envelope(&mut raw, &envelope).unwrap();
        assert!(read_deflated_raw_save_envelope(raw.as_slice()).is_err());
    }

    #[test]
    fn deflated_save_meta_reads_meta_region_payload() {
        let mut tags = BTreeMap::new();
        tags.insert("version".into(), "157".into());
        tags.insert("build".into(), "1574".into());
        tags.insert("saved".into(), "123456".into());
        tags.insert("playtime".into(), "987".into());
        tags.insert("mapname".into(), "serpulo".into());
        tags.insert("wave".into(), "42".into());
        tags.insert("mods".into(), r#"["foo"]"#.into());

        let mut meta_payload = Vec::new();
        write_string_map(&mut meta_payload, &tags).unwrap();

        let mut envelope = RawSaveEnvelope::new(LATEST_SAVE_VERSION);
        envelope.set(SaveRegion::Meta, meta_payload).unwrap();

        let mut deflated = Vec::new();
        write_deflated_raw_save_envelope(&mut deflated, &envelope).unwrap();
        let meta = read_deflated_save_meta(deflated.as_slice()).unwrap();

        assert_eq!(meta.version, 157);
        assert_eq!(meta.build, 1574);
        assert_eq!(meta.timestamp, 123456);
        assert_eq!(meta.time_played, 987);
        assert_eq!(meta.map_name.as_deref(), Some("serpulo"));
        assert_eq!(meta.wave, 42);
        assert_eq!(meta.mods, vec!["foo"]);
    }

    #[test]
    fn deflated_save_meta_reader_matches_java_get_meta_prefix_only_behavior() {
        let mut tags = BTreeMap::new();
        tags.insert("version".into(), "157".into());
        tags.insert("build".into(), "1574".into());
        tags.insert("saved".into(), "777".into());
        tags.insert("playtime".into(), "888".into());
        tags.insert("mapname".into(), "prefix-only".into());
        tags.insert("wave".into(), "9".into());

        let mut deflated = Vec::new();
        write_deflated_save_meta_prefix(&mut deflated, LATEST_SAVE_VERSION, &tags).unwrap();

        assert!(is_deflated_save_valid(deflated.as_slice()));
        let meta = read_deflated_save_meta(deflated.as_slice()).unwrap();
        assert_eq!(meta.map_name.as_deref(), Some("prefix-only"));
        assert_eq!(meta.timestamp, 777);
        assert_eq!(meta.time_played, 888);

        // This is intentionally not a full envelope; full save parsing must reject it.
        assert!(read_deflated_raw_save_envelope(deflated.as_slice()).is_err());
    }

    #[test]
    fn deflated_save_meta_falls_back_to_backup_like_save_io() {
        let mut tags = BTreeMap::new();
        tags.insert("version".into(), "157".into());
        tags.insert("mapname".into(), "backup-map".into());

        let mut backup = Vec::new();
        write_deflated_save_meta_prefix(&mut backup, LATEST_SAVE_VERSION, &tags).unwrap();

        let meta = read_deflated_save_meta_with_backup(b"not a save", Some(backup.as_slice()))
            .expect("backup should be used when primary is invalid");
        assert_eq!(meta.map_name.as_deref(), Some("backup-map"));

        assert!(read_deflated_save_meta_with_backup(b"not a save", None).is_err());
    }

    #[test]
    fn raw_save_envelope_rejects_regions_not_present_for_version() {
        let mut envelope = RawSaveEnvelope::new(10);
        let err = envelope
            .set(SaveRegion::Patches, vec![1])
            .expect_err("patches region should not exist before save v11");
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn content_header_snapshot_roundtrips_in_order() {
        let snapshot = ContentHeaderSnapshot {
            entries: vec![
                ContentHeaderEntry {
                    content_type: 3,
                    names: vec!["alpha".into(), "beta".into()],
                },
                ContentHeaderEntry {
                    content_type: 15,
                    names: vec!["delta".into()],
                },
            ],
        };

        let mut bytes = Vec::new();
        write_content_header_snapshot(&mut bytes, &snapshot).unwrap();
        assert_eq!(bytes[0], 2);
        assert_eq!(
            read_content_header_snapshot(&mut bytes.as_slice()).unwrap(),
            snapshot
        );
    }

    #[test]
    fn content_patches_roundtrip_as_opaque_bytes() {
        let patches = ContentPatchSet {
            patches: vec![b"alpha".to_vec(), vec![0, 1, 2]],
        };
        let mut bytes = Vec::new();
        write_content_patches(&mut bytes, &patches).unwrap();
        assert_eq!(bytes[0], 2);
        assert_eq!(
            read_content_patches(&mut bytes.as_slice()).unwrap(),
            patches
        );
    }

    #[test]
    fn marker_region_bytes_are_passthrough() {
        let markers = MarkerRegionBytes {
            bytes: br#"{"1":{"type":"point"}}"#.to_vec(),
        };
        let mut bytes = Vec::new();
        write_marker_region_bytes(&mut bytes, &markers).unwrap();
        assert_eq!(
            read_marker_region_bytes(&mut bytes.as_slice()).unwrap(),
            markers
        );
    }

    #[test]
    fn custom_chunks_roundtrip_and_preserve_unknown_payloads() {
        let mut chunks = CustomChunkSet::default();
        chunks.insert_or_replace("mod-a", vec![1, 2, 3]);
        chunks.insert_or_replace(CUSTOM_CHUNK_STATIC_FOG_DATA, vec![9]);
        chunks.insert_or_replace("unknown", b"payload".to_vec());
        chunks.insert_or_replace(CUSTOM_CHUNK_STATIC_FOG_DATA, vec![4, 5, 6]);

        let mut bytes = Vec::new();
        write_custom_chunks(&mut bytes, &chunks).unwrap();
        let decoded = read_custom_chunks(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded, chunks);
        assert_eq!(decoded.get("mod-a"), Some(&[1, 2, 3][..]));
        assert_eq!(
            decoded.get(CUSTOM_CHUNK_STATIC_FOG_DATA),
            Some(&[4, 5, 6][..])
        );
        assert_eq!(decoded.get("unknown"), Some(&b"payload"[..]));
        assert_eq!(
            decoded
                .chunks
                .iter()
                .map(|chunk| chunk.name.as_str())
                .collect::<Vec<_>>(),
            vec!["mod-a", CUSTOM_CHUNK_STATIC_FOG_DATA, "unknown"]
        );
    }
}
