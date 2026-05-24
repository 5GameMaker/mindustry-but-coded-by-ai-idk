//! Save-version adapters mirroring upstream `mindustry.io.versions`.

use std::{
    collections::HashSet,
    io::{self, Read, Write},
};

use crate::mindustry::io::{
    save::{read_chunk, read_legacy_short_chunk, write_chunk},
    type_io::{
        read_i16, read_i32, read_java_utf, read_object, read_u16, read_u8, write_i16, write_i32,
        write_object, write_u16, write_u8, TypeValue,
    },
};
use crate::mindustry::world::{Tile, Tiles};

pub mod legacy_io;
pub mod save1;
pub mod save10;
pub mod save11;
pub mod save2;
pub mod save3;
pub mod save4;
pub mod save5;
pub mod save6;
pub mod save7;
pub mod save8;
pub mod save9;
pub mod short_chunk_save_version;

pub use legacy_io::{
    legacy_unit_name, read_legacy_servers, read_legacy_servers_from, read_legacy_servers_result,
    LegacyServer, LEGACY_SERVER_LIST_SETTING, LEGACY_UNIT_MAP,
};
pub use save1::Save1;
pub use save10::Save10;
pub use save11::Save11;
pub use save2::Save2;
pub use save3::Save3;
pub use save4::Save4;
pub use save5::Save5;
pub use save6::Save6;
pub use save7::Save7;
pub use save8::Save8;
pub use save9::Save9;
pub use short_chunk_save_version::ShortChunkSaveVersion;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyEntityChunk {
    pub group_index: u8,
    pub entity_index: i32,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyEntityGroup {
    pub group_index: u8,
    pub chunks: Vec<LegacyEntityChunk>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LegacyEntityGroups {
    pub groups: Vec<LegacyEntityGroup>,
}

impl LegacyEntityGroups {
    pub fn total_entities(&self) -> usize {
        self.groups.iter().map(|group| group.chunks.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty() || self.total_entities() == 0
    }
}

pub fn read_legacy_entity_groups<R: Read>(read: &mut R) -> io::Result<LegacyEntityGroups> {
    let group_count = read_u8(read)?;
    let mut groups = Vec::with_capacity(group_count as usize);
    for group_index in 0..group_count {
        let amount = read_i32(read)?;
        if amount < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "negative legacy entity count",
            ));
        }

        let mut chunks = Vec::with_capacity(amount as usize);
        for entity_index in 0..amount {
            chunks.push(LegacyEntityChunk {
                group_index,
                entity_index,
                bytes: read_legacy_short_chunk(read)?,
            });
        }
        groups.push(LegacyEntityGroup {
            group_index,
            chunks,
        });
    }
    Ok(LegacyEntityGroups { groups })
}

pub fn skip_legacy_entity_groups<R: Read>(read: &mut R) -> io::Result<usize> {
    read_legacy_entity_groups(read).map(|groups| groups.total_entities())
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTeamBlockPlan {
    pub x: i16,
    pub y: i16,
    pub rotation: i16,
    pub block_id: i16,
    pub config: TypeValue,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LegacyTeamBlockGroup {
    pub team_id: i32,
    pub plans: Vec<LegacyTeamBlockPlan>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LegacyTeamBlocks {
    pub groups: Vec<LegacyTeamBlockGroup>,
}

impl LegacyTeamBlocks {
    pub fn total_plans(&self) -> usize {
        self.groups.iter().map(|group| group.plans.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty() || self.total_plans() == 0
    }
}

pub fn read_legacy_team_blocks<R: Read>(read: &mut R) -> io::Result<LegacyTeamBlocks> {
    let team_count = read_i32(read)?;
    if team_count < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative legacy team block group count",
        ));
    }

    let mut groups = Vec::with_capacity(team_count as usize);
    for _ in 0..team_count {
        let team_id = read_i32(read)?;
        let block_count = read_i32(read)?;
        if block_count < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "negative legacy team block plan count",
            ));
        }

        let mut plans = Vec::with_capacity(block_count.min(1000) as usize);
        let mut seen_positions = HashSet::new();
        for _ in 0..block_count {
            let x = read_i16(read)?;
            let y = read_i16(read)?;
            let rotation = read_i16(read)?;
            let block_id = read_i16(read)?;
            let config = read_object(read)?;

            // Java `SaveVersion.readTeamBlocks` ignores duplicate positions
            // after consuming the config object.
            if seen_positions.insert((x, y)) {
                plans.push(LegacyTeamBlockPlan {
                    x,
                    y,
                    rotation,
                    block_id,
                    config,
                });
            }
        }
        groups.push(LegacyTeamBlockGroup { team_id, plans });
    }

    Ok(LegacyTeamBlocks { groups })
}

