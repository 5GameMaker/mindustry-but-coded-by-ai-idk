//! Builder component shell mirroring upstream `mindustry.entities.comp.BuilderComp`.
//!
//! The Java implementation mixes queue management, world validation, resource
//! checks, build effects and rendering. This Rust port starts with the pure
//! queue/state portion and keeps world-dependent decisions behind lightweight
//! snapshots so later runtime ports can plug in real `World`/`Building` state.

use std::collections::VecDeque;

use crate::mindustry::ai::{
    builder_ai_idle_retreat, claim_builder_ai_hold_plan, claim_builder_ai_rebuild_plan,
    prebuild_ai_look_action, prebuild_ai_should_boost, prebuild_ai_update_movement_tick,
    sync_builder_ai_follow_plan, validate_builder_ai_current_plan, BuilderAiFollowAction,
    BuilderAiFollowSync, BuilderAiPlanAction, BuilderAiPlanValidation, BuilderAiRetreatDecision,
    PrebuildAiBlockInfo, PrebuildAiLookAction, PrebuildAiPlanSnapshot, PrebuildAiTickDecision,
    BUILDER_AI_DEFAULT_REBUILD_PERIOD,
};
use crate::mindustry::ctype::ContentId;
use crate::mindustry::entities::units::BuildPlan;
use crate::mindustry::game::TEAM_DERELICT;
use crate::mindustry::game::{BlockPlan as TeamBlockPlan, TeamData, TeamPlanClaim};
use crate::mindustry::io::{TeamId, TypeValue};
use crate::mindustry::r#type::UnitType;

pub const BUILDER_AI_BUILD_RADIUS: f32 = 1500.0;
pub const BUILDER_AI_RETREAT_DST: f32 = 110.0;
pub const BUILDER_AI_RETREAT_DELAY: f32 = 60.0 * 2.0;
pub const BUILDER_AI_ASSIST_EXTRA_RANGE: f32 = 60.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderRequirement {
    pub item: String,
    pub amount: i32,
}

