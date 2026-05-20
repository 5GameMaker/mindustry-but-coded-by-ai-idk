use std::collections::{BTreeSet, HashMap};

use crate::mindustry::r#type::ItemStack;

pub const VALUE_WINDOW: usize = 60;
pub const REFRESH_PERIOD: f32 = 60.0;

#[derive(Debug, Clone, PartialEq)]
pub struct SectorImportSnapshot {
    pub id: String,
    pub has_base: bool,
    pub destination: Option<String>,
    pub export: HashMap<String, ExportStat>,
}

impl SectorImportSnapshot {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            has_base: false,
            destination: None,
            export: HashMap::new(),
        }
    }

    pub fn with_base(mut self, has_base: bool) -> Self {
        self.has_base = has_base;
        self
    }

    pub fn with_destination(mut self, destination: impl Into<String>) -> Self {
        self.destination = Some(destination.into());
        self
    }

    pub fn with_export(mut self, item: impl Into<String>, mean: f32) -> Self {
        self.export.insert(
            item.into(),
            ExportStat {
                mean,
                ..ExportStat::new()
            },
        );
        self
    }

    pub fn any_exports(&self) -> bool {
        any_exports_in(&self.export)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportStat {
    pub counter: f32,
    pub means: Vec<f32>,
    pub loaded: bool,
    pub mean: f32,
}

impl ExportStat {
    pub fn new() -> Self {
        Self {
            counter: 0.0,
            means: Vec::new(),
            loaded: false,
            mean: 0.0,
        }
    }

    pub fn record_counter_sample(&mut self) {
        let sample = self.counter.max(0.0);
        self.counter = 0.0;
        self.record_sample(sample);
    }

    pub fn record_sample(&mut self, sample: f32) {
        self.ensure_loaded();
        if self.means.len() == VALUE_WINDOW {
            self.means.remove(0);
        }
        self.means.push(sample);
        self.mean = raw_mean(&self.means);
    }

    fn ensure_loaded(&mut self) {
        if !self.loaded {
            self.means = vec![self.mean; VALUE_WINDOW];
            self.loaded = true;
        }
    }
}

impl Default for ExportStat {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectorInfo {
    pub production: HashMap<String, ExportStat>,
    pub raw_production: HashMap<String, ExportStat>,
    pub export: HashMap<String, ExportStat>,
    pub imports: HashMap<String, ExportStat>,
    pub items: Vec<(String, i32)>,
    pub best_core_type: String,
    pub storage_capacity: i32,
    pub has_core: bool,
    pub last_preset_name: Option<String>,
    pub last_width: i32,
    pub last_height: i32,
    pub was_captured: bool,
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub resources: Vec<String>,
    pub waves: bool,
    pub attack: bool,
    pub has_spawns: bool,
    pub attempts: i32,
    pub wave: i32,
    pub win_wave: i32,
    pub wave_spacing: f32,
    pub spawn_position: i32,
    pub minutes_captured: f32,
    pub light_coverage: f32,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub content_icon: Option<String>,
    pub wave_version: i32,
    pub shown: bool,
    pub import_cooldown_timers: HashMap<String, f32>,
    pub import_rate_cache: Option<Vec<f32>>,
    pub last_imported: Vec<(String, i32)>,
    pub core_deltas: HashMap<String, i32>,
    pub production_deltas: HashMap<String, i32>,
}

impl Default for SectorInfo {
    fn default() -> Self {
        Self {
            production: HashMap::new(),
            raw_production: HashMap::new(),
            export: HashMap::new(),
            imports: HashMap::new(),
            items: Vec::new(),
            best_core_type: "core-shard".to_string(),
            storage_capacity: 0,
            has_core: true,
            last_preset_name: None,
            last_width: 0,
            last_height: 0,
            was_captured: false,
            origin: None,
            destination: None,
            resources: Vec::new(),
            waves: true,
            attack: false,
            has_spawns: true,
            attempts: 0,
            wave: 1,
            win_wave: -1,
            wave_spacing: 2.0 * 60.0 * 60.0,
            spawn_position: 0,
            minutes_captured: 0.0,
            light_coverage: 0.0,
            name: None,
            icon: None,
            content_icon: None,
            wave_version: -1,
            shown: false,
            import_cooldown_timers: HashMap::new(),
            import_rate_cache: None,
            last_imported: Vec::new(),
            core_deltas: HashMap::new(),
            production_deltas: HashMap::new(),
        }
    }
}

impl SectorInfo {
    pub fn sector_data_matches(&self, preset: Option<(&str, i32, i32)>) -> bool {
        match preset {
            Some((name, width, height)) => {
                if width != self.last_width || height != self.last_height {
                    return false;
                }
                self.last_preset_name.as_deref() == Some(name)
            }
            None => self.last_preset_name.is_none(),
        }
    }

    pub fn handle_core_item(&mut self, item: impl Into<String>, amount: i32) {
        *self.core_deltas.entry(item.into()).or_insert(0) += amount;
    }

    pub fn handle_production(&mut self, item: impl Into<String>, amount: i32) {
        *self.production_deltas.entry(item.into()).or_insert(0) += amount;
    }

    pub fn handle_item_export(&mut self, stack: &ItemStack) {
        self.handle_item_export_amount(stack.item.clone(), stack.amount);
    }

    pub fn handle_item_export_amount(&mut self, item: impl Into<String>, amount: i32) {
        self.export.entry(item.into()).or_default().counter += amount as f32;
    }

    pub fn handle_item_import(&mut self, item: impl Into<String>, amount: i32) {
        self.imports.entry(item.into()).or_default().counter += amount as f32;
    }

    pub fn get_export(&self, item: &str) -> f32 {
        self.export.get(item).map_or(0.0, |stat| stat.mean)
    }

    pub fn has_export(&self, item: &str) -> bool {
        self.export.get(item).is_some_and(|stat| stat.mean > 0.0)
    }

    pub fn export_rates(&self) -> HashMap<String, f32> {
        self.export
            .iter()
            .map(|(item, stat)| (item.clone(), stat.mean))
            .collect()
    }

    pub fn any_exports(&self) -> bool {
        any_exports_in(&self.export)
    }

    pub fn imported_sector_ids<'a>(
        &self,
        self_id: &str,
        sectors: &'a [SectorImportSnapshot],
    ) -> Vec<&'a str> {
        sectors
            .iter()
            .filter(|sector| {
                sector.has_base
                    && sector.id != self_id
                    && sector.destination.as_deref() == Some(self_id)
                    && sector.any_exports()
            })
            .map(|sector| sector.id.as_str())
            .collect()
    }

    pub fn import_stats_from(
        &self,
        self_id: &str,
        sectors: &[SectorImportSnapshot],
    ) -> HashMap<String, ExportStat> {
        let mut imports: HashMap<String, ExportStat> = HashMap::new();
        for sector in sectors.iter().filter(|sector| {
            sector.has_base
                && sector.id != self_id
                && sector.destination.as_deref() == Some(self_id)
                && sector.any_exports()
        }) {
            for (item, stat) in &sector.export {
                imports.entry(item.clone()).or_default().mean += stat.mean;
            }
        }
        imports
    }

    pub fn refresh_import_rates<I, S>(
        &mut self,
        self_id: &str,
        items: I,
        sectors: &[SectorImportSnapshot],
    ) where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let names: Vec<String> = items
            .into_iter()
            .map(|item| item.as_ref().to_string())
            .collect();
        let mut cache = vec![0.0; names.len()];
        let by_name: HashMap<&str, usize> = names
            .iter()
            .enumerate()
            .map(|(index, name)| (name.as_str(), index))
            .collect();

        for sector in sectors.iter().filter(|sector| {
            sector.has_base
                && sector.id != self_id
                && sector.destination.as_deref() == Some(self_id)
                && sector.any_exports()
        }) {
            for (item, stat) in &sector.export {
                if let Some(index) = by_name.get(item.as_str()) {
                    cache[*index] += stat.mean;
                }
            }
        }

        self.import_rate_cache = Some(cache);
    }

    pub fn get_import_rates<I, S>(
        &mut self,
        self_id: &str,
        items: I,
        sectors: &[SectorImportSnapshot],
    ) -> &[f32]
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if self.import_rate_cache.is_none() {
            self.refresh_import_rates(self_id, items, sectors);
        }
        self.import_rate_cache.as_deref().unwrap_or(&[])
    }

    pub fn get_import_rate<I, S>(
        &mut self,
        self_id: &str,
        items: I,
        sectors: &[SectorImportSnapshot],
        item: &str,
    ) -> f32
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let names: Vec<String> = items
            .into_iter()
            .map(|item| item.as_ref().to_string())
            .collect();
        let index = names.iter().position(|name| name == item);
        let rates = self.get_import_rates(self_id, names.iter().map(String::as_str), sectors);
        index
            .and_then(|index| rates.get(index).copied())
            .unwrap_or(0.0)
    }

    pub fn refresh_throughput<I, S>(&mut self, known_items: I, import_rates: &HashMap<String, f32>)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        update_stats(&mut self.export);
        update_stats(&mut self.imports);

        let mut item_names = BTreeSet::new();
        for item in known_items {
            item_names.insert(item.as_ref().to_string());
        }
        item_names.extend(self.core_deltas.keys().cloned());
        item_names.extend(self.production_deltas.keys().cloned());
        item_names.extend(self.production.keys().cloned());
        item_names.extend(self.raw_production.keys().cloned());
        item_names.extend(self.export.keys().cloned());
        item_names.extend(self.imports.keys().cloned());

        for item in item_names {
            let production_mean = update_delta(&mut self.production, &self.core_deltas, &item);
            let raw_mean = update_delta(&mut self.raw_production, &self.production_deltas, &item);
            let capped_production = production_mean.min(raw_mean);
            if let Some(stat) = self.production.get_mut(&item) {
                stat.mean = capped_production;
            }

            if let Some(stat) = self.export.get_mut(&item) {
                stat.mean = stat.mean.min(raw_mean + (-capped_production).max(0.0));
            }

            if let Some(stat) = self.imports.get_mut(&item) {
                if let Some(max_rate) = import_rates.get(&item) {
                    stat.mean = stat.mean.min(*max_rate);
                }
            }
        }

        self.core_deltas.clear();
        self.production_deltas.clear();
    }

    pub fn cap_production_at_raw(&mut self) {
        let items: Vec<String> = self.production.keys().cloned().collect();
        for item in items {
            let raw = self.raw_production.entry(item.clone()).or_default().mean;
            if let Some(stat) = self.production.get_mut(&item) {
                stat.mean = stat.mean.min(raw);
            }
        }
    }
}

