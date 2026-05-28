use std::io::{self, Read, Write};

use crate::mindustry::world::{BlockId, Tile};

use super::{
    legacy_unit_factory_remove_self, read_legacy_command_center_extra, read_legacy_mech_pad_extra,
    read_legacy_unit_factory_extra, write_legacy_command_center_extra, write_legacy_mech_pad_extra,
    write_legacy_unit_factory_extra, LegacyBlock, LegacyRemoval, LegacyUnitFactoryExtra,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCommandCenter {
    pub block: LegacyBlock,
    pub write_zero_byte: bool,
}

impl LegacyCommandCenter {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = LegacyBlock::new(0, name);
        block.base.update = true;

        Self {
            block,
            write_zero_byte: true,
        }
    }

    pub fn write_extra<W: Write>(&self, write: &mut W) -> io::Result<()> {
        write_legacy_command_center_extra(write)
    }

    pub fn read_extra<R: Read>(&self, read: &mut R) -> io::Result<u8> {
        read_legacy_command_center_extra(read)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMechPad {
    pub block: LegacyBlock,
    pub discarded_floats: usize,
}

impl LegacyMechPad {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = LegacyBlock::new(0, name);
        block.base.update = true;
        block.base.has_power = true;

        Self {
            block,
            discarded_floats: 3,
        }
    }

    pub fn read_extra<R: Read>(&self, read: &mut R) -> io::Result<[f32; 3]> {
        read_legacy_mech_pad_extra(read)
    }

    pub fn write_extra<W: Write>(&self, write: &mut W, values: [f32; 3]) -> io::Result<()> {
        write_legacy_mech_pad_extra(write, values)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUnitFactory {
    pub block: LegacyBlock,
    pub replacement: BlockId,
    pub replacement_name: Option<String>,
}

impl LegacyUnitFactory {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = LegacyBlock::new(0, name);
        block.base.update = true;
        block.base.has_power = true;
        block.base.has_items = true;
        block.base.solid = false;

        Self {
            block,
            replacement: Tile::AIR,
            replacement_name: None,
        }
    }

    pub fn remove_self(&self, tile: &Tile) -> LegacyRemoval {
        let rotation = tile.build.map(|build| build.rotation);
        legacy_unit_factory_remove_self(self.replacement, rotation)
    }

    pub fn read_extra<R: Read>(
        &self,
        read: &mut R,
        revision: u8,
    ) -> io::Result<LegacyUnitFactoryExtra> {
        read_legacy_unit_factory_extra(read, revision)
    }

    pub fn write_extra<W: Write>(
        &self,
        write: &mut W,
        revision: u8,
        extra: &LegacyUnitFactoryExtra,
    ) -> io::Result<()> {
        write_legacy_unit_factory_extra(write, revision, extra)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_shell_defaults_follow_java_constructors() {
        let center = LegacyCommandCenter::new("command-center");
        assert!(center.block.base.update);
        assert!(center.write_zero_byte);

        let mech = LegacyMechPad::new("mech-pad");
        assert!(mech.block.base.update);
        assert!(mech.block.base.has_power);
        assert_eq!(mech.discarded_floats, 3);

        let factory = LegacyUnitFactory::new("unit-factory");
        assert!(factory.block.base.update);
        assert!(factory.block.base.has_power);
        assert!(factory.block.base.has_items);
        assert!(!factory.block.base.solid);
        assert_eq!(factory.replacement, Tile::AIR);
    }

    #[test]
    fn legacy_shell_codec_and_removal_helpers_delegate_to_runtime_functions() {
        let center = LegacyCommandCenter::new("command-center");
        let mut bytes = Vec::new();
        center.write_extra(&mut bytes).unwrap();
        assert_eq!(bytes, vec![0]);
        assert_eq!(center.read_extra(&mut bytes.as_slice()).unwrap(), 0);

        let mech = LegacyMechPad::new("mech-pad");
        let mut bytes = Vec::new();
        mech.write_extra(&mut bytes, [1.25, -2.0, 3.5]).unwrap();
        assert_eq!(
            mech.read_extra(&mut bytes.as_slice()).unwrap(),
            [1.25, -2.0, 3.5]
        );

        let factory = LegacyUnitFactory::new("unit-factory");
        let mut tile = Tile::with_blocks(1, 2, 3, 4, 7);
        tile.build = Some(crate::mindustry::world::tile::BuildingRef {
            tile_pos: tile.pos(),
            block: 7,
            team: 1,
            rotation: 3,
        });
        assert_eq!(
            factory.remove_self(&tile),
            LegacyRemoval::Replace {
                block: Tile::AIR,
                rotation: 3
            }
        );

        let mut bytes = Vec::new();
        let extra = LegacyUnitFactoryExtra {
            build_time: 42.0,
            spawn_count: Some(7),
        };
        factory.write_extra(&mut bytes, 0, &extra).unwrap();
        assert_eq!(factory.read_extra(&mut bytes.as_slice(), 0).unwrap(), extra);
    }
}
