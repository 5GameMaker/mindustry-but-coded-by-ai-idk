//! Service statistic enum mirroring upstream `mindustry.service.SStat`.

pub trait StatService {
    fn get_stat(&self, _name: &str, def: i32) -> i32 {
        def
    }

    fn set_stat(&mut self, _name: &str, _amount: i32) {}

    fn store_stats(&mut self) {}
}

impl StatService for () {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SStat {
    UnitsDestroyed,
    AttacksWon,
    PvpsWon,
    TimesLaunched,
    BlocksDestroyed,
    ItemsLaunched,
    ReactorsOverheated,
    MaxUnitActive,
    UnitTypesBuilt,
    UnitsBuilt,
    BossesDefeated,
    MaxPlayersServer,
    MapsMade,
    MapsPublished,
    MaxWavesSurvived,
    BlocksBuilt,
    MaxProduction,
    SectorsControlled,
    SchematicsCreated,
    BouldersDeconstructed,
    TotalCampaignItems,
}

impl SStat {
    pub const ALL: [SStat; 21] = [
        SStat::UnitsDestroyed,
        SStat::AttacksWon,
        SStat::PvpsWon,
        SStat::TimesLaunched,
        SStat::BlocksDestroyed,
        SStat::ItemsLaunched,
        SStat::ReactorsOverheated,
        SStat::MaxUnitActive,
        SStat::UnitTypesBuilt,
        SStat::UnitsBuilt,
        SStat::BossesDefeated,
        SStat::MaxPlayersServer,
        SStat::MapsMade,
        SStat::MapsPublished,
        SStat::MaxWavesSurvived,
        SStat::BlocksBuilt,
        SStat::MaxProduction,
        SStat::SectorsControlled,
        SStat::SchematicsCreated,
        SStat::BouldersDeconstructed,
        SStat::TotalCampaignItems,
    ];

    pub fn name(self) -> &'static str {
        match self {
            SStat::UnitsDestroyed => "unitsDestroyed",
            SStat::AttacksWon => "attacksWon",
            SStat::PvpsWon => "pvpsWon",
            SStat::TimesLaunched => "timesLaunched",
            SStat::BlocksDestroyed => "blocksDestroyed",
            SStat::ItemsLaunched => "itemsLaunched",
            SStat::ReactorsOverheated => "reactorsOverheated",
            SStat::MaxUnitActive => "maxUnitActive",
            SStat::UnitTypesBuilt => "unitTypesBuilt",
            SStat::UnitsBuilt => "unitsBuilt",
            SStat::BossesDefeated => "bossesDefeated",
            SStat::MaxPlayersServer => "maxPlayersServer",
            SStat::MapsMade => "mapsMade",
            SStat::MapsPublished => "mapsPublished",
            SStat::MaxWavesSurvived => "maxWavesSurvived",
            SStat::BlocksBuilt => "blocksBuilt",
            SStat::MaxProduction => "maxProduction",
            SStat::SectorsControlled => "sectorsControlled",
            SStat::SchematicsCreated => "schematicsCreated",
            SStat::BouldersDeconstructed => "bouldersDeconstructed",
            SStat::TotalCampaignItems => "totalCampaignItems",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|stat| stat.name() == name)
    }

    pub fn get(self, service: &impl StatService) -> i32 {
        service.get_stat(self.name(), 0)
    }

    pub fn max(self, service: &mut impl StatService, amount: i32) {
        if amount > self.get(service) {
            self.set(service, amount);
        }
    }

    pub fn max_with_completion_check<S, F>(self, service: &mut S, amount: i32, check_completion: F)
    where
        S: StatService,
        F: FnMut(&mut S),
    {
        if amount > self.get(service) {
            self.set_with_completion_check(service, amount, check_completion);
        }
    }

    pub fn set(self, service: &mut impl StatService, amount: i32) {
        service.set_stat(self.name(), amount);
        service.store_stats();
    }

    pub fn set_with_completion_check<S, F>(
        self,
        service: &mut S,
        amount: i32,
        mut check_completion: F,
    ) where
        S: StatService,
        F: FnMut(&mut S),
    {
        self.set(service, amount);
        check_completion(service);
    }

