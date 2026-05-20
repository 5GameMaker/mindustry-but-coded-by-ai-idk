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
    pub infinite_resources: bool,
    pub respawn: bool,
    pub unit_damage_multiplier: f32,
    pub unit_health_multiplier: f32,
    pub unit_cost_multiplier: f32,
    pub unit_build_speed_multiplier: f32,
    pub block_damage_multiplier: f32,
    pub block_health_multiplier: f32,
    pub build_speed_multiplier: f32,
    pub rts_ai: bool,
    pub rts_max_squad: i32,
}

impl Default for TeamRule {
    fn default() -> Self {
        Self {
            infinite_resources: false,
            respawn: true,
            unit_damage_multiplier: 1.0,
            unit_health_multiplier: 1.0,
            unit_cost_multiplier: 1.0,
            unit_build_speed_multiplier: 1.0,
            block_damage_multiplier: 1.0,
            block_health_multiplier: 1.0,
            build_speed_multiplier: 1.0,
            rts_ai: false,
            rts_max_squad: 50,
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
}
