//! Mirrors the packed draw command format used by upstream logic displays.

use super::{logic_assembler::double_bits_to_rgba, GraphicsType, LVar};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicDisplayCommand {
    pub type_: u8,
    pub x: u16,
    pub y: u16,
    pub p1: u16,
    pub p2: u16,
    pub p3: u16,
    pub p4: u16,
}

impl LogicDisplayCommand {
    pub const DISPLAY_DRAW_TYPE: i32 = 30;
    pub const SCALE_STEP: f32 = 0.05;

    pub const fn get(type_: u8, x: u16, y: u16, p1: u16, p2: u16, p3: u16, p4: u16) -> u64 {
        (type_ as u64 & 0x0f)
            | ((x as u64 & 0x03ff) << 4)
            | ((y as u64 & 0x03ff) << 14)
            | ((p1 as u64 & 0x03ff) << 24)
            | ((p2 as u64 & 0x03ff) << 34)
            | ((p3 as u64 & 0x03ff) << 44)
            | ((p4 as u64 & 0x03ff) << 54)
    }

    pub const fn unpack(value: u64) -> Self {
        Self {
            type_: (value & 0x0f) as u8,
            x: ((value >> 4) & 0x03ff) as u16,
            y: ((value >> 14) & 0x03ff) as u16,
            p1: ((value >> 24) & 0x03ff) as u16,
            p2: ((value >> 34) & 0x03ff) as u16,
            p3: ((value >> 44) & 0x03ff) as u16,
            p4: ((value >> 54) & 0x03ff) as u16,
        }
    }

    pub const fn pack(value: i32) -> u16 {
        (value & 0b0111111111) as u16
    }

    pub fn pack_sign(value: i32) -> u16 {
        ((value.abs() & 0b0111111111) | if value < 0 { 0b1000000000 } else { 0 }) as u16
    }

    pub const fn unpack_sign(value: u16) -> i32 {
        ((value & 0b0111111111) as i32) * if (value & 0b1000000000) != 0 { -1 } else { 1 }
    }

    pub fn from_draw_instruction(
        type_: GraphicsType,
        x: &LVar,
        y: &LVar,
        p1: &LVar,
        p2: &LVar,
        p3: &LVar,
        p4: &LVar,
    ) -> Option<u64> {
        let type_id = type_.ordinal();
        if type_ == GraphicsType::Col {
            let rgba = double_bits_to_rgba(x.num());
            return Some(Self::get(
                GraphicsType::Color.ordinal(),
                Self::pack(((rgba >> 24) & 0xff) as i32),
                Self::pack(((rgba >> 16) & 0xff) as i32),
                Self::pack(((rgba >> 8) & 0xff) as i32),
                Self::pack((rgba & 0xff) as i32),
                0,
                0,
            ));
        }

        let mut num1 = Self::pack_sign(p1.numi());
        let mut num4 = Self::pack_sign(p4.numi());
        let mut xval = Self::pack_sign(x.numi());
        let mut yval = Self::pack_sign(y.numi());

        if type_ == GraphicsType::Image {
            let packed = -1;
            num1 = (packed & 0x3ff) as u16;
            num4 = ((packed >> 10) & 0x3ff) as u16;
        } else if type_ == GraphicsType::Scale {
            xval = Self::pack_sign((x.numf() / Self::SCALE_STEP) as i32);
            yval = Self::pack_sign((y.numf() / Self::SCALE_STEP) as i32);
        } else if type_ == GraphicsType::Print {
            return None;
        }

        Some(Self::get(
            type_id,
            xval,
            yval,
            num1,
            Self::pack_sign(p2.numi()),
            Self::pack_sign(p3.numi()),
            num4,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::LogicDisplayCommand;

    #[test]
    fn display_command_packing_matches_java_bit_layout() {
        let packed = LogicDisplayCommand::get(4, 1, 2, 3, 4, 5, 6);
        assert_eq!(
            LogicDisplayCommand::unpack(packed),
            LogicDisplayCommand {
                type_: 4,
                x: 1,
                y: 2,
                p1: 3,
                p2: 4,
                p3: 5,
                p4: 6
            }
        );
        assert_eq!(LogicDisplayCommand::pack(1025), 1);
        assert_eq!(LogicDisplayCommand::pack_sign(-12), 0b1000001100);
        assert_eq!(LogicDisplayCommand::unpack_sign(0b1000001100), -12);
    }
}
