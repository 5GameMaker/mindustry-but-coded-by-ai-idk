use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use ubjson::Container as UbjsonContainer;

use crate::mindustry::{
    game::map_objectives::{TextureHolder, OBJECTIVE_MARKER_DRAW_LAYER_OVERLAY_UI},
    game::{marker_type_by_java_name, MapMarkers, ObjectiveMarker, Rules, Vec2},
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
pub const SAVE_EXTENSION: &str = "msav";
pub const LAST_SECTOR_SAVE_SETTING: &str = "last-sector-save";
pub const LAST_SECTOR_SAVE_FALLBACK: &str = "<none>";
pub const SAVE_SLOT_SETTING_PREFIX: &str = "save-";

pub const SAVE_REGION_MANIFEST: &[&str] = &[
    SAVE_REGION_META,
    SAVE_REGION_CONTENT,
    SAVE_REGION_PATCHES,
    SAVE_REGION_MAP,
    SAVE_REGION_ENTITIES,
    SAVE_REGION_MARKERS,
    SAVE_REGION_CUSTOM,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavePathLayout {
    pub save_dir: PathBuf,
    pub save_extension: String,
}

impl SavePathLayout {
    pub fn new(save_dir: impl Into<PathBuf>) -> Self {
        Self {
            save_dir: save_dir.into(),
            save_extension: SAVE_EXTENSION.into(),
        }
    }

    pub fn with_extension(save_dir: impl Into<PathBuf>, save_extension: impl Into<String>) -> Self {
        Self {
            save_dir: save_dir.into(),
            save_extension: save_extension.into(),
        }
    }

    pub fn file_for_slot(&self, slot: i32) -> PathBuf {
        self.save_dir
            .join(format!("{}.{}", slot, self.save_extension))
    }

    pub fn sector_file(&self, planet: &str, sector_id: i32) -> PathBuf {
        self.save_dir.join(sector_file_name_with_extension(
            planet,
            sector_id,
            &self.save_extension,
        ))
    }

    pub fn backup_file_for(&self, file: impl AsRef<Path>) -> PathBuf {
        backup_file_for_path(file)
    }

    pub fn next_slot_file<I, P>(&self, existing: I) -> PathBuf
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let names: Vec<String> = existing
            .into_iter()
            .filter_map(|path| file_name_string(path.as_ref()))
            .collect();
        self.save_dir.join(next_slot_file_name_with_extension(
            names,
            &self.save_extension,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveSlotKind {
    Numbered(i32),
    Sector { planet: String, id: i32 },
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SaveSlotRecord {
    pub file: PathBuf,
    pub meta: Option<SaveMeta>,
}

impl SaveSlotRecord {
    pub fn new(file: impl Into<PathBuf>) -> Self {
        Self {
            file: file.into(),
            meta: None,
        }
    }

    pub fn with_meta(file: impl Into<PathBuf>, meta: SaveMeta) -> Self {
        Self {
            file: file.into(),
            meta: Some(meta),
        }
    }

    pub fn index(&self) -> String {
        file_stem_string(&self.file)
            .unwrap_or_else(|| file_name_string(&self.file).unwrap_or_default())
    }

    pub fn kind(&self) -> SaveSlotKind {
        save_slot_kind_from_file_name(&self.index())
    }

    pub fn name_setting_key(&self) -> String {
        format!("{SAVE_SLOT_SETTING_PREFIX}{}-name", self.index())
    }

    pub fn autosave_setting_key(&self) -> String {
        format!("{SAVE_SLOT_SETTING_PREFIX}{}-autosave", self.index())
    }

    pub fn preview_file(&self, preview_dir: impl AsRef<Path>) -> PathBuf {
        preview_dir
            .as_ref()
            .join(format!("save_slot_{}.png", self.index()))
    }

    pub fn load_preview_file(&self, preview_dir: impl AsRef<Path>) -> PathBuf {
        let preview = self.preview_file(preview_dir);
        let name = file_name_string(&preview).unwrap_or_default();
        preview.with_file_name(format!("{name}.spreview"))
    }

    pub fn is_sector_file(&self) -> bool {
        matches!(self.kind(), SaveSlotKind::Sector { .. })
    }

    pub fn is_sector(&self) -> bool {
        self.meta.as_ref().is_some_and(SaveMeta::has_sector)
    }

    pub fn timestamp(&self) -> i64 {
        self.meta.as_ref().map_or(0, |meta| meta.timestamp)
    }

    pub fn time_played(&self) -> i64 {
        self.meta.as_ref().map_or(0, |meta| meta.time_played)
    }

    pub fn delete_targets(&self) -> [PathBuf; 2] {
        [backup_file_for_path(&self.file), self.file.clone()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeflatedSaveFile {
    pub file: PathBuf,
    pub bytes: Vec<u8>,
    pub backup_bytes: Option<Vec<u8>>,
}

impl DeflatedSaveFile {
    pub fn new(file: impl Into<PathBuf>, bytes: Vec<u8>) -> Self {
        Self {
            file: file.into(),
            bytes,
            backup_bytes: None,
        }
    }

    pub fn with_backup(file: impl Into<PathBuf>, bytes: Vec<u8>, backup_bytes: Vec<u8>) -> Self {
        Self {
            file: file.into(),
            bytes,
            backup_bytes: Some(backup_bytes),
        }
    }

    pub fn read_meta(&self) -> io::Result<SaveMeta> {
        read_deflated_save_meta_with_backup(&self.bytes, self.backup_bytes.as_deref())
    }

    pub fn is_valid(&self) -> bool {
        is_deflated_save_valid_with_backup(&self.bytes, self.backup_bytes.as_deref())
    }

    pub fn slot_record(&self) -> io::Result<SaveSlotRecord> {
        self.read_meta()
            .map(|meta| SaveSlotRecord::with_meta(self.file.clone(), meta))
    }
}

pub fn slot_file_name(slot: i32) -> String {
    slot_file_name_with_extension(slot, SAVE_EXTENSION)
}

pub fn slot_file_name_with_extension(slot: i32, extension: &str) -> String {
    format!("{slot}.{extension}")
}

pub fn sector_file_name(planet: &str, sector_id: i32) -> String {
    sector_file_name_with_extension(planet, sector_id, SAVE_EXTENSION)
}

pub fn sector_file_name_with_extension(planet: &str, sector_id: i32, extension: &str) -> String {
    format!("sector-{planet}-{sector_id}.{extension}")
}

pub fn backup_file_name_for(file_name: &str) -> String {
    let extension = file_name
        .rsplit_once('.')
        .map(|(_, extension)| extension)
        .unwrap_or_default();
    format!("{file_name}-backup.{extension}")
}

pub fn backup_file_for_path(file: impl AsRef<Path>) -> PathBuf {
    let file = file.as_ref();
    let name = file_name_string(file).unwrap_or_default();
    let backup = backup_file_name_for(&name);
    file.parent()
        .map(|parent| parent.join(&backup))
        .unwrap_or_else(|| PathBuf::from(backup))
}

pub fn is_backup_save_name(name: &str) -> bool {
    name.contains("backup")
}

pub fn should_scan_save_file(file: impl AsRef<Path>) -> bool {
    file_name_string(file.as_ref()).is_some_and(|name| !is_backup_save_name(&name))
}

pub fn collect_valid_save_slot_records<I, P, F>(files: I, mut read_meta: F) -> Vec<SaveSlotRecord>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    F: FnMut(&Path) -> io::Result<SaveMeta>,
{
    let mut slots = Vec::new();
    for file in files {
        let file = file.as_ref();
        if !should_scan_save_file(file) {
            continue;
        }
        if let Ok(meta) = read_meta(file) {
            slots.push(SaveSlotRecord::with_meta(file.to_path_buf(), meta));
        }
    }
    slots
}

pub fn collect_valid_deflated_save_slots<'a, I>(files: I) -> Vec<SaveSlotRecord>
where
    I: IntoIterator<Item = &'a DeflatedSaveFile>,
{
    let mut slots = Vec::new();
    for file in files {
        if !should_scan_save_file(&file.file) {
            continue;
        }
        if let Ok(slot) = file.slot_record() {
            slots.push(slot);
        }
    }
    slots
}

pub fn find_last_sector_save<'a, F>(
    slots: &'a [SaveSlotRecord],
    stored_name: &str,
    mut name_for_slot: F,
) -> Option<&'a SaveSlotRecord>
where
    F: FnMut(&SaveSlotRecord) -> String,
{
    slots
        .iter()
        .find(|slot| slot.is_sector() && name_for_slot(slot) == stored_name)
}

pub fn next_slot_file_name<I, S>(existing_names: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    next_slot_file_name_with_extension(existing_names, SAVE_EXTENSION)
}

pub fn next_slot_file_name_with_extension<I, S>(existing_names: I, extension: &str) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let existing: BTreeSet<String> = existing_names
        .into_iter()
        .map(|name| name.as_ref().to_string())
        .collect();
    let mut slot = 0;
    loop {
        let candidate = slot_file_name_with_extension(slot, extension);
        if !existing.contains(&candidate) {
            return candidate;
        }
        slot += 1;
    }
}

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
        } else if version >= 7 {
            &[
                SaveRegion::Meta,
                SaveRegion::Content,
                SaveRegion::Map,
                SaveRegion::Entities,
                SaveRegion::Custom,
            ]
        } else {
            &[
                SaveRegion::Meta,
                SaveRegion::Content,
                SaveRegion::Map,
                SaveRegion::Entities,
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

    pub fn has_sector(&self) -> bool {
        json_field_is_non_null(&self.rules_json, "sector")
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

pub fn read_deflated_raw_save_envelope_with_backup(
    primary: &[u8],
    backup: Option<&[u8]>,
) -> io::Result<RawSaveEnvelope> {
    match read_deflated_raw_save_envelope(primary) {
        Ok(envelope) => Ok(envelope),
        Err(primary_error) => {
            if let Some(backup) = backup {
                read_deflated_raw_save_envelope(backup)
            } else {
                Err(primary_error)
            }
        }
    }
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

pub fn is_deflated_save_valid_with_backup(primary: &[u8], backup: Option<&[u8]>) -> bool {
    read_deflated_save_meta_with_backup(primary, backup).is_ok()
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

impl MarkerRegionBytes {
    pub fn ubjson_summary(&self) -> io::Result<MarkerRegionSummary> {
        summarize_marker_region_bytes(&self.bytes)
    }

    pub fn ubjson_markers(&self) -> io::Result<MapMarkers> {
        parse_marker_region_bytes(&self.bytes)
    }
}

pub fn write_marker_region_bytes<W: Write>(
    write: &mut W,
    markers: &MarkerRegionBytes,
) -> io::Result<()> {
    write.write_all(&markers.bytes)
}

pub fn write_marker_region_from_map_markers<W: Write>(
    write: &mut W,
    markers: &MapMarkers,
) -> io::Result<()> {
    let bytes = marker_region_bytes_from_map_markers(markers)?;
    write.write_all(&bytes)
}

pub fn marker_region_bytes_from_map_markers(markers: &MapMarkers) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    write_ubjson_object_start(&mut bytes);
    let mut entries = markers.entries().collect::<Vec<_>>();
    entries.sort_by_key(|(id, _)| *id);
    for (id, marker) in entries {
        write_ubjson_key(&mut bytes, &id.to_string())?;
        write_objective_marker_ubjson(&mut bytes, marker)?;
    }
    write_ubjson_object_end(&mut bytes);
    Ok(bytes)
}

pub fn read_marker_region_bytes<R: Read>(read: &mut R) -> io::Result<MarkerRegionBytes> {
    let mut bytes = Vec::new();
    read.read_to_end(&mut bytes)?;
    Ok(MarkerRegionBytes { bytes })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MarkerRegionSummary {
    pub total: usize,
    pub recognized_by_type: BTreeMap<String, usize>,
    pub unrecognized_type_count: usize,
    pub missing_class_count: usize,
}

impl MarkerRegionSummary {
    pub fn marker_count(&self) -> usize {
        self.total
    }

    pub fn marker_type_counts(&self) -> &BTreeMap<String, usize> {
        &self.recognized_by_type
    }

    pub fn unrecognized_or_missing_class_tag_count(&self) -> usize {
        self.unrecognized_type_count + self.missing_class_count
    }
}

pub fn summarize_marker_region_bytes(bytes: &[u8]) -> io::Result<MarkerRegionSummary> {
    let (remaining, markers) = ubjson::parse_one(bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse MapMarkers UBJSON: {error}"),
        )
    })?;
    if !remaining.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "trailing bytes after MapMarkers UBJSON",
        ));
    }

    let UbjsonContainer::Object(entries) = markers else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "MapMarkers UBJSON must be an object map",
        ));
    };

    let mut summary = MarkerRegionSummary {
        total: entries.len(),
        ..MarkerRegionSummary::default()
    };

    for marker in entries.values() {
        let Some(class_tag) = marker_class_tag(marker) else {
            summary.missing_class_count += 1;
            continue;
        };
        if let Some(canonical) = marker_type_by_java_name(class_tag) {
            *summary
                .recognized_by_type
                .entry(canonical.to_string())
                .or_insert(0) += 1;
        } else {
            summary.unrecognized_type_count += 1;
        }
    }

    Ok(summary)
}

pub fn parse_marker_region_bytes(bytes: &[u8]) -> io::Result<MapMarkers> {
    let (remaining, markers) = ubjson::parse_one(bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse MapMarkers UBJSON: {error}"),
        )
    })?;
    if !remaining.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "trailing bytes after MapMarkers UBJSON",
        ));
    }

    let UbjsonContainer::Object(entries) = markers else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "MapMarkers UBJSON must be an object map",
        ));
    };

    let mut decoded = entries
        .iter()
        .filter_map(|(id, marker)| {
            let id = id.parse::<i32>().ok()?;
            let marker = parse_marker_entry(marker)?;
            Some((id, marker))
        })
        .collect::<Vec<_>>();
    decoded.sort_by_key(|(id, _)| *id);

    Ok(MapMarkers::rebuild_from_entries(decoded))
}