    pub fn add(self, service: &mut impl StatService, amount: i32) {
        self.set(service, self.get(service) + amount);
    }

    pub fn add_with_completion_check<S, F>(self, service: &mut S, amount: i32, check_completion: F)
    where
        S: StatService,
        F: FnMut(&mut S),
    {
        self.set_with_completion_check(service, self.get(service) + amount, check_completion);
    }

    pub fn increment(self, service: &mut impl StatService) {
        self.add(service, 1);
    }

    pub fn increment_with_completion_check<S, F>(self, service: &mut S, check_completion: F)
    where
        S: StatService,
        F: FnMut(&mut S),
    {
        self.add_with_completion_check(service, 1, check_completion);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Debug, Default)]
    struct FakeService {
        stats: BTreeMap<String, i32>,
        stores: usize,
        calls: Vec<String>,
        checks: usize,
    }

    impl StatService for FakeService {
        fn get_stat(&self, name: &str, def: i32) -> i32 {
            self.stats.get(name).copied().unwrap_or(def)
        }

        fn set_stat(&mut self, name: &str, amount: i32) {
            self.calls.push(format!("set:{name}:{amount}"));
            self.stats.insert(name.into(), amount);
        }

        fn store_stats(&mut self) {
            self.calls.push("store".into());
            self.stores += 1;
        }
    }

    #[test]
    fn stat_order_and_names_match_upstream_enum() {
        let names: Vec<_> = SStat::ALL.iter().map(|stat| stat.name()).collect();

        assert_eq!(
            names,
            vec![
                "unitsDestroyed",
                "attacksWon",
                "pvpsWon",
                "timesLaunched",
                "blocksDestroyed",
                "itemsLaunched",
                "reactorsOverheated",
                "maxUnitActive",
                "unitTypesBuilt",
                "unitsBuilt",
                "bossesDefeated",
                "maxPlayersServer",
                "mapsMade",
                "mapsPublished",
                "maxWavesSurvived",
                "blocksBuilt",
                "maxProduction",
                "sectorsControlled",
                "schematicsCreated",
                "bouldersDeconstructed",
                "totalCampaignItems",
            ]
        );
        assert_eq!(
            SStat::from_name("maxProduction"),
            Some(SStat::MaxProduction)
        );
        assert_eq!(SStat::from_name("missing"), None);
    }

    #[test]
    fn default_service_returns_zero_like_java_default_stat_read() {
        assert_eq!(SStat::UnitsDestroyed.get(&()), 0);
    }

    #[test]
    fn set_add_and_increment_store_stats_like_java() {
        let mut service = FakeService::default();

        SStat::ItemsLaunched.set(&mut service, 10);
        SStat::ItemsLaunched.add(&mut service, 5);
        SStat::ItemsLaunched.increment(&mut service);

        assert_eq!(SStat::ItemsLaunched.get(&service), 16);
        assert_eq!(service.stores, 3);
        assert_eq!(
            service.calls,
            vec![
                "set:itemsLaunched:10",
                "store",
                "set:itemsLaunched:15",
                "store",
                "set:itemsLaunched:16",
                "store",
            ]
        );
    }

    #[test]
    fn max_only_writes_when_amount_exceeds_current_value() {
        let mut service = FakeService::default();
        SStat::MaxWavesSurvived.set(&mut service, 30);
        service.calls.clear();

        SStat::MaxWavesSurvived.max(&mut service, 20);
        SStat::MaxWavesSurvived.max(&mut service, 100);

        assert_eq!(SStat::MaxWavesSurvived.get(&service), 100);
        assert_eq!(service.calls, vec!["set:maxWavesSurvived:100", "store"]);
    }

    #[test]
    fn completion_check_hook_runs_after_store_to_mirror_achievement_all_scan() {
        let mut service = FakeService::default();

        SStat::BlocksBuilt.set_with_completion_check(&mut service, 5, |service| {
            service.calls.push("check".into());
            service.checks += 1;
        });

        assert_eq!(service.checks, 1);
        assert_eq!(service.calls, vec!["set:blocksBuilt:5", "store", "check"]);
    }
}
