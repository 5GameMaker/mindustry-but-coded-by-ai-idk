use std::collections::BTreeMap;

use super::{Attribute, Stat, StatCat, StatUnit, StatValue, StatValues};

#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    pub use_categories: bool,
    pub initialized: bool,
    pub time_period: f32,
    map: BTreeMap<StatCat, BTreeMap<Stat, Vec<StatValue>>>,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            use_categories: false,
            initialized: false,
            time_period: -1.0,
            map: BTreeMap::new(),
        }
    }
}

impl Stats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_value(&mut self, stat: Stat, value: StatValue) {
        self.map
            .entry(stat.category())
            .or_default()
            .entry(stat)
            .or_default()
            .push(value);
    }

    pub fn add_number(&mut self, stat: Stat, value: f32, unit: StatUnit) {
        self.add_value(stat, StatValues::number(value, unit));
    }

    pub fn add_number_none(&mut self, stat: Stat, value: f32) {
        self.add_number(stat, value, StatUnit::None);
    }

    pub fn add_percent(&mut self, stat: Stat, value: f32) {
        self.add_value(
            stat,
            StatValues::number((value * 100.0) as i32 as f32, StatUnit::Percent),
        );
    }

    pub fn add_mult_modifier(&mut self, stat: Stat, value: f32) {
        self.add_value(stat, StatValues::multiplier_modifier(value));
    }

    pub fn add_percent_modifier(&mut self, stat: Stat, value: f32) {
        self.add_value(stat, StatValues::percent_modifier(value));
    }

    pub fn add_bool(&mut self, stat: Stat, value: bool) {
        self.add_value(stat, StatValues::bool(value));
    }

    pub fn add_item(&mut self, stat: Stat, item: impl Into<String>, amount: i32) {
        self.add_value(stat, StatValues::item(item, amount));
    }

    pub fn add_liquid(
        &mut self,
        stat: Stat,
        liquid: impl Into<String>,
        amount: f32,
        per_second: bool,
    ) {
        self.add_value(stat, StatValues::liquid(liquid, amount, per_second));
    }

    pub fn add_attribute(&mut self, stat: Stat, attr: &Attribute) {
        self.add_attribute_scaled(stat, attr, false, 1.0, false);
    }

    pub fn add_attribute_scaled(
        &mut self,
        stat: Stat,
        attr: &Attribute,
        floating: bool,
        scale: f32,
        start_zero: bool,
    ) {
        self.add_value(stat, StatValues::blocks(attr, floating, scale, start_zero));
    }

    pub fn add_string(&mut self, stat: Stat, value: impl Into<String>) {
        self.add_value(stat, StatValues::string(value));
    }

    pub fn add_string_args(
        &mut self,
        stat: Stat,
        value: impl AsRef<str>,
        args: impl IntoIterator<Item = impl ToString>,
    ) {
        self.add_value(stat, StatValues::string_args(value, args));
    }

    pub fn replace(&mut self, stat: Stat, value: StatValue) {
        self.remove(stat);
        self.add_value(stat, value);
    }

    pub fn remove(&mut self, stat: Stat) -> Option<Vec<StatValue>> {
        self.map.entry(stat.category()).or_default().remove(&stat)
    }

    pub fn values(&self, stat: Stat) -> Option<&[StatValue]> {
        self.map
            .get(&stat.category())
            .and_then(|stats| stats.get(&stat))
            .map(Vec::as_slice)
    }

    pub fn to_map(&self) -> Vec<(StatCat, Vec<(Stat, Vec<StatValue>)>)> {
        self.map
            .iter()
            .map(|(cat, stats)| {
                (
                    *cat,
                    stats
                        .iter()
                        .map(|(stat, values)| (*stat, values.clone()))
                        .collect(),
                )
            })
            .collect()
    }

    pub fn stat_info_key(stat: Stat, has_info: bool) -> Option<String> {
        if has_info {
            Some(format!("@{}.info", stat.bundle_key()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_add_replace_remove_and_sort_by_java_category_and_stat_order() {
        let mut stats = Stats::new();
        assert!(!stats.use_categories);
        assert!(!stats.initialized);
        assert_eq!(stats.time_period, -1.0);

        stats.add_bool(Stat::TargetsAir, true);
        stats.add_number(Stat::Health, 100.0, StatUnit::None);
        stats.add_percent(Stat::Flammability, 0.456);
        stats.add_item(Stat::ItemCapacity, "copper", 5);
        stats.add_liquid(Stat::LiquidCapacity, "water", 12.0, false);
        stats.add_string(Stat::Instructions, "configure");

        let map = stats.to_map();
        assert_eq!(
            map.iter().map(|(cat, _)| *cat).collect::<Vec<_>>(),
            vec![
                StatCat::General,
                StatCat::Liquids,
                StatCat::Items,
                StatCat::Crafting,
                StatCat::Function
            ]
        );
        assert_eq!(
            map[0].1.iter().map(|(stat, _)| *stat).collect::<Vec<_>>(),
            vec![Stat::Health, Stat::Flammability]
        );
        assert_eq!(
            stats.values(Stat::Flammability).unwrap()[0],
            StatValue::Number {
                value: 45.0,
                unit: StatUnit::Percent,
                merge: false
            }
        );
        stats.replace(Stat::Health, StatValues::number(200.0, StatUnit::None));
        assert_eq!(stats.values(Stat::Health).unwrap().len(), 1);
        assert_eq!(
            stats.values(Stat::Health).unwrap()[0],
            StatValue::Number {
                value: 200.0,
                unit: StatUnit::None,
                merge: false
            }
        );
        assert!(stats.remove(Stat::TargetsAir).is_some());
        assert!(stats.values(Stat::TargetsAir).is_none());
        assert!(
            stats
                .to_map()
                .iter()
                .any(|(cat, entries)| *cat == StatCat::Function && entries.is_empty()),
            "Java Stats.remove() leaves an empty category map instead of pruning it"
        );
        assert_eq!(
            Stats::stat_info_key(Stat::BuildTime, true).unwrap(),
            "@stat.buildtime.info"
        );
        assert_eq!(Stats::stat_info_key(Stat::BuildTime, false), None);
    }

    #[test]
    fn stats_remove_missing_stat_creates_empty_category_like_java() {
        let mut stats = Stats::new();

        assert!(stats.remove(Stat::Explosiveness).is_none());

        assert_eq!(stats.to_map(), vec![(StatCat::General, Vec::new())]);
    }

    #[test]
    fn stats_add_string_args_delegates_to_stat_values_formatting() {
        let mut stats = Stats::new();

        stats.add_string_args(Stat::Armor, "armor @ / @", [3, 5]);

        assert_eq!(
            stats.values(Stat::Armor).unwrap()[0],
            StatValue::Text("armor 3 / 5".into())
        );
    }
}