fn parse_marker_entry(marker: &UbjsonContainer<'_>) -> Option<ObjectiveMarker> {
    let fields = match marker {
        UbjsonContainer::Object(fields) => fields,
        _ => return None,
    };

    let mut marker = ObjectiveMarker::default_for_java_name(marker_class_tag(marker)?)?;
    let common = marker.common_mut();

    if let Some(world) = fields.get("world").and_then(ubjson_bool) {
        common.world = world;
    }
    if let Some(minimap) = fields.get("minimap").and_then(ubjson_bool) {
        common.minimap = minimap;
    }
    if let Some(autoscale) = fields.get("autoscale").and_then(ubjson_bool) {
        common.autoscale = autoscale;
    }
    if let Some(draw_layer) = fields.get("drawLayer").and_then(ubjson_f32) {
        common.draw_layer = draw_layer;
    }

    match &mut marker {
        ObjectiveMarker::ShapeText(value) => {
            apply_pos_fields(&mut value.pos, fields.get("pos"));
            apply_string_field(&mut value.text, fields.get("text"));
            apply_f32_field(&mut value.font_size, fields.get("fontSize"));
            apply_f32_field(&mut value.text_height, fields.get("textHeight"));
            apply_u8_field(&mut value.flags, fields.get("flags"));
            apply_i32_field(&mut value.text_align, fields.get("textAlign"));
            apply_i32_field(&mut value.line_align, fields.get("lineAlign"));
            apply_f32_field(&mut value.radius, fields.get("radius"));
            apply_f32_field(&mut value.rotation, fields.get("rotation"));
            apply_i32_field(&mut value.sides, fields.get("sides"));
            apply_color_field(&mut value.color, fields.get("color"));
        }
        ObjectiveMarker::Point(value) => {
            apply_pos_fields(&mut value.pos, fields.get("pos"));
            apply_f32_field(&mut value.radius, fields.get("radius"));
            apply_f32_field(&mut value.stroke, fields.get("stroke"));
            apply_color_field(&mut value.color, fields.get("color"));
        }
        ObjectiveMarker::Shape(value) => {
            apply_pos_fields(&mut value.pos, fields.get("pos"));
            apply_f32_field(&mut value.radius, fields.get("radius"));
            apply_f32_field(&mut value.rotation, fields.get("rotation"));
            apply_f32_field(&mut value.stroke, fields.get("stroke"));
            apply_f32_field(&mut value.start_angle, fields.get("startAngle"));
            apply_f32_field(&mut value.end_angle, fields.get("endAngle"));
            apply_bool_field(&mut value.fill, fields.get("fill"));
            apply_bool_field(&mut value.outline, fields.get("outline"));
            apply_i32_field(&mut value.sides, fields.get("sides"));
            apply_color_field(&mut value.color, fields.get("color"));
        }
        ObjectiveMarker::Text(value) => {
            apply_pos_fields(&mut value.pos, fields.get("pos"));
            apply_string_field(&mut value.text, fields.get("text"));
            apply_f32_field(&mut value.font_size, fields.get("fontSize"));
            apply_u8_field(&mut value.flags, fields.get("flags"));
            apply_i32_field(&mut value.text_align, fields.get("textAlign"));
            apply_i32_field(&mut value.line_align, fields.get("lineAlign"));
        }
        ObjectiveMarker::Line(value) => {
            apply_pos_fields(&mut value.pos, fields.get("pos"));
            apply_pos_fields(&mut value.end_pos, fields.get("endPos"));
            apply_f32_field(&mut value.stroke, fields.get("stroke"));
            apply_bool_field(&mut value.outline, fields.get("outline"));
            apply_color_field(&mut value.color1, fields.get("color1"));
            apply_color_field(&mut value.color2, fields.get("color2"));
        }
        ObjectiveMarker::Texture(value) => {
            apply_pos_fields(&mut value.pos, fields.get("pos"));
            apply_f32_field(&mut value.rotation, fields.get("rotation"));
            apply_f32_field(&mut value.width, fields.get("width"));
            apply_f32_field(&mut value.height, fields.get("height"));
            if let Some(texture) = fields.get("texture").and_then(ubjson_texture_holder) {
                value.texture = texture;
            } else if let Some(texture_name) = fields.get("textureName").and_then(ubjson_string) {
                value.texture = crate::mindustry::game::map_objectives::TextureHolder::String(
                    texture_name.to_string(),
                );
            }
            apply_color_field(&mut value.color, fields.get("color"));
        }
        ObjectiveMarker::Quad(value) => {
            if let Some(texture) = fields.get("texture").and_then(ubjson_texture_holder) {
                value.texture = texture;
            } else if let Some(texture_name) = fields.get("textureName").and_then(ubjson_string) {
                value.texture = crate::mindustry::game::map_objectives::TextureHolder::String(
                    texture_name.to_string(),
                );
            }
            if let Some(vertices) = fields.get("vertices").and_then(ubjson_f32_array) {
                value.vertices = vertices;
            }
            apply_bool_field(&mut value.map_region, fields.get("mapRegion"));
        }
    }

    Some(marker)
}

