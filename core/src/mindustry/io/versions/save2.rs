//! Mirrors upstream `mindustry.io.versions.Save2`.

use std::io::{self, Read};

use super::{read_legacy_entity_groups, LegacyEntityGroups};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Save2;

impl Save2 {
    pub const VERSION: i32 = 2;

    pub const fn version(self) -> i32 {
        Self::VERSION
    }

    /// Java `Save2.readEntities(...)` still delegates to
    /// `LegacySaveVersion.readLegacyEntities(...)`; the wire layout is
    /// unchanged from `Save1`.
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
    fn save2_reads_legacy_entity_groups_like_save1() {
        let mut bytes = Vec::new();
        write_u8(&mut bytes, 1).unwrap();
        write_i32(&mut bytes, 2).unwrap();
        write_legacy_short_chunk(&mut bytes, &[9, 8]).unwrap();
        write_legacy_short_chunk(&mut bytes, &[7]).unwrap();

        let groups = Save2.read_entities(&mut bytes.as_slice()).unwrap();

        assert_eq!(Save2.version(), 2);
        assert_eq!(groups.total_entities(), 2);
        assert_eq!(groups.groups[0].chunks[0].bytes, vec![9, 8]);
        assert_eq!(groups.groups[0].chunks[1].entity_index, 1);
        assert_eq!(groups.groups[0].chunks[1].bytes, vec![7]);
    }

    #[test]
    fn save2_rejects_negative_legacy_entity_counts() {
        let mut bytes = Vec::new();
        write_u8(&mut bytes, 1).unwrap();
        write_i32(&mut bytes, -2).unwrap();

        let err = Save2.read_entities(&mut bytes.as_slice()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
