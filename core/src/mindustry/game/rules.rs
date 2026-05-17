#[derive(Debug, Clone, PartialEq)]
pub struct Rules {
    pub wave_timer: bool,
    pub waves: bool,
    pub infinite_resources: bool,
    pub allow_edit_rules: bool,
    pub attack_mode: bool,
    pub pvp: bool,
    pub editor: bool,
    pub instant_build: bool,
    pub enemy_core_build_radius: f32,
    pub build_cost_multiplier: f32,
    pub build_speed_multiplier: f32,
    pub unit_build_speed_multiplier: f32,
    pub wave_spacing: f32,
    pub wave_team: i32,
    pub teams: TeamRules,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            wave_timer: false,
            waves: false,
            infinite_resources: false,
            allow_edit_rules: false,
            attack_mode: false,
            pvp: false,
            editor: false,
            instant_build: false,
            enemy_core_build_radius: 400.0,
            build_cost_multiplier: 1.0,
            build_speed_multiplier: 1.0,
            unit_build_speed_multiplier: 1.0,
            wave_spacing: 2.0 * 60.0 * 60.0,
            wave_team: 1,
            teams: TeamRules::new(256),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TeamRule {
    pub infinite_resources: bool,
    pub respawn: bool,
    pub unit_damage_multiplier: f32,
    pub unit_health_multiplier: f32,
    pub block_damage_multiplier: f32,
    pub block_health_multiplier: f32,
    pub build_speed_multiplier: f32,
    pub rts_ai: bool,
}

impl Default for TeamRule {
    fn default() -> Self {
        Self {
            infinite_resources: false,
            respawn: true,
            unit_damage_multiplier: 1.0,
            unit_health_multiplier: 1.0,
            block_damage_multiplier: 1.0,
            block_health_multiplier: 1.0,
            build_speed_multiplier: 1.0,
            rts_ai: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TeamRules {
    values: Vec<Option<TeamRule>>,
}

impl TeamRules {
    pub fn new(capacity: usize) -> Self {
        Self {
            values: vec![None; capacity],
        }
    }

    pub fn get_or_default(&self, team_id: usize) -> TeamRule {
        self.values
            .get(team_id)
            .and_then(Clone::clone)
            .unwrap_or_default()
    }

    pub fn get_or_insert(&mut self, team_id: usize) -> &mut TeamRule {
        if team_id >= self.values.len() {
            self.values.resize_with(team_id + 1, || None);
        }
        self.values[team_id].get_or_insert_with(TeamRule::default)
    }
}

impl GamemodeApplier for crate::mindustry::game::Gamemode {
    fn apply(self, rules: &mut Rules) {
        match self {
            crate::mindustry::game::Gamemode::Survival => {
                rules.wave_timer = true;
                rules.waves = true;
            }
            crate::mindustry::game::Gamemode::Sandbox => {
                rules.infinite_resources = true;
                rules.allow_edit_rules = true;
                rules.waves = true;
                rules.wave_timer = false;
            }
            crate::mindustry::game::Gamemode::Attack => {
                rules.attack_mode = true;
                rules.wave_timer = true;
                rules.wave_spacing = 2.0 * 60.0 * 60.0;
                let team = rules.teams.get_or_insert(rules.wave_team as usize);
                team.infinite_resources = true;
            }
            crate::mindustry::game::Gamemode::Pvp => {
                rules.pvp = true;
                rules.enemy_core_build_radius = 600.0;
                rules.build_cost_multiplier = 1.0;
                rules.build_speed_multiplier = 1.0;
                rules.unit_build_speed_multiplier = 2.0;
                rules.attack_mode = true;
            }
            crate::mindustry::game::Gamemode::Editor => {
                rules.infinite_resources = true;
                rules.instant_build = true;
                rules.editor = true;
                rules.waves = false;
                rules.wave_timer = false;
            }
        }
    }
}

pub trait GamemodeApplier {
    fn apply(self, rules: &mut Rules);
}
