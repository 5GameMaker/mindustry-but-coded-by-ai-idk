use std::array;
use std::io::{self, Read, Write};

/// Packed equivalent of upstream generated `mindustry.gen.BufferItem`.
///
/// Java layout:
/// - `item` bits `[0..16]`
/// - `time` bits `[16..48]`, stored as raw `float` bits
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BufferItem {
    pub item: i16,
    pub time: f32,
}

impl BufferItem {
    pub fn new(item: i16, time: f32) -> Self {
        Self { item, time }
    }

    pub fn pack(self) -> u64 {
        (self.item as u16 as u64) | ((self.time.to_bits() as u64) << 16)
    }

    pub fn unpack(value: u64) -> Self {
        Self {
            item: (value & 0xffff) as u16 as i16,
            time: f32::from_bits((value >> 16) as u32),
        }
    }
}

/// Packed equivalent of upstream generated `mindustry.gen.BufferItemLegacy`.
///
/// Java layout:
/// - `item` bits `[0..8]`
/// - `time` bits `[8..40]`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BufferItemLegacy {
    pub item: i8,
    pub time: f32,
}

impl BufferItemLegacy {
    pub fn new(item: i8, time: f32) -> Self {
        Self { item, time }
    }

    pub fn pack(self) -> u64 {
        (self.item as u8 as u64) | ((self.time.to_bits() as u64) << 8)
    }

    pub fn unpack(value: u64) -> Self {
        Self {
            item: (value & 0xff) as u8 as i8,
            time: f32::from_bits((value >> 8) as u32),
        }
    }

    pub fn upgrade(self) -> BufferItem {
        BufferItem::new(self.item as i16, self.time)
    }
}

/// Four-direction FIFO item buffers used by junction-like distribution blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectionalItemBuffer {
    pub buffers: [Vec<u64>; 4],
    pub indexes: [usize; 5],
}

impl DirectionalItemBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffers: array::from_fn(|_| vec![0; capacity]),
            indexes: [0; 5],
        }
    }

    pub fn capacity(&self) -> usize {
        self.buffers[0].len()
    }

    pub fn len(&self, buffer: usize) -> usize {
        self.indexes.get(buffer).copied().unwrap_or(0)
    }

    pub fn accepts(&self, buffer: usize) -> bool {
        buffer < 4 && self.indexes[buffer] < self.buffers[buffer].len()
    }

    pub fn accept(&mut self, buffer: usize, item: i16, now: f32) -> bool {
        if !self.accepts(buffer) {
            return false;
        }

        self.buffers[buffer][self.indexes[buffer]] = BufferItem::new(item, now).pack();
        self.indexes[buffer] += 1;
        true
    }

    pub fn poll(&self, buffer: usize, speed: f32, now: f32) -> Option<i16> {
        self.peek(buffer, speed, now).map(|entry| entry.item)
    }

    pub fn peek(&self, buffer: usize, speed: f32, now: f32) -> Option<BufferItem> {
        if buffer >= 4 || self.indexes[buffer] == 0 {
            return None;
        }

        let entry = BufferItem::unpack(self.buffers[buffer][0]);
        if now >= entry.time + speed || now < entry.time {
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove(&mut self, buffer: usize) -> Option<BufferItem> {
        if buffer >= 4 || self.indexes[buffer] == 0 {
            return None;
        }

        let removed = BufferItem::unpack(self.buffers[buffer][0]);
        let len = self.indexes[buffer];
        if len > 1 {
            self.buffers[buffer].copy_within(1..len, 0);
        }
        self.indexes[buffer] -= 1;
        let next_len = self.indexes[buffer];
        if next_len < self.buffers[buffer].len() {
            self.buffers[buffer][next_len] = 0;
        }
        Some(removed)
    }

    pub fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
        for i in 0..4 {
            write_u8(write, self.indexes[i] as u8)?;
            write_u8(write, self.buffers[i].len() as u8)?;
            for value in &self.buffers[i] {
                write_u64(write, *value)?;
            }
        }
        Ok(())
    }

    pub fn read<R: Read>(&mut self, read: &mut R) -> io::Result<()> {
        self.read_with_legacy(read, false)
    }

    pub fn read_with_legacy<R: Read>(&mut self, read: &mut R, legacy: bool) -> io::Result<()> {
        for i in 0..4 {
            let index = read_u8(read)? as usize;
            let length = read_u8(read)? as usize;
            self.indexes[i] = index.min(self.buffers[i].len());
            for j in 0..length {
                let mut value = read_u64(read)?;
                if legacy {
                    value = BufferItemLegacy::unpack(value).upgrade().pack();
                }
                if j < self.buffers[i].len() {
                    self.buffers[i][j] = value;
                }
            }
        }
        Ok(())
    }
}

