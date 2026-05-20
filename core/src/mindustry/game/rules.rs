use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub struct Rules {
    pub static_fog: bool,
    pub fog: bool,
    pub show_spawns: bool,
    pub wave_timer: bool,
    pub wave_sending: bool,
    pub waves: bool,
    pub air_use_spawns: bool,
    pub waves_spawn_at_cores: bool,
    pub infinite_resources: bool,
    pub allow_edit_rules: bool,
    pub attack_mode: bool,
    pub pvp: bool,
    pub pvp_auto_pause: bool,
    pub editor: bool,
    pub wait_enemies: bool,
    pub derelict_repair: bool,
    pub can_game_over: bool,
    pub core_capture: bool,
    pub reactor_explosions: bool,
    pub possession_allowed: bool,
    pub schematics_allowed: bool,
    pub damage_explosions: bool,
    pub fire: bool,
    pub unit_ammo: bool,
    pub instant_build: bool,
    pub random_wave_ai: bool,
    pub unit_payload_update: bool,
    pub unit_payloads_explode: bool,
    pub unit_cap_variable: bool,
    pub ghost_blocks: bool,
    pub logic_unit_control: bool,
    pub logic_unit_build: bool,
    pub logic_unit_deconstruct: bool,
    pub allow_edit_world_processors: bool,
    pub disable_world_processors: bool,
    pub pause_disabled: bool,
    pub enemy_core_build_radius: f32,
    pub solar_multiplier: f32,
    pub build_cost_multiplier: f32,
    pub build_speed_multiplier: f32,
    pub block_health_multiplier: f32,
    pub block_damage_multiplier: f32,
    pub unit_build_speed_multiplier: f32,
    pub unit_cost_multiplier: f32,
    pub unit_damage_multiplier: f32,
    pub unit_health_multiplier: f32,
    pub unit_crash_damage_multiplier: f32,
    pub unit_mine_speed_multiplier: f32,
    pub deconstruct_refund_multiplier: f32,
    pub objective_timer_multiplier: f32,
    pub item_deposit_cooldown: f32,
    pub drop_zone_radius: f32,
    pub wave_spacing: f32,
    pub initial_wave_spacing: f32,
    pub win_wave: i32,
    pub unit_cap: i32,
    pub disable_unit_cap: bool,
    pub drag_multiplier: f32,
    pub env: u32,
    pub block_whitelist: bool,
    pub unit_whitelist: bool,
    pub banned_blocks: BTreeSet<String>,
    pub banned_units: BTreeSet<String>,
    pub revealed_blocks: BTreeSet<String>,
    pub researched: BTreeSet<String>,
    pub objective_flags: BTreeSet<String>,
    pub tags: BTreeMap<String, String>,
    pub core_incinerates: bool,
    pub border_darkness: bool,
    pub limit_map_area: bool,
    pub limit_x: i32,
    pub limit_y: i32,
    pub limit_width: i32,
    pub limit_height: i32,
    pub disable_outside_area: bool,
    pub background_speed: f32,
    pub background_scl: f32,
    pub background_offset_x: f32,
    pub background_offset_y: f32,
    pub allow_logic_data: bool,
    pub wave_team: i32,
    pub teams: TeamRules,
}

impl Rules {
    pub fn mode(&self) -> crate::mindustry::game::Gamemode {
        if self.pvp {
            crate::mindustry::game::Gamemode::Pvp
        } else if self.editor {
            crate::mindustry::game::Gamemode::Editor
        } else if self.attack_mode {
            crate::mindustry::game::Gamemode::Attack
        } else if self.infinite_resources {
            crate::mindustry::game::Gamemode::Sandbox
        } else {
            crate::mindustry::game::Gamemode::Survival
        }
    }

    pub fn build_radius(&self, team_id: usize) -> f32 {
        let team = self.teams.get_or_default(team_id);
        if team.protect_cores {
            self.enemy_core_build_radius + team.extra_core_build_radius
        } else {
            0.0
        }
    }