fn apply_pos_fields(pos: &mut crate::mindustry::game::Vec2, value: Option<&UbjsonContainer<'_>>) {
    if let Some((x, y)) = value.and_then(ubjson_vec2) {
        pos.x = x;
        pos.y = y;
    }
}

fn apply_string_field(target: &mut String, value: Option<&UbjsonContainer<'_>>) {
    if let Some(value) = value.and_then(ubjson_string) {
        *target = value.to_string();
    }
}

fn apply_f32_field(target: &mut f32, value: Option<&UbjsonContainer<'_>>) {
    if let Some(value) = value.and_then(ubjson_f32) {
        *target = value;
    }
}

fn apply_i32_field(target: &mut i32, value: Option<&UbjsonContainer<'_>>) {
    if let Some(value) = value.and_then(ubjson_i32) {
        *target = value;
    }
}

fn apply_u8_field(target: &mut u8, value: Option<&UbjsonContainer<'_>>) {
    if let Some(value) = value.and_then(ubjson_u8) {
        *target = value;
    }
}

fn apply_bool_field(target: &mut bool, value: Option<&UbjsonContainer<'_>>) {
    if let Some(value) = value.and_then(ubjson_bool) {
        *target = value;
    }
}

fn apply_color_field(target: &mut u32, value: Option<&UbjsonContainer<'_>>) {
    if let Some(value) = value.and_then(ubjson_color_rgba) {
        *target = value;
    }
}

fn marker_class_tag<'a>(marker: &'a UbjsonContainer<'a>) -> Option<&'a str> {
    let UbjsonContainer::Object(fields) = marker else {
        return None;
    };
    fields
        .iter()
        .find(|(key, _)| key.as_ref() == "class")
        .and_then(|(_, value)| ubjson_string(value))
}

fn ubjson_string<'a>(value: &'a UbjsonContainer<'a>) -> Option<&'a str> {
    match value {
        UbjsonContainer::String(value) => Some(value.as_ref()),
        _ => None,
    }
}

fn ubjson_bool(value: &UbjsonContainer<'_>) -> Option<bool> {
    match value {
        UbjsonContainer::Boolean(value) => Some(*value),
        _ => None,
    }
}

fn ubjson_i32(value: &UbjsonContainer<'_>) -> Option<i32> {
    match value {
        UbjsonContainer::Int8(value) => Some(*value as i32),
        UbjsonContainer::Uint8(value) => Some(*value as i32),
        UbjsonContainer::Int16(value) => Some(*value as i32),
        UbjsonContainer::Int32(value) => Some(*value),
        UbjsonContainer::Int64(value) => i32::try_from(*value).ok(),
        _ => None,
    }
}

fn ubjson_u8(value: &UbjsonContainer<'_>) -> Option<u8> {
    match value {
        UbjsonContainer::Int8(value) => u8::try_from(*value).ok(),
        UbjsonContainer::Uint8(value) => Some(*value),
        UbjsonContainer::Int16(value) => u8::try_from(*value).ok(),
        UbjsonContainer::Int32(value) => u8::try_from(*value).ok(),
        UbjsonContainer::Int64(value) => u8::try_from(*value).ok(),
        _ => None,
    }
}

fn ubjson_f32(value: &UbjsonContainer<'_>) -> Option<f32> {
    match value {
        UbjsonContainer::Float32(value) => Some(*value),
        UbjsonContainer::Float64(value) => Some(*value as f32),
        UbjsonContainer::Int8(value) => Some(*value as f32),
        UbjsonContainer::Uint8(value) => Some(*value as f32),
        UbjsonContainer::Int16(value) => Some(*value as f32),
        UbjsonContainer::Int32(value) => Some(*value as f32),
        UbjsonContainer::Int64(value) => Some(*value as f32),
        _ => None,
    }
}

fn ubjson_vec2(value: &UbjsonContainer<'_>) -> Option<(f32, f32)> {
    match value {
        UbjsonContainer::Object(fields) => Some((
            fields.get("x").and_then(ubjson_f32)?,
            fields.get("y").and_then(ubjson_f32)?,
        )),
        UbjsonContainer::Array(values) if values.len() >= 2 => {
            Some((ubjson_f32(&values[0])?, ubjson_f32(&values[1])?))
        }
        _ => None,
    }
}

fn ubjson_texture_holder(
    value: &UbjsonContainer<'_>,
) -> Option<crate::mindustry::game::map_objectives::TextureHolder> {
    let fields = match value {
        UbjsonContainer::Object(fields) => fields,
        _ => return None,
    };

    if let Some(value) = fields.get("string").and_then(ubjson_string) {
        Some(crate::mindustry::game::map_objectives::TextureHolder::String(value.to_string()))
    } else if let Some(value) = fields.get("content").and_then(ubjson_string) {
        Some(crate::mindustry::game::map_objectives::TextureHolder::Content(value.to_string()))
    } else {
        fields
            .get("building")
            .and_then(ubjson_i32)
            .map(crate::mindustry::game::map_objectives::TextureHolder::Building)
    }
}

fn ubjson_color_rgba(value: &UbjsonContainer<'_>) -> Option<u32> {
    match value {
        UbjsonContainer::String(value) => parse_hex_rgba(value.as_ref()),
        UbjsonContainer::Object(fields) => Some(pack_rgba(
            fields.get("r").and_then(ubjson_f32)?,
            fields.get("g").and_then(ubjson_f32)?,
            fields.get("b").and_then(ubjson_f32)?,
            fields.get("a").and_then(ubjson_f32).unwrap_or(1.0),
        )),
        UbjsonContainer::Array(values) if values.len() >= 3 => Some(pack_rgba(
            ubjson_f32(&values[0])?,
            ubjson_f32(&values[1])?,
            ubjson_f32(&values[2])?,
            values.get(3).and_then(ubjson_f32).unwrap_or(1.0),
        )),
        _ => None,
    }
}

fn ubjson_f32_array(value: &UbjsonContainer<'_>) -> Option<Vec<f32>> {
    let UbjsonContainer::Array(values) = value else {
        return None;
    };
    values.iter().map(ubjson_f32).collect()
}

fn parse_hex_rgba(value: &str) -> Option<u32> {
    let value = value.strip_prefix('#').unwrap_or(value);
    let value = match value.len() {
        6 => format!("{value}ff"),
        8 => value.to_string(),
        _ => return None,
    };
    u32::from_str_radix(&value, 16).ok()
}

fn pack_rgba(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let pack = |value: f32| -> u32 { (value.clamp(0.0, 1.0) * 255.0).round() as u32 };
    (pack(r) << 24) | (pack(g) << 16) | (pack(b) << 8) | pack(a)
}

