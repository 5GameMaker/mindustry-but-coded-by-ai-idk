use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GameStats {
    pub enemy_units_destroyed: i32,
    pub waves_lasted: i32,
    pub buildings_built: i32,
    pub buildings_deconstructed: i32,
    pub buildings_destroyed: i32,
    pub units_created: i32,
    pub placed_block_count: BTreeMap<String, i32>,
    pub destroyed_block_count: BTreeMap<String, i32>,
    pub core_item_count: BTreeMap<String, i32>,
}

impl GameStats {
    pub fn get_placed(&self, block: &str) -> i32 {
        get_count(&self.placed_block_count, block)
    }

    pub fn get_destroyed(&self, block: &str) -> i32 {
        get_count(&self.destroyed_block_count, block)
    }

    pub fn get_core_item(&self, item: &str) -> i32 {
        get_count(&self.core_item_count, item)
    }

    pub fn add_placed(&mut self, block: impl Into<String>, amount: i32) {
        add_count(&mut self.placed_block_count, block, amount);
        self.buildings_built += amount;
    }

    pub fn add_destroyed(&mut self, block: impl Into<String>, amount: i32) {
        add_count(&mut self.destroyed_block_count, block, amount);
        self.buildings_destroyed += amount;
    }

    pub fn add_core_item(&mut self, item: impl Into<String>, amount: i32) {
        add_count(&mut self.core_item_count, item, amount);
    }
}

fn add_count(map: &mut BTreeMap<String, i32>, key: impl Into<String>, amount: i32) {
    *map.entry(key.into()).or_insert(0) += amount;
}

fn get_count(map: &BTreeMap<String, i32>, key: &str) -> i32 {
    map.get(key).copied().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_stats_defaults_match_java_zero_initializers() {
        let stats = GameStats::default();
        assert_eq!(stats.enemy_units_destroyed, 0);
        assert_eq!(stats.waves_lasted, 0);
        assert_eq!(stats.buildings_built, 0);
        assert_eq!(stats.buildings_deconstructed, 0);
        assert_eq!(stats.buildings_destroyed, 0);
        assert_eq!(stats.units_created, 0);
        assert!(stats.placed_block_count.is_empty());
        assert!(stats.destroyed_block_count.is_empty());
        assert!(stats.core_item_count.is_empty());
    }

    #[test]
    fn game_stats_getters_return_zero_for_missing_keys_like_java_object_int_map() {
        let mut stats = GameStats::default();
        assert_eq!(stats.get_placed("router"), 0);
        assert_eq!(stats.get_destroyed("duo"), 0);
        assert_eq!(stats.get_core_item("copper"), 0);

        stats.add_placed("router", 2);
        stats.add_placed("router", 3);
        stats.add_destroyed("duo", 4);
        stats.add_core_item("copper", 5);

        assert_eq!(stats.get_placed("router"), 5);
        assert_eq!(stats.get_destroyed("duo"), 4);
        assert_eq!(stats.get_core_item("copper"), 5);
        assert_eq!(stats.buildings_built, 5);
        assert_eq!(stats.buildings_destroyed, 4);
    }
}