impl BuilderRequirement {
    pub fn new(item: impl Into<String>, amount: i32) -> Self {
        Self {
            item: item.into(),
            amount,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderBlockInfo {
    pub name: String,
    pub rotate: bool,
    pub is_overlay: bool,
    pub is_floor: bool,
    pub requirements: Vec<BuilderRequirement>,
}

impl BuilderBlockInfo {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rotate: true,
            is_overlay: false,
            is_floor: false,
            requirements: Vec::new(),
        }
    }

    pub fn overlay(name: impl Into<String>) -> Self {
        Self {
            rotate: false,
            is_overlay: true,
            is_floor: false,
            requirements: Vec::new(),
            name: name.into(),
        }
    }

    pub fn floor(name: impl Into<String>) -> Self {
        Self {
            rotate: false,
            is_overlay: false,
            is_floor: true,
            requirements: Vec::new(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuilderTileSnapshot {
    pub block: Option<String>,
    pub overlay: Option<String>,
    pub floor: Option<String>,
    pub build_block: Option<String>,
    pub build_team: Option<TeamId>,
    pub build_tile_x: i32,
    pub build_tile_y: i32,
    pub build_rotation: Option<i32>,
    pub construct_progress: Option<f32>,
}

impl BuilderTileSnapshot {
    pub fn air() -> Self {
        Self {
            block: None,
            overlay: None,
            floor: None,
            build_block: None,
            build_team: None,
            build_tile_x: 0,
            build_tile_y: 0,
            build_rotation: None,
            construct_progress: None,
        }
    }

    pub fn block(name: impl Into<String>) -> Self {
        Self {
            block: Some(name.into()),
            ..Self::air()
        }
    }

    pub fn with_build(
        mut self,
        block: impl Into<String>,
        team: TeamId,
        tile_x: i32,
        tile_y: i32,
        rotation: i32,
    ) -> Self {
        self.build_block = Some(block.into());
        self.build_team = Some(team);
        self.build_tile_x = tile_x;
        self.build_tile_y = tile_y;
        self.build_rotation = Some(rotation);
        self
    }

    pub fn with_overlay(mut self, overlay: impl Into<String>) -> Self {
        self.overlay = Some(overlay.into());
        self
    }

    pub fn with_floor(mut self, floor: impl Into<String>) -> Self {
        self.floor = Some(floor.into());
        self
    }

    pub fn with_construct_progress(mut self, progress: f32) -> Self {
        self.construct_progress = Some(progress);
        self
    }

    fn is_same_derelict(&self, plan: &BuildPlan) -> bool {
        plan.block.as_deref() == self.build_block.as_deref()
            && self.build_tile_x == plan.x
            && self.build_tile_y == plan.y
            && self.build_team == Some(TeamId(TEAM_DERELICT))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuilderSkipContext {
    pub rules_infinite_resources: bool,
    pub team_infinite_resources: bool,
    pub core_available: bool,
    pub plan_is_rotation: bool,
    pub plan_is_derelict_repair: bool,
    pub has_all_requirements: bool,
    pub has_missing_limited_item: bool,
}

impl BuilderSkipContext {
    pub const fn finite_with_core() -> Self {
        Self {
            rules_infinite_resources: false,
            team_infinite_resources: false,
            core_available: true,
            plan_is_rotation: false,
            plan_is_derelict_repair: false,
            has_all_requirements: true,
            has_missing_limited_item: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuilderComp {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub build_speed_multiplier: f32,
    pub type_info: UnitType,
    pub team: TeamId,
    pub plans: VecDeque<BuildPlan>,
    pub update_building: bool,
    pub build_counter: f32,
    pub last_active: Option<BuildPlan>,
    pub last_size: i32,
    pub build_alpha: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuilderAiRuntimeState {
    pub assist_following: Option<i32>,
    pub following: Option<i32>,
    pub enemy: Option<i32>,
    pub last_plan: Option<TeamBlockPlan>,
    pub retreat_timer: f32,
    pub always_flee: bool,
    pub only_assist: bool,
    pub flee_range: f32,
    pub rebuild_period: f32,
}

impl BuilderAiRuntimeState {
    pub fn new(always_flee: bool, flee_range: f32) -> Self {
        Self {
            always_flee,
            flee_range,
            ..Self::default()
        }
    }
}

impl Default for BuilderAiRuntimeState {
    fn default() -> Self {
        Self {
            assist_following: None,
            following: None,
            enemy: None,
            last_plan: None,
            retreat_timer: 0.0,
            always_flee: false,
            only_assist: false,
            flee_range: 370.0,
            rebuild_period: BUILDER_AI_DEFAULT_REBUILD_PERIOD,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrebuildAiRuntimeState {
    pub collecting_items: bool,
    pub mining: bool,
    pub last_target_item: Option<ContentId>,
    pub ore: Option<i32>,
    pub collect_block: Option<PrebuildAiBlockInfo>,
    pub last_plan: Option<TeamBlockPlan>,
}

impl Default for PrebuildAiRuntimeState {
    fn default() -> Self {
        Self {
            collecting_items: false,
            mining: false,
            last_target_item: None,
            ore: None,
            collect_block: None,
            last_plan: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrebuildAiRuntimeInput {
    pub unit_flying: bool,
    pub has_target: bool,
    pub should_shoot: bool,
    pub boost_when_building: bool,
    pub floor_is_duct: bool,
    pub floor_damage_taken: f32,
    pub floor_is_deep: bool,
    pub within_current_plan_range: bool,
    pub construct_current_matches: bool,
    pub timer_find_ready: bool,
}

impl Default for PrebuildAiRuntimeInput {
    fn default() -> Self {
        Self {
            unit_flying: false,
            has_target: false,
            should_shoot: false,
            boost_when_building: false,
            floor_is_duct: false,
            floor_damage_taken: 0.0,
            floor_is_deep: false,
            within_current_plan_range: true,
            construct_current_matches: false,
            timer_find_ready: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrebuildAiRuntimeStep {
    pub decision: PrebuildAiTickDecision,
    pub added_build_plan: Option<BuildPlan>,
    pub update_building: bool,
    pub collecting_items: bool,
    pub last_plan: Option<TeamBlockPlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuilderAiRuntimeInput {
    pub delta: f32,
    pub unit_flying: bool,
    pub has_target: bool,
    pub should_shoot: bool,
    pub boost_when_building: bool,
    pub floor_is_duct: bool,
    pub floor_damage_taken: f32,
    pub floor_is_deep: bool,
    pub hold_position: bool,
    pub infinite_resources: bool,
    pub has_core: bool,
    pub within_retreat_distance: bool,
    pub sensed_enemy: Option<i32>,
    pub timer_enemy_ready: bool,
    pub timer_follow_ready: bool,
    pub timer_find_ready: bool,
    pub within_current_plan_range: bool,
    pub within_hold_range: bool,
    pub construct_current_matches: bool,
    pub conflicting_breaker: bool,
    pub following_valid: bool,
    pub following_actively_building: bool,
    pub following_plan: Option<BuildPlan>,
    pub assist_valid: bool,
    pub assist_actively_building: bool,
    pub assist_plan: Option<BuildPlan>,
    pub assist_hit_size: f32,
    pub within_assist_range: bool,
    pub next_following: Option<i32>,
    pub next_assist_following: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuilderAiFollowCandidate {
    pub unit_id: i32,
    pub can_build: bool,
    pub is_self: bool,
    pub actively_building: bool,
    pub plan: Option<BuildPlan>,
    pub construct_dst: Option<f32>,
    pub construct_build_cost: Option<f32>,
}

impl BuilderAiFollowCandidate {
    pub fn new(
        unit_id: i32,
        plan: BuildPlan,
        construct_dst: f32,
        construct_build_cost: f32,
    ) -> Self {
        Self {
            unit_id,
            can_build: true,
            is_self: false,
            actively_building: true,
            plan: Some(plan),
            construct_dst: Some(construct_dst),
            construct_build_cost: Some(construct_build_cost),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuilderAiAssistCandidate {
    pub unit_id: i32,
    pub dead: bool,
    pub is_builder: bool,
    pub same_team: bool,
    pub dst2: f32,
}

impl BuilderAiAssistCandidate {
    pub const fn new(unit_id: i32, dst2: f32) -> Self {
        Self {
            unit_id,
            dead: false,
            is_builder: true,
            same_team: true,
            dst2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuilderAiFollowSearch {
    pub following: Option<i32>,
    pub assist_following: Option<i32>,
    pub found: bool,
}

impl BuilderAiRuntimeInput {
    pub fn with_follow_search(mut self, search: BuilderAiFollowSearch) -> Self {
        self.next_following = search.following;
        self.next_assist_following = search.assist_following;
        self
    }
}

impl Default for BuilderAiRuntimeInput {
    fn default() -> Self {
        Self {
            delta: 1.0,
            unit_flying: false,
            has_target: false,
            should_shoot: false,
            boost_when_building: false,
            floor_is_duct: false,
            floor_damage_taken: 0.0,
            floor_is_deep: false,
            hold_position: false,
            infinite_resources: false,
            has_core: false,
            within_retreat_distance: false,
            sensed_enemy: None,
            timer_enemy_ready: false,
            timer_follow_ready: false,
            timer_find_ready: false,
            within_current_plan_range: true,
            within_hold_range: true,
            construct_current_matches: false,
            conflicting_breaker: false,
            following_valid: false,
            following_actively_building: false,
            following_plan: None,
            assist_valid: false,
            assist_actively_building: false,
            assist_plan: None,
            assist_hit_size: 0.0,
            within_assist_range: true,
            next_following: None,
            next_assist_following: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderAiRuntimeBranch {
    FollowingInvalid,
    FollowingCurrentPlan,
    Retreat,
    CurrentPlan,
    AssistMove,
    FindNewPlan,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuilderAiMoveToPlan {
    pub range: f32,
    pub margin: f32,
    pub moving: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuilderAiMoveToAssist {
    pub range: f32,
    pub moving: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuilderAiRuntimeStep {
    pub look_action: PrebuildAiLookAction,
    pub update_building: bool,
    pub branch: BuilderAiRuntimeBranch,
    pub follow_sync: Option<BuilderAiFollowSync>,
    pub retreat: Option<BuilderAiRetreatDecision>,
    pub current_plan_validation: Option<BuilderAiPlanValidation>,
    pub move_to_plan: Option<BuilderAiMoveToPlan>,
    pub move_to_assist: Option<BuilderAiMoveToAssist>,
    pub claimed_team_plan: Option<TeamPlanClaim>,
    pub added_build_plan: Option<BuildPlan>,
    pub boosting: Option<bool>,
    pub moving: bool,
    pub following: Option<i32>,
    pub assist_following: Option<i32>,
    pub enemy: Option<i32>,
    pub retreat_timer: f32,
    pub last_plan: Option<TeamBlockPlan>,
}

impl BuilderComp {
    pub fn new(type_info: UnitType, team: TeamId) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            build_speed_multiplier: 1.0,
            type_info,
            team,
            plans: VecDeque::new(),
            update_building: true,
            build_counter: 0.0,
            last_active: None,
            last_size: 0,
            build_alpha: 0.0,
        }
    }

    pub fn build_plan_from_team_plan(plan: &TeamBlockPlan) -> BuildPlan {
        let mut build_plan = BuildPlan::new_place(
            plan.x as i32,
            plan.y as i32,
            plan.rotation as i32,
            plan.block.clone(),
        );
        if let Some(config) = &plan.config {
            build_plan.config = TypeValue::String(config.clone());
        }
        build_plan
    }

    pub fn add_team_plan(&mut self, place: &TeamBlockPlan) -> bool {
        self.add_build(Self::build_plan_from_team_plan(place))
    }

    pub fn add_claimed_team_plan(&mut self, claim: TeamPlanClaim) -> Option<BuildPlan> {
        let plan = claim.into_claimed_plan()?;
        let build_plan = Self::build_plan_from_team_plan(&plan);
        self.add_build(build_plan.clone()).then_some(build_plan)
    }

    pub fn can_build(&self) -> bool {
        self.type_info.build_speed > 0.0 && self.build_speed_multiplier > 0.0
    }

    pub fn validate_plans_with<T, B>(&mut self, mut tile_at: T, mut block_info: B) -> usize
    where
        T: FnMut(i32, i32) -> Option<BuilderTileSnapshot>,
        B: FnMut(&str) -> Option<BuilderBlockInfo>,
    {
        let mut removed = 0;
        let mut kept = VecDeque::with_capacity(self.plans.len());

        while let Some(plan) = self.plans.pop_front() {
            let tile = tile_at(plan.x, plan.y);
            let info = plan
                .block
                .as_deref()
                .and_then(|block| block_info(block))
                .unwrap_or_else(|| {
                    plan.block
                        .as_deref()
                        .map(BuilderBlockInfo::new)
                        .unwrap_or_else(|| BuilderBlockInfo::new(""))
                });

            if Self::plan_invalid(&plan, tile.as_ref(), &info) {
                removed += 1;
            } else {
                kept.push_back(plan);
            }
        }

        self.plans = kept;
        removed
    }

    pub fn plan_invalid(
        plan: &BuildPlan,
        tile: Option<&BuilderTileSnapshot>,
        block_info: &BuilderBlockInfo,
    ) -> bool {
        let Some(tile) = tile else {
            return true;
        };

        if plan.breaking {
            return tile.block.is_none();
        }

        let Some(block) = plan.block.as_deref() else {
            return true;
        };

        let same_derelict = tile.is_same_derelict(plan);
        let rotation_done = tile.build_rotation == Some(plan.rotation) && !same_derelict;
        let same_block = tile.block.as_deref() == Some(block) && !same_derelict;
        let same_overlay = block_info.is_overlay && tile.overlay.as_deref() == Some(block);
        let same_floor = block_info.is_floor && tile.floor.as_deref() == Some(block);

        (rotation_done || !block_info.rotate) && (same_block || same_overlay || same_floor)
    }

    pub fn should_skip_plan(plan: &BuildPlan, context: BuilderSkipContext) -> bool {
        if context.rules_infinite_resources
            || context.team_infinite_resources
            || plan.breaking
            || !context.core_available
            || context.plan_is_rotation
            || context.plan_is_derelict_repair
        {
            return false;
        }

        (plan.stuck && !context.has_all_requirements) || context.has_missing_limited_item
    }

    pub fn remove_build(&mut self, x: i32, y: i32, breaking: bool) -> Option<BuildPlan> {
        let index = self
            .plans
            .iter()
            .position(|plan| plan.breaking == breaking && plan.x == x && plan.y == y)?;
        self.plans.remove(index)
    }

    pub fn is_building(&self) -> bool {
        !self.plans.is_empty()
    }

    pub fn clear_building(&mut self) {
        self.plans.clear();
    }

    pub fn add_build(&mut self, place: BuildPlan) -> bool {
        self.add_build_with_progress(place, true, None)
    }

    pub fn add_build_front(&mut self, place: BuildPlan) -> bool {
        self.add_build_with_progress(place, false, None)
    }

    pub fn add_build_with_progress(
        &mut self,
        mut place: BuildPlan,
        tail: bool,
        construct_progress: Option<f32>,
    ) -> bool {
        if !self.can_build() {
            return false;
        }

        if let Some(index) = self
            .plans
            .iter()
            .position(|plan| plan.x == place.x && plan.y == place.y)
        {
            self.plans.remove(index);
        }

        if let Some(progress) = construct_progress {
            place.progress = progress;
        }

        if tail {
            self.plans.push_back(place);
        } else {
            self.plans.push_front(place);
        }
        true
    }

    pub fn actively_building(&self, is_editor: bool, within_current_plan: bool) -> bool {
        if self.is_building() && !is_editor && !within_current_plan {
            return false;
        }
        self.is_building() && self.update_building
    }

    pub fn build_plan(&self) -> Option<&BuildPlan> {
        self.plans.front()
    }

    pub fn advance_build_counter(&mut self, delta: f32) -> f32 {
        self.build_counter += delta;
        if self.build_counter.is_nan() || self.build_counter.is_infinite() {
            self.build_counter = 0.0;
        }
        self.build_counter = self.build_counter.min(10.0);
        self.build_counter
    }

    pub fn builder_ai_plan_move(&self, within_current_plan_range: bool) -> BuilderAiMoveToPlan {
        let range = (self.type_info.build_range - self.type_info.hit_size * 2.0)
            .min(BUILDER_AI_BUILD_RADIUS);
        BuilderAiMoveToPlan {
            range,
            margin: 20.0,
            moving: !within_current_plan_range,
        }
    }

    pub fn builder_ai_assist_move(
        &self,
        assist_hit_size: f32,
        within_assist_range: bool,
    ) -> BuilderAiMoveToAssist {
        let range = assist_hit_size + self.type_info.hit_size / 2.0 + BUILDER_AI_ASSIST_EXTRA_RANGE;
        BuilderAiMoveToAssist {
            range,
            moving: !within_assist_range,
        }
    }

    pub fn choose_builder_ai_following<'a>(
        &self,
        candidates: impl IntoIterator<Item = &'a BuilderAiFollowCandidate>,
    ) -> Option<i32> {
        candidates
            .into_iter()
            .find(|candidate| {
                if !candidate.can_build || candidate.is_self || !candidate.actively_building {
                    return false;
                }

                if candidate.plan.is_none() {
                    return false;
                };

                let Some(construct_dst) = candidate.construct_dst else {
                    return false;
                };
                let Some(construct_build_cost) = candidate.construct_build_cost else {
                    return false;
                };

                let speed = self.type_info.speed.max(f32::EPSILON);
                let dist = (construct_dst - self.type_info.build_range).min(0.0);
                dist / speed < construct_build_cost * 0.9
            })
            .map(|candidate| candidate.unit_id)
    }

    pub fn choose_builder_ai_assist_following<'a>(
        candidates: impl IntoIterator<Item = &'a BuilderAiAssistCandidate>,
    ) -> Option<i32> {
        candidates
            .into_iter()
            .filter(|candidate| !candidate.dead && candidate.is_builder && candidate.same_team)
            .min_by(|left, right| left.dst2.total_cmp(&right.dst2))
            .map(|candidate| candidate.unit_id)
    }

    pub fn search_builder_ai_follow_targets<'a>(
        &self,
        follow_candidates: impl IntoIterator<Item = &'a BuilderAiFollowCandidate>,
        assist_candidates: impl IntoIterator<Item = &'a BuilderAiAssistCandidate>,
        only_assist: bool,
    ) -> BuilderAiFollowSearch {
        let following = self.choose_builder_ai_following(follow_candidates);
        let assist_following = only_assist
            .then(|| Self::choose_builder_ai_assist_following(assist_candidates))
            .flatten();
        BuilderAiFollowSearch {
            following,
            assist_following,
            found: following.is_some(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn apply_builder_ai_tick<
        FValidBreak,
        FValidPlace,
        FWithinTeamPlanRange,
        FAlreadyPlaced,
        FTeamValidPlace,
        FNearEnemy,
    >(
        &mut self,
        state: &mut BuilderAiRuntimeState,
        team_data: &mut TeamData,
        input: BuilderAiRuntimeInput,
        valid_break: FValidBreak,
        valid_place: FValidPlace,
        within_team_plan_range: FWithinTeamPlanRange,
        already_placed: FAlreadyPlaced,
        team_valid_place: FTeamValidPlace,
        near_enemy: FNearEnemy,
    ) -> BuilderAiRuntimeStep
    where
        FValidBreak: FnMut(&BuildPlan) -> bool,
        FValidPlace: FnMut(&BuildPlan) -> bool,
        FWithinTeamPlanRange: FnMut(&TeamBlockPlan) -> bool,
        FAlreadyPlaced: FnMut(&TeamBlockPlan) -> bool,
        FTeamValidPlace: FnMut(&TeamBlockPlan) -> bool,
        FNearEnemy: FnMut(&TeamBlockPlan) -> bool,
    {
        let look_action =
            prebuild_ai_look_action(input.has_target, input.should_shoot, input.unit_flying);
        self.update_building = true;

        if state.assist_following.is_some() && !input.assist_valid {
            state.assist_following = None;
        }

        if state.following.is_some() && !input.following_valid {
            state.following = None;
        }

        if state.assist_following.is_some() && input.assist_actively_building {
            state.following = state.assist_following;
        }

        let mut branch = BuilderAiRuntimeBranch::Idle;
        let mut follow_sync = None;
        let mut retreat = None;
        let mut current_plan_validation = None;
        let mut move_to_plan = None;
        let mut move_to_assist = None;
        let mut claimed_team_plan = None;
        let mut added_build_plan = None;
        let mut moving = false;

        if state.following.is_some() {
            let sync = sync_builder_ai_follow_plan(
                &mut self.plans,
                &mut state.last_plan,
                true,
                input.following_valid,
                input.following_actively_building,
                input.following_plan.clone(),
            );
            state.retreat_timer = 0.0;

            if sync.action == BuilderAiFollowAction::ClearInvalidFollower {
                state.following = None;
                follow_sync = Some(sync);
                branch = BuilderAiRuntimeBranch::FollowingInvalid;
                let boosting = (!input.unit_flying).then_some(prebuild_ai_should_boost(
                    input.boost_when_building,
                    false,
                    input.floor_is_duct,
                    input.floor_damage_taken,
                    input.floor_is_deep,
                ));
                return BuilderAiRuntimeStep {
                    look_action,
                    update_building: self.update_building,
                    branch,
                    follow_sync,
                    retreat,
                    current_plan_validation,
                    move_to_plan,
                    move_to_assist,
                    claimed_team_plan,
                    added_build_plan,
                    boosting,
                    moving,
                    following: state.following,
                    assist_following: state.assist_following,
                    enemy: state.enemy,
                    retreat_timer: state.retreat_timer,
                    last_plan: state.last_plan.clone(),
                };
            }

            follow_sync = Some(sync);
            branch = BuilderAiRuntimeBranch::FollowingCurrentPlan;
        } else if (self.build_plan().is_none() || state.always_flee) && !input.hold_position {
            if input.timer_enemy_ready {
                state.enemy = input.sensed_enemy;
            }

            let retreat_decision = builder_ai_idle_retreat(
                state.retreat_timer,
                input.delta,
                BUILDER_AI_RETREAT_DELAY,
                state.always_flee,
                state.enemy.is_some(),
                input.has_core,
                input.within_retreat_distance,
            );
            state.retreat_timer = retreat_decision.retreat_timer;
            if retreat_decision.clear_building {
                self.clear_building();
            }
            moving = retreat_decision.moving;
            retreat = Some(retreat_decision);
            if retreat_decision.clear_building || retreat_decision.move_to_core {
                branch = BuilderAiRuntimeBranch::Retreat;
            }
        }

        if self.build_plan().is_some() {
            if !state.always_flee {
                state.retreat_timer = 0.0;
            }

            let validation = validate_builder_ai_current_plan(
                &mut self.plans,
                &mut state.last_plan,
                input.hold_position,
                input.infinite_resources,
                input.within_hold_range,
                input.conflicting_breaker,
                input.construct_current_matches,
                valid_break,
                valid_place,
            );

            if let Some((x, y)) = validation.remove_team_plan_at {
                team_data.remove_plan_at(x, y);
            }

            if validation.action == BuilderAiPlanAction::Keep && !input.hold_position {
                let plan_move = self.builder_ai_plan_move(input.within_current_plan_range);
                moving = plan_move.moving;
                move_to_plan = Some(plan_move);
            }

            if branch == BuilderAiRuntimeBranch::Idle {
                branch = BuilderAiRuntimeBranch::CurrentPlan;
            }
            current_plan_validation = Some(validation);
        } else {
            if state.assist_following.is_some() && !input.hold_position {
                let assist_move =
                    self.builder_ai_assist_move(input.assist_hit_size, input.within_assist_range);
                moving = moving || assist_move.moving;
                move_to_assist = Some(assist_move);
                branch = BuilderAiRuntimeBranch::AssistMove;
            }

            if input.timer_follow_ready {
                if let Some(following) = input.next_following {
                    state.following = Some(following);
                }

                if state.only_assist {
                    state.assist_following = input.next_assist_following;
                }
            }

            if !state.only_assist
                && !team_data.plans.is_empty()
                && state.following.is_none()
                && input.timer_find_ready
            {
                let claim = if input.hold_position {
                    claim_builder_ai_hold_plan(
                        team_data,
                        input.infinite_resources,
                        within_team_plan_range,
                        team_valid_place,
                    )
                } else {
                    claim_builder_ai_rebuild_plan(
                        team_data,
                        state.always_flee,
                        already_placed,
                        team_valid_place,
                        near_enemy,
                    )
                };

                if let Some(plan) = claim.clone().into_claimed_plan() {
                    let build_plan = Self::build_plan_from_team_plan(&plan);
                    if self.add_build(build_plan.clone()) {
                        state.last_plan = Some(plan);
                        added_build_plan = Some(build_plan);
                    }
                }

                claimed_team_plan = Some(claim);
                branch = BuilderAiRuntimeBranch::FindNewPlan;
            }
        }

        let boosting = (!input.unit_flying).then_some(prebuild_ai_should_boost(
            input.boost_when_building,
            moving,
            input.floor_is_duct,
            input.floor_damage_taken,
            input.floor_is_deep,
        ));

        BuilderAiRuntimeStep {
            look_action,
            update_building: self.update_building,
            branch,
            follow_sync,
            retreat,
            current_plan_validation,
            move_to_plan,
            move_to_assist,
            claimed_team_plan,
            added_build_plan,
            boosting,
            moving,
            following: state.following,
            assist_following: state.assist_following,
            enemy: state.enemy,
            retreat_timer: state.retreat_timer,
            last_plan: state.last_plan.clone(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn apply_prebuild_ai_tick<FValidBreak, FValidPlace, FPlanValid, FPlanCanBuild>(
        &mut self,
        state: &mut PrebuildAiRuntimeState,
        input: PrebuildAiRuntimeInput,
        valid_break: FValidBreak,
        valid_place: FValidPlace,
        next_plan: Option<PrebuildAiPlanSnapshot>,
        plan_valid_place: FPlanValid,
        plan_can_build: FPlanCanBuild,
    ) -> PrebuildAiRuntimeStep
    where
        FValidBreak: FnMut(&BuildPlan) -> bool,
        FValidPlace: FnMut(&BuildPlan) -> bool,
        FPlanValid: FnMut(&PrebuildAiPlanSnapshot) -> bool,
        FPlanCanBuild: FnMut(&PrebuildAiPlanSnapshot) -> bool,
    {
        let unit_has_current_plan = self.build_plan().is_some();
        let decision = prebuild_ai_update_movement_tick(
            state.collecting_items,
            unit_has_current_plan,
            input.unit_flying,
            input.has_target,
            input.should_shoot,
            input.boost_when_building,
            input.floor_is_duct,
            input.floor_damage_taken,
            input.floor_is_deep,
            self.type_info.build_range,
            input.within_current_plan_range,
            &mut self.plans,
            &mut state.last_plan,
            input.construct_current_matches,
            valid_break,
            valid_place,
            input.timer_find_ready,
            next_plan,
            plan_valid_place,
            plan_can_build,
        );

        self.update_building = decision.update_building;

        let added_build_plan = decision.accept_plan.as_ref().and_then(|accepted| {
            if !accepted.accepted {
                return None;
            }

            state.collecting_items = accepted.collecting_items;
            state.collect_block = accepted.collect_block.clone();
            state.last_target_item = accepted.last_target_item;
            state.ore = accepted.ore;
            state.last_plan = accepted.last_plan.clone();

            let build_plan = accepted.build_plan.clone()?;
            self.add_build(build_plan.clone()).then_some(build_plan)
        });

        PrebuildAiRuntimeStep {
            decision,
            added_build_plan,
            update_building: self.update_building,
            collecting_items: state.collecting_items,
            last_plan: state.last_plan.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builder_unit() -> UnitType {
        let mut unit = UnitType::new(1, "alpha");
        unit.build_speed = 1.0;
        unit.build_range = 60.0;
        unit
    }

    fn runtime_plan(
        x: i32,
        y: i32,
        name: &str,
        category: crate::mindustry::r#type::Category,
    ) -> PrebuildAiPlanSnapshot {
        let block = PrebuildAiBlockInfo::new(name, category);
        PrebuildAiPlanSnapshot::new(TeamBlockPlan::new(x, y, 0, name, None), block)
    }

    #[test]
    fn builder_component_queue_operations_replace_and_position_plans() {
        let mut builder = BuilderComp::new(builder_unit(), TeamId(1));
        assert!(builder.can_build());

        assert!(builder.add_build(BuildPlan::new_place(1, 2, 0, "duo")));
        assert!(builder.add_build(BuildPlan::new_place(3, 4, 0, "router")));
        assert_eq!(builder.plans.len(), 2);

        assert!(builder.add_build_front(BuildPlan::new_place(1, 2, 1, "scatter")));
        assert_eq!(builder.plans.len(), 2);
        assert_eq!(
            builder.build_plan().unwrap().block.as_deref(),
            Some("scatter")
        );
        assert_eq!(builder.build_plan().unwrap().rotation, 1);

        assert!(builder.add_build_with_progress(BuildPlan::new_break(5, 6), true, Some(0.45)));
        assert_eq!(builder.plans.back().unwrap().progress, 0.45);
        assert!(builder.remove_build(5, 6, true).is_some());
        assert!(builder.remove_build(5, 6, true).is_none());

        builder.clear_building();
        assert!(!builder.is_building());
        assert_eq!(builder.build_plan(), None);
    }

    #[test]
    fn builder_component_can_convert_and_queue_team_plans() {
        let team_plan = TeamBlockPlan::new(8, 9, 2, "router", Some("cfg".into()));
        let build_plan = BuilderComp::build_plan_from_team_plan(&team_plan);

        assert_eq!(build_plan.x, 8);
        assert_eq!(build_plan.y, 9);
        assert_eq!(build_plan.rotation, 2);
        assert_eq!(build_plan.block.as_deref(), Some("router"));
        assert_eq!(build_plan.config, TypeValue::String("cfg".into()));

        let mut builder = BuilderComp::new(builder_unit(), TeamId(1));
        assert!(builder.add_team_plan(&team_plan));
        assert_eq!(builder.build_plan(), Some(&build_plan));
    }

    #[test]
    fn builder_component_claims_team_queue_into_unit_plan_like_builder_ai() {
        let mut builder = BuilderComp::new(builder_unit(), TeamId(1));
        let mut teams = crate::mindustry::game::Teams::default();
        teams.replace_plans([(
            1,
            vec![
                TeamBlockPlan::new(1, 1, 0, "duo", None),
                TeamBlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
            ],
        )]);

        let rotated = teams.claim_front_plan(1, |_| false, |plan| plan.block == "router");
        assert_eq!(builder.add_claimed_team_plan(rotated), None);
        assert!(builder.plans.is_empty());
        assert_eq!(
            teams.get_or_null(1).unwrap().plans,
            vec![
                TeamBlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
                TeamBlockPlan::new(1, 1, 0, "duo", None),
            ]
        );

        let claimed = teams.claim_front_plan(1, |_| false, |plan| plan.block == "router");
        let build_plan = builder
            .add_claimed_team_plan(claimed)
            .expect("usable team plan should become a unit build plan");

        assert_eq!(build_plan.x, 2);
        assert_eq!(build_plan.y, 2);
        assert_eq!(build_plan.rotation, 1);
        assert_eq!(build_plan.block.as_deref(), Some("router"));
        assert_eq!(build_plan.config, TypeValue::String("cfg".into()));
        assert_eq!(builder.build_plan(), Some(&build_plan));
        assert_eq!(
            teams.get_or_null(1).unwrap().plans,
            vec![
                TeamBlockPlan::new(1, 1, 0, "duo", None),
                TeamBlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
            ]
        );
    }

    #[test]
    fn builder_component_can_build_and_active_state_follow_java_guards() {
        let mut unit = builder_unit();
        unit.build_speed = -1.0;
        let mut builder = BuilderComp::new(unit, TeamId(1));
        assert!(!builder.can_build());
        assert!(!builder.add_build(BuildPlan::new_place(0, 0, 0, "duo")));

        builder.type_info.build_speed = 1.0;
        builder.build_speed_multiplier = 0.0;
        assert!(!builder.can_build());

        builder.build_speed_multiplier = 1.0;
        assert!(builder.add_build(BuildPlan::new_place(0, 0, 0, "duo")));
        assert!(!builder.actively_building(false, false));
        assert!(builder.actively_building(true, false));

        builder.update_building = false;
        assert!(!builder.actively_building(true, true));
    }

    #[test]
    fn builder_component_validate_plans_filters_like_upstream_rules() {
        let mut builder = BuilderComp::new(builder_unit(), TeamId(1));
        builder.add_build(BuildPlan::new_break(0, 0));
        builder.add_build(BuildPlan::new_place(1, 1, 0, "duo"));
        builder.add_build(BuildPlan::new_place(2, 2, 0, "duo"));
        builder.add_build(BuildPlan::new_place(3, 3, 0, "ore-copper"));
        builder.add_build(BuildPlan::new_place(4, 4, 0, "router"));

        let removed = builder.validate_plans_with(
            |x, y| match (x, y) {
                (0, 0) => Some(BuilderTileSnapshot::air()),
                (1, 1) => {
                    Some(BuilderTileSnapshot::block("duo").with_build("duo", TeamId(1), 1, 1, 0))
                }
                (2, 2) => Some(BuilderTileSnapshot::block("duo").with_build(
                    "duo",
                    TeamId(TEAM_DERELICT),
                    2,
                    2,
                    0,
                )),
                (3, 3) => Some(BuilderTileSnapshot::air().with_overlay("ore-copper")),
                (4, 4) => Some(BuilderTileSnapshot::block("air")),
                _ => None,
            },
            |name| match name {
                "ore-copper" => Some(BuilderBlockInfo::overlay(name)),
                other => Some(BuilderBlockInfo::new(other)),
            },
        );

        assert_eq!(removed, 3);
        let left: Vec<_> = builder
            .plans
            .iter()
            .map(|plan| (plan.x, plan.y, plan.block.as_deref()))
            .collect();
        assert_eq!(left, vec![(2, 2, Some("duo")), (4, 4, Some("router"))]);
    }

    #[test]
    fn builder_component_should_skip_and_counter_match_java_branches() {
        let mut plan = BuildPlan::new_place(2, 3, 0, "duo");
        let mut context = BuilderSkipContext::finite_with_core();

        assert!(!BuilderComp::should_skip_plan(&plan, context));

        plan.stuck = true;
        context.has_all_requirements = false;
        assert!(BuilderComp::should_skip_plan(&plan, context));

        context.rules_infinite_resources = true;
        assert!(!BuilderComp::should_skip_plan(&plan, context));

        context = BuilderSkipContext::finite_with_core();
        context.has_missing_limited_item = true;
        assert!(BuilderComp::should_skip_plan(&plan, context));

        let break_plan = BuildPlan::new_break(2, 3);
        assert!(!BuilderComp::should_skip_plan(&break_plan, context));

        let mut builder = BuilderComp::new(builder_unit(), TeamId(1));
        assert_eq!(builder.advance_build_counter(4.0), 4.0);
        assert_eq!(builder.advance_build_counter(20.0), 10.0);
        builder.build_counter = f32::NAN;
        assert_eq!(builder.advance_build_counter(1.0), 0.0);
    }

    #[test]
    fn builder_component_applies_prebuild_ai_tick_to_queue_and_runtime_state() {
        let mut builder = BuilderComp::new(builder_unit(), TeamId(1));
        let mut state = PrebuildAiRuntimeState::default();
        let next = runtime_plan(
            6,
            7,
            "router",
            crate::mindustry::r#type::Category::Distribution,
        );

        let step = builder.apply_prebuild_ai_tick(
            &mut state,
            PrebuildAiRuntimeInput {
                timer_find_ready: true,
                ..PrebuildAiRuntimeInput::default()
            },
            |_| false,
            |_| false,
            Some(next.clone()),
            |_| true,
            |_| false,
        );

        assert_eq!(
            step.decision.branch,
            crate::mindustry::ai::PrebuildAiTickBranch::FindNewPlan
        );
        assert!(step.collecting_items);
        assert!(state.collecting_items);
        assert_eq!(state.collect_block, Some(next.block));
        assert_eq!(state.last_plan, Some(next.plan));
        assert_eq!(
            step.added_build_plan,
            Some(BuildPlan::new_place(6, 7, 0, "router"))
        );
        assert_eq!(
            builder.build_plan(),
            Some(&BuildPlan::new_place(6, 7, 0, "router"))
        );
        assert!(builder.update_building);
    }

    #[test]
    fn builder_component_applies_prebuild_ai_current_and_collecting_branches() {
        let mut builder = BuilderComp::new(builder_unit(), TeamId(1));
        builder.add_build(BuildPlan::new_place(2, 2, 0, "duo"));
        let mut state = PrebuildAiRuntimeState {
            last_plan: Some(TeamBlockPlan::new(2, 2, 0, "duo", None)),
            ..PrebuildAiRuntimeState::default()
        };

        let current = builder.apply_prebuild_ai_tick(
            &mut state,
            PrebuildAiRuntimeInput {
                within_current_plan_range: false,
                ..PrebuildAiRuntimeInput::default()
            },
            |_| false,
            |plan| plan.block.as_deref() == Some("duo"),
            None,
            |_| false,
            |_| false,
        );

        assert_eq!(
            current.decision.branch,
            crate::mindustry::ai::PrebuildAiTickBranch::CurrentPlan
        );
        assert_eq!(current.added_build_plan, None);
        assert!(builder.update_building);
        assert_eq!(builder.plans.len(), 1);
        assert_eq!(current.decision.boosting, Some(true));

        state.collecting_items = true;
        let collecting = builder.apply_prebuild_ai_tick(
            &mut state,
            PrebuildAiRuntimeInput {
                boost_when_building: true,
                ..PrebuildAiRuntimeInput::default()
            },
            |_| false,
            |_| false,
            None,
            |_| false,
            |_| false,
        );
        assert_eq!(
            collecting.decision.branch,
            crate::mindustry::ai::PrebuildAiTickBranch::Collecting
        );
        assert!(!builder.update_building);
        assert_eq!(collecting.added_build_plan, None);
    }

    #[test]
    fn builder_component_builder_ai_tick_copies_follower_plan_and_clears_invalid_follower() {
        let mut builder = BuilderComp::new(builder_unit(), TeamId(1));
        let mut state = BuilderAiRuntimeState {
            following: Some(42),
            last_plan: Some(TeamBlockPlan::new(1, 1, 0, "duo", None)),
            retreat_timer: 8.0,
            ..BuilderAiRuntimeState::default()
        };
        let mut team_data = crate::mindustry::game::TeamData::new(1);

        let copied = builder.apply_builder_ai_tick(
            &mut state,
            &mut team_data,
            BuilderAiRuntimeInput {
                following_valid: true,
                following_actively_building: true,
                following_plan: Some(BuildPlan::new_place(3, 4, 1, "router")),
                ..BuilderAiRuntimeInput::default()
            },
            |_| false,
            |_| true,
            |_| false,
            |_| false,
            |_| false,
            |_| false,
        );

        assert_eq!(copied.branch, BuilderAiRuntimeBranch::FollowingCurrentPlan);
        assert_eq!(
            copied.follow_sync.as_ref().map(|sync| sync.action),
            Some(BuilderAiFollowAction::CopyFollowerPlan)
        );
        assert_eq!(
            builder.build_plan(),
            Some(&BuildPlan::new_place(3, 4, 1, "router"))
        );
        assert_eq!(state.last_plan, None);
        assert_eq!(state.retreat_timer, 0.0);

        let cleared = builder.apply_builder_ai_tick(
            &mut state,
            &mut team_data,
            BuilderAiRuntimeInput {
                following_valid: true,
                following_actively_building: false,
                ..BuilderAiRuntimeInput::default()
            },
            |_| false,
            |_| true,
            |_| false,
            |_| false,
            |_| false,
            |_| false,
        );

        assert_eq!(cleared.branch, BuilderAiRuntimeBranch::FollowingInvalid);
        assert_eq!(state.following, None);
        assert!(builder.plans.is_empty());
    }

    #[test]
    fn builder_component_builder_ai_tick_retreats_when_idle_and_enemy_seen() {
        let mut builder = BuilderComp::new(builder_unit(), TeamId(1));
        builder.add_build(BuildPlan::new_place(1, 1, 0, "duo"));
        let mut state = BuilderAiRuntimeState {
            always_flee: true,
            ..BuilderAiRuntimeState::default()
        };
        let mut team_data = crate::mindustry::game::TeamData::new(1);

        let step = builder.apply_builder_ai_tick(
            &mut state,
            &mut team_data,
            BuilderAiRuntimeInput {
                delta: 1.0,
                has_core: true,
                timer_enemy_ready: true,
                sensed_enemy: Some(99),
                within_retreat_distance: false,
                floor_is_deep: true,
                ..BuilderAiRuntimeInput::default()
            },
            |_| false,
            |_| true,
            |_| false,
            |_| false,
            |_| false,
            |_| false,
        );

        assert_eq!(step.branch, BuilderAiRuntimeBranch::Retreat);
        assert_eq!(state.enemy, Some(99));
        assert!(builder.plans.is_empty());
        assert!(step.retreat.unwrap().move_to_core);
        assert_eq!(step.boosting, Some(true));
        assert!(step.moving);
    }

    #[test]
    fn builder_component_builder_ai_tick_validates_current_plan_and_removes_team_conflict() {
        let mut builder = BuilderComp::new(builder_unit(), TeamId(1));
        builder.add_build(BuildPlan::new_place(4, 5, 0, "duo"));
        let mut state = BuilderAiRuntimeState {
            last_plan: Some(TeamBlockPlan::new(4, 5, 0, "duo", None)),
            ..BuilderAiRuntimeState::default()
        };
        let mut team_data = crate::mindustry::game::TeamData::new(1);
        team_data.plans = vec![TeamBlockPlan::new(4, 5, 0, "duo", None)];

        let conflict = builder.apply_builder_ai_tick(
            &mut state,
            &mut team_data,
            BuilderAiRuntimeInput {
                conflicting_breaker: true,
                within_current_plan_range: false,
                ..BuilderAiRuntimeInput::default()
            },
            |_| false,
            |_| true,
            |_| false,
            |_| false,
            |_| false,
            |_| false,
        );

        assert_eq!(conflict.branch, BuilderAiRuntimeBranch::CurrentPlan);
        assert_eq!(
            conflict
                .current_plan_validation
                .as_ref()
                .map(|validation| validation.action),
            Some(BuilderAiPlanAction::DropConflictingBreak)
        );
        assert!(builder.plans.is_empty());
        assert!(team_data.plans.is_empty());
    }

    #[test]
    fn builder_component_builder_ai_tick_claims_hold_and_rebuild_team_plans() {
        let mut builder = BuilderComp::new(builder_unit(), TeamId(1));
        let mut state = BuilderAiRuntimeState::default();
        let mut team_data = crate::mindustry::game::TeamData::new(1);
        team_data.plans = vec![
            TeamBlockPlan::new(1, 1, 0, "duo", None),
            TeamBlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
        ];

        let hold = builder.apply_builder_ai_tick(
            &mut state,
            &mut team_data,
            BuilderAiRuntimeInput {
                hold_position: true,
                timer_find_ready: true,
                ..BuilderAiRuntimeInput::default()
            },
            |_| false,
            |_| true,
            |plan| plan.block == "router",
            |_| false,
            |plan| plan.block == "router",
            |_| false,
        );

        assert_eq!(hold.branch, BuilderAiRuntimeBranch::FindNewPlan);
        assert_eq!(
            hold.claimed_team_plan,
            Some(TeamPlanClaim::Claimed(TeamBlockPlan::new(
                2,
                2,
                1,
                "router",
                Some("cfg".into())
            )))
        );
        assert_eq!(
            hold.added_build_plan,
            Some(BuildPlan::new_string_config(2, 2, 1, "router", "cfg"))
        );
        assert_eq!(
            state.last_plan,
            Some(TeamBlockPlan::new(2, 2, 1, "router", Some("cfg".into())))
        );

        builder.clear_building();
        state.last_plan = None;
        let rebuild = builder.apply_builder_ai_tick(
            &mut state,
            &mut team_data,
            BuilderAiRuntimeInput {
                timer_find_ready: true,
                ..BuilderAiRuntimeInput::default()
            },
            |_| false,
            |_| true,
            |_| false,
            |_| false,
            |plan| plan.block == "duo",
            |_| false,
        );

        assert_eq!(rebuild.branch, BuilderAiRuntimeBranch::FindNewPlan);
        assert_eq!(
            rebuild.claimed_team_plan,
            Some(TeamPlanClaim::Claimed(TeamBlockPlan::new(
                1, 1, 0, "duo", None
            )))
        );
        assert_eq!(
            rebuild.added_build_plan,
            Some(BuildPlan::new_place(1, 1, 0, "duo"))
        );
    }

    #[test]
    fn builder_ai_follow_search_selects_first_reachable_active_builder() {
        let mut unit = builder_unit();
        unit.speed = 2.0;
        unit.build_range = 40.0;
        let builder = BuilderComp::new(unit, TeamId(1));

        let self_candidate = BuilderAiFollowCandidate {
            unit_id: 1,
            is_self: true,
            ..BuilderAiFollowCandidate::new(1, BuildPlan::new_place(1, 1, 0, "duo"), 80.0, 10.0)
        };
        let inactive = BuilderAiFollowCandidate {
            unit_id: 2,
            actively_building: false,
            ..BuilderAiFollowCandidate::new(2, BuildPlan::new_place(2, 2, 0, "router"), 80.0, 10.0)
        };
        let no_construct = BuilderAiFollowCandidate {
            unit_id: 3,
            construct_dst: None,
            construct_build_cost: None,
            ..BuilderAiFollowCandidate::new(3, BuildPlan::new_place(3, 3, 0, "wall"), 80.0, 10.0)
        };
        let first_valid =
            BuilderAiFollowCandidate::new(4, BuildPlan::new_place(4, 4, 0, "scatter"), 80.0, 10.0);
        let later_valid =
            BuilderAiFollowCandidate::new(5, BuildPlan::new_place(5, 5, 0, "duo"), 80.0, 10.0);

        let selected = builder.choose_builder_ai_following(
            [
                self_candidate,
                inactive,
                no_construct,
                first_valid,
                later_valid,
            ]
            .iter(),
        );

        assert_eq!(selected, Some(4));

        let too_late =
            BuilderAiFollowCandidate::new(6, BuildPlan::new_place(6, 6, 0, "router"), 100.0, -1.0);
        assert_eq!(builder.choose_builder_ai_following([too_late].iter()), None);
    }

    #[test]
    fn builder_ai_assist_search_selects_nearest_valid_builder_player() {
        let builder = BuilderComp::new(builder_unit(), TeamId(1));
        let candidates = [
            BuilderAiAssistCandidate::new(10, 100.0),
            BuilderAiAssistCandidate::new(11, 50.0),
            BuilderAiAssistCandidate {
                unit_id: 12,
                dead: true,
                dst2: 1.0,
                ..BuilderAiAssistCandidate::new(12, 1.0)
            },
            BuilderAiAssistCandidate {
                unit_id: 13,
                same_team: false,
                dst2: 0.5,
                ..BuilderAiAssistCandidate::new(13, 0.5)
            },
        ];

        assert_eq!(
            BuilderComp::choose_builder_ai_assist_following(candidates.iter()),
            Some(11)
        );

        let follow =
            BuilderAiFollowCandidate::new(21, BuildPlan::new_place(1, 1, 0, "router"), 32.0, 3.0);
        let search = builder.search_builder_ai_follow_targets(
            [follow.clone()].iter(),
            candidates.iter(),
            true,
        );
        assert_eq!(
            search,
            BuilderAiFollowSearch {
                following: Some(21),
                assist_following: Some(11),
                found: true,
            }
        );

        let no_assist =
            builder.search_builder_ai_follow_targets([follow].iter(), candidates.iter(), false);
        assert_eq!(no_assist.assist_following, None);
    }

    #[test]
    fn builder_ai_runtime_consumes_follow_search_results_and_only_assist_blocks_rebuild() {
        let builder = BuilderComp::new(builder_unit(), TeamId(1));
        let search = builder.search_builder_ai_follow_targets(
            [BuilderAiFollowCandidate::new(
                31,
                BuildPlan::new_place(1, 1, 0, "router"),
                32.0,
                3.0,
            )]
            .iter(),
            [BuilderAiAssistCandidate::new(41, 16.0)].iter(),
            true,
        );

        let mut builder = BuilderComp::new(builder_unit(), TeamId(1));
        let mut state = BuilderAiRuntimeState {
            only_assist: true,
            ..BuilderAiRuntimeState::default()
        };
        let mut team_data = crate::mindustry::game::TeamData::new(1);
        team_data.plans = vec![TeamBlockPlan::new(9, 9, 0, "duo", None)];

        let step = builder.apply_builder_ai_tick(
            &mut state,
            &mut team_data,
            BuilderAiRuntimeInput {
                timer_follow_ready: true,
                timer_find_ready: true,
                ..BuilderAiRuntimeInput::default().with_follow_search(search)
            },
            |_| false,
            |_| true,
            |_| true,
            |_| false,
            |_| true,
            |_| false,
        );

        assert_eq!(step.branch, BuilderAiRuntimeBranch::Idle);
        assert_eq!(state.following, Some(31));
        assert_eq!(state.assist_following, Some(41));
        assert_eq!(step.claimed_team_plan, None);
        assert!(builder.plans.is_empty());
        assert_eq!(
            team_data.plans,
            vec![TeamBlockPlan::new(9, 9, 0, "duo", None)]
        );
    }
}
