//! Mirrors upstream `mindustry.io.versions.Save5`.

use std::io::{self, Read};

use super::{
    read_legacy_entity_mapping, read_legacy_short_chunk_map,
    read_legacy_short_world_entities_without_ids, read_legacy_team_blocks, LegacyEntityMapping,
    LegacyShortChunkMap, LegacyTeamBlocks, LegacyWorldEntities,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Save5Entities {
    pub entity_mapping: LegacyEntityMapping,
    pub team_blocks: LegacyTeamBlocks,
    pub world_entities: LegacyWorldEntities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Save5;

impl Save5 {
    pub const VERSION: i32 = 5;

    pub const fn version(self) -> i32 {
        Self::VERSION
    }

    /// Java `Save5` extends `LegacySaveVersion2`: it inherits
    /// `SaveVersion.readEntities(...)` but its `readWorldEntities(...)`
    /// override uses legacy short chunks with no entity IDs.
    pub fn read_entities<R: Read>(self, read: &mut R) -> io::Result<Save5Entities> {
        let entity_mapping = read_legacy_entity_mapping(read)?;
        let team_blocks = read_legacy_team_blocks(read)?;
        let world_entities = read_legacy_short_world_entities_without_ids(read)?;
        Ok(Save5Entities {
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
        type_io::{
            write_i16, write_i32, write_java_utf, write_object, write_u16, write_u8, TypeValue,
        },
    };

    #[test]
    fn save5_reads_mapping_team_blocks_and_no_id_short_entities() {
        let mut bytes = Vec::new();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 4).unwrap();
        write_java_utf(&mut bytes, "legacy-unit").unwrap();

        write_i32(&mut bytes, 1).unwrap();
        write_i32(&mut bytes, 6).unwrap();
        write_i32(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 7).unwrap();
        write_i16(&mut bytes, 8).unwrap();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 9).unwrap();
        write_object(&mut bytes, &TypeValue::String("cfg".into())).unwrap();

        write_i32(&mut bytes, 1).unwrap();
        write_legacy_short_chunk(&mut bytes, &[4, 10, 11]).unwrap();

        let entities = Save5.read_entities(&mut bytes.as_slice()).unwrap();

        assert_eq!(Save5.version(), 5);
        assert_eq!(entities.entity_mapping.entries[0].id, 4);
        assert_eq!(entities.team_blocks.total_plans(), 1);
        assert_eq!(entities.world_entities.chunks[0].type_id, 4);
        assert_eq!(entities.world_entities.chunks[0].entity_id, None);
        assert_eq!(entities.world_entities.chunks[0].body, vec![10, 11]);
    }

    #[test]
    fn save5_reads_short_chunk_map_format() {
        let mut bytes = Vec::new();
        write_u16(&mut bytes, 1).unwrap();
        write_u16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 0).unwrap();
        write_u8(&mut bytes, 0).unwrap();
        write_i16(&mut bytes, 2).unwrap();
        write_u8(&mut bytes, 0).unwrap();
        write_u8(&mut bytes, 0).unwrap();

        let map = Save5.read_map(&mut bytes.as_slice()).unwrap();

        assert_eq!(map.tile_count(), 1);
        assert_eq!(map.blocks[0].block_id, 2);
    }
}
