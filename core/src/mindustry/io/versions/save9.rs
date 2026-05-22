//! Mirrors upstream `mindustry.io.versions.Save9`.

use std::io::{self, Read};

use super::{
    read_legacy_entity_mapping, read_legacy_short_chunk_map, read_legacy_short_world_entities,
    read_legacy_team_blocks, LegacyEntityMapping, LegacyShortChunkMap, LegacyTeamBlocks,
    LegacyWorldEntities,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Save9Entities {
    pub entity_mapping: LegacyEntityMapping,
    pub team_blocks: LegacyTeamBlocks,
    pub world_entities: LegacyWorldEntities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Save9;

impl Save9 {
    pub const VERSION: i32 = 9;

    pub const fn version(self) -> i32 {
        Self::VERSION
    }

    /// Java `Save9` extends `ShortChunkSaveVersion`. It inherits
    /// `SaveVersion.readEntities(...)`, but its world entity chunks use the
    /// legacy unsigned-short length prefix instead of the later int chunk.
    pub fn read_entities<R: Read>(self, read: &mut R) -> io::Result<Save9Entities> {
        let entity_mapping = read_legacy_entity_mapping(read)?;
        let team_blocks = read_legacy_team_blocks(read)?;
        let world_entities = read_legacy_short_world_entities(read)?;
        Ok(Save9Entities {
            entity_mapping,
            team_blocks,
            world_entities,
        })
    }

    /// Reads the short-chunk tile map format implemented by Java
    /// `ShortChunkSaveVersion.readMap(...)`.
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
    fn save9_reads_entity_mapping_team_blocks_and_short_world_entities() {
        let mut bytes = Vec::new();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 12).unwrap();
        write_java_utf(&mut bytes, "crawler").unwrap();

        write_i32(&mut bytes, 1).unwrap(); // team groups
        write_i32(&mut bytes, 1).unwrap(); // team id
        write_i32(&mut bytes, 1).unwrap(); // plans
        write_i16(&mut bytes, 4).unwrap();
        write_i16(&mut bytes, 5).unwrap();
        write_i16(&mut bytes, 2).unwrap();
        write_i16(&mut bytes, 33).unwrap();
        write_object(&mut bytes, &TypeValue::Null).unwrap();

        write_i32(&mut bytes, 1).unwrap(); // world entities
        let mut entity = Vec::new();
        entity.push(12);
        write_i32(&mut entity, 77).unwrap();
        entity.extend_from_slice(&[8, 9]);
        write_legacy_short_chunk(&mut bytes, &entity).unwrap();

        let entities = Save9.read_entities(&mut bytes.as_slice()).unwrap();

        assert_eq!(Save9.version(), 9);
        assert_eq!(entities.entity_mapping.entries[0].id, 12);
        assert_eq!(entities.entity_mapping.entries[0].name, "crawler");
        assert_eq!(entities.team_blocks.total_plans(), 1);
        assert_eq!(entities.world_entities.len(), 1);
        assert_eq!(entities.world_entities.chunks[0].type_id, 12);
        assert_eq!(entities.world_entities.chunks[0].entity_id, Some(77));
        assert_eq!(entities.world_entities.chunks[0].body, vec![8, 9]);
    }

    #[test]
    fn save9_reads_short_chunk_map_with_old_new_data_and_buildings() {
        let mut bytes = Vec::new();
        write_u16(&mut bytes, 3).unwrap();
        write_u16(&mut bytes, 1).unwrap();

        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 2).unwrap();
        write_u8(&mut bytes, 2).unwrap(); // one floor run covers 3 tiles

        write_i16(&mut bytes, 10).unwrap();
        write_u8(&mut bytes, 4).unwrap(); // new 7-byte tile data
        write_u8(&mut bytes, 3).unwrap();
        write_u8(&mut bytes, 4).unwrap();
        write_u8(&mut bytes, 5).unwrap();
        write_i32(&mut bytes, 0x0102_0304).unwrap();

        write_i16(&mut bytes, 11).unwrap();
        write_u8(&mut bytes, 2).unwrap(); // old 1-byte tile data
        write_u8(&mut bytes, 55).unwrap();

        write_i16(&mut bytes, 12).unwrap();
        write_u8(&mut bytes, 1).unwrap(); // building entity
        write_u8(&mut bytes, 1).unwrap(); // center
        write_legacy_short_chunk(&mut bytes, &[7, 8]).unwrap();

        let map = Save9.read_map(&mut bytes.as_slice()).unwrap();

        assert_eq!(map.tile_count(), 3);
        assert_eq!(map.floors.len(), 1);
        assert_eq!(map.floors[0].len(), 3);
        assert_eq!(map.floors[0].floor_id, 1);
        assert_eq!(map.floors[0].ore_id, 2);

        assert_eq!(map.blocks.len(), 3);
        assert_eq!(map.blocks[0].new_data.as_ref().unwrap().data, 3);
        assert_eq!(map.blocks[0].new_data.as_ref().unwrap().floor_data, 4);
        assert_eq!(map.blocks[0].new_data.as_ref().unwrap().overlay_data, 5);
        assert_eq!(
            map.blocks[0].new_data.as_ref().unwrap().extra_data,
            0x0102_0304
        );
        assert_eq!(map.blocks[1].old_data, Some(55));
        assert_eq!(map.blocks[2].building, Some(vec![7, 8]));
    }

    #[test]
    fn save9_rejects_runs_that_exceed_map_bounds() {
        let mut bytes = Vec::new();
        write_u16(&mut bytes, 1).unwrap();
        write_u16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 0).unwrap();
        write_u8(&mut bytes, 1).unwrap();

        let err = Save9.read_map(&mut bytes.as_slice()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
