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
