//! Mirrors upstream `mindustry.logic.LogicRule`.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicRule {
    CurrentWaveTime,
    WaveTimer,
    Waves,
    Wave,
    WaveSpacing,
    WaveSending,
    AttackMode,
    EnemyCoreBuildRadius,
    DropZoneRadius,
    UnitCap,
    MapArea,
    Lighting,
    CanGameOver,
    AmbientLight,
    SolarMultiplier,
    DragMultiplier,
    Ban,
    Unban,
    PauseDisabled,
    BuildSpeed,
    UnitHealth,
    UnitBuildSpeed,
    UnitMineSpeed,
    UnitCost,
    UnitDamage,
    BlockHealth,
    BlockDamage,
    RtsMinWeight,
    RtsMinSquad,
}

impl LogicRule {
    pub const ALL: [LogicRule; 29] = [
        LogicRule::CurrentWaveTime,
        LogicRule::WaveTimer,
        LogicRule::Waves,
        LogicRule::Wave,
        LogicRule::WaveSpacing,
        LogicRule::WaveSending,
        LogicRule::AttackMode,
        LogicRule::EnemyCoreBuildRadius,
        LogicRule::DropZoneRadius,
        LogicRule::UnitCap,
        LogicRule::MapArea,
        LogicRule::Lighting,
        LogicRule::CanGameOver,
        LogicRule::AmbientLight,
        LogicRule::SolarMultiplier,
        LogicRule::DragMultiplier,
        LogicRule::Ban,
        LogicRule::Unban,
        LogicRule::PauseDisabled,
        LogicRule::BuildSpeed,
        LogicRule::UnitHealth,
        LogicRule::UnitBuildSpeed,
        LogicRule::UnitMineSpeed,
        LogicRule::UnitCost,
        LogicRule::UnitDamage,
        LogicRule::BlockHealth,
        LogicRule::BlockDamage,
        LogicRule::RtsMinWeight,
        LogicRule::RtsMinSquad,
    ];

    pub const WIRE_NAMES: [&'static str; 29] = [
        "currentWaveTime",
        "waveTimer",
        "waves",
        "wave",
        "waveSpacing",
        "waveSending",
        "attackMode",
        "enemyCoreBuildRadius",
        "dropZoneRadius",
        "unitCap",
        "mapArea",
        "lighting",
        "canGameOver",
        "ambientLight",
        "solarMultiplier",
        "dragMultiplier",
        "ban",
        "unban",
        "pauseDisabled",
        "buildSpeed",
        "unitHealth",
        "unitBuildSpeed",
        "unitMineSpeed",
        "unitCost",
        "unitDamage",
        "blockHealth",
        "blockDamage",
        "rtsMinWeight",
        "rtsMinSquad",
    ];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::LogicRule;

    #[test]
    fn logic_rule_order_and_wire_names_match_java_enum() {
        assert_eq!(LogicRule::ALL.len(), 29);
        assert_eq!(LogicRule::CurrentWaveTime.ordinal(), 0);
        assert_eq!(LogicRule::PauseDisabled.ordinal(), 18);
        assert_eq!(LogicRule::BuildSpeed.ordinal(), 19);
        assert_eq!(LogicRule::RtsMinSquad.ordinal(), 28);
        assert_eq!(
            LogicRule::ALL
                .iter()
                .map(|rule| rule.wire_name())
                .collect::<Vec<_>>(),
            LogicRule::WIRE_NAMES.to_vec()
        );
        assert_eq!(
            LogicRule::by_wire_name("enemyCoreBuildRadius"),
            Some(LogicRule::EnemyCoreBuildRadius)
        );
        assert_eq!(LogicRule::from_ordinal(29), None);
        assert_eq!(LogicRule::by_wire_name("missing"), None);
    }
}
