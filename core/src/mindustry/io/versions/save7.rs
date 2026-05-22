//! Mirrors upstream `mindustry.io.versions.Save7`.

use std::io::{self, Read};

use super::{
    read_legacy_entity_mapping, read_legacy_short_chunk_map, read_legacy_short_world_entities,
    read_legacy_team_blocks, LegacyEntityMapping, LegacyShortChunkMap, LegacyTeamBlocks,
    LegacyWorldEntities,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Save7Entities {
    pub entity_mapping: LegacyEntityMapping,
    pub team_blocks: LegacyTeamBlocks,
    pub world_entities: LegacyWorldEntities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Save7;

impl Save7 {
    pub const VERSION: i32 = 7;

    pub const fn version(self) -> i32 {
        Self::VERSION
    }

    /// Java `Save7` extends `ShortChunkSaveVersion`; world entities have
    /// per-save mapping, team blocks, and legacy short chunks with IDs.
    pub fn read_entities<R: Read>(self, read: &mut R) -> io::Result<Save7Entities> {
        let entity_mapping = read_legacy_entity_mapping(read)?;
        let team_blocks = read_legacy_team_blocks(read)?;
        let world_entities = read_legacy_short_world_entities(read)?;
        Ok(Save7Entities {
            entity_mapping,
            team_blocks,
            world_entities,
        })
    }

    pub fn read_map<R: Read>(self, read: &mut R) -> io::Result<LegacyShortChunkMap> {
        read_legacy_short_chunk_map(read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::io::{
        save::write_legacy_short_chunk,
        type_io::{write_i16, write_i32, write_java_utf},
    };

    #[test]
    fn save7_reads_short_world_entities_with_ids() {
        let mut bytes = Vec::new();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 7).unwrap();
        write_java_utf(&mut bytes, "player").unwrap();

        write_i32(&mut bytes, 0).unwrap(); // team block groups
        write_i32(&mut bytes, 1).unwrap(); // entities

        let mut entity = Vec::new();
        entity.push(7);
        write_i32(&mut entity, 70).unwrap();
        entity.push(1);
        write_legacy_short_chunk(&mut bytes, &entity).unwrap();

        let entities = Save7.read_entities(&mut bytes.as_slice()).unwrap();

        assert_eq!(Save7.version(), 7);
        assert_eq!(entities.entity_mapping.entries[0].id, 7);
        assert_eq!(entities.world_entities.chunks[0].entity_id, Some(70));
        assert_eq!(entities.world_entities.chunks[0].body, vec![1]);
    }
}
