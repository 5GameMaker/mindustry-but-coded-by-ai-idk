//! Mirrors upstream `mindustry.io.versions.Save11`.

use std::io::{self, Read};

use crate::mindustry::io::save::{read_content_patches, ContentPatchSet};

use super::{
    read_chunk_map, read_legacy_entity_mapping, read_legacy_team_blocks,
    read_legacy_world_entities, LegacyEntityMapping, LegacyShortChunkMap, LegacyTeamBlocks,
    LegacyWorldEntities,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Save11Entities {
    pub entity_mapping: LegacyEntityMapping,
    pub team_blocks: LegacyTeamBlocks,
    pub world_entities: LegacyWorldEntities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Save11;

impl Save11 {
    pub const VERSION: i32 = 11;

    pub const fn version(self) -> i32 {
        Self::VERSION
    }

    /// Java `Save11` extends `SaveVersion`; entity/map wire formats match
    /// `Save10`, while the outer save now includes a patches region.
    pub fn read_entities<R: Read>(self, read: &mut R) -> io::Result<Save11Entities> {
        let entity_mapping = read_legacy_entity_mapping(read)?;
        let team_blocks = read_legacy_team_blocks(read)?;
        let world_entities = read_legacy_world_entities(read)?;
        Ok(Save11Entities {
            entity_mapping,
            team_blocks,
            world_entities,
        })
    }

    pub fn read_map<R: Read>(self, read: &mut R) -> io::Result<LegacyShortChunkMap> {
        read_chunk_map(read)
    }

    pub fn read_patches<R: Read>(self, read: &mut R) -> io::Result<ContentPatchSet> {
        read_content_patches(read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::io::{
        save::{write_chunk, write_content_patches, ContentPatchSet, SaveRegion},
        type_io::{write_i16, write_i32, write_java_utf},
    };

    #[test]
    fn save11_reads_int_chunk_world_entities_with_ids() {
        let mut bytes = Vec::new();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 11).unwrap();
        write_java_utf(&mut bytes, "patched").unwrap();

        write_i32(&mut bytes, 0).unwrap();
        write_i32(&mut bytes, 1).unwrap();
        write_chunk(&mut bytes, |chunk| {
            chunk.push(11);
            write_i32(chunk, 111).unwrap();
            chunk.push(42);
            Ok(())
        })
        .unwrap();

        let entities = Save11.read_entities(&mut bytes.as_slice()).unwrap();

        assert_eq!(Save11.version(), 11);
        assert_eq!(entities.world_entities.chunks[0].entity_id, Some(111));
        assert_eq!(entities.world_entities.chunks[0].body, vec![42]);
    }

    #[test]
    fn save11_reads_content_patches_region_payload() {
        let patches = ContentPatchSet {
            patches: vec![b"one".to_vec(), b"two".to_vec()],
        };
        let mut bytes = Vec::new();
        write_content_patches(&mut bytes, &patches).unwrap();

        assert_eq!(Save11.read_patches(&mut bytes.as_slice()).unwrap(), patches);
    }

    #[test]
    fn save11_manifest_places_patches_before_map() {
        let manifest = SaveRegion::manifest_for_version(Save11::VERSION);
        assert_eq!(manifest, SaveRegion::manifest());
        assert_eq!(
            manifest,
            &[
                SaveRegion::Meta,
                SaveRegion::Content,
                SaveRegion::Patches,
                SaveRegion::Map,
                SaveRegion::Entities,
                SaveRegion::Markers,
                SaveRegion::Custom,
            ]
        );

        let patches = manifest
            .iter()
            .position(|region| *region == SaveRegion::Patches)
            .unwrap();
        let map = manifest
            .iter()
            .position(|region| *region == SaveRegion::Map)
            .unwrap();

        assert!(patches < map);
    }
}
