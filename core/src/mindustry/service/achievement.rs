//! Achievement enum and completion rules mirroring upstream `mindustry.service.Achievement`.

use std::collections::BTreeSet;

use super::{SStat, StatService};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AchievementData {
    pub achievement: Achievement,
    pub name: &'static str,
    pub stat: Option<SStat>,
    pub stat_goal: i32,
}

impl AchievementData {
    pub const fn event(achievement: Achievement, name: &'static str) -> Self {
        Self {
            achievement,
            name,
            stat: None,
            stat_goal: 0,
        }
    }

    pub const fn stat(
        achievement: Achievement,
        name: &'static str,
        stat: SStat,
        stat_goal: i32,
    ) -> Self {
        Self {
            achievement,
            name,
            stat: Some(stat),
            stat_goal,
        }
    }
}

pub trait AchievementService: StatService {
    fn complete_achievement(&mut self, _name: &str) {}

    fn clear_achievement(&mut self, _name: &str) {}

    fn is_achieved(&self, _name: &str) -> bool {
        false
    }
}

impl AchievementService for () {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AchievementContext {
    pub console_shown: bool,
    pub username_is_anuke: bool,
}

impl AchievementContext {
    pub fn normal() -> Self {
        Self {
            console_shown: false,
            username_is_anuke: false,
        }
    }

    pub fn with_console(console_shown: bool, username: &str) -> Self {
        Self {
            console_shown,
            username_is_anuke: username == "anuke",
        }
    }