pub fn write_legacy_team_blocks<W: Write>(
    write: &mut W,
    blocks: &LegacyTeamBlocks,
) -> io::Result<()> {
    if blocks.groups.len() > i32::MAX as usize {
        return Err(invalid_input("legacy team block group count too large"));
    }

    write_i32(write, blocks.groups.len() as i32)?;
    for group in &blocks.groups {
        if group.plans.len() > i32::MAX as usize {
            return Err(invalid_input("legacy team block plan count too large"));
        }

        write_i32(write, group.team_id)?;
        write_i32(write, group.plans.len() as i32)?;
        for plan in &group.plans {
            write_i16(write, plan.x)?;
            write_i16(write, plan.y)?;
            write_i16(write, plan.rotation)?;
            write_i16(write, plan.block_id)?;
            write_object(write, &plan.config)?;
        }
    }

    Ok(())
}

pub fn read_legacy_int_config_team_blocks<R: Read>(read: &mut R) -> io::Result<LegacyTeamBlocks> {
    let team_count = read_i32(read)?;
    if team_count < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative legacy team block group count",
        ));
    }

    let mut groups = Vec::with_capacity(team_count as usize);
    for _ in 0..team_count {
        let team_id = read_i32(read)?;
        let block_count = read_i32(read)?;
        if block_count < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "negative legacy team block plan count",
            ));
        }

        let mut plans = Vec::with_capacity(block_count as usize);
        for _ in 0..block_count {
            plans.push(LegacyTeamBlockPlan {
                x: read_i16(read)?,
                y: read_i16(read)?,
                rotation: read_i16(read)?,
                block_id: read_i16(read)?,
                config: TypeValue::Int(read_i32(read)?),
            });
        }
        groups.push(LegacyTeamBlockGroup { team_id, plans });
    }

    Ok(LegacyTeamBlocks { groups })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyEntityMappingEntry {
    pub id: i16,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LegacyEntityMapping {
    pub entries: Vec<LegacyEntityMappingEntry>,
}

pub fn read_legacy_entity_mapping<R: Read>(read: &mut R) -> io::Result<LegacyEntityMapping> {
    let amount = read_i16(read)?;
    if amount < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative legacy entity mapping count",
        ));
    }

    let mut entries = Vec::with_capacity(amount as usize);
    for _ in 0..amount {
        entries.push(LegacyEntityMappingEntry {
            id: read_i16(read)?,
            name: read_java_utf(read)?,
        });
    }
    Ok(LegacyEntityMapping { entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyWorldEntityChunk {
    pub type_id: u8,
    pub entity_id: Option<i32>,
    pub body: Vec<u8>,
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LegacyWorldEntities {
    pub chunks: Vec<LegacyWorldEntityChunk>,
}

impl LegacyWorldEntities {
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

pub fn read_legacy_world_entities<R: Read>(read: &mut R) -> io::Result<LegacyWorldEntities> {
    let amount = read_i32(read)?;
    if amount < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative legacy world entity count",
        ));
    }

    let mut chunks = Vec::with_capacity(amount as usize);
    for _ in 0..amount {
        chunks.push(parse_legacy_world_entity_chunk(read_chunk(read)?)?);
    }

    Ok(LegacyWorldEntities { chunks })
}

pub fn skip_legacy_world_entities<R: Read>(read: &mut R) -> io::Result<usize> {
    read_legacy_world_entities(read).map(|entities| entities.len())
}

pub fn read_legacy_short_world_entities<R: Read>(read: &mut R) -> io::Result<LegacyWorldEntities> {
    let amount = read_i32(read)?;
    if amount < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative legacy world entity count",
        ));
    }

    let mut chunks = Vec::with_capacity(amount as usize);
    for _ in 0..amount {
        chunks.push(parse_legacy_world_entity_chunk(read_legacy_short_chunk(
            read,
        )?)?);
    }

    Ok(LegacyWorldEntities { chunks })
}

pub fn skip_legacy_short_world_entities<R: Read>(read: &mut R) -> io::Result<usize> {
    read_legacy_short_world_entities(read).map(|entities| entities.len())
}

