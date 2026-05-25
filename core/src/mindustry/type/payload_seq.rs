use std::collections::BTreeMap;

use crate::mindustry::ctype::{ContentId, ContentType};

use super::PayloadStack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PayloadKey {
    pub content_type: ContentType,
    pub id: ContentId,
}

impl PayloadKey {
    pub fn new(content_type: ContentType, id: ContentId) -> Self {
        Self { content_type, id }
    }
}

impl PartialOrd for PayloadKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PayloadKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.content_type.ordinal(), self.id).cmp(&(other.content_type.ordinal(), other.id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PayloadSeq {
    payloads: BTreeMap<PayloadKey, i32>,
    total: i32,
}

impl PayloadSeq {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    pub fn any(&self) -> bool {
        self.total > 0
    }

    pub fn total(&self) -> i32 {
        self.total
    }

    pub fn len(&self) -> usize {
        self.payloads.len()
    }

    pub fn add(&mut self, key: PayloadKey, amount: i32) {
        let entry = self.payloads.entry(key).or_insert(0);
        *entry += amount;
        self.total += amount;
    }

    pub fn add_one(&mut self, key: PayloadKey) {
        self.add(key, 1);
    }

    pub fn add_stack(&mut self, stack: &PayloadStack) {
        self.add(PayloadKey::new(stack.content_type, stack.id), stack.amount);
    }

    pub fn remove(&mut self, key: PayloadKey, amount: i32) {
        self.add(key, -amount);
    }

    pub fn remove_stack(&mut self, stack: &PayloadStack) {
        self.remove(PayloadKey::new(stack.content_type, stack.id), stack.amount);
    }

    pub fn remove_all<F>(&mut self, mut pred: F)
    where
        F: FnMut(PayloadKey) -> bool,
    {
        let keys: Vec<_> = self
            .payloads
            .keys()
            .copied()
            .filter(|key| pred(*key))
            .collect();
        for key in keys {
            if let Some(value) = self.payloads.remove(&key) {
                self.total -= value;
            }
        }
    }

    pub fn clear(&mut self) {
        self.payloads.clear();
        self.total = 0;
    }

    pub fn get(&self, key: PayloadKey) -> i32 {
        self.payloads.get(&key).copied().unwrap_or(0)
    }

    pub fn contains(&self, key: PayloadKey, amount: i32) -> bool {
        self.get(key) >= amount
    }

    pub fn contains_stack(&self, stack: &PayloadStack) -> bool {
        self.contains(PayloadKey::new(stack.content_type, stack.id), stack.amount)
    }

    pub fn contains_all(&self, stacks: &[PayloadStack]) -> bool {
        stacks.iter().all(|stack| self.contains_stack(stack))
    }

    pub fn entries(&self) -> impl Iterator<Item = (PayloadKey, i32)> + '_ {
        self.payloads.iter().map(|(key, amount)| (*key, *amount))
    }

    pub fn write_java_new(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(2 + self.payloads.len() * 7);
        out.extend_from_slice(&(-(self.payloads.len() as i16)).to_be_bytes());
        for (key, amount) in self.entries() {
            out.push(key.content_type.ordinal());
            out.extend_from_slice(&(key.id as i16).to_be_bytes());
            out.extend_from_slice(&amount.to_be_bytes());
        }
        out
    }

    pub fn read_java_new(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 2 {
            return Err("payload seq requires size short".to_string());
        }
        let count = i16::from_be_bytes([bytes[0], bytes[1]]);
        if count >= 0 {
            let count = count as usize;
            let expected = 2 + count * 6;
            if bytes.len() < expected {
                return Err(format!(
                    "legacy payload seq too short: expected {expected}, got {}",
                    bytes.len()
                ));
            }
            let mut seq = Self::new();
            let mut offset = 2;
            for _ in 0..count {
                let id = i16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as ContentId;
                offset += 2;
                let amount = i32::from_be_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                offset += 4;
                seq.add(PayloadKey::new(ContentType::Block, id), amount);
            }
            return Ok(seq);
        }
        let count = (-(count as i32)) as usize;
        let expected = 2 + count * 7;
        if bytes.len() < expected {
            return Err(format!(
                "payload seq too short: expected {expected}, got {}",
                bytes.len()
            ));
        }
        let mut seq = Self::new();
        let mut offset = 2;
        for _ in 0..count {
            let content_type = ContentType::from_ordinal(bytes[offset])
                .ok_or_else(|| format!("unknown content type ordinal {}", bytes[offset]))?;
            offset += 1;
            let id = i16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as ContentId;
            offset += 2;
            let amount = i32::from_be_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            offset += 4;
            seq.add(PayloadKey::new(content_type, id), amount);
        }
        Ok(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_seq_matches_java_new_wire_layout_and_container_semantics() {
        let router = PayloadKey::new(ContentType::Block, 1);
        let dagger = PayloadKey::new(ContentType::Unit, 7);
        let mut seq = PayloadSeq::new();
        assert!(seq.is_empty());
        seq.add(router, 3);
        seq.add_one(dagger);
        seq.remove(router, 1);
        assert!(seq.any());
        assert_eq!(seq.total(), 3);
        assert_eq!(seq.get(router), 2);
        assert!(seq.contains(router, 2));
        assert!(seq.contains_stack(&PayloadStack::new(ContentType::Unit, 7, "dagger", 1)));
        assert!(seq.contains_all(&[
            PayloadStack::new(ContentType::Block, 1, "router", 2),
            PayloadStack::new(ContentType::Unit, 7, "dagger", 1),
        ]));

        let bytes = seq.write_java_new();
        assert_eq!(
            bytes,
            vec![
                0xff, 0xfe, // negated short size = -2
                0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, // block #1 amount 2
                0x06, 0x00, 0x07, 0x00, 0x00, 0x00, 0x01, // unit #7 amount 1
            ]
        );
        let decoded = PayloadSeq::read_java_new(&bytes).unwrap();
        assert_eq!(decoded, seq);

        seq.remove_all(|key| key.content_type == ContentType::Block);
        assert_eq!(seq.get(router), 0);
        assert_eq!(seq.total(), 1);
        seq.clear();
        assert!(seq.is_empty());
    }

    #[test]
    fn payload_seq_reads_java_legacy_block_only_format() {
        let bytes = vec![
            0x00, 0x02, // positive short size = old block-only format
            0x00, 0x05, 0x00, 0x00, 0x00, 0x03, // block #5 amount 3
            0x00, 0x07, 0x00, 0x00, 0x00, 0x02, // block #7 amount 2
        ];
        let decoded = PayloadSeq::read_java_new(&bytes).unwrap();

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded.total(), 5);
        assert_eq!(decoded.get(PayloadKey::new(ContentType::Block, 5)), 3);
        assert_eq!(decoded.get(PayloadKey::new(ContentType::Block, 7)), 2);
        assert_eq!(decoded.get(PayloadKey::new(ContentType::Unit, 5)), 0);
    }
}
