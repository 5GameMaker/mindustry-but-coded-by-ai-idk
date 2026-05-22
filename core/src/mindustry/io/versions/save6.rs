//! Mirrors upstream `mindustry.io.versions.Save6`.

use std::io::{self, Read};

use super::{
    read_legacy_entity_mapping, read_legacy_short_chunk_map, read_legacy_short_world_entities,
    read_legacy_team_blocks, LegacyEntityMapping, LegacyShortChunkMap, LegacyTeamBlocks,
    LegacyWorldEntities,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Save6Entities {
    pub entity_mapping: LegacyEntityMapping,
    pub team_blocks: LegacyTeamBlocks,
    pub world_entities: LegacyWorldEntities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Save6;

impl Save6 {
    pub const VERSION: i32 = 6;

    pub const fn version(self) -> i32 {
        Self::VERSION
    }

    /// Java `Save6` extends `LegacyRegionSaveVersion`, which keeps the old
    /// region list but uses `ShortChunkSaveVersion` entity/map readers.
    pub fn read_entities<R: Read>(self, read: &mut R) -> io::Result<Save6Entities> {
        let entity_mapping = read_legacy_entity_mapping(read)?;
        let team_blocks = read_legacy_team_blocks(read)?;
        let world_entities = read_legacy_short_world_entities(read)?;
        Ok(Save6Entities {
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
        type_io::{write_i16, write_i32, write_java_utf, write_u16, write_u8},
    };

    #[test]
    fn save6_reads_short_world_entities_with_ids() {
        let mut bytes = Vec::new();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 2).unwrap();
        write_java_utf(&mut bytes, "unit").unwrap();

        write_i32(&mut bytes, 0).unwrap(); // team block groups

        write_i32(&mut bytes, 1).unwrap(); // entities
        let mut entity = Vec::new();
        entity.push(2);
        write_i32(&mut entity, 345).unwrap();
        entity.extend_from_slice(&[6, 7]);
        write_legacy_short_chunk(&mut bytes, &entity).unwrap();

        let entities = Save6.read_entities(&mut bytes.as_slice()).unwrap();

        assert_eq!(Save6.version(), 6);
        assert_eq!(entities.entity_mapping.entries[0].name, "unit");
        assert_eq!(entities.team_blocks.total_plans(), 0);
        assert_eq!(entities.world_entities.chunks[0].type_id, 2);
        assert_eq!(entities.world_entities.chunks[0].entity_id, Some(345));
        assert_eq!(entities.world_entities.chunks[0].body, vec![6, 7]);
    }

    #[test]
    fn save6_reads_short_chunk_map_format() {
        let mut bytes = Vec::new();
        write_u16(&mut bytes, 1).unwrap();
        write_u16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 0).unwrap();
        write_u8(&mut bytes, 0).unwrap();
        write_i16(&mut bytes, 2).unwrap();
        write_u8(&mut bytes, 0).unwrap();
        write_u8(&mut bytes, 0).unwrap();

        let map = Save6.read_map(&mut bytes.as_slice()).unwrap();

        assert_eq!(map.tile_count(), 1);
        assert_eq!(map.blocks[0].block_id, 2);
    }
}