    pub fn unit_build_speed(&self, team_id: usize) -> f32 {
        self.unit_build_speed_multiplier
            * self
                .teams
                .get_or_default(team_id)
                .unit_build_speed_multiplier
    }

    pub fn unit_cost(&self, team_id: usize) -> f32 {
        self.unit_cost_multiplier * self.teams.get_or_default(team_id).unit_cost_multiplier
    }

    pub fn unit_damage(&self, team_id: usize) -> f32 {
        self.unit_damage_multiplier * self.teams.get_or_default(team_id).unit_damage_multiplier
    }

    pub fn unit_health(&self, team_id: usize) -> f32 {
        (self.unit_health_multiplier * self.teams.get_or_default(team_id).unit_health_multiplier)
            .max(0.000001)
    }

    pub fn unit_crash_damage(&self, team_id: usize) -> f32 {
        self.unit_damage(team_id)
            * self.unit_crash_damage_multiplier
            * self
                .teams
                .get_or_default(team_id)
                .unit_crash_damage_multiplier
    }

    pub fn unit_mine_speed(&self, team_id: usize) -> f32 {
        self.unit_mine_speed_multiplier
            * self
                .teams
                .get_or_default(team_id)
                .unit_mine_speed_multiplier
    }

    pub fn block_health(&self, team_id: usize) -> f32 {
        self.block_health_multiplier * self.teams.get_or_default(team_id).block_health_multiplier
    }

    pub fn block_damage(&self, team_id: usize) -> f32 {
        self.block_damage_multiplier * self.teams.get_or_default(team_id).block_damage_multiplier
    }

    pub fn build_speed(&self, team_id: usize) -> f32 {
        self.build_speed_multiplier * self.teams.get_or_default(team_id).build_speed_multiplier
    }

    pub fn has_env(&self, env: u32) -> bool {
        (self.env & env) != 0
    }

    pub fn is_block_banned(&self, block_name: &str) -> bool {
        self.block_whitelist != self.banned_blocks.contains(block_name)
    }

    pub fn is_unit_banned(&self, unit_name: &str) -> bool {
        self.unit_whitelist != self.banned_units.contains(unit_name)
    }

    pub fn reveal_block(&mut self, block_name: impl Into<String>) {
        self.revealed_blocks.insert(block_name.into());
    }

    pub fn research_content(&mut self, content_name: impl Into<String>) {
        self.researched.insert(content_name.into());
    }

