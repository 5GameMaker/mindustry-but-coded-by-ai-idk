//! Save-version adapters mirroring upstream `mindustry.io.versions`.

use std::{
    collections::HashSet,
    io::{self, Read},
};

use crate::mindustry::io::{
    save::{read_chunk, read_legacy_short_chunk},
    type_io::{read_i16, read_i32, read_object, read_u8, TypeValue},
};

pub mod save1;
pub mod save2;
pub mod save4;

pub use save1::Save1;
pub use save2::Save2;
pub use save4::Save4;

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
        let raw = read_chunk(read)?;
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

        chunks.push(LegacyWorldEntityChunk {
            type_id,
            entity_id,
            body,
            raw,
        });
    }

    Ok(LegacyWorldEntities { chunks })
}

pub fn skip_legacy_world_entities<R: Read>(read: &mut R) -> io::Result<usize> {
    read_legacy_world_entities(read).map(|entities| entities.len())
}
