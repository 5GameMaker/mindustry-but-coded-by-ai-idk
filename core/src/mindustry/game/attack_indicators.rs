//! Minimap attack indicator runtime state.
//!
//! Mirrors upstream `mindustry.game.AttackIndicators`, including the generated
//! `@Struct` bit layout for `Indicator` (`int pos`, `float time`) and the
//! unordered `LongSeq` removal behavior used while indicators expire.

use crate::mindustry::world::{point2_pack, point2_x, point2_y};

use std::collections::BTreeMap;

pub const ATTACK_INDICATOR_DURATION: f32 = 15.0 * 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AttackIndicator(pub u64);

impl AttackIndicator {
    pub fn get(pos: i32, time: f32) -> Self {
        let pos_bits = pos as u32 as u64;
        let time_bits = (time.to_bits() as u64) << 32;
        Self(pos_bits | time_bits)
    }

    pub fn from_xy(x: i32, y: i32, time: f32) -> Self {
        Self::get(point2_pack(x, y), time)
    }

    pub fn pos(self) -> i32 {
        self.0 as u32 as i32
    }

    pub fn x(self) -> i16 {
        point2_x(self.pos())
    }

    pub fn y(self) -> i16 {
        point2_y(self.pos())
    }

    pub fn time(self) -> f32 {
        f32::from_bits((self.0 >> 32) as u32)
    }

    pub fn with_pos(self, pos: i32) -> Self {
        Self((self.0 & 0xffff_ffff_0000_0000) | pos as u32 as u64)
    }

    pub fn with_time(self, time: f32) -> Self {
        Self((self.0 & 0x0000_0000_ffff_ffff) | ((time.to_bits() as u64) << 32))
    }
}

impl From<AttackIndicator> for u64 {
    fn from(value: AttackIndicator) -> Self {
        value.0
    }
}

impl From<u64> for AttackIndicator {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AttackIndicators {
    indicators: Vec<AttackIndicator>,
    pos_to_index: BTreeMap<i32, usize>,
}

impl AttackIndicators {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> &[AttackIndicator] {
        &self.indicators
    }

    pub fn raw_list(&self) -> Vec<u64> {
        self.indicators.iter().copied().map(Into::into).collect()
    }

    pub fn len(&self) -> usize {
        self.indicators.len()
    }

    pub fn is_empty(&self) -> bool {
        self.indicators.is_empty()
    }

    pub fn clear(&mut self) {
        self.indicators.clear();
        self.pos_to_index.clear();
    }

    pub fn add(&mut self, x: i32, y: i32) {
        self.add_packed(point2_pack(x, y));
    }

    pub fn add_packed(&mut self, pos: i32) {
        if let Some(&index) = self.pos_to_index.get(&pos) {
            if let Some(indicator) = self.indicators.get_mut(index) {
                *indicator = indicator.with_time(0.0);
            }
        } else {
            self.indicators.push(AttackIndicator::get(pos, 0.0));
            self.pos_to_index.insert(pos, self.indicators.len() - 1);
        }
    }

    pub fn update(&mut self, delta: f32) {
        let mut index = 0;
        while index < self.indicators.len() {
            let updated = self.indicators[index].with_time(self.indicators[index].time() + delta);
            self.indicators[index] = updated;

            if updated.time() >= ATTACK_INDICATOR_DURATION {
                let removed = self.indicators.swap_remove(index);
                self.pos_to_index.remove(&removed.pos());

                if let Some(relocated) = self.indicators.get(index) {
                    self.pos_to_index.insert(relocated.pos(), index);
                }
            } else {
                index += 1;
            }
        }
    }

    pub fn index_of_packed(&self, pos: i32) -> Option<usize> {
        self.pos_to_index.get(&pos).copied()
    }

    pub fn index_of(&self, x: i32, y: i32) -> Option<usize> {
        self.index_of_packed(point2_pack(x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicator_struct_packs_int_pos_and_float_time_like_generated_java_struct() {
        let packed_pos = point2_pack(-1, 2);
        let indicator = AttackIndicator::get(packed_pos, 1.5);

        assert_eq!(indicator.pos(), packed_pos);
        assert_eq!(indicator.x(), -1);
        assert_eq!(indicator.y(), 2);
        assert_eq!(indicator.time(), 1.5);
        assert_eq!(
            u64::from(indicator),
            ((1.5f32.to_bits() as u64) << 32) | packed_pos as u32 as u64
        );

        let changed = indicator.with_time(2.25).with_pos(point2_pack(3, 4));
        assert_eq!(changed.pos(), point2_pack(3, 4));
        assert_eq!(changed.time(), 2.25);
    }

    #[test]
    fn add_resets_existing_indicator_time_or_appends_new_entry() {
        let mut indicators = AttackIndicators::new();
        indicators.add(1, 2);
        indicators.update(30.0);
        indicators.add(3, 4);
        indicators.add(1, 2);

        assert_eq!(indicators.len(), 2);
        assert_eq!(indicators.index_of(1, 2), Some(0));
        assert_eq!(indicators.index_of(3, 4), Some(1));
        assert_eq!(indicators.list()[0].time(), 0.0);
        assert_eq!(indicators.list()[1].time(), 0.0);
    }

    #[test]
    fn update_expires_entries_with_unordered_long_seq_swap_remove_semantics() {
        let mut indicators = AttackIndicators::new();
        indicators.add(1, 1);
        indicators.add(2, 2);
        indicators.add(3, 3);

        indicators.update(ATTACK_INDICATOR_DURATION - 1.0);
        indicators.add(2, 2);
        indicators.update(1.0);

        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators.list()[0].pos(), point2_pack(2, 2));
        assert_eq!(indicators.index_of(2, 2), Some(0));
        assert_eq!(indicators.index_of(1, 1), None);
        assert_eq!(indicators.index_of(3, 3), None);

        indicators.update(ATTACK_INDICATOR_DURATION);
        assert!(indicators.is_empty());
    }
}
