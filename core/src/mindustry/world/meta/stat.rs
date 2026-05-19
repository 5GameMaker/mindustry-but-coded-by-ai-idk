use super::StatCat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub enum Stat {
    Health,
    Armor,
    Size,
    DisplaySize,
    BuildTime,
    BuildCost,
    MemoryCapacity,
    Explosiveness,
    Flammability,
    Radioactivity,
    Charge,
    HeatCapacity,
    Viscosity,
    Temperature,
    Flying,
    Speed,
    BuildSpeed,
    MineSpeed,
    MineTier,
    PayloadCapacity,
    BaseDeflectChance,
    LightningChance,
    LightningDamage,
    Abilities,
    CanBoost,
    BoostingSpeed,
    MaxUnits,
    DamageMultiplier,
    HealthMultiplier,
    SpeedMultiplier,
    ReloadMultiplier,
    BuildSpeedMultiplier,
    Reactive,
    Healing,
    Immunities,
    ItemCapacity,
    ItemsMoved,
    LaunchTime,
    MaxConsecutive,
    LiquidCapacity,
    PowerCapacity,
    PowerUse,
    PowerDamage,
    PowerRange,
    PowerConnections,
    BasePowerGeneration,
    WarmupTime,
    Tiles,
    Input,
    Output,
    ProductionTime,
    MaxEfficiency,
    DrillTier,
    DrillSpeed,
    LinkRange,
    Instructions,
    Weapons,
    Bullet,
    SpeedIncrease,
    RepairTime,
    RepairSpeed,
    Range,
    ShootRange,
    Inaccuracy,
    Shots,
    Reload,
    CrushDamage,
    LegSplashDamage,
    TargetsAir,
    TargetsGround,
    Damage,
    Frequency,
    Ammo,
    AmmoCapacity,
    AmmoUse,
    ShieldHealth,
    CooldownTime,
    RegenerationRate,
    ActivationTime,
    ModuleTier,
    UnitType,
    Booster,
    BoostEffect,
    Affinities,
    Opposites,
}

impl Stat {
    pub const ALL: [Stat; 85] = [
        Stat::Health,
        Stat::Armor,
        Stat::Size,
        Stat::DisplaySize,
        Stat::BuildTime,
        Stat::BuildCost,
        Stat::MemoryCapacity,
        Stat::Explosiveness,
        Stat::Flammability,
        Stat::Radioactivity,
        Stat::Charge,
        Stat::HeatCapacity,
        Stat::Viscosity,
        Stat::Temperature,
        Stat::Flying,
        Stat::Speed,
        Stat::BuildSpeed,
        Stat::MineSpeed,
        Stat::MineTier,
        Stat::PayloadCapacity,
        Stat::BaseDeflectChance,
        Stat::LightningChance,
        Stat::LightningDamage,
        Stat::Abilities,
        Stat::CanBoost,
        Stat::BoostingSpeed,
        Stat::MaxUnits,
        Stat::DamageMultiplier,
        Stat::HealthMultiplier,
        Stat::SpeedMultiplier,
        Stat::ReloadMultiplier,
        Stat::BuildSpeedMultiplier,
        Stat::Reactive,
        Stat::Healing,
        Stat::Immunities,
        Stat::ItemCapacity,
        Stat::ItemsMoved,
        Stat::LaunchTime,
        Stat::MaxConsecutive,
        Stat::LiquidCapacity,
        Stat::PowerCapacity,
        Stat::PowerUse,
        Stat::PowerDamage,
        Stat::PowerRange,
        Stat::PowerConnections,
        Stat::BasePowerGeneration,
        Stat::WarmupTime,
        Stat::Tiles,
        Stat::Input,
        Stat::Output,
        Stat::ProductionTime,
        Stat::MaxEfficiency,
        Stat::DrillTier,
        Stat::DrillSpeed,
        Stat::LinkRange,
        Stat::Instructions,
        Stat::Weapons,
        Stat::Bullet,
        Stat::SpeedIncrease,
        Stat::RepairTime,
        Stat::RepairSpeed,
        Stat::Range,
        Stat::ShootRange,
        Stat::Inaccuracy,
        Stat::Shots,
        Stat::Reload,
        Stat::CrushDamage,
        Stat::LegSplashDamage,
        Stat::TargetsAir,
        Stat::TargetsGround,
        Stat::Damage,
        Stat::Frequency,
        Stat::Ammo,
        Stat::AmmoCapacity,
        Stat::AmmoUse,
        Stat::ShieldHealth,
        Stat::CooldownTime,
        Stat::RegenerationRate,
        Stat::ActivationTime,
        Stat::ModuleTier,
        Stat::UnitType,
        Stat::Booster,
        Stat::BoostEffect,
        Stat::Affinities,
        Stat::Opposites,
    ];

