//! Adapter for upstream `mindustry.io.versions.ShortChunkSaveVersion`.

use std::io::{self, Read};

use super::{
    read_legacy_short_chunk_map, read_legacy_short_world_entities, LegacyShortChunkMap,
    LegacyWorldEntities,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortChunkSaveVersion {
    version: i32,
}

impl ShortChunkSaveVersion {
    pub const fn new(version: i32) -> Self {
        Self { version }
    }

    pub const fn version(self) -> i32 {
        self.version
    }

    /// Java `readWorldEntities(...)` for the short-chunk save lineage:
    /// an entity count followed by legacy short chunks whose first byte is
    /// the type id and whose next four bytes are the entity id.
    pub fn read_world_entities<R: Read>(self, read: &mut R) -> io::Result<LegacyWorldEntities> {
        read_legacy_short_world_entities(read)
    }

    /// Java `readMap(...)` for short-chunk versions.  The returned
    /// `LegacyShortChunkMap` is a lightweight Rust mirror of the world-context
    /// mutations Java performs while reading floors, blocks, tile data and
    /// optional building chunks.
    pub fn read_map<R: Read>(self, read: &mut R) -> io::Result<LegacyShortChunkMap> {
        read_legacy_short_chunk_map(read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::io::{
        save::write_legacy_short_chunk,
        type_io::{write_i16, write_i32, write_u16, write_u8},
    };

    #[test]
    fn short_chunk_save_version_reads_world_entities_with_ids() {
        let version = ShortChunkSaveVersion::new(7);
        let mut bytes = Vec::new();
        write_i32(&mut bytes, 2).unwrap();
        write_legacy_short_chunk(&mut bytes, &[4, 0, 0, 0, 42, 9, 8]).unwrap();
        write_legacy_short_chunk(&mut bytes, &[5, 0, 0, 0, 43]).unwrap();

        let entities = version.read_world_entities(&mut bytes.as_slice()).unwrap();

        assert_eq!(version.version(), 7);
        assert_eq!(entities.len(), 2);
        assert_eq!(entities.chunks[0].type_id, 4);
        assert_eq!(entities.chunks[0].entity_id, Some(42));
        assert_eq!(entities.chunks[0].body, vec![9, 8]);
        assert_eq!(entities.chunks[1].type_id, 5);
        assert_eq!(entities.chunks[1].entity_id, Some(43));
    }

    #[test]
    fn short_chunk_save_version_reads_map_via_legacy_short_chunk_map() {
        let version = ShortChunkSaveVersion::new(8);
        let mut bytes = Vec::new();
        write_u16(&mut bytes, 1).unwrap();
        write_u16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 2).unwrap(); // floor
        write_i16(&mut bytes, 3).unwrap(); // ore
        write_u8(&mut bytes, 0).unwrap(); // floor run
        write_i16(&mut bytes, 4).unwrap(); // block
        write_u8(&mut bytes, 0).unwrap(); // no entity/data
        write_u8(&mut bytes, 0).unwrap(); // block run

        let map = version.read_map(&mut bytes.as_slice()).unwrap();

        assert_eq!(map.width, 1);
        assert_eq!(map.height, 1);
        assert_eq!(map.floors[0].floor_id, 2);
        assert_eq!(map.floors[0].ore_id, 3);
        assert_eq!(map.blocks[0].block_id, 4);
    }
}
