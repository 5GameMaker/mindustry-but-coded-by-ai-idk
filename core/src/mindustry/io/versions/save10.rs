//! Mirrors upstream `mindustry.io.versions.Save10`.

use std::io::{self, Read};

use super::{
    read_chunk_map, read_legacy_entity_mapping, read_legacy_team_blocks,
    read_legacy_world_entities, LegacyEntityMapping, LegacyShortChunkMap, LegacyTeamBlocks,
    LegacyWorldEntities,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Save10Entities {
    pub entity_mapping: LegacyEntityMapping,
    pub team_blocks: LegacyTeamBlocks,
    pub world_entities: LegacyWorldEntities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Save10;

impl Save10 {
    pub const VERSION: i32 = 10;

    pub const fn version(self) -> i32 {
        Self::VERSION
    }

    /// Java `Save10` extends `SaveVersion`: world entity chunks use the normal
    /// 4-byte length prefix and include entity IDs.
    pub fn read_entities<R: Read>(self, read: &mut R) -> io::Result<Save10Entities> {
        let entity_mapping = read_legacy_entity_mapping(read)?;
        let team_blocks = read_legacy_team_blocks(read)?;
        let world_entities = read_legacy_world_entities(read)?;
        Ok(Save10Entities {
            entity_mapping,
            team_blocks,
            world_entities,
        })
    }

    pub fn read_map<R: Read>(self, read: &mut R) -> io::Result<LegacyShortChunkMap> {
        read_chunk_map(read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::io::{
        save::{write_chunk, SaveRegion},
        type_io::{write_i16, write_i32, write_java_utf, write_u16, write_u8},
    };

    #[test]
    fn save10_reads_int_chunk_world_entities_with_ids() {
        let mut bytes = Vec::new();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 10).unwrap();
        write_java_utf(&mut bytes, "modern").unwrap();

        write_i32(&mut bytes, 0).unwrap(); // team block groups
        write_i32(&mut bytes, 1).unwrap(); // entities

        write_chunk(&mut bytes, |chunk| {
            chunk.push(10);
            write_i32(chunk, 1010)?;
            chunk.extend_from_slice(&[1, 2]);
            Ok(())
        })
        .unwrap();

        let entities = Save10.read_entities(&mut bytes.as_slice()).unwrap();

        assert_eq!(Save10.version(), 10);
        assert_eq!(entities.entity_mapping.entries[0].id, 10);
        assert_eq!(entities.world_entities.chunks[0].entity_id, Some(1010));
        assert_eq!(entities.world_entities.chunks[0].body, vec![1, 2]);
    }

    #[test]
    fn save10_reads_map_building_with_int_chunk() {
        let mut bytes = Vec::new();
        write_u16(&mut bytes, 1).unwrap();
        write_u16(&mut bytes, 1).unwrap();

        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 0).unwrap();
        write_u8(&mut bytes, 0).unwrap();

        write_i16(&mut bytes, 12).unwrap();
        write_u8(&mut bytes, 1).unwrap(); // had entity
        write_u8(&mut bytes, 1).unwrap(); // center
        write_chunk(&mut bytes, |chunk| {
            chunk.extend_from_slice(&[9, 8, 7]);
            Ok(())
        })
        .unwrap();

        let map = Save10.read_map(&mut bytes.as_slice()).unwrap();

        assert_eq!(map.tile_count(), 1);
        assert_eq!(map.blocks[0].block_id, 12);
        assert_eq!(map.blocks[0].building, Some(vec![9, 8, 7]));
    }

    #[test]
    fn save10_manifest_has_markers_custom_but_no_patches() {
        let manifest = SaveRegion::manifest_for_version(Save10::VERSION);

        assert!(manifest.contains(&SaveRegion::Markers));
        assert!(manifest.contains(&SaveRegion::Custom));
        assert!(!manifest.contains(&SaveRegion::Patches));
    }
}