    pub const fn id(self) -> usize {
        self as usize
    }

    pub const fn name(self) -> &'static str {
        match self {
            Stat::Health => "health",
            Stat::Armor => "armor",
            Stat::Size => "size",
            Stat::DisplaySize => "displaySize",
            Stat::BuildTime => "buildTime",
            Stat::BuildCost => "buildCost",
            Stat::MemoryCapacity => "memoryCapacity",
            Stat::Explosiveness => "explosiveness",
            Stat::Flammability => "flammability",
            Stat::Radioactivity => "radioactivity",
            Stat::Charge => "charge",
            Stat::HeatCapacity => "heatCapacity",
            Stat::Viscosity => "viscosity",
            Stat::Temperature => "temperature",
            Stat::Flying => "flying",
            Stat::Speed => "speed",
            Stat::BuildSpeed => "buildSpeed",
            Stat::MineSpeed => "mineSpeed",
            Stat::MineTier => "mineTier",
            Stat::PayloadCapacity => "payloadCapacity",
            Stat::BaseDeflectChance => "baseDeflectChance",
            Stat::LightningChance => "lightningChance",
            Stat::LightningDamage => "lightningDamage",
            Stat::Abilities => "abilities",
            Stat::CanBoost => "canBoost",
            Stat::BoostingSpeed => "boostingspeed",
            Stat::MaxUnits => "maxUnits",
            Stat::DamageMultiplier => "damageMultiplier",
            Stat::HealthMultiplier => "healthMultiplier",
            Stat::SpeedMultiplier => "speedMultiplier",
            Stat::ReloadMultiplier => "reloadMultiplier",
            Stat::BuildSpeedMultiplier => "buildSpeedMultiplier",
            Stat::Reactive => "reactive",
            Stat::Healing => "healing",
            Stat::Immunities => "immunities",
            Stat::ItemCapacity => "itemCapacity",
            Stat::ItemsMoved => "itemsMoved",
            Stat::LaunchTime => "launchTime",
            Stat::MaxConsecutive => "maxConsecutive",
            Stat::LiquidCapacity => "liquidCapacity",
            Stat::PowerCapacity => "powerCapacity",
            Stat::PowerUse => "powerUse",
            Stat::PowerDamage => "powerDamage",
            Stat::PowerRange => "powerRange",
            Stat::PowerConnections => "powerConnections",
            Stat::BasePowerGeneration => "basePowerGeneration",
            Stat::WarmupTime => "warmupTime",
            Stat::Tiles => "tiles",
            Stat::Input => "input",
            Stat::Output => "output",
            Stat::ProductionTime => "productionTime",
            Stat::MaxEfficiency => "maxEfficiency",
            Stat::DrillTier => "drillTier",
            Stat::DrillSpeed => "drillSpeed",
            Stat::LinkRange => "linkRange",
            Stat::Instructions => "instructions",
            Stat::Weapons => "weapons",
            Stat::Bullet => "bullet",
            Stat::SpeedIncrease => "speedIncrease",
            Stat::RepairTime => "repairTime",
            Stat::RepairSpeed => "repairSpeed",
            Stat::Range => "range",
            Stat::ShootRange => "shootRange",
            Stat::Inaccuracy => "inaccuracy",
            Stat::Shots => "shots",
            Stat::Reload => "reload",
            Stat::CrushDamage => "crushDamage",
            Stat::LegSplashDamage => "legSplashDamage",
            Stat::TargetsAir => "targetsAir",
            Stat::TargetsGround => "targetsGround",
            Stat::Damage => "damage",
            Stat::Frequency => "frequency",
            Stat::Ammo => "ammo",
            Stat::AmmoCapacity => "ammoCapacity",
            Stat::AmmoUse => "ammoUse",
            Stat::ShieldHealth => "shieldHealth",
            Stat::CooldownTime => "cooldownTime",
            Stat::RegenerationRate => "regenerationRate",
            Stat::ActivationTime => "activationTime",
            Stat::ModuleTier => "moduletier",
            Stat::UnitType => "unittype",
            Stat::Booster => "booster",
            Stat::BoostEffect => "boostEffect",
            Stat::Affinities => "affinities",
            Stat::Opposites => "opposites",
        }
    }

    pub const fn category(self) -> StatCat {
        match self {
            Stat::ItemCapacity | Stat::ItemsMoved | Stat::LaunchTime | Stat::MaxConsecutive => {
                StatCat::Items
            }
            Stat::LiquidCapacity => StatCat::Liquids,
            Stat::PowerCapacity
            | Stat::PowerUse
            | Stat::PowerDamage
            | Stat::PowerRange
            | Stat::PowerConnections
            | Stat::BasePowerGeneration
            | Stat::WarmupTime => StatCat::Power,
            Stat::Tiles
            | Stat::Input
            | Stat::Output
            | Stat::ProductionTime
            | Stat::MaxEfficiency
            | Stat::DrillTier
            | Stat::DrillSpeed
            | Stat::LinkRange
            | Stat::Instructions => StatCat::Crafting,
            Stat::Weapons
            | Stat::Bullet
            | Stat::SpeedIncrease
            | Stat::RepairTime
            | Stat::RepairSpeed
            | Stat::Range
            | Stat::ShootRange
            | Stat::Inaccuracy
            | Stat::Shots
            | Stat::Reload
            | Stat::CrushDamage
            | Stat::LegSplashDamage
            | Stat::TargetsAir
            | Stat::TargetsGround
            | Stat::Damage
            | Stat::Frequency
            | Stat::Ammo
            | Stat::AmmoCapacity
            | Stat::AmmoUse
            | Stat::ShieldHealth
            | Stat::CooldownTime
            | Stat::RegenerationRate
            | Stat::ActivationTime
            | Stat::ModuleTier
            | Stat::UnitType => StatCat::Function,
            Stat::Booster | Stat::BoostEffect | Stat::Affinities | Stat::Opposites => {
                StatCat::Optional
            }
            _ => StatCat::General,
        }
    }

    pub fn bundle_key(self) -> String {
        format!("stat.{}", self.name().to_ascii_lowercase())
    }
}

impl core::fmt::Display for Stat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_keep_java_registration_order_names_and_categories() {
        assert_eq!(Stat::ALL.len(), 85);
        assert_eq!(Stat::Health.id(), 0);
        assert_eq!(Stat::MaxUnits.id(), 26);
        assert_eq!(Stat::ItemCapacity.id(), 35);
        assert_eq!(Stat::Opposites.id(), 84);
        assert_eq!(Stat::BoostingSpeed.name(), "boostingspeed");
        assert_eq!(Stat::ModuleTier.name(), "moduletier");
        assert_eq!(Stat::UnitType.name(), "unittype");
        assert_eq!(Stat::Health.category(), StatCat::General);
        assert_eq!(Stat::PowerUse.category(), StatCat::Power);
        assert_eq!(Stat::Output.category(), StatCat::Crafting);
        assert_eq!(Stat::Damage.category(), StatCat::Function);
        assert_eq!(Stat::Opposites.category(), StatCat::Optional);
        assert_eq!(Stat::BuildTime.bundle_key(), "stat.buildtime");
    }
}