fn write_objective_marker_ubjson(write: &mut Vec<u8>, marker: &ObjectiveMarker) -> io::Result<()> {
    write_ubjson_object_start(write);
    write_marker_common_ubjson(write, marker)?;

    match marker {
        ObjectiveMarker::ShapeText(value) => {
            write_ubjson_vec2_field(write, "pos", value.pos)?;
            write_ubjson_string_field(write, "text", &value.text)?;
            write_ubjson_f32_field(write, "fontSize", value.font_size)?;
            write_ubjson_f32_field(write, "textHeight", value.text_height)?;
            write_ubjson_u8_field(write, "flags", value.flags)?;
            write_ubjson_i32_field(write, "textAlign", value.text_align)?;
            write_ubjson_i32_field(write, "lineAlign", value.line_align)?;
            write_ubjson_f32_field(write, "radius", value.radius)?;
            write_ubjson_f32_field(write, "rotation", value.rotation)?;
            write_ubjson_i32_field(write, "sides", value.sides)?;
            write_ubjson_color_field(write, "color", value.color)?;
        }
        ObjectiveMarker::Point(value) => {
            write_ubjson_vec2_field(write, "pos", value.pos)?;
            write_ubjson_f32_field(write, "radius", value.radius)?;
            write_ubjson_f32_field(write, "stroke", value.stroke)?;
            write_ubjson_color_field(write, "color", value.color)?;
        }
        ObjectiveMarker::Shape(value) => {
            write_ubjson_vec2_field(write, "pos", value.pos)?;
            write_ubjson_f32_field(write, "radius", value.radius)?;
            write_ubjson_f32_field(write, "rotation", value.rotation)?;
            write_ubjson_f32_field(write, "stroke", value.stroke)?;
            write_ubjson_f32_field(write, "startAngle", value.start_angle)?;
            write_ubjson_f32_field(write, "endAngle", value.end_angle)?;
            write_ubjson_bool_field(write, "fill", value.fill)?;
            write_ubjson_bool_field(write, "outline", value.outline)?;
            write_ubjson_i32_field(write, "sides", value.sides)?;
            write_ubjson_color_field(write, "color", value.color)?;
        }
        ObjectiveMarker::Text(value) => {
            write_ubjson_vec2_field(write, "pos", value.pos)?;
            write_ubjson_string_field(write, "text", &value.text)?;
            write_ubjson_f32_field(write, "fontSize", value.font_size)?;
            write_ubjson_u8_field(write, "flags", value.flags)?;
            write_ubjson_i32_field(write, "textAlign", value.text_align)?;
            write_ubjson_i32_field(write, "lineAlign", value.line_align)?;
        }
        ObjectiveMarker::Line(value) => {
            write_ubjson_vec2_field(write, "pos", value.pos)?;
            write_ubjson_vec2_field(write, "endPos", value.end_pos)?;
            write_ubjson_f32_field(write, "stroke", value.stroke)?;
            write_ubjson_bool_field(write, "outline", value.outline)?;
            write_ubjson_color_field(write, "color1", value.color1)?;
            write_ubjson_color_field(write, "color2", value.color2)?;
        }
        ObjectiveMarker::Texture(value) => {
            write_ubjson_vec2_field(write, "pos", value.pos)?;
            write_ubjson_f32_field(write, "rotation", value.rotation)?;
            write_ubjson_f32_field(write, "width", value.width)?;
            write_ubjson_f32_field(write, "height", value.height)?;
            write_ubjson_texture_holder_field(write, "texture", &value.texture)?;
            write_ubjson_color_field(write, "color", value.color)?;
        }
        ObjectiveMarker::Quad(value) => {
            write_ubjson_texture_holder_field(write, "texture", &value.texture)?;
            write_ubjson_f32_array_field(write, "vertices", &value.vertices)?;
            write_ubjson_bool_field(write, "mapRegion", value.map_region)?;
        }
    }

    write_ubjson_object_end(write);
    Ok(())
}

fn write_marker_common_ubjson(write: &mut Vec<u8>, marker: &ObjectiveMarker) -> io::Result<()> {
    write_ubjson_string_field(write, "class", marker.type_name())?;
    let common = marker.common();
    write_ubjson_bool_field(write, "world", common.world)?;
    write_ubjson_bool_field(write, "minimap", common.minimap)?;
    write_ubjson_bool_field(write, "autoscale", common.autoscale)?;
    if common.draw_layer != OBJECTIVE_MARKER_DRAW_LAYER_OVERLAY_UI {
        write_ubjson_f32_field(write, "drawLayer", common.draw_layer)?;
    }
    Ok(())
}

fn write_ubjson_object_start(write: &mut Vec<u8>) {
    write.push(b'{');
}

fn write_ubjson_object_end(write: &mut Vec<u8>) {
    write.push(b'}');
}

fn write_ubjson_key(write: &mut Vec<u8>, key: &str) -> io::Result<()> {
    write_ubjson_raw_string(write, key)
}

fn write_ubjson_string_field(write: &mut Vec<u8>, key: &str, value: &str) -> io::Result<()> {
    write_ubjson_key(write, key)?;
    write.push(b'S');
    write_ubjson_raw_string(write, value)
}

fn write_ubjson_bool_field(write: &mut Vec<u8>, key: &str, value: bool) -> io::Result<()> {
    write_ubjson_key(write, key)?;
    write.push(if value { b'T' } else { b'F' });
    Ok(())
}

fn write_ubjson_i32_field(write: &mut Vec<u8>, key: &str, value: i32) -> io::Result<()> {
    write_ubjson_key(write, key)?;
    write.push(b'l');
    write.extend_from_slice(&value.to_be_bytes());
    Ok(())
}

fn write_ubjson_u8_field(write: &mut Vec<u8>, key: &str, value: u8) -> io::Result<()> {
    write_ubjson_key(write, key)?;
    write.push(b'U');
    write.push(value);
    Ok(())
}

fn write_ubjson_f32_field(write: &mut Vec<u8>, key: &str, value: f32) -> io::Result<()> {
    write_ubjson_key(write, key)?;
    write.push(b'd');
    write.extend_from_slice(&value.to_be_bytes());
    Ok(())
}

fn write_ubjson_vec2_field(write: &mut Vec<u8>, key: &str, value: Vec2) -> io::Result<()> {
    write_ubjson_key(write, key)?;
    write_ubjson_object_start(write);
    write_ubjson_f32_field(write, "x", value.x)?;
    write_ubjson_f32_field(write, "y", value.y)?;
    write_ubjson_object_end(write);
    Ok(())
}

fn write_ubjson_color_field(write: &mut Vec<u8>, key: &str, value: u32) -> io::Result<()> {
    write_ubjson_string_field(write, key, &format!("{value:08x}"))
}

fn write_ubjson_texture_holder_field(
    write: &mut Vec<u8>,
    key: &str,
    value: &TextureHolder,
) -> io::Result<()> {
    write_ubjson_key(write, key)?;
    write_ubjson_object_start(write);
    match value {
        TextureHolder::String(value) => write_ubjson_string_field(write, "string", value)?,
        TextureHolder::Content(value) => write_ubjson_string_field(write, "content", value)?,
        TextureHolder::Building(value) => write_ubjson_i32_field(write, "building", *value)?,
    }
    write_ubjson_object_end(write);
    Ok(())
}

fn write_ubjson_f32_array_field(write: &mut Vec<u8>, key: &str, values: &[f32]) -> io::Result<()> {
    write_ubjson_key(write, key)?;
    write.push(b'[');
    for value in values {
        write.push(b'd');
        write.extend_from_slice(&value.to_be_bytes());
    }
    write.push(b']');
    Ok(())
}

fn write_ubjson_raw_string(write: &mut Vec<u8>, value: &str) -> io::Result<()> {
    let bytes = value.as_bytes();
    if bytes.len() <= u8::MAX as usize {
        write.push(b'U');
        write.push(bytes.len() as u8);
    } else if bytes.len() <= i16::MAX as usize {
        write.push(b'I');
        write.extend_from_slice(&(bytes.len() as i16).to_be_bytes());
    } else if bytes.len() <= i32::MAX as usize {
        write.push(b'l');
        write.extend_from_slice(&(bytes.len() as i32).to_be_bytes());
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "UBJSON string too large",
        ));
    }
    write.extend_from_slice(bytes);
    Ok(())
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

fn json_field_is_non_null(json: &str, key: &str) -> bool {
    let quoted_key = format!("\"{key}\"");
    let Some(key_index) = json.find(&quoted_key) else {
        return false;
    };
    let after_key = &json[key_index + quoted_key.len()..];
    let Some(colon_index) = after_key.find(':') else {
        return false;
    };
    !after_key[colon_index + 1..]
        .trim_start()
        .starts_with("null")
}

