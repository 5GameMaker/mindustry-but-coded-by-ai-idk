//! Lightweight team-state container mirroring the pure data portions of
//! upstream `mindustry.game.Teams`.
//!
//! Entity-heavy operations (`QuadTree`, AI controllers, derelict scheduling)
//! are represented as data hooks so the networking/game-state layer can share
//! Java-compatible team IDs and active/enemy semantics now.

use crate::mindustry::game::{TEAM_COUNT, TEAM_CRUX, TEAM_NEOPLASTIC};
use crate::mindustry::world::{point2_x, point2_y};

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoreInfo {
    pub id: i32,
    pub team: u8,
    pub x: f32,
    pub y: f32,
}

impl CoreInfo {
    pub const fn new(id: i32, team: u8, x: f32, y: f32) -> Self {
        Self { id, team, x, y }
    }

    pub fn dst2(&self, x: f32, y: f32) -> f32 {
        let dx = self.x - x;
        let dy = self.y - y;
        dx * dx + dy * dy
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPlan {
    pub x: i16,
    pub y: i16,
    pub rotation: i16,
    pub block: String,
    pub config: Option<String>,
    pub removed: bool,
}

impl BlockPlan {
    pub fn new(
        x: i32,
        y: i32,
        rotation: i16,
        block: impl Into<String>,
        config: Option<String>,
    ) -> Self {
        Self {
            x: x as i16,
            y: y as i16,
            rotation,
            block: block.into(),
            config,
            removed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeamPlanClaim {
    NoPlans,
    NoUsablePlan,
    AlreadyPlaced(BlockPlan),
    Claimed(BlockPlan),
    Rotated(BlockPlan),
}

impl TeamPlanClaim {
    pub fn into_claimed_plan(self) -> Option<BlockPlan> {
        match self {
            TeamPlanClaim::Claimed(plan) => Some(plan),
            TeamPlanClaim::NoPlans
            | TeamPlanClaim::NoUsablePlan
            | TeamPlanClaim::AlreadyPlaced(_)
            | TeamPlanClaim::Rotated(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TeamData {
    pub team: u8,
    pub present_flag: bool,
    pub core_enemies: Vec<u8>,
    pub plans: Vec<BlockPlan>,
    pub cores: Vec<CoreInfo>,
    pub core_items: BTreeMap<i16, i32>,
    pub last_core: Option<CoreInfo>,
    pub unit_cap: i32,
    pub unit_count: i32,
    pub type_counts: BTreeMap<i32, i32>,
    pub building_counts: BTreeMap<String, i32>,
    pub units: Vec<i32>,
    pub players: Vec<i32>,
    pub building_ids: Vec<i32>,
}

impl TeamData {
    pub fn new(team: u8) -> Self {
        Self {
            team,
            present_flag: false,
            core_enemies: Vec::new(),
            plans: Vec::new(),
            cores: Vec::new(),
            core_items: BTreeMap::new(),
            last_core: None,
            unit_cap: 0,
            unit_count: 0,
            type_counts: BTreeMap::new(),
            building_counts: BTreeMap::new(),
            units: Vec::new(),
            players: Vec::new(),
            building_ids: Vec::new(),
        }
    }

    pub fn get_count(&self, block: &str) -> i32 {
        *self.building_counts.get(block).unwrap_or(&0)
    }

    pub fn add_building(&mut self, id: i32, block: impl Into<String>) {
        let block = block.into();
        if !self.building_ids.contains(&id) {
            self.building_ids.push(id);
            *self.building_counts.entry(block).or_insert(0) += 1;
        }
        self.present_flag = true;
    }

    pub fn update_count(&mut self, unit_type_id: i32, amount: i32) {
        if unit_type_id < 0 {
            return;
        }
        self.unit_count = (self.unit_count + amount).max(0);
        let entry = self.type_counts.entry(unit_type_id).or_insert(0);
        *entry = (*entry + amount).max(0);
    }

    pub fn count_type(&self, unit_type_id: i32) -> i32 {
        *self.type_counts.get(&unit_type_id).unwrap_or(&0)
    }

    pub fn active(&self, waves: bool, wave_team: u8) -> bool {
        (self.team == wave_team && waves)
            || !self.cores.is_empty()
            || !self.building_ids.is_empty()
            || (self.team == TEAM_NEOPLASTIC && !self.units.is_empty())
    }

    pub fn has_core(&self) -> bool {
        !self.cores.is_empty()
    }

    pub fn is_alive(&self) -> bool {
        self.has_core()
    }

    pub fn no_cores(&self) -> bool {
        self.cores.is_empty()
    }

    pub fn core(&self) -> Option<&CoreInfo> {
        self.cores.first()
    }

    pub fn add_plan_front(&mut self, plan: BlockPlan, check_previous: bool) -> Option<BlockPlan> {
        let removed = check_previous
            .then(|| self.remove_plan_at(plan.x as i32, plan.y as i32))
            .flatten();
        self.plans.insert(0, plan);
        removed
    }

    pub fn remove_plan_at(&mut self, x: i32, y: i32) -> Option<BlockPlan> {
        let index = self
            .plans
            .iter()
            .position(|plan| plan.x as i32 == x && plan.y as i32 == y)?;
        Some(self.plans.remove(index))
    }

    pub fn delete_plans_at_positions(&mut self, positions: &[i32]) -> Vec<BlockPlan> {
        let mut removed = Vec::new();
        let mut index = 0usize;
        while index < self.plans.len() {
            let should_remove = positions.iter().any(|position| {
                self.plans[index].x == point2_x(*position)
                    && self.plans[index].y == point2_y(*position)
            });
            if should_remove {
                let mut plan = self.plans.remove(index);
                plan.removed = true;
                removed.push(plan);
            } else {
                index += 1;
            }
        }
        removed
    }

    pub fn claim_front_plan<FPlaced, FUsable>(
        &mut self,
        mut already_placed: FPlaced,
        mut usable: FUsable,
    ) -> TeamPlanClaim
    where
        FPlaced: FnMut(&BlockPlan) -> bool,
        FUsable: FnMut(&BlockPlan) -> bool,
    {
        let Some(front) = self.plans.first() else {
            return TeamPlanClaim::NoPlans;
        };

        if already_placed(front) {
            return TeamPlanClaim::AlreadyPlaced(self.plans.remove(0));
        }

        let plan = self.plans.remove(0);
        let outcome = if usable(&plan) {
            TeamPlanClaim::Claimed(plan.clone())
        } else {
            TeamPlanClaim::Rotated(plan.clone())
        };
        self.plans.push(plan);
        outcome
    }

    pub fn claim_first_usable_plan<FUsable>(&mut self, mut usable: FUsable) -> TeamPlanClaim
    where
        FUsable: FnMut(&BlockPlan) -> bool,
    {
        if self.plans.is_empty() {
            return TeamPlanClaim::NoPlans;
        }

        let Some(index) = self.plans.iter().position(|plan| usable(plan)) else {
            return TeamPlanClaim::NoUsablePlan;
        };

        let plan = self.plans.remove(index);
        self.plans.push(plan.clone());
        TeamPlanClaim::Claimed(plan)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Teams {
    map: Vec<Option<TeamData>>,
    active: Vec<u8>,
    present: Vec<u8>,
    bosses: Vec<i32>,
    waves: bool,
    wave_team: u8,
    default_team: u8,
}

impl Default for Teams {
    fn default() -> Self {
        Self::new(TEAM_CRUX, false, TEAM_CRUX, 1)
    }
}

impl Teams {
    pub fn new(initial_active_team: u8, waves: bool, wave_team: u8, default_team: u8) -> Self {
        let mut teams = Self {
            map: vec![None; TEAM_COUNT],
            active: Vec::new(),
            present: Vec::new(),
            bosses: Vec::new(),
            waves,
            wave_team,
            default_team,
        };
        teams.active.push(initial_active_team);
        teams.get(initial_active_team);
        teams.update_enemies();
        teams
    }

    pub fn get(&mut self, team: u8) -> &mut TeamData {
        self.map[team as usize].get_or_insert_with(|| TeamData::new(team))
    }

    pub fn get_or_null(&self, team: u8) -> Option<&TeamData> {
        self.map.get(team as usize).and_then(Option::as_ref)
    }

    pub fn get_or_null_mut(&mut self, team: u8) -> Option<&mut TeamData> {
        self.map.get_mut(team as usize).and_then(Option::as_mut)
    }

    pub fn active_ids(&self) -> &[u8] {
        &self.active
    }

    pub fn present_ids(&self) -> &[u8] {
        &self.present
    }

    pub fn bosses(&self) -> &[i32] {
        &self.bosses
    }

    pub fn player_cores(&mut self) -> &[CoreInfo] {
        &self.get(self.default_team).cores
    }

    pub fn cores(&mut self, team: u8) -> &[CoreInfo] {
        &self.get(team).cores
    }

    pub fn is_active(&mut self, team: u8) -> bool {
        let waves = self.waves;
        let wave_team = self.wave_team;
        self.get(team).active(waves, wave_team)
    }

    pub fn can_interact(team: u8, other: u8) -> bool {
        team == other || other == 0
    }

    pub fn set_rules(&mut self, waves: bool, wave_team: u8, default_team: u8) {
        self.waves = waves;
        self.wave_team = wave_team;
        self.default_team = default_team;
        self.refresh_active();
    }

    pub fn get_active(&mut self) -> &[u8] {
        self.refresh_active();
        &self.active
    }

    pub fn replace_plans(&mut self, plans_by_team: impl IntoIterator<Item = (u8, Vec<BlockPlan>)>) {
        for data in self.map.iter_mut().flatten() {
            data.plans.clear();
        }

        for (team, plans) in plans_by_team {
            self.get(team).plans = plans;
        }
    }

    pub fn replace_core_items(&mut self, team: u8, items: BTreeMap<i16, i32>) {
        self.get(team).core_items = items;
    }

    pub fn add_plan_front(
        &mut self,
        team: u8,
        plan: BlockPlan,
        check_previous: bool,
    ) -> Option<BlockPlan> {
        self.get(team).add_plan_front(plan, check_previous)
    }

    pub fn remove_plan_at(&mut self, team: u8, x: i32, y: i32) -> Option<BlockPlan> {
        self.get(team).remove_plan_at(x, y)
    }

    pub fn delete_plans_at_positions(&mut self, team: u8, positions: &[i32]) -> Vec<BlockPlan> {
        self.get(team).delete_plans_at_positions(positions)
    }

    pub fn claim_front_plan<FPlaced, FUsable>(
        &mut self,
        team: u8,
        already_placed: FPlaced,
        usable: FUsable,
    ) -> TeamPlanClaim
    where
        FPlaced: FnMut(&BlockPlan) -> bool,
        FUsable: FnMut(&BlockPlan) -> bool,
    {
        self.get(team).claim_front_plan(already_placed, usable)
    }

    pub fn claim_first_usable_plan<FUsable>(&mut self, team: u8, usable: FUsable) -> TeamPlanClaim
    where
        FUsable: FnMut(&BlockPlan) -> bool,
    {
        self.get(team).claim_first_usable_plan(usable)
    }

    pub fn update_active(&mut self, team: u8) {
        if self.is_active(team) && !self.active.contains(&team) {
            self.active.push(team);
            self.update_enemies();
        }
    }

    pub fn register_core(&mut self, core: CoreInfo) {
        let data = self.get(core.team);
        if !data.cores.iter().any(|existing| existing.id == core.id) {
            data.cores.push(core);
        }
        data.last_core = data.cores.first().copied();
        self.update_active(core.team);
    }

    pub fn unregister_core(&mut self, team: u8, core_id: i32) -> Option<CoreInfo> {
        let removed = {
            let data = self.get(team);
            let index = data.cores.iter().position(|core| core.id == core_id)?;
            let removed = data.cores.remove(index);
            if data.cores.is_empty() {
                data.last_core = Some(removed);
            } else {
                data.last_core = data.cores.first().copied();
            }
            removed
        };
        self.refresh_active();
        Some(removed)
    }

    pub fn closest_core(&mut self, team: u8, x: f32, y: f32) -> Option<CoreInfo> {
        self.get(team)
            .cores
            .iter()
            .min_by(|a, b| a.dst2(x, y).total_cmp(&b.dst2(x, y)))
            .copied()
    }

    pub fn closest_enemy_core(&mut self, team: u8, x: f32, y: f32) -> Option<CoreInfo> {
        let enemies = self
            .get_or_null(team)
            .map(|data| data.core_enemies.clone())
            .unwrap_or_default();

        enemies
            .into_iter()
            .flat_map(|enemy| {
                self.get_or_null(enemy)
                    .map(|data| data.cores.clone())
                    .unwrap_or_default()
            })
            .min_by(|a, b| a.dst2(x, y).total_cmp(&b.dst2(x, y)))
    }

    pub fn update_team_stats(
        &mut self,
        present_building_teams: impl IntoIterator<Item = u8>,
        unit_records: impl IntoIterator<Item = (u8, i32, i32, bool)>,
        player_teams: impl IntoIterator<Item = u8>,
    ) {
        self.present.clear();
        self.bosses.clear();

        for data in self.map.iter_mut().flatten() {
            data.present_flag = !data.building_ids.is_empty();
            data.unit_count = 0;
            data.type_counts.clear();
            data.units.clear();
            data.players.clear();
            if !data.cores.is_empty() {
                data.last_core = data.cores.first().copied();
            }
        }

        for team in present_building_teams {
            self.get(team).present_flag = true;
        }

        for (team, unit_id, unit_type_id, boss) in unit_records {
            let wave_team = self.wave_team;
            let data = self.get(team);
            data.units.push(unit_id);
            data.present_flag = true;
            data.update_count(unit_type_id, 1);
            if team == wave_team && boss {
                self.bosses.push(unit_id);
            }
        }

        for team in player_teams {
            self.get(team).players.push(team as i32);
        }

        for team in (0..TEAM_COUNT).map(|team| team as u8) {
            let Some(data) = self.get_or_null(team) else {
                continue;
            };
            if data.present_flag || data.active(self.waves, self.wave_team) {
                self.present.push(team);
            }
        }
    }

    fn refresh_active(&mut self) {
        let waves = self.waves;
        let wave_team = self.wave_team;
        self.active.retain(|team| {
            self.map
                .get(*team as usize)
                .and_then(Option::as_ref)
                .is_some_and(|data| data.active(waves, wave_team))
        });
        if self.waves && !self.active.contains(&self.wave_team) {
            self.active.push(self.wave_team);
            self.get(self.wave_team);
        }
        self.update_enemies();
    }

    fn update_enemies(&mut self) {
        if self.waves && !self.active.contains(&self.wave_team) {
            self.active.push(self.wave_team);
            self.get(self.wave_team);
        }

        let active: BTreeSet<u8> = self.active.iter().copied().collect();
        for team in active.iter().copied() {
            let enemies = active
                .iter()
                .copied()
                .filter(|other| *other != team)
                .collect::<Vec<_>>();
            self.get(team).core_enemies = enemies;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::game::{TEAM_CRUX, TEAM_DERELICT, TEAM_MALIS, TEAM_SHARDED};

    #[test]
    fn teams_starts_with_crux_active_like_java_constructor() {
        let teams = Teams::default();
        assert_eq!(teams.active_ids(), &[TEAM_CRUX]);
        assert!(teams.get_or_null(TEAM_CRUX).is_some());
        assert!(Teams::can_interact(TEAM_SHARDED, TEAM_SHARDED));
        assert!(Teams::can_interact(TEAM_SHARDED, TEAM_DERELICT));
        assert!(!Teams::can_interact(TEAM_SHARDED, TEAM_CRUX));
    }

    #[test]
    fn register_and_unregister_core_updates_active_and_enemy_lists() {
        let mut teams = Teams::new(TEAM_CRUX, false, TEAM_CRUX, TEAM_SHARDED);
        teams.register_core(CoreInfo::new(10, TEAM_SHARDED, 0.0, 0.0));
        teams.register_core(CoreInfo::new(20, TEAM_MALIS, 8.0, 0.0));

        assert_eq!(teams.get_active(), &[TEAM_SHARDED, TEAM_MALIS]);
        assert_eq!(
            teams
                .get_or_null(TEAM_SHARDED)
                .unwrap()
                .core_enemies
                .as_slice(),
            &[TEAM_MALIS]
        );
        assert_eq!(teams.closest_core(TEAM_MALIS, 10.0, 0.0).unwrap().id, 20);
        assert_eq!(
            teams
                .closest_enemy_core(TEAM_SHARDED, 9.0, 0.0)
                .unwrap()
                .team,
            TEAM_MALIS
        );

        let removed = teams.unregister_core(TEAM_MALIS, 20).unwrap();
        assert_eq!(removed.id, 20);
        assert_eq!(teams.get_active(), &[TEAM_SHARDED]);
        assert!(teams
            .get_or_null(TEAM_SHARDED)
            .unwrap()
            .core_enemies
            .is_empty());
    }

    #[test]
    fn wave_team_is_active_when_waves_are_enabled() {
        let mut teams = Teams::new(TEAM_SHARDED, true, TEAM_CRUX, TEAM_SHARDED);
        assert!(teams.is_active(TEAM_CRUX));
        assert_eq!(teams.get_active(), &[TEAM_CRUX]);
    }

    #[test]
    fn team_data_counts_units_blocks_and_presence_like_java_stats_pass() {
        let mut teams = Teams::default();
        teams.get(TEAM_SHARDED).add_building(7, "router");
        teams.update_team_stats(
            [TEAM_MALIS],
            [
                (TEAM_CRUX, 100, 3, true),
                (TEAM_SHARDED, 101, 3, false),
                (TEAM_SHARDED, 102, 4, false),
            ],
            [TEAM_SHARDED],
        );

        let sharded = teams.get_or_null(TEAM_SHARDED).unwrap();
        assert_eq!(sharded.get_count("router"), 1);
        assert_eq!(sharded.unit_count, 2);
        assert_eq!(sharded.count_type(3), 1);
        assert_eq!(sharded.count_type(4), 1);
        assert_eq!(sharded.players, vec![TEAM_SHARDED as i32]);
        assert_eq!(teams.bosses(), &[100]);
        assert!(teams.present_ids().contains(&TEAM_SHARDED));
        assert!(teams.present_ids().contains(&TEAM_CRUX));
        assert!(teams.present_ids().contains(&TEAM_MALIS));
    }

    #[test]
    fn block_plan_keeps_java_field_shape() {
        let mut plan = BlockPlan::new(5, 6, 2, "router", Some("config".into()));
        assert_eq!(plan.x, 5);
        assert_eq!(plan.y, 6);
        assert_eq!(plan.rotation, 2);
        assert_eq!(plan.block, "router");
        assert!(!plan.removed);
        plan.removed = true;
        assert!(plan.removed);
    }

    #[test]
    fn replace_plans_clears_existing_entries_and_replaces_target_teams() {
        let mut teams = Teams::default();
        teams
            .get(crate::mindustry::game::TEAM_SHARDED)
            .plans
            .push(BlockPlan::new(1, 2, 0, "duo", None));
        teams
            .get(crate::mindustry::game::TEAM_CRUX)
            .plans
            .push(BlockPlan::new(3, 4, 1, "router", Some("cfg".into())));

        teams.replace_plans([
            (
                crate::mindustry::game::TEAM_MALIS,
                vec![BlockPlan::new(5, 6, 2, "wall", Some("9".into()))],
            ),
            (crate::mindustry::game::TEAM_SHARDED, Vec::new()),
        ]);

        assert!(teams
            .get_or_null(crate::mindustry::game::TEAM_CRUX)
            .unwrap()
            .plans
            .is_empty());
        assert!(teams
            .get_or_null(crate::mindustry::game::TEAM_SHARDED)
            .unwrap()
            .plans
            .is_empty());
        assert_eq!(
            teams
                .get_or_null(crate::mindustry::game::TEAM_MALIS)
                .unwrap()
                .plans,
            vec![BlockPlan::new(5, 6, 2, "wall", Some("9".into()))]
        );
    }

    #[test]
    fn team_plan_queue_claims_and_rotates_like_builder_ai() {
        let mut teams = Teams::default();
        teams.replace_plans([(
            TEAM_SHARDED,
            vec![
                BlockPlan::new(1, 1, 0, "duo", None),
                BlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
            ],
        )]);

        let placed = teams.claim_front_plan(
            TEAM_SHARDED,
            |plan| plan.block == "duo",
            |_| panic!("already placed should skip validity check"),
        );
        assert_eq!(
            placed,
            TeamPlanClaim::AlreadyPlaced(BlockPlan::new(1, 1, 0, "duo", None))
        );
        assert_eq!(
            teams.get_or_null(TEAM_SHARDED).unwrap().plans,
            vec![BlockPlan::new(2, 2, 1, "router", Some("cfg".into()))]
        );

        let rotated = teams.claim_front_plan(TEAM_SHARDED, |_| false, |_| false);
        assert_eq!(
            rotated,
            TeamPlanClaim::Rotated(BlockPlan::new(2, 2, 1, "router", Some("cfg".into())))
        );
        assert_eq!(
            teams.get_or_null(TEAM_SHARDED).unwrap().plans,
            vec![BlockPlan::new(2, 2, 1, "router", Some("cfg".into()))]
        );

        let claimed = teams.claim_front_plan(TEAM_SHARDED, |_| false, |_| true);
        assert_eq!(
            claimed,
            TeamPlanClaim::Claimed(BlockPlan::new(2, 2, 1, "router", Some("cfg".into())))
        );
        assert_eq!(
            teams.get_or_null(TEAM_SHARDED).unwrap().plans,
            vec![BlockPlan::new(2, 2, 1, "router", Some("cfg".into()))]
        );
    }

    #[test]
    fn team_plan_queue_claims_first_usable_like_hold_builder_or_build_turret() {
        let mut teams = Teams::default();
        teams.replace_plans([(
            TEAM_SHARDED,
            vec![
                BlockPlan::new(1, 1, 0, "duo", None),
                BlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
                BlockPlan::new(3, 3, 2, "wall", None),
            ],
        )]);

        let claimed = teams.claim_first_usable_plan(TEAM_SHARDED, |plan| plan.block == "router");

        assert_eq!(
            claimed,
            TeamPlanClaim::Claimed(BlockPlan::new(2, 2, 1, "router", Some("cfg".into())))
        );
        assert_eq!(
            teams.get_or_null(TEAM_SHARDED).unwrap().plans,
            vec![
                BlockPlan::new(1, 1, 0, "duo", None),
                BlockPlan::new(3, 3, 2, "wall", None),
                BlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
            ]
        );

        assert_eq!(
            teams.claim_first_usable_plan(TEAM_SHARDED, |plan| plan.block == "missing"),
            TeamPlanClaim::NoUsablePlan
        );
    }

    #[test]
    fn team_plan_queue_add_front_deduplicates_and_remove_matches_building_comp() {
        let mut teams = Teams::default();
        teams.replace_plans([(
            TEAM_SHARDED,
            vec![
                BlockPlan::new(1, 1, 0, "duo", None),
                BlockPlan::new(2, 2, 1, "router", None),
            ],
        )]);

        let removed = teams.add_plan_front(
            TEAM_SHARDED,
            BlockPlan::new(2, 2, 3, "scatter", Some("cfg".into())),
            true,
        );

        assert_eq!(removed, Some(BlockPlan::new(2, 2, 1, "router", None)));
        assert_eq!(
            teams.get_or_null(TEAM_SHARDED).unwrap().plans,
            vec![
                BlockPlan::new(2, 2, 3, "scatter", Some("cfg".into())),
                BlockPlan::new(1, 1, 0, "duo", None),
            ]
        );

        assert_eq!(
            teams.remove_plan_at(TEAM_SHARDED, 1, 1),
            Some(BlockPlan::new(1, 1, 0, "duo", None))
        );
        assert_eq!(
            teams.get_or_null(TEAM_SHARDED).unwrap().plans,
            vec![BlockPlan::new(2, 2, 3, "scatter", Some("cfg".into()))]
        );
    }

    #[test]
    fn team_plan_claim_can_feed_builder_plan_conversion_helpers() {
        let claim = TeamPlanClaim::Claimed(BlockPlan::new(4, 5, 1, "router", Some("cfg".into())));
        let plan = claim
            .into_claimed_plan()
            .expect("claimed plan should be extractable");

        let build_plan =
            crate::mindustry::entities::comp::builder::BuilderComp::build_plan_from_team_plan(
                &plan,
            );

        assert_eq!(build_plan.x, 4);
        assert_eq!(build_plan.y, 5);
        assert_eq!(build_plan.rotation, 1);
        assert_eq!(build_plan.block.as_deref(), Some("router"));
        assert_eq!(
            build_plan.config,
            crate::mindustry::io::TypeValue::String("cfg".into())
        );
    }

    #[test]
    fn delete_plans_at_positions_marks_removed_and_matches_java_delete_plans() {
        let mut teams = Teams::default();
        teams.replace_plans([
            (
                TEAM_SHARDED,
                vec![
                    BlockPlan::new(1, 1, 0, "duo", None),
                    BlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
                ],
            ),
            (TEAM_MALIS, vec![BlockPlan::new(2, 2, 0, "wall", None)]),
        ]);

        let removed = teams.delete_plans_at_positions(
            TEAM_SHARDED,
            &[
                crate::mindustry::world::point2_pack(2, 2),
                crate::mindustry::world::point2_pack(9, 9),
            ],
        );

        assert_eq!(removed.len(), 1);
        assert!(removed[0].removed);
        assert_eq!(
            removed[0],
            BlockPlan {
                removed: true,
                ..BlockPlan::new(2, 2, 1, "router", Some("cfg".into()))
            }
        );
        assert_eq!(
            teams.get_or_null(TEAM_SHARDED).unwrap().plans,
            vec![BlockPlan::new(1, 1, 0, "duo", None)]
        );
        assert_eq!(
            teams.get_or_null(TEAM_MALIS).unwrap().plans,
            vec![BlockPlan::new(2, 2, 0, "wall", None)]
        );
    }

    #[test]
    fn replace_core_items_keeps_snapshot_counts_by_item_id() {
        let mut teams = Teams::default();
        teams.replace_core_items(
            crate::mindustry::game::TEAM_SHARDED,
            BTreeMap::from([(0, 75), (3, 12)]),
        );
        teams.replace_core_items(crate::mindustry::game::TEAM_CRUX, BTreeMap::from([(1, 5)]));

        assert_eq!(
            teams
                .get_or_null(crate::mindustry::game::TEAM_SHARDED)
                .unwrap()
                .core_items,
            BTreeMap::from([(0, 75), (3, 12)])
        );
        assert_eq!(
            teams
                .get_or_null(crate::mindustry::game::TEAM_CRUX)
                .unwrap()
                .core_items,
            BTreeMap::from([(1, 5)])
        );

        teams.replace_core_items(crate::mindustry::game::TEAM_SHARDED, BTreeMap::new());
        assert!(teams
            .get_or_null(crate::mindustry::game::TEAM_SHARDED)
            .unwrap()
            .core_items
            .is_empty());
    }
}
