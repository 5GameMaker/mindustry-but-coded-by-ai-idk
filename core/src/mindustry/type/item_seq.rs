use std::collections::BTreeMap;

use super::ItemStack;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSeq {
    values: Vec<i32>,
    names: Vec<String>,
    pub total: i32,
}

impl ItemSeq {
    pub fn new<I, S>(items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let names: Vec<String> = items.into_iter().map(Into::into).collect();
        Self {
            values: vec![0; names.len()],
            names,
            total: 0,
        }
    }

    pub fn from_stacks<I, S>(items: I, stacks: &[ItemStack]) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut out = Self::new(items);
        for stack in stacks {
            out.add_name(&stack.item, stack.amount);
        }
        out
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    pub fn check_negative(&mut self) {
        for value in &mut self.values {
            if *value < 0 {
                self.total -= *value;
                *value = 0;
            }
        }
    }

    pub fn clear(&mut self) {
        self.total = 0;
        self.values.fill(0);
    }

    pub fn min(&mut self, number: i32) {
        for index in 0..self.values.len() {
            let current = self.values[index];
            if current > number {
                self.set(index, number);
            }
        }
    }

    pub fn has(&self, id: usize) -> bool {
        self.get(id) > 0
    }

    pub fn has_amount(&self, id: usize, amount: i32) -> bool {
        self.get(id) >= amount
    }

    pub fn has_seq(&self, seq: &ItemSeq) -> bool {
        seq.values
            .iter()
            .enumerate()
            .all(|(index, amount)| *amount <= self.get(index))
    }

    pub fn get(&self, id: usize) -> i32 {
        self.values.get(id).copied().unwrap_or(0)
    }

    pub fn get_name(&self, name: &str) -> i32 {
        self.index_of(name).map_or(0, |index| self.get(index))
    }

    pub fn set(&mut self, id: usize, amount: i32) {
        let current = self.get(id);
        self.add(id, amount - current);
    }

    pub fn add(&mut self, id: usize, amount: i32) {
        if let Some(value) = self.values.get_mut(id) {
            *value += amount;
            self.total += amount;
        }
    }

    pub fn add_name(&mut self, name: &str, amount: i32) {
        if let Some(index) = self.index_of(name) {
            self.add(index, amount);
        }
    }

    pub fn add_stack(&mut self, stack: &ItemStack) {
        self.add_name(&stack.item, stack.amount);
    }

    pub fn add_seq(&mut self, seq: &ItemSeq) {
        for (index, value) in seq.values.iter().copied().enumerate() {
            self.add(index, value);
        }
    }

    pub fn remove(&mut self, id: usize, amount: i32) {
        self.add(id, -amount);
    }

    pub fn to_vec(&self) -> Vec<ItemStack> {
        self.values
            .iter()
            .enumerate()
            .filter_map(|(index, amount)| {
                if *amount != 0 {
                    Some(ItemStack::new(self.names[index].clone(), *amount))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn to_json_map(&self) -> BTreeMap<String, i32> {
        self.to_vec()
            .into_iter()
            .map(|stack| (stack.item, stack.amount))
            .collect()
    }

    pub fn read_json_map(&mut self, values: &BTreeMap<String, i32>) {
        self.clear();
        let names = self.names.clone();
        for (index, name) in names.iter().enumerate() {
            self.set(index, values.get(name).copied().unwrap_or(0));
        }
    }

    fn index_of(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|item| item == name)
    }
}

impl IntoIterator for ItemSeq {
    type Item = ItemStack;
    type IntoIter = std::vec::IntoIter<ItemStack>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_seq_follows_java_amount_total_and_json_semantics() {
        let mut seq = ItemSeq::new(["copper", "lead", "graphite"]);
        assert_eq!(seq.len(), 3);
        assert!(seq.is_empty());
        seq.add_name("copper", 3);
        seq.add(1, 2);
        seq.remove(0, 1);
        assert_eq!(seq.total, 4);
        assert_eq!(seq.get_name("copper"), 2);
        assert_eq!(seq.get_name("lead"), 2);
        assert!(seq.has_amount(0, 2));
        assert_eq!(
            seq.to_vec(),
            vec![ItemStack::new("copper", 2), ItemStack::new("lead", 2)]
        );

        let mut req = ItemSeq::new(["copper", "lead", "graphite"]);
        req.add_name("lead", 1);
        assert!(seq.has_seq(&req));
        req.add_name("graphite", 1);
        assert!(!seq.has_seq(&req));

        seq.add_name("lead", -5);
        assert_eq!(seq.get_name("lead"), -3);
        seq.check_negative();
        assert_eq!(seq.get_name("lead"), 0);
        assert_eq!(seq.total, 2);
        seq.min(1);
        assert_eq!(seq.get_name("copper"), 1);
        assert_eq!(seq.total, 1);

        let json = BTreeMap::from([(String::from("lead"), 4)]);
        seq.read_json_map(&json);
        assert_eq!(seq.to_json_map(), json);
        assert_eq!(seq.total, 4);
    }
}