    pub fn set_objective_flag(&mut self, flag: impl Into<String>) {
        self.objective_flags.insert(flag.into());
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            static_fog: true,
            fog: false,
            show_spawns: false,
            wave_timer: true,
            wave_sending: true,
            waves: false,
            air_use_spawns: false,
            waves_spawn_at_cores: true,
            infinite_resources: false,
            allow_edit_rules: false,
            attack_mode: false,
            pvp: false,
            pvp_auto_pause: true,
            editor: false,
            wait_enemies: false,
            derelict_repair: true,
            can_game_over: true,
            core_capture: false,
            reactor_explosions: true,
            possession_allowed: true,
            schematics_allowed: true,
            damage_explosions: true,
            fire: true,
            unit_ammo: false,
            instant_build: false,
            random_wave_ai: false,
            unit_payload_update: false,
            unit_payloads_explode: false,
            unit_cap_variable: true,
            ghost_blocks: true,
            logic_unit_control: true,
            logic_unit_build: true,
            logic_unit_deconstruct: false,
            allow_edit_world_processors: false,
            disable_world_processors: false,
            pause_disabled: false,
            enemy_core_build_radius: 400.0,
            solar_multiplier: 1.0,
            build_cost_multiplier: 1.0,
            build_speed_multiplier: 1.0,
            block_health_multiplier: 1.0,
            block_damage_multiplier: 1.0,
            unit_build_speed_multiplier: 1.0,
            unit_cost_multiplier: 1.0,
            unit_damage_multiplier: 1.0,
            unit_health_multiplier: 1.0,
            unit_crash_damage_multiplier: 1.0,
            unit_mine_speed_multiplier: 1.0,
            deconstruct_refund_multiplier: 0.5,
            objective_timer_multiplier: 1.0,
            item_deposit_cooldown: 0.5,
            drop_zone_radius: 300.0,
            wave_spacing: 2.0 * 60.0 * 60.0,
            initial_wave_spacing: 0.0,
            win_wave: 0,
            unit_cap: 0,
            disable_unit_cap: false,
            drag_multiplier: 1.0,
            env: crate::mindustry::world::meta::Env::ANY,
            block_whitelist: false,
            unit_whitelist: false,
            banned_blocks: BTreeSet::new(),
            banned_units: BTreeSet::new(),
            revealed_blocks: BTreeSet::new(),
            researched: BTreeSet::new(),
            objective_flags: BTreeSet::new(),
            tags: BTreeMap::new(),
            core_incinerates: true,
            border_darkness: true,
            limit_map_area: false,
            limit_x: 0,
            limit_y: 0,
            limit_width: 1,
            limit_height: 1,
            disable_outside_area: true,
            background_speed: 27000.0,
            background_scl: 1.0,
            background_offset_x: 0.1,
            background_offset_y: 0.1,
            allow_logic_data: false,
            wave_team: 1,
            teams: TeamRules::new(256),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TeamRule {
    pub ai_core_spawn: bool,
    pub protect_cores: bool,
    pub check_placement: bool,
    pub cheat: bool,
    pub fill_items: bool,
    pub infinite_resources: bool,
    pub infinite_ammo: bool,
    pub prebuild_ai: bool,
    pub build_ai: bool,
    pub build_ai_tier: f32,
    pub respawn: bool,
    pub unit_damage_multiplier: f32,
    pub unit_health_multiplier: f32,
    pub unit_crash_damage_multiplier: f32,
    pub unit_mine_speed_multiplier: f32,
    pub unit_cost_multiplier: f32,
    pub unit_build_speed_multiplier: f32,
    pub block_damage_multiplier: f32,
    pub block_health_multiplier: f32,
    pub build_speed_multiplier: f32,
    pub rts_ai: bool,
    pub rts_min_squad: i32,
    pub rts_max_squad: i32,
    pub rts_min_weight: f32,
    pub unit_factory_activation_delay: f32,
    pub extra_core_build_radius: f32,
}

impl Default for TeamRule {
    fn default() -> Self {
        Self {
            ai_core_spawn: true,
            protect_cores: true,
            check_placement: true,
            cheat: false,
            fill_items: false,
            infinite_resources: false,
            infinite_ammo: false,
            prebuild_ai: false,
            build_ai: false,
            build_ai_tier: 1.0,
            respawn: true,
            unit_damage_multiplier: 1.0,
            unit_health_multiplier: 1.0,
            unit_crash_damage_multiplier: 1.0,
            unit_mine_speed_multiplier: 1.0,
            unit_cost_multiplier: 1.0,
            unit_build_speed_multiplier: 1.0,
            block_damage_multiplier: 1.0,
            block_health_multiplier: 1.0,
            build_speed_multiplier: 1.0,
            rts_ai: false,
            rts_min_squad: 4,
            rts_max_squad: 50,
            rts_min_weight: 1.2,
            unit_factory_activation_delay: 0.0,
            extra_core_build_radius: 0.0,
        }
    }
}

impl TeamRule {
    pub fn for_team_id(team_id: usize) -> Self {
        let mut rule = Self::default();
        if team_id == crate::mindustry::game::TEAM_DERELICT as usize {
            rule.protect_cores = false;
            rule.check_placement = false;
        }
        rule
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
            .unwrap_or_else(|| TeamRule::for_team_id(team_id))
    }

    pub fn get_or_insert(&mut self, team_id: usize) -> &mut TeamRule {
        if team_id >= self.values.len() {
            self.values.resize_with(team_id + 1, || None);
        }
        self.values[team_id].get_or_insert_with(|| TeamRule::for_team_id(team_id))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::game::Gamemode;

    #[test]
    fn rules_defaults_match_upstream_low_risk_field_initializers() {
        let rules = Rules::default();
        assert!(rules.wave_timer);
        assert!(rules.wave_sending);
        assert!(!rules.air_use_spawns);
        assert!(rules.waves_spawn_at_cores);
        assert!(rules.pvp_auto_pause);
        assert!(!rules.wait_enemies);
        assert!(rules.derelict_repair);
        assert!(rules.can_game_over);
        assert!(!rules.core_capture);
        assert!(rules.reactor_explosions);
        assert!(rules.possession_allowed);
        assert!(rules.schematics_allowed);
        assert!(rules.damage_explosions);
        assert!(rules.fire);
        assert!(!rules.unit_ammo);
        assert!(rules.unit_cap_variable);
        assert!(rules.ghost_blocks);
        assert!(rules.logic_unit_control);
        assert!(rules.logic_unit_build);
        assert!(!rules.logic_unit_deconstruct);
        assert_eq!(rules.solar_multiplier, 1.0);
        assert_eq!(rules.deconstruct_refund_multiplier, 0.5);
        assert_eq!(rules.item_deposit_cooldown, 0.5);
        assert_eq!(rules.drop_zone_radius, 300.0);
        assert_eq!(rules.initial_wave_spacing, 0.0);
        assert_eq!(rules.win_wave, 0);
        assert_eq!(rules.unit_cap, 0);
        assert_eq!(rules.drag_multiplier, 1.0);
        assert_eq!(rules.env, crate::mindustry::world::meta::Env::ANY);
        assert!(!rules.block_whitelist);
        assert!(!rules.unit_whitelist);
        assert!(rules.banned_blocks.is_empty());
        assert!(rules.banned_units.is_empty());
        assert!(rules.revealed_blocks.is_empty());
        assert!(rules.researched.is_empty());
        assert!(rules.objective_flags.is_empty());
        assert!(rules.tags.is_empty());
        assert!(rules.core_incinerates);
        assert!(rules.border_darkness);
        assert!(!rules.limit_map_area);
        assert_eq!(
            (
                rules.limit_x,
                rules.limit_y,
                rules.limit_width,
                rules.limit_height
            ),
            (0, 0, 1, 1)
        );
        assert!(rules.disable_outside_area);
        assert_eq!(rules.background_speed, 27000.0);
        assert_eq!(rules.background_scl, 1.0);
        assert_eq!(rules.background_offset_x, 0.1);
        assert_eq!(rules.background_offset_y, 0.1);
        assert!(!rules.allow_logic_data);
    }

    #[test]
    fn gamemode_application_keeps_existing_behavior_after_field_expansion() {
        let mut attack = Rules::default();
        Gamemode::Attack.apply(&mut attack);
        assert!(attack.attack_mode);
        assert!(attack.wave_timer);
        assert_eq!(attack.wave_spacing, 2.0 * 60.0 * 60.0);
        assert!(
            attack
                .teams
                .get_or_default(attack.wave_team as usize)
                .infinite_resources
        );

        let mut editor = Rules::default();
        Gamemode::Editor.apply(&mut editor);
        assert!(editor.infinite_resources);
        assert!(editor.instant_build);
        assert!(editor.editor);
        assert!(!editor.waves);
        assert!(!editor.wave_timer);
    }

    #[test]
    fn team_rule_defaults_and_derelict_special_case_match_java_constructor() {
        let rules = TeamRules::new(256);
        let derelict = rules.get_or_default(crate::mindustry::game::TEAM_DERELICT as usize);
        assert!(!derelict.protect_cores);
        assert!(!derelict.check_placement);

        let sharded = rules.get_or_default(crate::mindustry::game::TEAM_SHARDED as usize);
        assert!(sharded.ai_core_spawn);
        assert!(sharded.protect_cores);
        assert!(sharded.check_placement);
        assert_eq!(sharded.build_ai_tier, 1.0);
        assert_eq!(sharded.rts_min_squad, 4);
        assert_eq!(sharded.rts_max_squad, 50);
        assert_eq!(sharded.rts_min_weight, 1.2);
        assert_eq!(sharded.extra_core_build_radius, 0.0);
    }

    #[test]
    fn rules_mode_and_team_multipliers_follow_java_formulas() {
        let mut rules = Rules::default();
        assert_eq!(rules.mode(), Gamemode::Survival);
        rules.infinite_resources = true;
        assert_eq!(rules.mode(), Gamemode::Sandbox);
        rules.attack_mode = true;
        assert_eq!(rules.mode(), Gamemode::Attack);
        rules.editor = true;
        assert_eq!(rules.mode(), Gamemode::Editor);
        rules.pvp = true;
        assert_eq!(rules.mode(), Gamemode::Pvp);

        let team_id = crate::mindustry::game::TEAM_SHARDED as usize;
        let team = rules.teams.get_or_insert(team_id);
        team.unit_build_speed_multiplier = 2.0;
        team.unit_cost_multiplier = 3.0;
        team.unit_damage_multiplier = 4.0;
        team.unit_health_multiplier = 0.0;
        team.unit_crash_damage_multiplier = 5.0;
        team.unit_mine_speed_multiplier = 6.0;
        team.block_health_multiplier = 7.0;
        team.block_damage_multiplier = 8.0;
        team.build_speed_multiplier = 9.0;
        team.extra_core_build_radius = 10.0;

        rules.unit_build_speed_multiplier = 1.5;
        rules.unit_cost_multiplier = 2.0;
        rules.unit_damage_multiplier = 2.5;
        rules.unit_health_multiplier = 0.0;
        rules.unit_crash_damage_multiplier = 3.0;
        rules.unit_mine_speed_multiplier = 3.5;
        rules.block_health_multiplier = 4.0;
        rules.block_damage_multiplier = 4.5;
        rules.build_speed_multiplier = 5.0;
        rules.enemy_core_build_radius = 400.0;

        assert_eq!(rules.build_radius(team_id), 410.0);
        assert_eq!(rules.unit_build_speed(team_id), 3.0);
        assert_eq!(rules.unit_cost(team_id), 6.0);
        assert_eq!(rules.unit_damage(team_id), 10.0);
        assert_eq!(rules.unit_health(team_id), 0.000001);
        assert_eq!(rules.unit_crash_damage(team_id), 150.0);
        assert_eq!(rules.unit_mine_speed(team_id), 21.0);
        assert_eq!(rules.block_health(team_id), 28.0);
        assert_eq!(rules.block_damage(team_id), 36.0);
        assert_eq!(rules.build_speed(team_id), 45.0);
        assert_eq!(
            rules.build_radius(crate::mindustry::game::TEAM_DERELICT as usize),
            0.0
        );
    }

    #[test]
    fn rules_env_and_banned_sets_follow_java_boolean_xor_semantics() {
        let mut rules = Rules::default();
        rules.env = crate::mindustry::world::meta::Env::TERRESTRIAL;
        assert!(rules.has_env(crate::mindustry::world::meta::Env::TERRESTRIAL));
        assert!(!rules.has_env(crate::mindustry::world::meta::Env::SPACE));

        rules.banned_blocks.insert("router".into());
        assert!(rules.is_block_banned("router"));
        assert!(!rules.is_block_banned("duo"));
        rules.block_whitelist = true;
        assert!(!rules.is_block_banned("router"));
        assert!(rules.is_block_banned("duo"));

        rules.banned_units.insert("dagger".into());
        assert!(rules.is_unit_banned("dagger"));
        assert!(!rules.is_unit_banned("flare"));
        rules.unit_whitelist = true;
        assert!(!rules.is_unit_banned("dagger"));
        assert!(rules.is_unit_banned("flare"));

        rules.reveal_block("core-shard");
        rules.research_content("copper");
        rules.set_objective_flag("launch");
        rules.tags.insert("author".into(), "rust".into());
        assert!(rules.revealed_blocks.contains("core-shard"));
        assert!(rules.researched.contains("copper"));
        assert!(rules.objective_flags.contains("launch"));
        assert_eq!(rules.tags.get("author").map(String::as_str), Some("rust"));
    }
}
