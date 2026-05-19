use std::{
    collections::BTreeMap,
    io::{self, Read, Write},
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LiquidModule {
    liquids: BTreeMap<i16, f32>,
    current: Option<i16>,
    flow_active: bool,
    cache_sums: BTreeMap<i16, f32>,
    display_flow: BTreeMap<i16, f32>,
    flow_seen: Vec<i16>,
}

impl LiquidModule {
    pub fn current(&self) -> Option<i16> {
        self.current
    }
    pub fn current_amount(&self) -> f32 {
        self.current.map(|id| self.get(id)).unwrap_or(0.0)
    }
    pub fn get(&self, liquid_id: i16) -> f32 {
        *self.liquids.get(&liquid_id).unwrap_or(&0.0)
    }

    pub fn reset(&mut self, liquid_id: i16, amount: f32) {
        self.liquids.clear();
        self.liquids.insert(liquid_id, amount);
        self.current = Some(liquid_id);
    }

    pub fn set(&mut self, liquid_id: i16, amount: f32) {
        if self
            .current
            .map(|id| amount >= self.get(id))
            .unwrap_or(true)
        {
            self.current = Some(liquid_id);
        }
        if amount == 0.0 {
            self.liquids.remove(&liquid_id);
        } else {
            self.liquids.insert(liquid_id, amount);
        }
    }

    pub fn add(&mut self, liquid_id: i16, amount: f32) {
        self.set(liquid_id, self.get(liquid_id) + amount);
        self.current = Some(liquid_id);
        if self.flow_active {
            let positive = amount.max(0.0);
            *self.cache_sums.entry(liquid_id).or_insert(0.0) += positive;
            if positive > 0.0 && !self.flow_seen.contains(&liquid_id) {
                self.flow_seen.push(liquid_id);
            }
        }
    }

    pub fn handle_flow(&mut self, liquid_id: i16, amount: f32) {
        if self.flow_active {
            let positive = amount.max(0.0);
            *self.cache_sums.entry(liquid_id).or_insert(0.0) += positive;
            if positive > 0.0 && !self.flow_seen.contains(&liquid_id) {
                self.flow_seen.push(liquid_id);
            }
        }
    }

    pub fn remove(&mut self, liquid_id: i16, amount: f32) {
        self.add(liquid_id, amount.min(self.get(liquid_id)) * -1.0);
    }

    pub fn clear(&mut self) {
        self.liquids.clear();
        self.current = None;
    }

    pub fn each(&self) -> impl Iterator<Item = (i16, f32)> + '_ {
        self.liquids
            .iter()
            .filter(|(_, amount)| **amount > 0.0)
            .map(|(id, amount)| (*id, *amount))
    }

    pub fn sum(&self, calc: impl Fn(i16, f32) -> f32) -> f32 {
        self.each().map(|(id, amount)| calc(id, amount)).sum()
    }

    pub fn start_flow(&mut self) {
        self.flow_active = true;
        self.cache_sums.clear();
        self.display_flow.clear();
        self.flow_seen.clear();
    }

    pub fn stop_flow(&mut self) {
        self.flow_active = false;
    }

    pub fn has_flow_liquid(&self, liquid_id: i16) -> bool {
        self.flow_active && self.flow_seen.contains(&liquid_id)
    }

    pub fn get_flow_rate(&self, liquid_id: i16) -> f32 {
        if self.flow_active {
            self.display_flow.get(&liquid_id).copied().unwrap_or(-1.0) * 60.0
        } else {
            -1.0
        }
    }

    pub fn poll_flow(&mut self, poll_scl: f32, update_display: bool) {
        if !self.flow_active {
            return;
        }
        let sums: Vec<_> = self
            .cache_sums
            .iter()
            .map(|(id, amount)| (*id, *amount))
            .collect();
        for (id, amount) in sums {
            if amount > 0.0 && !self.flow_seen.contains(&id) {
                self.flow_seen.push(id);
            }
            if update_display {
                self.display_flow.insert(id, amount / poll_scl);
            }
            self.cache_sums.insert(id, 0.0);
        }
    }

    pub fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
        let positive: Vec<_> = self.each().collect();
        write_i16(write, positive.len() as i16)?;
        for (id, amount) in positive {
            write_i16(write, id)?;
            write_f32(write, amount)?;
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
        for _ in 0..count {
            let id = if legacy {
                read_u8(read)? as i16
            } else {
                read_i16(read)?
            };
            let amount = read_f32(read)?;
            self.liquids.insert(id, amount);
            if self
                .current
                .map(|current| amount > self.get(current))
                .unwrap_or(true)
            {
                self.current = Some(id);
            }
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

fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liquid_module_runtime_flow_and_serialization_follow_java_layout() {
        let mut module = LiquidModule::default();
        module.reset(1, 2.0);
        module.set(2, 3.0);
        assert_eq!(module.current(), Some(2));
        assert_eq!(module.current_amount(), 3.0);
        module.start_flow();
        module.add(1, 5.0);
        module.handle_flow(2, 4.0);
        assert!(module.has_flow_liquid(1));
        module.poll_flow(20.0, true);
        assert_eq!(module.get_flow_rate(1), 15.0);
        assert_eq!(module.get_flow_rate(2), 12.0);
        module.remove(1, 100.0);
        assert_eq!(module.get(1), 0.0);
        assert_eq!(module.sum(|id, amount| id as f32 * amount), 6.0);

        let mut bytes = Vec::new();
        module.write(&mut bytes).unwrap();
        assert_eq!(bytes[0..2], 1i16.to_be_bytes());
        let mut restored = LiquidModule::default();
        restored.read(&mut bytes.as_slice(), false).unwrap();
        assert_eq!(restored.get(2), 3.0);
        assert_eq!(restored.current(), Some(2));

        let legacy = [1u8, 7u8, 64, 32, 0, 0];
        let mut restored = LiquidModule::default();
        restored.read(&mut legacy.as_slice(), true).unwrap();
        assert_eq!(restored.get(7), 2.5);
    }
}
