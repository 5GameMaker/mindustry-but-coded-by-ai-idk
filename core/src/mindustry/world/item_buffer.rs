use std::io::{self, Read, Write};

/// Packed equivalent of upstream generated `mindustry.gen.TimeItem`.
///
/// Java layout:
/// - `data` bits `[0..16]`
/// - `item` bits `[16..32]`
/// - `time` bits `[32..64]`, stored as raw `float` bits
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeItem {
    pub data: i16,
    pub item: i16,
    pub time: f32,
}

impl TimeItem {
    pub fn new(data: i16, item: i16, time: f32) -> Self {
        Self { data, item, time }
    }

    pub fn pack(self) -> u64 {
        (self.data as u16 as u64)
            | ((self.item as u16 as u64) << 16)
            | ((self.time.to_bits() as u64) << 32)
    }

    pub fn unpack(value: u64) -> Self {
        Self {
            data: (value & 0xffff) as u16 as i16,
            item: ((value >> 16) & 0xffff) as u16 as i16,
            time: f32::from_bits((value >> 32) as u32),
        }
    }
}

/// FIFO item buffer used by distribution blocks.
///
/// This mirrors `mindustry.world.ItemBuffer` while keeping content lookup out of
/// the structure; `poll` returns the raw item id that callers may resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemBuffer {
    buffer: Vec<u64>,
    index: usize,
}

impl ItemBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0; capacity],
            index: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    pub fn len(&self) -> usize {
        self.index
    }

    pub fn is_empty(&self) -> bool {
        self.index == 0
    }

    pub fn accepts(&self) -> bool {
        self.index < self.buffer.len()
    }

    pub fn accept(&mut self, item: i16, data: i16, now: f32) -> bool {
        if !self.accepts() {
            return false;
        }
        self.buffer[self.index] = TimeItem::new(data, item, now).pack();
        self.index += 1;
        true
    }

    pub fn accept_item(&mut self, item: i16, now: f32) -> bool {
        self.accept(item, -1, now)
    }

    pub fn poll(&self, speed: f32, now: f32) -> Option<i16> {
        self.peek(speed, now).map(|entry| entry.item)
    }

    pub fn peek(&self, speed: f32, now: f32) -> Option<TimeItem> {
        if self.index == 0 {
            return None;
        }

        let entry = TimeItem::unpack(self.buffer[0]);
        if now >= entry.time + speed || now < entry.time {
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove(&mut self) -> Option<TimeItem> {
        if self.index == 0 {
            return None;
        }

        let removed = TimeItem::unpack(self.buffer[0]);
        if self.index > 1 {
            self.buffer.copy_within(1..self.index, 0);
        }
        self.index -= 1;
        if self.index < self.buffer.len() {
            self.buffer[self.index] = 0;
        }
        Some(removed)
    }

    pub fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
        write_u8(write, self.index as u8)?;
        write_u8(write, self.buffer.len() as u8)?;
        for value in &self.buffer {
            write_u64(write, *value)?;
        }
        Ok(())
    }

    pub fn read<R: Read>(&mut self, read: &mut R) -> io::Result<()> {
        let index = read_u8(read)? as usize;
        let length = read_u8(read)? as usize;
        for i in 0..length {
            let value = read_u64(read)?;
            if i < self.buffer.len() {
                self.buffer[i] = value;
            }
        }

        let java_max_index = length.saturating_sub(1);
        self.index = index.min(java_max_index).min(self.buffer.len());
        Ok(())
    }

    pub fn raw_buffer(&self) -> &[u64] {
        &self.buffer
    }
}

impl Default for ItemBuffer {
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
    fn time_item_packs_like_generated_java_struct() {
        let packed = TimeItem::new(-1, 7, 12.5).pack();

        assert_eq!(packed & 0xffff, u16::MAX as u64);
        assert_eq!((packed >> 16) & 0xffff, 7);
        assert_eq!((packed >> 32) as u32, 12.5f32.to_bits());
        assert_eq!(TimeItem::unpack(packed), TimeItem::new(-1, 7, 12.5));
    }

    #[test]
    fn item_buffer_accepts_polls_after_delay_and_removes_fifo() {
        let mut buffer = ItemBuffer::new(2);

        assert!(buffer.accept_item(3, 10.0));
        assert!(buffer.accept(4, 99, 12.0));
        assert!(!buffer.accept_item(5, 13.0));
        assert!(!buffer.accepts());
        assert_eq!(buffer.poll(5.0, 14.9), None);
        assert_eq!(buffer.poll(5.0, 15.0), Some(3));

        let removed = buffer.remove().unwrap();
        assert_eq!(removed.item, 3);
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.peek(0.0, 12.0).unwrap().data, 99);
        assert_eq!(buffer.remove().unwrap().item, 4);
        assert!(buffer.remove().is_none());
    }

    #[test]
    fn item_buffer_poll_allows_time_wrap_like_java() {
        let mut buffer = ItemBuffer::new(1);
        buffer.accept_item(2, 100.0);

        assert_eq!(buffer.poll(30.0, 10.0), Some(2));
    }

    #[test]
    fn item_buffer_serialization_uses_java_order_and_read_clamp() {
        let mut buffer = ItemBuffer::new(3);
        buffer.accept_item(1, 1.0);
        buffer.accept_item(2, 2.0);
        buffer.accept_item(3, 3.0);

        let mut bytes = Vec::new();
        buffer.write(&mut bytes).unwrap();
        assert_eq!(bytes[0], 3);
        assert_eq!(bytes[1], 3);
        assert_eq!(
            u64::from_be_bytes(bytes[2..10].try_into().unwrap()),
            TimeItem::new(-1, 1, 1.0).pack()
        );

        let mut restored = ItemBuffer::new(2);
        restored.read(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored.len(), 2);
        assert_eq!(restored.poll(0.0, 1.0), Some(1));
        restored.remove();
        assert_eq!(restored.poll(0.0, 2.0), Some(2));
    }
}