fn update_stats(map: &mut HashMap<String, ExportStat>) {
    for stat in map.values_mut() {
        stat.record_counter_sample();
    }
}

fn update_delta(
    map: &mut HashMap<String, ExportStat>,
    deltas: &HashMap<String, i32>,
    item: &str,
) -> f32 {
    let sample = deltas.get(item).copied().unwrap_or(0) as f32;
    let stat = map.entry(item.to_string()).or_default();
    stat.record_sample(sample);
    stat.mean
}

fn raw_mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

fn any_exports_in(export: &HashMap<String, ExportStat>) -> bool {
    !export.is_empty() && export.values().map(|stat| stat.mean).sum::<f32>() >= 0.01
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx_eq(left: f32, right: f32) {
        assert!((left - right).abs() < 0.00001, "left={left}, right={right}");
    }

    #[test]
    fn defaults_match_upstream_sector_info_initializers() {
        let info = SectorInfo::default();
        assert!(info.production.is_empty());
        assert!(info.raw_production.is_empty());
        assert!(info.export.is_empty());
        assert!(info.imports.is_empty());
        assert!(info.items.is_empty());
        assert_eq!(info.best_core_type, "core-shard");
        assert_eq!(info.storage_capacity, 0);
        assert!(info.has_core);
        assert_eq!(info.last_preset_name, None);
        assert!(!info.was_captured);
        assert_eq!(info.origin, None);
        assert_eq!(info.destination, None);
        assert!(info.resources.is_empty());
        assert!(info.waves);
        assert!(!info.attack);
        assert!(info.has_spawns);
        assert_eq!(info.attempts, 0);
        assert_eq!(info.wave, 1);
        assert_eq!(info.win_wave, -1);
        assert_eq!(info.wave_spacing, 2.0 * 60.0 * 60.0);
        assert_eq!(info.spawn_position, 0);
        assert_eq!(info.minutes_captured, 0.0);
        assert_eq!(info.light_coverage, 0.0);
        assert_eq!(info.name, None);
        assert_eq!(info.icon, None);
        assert_eq!(info.content_icon, None);
        assert_eq!(info.wave_version, -1);
        assert!(!info.shown);
        assert!(info.import_cooldown_timers.is_empty());
        assert_eq!(info.import_rate_cache, None);
        assert!(info.last_imported.is_empty());
    }

    #[test]
    fn sector_data_matches_uses_preset_name_and_dimensions_like_java() {
        let mut info = SectorInfo {
            last_preset_name: Some("craters".into()),
            last_width: 128,
            last_height: 256,
            ..SectorInfo::default()
        };

        assert!(info.sector_data_matches(Some(("craters", 128, 256))));
        assert!(!info.sector_data_matches(Some(("craters", 127, 256))));
        assert!(!info.sector_data_matches(Some(("other", 128, 256))));
        assert!(!info.sector_data_matches(None));

        info.last_preset_name = None;
        assert!(info.sector_data_matches(None));
    }

    #[test]
    fn export_stat_initializes_loaded_window_before_sampling() {
        let mut stat = ExportStat {
            mean: 60.0,
            counter: 120.0,
            ..ExportStat::new()
        };

        stat.record_counter_sample();

        assert!(stat.loaded);
        assert_eq!(stat.means.len(), VALUE_WINDOW);
        assert_eq!(stat.counter, 0.0);
        assert_approx_eq(stat.mean, 61.0);
    }

    #[test]
    fn throughput_refresh_matches_java_counter_delta_caps() {
        let mut info = SectorInfo::default();
        info.handle_core_item("copper", 120);
        info.handle_production("copper", 60);
        info.handle_item_export(&ItemStack::new("copper", 90));
        info.handle_item_import("lead", 50);

        let import_rates = HashMap::from([(String::from("lead"), 0.5)]);
        info.refresh_throughput(["copper", "lead"], &import_rates);

        assert_approx_eq(info.raw_production["copper"].mean, 1.0);
        assert_approx_eq(info.production["copper"].mean, 1.0);
        assert_approx_eq(info.get_export("copper"), 1.0);
        assert!(info.has_export("copper"));
        assert!(info.any_exports());
        assert_approx_eq(info.imports["lead"].mean, 0.5);
        assert!(info.core_deltas.is_empty());
        assert!(info.production_deltas.is_empty());

        let rates = info.export_rates();
        assert_approx_eq(rates["copper"], 1.0);
    }

    #[test]
    fn export_can_be_limited_by_items_removed_from_core() {
        let mut info = SectorInfo::default();
        info.handle_core_item("copper", -30);
        info.handle_item_export_amount("copper", 60);

        info.refresh_throughput(["copper"], &HashMap::new());

        assert_approx_eq(info.production["copper"].mean, -0.5);
        assert_approx_eq(info.raw_production["copper"].mean, 0.0);
        assert_approx_eq(info.get_export("copper"), 0.5);
    }

    #[test]
    fn import_snapshot_filters_sources_like_each_import() {
        let info = SectorInfo::default();
        let sectors = vec![
            SectorImportSnapshot::new("source-a")
                .with_base(true)
                .with_destination("target")
                .with_export("copper", 1.0),
            SectorImportSnapshot::new("source-b")
                .with_base(true)
                .with_destination("target")
                .with_export("lead", 0.009),
            SectorImportSnapshot::new("source-c")
                .with_base(false)
                .with_destination("target")
                .with_export("graphite", 2.0),
            SectorImportSnapshot::new("target")
                .with_base(true)
                .with_destination("target")
                .with_export("silicon", 3.0),
            SectorImportSnapshot::new("other")
                .with_base(true)
                .with_destination("elsewhere")
                .with_export("titanium", 4.0),
        ];

        assert_eq!(
            info.imported_sector_ids("target", &sectors),
            vec!["source-a"]
        );
    }

    #[test]
    fn import_stats_and_rate_cache_sum_matching_source_exports() {
        let mut info = SectorInfo::default();
        let sectors = vec![
            SectorImportSnapshot::new("source-a")
                .with_base(true)
                .with_destination("target")
                .with_export("copper", 1.0)
                .with_export("lead", 2.0),
            SectorImportSnapshot::new("source-b")
                .with_base(true)
                .with_destination("target")
                .with_export("copper", 3.0),
            SectorImportSnapshot::new("source-c")
                .with_base(true)
                .with_destination("target")
                .with_export("graphite", 0.0),
        ];

        let stats = info.import_stats_from("target", &sectors);
        assert_approx_eq(stats["copper"].mean, 4.0);
        assert_approx_eq(stats["lead"].mean, 2.0);
        assert!(!stats.contains_key("graphite"));

        info.refresh_import_rates("target", ["copper", "lead", "graphite"], &sectors);
        assert_eq!(
            info.import_rate_cache.as_ref().unwrap(),
            &vec![4.0, 2.0, 0.0]
        );
        assert_approx_eq(
            info.get_import_rate("target", ["copper", "lead", "graphite"], &sectors, "lead"),
            2.0,
        );
    }
}