    pub fn allows_completion(self, achievement: Achievement) -> bool {
        !self.console_shown || self.username_is_anuke || achievement == Achievement::OpenConsole
    }
}

impl Default for AchievementContext {
    fn default() -> Self {
        Self::normal()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AchievementState {
    completed: BTreeSet<Achievement>,
}

impl AchievementState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_cached(&self, achievement: Achievement) -> bool {
        self.completed.contains(&achievement)
    }

    pub fn complete<S: AchievementService>(
        &mut self,
        achievement: Achievement,
        service: &mut S,
        context: AchievementContext,
    ) {
        if !self.is_achieved(achievement, service) {
            // Java refuses normal achievements while the dev console is shown,
            // except for the console-opening achievement itself and the author.
            if !context.allows_completion(achievement) {
                return;
            }

            service.complete_achievement(achievement.name());
            service.store_stats();
        }
    }

    pub fn uncomplete<S: AchievementService>(&mut self, achievement: Achievement, service: &mut S) {
        if self.is_achieved(achievement, service) {
            service.clear_achievement(achievement.name());
            self.completed.remove(&achievement);
        }
    }

    pub fn check_completion<S: AchievementService>(
        &mut self,
        achievement: Achievement,
        service: &mut S,
        context: AchievementContext,
    ) {
        let data = achievement.data();
        if !self.is_achieved(achievement, service)
            && data
                .stat
                .map(|stat| stat.get(service) >= data.stat_goal)
                .unwrap_or(false)
        {
            self.complete(achievement, service, context);
        }
    }

    pub fn check_all_completions<S: AchievementService>(
        &mut self,
        service: &mut S,
        context: AchievementContext,
    ) {
        for achievement in Achievement::ALL {
            self.check_completion(achievement, service, context);
        }
    }

    pub fn is_achieved<S: AchievementService>(
        &mut self,
        achievement: Achievement,
        service: &S,
    ) -> bool {
        if self.completed.contains(&achievement) {
            return true;
        }
        if service.is_achieved(achievement.name()) {
            self.completed.insert(achievement);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(usize)]
pub enum Achievement {
    Kill1kEnemies,
    Kill100kEnemies,
    Launch100kItems,
    Produce5kMin,
    Produce50kMin,
    Win10Attack,
    Win10PvP,
    DefeatAttack5Waves,
    Launch30Times,
    CaptureBackground,
    Survive100Waves,
    ResearchAll,
    ShockWetEnemy,
    KillEnemyPhaseWall,
    ResearchRouter,
    Place10kBlocks,
    Destroy1kBlocks,
    OverheatReactor,
    Make10maps,
    DownloadMapWorkshop,
    PublishMap,
    DefeatBoss,
    CaptureAllSectors,
    Control10Sectors,
    Drop10kitems,
    PowerupImpactReactor,
    ObtainThorium,
    ObtainTitanium,
    SuicideBomb,
    BuildGroundFactory,
    IssueAttackCommand,
    Active100Units,
    Build1000Units,
    BuildAllUnits,
    BuildT5,
    PickupT5,
    Active10Polys,
    DieExclusion,
    Drown,
    FillCoreAllCampaign,
    HostServer10,
    BuildMeltdownSpectre,
    LaunchItemPad,
    ChainRouters,
    CircleConveyor,
    BecomeRouter,
    Create20Schematics,
    Create500Schematics,
    Survive10WavesNoBlocks,
    CaptureNoBlocksBroken,
    UseFlameAmmo,
    CoolTurret,
    EnablePixelation,
    OpenWiki,
    AllTransportOneMap,
    BuildOverdriveProjector,
    BuildMendProjector,
    BuildWexWater,
    Have10mItems,
    KillEclipseDuo,
    KillMassDriver,
    CompleteErekir,
    CompleteSerpulo,
    LaunchCoreSchematic,
    NucleusGroundZero,
    NeoplasmWater,
    BlastFrozenUnit,
    AllBlocksSerpulo,
    AllBlocksErekir,
    BreakForceProjector,
    ResearchLogic,
    Negative10kPower,
    Positive100kPower,
    Store1milPower,
    BlastGenerator,
    NeoplasiaExplosion,
    InstallMod,
    RouterLanguage,
    JoinCommunityServer,
    OpenConsole,
    ControlTurret,
    DropUnitsCoreZone,
    DestroyScatterFlare,
    BoostUnit,
    BoostBuildingFloor,
    HoverUnitLiquid,
    Break100Boulders,
    Break10000Boulders,
    ShockwaveTowerUse,
    UseAnimdustryEmoji,
}

impl Achievement {
    pub const ALL: [Achievement; 90] = [
        Achievement::Kill1kEnemies,
        Achievement::Kill100kEnemies,
        Achievement::Launch100kItems,
        Achievement::Produce5kMin,
        Achievement::Produce50kMin,
        Achievement::Win10Attack,
        Achievement::Win10PvP,
        Achievement::DefeatAttack5Waves,
        Achievement::Launch30Times,
        Achievement::CaptureBackground,
        Achievement::Survive100Waves,
        Achievement::ResearchAll,
        Achievement::ShockWetEnemy,
        Achievement::KillEnemyPhaseWall,
        Achievement::ResearchRouter,
        Achievement::Place10kBlocks,
        Achievement::Destroy1kBlocks,
        Achievement::OverheatReactor,
        Achievement::Make10maps,
        Achievement::DownloadMapWorkshop,
        Achievement::PublishMap,
        Achievement::DefeatBoss,
        Achievement::CaptureAllSectors,
        Achievement::Control10Sectors,
        Achievement::Drop10kitems,
        Achievement::PowerupImpactReactor,
        Achievement::ObtainThorium,
        Achievement::ObtainTitanium,
        Achievement::SuicideBomb,
        Achievement::BuildGroundFactory,
        Achievement::IssueAttackCommand,
        Achievement::Active100Units,
        Achievement::Build1000Units,
        Achievement::BuildAllUnits,
        Achievement::BuildT5,
        Achievement::PickupT5,
        Achievement::Active10Polys,
        Achievement::DieExclusion,
        Achievement::Drown,
        Achievement::FillCoreAllCampaign,
        Achievement::HostServer10,
        Achievement::BuildMeltdownSpectre,
        Achievement::LaunchItemPad,
        Achievement::ChainRouters,
        Achievement::CircleConveyor,
        Achievement::BecomeRouter,
        Achievement::Create20Schematics,
        Achievement::Create500Schematics,
        Achievement::Survive10WavesNoBlocks,
        Achievement::CaptureNoBlocksBroken,
        Achievement::UseFlameAmmo,
        Achievement::CoolTurret,
        Achievement::EnablePixelation,
        Achievement::OpenWiki,
        Achievement::AllTransportOneMap,
        Achievement::BuildOverdriveProjector,
        Achievement::BuildMendProjector,
        Achievement::BuildWexWater,
        Achievement::Have10mItems,
        Achievement::KillEclipseDuo,
        Achievement::KillMassDriver,
        Achievement::CompleteErekir,
        Achievement::CompleteSerpulo,
        Achievement::LaunchCoreSchematic,
        Achievement::NucleusGroundZero,
        Achievement::NeoplasmWater,
        Achievement::BlastFrozenUnit,
        Achievement::AllBlocksSerpulo,
        Achievement::AllBlocksErekir,
        Achievement::BreakForceProjector,
        Achievement::ResearchLogic,
        Achievement::Negative10kPower,
        Achievement::Positive100kPower,
        Achievement::Store1milPower,
        Achievement::BlastGenerator,
        Achievement::NeoplasiaExplosion,
        Achievement::InstallMod,
        Achievement::RouterLanguage,
        Achievement::JoinCommunityServer,
        Achievement::OpenConsole,
        Achievement::ControlTurret,
        Achievement::DropUnitsCoreZone,
        Achievement::DestroyScatterFlare,
        Achievement::BoostUnit,
        Achievement::BoostBuildingFloor,
        Achievement::HoverUnitLiquid,
        Achievement::Break100Boulders,
        Achievement::Break10000Boulders,
        Achievement::ShockwaveTowerUse,
        Achievement::UseAnimdustryEmoji,
    ];

    pub fn data(self) -> &'static AchievementData {
        &ACHIEVEMENT_DATA[self as usize]
    }

    pub fn name(self) -> &'static str {
        self.data().name
    }

    pub fn stat(self) -> Option<SStat> {
        self.data().stat
    }

    pub fn stat_goal(self) -> i32 {
        self.data().stat_goal
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|achievement| achievement.name() == name)
    }
}

pub const ACHIEVEMENT_DATA: [AchievementData; 90] = [
    AchievementData::stat(
        Achievement::Kill1kEnemies,
        "kill1kEnemies",
        SStat::UnitsDestroyed,
        1000,
    ),
    AchievementData::stat(
        Achievement::Kill100kEnemies,
        "kill100kEnemies",
        SStat::UnitsDestroyed,
        100_000,
    ),
    AchievementData::stat(
        Achievement::Launch100kItems,
        "launch100kItems",
        SStat::ItemsLaunched,
        100_000,
    ),
    AchievementData::stat(
        Achievement::Produce5kMin,
        "produce5kMin",
        SStat::MaxProduction,
        5000,
    ),
    AchievementData::stat(
        Achievement::Produce50kMin,
        "produce50kMin",
        SStat::MaxProduction,
        50_000,
    ),
    AchievementData::stat(
        Achievement::Win10Attack,
        "win10Attack",
        SStat::AttacksWon,
        10,
    ),
    AchievementData::stat(Achievement::Win10PvP, "win10PvP", SStat::PvpsWon, 10),
    AchievementData::event(Achievement::DefeatAttack5Waves, "defeatAttack5Waves"),
    AchievementData::stat(
        Achievement::Launch30Times,
        "launch30Times",
        SStat::TimesLaunched,
        30,
    ),
    AchievementData::event(Achievement::CaptureBackground, "captureBackground"),
    AchievementData::stat(
        Achievement::Survive100Waves,
        "survive100Waves",
        SStat::MaxWavesSurvived,
        100,
    ),
    AchievementData::event(Achievement::ResearchAll, "researchAll"),
    AchievementData::event(Achievement::ShockWetEnemy, "shockWetEnemy"),
    AchievementData::event(Achievement::KillEnemyPhaseWall, "killEnemyPhaseWall"),
    AchievementData::event(Achievement::ResearchRouter, "researchRouter"),
    AchievementData::stat(
        Achievement::Place10kBlocks,
        "place10kBlocks",
        SStat::BlocksBuilt,
        10_000,
    ),
    AchievementData::stat(
        Achievement::Destroy1kBlocks,
        "destroy1kBlocks",
        SStat::BlocksDestroyed,
        1000,
    ),
    AchievementData::stat(
        Achievement::OverheatReactor,
        "overheatReactor",
        SStat::ReactorsOverheated,
        1,
    ),
    AchievementData::stat(Achievement::Make10maps, "make10maps", SStat::MapsMade, 10),
    AchievementData::event(Achievement::DownloadMapWorkshop, "downloadMapWorkshop"),
    AchievementData::stat(
        Achievement::PublishMap,
        "publishMap",
        SStat::MapsPublished,
        1,
    ),
    AchievementData::stat(
        Achievement::DefeatBoss,
        "defeatBoss",
        SStat::BossesDefeated,
        1,
    ),
    AchievementData::event(Achievement::CaptureAllSectors, "captureAllSectors"),
    AchievementData::stat(
        Achievement::Control10Sectors,
        "control10Sectors",
        SStat::SectorsControlled,
        10,
    ),
    AchievementData::event(Achievement::Drop10kitems, "drop10kitems"),
    AchievementData::event(Achievement::PowerupImpactReactor, "powerupImpactReactor"),
    AchievementData::event(Achievement::ObtainThorium, "obtainThorium"),
    AchievementData::event(Achievement::ObtainTitanium, "obtainTitanium"),
    AchievementData::event(Achievement::SuicideBomb, "suicideBomb"),
    AchievementData::event(Achievement::BuildGroundFactory, "buildGroundFactory"),
    AchievementData::event(Achievement::IssueAttackCommand, "issueAttackCommand"),
    AchievementData::stat(
        Achievement::Active100Units,
        "active100Units",
        SStat::MaxUnitActive,
        100,
    ),
    AchievementData::stat(
        Achievement::Build1000Units,
        "build1000Units",
        SStat::UnitsBuilt,
        1000,
    ),
    AchievementData::stat(
        Achievement::BuildAllUnits,
        "buildAllUnits",
        SStat::UnitTypesBuilt,
        50,
    ),
    AchievementData::event(Achievement::BuildT5, "buildT5"),
    AchievementData::event(Achievement::PickupT5, "pickupT5"),
    AchievementData::event(Achievement::Active10Polys, "active10Polys"),
    AchievementData::event(Achievement::DieExclusion, "dieExclusion"),
    AchievementData::event(Achievement::Drown, "drown"),
    AchievementData::event(Achievement::FillCoreAllCampaign, "fillCoreAllCampaign"),
    AchievementData::stat(
        Achievement::HostServer10,
        "hostServer10",
        SStat::MaxPlayersServer,
        10,
    ),
    AchievementData::event(Achievement::BuildMeltdownSpectre, "buildMeltdownSpectre"),
    AchievementData::event(Achievement::LaunchItemPad, "launchItemPad"),
    AchievementData::event(Achievement::ChainRouters, "chainRouters"),
    AchievementData::event(Achievement::CircleConveyor, "circleConveyor"),
    AchievementData::event(Achievement::BecomeRouter, "becomeRouter"),
    AchievementData::stat(
        Achievement::Create20Schematics,
        "create20Schematics",
        SStat::SchematicsCreated,
        20,
    ),
    AchievementData::stat(
        Achievement::Create500Schematics,
        "create500Schematics",
        SStat::SchematicsCreated,
        500,
    ),
    AchievementData::event(
        Achievement::Survive10WavesNoBlocks,
        "survive10WavesNoBlocks",
    ),
    AchievementData::event(Achievement::CaptureNoBlocksBroken, "captureNoBlocksBroken"),
    AchievementData::event(Achievement::UseFlameAmmo, "useFlameAmmo"),
    AchievementData::event(Achievement::CoolTurret, "coolTurret"),
    AchievementData::event(Achievement::EnablePixelation, "enablePixelation"),
    AchievementData::event(Achievement::OpenWiki, "openWiki"),
    AchievementData::event(Achievement::AllTransportOneMap, "allTransportOneMap"),
    AchievementData::event(
        Achievement::BuildOverdriveProjector,
        "buildOverdriveProjector",
    ),
    AchievementData::event(Achievement::BuildMendProjector, "buildMendProjector"),
    AchievementData::event(Achievement::BuildWexWater, "buildWexWater"),
    AchievementData::stat(
        Achievement::Have10mItems,
        "have10mItems",
        SStat::TotalCampaignItems,
        10_000_000,
    ),
    AchievementData::event(Achievement::KillEclipseDuo, "killEclipseDuo"),
    AchievementData::event(Achievement::KillMassDriver, "killMassDriver"),
    AchievementData::event(Achievement::CompleteErekir, "completeErekir"),
    AchievementData::event(Achievement::CompleteSerpulo, "completeSerpulo"),
    AchievementData::event(Achievement::LaunchCoreSchematic, "launchCoreSchematic"),
    AchievementData::event(Achievement::NucleusGroundZero, "nucleusGroundZero"),
    AchievementData::event(Achievement::NeoplasmWater, "neoplasmWater"),
    AchievementData::event(Achievement::BlastFrozenUnit, "blastFrozenUnit"),
    AchievementData::event(Achievement::AllBlocksSerpulo, "allBlocksSerpulo"),
    AchievementData::event(Achievement::AllBlocksErekir, "allBlocksErekir"),
    AchievementData::event(Achievement::BreakForceProjector, "breakForceProjector"),
    AchievementData::event(Achievement::ResearchLogic, "researchLogic"),
    AchievementData::event(Achievement::Negative10kPower, "negative10kPower"),
    AchievementData::event(Achievement::Positive100kPower, "positive100kPower"),
    AchievementData::event(Achievement::Store1milPower, "store1milPower"),
    AchievementData::event(Achievement::BlastGenerator, "blastGenerator"),
    AchievementData::event(Achievement::NeoplasiaExplosion, "neoplasiaExplosion"),
    AchievementData::event(Achievement::InstallMod, "installMod"),
    AchievementData::event(Achievement::RouterLanguage, "routerLanguage"),
    AchievementData::event(Achievement::JoinCommunityServer, "joinCommunityServer"),
    AchievementData::event(Achievement::OpenConsole, "openConsole"),
    AchievementData::event(Achievement::ControlTurret, "controlTurret"),
    AchievementData::event(Achievement::DropUnitsCoreZone, "dropUnitsCoreZone"),
    AchievementData::event(Achievement::DestroyScatterFlare, "destroyScatterFlare"),
    AchievementData::event(Achievement::BoostUnit, "boostUnit"),
    AchievementData::event(Achievement::BoostBuildingFloor, "boostBuildingFloor"),
    AchievementData::event(Achievement::HoverUnitLiquid, "hoverUnitLiquid"),
    AchievementData::stat(
        Achievement::Break100Boulders,
        "break100Boulders",
        SStat::BouldersDeconstructed,
        100,
    ),
    AchievementData::stat(
        Achievement::Break10000Boulders,
        "break10000Boulders",
        SStat::BouldersDeconstructed,
        10_000,
    ),
    AchievementData::event(Achievement::ShockwaveTowerUse, "shockwaveTowerUse"),
    AchievementData::event(Achievement::UseAnimdustryEmoji, "useAnimdustryEmoji"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    #[derive(Debug, Default)]
    struct FakeService {
        stats: BTreeMap<String, i32>,
        achievements: BTreeSet<String>,
        calls: Vec<String>,
        stores: usize,
    }

    impl StatService for FakeService {
        fn get_stat(&self, name: &str, def: i32) -> i32 {
            self.stats.get(name).copied().unwrap_or(def)
        }

        fn set_stat(&mut self, name: &str, amount: i32) {
            self.stats.insert(name.into(), amount);
        }

        fn store_stats(&mut self) {
            self.calls.push("store".into());
            self.stores += 1;
        }
    }

    impl AchievementService for FakeService {
        fn complete_achievement(&mut self, name: &str) {
            self.calls.push(format!("complete:{name}"));
            self.achievements.insert(name.into());
        }

        fn clear_achievement(&mut self, name: &str) {
            self.calls.push(format!("clear:{name}"));
            self.achievements.remove(name);
        }

        fn is_achieved(&self, name: &str) -> bool {
            self.achievements.contains(name)
        }
    }

    #[test]
    fn achievement_order_and_names_match_upstream_enum() {
        let names: Vec<_> = Achievement::ALL
            .iter()
            .map(|achievement| achievement.name())
            .collect();

        assert_eq!(names.len(), 90);
        assert_eq!(names[0], "kill1kEnemies");
        assert_eq!(names[2], "launch100kItems");
        assert_eq!(names[41], "buildMeltdownSpectre");
        assert_eq!(names[42], "launchItemPad");
        assert_eq!(names[79], "openConsole");
        assert_eq!(names[89], "useAnimdustryEmoji");
        assert_eq!(
            Achievement::from_name("break10000Boulders"),
            Some(Achievement::Break10000Boulders)
        );
        assert_eq!(Achievement::from_name("missing"), None);
    }

    #[test]
    fn stat_backed_achievement_goals_match_java_constructors() {
        assert_eq!(
            Achievement::Kill1kEnemies.stat(),
            Some(SStat::UnitsDestroyed)
        );
        assert_eq!(Achievement::Kill1kEnemies.stat_goal(), 1000);
        assert_eq!(
            Achievement::Have10mItems.stat(),
            Some(SStat::TotalCampaignItems)
        );
        assert_eq!(Achievement::Have10mItems.stat_goal(), 10_000_000);
        assert_eq!(
            Achievement::Break10000Boulders.stat(),
            Some(SStat::BouldersDeconstructed)
        );
        assert_eq!(Achievement::Break10000Boulders.stat_goal(), 10_000);
        assert_eq!(Achievement::OpenConsole.stat(), None);
    }

    #[test]
    fn complete_calls_service_and_store_once_when_not_already_achieved() {
        let mut service = FakeService::default();
        let mut state = AchievementState::new();

        state.complete(
            Achievement::OpenWiki,
            &mut service,
            AchievementContext::normal(),
        );
        state.complete(
            Achievement::OpenWiki,
            &mut service,
            AchievementContext::normal(),
        );

        assert!(state.is_achieved(Achievement::OpenWiki, &service));
        assert_eq!(service.stores, 1);
        assert_eq!(service.calls, vec!["complete:openWiki", "store"]);
    }

    #[test]
    fn console_blocks_normal_achievements_but_not_open_console_or_anuke() {
        let mut service = FakeService::default();
        let mut state = AchievementState::new();

        state.complete(
            Achievement::OpenWiki,
            &mut service,
            AchievementContext::with_console(true, "player"),
        );
        state.complete(
            Achievement::OpenConsole,
            &mut service,
            AchievementContext::with_console(true, "player"),
        );
        state.complete(
            Achievement::RouterLanguage,
            &mut service,
            AchievementContext::with_console(true, "anuke"),
        );

        assert!(!service.achievements.contains("openWiki"));
        assert!(service.achievements.contains("openConsole"));
        assert!(service.achievements.contains("routerLanguage"));
    }

    #[test]
    fn check_completion_only_completes_stat_backed_thresholds() {
        let mut service = FakeService::default();
        let mut state = AchievementState::new();
        service.stats.insert("unitsDestroyed".into(), 1000);

        state.check_completion(
            Achievement::Kill1kEnemies,
            &mut service,
            AchievementContext::normal(),
        );
        state.check_completion(
            Achievement::Kill100kEnemies,
            &mut service,
            AchievementContext::normal(),
        );
        state.check_completion(
            Achievement::OpenWiki,
            &mut service,
            AchievementContext::normal(),
        );

        assert!(service.achievements.contains("kill1kEnemies"));
        assert!(!service.achievements.contains("kill100kEnemies"));
        assert!(!service.achievements.contains("openWiki"));
    }

    #[test]
    fn uncomplete_clears_remote_and_local_cache_like_java() {
        let mut service = FakeService::default();
        service.achievements.insert("openWiki".into());
        let mut state = AchievementState::new();
        assert!(state.is_achieved(Achievement::OpenWiki, &service));

        state.uncomplete(Achievement::OpenWiki, &mut service);

        assert!(!state.is_cached(Achievement::OpenWiki));
        assert!(!service.achievements.contains("openWiki"));
        assert_eq!(service.calls, vec!["clear:openWiki"]);
    }
}
