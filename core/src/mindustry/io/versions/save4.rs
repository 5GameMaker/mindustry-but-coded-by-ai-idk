//! Mirrors upstream `mindustry.io.versions.Save4`.

use std::io::{self, Read};

use super::{
    read_legacy_short_chunk_map, read_legacy_short_world_entities_without_ids,
    read_legacy_team_blocks, LegacyShortChunkMap, LegacyTeamBlocks, LegacyWorldEntities,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Save4Entities {
    pub team_blocks: LegacyTeamBlocks,
    pub world_entities: LegacyWorldEntities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Save4;

impl Save4 {
    pub const VERSION: i32 = 4;

    pub const fn version(self) -> i32 {
        Self::VERSION
    }

    /// Java `Save4.readEntities(...)` has no custom entity-id mapping chunk
    /// and inherits `LegacySaveVersion2.readWorldEntities(...)`; world
    /// entities are legacy short chunks without per-entity IDs.
    pub fn read_entities<R: Read>(self, read: &mut R) -> io::Result<Save4Entities> {
        let team_blocks = read_legacy_team_blocks(read)?;
        let world_entities = read_legacy_short_world_entities_without_ids(read)?;
        Ok(Save4Entities {
            team_blocks,
            world_entities,
        })
    }

    /// Java `Save4` also inherits `ShortChunkSaveVersion.readMap(...)`.
    pub fn read_map<R: Read>(self, read: &mut R) -> io::Result<LegacyShortChunkMap> {
        read_legacy_short_chunk_map(read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::io::{
        save::write_legacy_short_chunk,
        type_io::{write_i16, write_i32, write_object, write_u16, write_u8, TypeValue},
    };

    #[test]
    fn save4_reads_team_blocks_and_world_entities() {
        let mut bytes = Vec::new();
        write_i32(&mut bytes, 1).unwrap(); // team count
        write_i32(&mut bytes, 3).unwrap(); // team id
        write_i32(&mut bytes, 3).unwrap(); // plan count

        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 2).unwrap();
        write_i16(&mut bytes, 0).unwrap();
        write_i16(&mut bytes, 7).unwrap();
        write_object(&mut bytes, &TypeValue::Null).unwrap();

        // Duplicate position is consumed but omitted, matching Java IntSet use.
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 2).unwrap();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 8).unwrap();
        write_object(&mut bytes, &TypeValue::String("ignored".into())).unwrap();

        write_i16(&mut bytes, -4).unwrap();
        write_i16(&mut bytes, 5).unwrap();
        write_i16(&mut bytes, 2).unwrap();
        write_i16(&mut bytes, 9).unwrap();
        write_object(&mut bytes, &TypeValue::Int(42)).unwrap();

        write_i32(&mut bytes, 2).unwrap(); // world entity count
        write_legacy_short_chunk(&mut bytes, &[6, 1, 2, 3]).unwrap();
        write_legacy_short_chunk(&mut bytes, &[9, 4, 5]).unwrap();

        let entities = Save4.read_entities(&mut bytes.as_slice()).unwrap();

        assert_eq!(Save4.version(), 4);
        assert_eq!(entities.team_blocks.total_plans(), 2);
        assert_eq!(entities.team_blocks.groups[0].team_id, 3);
        assert_eq!(entities.team_blocks.groups[0].plans[0].block_id, 7);
        assert_eq!(entities.team_blocks.groups[0].plans[1].x, -4);
        assert_eq!(
            entities.team_blocks.groups[0].plans[1].config,
            TypeValue::Int(42)
        );

        assert_eq!(entities.world_entities.len(), 2);
        assert_eq!(entities.world_entities.chunks[0].type_id, 6);
        assert_eq!(entities.world_entities.chunks[0].entity_id, None);
        assert_eq!(entities.world_entities.chunks[0].body, vec![1, 2, 3]);
        assert_eq!(entities.world_entities.chunks[1].type_id, 9);
        assert_eq!(entities.world_entities.chunks[1].entity_id, None);
        assert_eq!(entities.world_entities.chunks[1].body, vec![4, 5]);
    }

    #[test]
    fn save4_rejects_negative_counts() {
        let mut bytes = Vec::new();
        write_i32(&mut bytes, -1).unwrap();

        let err = Save4.read_entities(&mut bytes.as_slice()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn save4_reads_short_chunk_map_format() {
        let mut bytes = Vec::new();
        write_u16(&mut bytes, 2).unwrap();
        write_u16(&mut bytes, 1).unwrap();

        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 0).unwrap();
        write_u8(&mut bytes, 1).unwrap();

        write_i16(&mut bytes, 5).unwrap();
        write_u8(&mut bytes, 0).unwrap();
        write_u8(&mut bytes, 1).unwrap();

        let map = Save4.read_map(&mut bytes.as_slice()).unwrap();

        assert_eq!(map.width, 2);
        assert_eq!(map.height, 1);
        assert_eq!(map.floors[0].len(), 2);
        assert_eq!(map.blocks[0].block_id, 5);
        assert_eq!(map.blocks[0].len(), 2);
    }
}
