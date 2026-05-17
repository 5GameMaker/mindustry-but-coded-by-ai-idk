use std::io::{Read, Write};

pub const PRIORITY_LOW: i32 = 0;
pub const PRIORITY_NORMAL: i32 = 1;
pub const PRIORITY_HIGH: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketPriority {
    Low,
    Normal,
    High,
}

impl PacketPriority {
    pub const fn wire_value(self) -> i32 {
        match self {
            PacketPriority::Low => PRIORITY_LOW,
            PacketPriority::Normal => PRIORITY_NORMAL,
            PacketPriority::High => PRIORITY_HIGH,
        }
    }
}

pub trait PacketRuntime {
    fn priority(&self) -> PacketPriority {
        PacketPriority::Normal
    }

    fn allow(&self, _server: bool) -> bool {
        true
    }

    fn handled(&mut self) {}
}

pub trait PacketCodec: Sized {
    fn read_from<R: Read>(read: &mut R) -> std::io::Result<Self>;
    fn write_to<W: Write>(&self, write: &mut W) -> std::io::Result<()>;
}
