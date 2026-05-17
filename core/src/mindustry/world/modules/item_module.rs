use std::collections::BTreeMap;

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
}
