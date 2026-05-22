#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    Casual,
    Easy,
    Normal,
    Hard,
    Eradication,
}

impl Difficulty {
    pub const ALL: [Difficulty; 5] = [
        Difficulty::Casual,
        Difficulty::Easy,
        Difficulty::Normal,
        Difficulty::Hard,
        Difficulty::Eradication,
    ];

    pub const fn multipliers(self) -> DifficultyMultipliers {
        match self {
            Difficulty::Casual => DifficultyMultipliers::new(0.5, 0.5, 2.0),
            Difficulty::Easy => DifficultyMultipliers::new(1.0, 0.75, 1.5),
            Difficulty::Normal => DifficultyMultipliers::new(1.0, 1.0, 1.0),
            Difficulty::Hard => DifficultyMultipliers::new(1.25, 1.5, 0.8),
            Difficulty::Eradication => DifficultyMultipliers::new(1.5, 2.0, 0.6),
        }
    }

    pub const fn wire_name(self) -> &'static str {
        match self {
            Difficulty::Casual => "casual",
            Difficulty::Easy => "easy",
            Difficulty::Normal => "normal",
            Difficulty::Hard => "hard",
            Difficulty::Eradication => "eradication",
        }
    }

    pub fn info(self) -> String {
        self.info_with(&DifficultyBundleKeys)
    }

    pub fn info_with(self, bundle: &impl DifficultyTextBundle) -> String {
        let multipliers = self.multipliers();
        let mut result = String::new();

        if multipliers.enemy_health_multiplier != 1.0 {
            result.push_str(&bundle.format(
                "difficulty.enemyHealthMultiplier",
                &Self::percent_stat(multipliers.enemy_health_multiplier),
            ));
            result.push('\n');
        }

        if multipliers.enemy_spawn_multiplier != 1.0 {
            result.push_str(&bundle.format(
                "difficulty.enemySpawnMultiplier",
                &Self::percent_stat(multipliers.enemy_spawn_multiplier),
            ));
            result.push('\n');
        }

        if multipliers.wave_time_multiplier != 1.0 {
            result.push_str(&bundle.format(
                "difficulty.waveTimeMultiplier",
                &Self::percent_stat_neg(multipliers.wave_time_multiplier),
            ));
            result.push('\n');
        }

        if result.is_empty() {
            bundle.get("difficulty.nomodifiers")
        } else {
            result
        }
    }

    pub fn localized(self) -> String {
        self.localized_with(&DifficultyBundleKeys)
    }

    pub fn localized_with(self, bundle: &impl DifficultyTextBundle) -> String {
        bundle.get(&format!("difficulty.{}", self.wire_name()))
    }

    pub fn percent_stat(value: f32) -> String {
        let percent = (value * 100.0 - 100.0) as i32;
        let prefix = if percent > 0 { "[negstat]+" } else { "[stat]" };
        format!("{prefix}{percent}%[]")
    }

    pub fn percent_stat_neg(value: f32) -> String {
        let percent = (value * 100.0 - 100.0) as i32;
        let prefix = if percent > 0 { "[stat]+" } else { "[negstat]" };
        format!("{prefix}{percent}%[]")
    }
}

pub trait DifficultyTextBundle {
    fn get(&self, key: &str) -> String;
    fn format(&self, key: &str, value: &str) -> String;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DifficultyBundleKeys;

impl DifficultyTextBundle for DifficultyBundleKeys {
    fn get(&self, key: &str) -> String {
        key.to_string()
    }

    fn format(&self, key: &str, value: &str) -> String {
        format!("{key}:{value}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DifficultyMultipliers {
    pub enemy_health_multiplier: f32,
    pub enemy_spawn_multiplier: f32,
    pub wave_time_multiplier: f32,
}

impl DifficultyMultipliers {
    pub const fn new(
        enemy_health_multiplier: f32,
        enemy_spawn_multiplier: f32,
        wave_time_multiplier: f32,
    ) -> Self {
        Self {
            enemy_health_multiplier,
            enemy_spawn_multiplier,
            wave_time_multiplier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBundle;

    impl DifficultyTextBundle for TestBundle {
        fn get(&self, key: &str) -> String {
            format!("@{key}")
        }

        fn format(&self, key: &str, value: &str) -> String {
            format!("@{key}({value})")
        }
    }

    #[test]
    fn difficulty_order_names_and_multipliers_match_java() {
        assert_eq!(
            Difficulty::ALL
                .iter()
                .map(|difficulty| difficulty.wire_name())
                .collect::<Vec<_>>(),
            vec!["casual", "easy", "normal", "hard", "eradication"]
        );

        assert_eq!(
            Difficulty::Casual.multipliers(),
            DifficultyMultipliers::new(0.5, 0.5, 2.0)
        );
        assert_eq!(
            Difficulty::Easy.multipliers(),
            DifficultyMultipliers::new(1.0, 0.75, 1.5)
        );
        assert_eq!(
            Difficulty::Normal.multipliers(),
            DifficultyMultipliers::new(1.0, 1.0, 1.0)
        );
        assert_eq!(
            Difficulty::Hard.multipliers(),
            DifficultyMultipliers::new(1.25, 1.5, 0.8)
        );
        assert_eq!(
            Difficulty::Eradication.multipliers(),
            DifficultyMultipliers::new(1.5, 2.0, 0.6)
        );
    }

    #[test]
    fn difficulty_localized_uses_java_bundle_keys() {
        assert_eq!(Difficulty::Casual.localized(), "difficulty.casual");
        assert_eq!(
            Difficulty::Eradication.localized_with(&TestBundle),
            "@difficulty.eradication"
        );
    }

    #[test]
    fn difficulty_info_formats_modifier_lines_like_java() {
        assert_eq!(
            Difficulty::Casual.info_with(&TestBundle),
            concat!(
                "@difficulty.enemyHealthMultiplier([stat]-50%[])\n",
                "@difficulty.enemySpawnMultiplier([stat]-50%[])\n",
                "@difficulty.waveTimeMultiplier([stat]+100%[])\n",
            )
        );

        assert_eq!(
            Difficulty::Hard.info(),
            concat!(
                "difficulty.enemyHealthMultiplier:[negstat]+25%[]\n",
                "difficulty.enemySpawnMultiplier:[negstat]+50%[]\n",
                "difficulty.waveTimeMultiplier:[negstat]-20%[]\n",
            )
        );

        assert_eq!(
            Difficulty::Eradication.info_with(&TestBundle),
            concat!(
                "@difficulty.enemyHealthMultiplier([negstat]+50%[])\n",
                "@difficulty.enemySpawnMultiplier([negstat]+100%[])\n",
                "@difficulty.waveTimeMultiplier([negstat]-39%[])\n",
            )
        );
    }

    #[test]
    fn difficulty_info_returns_no_modifiers_bundle_for_normal() {
        assert_eq!(Difficulty::Normal.info(), "difficulty.nomodifiers");
        assert_eq!(
            Difficulty::Normal.info_with(&TestBundle),
            "@difficulty.nomodifiers"
        );
    }
}