pub fn read_legacy_short_world_entities_without_ids<R: Read>(
    read: &mut R,
) -> io::Result<LegacyWorldEntities> {
    let amount = read_i32(read)?;
    if amount < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative legacy world entity count",
        ));
    }

    let mut chunks = Vec::with_capacity(amount as usize);
    for _ in 0..amount {
        chunks.push(parse_legacy_world_entity_chunk_without_id(
            read_legacy_short_chunk(read)?,
        )?);
    }

    Ok(LegacyWorldEntities { chunks })
}

pub fn skip_legacy_short_world_entities_without_ids<R: Read>(read: &mut R) -> io::Result<usize> {
    read_legacy_short_world_entities_without_ids(read).map(|entities| entities.len())
}

fn parse_legacy_world_entity_chunk(raw: Vec<u8>) -> io::Result<LegacyWorldEntityChunk> {
    let (&type_id, rest) = raw.split_first().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "empty legacy world entity chunk",
        )
    })?;

    let (entity_id, body) = if rest.len() >= 4 {
        let id = i32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]);
        (Some(id), rest[4..].to_vec())
    } else {
        (None, rest.to_vec())
    };

    Ok(LegacyWorldEntityChunk {
        type_id,
        entity_id,
        body,
        raw,
    })
}

fn parse_legacy_world_entity_chunk_without_id(raw: Vec<u8>) -> io::Result<LegacyWorldEntityChunk> {
    let (&type_id, rest) = raw.split_first().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "empty legacy world entity chunk",
        )
    })?;

    Ok(LegacyWorldEntityChunk {
        type_id,
        entity_id: None,
        body: rest.to_vec(),
        raw,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyMapFloorRecord {
    pub index: usize,
    pub floor_id: i16,
    pub ore_id: i16,
    pub consecutives: u8,
}

impl LegacyMapFloorRecord {
    pub fn len(&self) -> usize {
        self.consecutives as usize + 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyMapTileData {
    pub data: u8,
    pub floor_data: u8,
    pub overlay_data: u8,
    pub extra_data: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyMapBlockRecord {
    pub index: usize,
    pub block_id: i16,
    pub packed_flags: u8,
    pub has_entity: bool,
    pub has_old_data: bool,
    pub has_new_data: bool,
    pub is_center: bool,
    pub new_data: Option<LegacyMapTileData>,
    pub old_data: Option<u8>,
    pub building: Option<Vec<u8>>,
    pub consecutives: u8,
}

impl LegacyMapBlockRecord {
    pub fn len(&self) -> usize {
        self.consecutives as usize + 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LegacyShortChunkMap {
    pub width: u16,
    pub height: u16,
    pub floors: Vec<LegacyMapFloorRecord>,
    pub blocks: Vec<LegacyMapBlockRecord>,
}

impl LegacyShortChunkMap {
    pub fn tile_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// Expands the Java save-map run records into the lightweight Rust tile
    /// container. Building payload bytes are intentionally left as raw data on
    /// `LegacyMapBlockRecord` until generated building codecs are complete.
    pub fn to_tiles(&self) -> Tiles {
        let mut tiles = Tiles::new(self.width as usize, self.height as usize);
        self.apply_to_tiles(&mut tiles);
        tiles
    }

    pub fn apply_to_tiles(&self, tiles: &mut Tiles) {
        let width = self.width as usize;
        let height = self.height as usize;
        if tiles.width != width || tiles.height != height {
            *tiles = Tiles::new(width, height);
        } else {
            tiles.fill();
        }

        for record in &self.floors {
            for index in record.index..record.index + record.len() {
                if let Some(tile) = tile_mut_by_index(tiles, index) {
                    tile.floor = record.floor_id;
                    tile.overlay = record.ore_id;
                }
            }
        }

        for record in &self.blocks {
            for index in record.index..record.index + record.len() {
                if let Some(tile) = tile_mut_by_index(tiles, index) {
                    tile.block = record.block_id;
                }
            }

            if let Some(tile) = tile_mut_by_index(tiles, record.index) {
                if let Some(data) = &record.new_data {
                    tile.data = data.data;
                    tile.floor_data = data.floor_data;
                    tile.overlay_data = data.overlay_data;
                    tile.extra_data = data.extra_data;
                } else if let Some(data) = record.old_data {
                    tile.data = data;
                }
            }
        }
    }
}

fn tile_mut_by_index(tiles: &mut Tiles, index: usize) -> Option<&mut Tile> {
    let width = tiles.width;
    if width == 0 {
        return None;
    }
    let x = index % width;
    let y = index / width;
    tiles.get_mut(x as i32, y as i32)
}

pub fn read_legacy_short_chunk_map<R: Read>(read: &mut R) -> io::Result<LegacyShortChunkMap> {
    let width = read_u16(read)?;
    let height = read_u16(read)?;
    let tile_count = width as usize * height as usize;

    let mut floors = Vec::new();
    let mut i = 0usize;
    while i < tile_count {
        let floor_id = read_i16(read)?;
        let ore_id = read_i16(read)?;
        let consecutives = read_u8(read)?;
        let len = consecutives as usize + 1;
        if i + len > tile_count {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "legacy floor run exceeds map bounds",
            ));
        }
        floors.push(LegacyMapFloorRecord {
            index: i,
            floor_id,
            ore_id,
            consecutives,
        });
        i += len;
    }

    let mut blocks = Vec::new();
    let mut i = 0usize;
    while i < tile_count {
        let block_id = read_i16(read)?;
        let packed_flags = read_u8(read)?;
        let has_entity = (packed_flags & 1) != 0;
        let has_old_data = (packed_flags & 2) != 0;
        let has_new_data = (packed_flags & 4) != 0;

        let new_data = if has_new_data {
            Some(LegacyMapTileData {
                data: read_u8(read)?,
                floor_data: read_u8(read)?,
                overlay_data: read_u8(read)?,
                extra_data: read_i32(read)?,
            })
        } else {
            None
        };

        let is_center = if has_entity {
            read_u8(read)? != 0
        } else {
            true
        };

        let (building, old_data, consecutives) = if has_entity {
            let building = if is_center {
                Some(read_legacy_short_chunk(read)?)
            } else {
                None
            };
            (building, None, 0)
        } else if has_old_data || has_new_data {
            let old_data = if has_old_data {
                Some(read_u8(read)?)
            } else {
                None
            };
            (None, old_data, 0)
        } else {
            let consecutives = read_u8(read)?;
            let len = consecutives as usize + 1;
            if i + len > tile_count {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "legacy block run exceeds map bounds",
                ));
            }
            (None, None, consecutives)
        };

        blocks.push(LegacyMapBlockRecord {
            index: i,
            block_id,
            packed_flags,
            has_entity,
            has_old_data,
            has_new_data,
            is_center,
            new_data,
            old_data,
            building,
            consecutives,
        });

        i += consecutives as usize + 1;
    }

    Ok(LegacyShortChunkMap {
        width,
        height,
        floors,
        blocks,
    })
}

pub fn read_chunk_map<R: Read>(read: &mut R) -> io::Result<LegacyShortChunkMap> {
    let width = read_u16(read)?;
    let height = read_u16(read)?;
    let tile_count = width as usize * height as usize;

    let mut floors = Vec::new();
    let mut i = 0usize;
    while i < tile_count {
        let floor_id = read_i16(read)?;
        let ore_id = read_i16(read)?;
        let consecutives = read_u8(read)?;
        let len = consecutives as usize + 1;
        if i + len > tile_count {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "floor run exceeds map bounds",
            ));
        }
        floors.push(LegacyMapFloorRecord {
            index: i,
            floor_id,
            ore_id,
            consecutives,
        });
        i += len;
    }

    let mut blocks = Vec::new();
    let mut i = 0usize;
    while i < tile_count {
        let block_id = read_i16(read)?;
        let packed_flags = read_u8(read)?;
        let has_entity = (packed_flags & 1) != 0;
        let has_old_data = (packed_flags & 2) != 0;
        let has_new_data = (packed_flags & 4) != 0;

        let new_data = if has_new_data {
            Some(LegacyMapTileData {
                data: read_u8(read)?,
                floor_data: read_u8(read)?,
                overlay_data: read_u8(read)?,
                extra_data: read_i32(read)?,
            })
        } else {
            None
        };

        let is_center = if has_entity {
            read_u8(read)? != 0
        } else {
            true
        };

        let (building, consecutives) = if has_entity {
            let building = if is_center {
                Some(read_chunk(read)?)
            } else {
                None
            };
            (building, 0)
        } else if has_new_data {
            (None, 0)
        } else {
            let consecutives = read_u8(read)?;
            let len = consecutives as usize + 1;
            if i + len > tile_count {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "block run exceeds map bounds",
                ));
            }
            (None, consecutives)
        };

        blocks.push(LegacyMapBlockRecord {
            index: i,
            block_id,
            packed_flags,
            has_entity,
            has_old_data,
            has_new_data,
            is_center,
            new_data,
            old_data: None,
            building,
            consecutives,
        });

        i += consecutives as usize + 1;
    }

    Ok(LegacyShortChunkMap {
        width,
        height,
        floors,
        blocks,
    })
}

