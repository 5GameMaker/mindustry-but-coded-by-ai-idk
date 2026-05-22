//! Save-version adapters mirroring upstream `mindustry.io.versions`.

use std::io::{self, Read};

use crate::mindustry::io::{
    save::read_legacy_short_chunk,
    type_io::{read_i32, read_u8},
};

pub mod save1;

pub use save1::Save1;

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
