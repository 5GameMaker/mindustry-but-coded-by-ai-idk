//! Mirrors upstream `mindustry.io.versions.Save3`.

use std::io::{self, Read};

use super::{
    read_legacy_entity_groups, read_legacy_int_config_team_blocks, LegacyEntityGroups,
    LegacyTeamBlocks,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Save3Entities {
    pub team_blocks: LegacyTeamBlocks,
    pub legacy_entities: LegacyEntityGroups,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Save3;

impl Save3 {
    pub const VERSION: i32 = 3;

    pub const fn version(self) -> i32 {
        Self::VERSION
    }

    /// Java `Save3.readEntities(...)` first reads team build plans whose
    /// config is a raw int, then skips legacy entity groups.
    pub fn read_entities<R: Read>(self, read: &mut R) -> io::Result<Save3Entities> {
        let team_blocks = read_legacy_int_config_team_blocks(read)?;
        let legacy_entities = read_legacy_entity_groups(read)?;
        Ok(Save3Entities {
            team_blocks,
            legacy_entities,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::io::{
        save::write_legacy_short_chunk,
        type_io::{write_i16, write_i32, write_u8, TypeValue},
    };

    #[test]
    fn save3_reads_int_config_team_blocks_then_legacy_entities() {
        let mut bytes = Vec::new();
        write_i32(&mut bytes, 1).unwrap(); // team count
        write_i32(&mut bytes, 2).unwrap(); // team id
        write_i32(&mut bytes, 2).unwrap(); // plans

        write_i16(&mut bytes, 3).unwrap();
        write_i16(&mut bytes, 4).unwrap();
        write_i16(&mut bytes, 1).unwrap();
        write_i16(&mut bytes, 8).unwrap();
        write_i32(&mut bytes, 99).unwrap();

        // Save3 does not de-duplicate positions here.
        write_i16(&mut bytes, 3).unwrap();
        write_i16(&mut bytes, 4).unwrap();
        write_i16(&mut bytes, 2).unwrap();
        write_i16(&mut bytes, 9).unwrap();
        write_i32(&mut bytes, -7).unwrap();

        write_u8(&mut bytes, 1).unwrap(); // legacy entity groups
        write_i32(&mut bytes, 1).unwrap();
        write_legacy_short_chunk(&mut bytes, &[5, 6]).unwrap();

        let entities = Save3.read_entities(&mut bytes.as_slice()).unwrap();

        assert_eq!(Save3.version(), 3);
        assert_eq!(entities.team_blocks.groups[0].team_id, 2);
        assert_eq!(entities.team_blocks.total_plans(), 2);
        assert_eq!(
            entities.team_blocks.groups[0].plans[0].config,
            TypeValue::Int(99)
        );
        assert_eq!(
            entities.team_blocks.groups[0].plans[1].config,
            TypeValue::Int(-7)
        );
        assert_eq!(entities.legacy_entities.total_entities(), 1);
        assert_eq!(
            entities.legacy_entities.groups[0].chunks[0].bytes,
            vec![5, 6]
        );
    }

    #[test]
    fn save3_rejects_negative_team_plan_counts() {
        let mut bytes = Vec::new();
        write_i32(&mut bytes, 1).unwrap();
        write_i32(&mut bytes, 2).unwrap();
        write_i32(&mut bytes, -1).unwrap();

        let err = Save3.read_entities(&mut bytes.as_slice()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
