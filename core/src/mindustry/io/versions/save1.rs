//! Mirrors upstream `mindustry.io.versions.Save1`.

use std::io::{self, Read};

use super::{read_legacy_entity_groups, LegacyEntityGroups};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Save1;

impl Save1 {
    pub const VERSION: i32 = 1;

    pub const fn version(self) -> i32 {
        Self::VERSION
    }

    /// Java `Save1.readEntities(...)` delegates directly to
    /// `LegacySaveVersion.readLegacyEntities(...)`, which reads legacy entity
    /// groups and skips each short chunk.
    pub fn read_entities<R: Read>(self, read: &mut R) -> io::Result<LegacyEntityGroups> {
        read_legacy_entity_groups(read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::io::{
        save::write_legacy_short_chunk,
        type_io::{write_i32, write_u8},
    };

    #[test]
    fn save1_reads_legacy_entity_groups_as_short_chunks() {
        let mut bytes = Vec::new();
        write_u8(&mut bytes, 2).unwrap();
        write_i32(&mut bytes, 2).unwrap();
        write_legacy_short_chunk(&mut bytes, &[1, 2, 3]).unwrap();
        write_legacy_short_chunk(&mut bytes, &[4]).unwrap();
        write_i32(&mut bytes, 1).unwrap();
        write_legacy_short_chunk(&mut bytes, &[5, 6]).unwrap();

        let groups = Save1.read_entities(&mut bytes.as_slice()).unwrap();

        assert_eq!(Save1.version(), 1);
        assert_eq!(groups.total_entities(), 3);
        assert_eq!(groups.groups[0].group_index, 0);
        assert_eq!(groups.groups[0].chunks[0].bytes, vec![1, 2, 3]);
        assert_eq!(groups.groups[0].chunks[1].entity_index, 1);
        assert_eq!(groups.groups[1].group_index, 1);
        assert_eq!(groups.groups[1].chunks[0].bytes, vec![5, 6]);
    }

    #[test]
    fn save1_rejects_negative_legacy_entity_counts() {
        let mut bytes = Vec::new();
        write_u8(&mut bytes, 1).unwrap();
        write_i32(&mut bytes, -1).unwrap();

        let err = Save1.read_entities(&mut bytes.as_slice()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
