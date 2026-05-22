use std::{
    collections::BTreeMap,
    io::{self, Read, Write},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemModule {
    items: BTreeMap<i16, i32>,
    total: i32,
    take_rotation: usize,
}

impl ItemModule {
    pub fn total(&self) -> i32 {
        self.total
    }
    pub fn any(&self) -> bool {
        self.total > 0
    }
    pub fn empty(&self) -> bool {
        self.total == 0
    }
    pub fn get(&self, item_id: i16) -> i32 {
        *self.items.get(&item_id).unwrap_or(&0)
    }

    pub fn set(&mut self, item_id: i16, amount: i32) {
        let prev = self.get(item_id);
        self.total += amount - prev;
        if amount == 0 {
            self.items.remove(&item_id);
        } else {
            self.items.insert(item_id, amount);
        }
    }

    pub fn add(&mut self, item_id: i16, amount: i32) {
        self.set(item_id, self.get(item_id) + amount);
    }

    pub fn remove(&mut self, item_id: i16, amount: i32) {
        let remove = amount.min(self.get(item_id));
        self.set(item_id, self.get(item_id) - remove);
    }

    pub fn take(&mut self) -> Option<i16> {
        if self.items.is_empty() {
            return None;
        }
        let keys: Vec<_> = self.items.keys().copied().collect();
        for offset in 0..keys.len() {
            let idx = (self.take_rotation + offset) % keys.len();
            let id = keys[idx];
            if self.get(id) > 0 {
                self.remove(id, 1);
                self.take_rotation = idx + 1;
                return Some(id);
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.total = 0;
    }

    pub fn each(&self) -> impl Iterator<Item = (i16, i32)> + '_ {
        self.items
            .iter()
            .filter(|(_, amount)| **amount > 0)
            .map(|(id, amount)| (*id, *amount))
    }

    pub fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
        let positive: Vec<_> = self.each().collect();
        write_i16(write, positive.len() as i16)?;
        for (id, amount) in positive {
            write_i16(write, id)?;
            write_i32(write, amount)?;
        }
        Ok(())
    }

    pub fn read<R: Read>(&mut self, read: &mut R, legacy: bool) -> io::Result<()> {
        self.clear();
        let count = if legacy {
            read_u8(read)? as i16
        } else {
            read_i16(read)?
        };
        if count < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "negative item module entry count",
            ));
        }
        for _ in 0..count {
            let id = if legacy {
                read_u8(read)? as i16
            } else {
                read_i16(read)?
            };
            let amount = read_i32(read)?;
            self.set(id, amount);
        }
        Ok(())
    }
}

fn write_i16<W: Write>(write: &mut W, value: i16) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_i16<R: Read>(read: &mut R) -> io::Result<i16> {
    let mut buf = [0; 2];
    read.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

fn read_u8<R: Read>(read: &mut R) -> io::Result<u8> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn write_i32<W: Write>(write: &mut W, value: i32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_i32<R: Read>(read: &mut R) -> io::Result<i32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_remove_total_matches_java_semantics() {
        let mut module = ItemModule::default();
        module.add(1, 5);
        module.remove(1, 2);
        assert_eq!(module.get(1), 3);
        assert_eq!(module.total(), 3);
        assert_eq!(module.take(), Some(1));
        assert_eq!(module.total(), 2);
    }

    #[test]
    fn item_module_serialization_follows_java_layout() {
        let mut module = ItemModule::default();
        module.set(1, 3);
        module.set(4, 7);
        module.set(9, 0);

        let mut bytes = Vec::new();
        module.write(&mut bytes).unwrap();
        assert_eq!(bytes[0..2], 2i16.to_be_bytes());
        assert_eq!(bytes[2..4], 1i16.to_be_bytes());
        assert_eq!(bytes[4..8], 3i32.to_be_bytes());
        assert_eq!(bytes[8..10], 4i16.to_be_bytes());
        assert_eq!(bytes[10..14], 7i32.to_be_bytes());

        let mut restored = ItemModule::default();
        restored.read(&mut bytes.as_slice(), false).unwrap();
        assert_eq!(restored.get(1), 3);
        assert_eq!(restored.get(4), 7);
        assert_eq!(restored.total(), 10);
    }

    #[test]
    fn item_module_legacy_read_uses_unsigned_byte_ids() {
        let legacy = [1u8, 7u8, 0, 0, 0, 11];
        let mut restored = ItemModule::default();

        restored.read(&mut legacy.as_slice(), true).unwrap();

        assert_eq!(restored.get(7), 11);
        assert_eq!(restored.total(), 11);
    }
}
