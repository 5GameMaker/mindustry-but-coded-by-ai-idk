use std::io::{self, Read, Write};

use crate::mindustry::world::BlockId;

pub mod legacy_block;

pub use legacy_block::LegacyBlock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyRemoval {
    Remove,
    Replace { block: BlockId, rotation: i32 },
}

pub fn legacy_remove_self() -> LegacyRemoval {
    LegacyRemoval::Remove
}

pub fn legacy_unit_factory_remove_self(
    replacement: BlockId,
    build_rotation: Option<i32>,
) -> LegacyRemoval {
    LegacyRemoval::Replace {
        block: replacement,
        rotation: build_rotation.unwrap_or(0),
    }
}

pub fn read_legacy_mech_pad_extra<R: Read>(read: &mut R) -> io::Result<[f32; 3]> {
    Ok([read_f32(read)?, read_f32(read)?, read_f32(read)?])
}

pub fn write_legacy_mech_pad_extra<W: Write>(write: &mut W, values: [f32; 3]) -> io::Result<()> {
    for value in values {
        write_f32(write, value)?;
    }
    Ok(())
}

pub fn read_legacy_unit_factory_extra<R: Read>(
    read: &mut R,
    revision: u8,
) -> io::Result<LegacyUnitFactoryExtra> {
    let build_time = read_f32(read)?;
    let spawn_count = if revision == 0 {
        Some(read_i32(read)?)
    } else {
        None
    };
    Ok(LegacyUnitFactoryExtra {
        build_time,
        spawn_count,
    })
}

pub fn write_legacy_unit_factory_extra<W: Write>(
    write: &mut W,
    revision: u8,
    extra: &LegacyUnitFactoryExtra,
) -> io::Result<()> {
    write_f32(write, extra.build_time)?;
    if revision == 0 {
        write_i32(write, extra.spawn_count.unwrap_or(0))?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegacyUnitFactoryExtra {
    pub build_time: f32,
    pub spawn_count: Option<i32>,
}

pub fn write_legacy_command_center_extra<W: Write>(write: &mut W) -> io::Result<()> {
    write.write_all(&[0])
}

pub fn read_legacy_command_center_extra<R: Read>(read: &mut R) -> io::Result<u8> {
    read_u8(read)
}

fn read_u8<R: Read>(read: &mut R) -> io::Result<u8> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_i32<R: Read>(read: &mut R) -> io::Result<i32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

fn write_i32<W: Write>(write: &mut W, value: i32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_remove_self_defaults_to_remove_tile() {
        assert_eq!(legacy_remove_self(), LegacyRemoval::Remove);
    }

    #[test]
    fn legacy_unit_factory_replaces_with_preserved_or_zero_rotation() {
        assert_eq!(
            legacy_unit_factory_remove_self(12, Some(3)),
            LegacyRemoval::Replace {
                block: 12,
                rotation: 3
            }
        );
        assert_eq!(
            legacy_unit_factory_remove_self(12, None),
            LegacyRemoval::Replace {
                block: 12,
                rotation: 0
            }
        );
    }

    #[test]
    fn legacy_mech_pad_discards_three_float_payload_values() {
        let mut bytes = Vec::new();
        write_legacy_mech_pad_extra(&mut bytes, [1.25, -2.0, 3.5]).unwrap();
        let extra = read_legacy_mech_pad_extra(&mut bytes.as_slice()).unwrap();
        assert_eq!(extra, [1.25, -2.0, 3.5]);
    }

    #[test]
    fn legacy_unit_factory_reads_build_time_and_revision_zero_spawn_count() {
        let mut bytes = Vec::new();
        write_legacy_unit_factory_extra(
            &mut bytes,
            0,
            &LegacyUnitFactoryExtra {
                build_time: 42.0,
                spawn_count: Some(7),
            },
        )
        .unwrap();
        let extra = read_legacy_unit_factory_extra(&mut bytes.as_slice(), 0).unwrap();
        assert_eq!(
            extra,
            LegacyUnitFactoryExtra {
                build_time: 42.0,
                spawn_count: Some(7)
            }
        );

        let mut bytes = Vec::new();
        write_legacy_unit_factory_extra(
            &mut bytes,
            1,
            &LegacyUnitFactoryExtra {
                build_time: 11.0,
                spawn_count: Some(99),
            },
        )
        .unwrap();
        let extra = read_legacy_unit_factory_extra(&mut bytes.as_slice(), 1).unwrap();
        assert_eq!(extra.spawn_count, None);
        assert_eq!(extra.build_time, 11.0);
    }

    #[test]
    fn legacy_command_center_writes_and_reads_zero_byte() {
        let mut bytes = Vec::new();
        write_legacy_command_center_extra(&mut bytes).unwrap();
        assert_eq!(bytes, vec![0]);
        assert_eq!(
            read_legacy_command_center_extra(&mut bytes.as_slice()).unwrap(),
            0
        );
    }
}