pub fn write_chunk_map<W: Write>(write: &mut W, map: &LegacyShortChunkMap) -> io::Result<()> {
    validate_run_cover(
        map.tile_count(),
        map.floors.iter().map(|record| (record.index, record.len())),
        "floor",
    )?;
    validate_run_cover(
        map.tile_count(),
        map.blocks.iter().map(|record| (record.index, record.len())),
        "block",
    )?;

    write_u16(write, map.width)?;
    write_u16(write, map.height)?;

    for record in &map.floors {
        write_i16(write, record.floor_id)?;
        write_i16(write, record.ore_id)?;
        write_u8(write, record.consecutives)?;
    }

    for record in &map.blocks {
        let has_entity = record.has_entity || record.building.is_some();
        let has_old_data = record.has_old_data || record.old_data.is_some();
        let has_new_data = record.has_new_data || record.new_data.is_some();

        if has_old_data {
            return Err(invalid_input(
                "legacy old tile data cannot be written as modern chunk map",
            ));
        }
        if has_new_data != record.new_data.is_some() {
            return Err(invalid_input("chunk map new data flag mismatch"));
        }
        if has_entity && record.is_center && record.building.is_none() {
            return Err(invalid_input("center entity record missing building chunk"));
        }
        if !has_entity && record.building.is_some() {
            return Err(invalid_input("building chunk without entity flag"));
        }
        if (has_entity || has_new_data) && record.consecutives != 0 {
            return Err(invalid_input(
                "entity or data block record cannot encode consecutives",
            ));
        }

        let packed_flags = (record.packed_flags & !0x07)
            | if has_entity { 1 } else { 0 }
            | if has_new_data { 4 } else { 0 };

        write_i16(write, record.block_id)?;
        write_u8(write, packed_flags)?;

        if let Some(data) = &record.new_data {
            write_u8(write, data.data)?;
            write_u8(write, data.floor_data)?;
            write_u8(write, data.overlay_data)?;
            write_i32(write, data.extra_data)?;
        }

        if has_entity {
            write_u8(write, record.is_center as u8)?;
            if record.is_center {
                let building = record
                    .building
                    .as_ref()
                    .ok_or_else(|| invalid_input("center entity record missing building chunk"))?;
                write_chunk(write, |payload| {
                    payload.write_all(building)?;
                    Ok(())
                })?;
            }
        } else if !has_new_data {
            write_u8(write, record.consecutives)?;
        }
    }

    Ok(())
}