impl Default for DirectionalItemBuffer {
    fn default() -> Self {
        Self::new(0)
    }
}

fn write_u8<W: Write>(write: &mut W, value: u8) -> io::Result<()> {
    write.write_all(&[value])
}

fn read_u8<R: Read>(read: &mut R) -> io::Result<u8> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn write_u64<W: Write>(write: &mut W, value: u64) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_u64<R: Read>(read: &mut R) -> io::Result<u64> {
    let mut buf = [0; 8];
    read.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_item_packs_like_generated_java_struct() {
        let packed = BufferItem::new(7, 12.5).pack();

        assert_eq!(packed & 0xffff, 7);
        assert_eq!((packed >> 16) as u32, 12.5f32.to_bits());
        assert_eq!(BufferItem::unpack(packed), BufferItem::new(7, 12.5));
    }

    #[test]
    fn legacy_buffer_item_upgrades_one_byte_item_layout() {
        let legacy = BufferItemLegacy::new(42, 9.25).pack();
        let upgraded = BufferItemLegacy::unpack(legacy).upgrade();

        assert_eq!(legacy & 0xff, 42);
        assert_eq!((legacy >> 8) as u32, 9.25f32.to_bits());
        assert_eq!(upgraded, BufferItem::new(42, 9.25));
    }

    #[test]
    fn directional_item_buffer_accepts_polls_and_removes_per_side() {
        let mut buffer = DirectionalItemBuffer::new(2);

        assert!(buffer.accept(0, 3, 10.0));
        assert!(buffer.accept(0, 4, 12.0));
        assert!(buffer.accept(1, 7, 5.0));
        assert!(!buffer.accept(0, 5, 13.0));
        assert_eq!(buffer.len(0), 2);
        assert_eq!(buffer.len(1), 1);

        assert_eq!(buffer.poll(0, 5.0, 14.9), None);
        assert_eq!(buffer.poll(0, 5.0, 15.0), Some(3));
        assert_eq!(buffer.poll(1, 1.0, 6.0), Some(7));

        assert_eq!(buffer.remove(0).unwrap().item, 3);
        assert_eq!(buffer.poll(0, 0.0, 12.0), Some(4));
        assert_eq!(buffer.remove(4), None);
    }

    #[test]
    fn directional_item_buffer_serialization_keeps_four_side_order() {
        let mut buffer = DirectionalItemBuffer::new(1);
        buffer.accept(0, 1, 1.0);
        buffer.accept(2, 3, 3.0);

        let mut bytes = Vec::new();
        buffer.write(&mut bytes).unwrap();
        assert_eq!(bytes.len(), 4 * (2 + 8));
        assert_eq!(bytes[0], 1);
        assert_eq!(bytes[1], 1);
        assert_eq!(
            u64::from_be_bytes(bytes[2..10].try_into().unwrap()),
            BufferItem::new(1, 1.0).pack()
        );
        let side_two = 2 * (2 + 8);
        assert_eq!(bytes[side_two], 1);
        assert_eq!(
            u64::from_be_bytes(bytes[side_two + 2..side_two + 10].try_into().unwrap()),
            BufferItem::new(3, 3.0).pack()
        );

        let mut restored = DirectionalItemBuffer::new(1);
        restored.read(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored.poll(0, 0.0, 1.0), Some(1));
        assert_eq!(restored.poll(2, 0.0, 3.0), Some(3));
    }

    #[test]
    fn directional_item_buffer_reads_legacy_entries_as_modern_buffer_items() {
        let mut bytes = Vec::new();
        for side in 0..4 {
            bytes.push(if side == 1 { 1 } else { 0 });
            bytes.push(1);
            let value = if side == 1 {
                BufferItemLegacy::new(11, 22.0).pack()
            } else {
                0
            };
            bytes.extend_from_slice(&value.to_be_bytes());
        }

        let mut restored = DirectionalItemBuffer::new(1);
        restored
            .read_with_legacy(&mut bytes.as_slice(), true)
            .unwrap();

        assert_eq!(restored.peek(1, 0.0, 22.0), Some(BufferItem::new(11, 22.0)));
        assert_eq!(restored.poll(0, 0.0, 0.0), None);
    }
}
