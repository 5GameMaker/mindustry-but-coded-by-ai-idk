use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CampaignStats {
    pub enemy_units_destroyed: BTreeMap<String, i32>,
    pub enemy_buildings_destroyed: BTreeMap<String, i32>,
    pub units_produced: BTreeMap<String, i32>,
    pub units_destroyed: BTreeMap<String, i32>,
    pub buildings_built: BTreeMap<String, i32>,
    pub buildings_deconstructed: BTreeMap<String, i32>,
    pub buildings_destroyed: BTreeMap<String, i32>,
    pub playtime: i64,
    pub sectors_lost: i32,
    pub sectors_captured: i32,
    pub waves_lasted: i32,
}

impl CampaignStats {
    pub fn add_enemy_unit_destroyed(&mut self, unit: impl Into<String>, amount: i32) {
        add_count(&mut self.enemy_units_destroyed, unit, amount);
    }

    pub fn add_enemy_building_destroyed(&mut self, block: impl Into<String>, amount: i32) {
        add_count(&mut self.enemy_buildings_destroyed, block, amount);
    }

    pub fn add_unit_produced(&mut self, unit: impl Into<String>, amount: i32) {
        add_count(&mut self.units_produced, unit, amount);
    }

    pub fn add_unit_destroyed(&mut self, unit: impl Into<String>, amount: i32) {
        add_count(&mut self.units_destroyed, unit, amount);
    }

    pub fn add_building_built(&mut self, block: impl Into<String>, amount: i32) {
        add_count(&mut self.buildings_built, block, amount);
    }

    pub fn add_building_deconstructed(&mut self, block: impl Into<String>, amount: i32) {
        add_count(&mut self.buildings_deconstructed, block, amount);
    }

    pub fn add_building_destroyed(&mut self, block: impl Into<String>, amount: i32) {
        add_count(&mut self.buildings_destroyed, block, amount);
    }

    pub fn get_enemy_unit_destroyed(&self, unit: &str) -> i32 {
        get_count(&self.enemy_units_destroyed, unit)
    }

    pub fn get_enemy_building_destroyed(&self, block: &str) -> i32 {
        get_count(&self.enemy_buildings_destroyed, block)
    }

    pub fn get_unit_produced(&self, unit: &str) -> i32 {
        get_count(&self.units_produced, unit)
    }

    pub fn get_unit_destroyed(&self, unit: &str) -> i32 {
        get_count(&self.units_destroyed, unit)
    }

    pub fn get_building_built(&self, block: &str) -> i32 {
        get_count(&self.buildings_built, block)
    }

    pub fn get_building_deconstructed(&self, block: &str) -> i32 {
        get_count(&self.buildings_deconstructed, block)
    }

    pub fn get_building_destroyed(&self, block: &str) -> i32 {
        get_count(&self.buildings_destroyed, block)
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
    fn campaign_stats_defaults_match_java_zero_initializers() {
        let stats = CampaignStats::default();
        assert!(stats.enemy_units_destroyed.is_empty());
        assert!(stats.enemy_buildings_destroyed.is_empty());
        assert!(stats.units_produced.is_empty());
        assert!(stats.units_destroyed.is_empty());
        assert!(stats.buildings_built.is_empty());
        assert!(stats.buildings_deconstructed.is_empty());
        assert!(stats.buildings_destroyed.is_empty());
        assert_eq!(stats.playtime, 0);
        assert_eq!(stats.sectors_lost, 0);
        assert_eq!(stats.sectors_captured, 0);
        assert_eq!(stats.waves_lasted, 0);
    }

    #[test]
    fn campaign_stats_count_maps_accumulate_like_object_int_map() {
        let mut stats = CampaignStats::default();
        stats.add_enemy_unit_destroyed("dagger", 2);
        stats.add_enemy_unit_destroyed("dagger", 3);
        stats.add_enemy_building_destroyed("duo", 4);
        stats.add_unit_produced("flare", 5);
        stats.add_unit_destroyed("mono", 6);
        stats.add_building_built("router", 7);
        stats.add_building_deconstructed("conveyor", 8);
        stats.add_building_destroyed("core-shard", 9);

        assert_eq!(stats.get_enemy_unit_destroyed("dagger"), 5);
        assert_eq!(stats.get_enemy_unit_destroyed("missing"), 0);
        assert_eq!(stats.get_enemy_building_destroyed("duo"), 4);
        assert_eq!(stats.get_unit_produced("flare"), 5);
        assert_eq!(stats.get_unit_destroyed("mono"), 6);
        assert_eq!(stats.get_building_built("router"), 7);
        assert_eq!(stats.get_building_deconstructed("conveyor"), 8);
        assert_eq!(stats.get_building_destroyed("core-shard"), 9);
    }
}
