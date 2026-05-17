use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LiquidModule {
    liquids: BTreeMap<i16, f32>,
    current: Option<i16>,
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
    }

    pub fn remove(&mut self, liquid_id: i16, amount: f32) {
        self.add(liquid_id, amount.min(self.get(liquid_id)) * -1.0);
    }

    pub fn clear(&mut self) {
        self.liquids.clear();
        self.current = None;
    }
}
