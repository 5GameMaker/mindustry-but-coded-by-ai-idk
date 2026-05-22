//! Mirrors upstream `mindustry.io.versions.Save8`.

use std::io::{self, Read};

use super::{
    read_legacy_entity_mapping, read_legacy_short_chunk_map, read_legacy_short_world_entities,
    read_legacy_team_blocks, LegacyEntityMapping, LegacyShortChunkMap, LegacyTeamBlocks,
    LegacyWorldEntities,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Save8Entities {
    pub entity_mapping: LegacyEntityMapping,
    pub team_blocks: LegacyTeamBlocks,
    pub world_entities: LegacyWorldEntities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Save8;

impl Save8 {
    pub const VERSION: i32 = 8;

    pub const fn version(self) -> i32 {
        Self::VERSION
    }

    /// Java `Save8` only changes the outer region manifest by enabling marker
    /// binary data; entity/map decoding is still `ShortChunkSaveVersion`.
    pub fn read_entities<R: Read>(self, read: &mut R) -> io::Result<Save8Entities> {
        let entity_mapping = read_legacy_entity_mapping(read)?;
        let team_blocks = read_legacy_team_blocks(read)?;
        let world_entities = read_legacy_short_world_entities(read)?;
        Ok(Save8Entities {
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
        save::{write_legacy_short_chunk, SaveRegion},
        type_io::{write_i16, write_i32, write_java_utf},
    };

    #[test]
    fn save8_reads_short_world_entities_with_ids() {
        let mut bytes = Vec::new();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 8).unwrap();
        write_java_utf(&mut bytes, "marker-era").unwrap();

        write_i32(&mut bytes, 0).unwrap();
        write_i32(&mut bytes, 1).unwrap();

        let mut entity = Vec::new();
        entity.push(8);
        write_i32(&mut entity, 88).unwrap();
        entity.extend_from_slice(&[3, 4]);
        write_legacy_short_chunk(&mut bytes, &entity).unwrap();

        let entities = Save8.read_entities(&mut bytes.as_slice()).unwrap();

        assert_eq!(Save8.version(), 8);
        assert_eq!(entities.world_entities.chunks[0].entity_id, Some(88));
        assert_eq!(entities.world_entities.chunks[0].body, vec![3, 4]);
    }

    #[test]
    fn save8_manifest_includes_markers_but_not_patches() {
        let manifest = SaveRegion::manifest_for_version(Save8::VERSION);

        assert!(manifest.contains(&SaveRegion::Markers));
        assert!(manifest.contains(&SaveRegion::Custom));
        assert!(!manifest.contains(&SaveRegion::Patches));
    }
}