fn save_slot_kind_from_file_name(stem: &str) -> SaveSlotKind {
    if let Ok(slot) = stem.parse::<i32>() {
        return SaveSlotKind::Numbered(slot);
    }

    let Some(rest) = stem.strip_prefix("sector-") else {
        return SaveSlotKind::Other;
    };
    let Some((planet, id)) = rest.rsplit_once('-') else {
        return SaveSlotKind::Other;
    };
    let Ok(id) = id.parse::<i32>() else {
        return SaveSlotKind::Other;
    };
    SaveSlotKind::Sector {
        planet: planet.into(),
        id,
    }
}

fn file_name_string(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
}

fn file_stem_string(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|name| name.to_str())
        .map(str::to_string)
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
        assert!(SaveRegion::manifest_for_version(7).contains(&SaveRegion::Custom));
        assert!(!SaveRegion::manifest_for_version(7).contains(&SaveRegion::Patches));
        assert!(!SaveRegion::manifest_for_version(7).contains(&SaveRegion::Markers));
        assert!(!SaveRegion::manifest_for_version(6).contains(&SaveRegion::Custom));
        assert_eq!(
            SaveRegion::manifest_for_version(4),
            &[
                SaveRegion::Meta,
                SaveRegion::Content,
                SaveRegion::Map,
                SaveRegion::Entities,
            ]
        );
    }

    #[test]
    fn save_region_manifest_for_versions_matches_java_save_version_gates() {
        assert_eq!(
            SaveRegion::manifest_for_version(11),
            &[
                SaveRegion::Meta,
                SaveRegion::Content,
                SaveRegion::Patches,
                SaveRegion::Map,
                SaveRegion::Entities,
                SaveRegion::Markers,
                SaveRegion::Custom,
            ]
        );
        assert_eq!(
            SaveRegion::manifest_for_version(10),
            &[
                SaveRegion::Meta,
                SaveRegion::Content,
                SaveRegion::Map,
                SaveRegion::Entities,
                SaveRegion::Markers,
                SaveRegion::Custom,
            ]
        );
        assert_eq!(
            SaveRegion::manifest_for_version(8),
            SaveRegion::manifest_for_version(10)
        );
        assert_eq!(
            SaveRegion::manifest_for_version(7),
            &[
                SaveRegion::Meta,
                SaveRegion::Content,
                SaveRegion::Map,
                SaveRegion::Entities,
                SaveRegion::Custom,
            ]
        );
        assert_eq!(
            SaveRegion::manifest_for_version(6),
            &[
                SaveRegion::Meta,
                SaveRegion::Content,
                SaveRegion::Map,
                SaveRegion::Entities,
            ]
        );
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
    fn deflated_raw_save_envelope_roundtrips_all_java_save_version_regions() {
        let mut envelope = RawSaveEnvelope::new(11);
        for (index, region) in SaveRegion::manifest_for_version(11).iter().enumerate() {
            envelope
                .set(*region, vec![index as u8, region.as_str().len() as u8])
                .unwrap();
        }

        let mut deflated = Vec::new();
        write_deflated_raw_save_envelope(&mut deflated, &envelope).unwrap();
        let decoded = read_deflated_raw_save_envelope(deflated.as_slice()).unwrap();

        assert_eq!(decoded, envelope);
        assert_eq!(decoded.version, 11);
        assert_eq!(
            decoded
                .regions
                .iter()
                .map(|entry| entry.region)
                .collect::<Vec<_>>(),
            SaveRegion::manifest_for_version(11)
        );
        assert_eq!(
            decoded.get(SaveRegion::Markers),
            envelope.get(SaveRegion::Markers)
        );
        assert_eq!(
            decoded.get(SaveRegion::Custom),
            envelope.get(SaveRegion::Custom)
        );
    }

    #[test]
    fn deflated_raw_save_envelope_roundtrips_version_gated_region_sets() {
        for version in [10, 8, 7, 6] {
            let mut envelope = RawSaveEnvelope::new(version);
            for (index, region) in SaveRegion::manifest_for_version(version).iter().enumerate() {
                envelope
                    .set(*region, vec![version as u8, index as u8])
                    .unwrap();
            }

            let mut deflated = Vec::new();
            write_deflated_raw_save_envelope(&mut deflated, &envelope).unwrap();
            let decoded = read_deflated_raw_save_envelope(deflated.as_slice()).unwrap();

            assert_eq!(decoded, envelope);
            assert_eq!(
                decoded
                    .regions
                    .iter()
                    .map(|entry| entry.region)
                    .collect::<Vec<_>>(),
                SaveRegion::manifest_for_version(version)
            );
        }
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
    fn save_path_layout_matches_java_slot_sector_and_backup_names() {
        let layout = SavePathLayout::new("saves");

        assert_eq!(slot_file_name(0), "0.msav");
        assert_eq!(layout.file_for_slot(2), PathBuf::from("saves/2.msav"));
        assert_eq!(sector_file_name("serpulo", 173), "sector-serpulo-173.msav");
        assert_eq!(
            layout.sector_file("erekir", 12),
            PathBuf::from("saves/sector-erekir-12.msav")
        );

        assert_eq!(backup_file_name_for("0.msav"), "0.msav-backup.msav");
        assert_eq!(
            backup_file_for_path("saves/0.msav"),
            PathBuf::from("saves/0.msav-backup.msav")
        );
        assert!(is_backup_save_name("0.msav-backup.msav"));
    }

    #[test]
    fn next_slot_file_uses_first_missing_number_like_saves_get_next_slot_file() {
        let layout = SavePathLayout::new("saves");
        let existing = [
            PathBuf::from("saves/0.msav"),
            PathBuf::from("saves/1.msav"),
            PathBuf::from("saves/3.msav"),
            PathBuf::from("saves/2.msav-backup.msav"),
        ];

        assert_eq!(
            next_slot_file_name(["0.msav", "1.msav", "3.msav", "2.msav-backup.msav"]),
            "2.msav"
        );
        assert_eq!(
            layout.next_slot_file(existing),
            PathBuf::from("saves/2.msav")
        );
    }

    #[test]
    fn save_slot_record_derives_java_settings_and_preview_keys() {
        let numbered = SaveSlotRecord::new("saves/7.msav");
        assert_eq!(numbered.index(), "7");
        assert_eq!(numbered.kind(), SaveSlotKind::Numbered(7));
        assert_eq!(numbered.name_setting_key(), "save-7-name");
        assert_eq!(numbered.autosave_setting_key(), "save-7-autosave");
        assert_eq!(
            numbered.preview_file("previews"),
            PathBuf::from("previews/save_slot_7.png")
        );
        assert_eq!(
            numbered.load_preview_file("previews"),
            PathBuf::from("previews/save_slot_7.png.spreview")
        );

        let sector = SaveSlotRecord::new("saves/sector-serpulo-85.msav");
        assert_eq!(
            sector.kind(),
            SaveSlotKind::Sector {
                planet: "serpulo".into(),
                id: 85
            }
        );
        assert!(sector.is_sector_file());
        assert_eq!(LAST_SECTOR_SAVE_SETTING, "last-sector-save");
    }

    #[test]
    fn deflated_raw_save_envelope_falls_back_to_backup_like_save_io_load() {
        let mut tags = BTreeMap::new();
        tags.insert("version".into(), "157".into());
        tags.insert("mapname".into(), "backup-envelope".into());
        let mut meta_payload = Vec::new();
        write_string_map(&mut meta_payload, &tags).unwrap();

        let mut backup_envelope = RawSaveEnvelope::new(LATEST_SAVE_VERSION);
        backup_envelope.set(SaveRegion::Meta, meta_payload).unwrap();
        let mut backup = Vec::new();
        write_deflated_raw_save_envelope(&mut backup, &backup_envelope).unwrap();

        let decoded = read_deflated_raw_save_envelope_with_backup(b"broken", Some(&backup))
            .expect("backup envelope should load when primary is invalid");
        assert_eq!(decoded, backup_envelope);
        assert!(is_deflated_save_valid_with_backup(b"broken", Some(&backup)));
        assert!(read_deflated_raw_save_envelope_with_backup(b"broken", None).is_err());
    }

    #[test]
    fn saves_load_scan_skips_backups_and_keeps_valid_slots_only() {
        let mut tags = BTreeMap::new();
        tags.insert("version".into(), "157".into());
        tags.insert("mapname".into(), "primary".into());
        tags.insert("saved".into(), "11".into());
        let primary = deflated_meta_bytes(&tags);

        let mut backup_tags = BTreeMap::new();
        backup_tags.insert("version".into(), "157".into());
        backup_tags.insert("mapname".into(), "backup".into());
        backup_tags.insert("saved".into(), "22".into());
        let backup = deflated_meta_bytes(&backup_tags);

        let files = vec![
            DeflatedSaveFile::new("saves/0.msav", primary),
            DeflatedSaveFile::with_backup("saves/1.msav", b"broken".to_vec(), backup),
            DeflatedSaveFile::new("saves/2.msav-backup.msav", deflated_meta_bytes(&tags)),
            DeflatedSaveFile::new("saves/3.msav", b"broken".to_vec()),
        ];

        let slots = collect_valid_deflated_save_slots(&files);
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].index(), "0");
        assert_eq!(
            slots[0].meta.as_ref().unwrap().map_name.as_deref(),
            Some("primary")
        );
        assert_eq!(slots[1].index(), "1");
        assert_eq!(slots[1].timestamp(), 22);
        assert_eq!(
            slots[1].meta.as_ref().unwrap().map_name.as_deref(),
            Some("backup")
        );
    }

    #[test]
    fn last_sector_save_matches_java_name_setting_value() {
        let mut tags = BTreeMap::new();
        tags.insert("version".into(), "157".into());
        tags.insert("rules".into(), r#"{"sector":{"id":10}}"#.into());
        let sector = SaveSlotRecord::with_meta("saves/1.msav", SaveMeta::from_tags(tags));

        let mut non_sector_tags = BTreeMap::new();
        non_sector_tags.insert("version".into(), "157".into());
        non_sector_tags.insert("rules".into(), r#"{"sector":null}"#.into());
        let normal = SaveSlotRecord::with_meta(
            "saves/sector-serpulo-10.msav",
            SaveMeta::from_tags(non_sector_tags),
        );
        let slots = vec![normal, sector];
        assert!(slots[0].is_sector_file());
        assert!(!slots[0].is_sector());
        assert!(!slots[1].is_sector_file());
        assert!(slots[1].is_sector());

        let last = find_last_sector_save(&slots, "1", |slot| slot.index())
            .expect("sector slot should match by stored setting name");
        assert_eq!(last.index(), "1");
        assert_eq!(LAST_SECTOR_SAVE_FALLBACK, "<none>");

        assert!(find_last_sector_save(&slots, "sector-serpulo-10", |slot| slot.index()).is_none());
    }

    #[test]
    fn save_slot_delete_targets_delete_backup_before_primary_like_java() {
        let slot = SaveSlotRecord::new("saves/4.msav");
        assert_eq!(
            slot.delete_targets(),
            [
                PathBuf::from("saves/4.msav-backup.msav"),
                PathBuf::from("saves/4.msav")
            ]
        );
    }

    #[test]
    fn raw_save_envelope_rejects_regions_not_present_for_version() {
        let mut envelope = RawSaveEnvelope::new(10);
        let err = envelope
            .set(SaveRegion::Patches, vec![1])
            .expect_err("patches region should not exist before save v11");
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    fn deflated_meta_bytes(tags: &BTreeMap<String, String>) -> Vec<u8> {
        let mut bytes = Vec::new();
        write_deflated_save_meta_prefix(&mut bytes, LATEST_SAVE_VERSION, tags).unwrap();
        bytes
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
    fn marker_region_ubjson_summary_counts_types_and_missing_or_unknown_class_tags() {
        let markers = MarkerRegionBytes {
            bytes: ubjson_object(vec![
                ("1", ubjson_marker_object(Some("Minimap"))),
                ("2", ubjson_marker_object(None)),
                ("3", ubjson_marker_object(Some("Bogus"))),
            ]),
        };

        let summary = markers.ubjson_summary().unwrap();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.recognized_by_type.get("point"), Some(&1));
        assert_eq!(summary.missing_class_count, 1);
        assert_eq!(summary.unrecognized_type_count, 1);
    }

    #[test]
    fn marker_region_ubjson_summary_rejects_invalid_or_non_object_blobs() {
        assert!(summarize_marker_region_bytes(b"not ubjson").is_err());
        assert!(summarize_marker_region_bytes(b"[Z]").is_err());
    }

    #[test]
    fn marker_region_ubjson_parse_materializes_point_markers_and_common_fields() {
        let markers = MarkerRegionBytes {
            bytes: ubjson_object(vec![(
                "7",
                ubjson_marker_object_with_fields(
                    Some("Minimap"),
                    Some(false),
                    Some(true),
                    Some(true),
                    Some(33.5),
                ),
            )]),
        };

        let decoded = markers.ubjson_markers().unwrap();
        assert_eq!(decoded.size(), 1);
        assert_eq!(decoded.ids().collect::<Vec<_>>(), vec![7]);
        let marker = decoded.get(7).unwrap();
        assert_eq!(marker.type_name(), "Point");
        assert_eq!(marker.common().array_index, 0);
        assert!(!marker.common().world);
        assert!(marker.common().minimap);
        assert!(marker.common().autoscale);
        assert!((marker.common().draw_layer - 33.5).abs() < f32::EPSILON);
    }

    #[test]
    fn marker_region_ubjson_parse_preserves_marker_id_and_array_index_semantics() {
        let markers = MarkerRegionBytes {
            bytes: ubjson_object(vec![
                ("2", ubjson_marker_object(Some("Shape"))),
                ("9", ubjson_marker_object(Some("Text"))),
                ("9", ubjson_marker_object(Some("Minimap"))),
            ]),
        };

        let decoded = markers.ubjson_markers().unwrap();
        assert_eq!(decoded.size(), 2);
        assert_eq!(decoded.ids().collect::<Vec<_>>(), vec![2, 9]);
        assert_eq!(decoded.get(2).unwrap().common().array_index, 0);
        assert_eq!(decoded.get(9).unwrap().type_name(), "Point");
        assert_eq!(decoded.get(9).unwrap().common().array_index, 1);
    }

    #[test]
    fn marker_region_ubjson_parse_skips_unknown_and_missing_class_entries() {
        let markers = MarkerRegionBytes {
            bytes: ubjson_object(vec![
                ("1", ubjson_marker_object(Some("Bogus"))),
                ("2", ubjson_marker_object(None)),
                ("3", ubjson_marker_object(Some("Shape"))),
            ]),
        };

        let decoded = markers.ubjson_markers().unwrap();
        assert_eq!(decoded.size(), 1);
        assert_eq!(decoded.ids().collect::<Vec<_>>(), vec![3]);
        assert_eq!(decoded.get(3).unwrap().type_name(), "Shape");
    }

    #[test]
    fn marker_region_ubjson_parse_point_text_and_shape_text_fields() {
        let markers = MarkerRegionBytes {
            bytes: ubjson_object(vec![
                (
                    "4",
                    ubjson_marker_object_with_extra_fields(
                        Some("Minimap"),
                        vec![
                            ubjson_vec2_field("pos", 16.0, 24.0),
                            ubjson_f32_field("radius", 3.5),
                            ubjson_f32_field("stroke", 9.0),
                        ],
                    ),
                ),
                (
                    "5",
                    ubjson_marker_object_with_extra_fields(
                        Some("Text"),
                        vec![
                            ubjson_vec2_field("pos", 2.0, 4.0),
                            ubjson_string_field_bytes("text", "hello"),
                            ubjson_f32_field("fontSize", 1.5),
                            ubjson_i32_field("flags", 3),
                            ubjson_i32_field("textAlign", 5),
                            ubjson_i32_field("lineAlign", 1),
                        ],
                    ),
                ),
                (
                    "6",
                    ubjson_marker_object_with_extra_fields(
                        Some("ShapeText"),
                        vec![
                            ubjson_vec2_field("pos", 8.0, 12.0),
                            ubjson_string_field_bytes("text", "alert"),
                            ubjson_f32_field("fontSize", 2.0),
                            ubjson_f32_field("textHeight", 11.0),
                            ubjson_f32_field("radius", 7.0),
                            ubjson_f32_field("rotation", 30.0),
                            ubjson_i32_field("sides", 5),
                        ],
                    ),
                ),
            ]),
        };

        let decoded = parse_marker_region_bytes(&markers.bytes).unwrap();

        match decoded.get(4).unwrap() {
            ObjectiveMarker::Point(point) => {
                assert_eq!(point.pos, crate::mindustry::game::Vec2::new(16.0, 24.0));
                assert!((point.radius - 3.5).abs() < f32::EPSILON);
                assert!((point.stroke - 9.0).abs() < f32::EPSILON);
            }
            other => panic!("expected Point marker, got {other:?}"),
        }

        match decoded.get(5).unwrap() {
            ObjectiveMarker::Text(text) => {
                assert_eq!(text.pos, crate::mindustry::game::Vec2::new(2.0, 4.0));
                assert_eq!(text.text, "hello");
                assert!((text.font_size - 1.5).abs() < f32::EPSILON);
                assert_eq!(text.flags, 3);
                assert_eq!(text.text_align, 5);
                assert_eq!(text.line_align, 1);
            }
            other => panic!("expected Text marker, got {other:?}"),
        }

        match decoded.get(6).unwrap() {
            ObjectiveMarker::ShapeText(text) => {
                assert_eq!(text.pos, crate::mindustry::game::Vec2::new(8.0, 12.0));
                assert_eq!(text.text, "alert");
                assert!((text.font_size - 2.0).abs() < f32::EPSILON);
                assert!((text.text_height - 11.0).abs() < f32::EPSILON);
                assert!((text.radius - 7.0).abs() < f32::EPSILON);
                assert!((text.rotation - 30.0).abs() < f32::EPSILON);
                assert_eq!(text.sides, 5);
            }
            other => panic!("expected ShapeText marker, got {other:?}"),
        }
    }

    #[test]
    fn marker_region_ubjson_parse_shape_line_and_texture_fields() {
        let markers = MarkerRegionBytes {
            bytes: ubjson_object(vec![
                (
                    "7",
                    ubjson_marker_object_with_extra_fields(
                        Some("Shape"),
                        vec![
                            ubjson_vec2_field("pos", 1.0, 3.0),
                            ubjson_f32_field("radius", 4.0),
                            ubjson_f32_field("rotation", 45.0),
                            ubjson_f32_field("stroke", 2.5),
                            ubjson_bool_field_bytes("fill", true),
                            ubjson_bool_field_bytes("outline", false),
                            ubjson_i32_field("sides", 6),
                            ubjson_f32_field("startAngle", 15.0),
                            ubjson_f32_field("endAngle", 180.0),
                        ],
                    ),
                ),
                (
                    "8",
                    ubjson_marker_object_with_extra_fields(
                        Some("Line"),
                        vec![
                            ubjson_vec2_field("pos", 0.0, 1.0),
                            ubjson_vec2_field("endPos", 4.0, 5.0),
                            ubjson_f32_field("stroke", 3.0),
                            ubjson_bool_field_bytes("outline", false),
                        ],
                    ),
                ),
                (
                    "9",
                    ubjson_marker_object_with_extra_fields(
                        Some("Texture"),
                        vec![
                            ubjson_vec2_field("pos", 9.0, 10.0),
                            ubjson_f32_field("rotation", 90.0),
                            ubjson_f32_field("width", 32.0),
                            ubjson_f32_field("height", 64.0),
                            ubjson_texture_holder_field("texture", "string", "router"),
                        ],
                    ),
                ),
            ]),
        };

        let decoded = parse_marker_region_bytes(&markers.bytes).unwrap();

        match decoded.get(7).unwrap() {
            ObjectiveMarker::Shape(shape) => {
                assert_eq!(shape.pos, crate::mindustry::game::Vec2::new(1.0, 3.0));
                assert!((shape.radius - 4.0).abs() < f32::EPSILON);
                assert!((shape.rotation - 45.0).abs() < f32::EPSILON);
                assert!((shape.stroke - 2.5).abs() < f32::EPSILON);
                assert!(shape.fill);
                assert!(!shape.outline);
                assert_eq!(shape.sides, 6);
                assert!((shape.start_angle - 15.0).abs() < f32::EPSILON);
                assert!((shape.end_angle - 180.0).abs() < f32::EPSILON);
            }
            other => panic!("expected Shape marker, got {other:?}"),
        }

        match decoded.get(8).unwrap() {
            ObjectiveMarker::Line(line) => {
                assert_eq!(line.pos, crate::mindustry::game::Vec2::new(0.0, 1.0));
                assert_eq!(line.end_pos, crate::mindustry::game::Vec2::new(4.0, 5.0));
                assert!((line.stroke - 3.0).abs() < f32::EPSILON);
                assert!(!line.outline);
            }
            other => panic!("expected Line marker, got {other:?}"),
        }

        match decoded.get(9).unwrap() {
            ObjectiveMarker::Texture(texture) => {
                assert_eq!(texture.pos, crate::mindustry::game::Vec2::new(9.0, 10.0));
                assert!((texture.rotation - 90.0).abs() < f32::EPSILON);
                assert!((texture.width - 32.0).abs() < f32::EPSILON);
                assert!((texture.height - 64.0).abs() < f32::EPSILON);
                assert_eq!(
                    texture.texture,
                    crate::mindustry::game::map_objectives::TextureHolder::String("router".into())
                );
            }
            other => panic!("expected Texture marker, got {other:?}"),
        }
    }

    #[test]
    fn marker_region_ubjson_parse_marker_colors_and_legacy_texture_name() {
        let markers = MarkerRegionBytes {
            bytes: ubjson_object(vec![
                (
                    "10",
                    ubjson_marker_object_with_extra_fields(
                        Some("ShapeText"),
                        vec![ubjson_string_field_bytes("color", "11223344")],
                    ),
                ),
                (
                    "11",
                    ubjson_marker_object_with_extra_fields(
                        Some("Point"),
                        vec![ubjson_color_object_field("color", 0.25, 0.5, 0.75, 1.0)],
                    ),
                ),
                (
                    "12",
                    ubjson_marker_object_with_extra_fields(
                        Some("Shape"),
                        vec![ubjson_f32_array_field("color", &[1.0, 0.5, 0.0, 0.25])],
                    ),
                ),
                (
                    "13",
                    ubjson_marker_object_with_extra_fields(
                        Some("Line"),
                        vec![
                            ubjson_string_field_bytes("color1", "#01020304"),
                            ubjson_string_field_bytes("color2", "05060708"),
                        ],
                    ),
                ),
                (
                    "14",
                    ubjson_marker_object_with_extra_fields(
                        Some("Texture"),
                        vec![
                            ubjson_string_field_bytes("textureName", "router"),
                            ubjson_string_field_bytes("color", "aabbccdd"),
                        ],
                    ),
                ),
            ]),
        };

        let decoded = parse_marker_region_bytes(&markers.bytes).unwrap();

        match decoded.get(10).unwrap() {
            ObjectiveMarker::ShapeText(marker) => assert_eq!(marker.color, 0x11223344),
            other => panic!("expected ShapeText marker, got {other:?}"),
        }
        match decoded.get(11).unwrap() {
            ObjectiveMarker::Point(marker) => assert_eq!(marker.color, 0x4080bfff),
            other => panic!("expected Point marker, got {other:?}"),
        }
        match decoded.get(12).unwrap() {
            ObjectiveMarker::Shape(marker) => assert_eq!(marker.color, 0xff800040),
            other => panic!("expected Shape marker, got {other:?}"),
        }
        match decoded.get(13).unwrap() {
            ObjectiveMarker::Line(marker) => {
                assert_eq!(marker.color1, 0x01020304);
                assert_eq!(marker.color2, 0x05060708);
            }
            other => panic!("expected Line marker, got {other:?}"),
        }
        match decoded.get(14).unwrap() {
            ObjectiveMarker::Texture(marker) => {
                assert_eq!(marker.color, 0xaabbccdd);
                assert_eq!(
                    marker.texture,
                    crate::mindustry::game::map_objectives::TextureHolder::String("router".into())
                );
            }
            other => panic!("expected Texture marker, got {other:?}"),
        }
    }

    #[test]
    fn marker_region_ubjson_parse_quad_vertices_texture_name_and_map_region() {
        let vertices = [
            1.0, 2.0, 3.0, 0.0, 1.0, 0.0, 4.0, 5.0, 6.0, 1.0, 1.0, 0.0, 7.0, 8.0, 9.0, 1.0, 0.0,
            0.0, 10.0, 11.0, 12.0, 0.0, 0.0, 0.0,
        ];
        let markers = MarkerRegionBytes {
            bytes: ubjson_object(vec![(
                "15",
                ubjson_marker_object_with_extra_fields(
                    Some("Quad"),
                    vec![
                        ubjson_string_field_bytes("textureName", "white"),
                        ubjson_f32_array_field("vertices", &vertices),
                        ubjson_bool_field_bytes("mapRegion", false),
                    ],
                ),
            )]),
        };

        let decoded = parse_marker_region_bytes(&markers.bytes).unwrap();

        match decoded.get(15).unwrap() {
            ObjectiveMarker::Quad(marker) => {
                assert_eq!(
                    marker.texture,
                    crate::mindustry::game::map_objectives::TextureHolder::String("white".into())
                );
                assert_eq!(marker.vertices, vertices);
                assert!(!marker.map_region);
            }
            other => panic!("expected Quad marker, got {other:?}"),
        }
    }

    #[test]
    fn marker_region_writer_roundtrips_all_marker_variants() {
        use crate::mindustry::game::map_objectives::{
            LineMarker, PointMarker, QuadMarker, ShapeMarker, ShapeTextMarker, TextMarker,
            TextureMarker,
        };

        let mut markers = MapMarkers::new();

        let mut shape_text = ShapeTextMarker::default();
        shape_text.common.world = false;
        shape_text.common.minimap = true;
        shape_text.common.autoscale = true;
        shape_text.common.draw_layer = 99.5;
        shape_text.pos = Vec2::new(1.0, 2.0);
        shape_text.text = "alpha".into();
        shape_text.font_size = 2.0;
        shape_text.text_height = 8.0;
        shape_text.flags = 1;
        shape_text.text_align = 2;
        shape_text.line_align = 3;
        shape_text.radius = 4.0;
        shape_text.rotation = 5.0;
        shape_text.sides = 6;
        shape_text.color = 0x11223344;
        markers.add(7, ObjectiveMarker::ShapeText(shape_text));

        let mut point = PointMarker::default();
        point.pos = Vec2::new(3.0, 4.0);
        point.radius = 5.5;
        point.stroke = 6.5;
        point.color = 0x55667788;
        markers.add(1, ObjectiveMarker::Point(point));

        let mut shape = ShapeMarker::default();
        shape.pos = Vec2::new(5.0, 6.0);
        shape.radius = 7.0;
        shape.rotation = 8.0;
        shape.stroke = 9.0;
        shape.start_angle = 10.0;
        shape.end_angle = 11.0;
        shape.fill = true;
        shape.outline = false;
        shape.sides = 12;
        shape.color = 0x99aabbcc;
        markers.add(3, ObjectiveMarker::Shape(shape));

        let mut text = TextMarker::default();
        text.pos = Vec2::new(7.0, 8.0);
        text.text = "beta".into();
        text.font_size = 3.0;
        text.flags = 2;
        text.text_align = 4;
        text.line_align = 5;
        markers.add(4, ObjectiveMarker::Text(text));

        let mut line = LineMarker::default();
        line.pos = Vec2::new(9.0, 10.0);
        line.end_pos = Vec2::new(11.0, 12.0);
        line.stroke = 13.0;
        line.outline = false;
        line.color1 = 0x01020304;
        line.color2 = 0x05060708;
        markers.add(5, ObjectiveMarker::Line(line));

        let mut texture = TextureMarker::default();
        texture.pos = Vec2::new(13.0, 14.0);
        texture.rotation = 15.0;
        texture.width = 16.0;
        texture.height = 17.0;
        texture.texture = TextureHolder::Content("router".into());
        texture.color = 0xaabbccdd;
        markers.add(6, ObjectiveMarker::Texture(texture));

        let mut quad = QuadMarker::default();
        quad.texture = TextureHolder::Building(42);
        quad.vertices = (0..24).map(|index| index as f32 + 0.25).collect();
        quad.map_region = false;
        markers.add(2, ObjectiveMarker::Quad(quad));

        let bytes = marker_region_bytes_from_map_markers(&markers).unwrap();
        let decoded = parse_marker_region_bytes(&bytes).unwrap();
        let mut expected_entries = markers
            .entries()
            .map(|(id, marker)| (id, marker.clone()))
            .collect::<Vec<_>>();
        expected_entries.sort_by_key(|(id, _)| *id);
        let expected = MapMarkers::rebuild_from_entries(expected_entries);

        assert_eq!(decoded.ids().collect::<Vec<_>>(), vec![1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(decoded, expected);
    }

    fn ubjson_object(entries: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'{');
        for (key, value) in entries {
            write_ubjson_string_map_entry(&mut bytes, key, &value);
        }
        bytes.push(b'}');
        bytes
    }

    fn ubjson_marker_object(class: Option<&str>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'{');
        if let Some(class) = class {
            write_ubjson_string_field(&mut bytes, "class", class);
        }
        bytes.push(b'}');
        bytes
    }

    fn ubjson_marker_object_with_fields(
        class: Option<&str>,
        world: Option<bool>,
        minimap: Option<bool>,
        autoscale: Option<bool>,
        draw_layer: Option<f32>,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'{');
        if let Some(class) = class {
            write_ubjson_string_field(&mut bytes, "class", class);
        }
        if let Some(world) = world {
            write_ubjson_bool_field(&mut bytes, "world", world);
        }
        if let Some(minimap) = minimap {
            write_ubjson_bool_field(&mut bytes, "minimap", minimap);
        }
        if let Some(autoscale) = autoscale {
            write_ubjson_bool_field(&mut bytes, "autoscale", autoscale);
        }
        if let Some(draw_layer) = draw_layer {
            write_ubjson_f32_field(&mut bytes, "drawLayer", draw_layer);
        }
        bytes.push(b'}');
        bytes
    }

    fn ubjson_marker_object_with_extra_fields(
        class: Option<&str>,
        extra_fields: Vec<Vec<u8>>,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'{');
        if let Some(class) = class {
            write_ubjson_string_field(&mut bytes, "class", class);
        }
        for field in extra_fields {
            bytes.extend_from_slice(&field);
        }
        bytes.push(b'}');
        bytes
    }

    fn write_ubjson_string_map_entry(write: &mut Vec<u8>, key: &str, value: &[u8]) {
        write.push(b'U');
        write.push(key.len() as u8);
        write.extend_from_slice(key.as_bytes());
        write.extend_from_slice(value);
    }

    fn write_ubjson_string_field(write: &mut Vec<u8>, key: &str, value: &str) {
        write.push(b'U');
        write.push(key.len() as u8);
        write.extend_from_slice(key.as_bytes());
        write.push(b'S');
        write.push(b'U');
        write.push(value.len() as u8);
        write.extend_from_slice(value.as_bytes());
    }

    fn write_ubjson_bool_field(write: &mut Vec<u8>, key: &str, value: bool) {
        write.push(b'U');
        write.push(key.len() as u8);
        write.extend_from_slice(key.as_bytes());
        write.push(if value { b'T' } else { b'F' });
    }

    fn write_ubjson_f32_field(write: &mut Vec<u8>, key: &str, value: f32) {
        write.push(b'U');
        write.push(key.len() as u8);
        write.extend_from_slice(key.as_bytes());
        write.push(b'd');
        write.extend_from_slice(&value.to_be_bytes());
    }

    fn ubjson_string_field_bytes(key: &str, value: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        write_ubjson_string_field(&mut bytes, key, value);
        bytes
    }

    fn ubjson_bool_field_bytes(key: &str, value: bool) -> Vec<u8> {
        let mut bytes = Vec::new();
        write_ubjson_bool_field(&mut bytes, key, value);
        bytes
    }

    fn ubjson_f32_field(key: &str, value: f32) -> Vec<u8> {
        let mut bytes = Vec::new();
        write_ubjson_f32_field(&mut bytes, key, value);
        bytes
    }

    fn ubjson_f32_array_field(key: &str, values: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'U');
        bytes.push(key.len() as u8);
        bytes.extend_from_slice(key.as_bytes());
        bytes.push(b'[');
        for value in values {
            bytes.push(b'd');
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        bytes.push(b']');
        bytes
    }

    fn ubjson_i32_field(key: &str, value: i32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'U');
        bytes.push(key.len() as u8);
        bytes.extend_from_slice(key.as_bytes());
        bytes.push(b'l');
        bytes.extend_from_slice(&value.to_be_bytes());
        bytes
    }

    fn ubjson_vec2_field(key: &str, x: f32, y: f32) -> Vec<u8> {
        let mut value = Vec::new();
        value.push(b'{');
        write_ubjson_f32_field(&mut value, "x", x);
        write_ubjson_f32_field(&mut value, "y", y);
        value.push(b'}');

        let mut bytes = Vec::new();
        bytes.push(b'U');
        bytes.push(key.len() as u8);
        bytes.extend_from_slice(key.as_bytes());
        bytes.extend_from_slice(&value);
        bytes
    }

    fn ubjson_color_object_field(key: &str, r: f32, g: f32, b: f32, a: f32) -> Vec<u8> {
        let mut value = Vec::new();
        value.push(b'{');
        write_ubjson_f32_field(&mut value, "r", r);
        write_ubjson_f32_field(&mut value, "g", g);
        write_ubjson_f32_field(&mut value, "b", b);
        write_ubjson_f32_field(&mut value, "a", a);
        value.push(b'}');

        let mut bytes = Vec::new();
        bytes.push(b'U');
        bytes.push(key.len() as u8);
        bytes.extend_from_slice(key.as_bytes());
        bytes.extend_from_slice(&value);
        bytes
    }

    fn ubjson_texture_holder_field(key: &str, nested_key: &str, value: &str) -> Vec<u8> {
        let mut nested = Vec::new();
        nested.push(b'{');
        write_ubjson_string_field(&mut nested, nested_key, value);
        nested.push(b'}');

        let mut bytes = Vec::new();
        bytes.push(b'U');
        bytes.push(key.len() as u8);
        bytes.extend_from_slice(key.as_bytes());
        bytes.extend_from_slice(&nested);
        bytes
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