fn validate_run_cover<I>(tile_count: usize, runs: I, label: &'static str) -> io::Result<()>
where
    I: IntoIterator<Item = (usize, usize)>,
{
    let mut expected = 0usize;
    for (index, len) in runs {
        if len == 0 {
            return Err(invalid_input(label));
        }
        if index != expected {
            return Err(invalid_input(label));
        }
        expected = expected
            .checked_add(len)
            .ok_or_else(|| invalid_input(label))?;
        if expected > tile_count {
            return Err(invalid_input(label));
        }
    }

    if expected != tile_count {
        return Err(invalid_input(label));
    }

    Ok(())
}

fn invalid_input(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_chunk_map_expands_floor_and_block_runs_into_tiles() {
        let map = LegacyShortChunkMap {
            width: 2,
            height: 1,
            floors: vec![LegacyMapFloorRecord {
                index: 0,
                floor_id: 2,
                ore_id: 3,
                consecutives: 1,
            }],
            blocks: vec![LegacyMapBlockRecord {
                index: 0,
                block_id: 5,
                packed_flags: 0,
                has_entity: false,
                has_old_data: false,
                has_new_data: false,
                is_center: true,
                new_data: None,
                old_data: None,
                building: None,
                consecutives: 1,
            }],
        };

        let tiles = map.to_tiles();

        assert_eq!(tiles.width, 2);
        assert_eq!(tiles.height, 1);
        for x in 0..2 {
            let tile = tiles.get(x, 0).unwrap();
            assert_eq!(tile.floor, 2);
            assert_eq!(tile.overlay, 3);
            assert_eq!(tile.block, 5);
        }
    }

    #[test]
    fn legacy_chunk_map_applies_new_tile_data_to_record_start() {
        let map = LegacyShortChunkMap {
            width: 1,
            height: 1,
            floors: vec![LegacyMapFloorRecord {
                index: 0,
                floor_id: 7,
                ore_id: 8,
                consecutives: 0,
            }],
            blocks: vec![LegacyMapBlockRecord {
                index: 0,
                block_id: 9,
                packed_flags: 4,
                has_entity: false,
                has_old_data: false,
                has_new_data: true,
                is_center: true,
                new_data: Some(LegacyMapTileData {
                    data: 1,
                    floor_data: 2,
                    overlay_data: 3,
                    extra_data: 4,
                }),
                old_data: None,
                building: None,
                consecutives: 0,
            }],
        };

        let tile = map.to_tiles().get(0, 0).cloned().unwrap();

        assert_eq!(tile.floor, 7);
        assert_eq!(tile.overlay, 8);
        assert_eq!(tile.block, 9);
        assert_eq!(tile.data, 1);
        assert_eq!(tile.floor_data, 2);
        assert_eq!(tile.overlay_data, 3);
        assert_eq!(tile.extra_data, 4);
    }

    #[test]
    fn chunk_map_writer_roundtrips_modern_runs_data_and_buildings() {
        let map = LegacyShortChunkMap {
            width: 3,
            height: 1,
            floors: vec![LegacyMapFloorRecord {
                index: 0,
                floor_id: 2,
                ore_id: 0,
                consecutives: 2,
            }],
            blocks: vec![
                LegacyMapBlockRecord {
                    index: 0,
                    block_id: 5,
                    packed_flags: 0,
                    has_entity: false,
                    has_old_data: false,
                    has_new_data: false,
                    is_center: true,
                    new_data: None,
                    old_data: None,
                    building: None,
                    consecutives: 0,
                },
                LegacyMapBlockRecord {
                    index: 1,
                    block_id: 6,
                    packed_flags: 4,
                    has_entity: false,
                    has_old_data: false,
                    has_new_data: true,
                    is_center: true,
                    new_data: Some(LegacyMapTileData {
                        data: 1,
                        floor_data: 2,
                        overlay_data: 3,
                        extra_data: 4,
                    }),
                    old_data: None,
                    building: None,
                    consecutives: 0,
                },
                LegacyMapBlockRecord {
                    index: 2,
                    block_id: 7,
                    packed_flags: 1,
                    has_entity: true,
                    has_old_data: false,
                    has_new_data: false,
                    is_center: true,
                    new_data: None,
                    old_data: None,
                    building: Some(vec![9, 8]),
                    consecutives: 0,
                },
            ],
        };

        let mut bytes = Vec::new();
        write_chunk_map(&mut bytes, &map).unwrap();

        assert_eq!(read_chunk_map(&mut bytes.as_slice()).unwrap(), map);
    }

    #[test]
    fn chunk_map_writer_rejects_legacy_old_tile_data() {
        let map = LegacyShortChunkMap {
            width: 1,
            height: 1,
            floors: vec![LegacyMapFloorRecord {
                index: 0,
                floor_id: 2,
                ore_id: 0,
                consecutives: 0,
            }],
            blocks: vec![LegacyMapBlockRecord {
                index: 0,
                block_id: 5,
                packed_flags: 2,
                has_entity: false,
                has_old_data: true,
                has_new_data: false,
                is_center: true,
                new_data: None,
                old_data: Some(7),
                building: None,
                consecutives: 0,
            }],
        };

        assert_eq!(
            write_chunk_map(&mut Vec::new(), &map).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn legacy_team_blocks_writer_preserves_plan_count_before_reader_dedupes() {
        let blocks = LegacyTeamBlocks {
            groups: vec![LegacyTeamBlockGroup {
                team_id: 1,
                plans: vec![
                    LegacyTeamBlockPlan {
                        x: 2,
                        y: 3,
                        rotation: 1,
                        block_id: 4,
                        config: TypeValue::Null,
                    },
                    LegacyTeamBlockPlan {
                        x: 2,
                        y: 3,
                        rotation: 2,
                        block_id: 5,
                        config: TypeValue::Int(7),
                    },
                ],
            }],
        };

        let mut bytes = Vec::new();
        write_legacy_team_blocks(&mut bytes, &blocks).unwrap();

        let mut cursor = bytes.as_slice();
        assert_eq!(read_i32(&mut cursor).unwrap(), 1);
        assert_eq!(read_i32(&mut cursor).unwrap(), 1);
        assert_eq!(read_i32(&mut cursor).unwrap(), 2);

        let decoded = read_legacy_team_blocks(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded.total_plans(), 1);
        assert_eq!(decoded.groups[0].plans[0], blocks.groups[0].plans[0]);
    }
}
