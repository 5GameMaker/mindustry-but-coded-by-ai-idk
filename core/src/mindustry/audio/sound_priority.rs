//! Sound priority tuning plan mirroring upstream `mindustry.audio.SoundPriority`.
//!
//! Java mutates Arc `Sound` objects directly. This Rust version records the
//! same mutations as a deterministic plan so platform audio backends can apply
//! them once generated sound assets exist.

#[derive(Debug, Clone, PartialEq)]
pub struct SoundAssetLength {
    pub name: String,
    pub length: f32,
}

impl SoundAssetLength {
    pub fn new(name: impl Into<String>, length: f32) -> Self {
        Self {
            name: name.into(),
            length,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundFloatSetting {
    pub sound: String,
    pub value: f32,
}

impl SoundFloatSetting {
    pub fn new(sound: impl Into<String>, value: f32) -> Self {
        Self {
            sound: sound.into(),
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundIntSetting {
    pub sound: String,
    pub value: i32,
}

impl SoundIntSetting {
    pub fn new(sound: impl Into<String>, value: i32) -> Self {
        Self {
            sound: sound.into(),
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundGroupSetting {
    pub sound: String,
    pub group: i32,
}

impl SoundGroupSetting {
    pub fn new(sound: impl Into<String>, group: i32) -> Self {
        Self {
            sound: sound.into(),
            group,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SoundPriorityPlan {
    pub ui_bus_sounds: Vec<String>,
    pub priorities: Vec<SoundFloatSetting>,
    pub max_concurrent: Vec<SoundIntSetting>,
    pub concurrent_groups: Vec<SoundGroupSetting>,
    pub min_concurrent_interrupts: Vec<SoundFloatSetting>,
    pub falloff_offsets: Vec<SoundFloatSetting>,
}

impl SoundPriorityPlan {
    pub fn priority_for(&self, sound: &str) -> Option<f32> {
        latest_float(&self.priorities, sound)
    }

    pub fn max_concurrent_for(&self, sound: &str) -> Option<i32> {
        self.max_concurrent
            .iter()
            .rev()
            .find(|setting| setting.sound == sound)
            .map(|setting| setting.value)
    }

    pub fn concurrent_group_for(&self, sound: &str) -> Option<i32> {
        self.concurrent_groups
            .iter()
            .rev()
            .find(|setting| setting.sound == sound)
            .map(|setting| setting.group)
    }

    pub fn min_concurrent_interrupt_for(&self, sound: &str) -> Option<f32> {
        latest_float(&self.min_concurrent_interrupts, sound)
    }

    pub fn falloff_offset_for(&self, sound: &str) -> Option<f32> {
        latest_float(&self.falloff_offsets, sound)
    }

    pub fn uses_ui_bus(&self, sound: &str) -> bool {
        self.ui_bus_sounds
            .iter()
            .any(|candidate| candidate == sound)
    }
}

pub struct SoundPriority;

impl SoundPriority {
    pub fn init_plan(assets: impl IntoIterator<Item = SoundAssetLength>) -> SoundPriorityPlan {
        let mut plan = SoundPriorityPlan::default();

        // launching should not get interrupted by the loading screen
        plan.ui_bus_sounds.push("coreLaunch".into());

        max(
            &mut plan,
            7,
            &["beamPlasma", "shootMeltdown", "beamMeltdown"],
        );

        // priority 3: absolutely do not interrupt these
        set(
            &mut plan,
            3.0,
            &[
                "acceleratorLaunch",
                "acceleratorCharge",
                "coreLand",
                "coreLaunch",
            ],
        );

        // priority 2: long weapon loops and big explosions
        set(
            &mut plan,
            2.0,
            &[
                "beamMeltdown",
                "beamLustre",
                "beamPlasma",
                "explosionReactor",
                "explosionReactor2",
                "explosionReactorNeoplasm",
                "explosionCore",
                "blockExplodeElectricBig",
                "blockExplodeExplosive",
                "blockExplodeExplosiveAlt",
            ],
        );

        // priority 1.5: big weapon sounds, not loops
        set(
            &mut plan,
            1.5,
            &[
                "shootMeltdown",
                "shootSublimate",
                "shootForeshadow",
                "shootConquer",
                "shootCorvus",
                "chargeCorvus",
                "chargeVela",
                "chargeLancer",
                "shootReign",
                "shootEclipse",
                "shootArtillerySapBig",
                "shootToxopidShotgun",
                "beamPlasmaSmall",
                "shootNavanax",
                "explosionNavanax",
            ],
        );

        // priority 1: ambient noises
        set(
            &mut plan,
            1.0,
            &[
                "loopConveyor",
                "loopSmelter",
                "loopDrill",
                "loopExtract",
                "loopFlux",
                "loopHum",
                "loopBio",
                "loopTech",
                "loopUnitBuilding",
            ],
        );

        // very loud
        max(&mut plan, 5, &["shootLancer"]);

        let mut last_group = 1;
        same_group(
            &mut plan,
            &mut last_group,
            &["shootFlame", "shootFlamePlasma"],
        );
        same_group(
            &mut plan,
            &mut last_group,
            &[
                "shootMissile",
                "shootMissileShort",
                "shootMissilePlasmaShort",
            ],
        );
        same_group(&mut plan, &mut last_group, &["shootArc", "shootPulsar"]);

        for asset in assets {
            plan.min_concurrent_interrupts.push(SoundFloatSetting::new(
                asset.name,
                asset.length.mul_add(0.5, 0.0).min(0.25),
            ));
        }

        set_min_interrupt(&mut plan, 0.5, &["mechStepSmall", "mechStep"]);
        set_min_interrupt(&mut plan, 0.6, &["walkerStep", "mechStepHeavy"]);

        max(&mut plan, 4, &["shieldHit"]);

        max(
            &mut plan,
            5,
            &[
                "mechStep",
                "mechStepHeavy",
                "walkerStep",
                "walkerStepSmall",
                "walkerStepTiny",
            ],
        );

        // repair sounds are lower priority and generally not important
        set(&mut plan, -1.0, &["blockHeal", "healWave"]);

        // step sounds are low priority
        set(
            &mut plan,
            -2.0,
            &[
                "mechStep",
                "mechStepHeavy",
                "walkerStep",
                "walkerStepSmall",
                "walkerStepTiny",
                "mechStepSmall",
            ],
        );

        set_falloff(&mut plan, 100.0, &["explosionCore"]);
        set_falloff(&mut plan, 70.0, &["blockExplodeElectricBig"]);

        plan
    }
}

fn max(plan: &mut SoundPriorityPlan, value: i32, sounds: &[&str]) {
    plan.max_concurrent.extend(
        sounds
            .iter()
            .map(|sound| SoundIntSetting::new(*sound, value)),
    );
}

fn same_group(plan: &mut SoundPriorityPlan, last_group: &mut i32, sounds: &[&str]) {
    let id = *last_group;
    *last_group += 1;
    plan.concurrent_groups.extend(
        sounds
            .iter()
            .map(|sound| SoundGroupSetting::new(*sound, id)),
    );
}

fn set(plan: &mut SoundPriorityPlan, value: f32, sounds: &[&str]) {
    plan.priorities.extend(
        sounds
            .iter()
            .map(|sound| SoundFloatSetting::new(*sound, value)),
    );
}

fn set_min_interrupt(plan: &mut SoundPriorityPlan, value: f32, sounds: &[&str]) {
    plan.min_concurrent_interrupts.extend(
        sounds
            .iter()
            .map(|sound| SoundFloatSetting::new(*sound, value)),
    );
}

fn set_falloff(plan: &mut SoundPriorityPlan, value: f32, sounds: &[&str]) {
    plan.falloff_offsets.extend(
        sounds
            .iter()
            .map(|sound| SoundFloatSetting::new(*sound, value)),
    );
}

fn latest_float(settings: &[SoundFloatSetting], sound: &str) -> Option<f32> {
    settings
        .iter()
        .rev()
        .find(|setting| setting.sound == sound)
        .map(|setting| setting.value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> SoundPriorityPlan {
        SoundPriority::init_plan([
            SoundAssetLength::new("short", 0.1),
            SoundAssetLength::new("long", 3.0),
            SoundAssetLength::new("mechStep", 0.1),
        ])
    }

    #[test]
    fn priority_plan_keeps_upstream_priority_bands_and_late_overrides() {
        let plan = plan();

        assert_eq!(plan.priority_for("acceleratorLaunch"), Some(3.0));
        assert_eq!(plan.priority_for("coreLaunch"), Some(3.0));
        assert_eq!(plan.priority_for("beamMeltdown"), Some(2.0));
        assert_eq!(plan.priority_for("explosionCore"), Some(2.0));
        assert_eq!(plan.priority_for("shootForeshadow"), Some(1.5));
        assert_eq!(plan.priority_for("loopDrill"), Some(1.0));
        assert_eq!(plan.priority_for("blockHeal"), Some(-1.0));
        assert_eq!(plan.priority_for("mechStep"), Some(-2.0));
    }

    #[test]
    fn max_concurrent_and_falloff_offsets_match_java_mutations() {
        let plan = plan();

        assert!(plan.uses_ui_bus("coreLaunch"));
        assert_eq!(plan.max_concurrent_for("beamPlasma"), Some(7));
        assert_eq!(plan.max_concurrent_for("shootMeltdown"), Some(7));
        assert_eq!(plan.max_concurrent_for("shootLancer"), Some(5));
        assert_eq!(plan.max_concurrent_for("shieldHit"), Some(4));
        assert_eq!(plan.max_concurrent_for("mechStep"), Some(5));
        assert_eq!(plan.falloff_offset_for("explosionCore"), Some(100.0));
        assert_eq!(
            plan.falloff_offset_for("blockExplodeElectricBig"),
            Some(70.0)
        );
    }

    #[test]
    fn same_group_assigns_incrementing_group_ids_from_one() {
        let plan = plan();

        assert_eq!(plan.concurrent_group_for("shootFlame"), Some(1));
        assert_eq!(plan.concurrent_group_for("shootFlamePlasma"), Some(1));
        assert_eq!(plan.concurrent_group_for("shootMissile"), Some(2));
        assert_eq!(plan.concurrent_group_for("shootMissileShort"), Some(2));
        assert_eq!(
            plan.concurrent_group_for("shootMissilePlasmaShort"),
            Some(2)
        );
        assert_eq!(plan.concurrent_group_for("shootArc"), Some(3));
        assert_eq!(plan.concurrent_group_for("shootPulsar"), Some(3));
    }

    #[test]
    fn min_concurrent_interrupts_use_asset_lengths_then_step_overrides() {
        let plan = plan();

        assert_eq!(plan.min_concurrent_interrupt_for("short"), Some(0.05));
        assert_eq!(plan.min_concurrent_interrupt_for("long"), Some(0.25));
        assert_eq!(plan.min_concurrent_interrupt_for("mechStep"), Some(0.5));
        assert_eq!(
            plan.min_concurrent_interrupt_for("mechStepHeavy"),
            Some(0.6)
        );
        assert_eq!(plan.min_concurrent_interrupt_for("walkerStep"), Some(0.6));
    }
}
