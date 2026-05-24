//! Builder component shell mirroring upstream `mindustry.entities.comp.BuilderComp`.
//!
//! The Java implementation mixes queue management, world validation, resource
//! checks, build effects and rendering. This Rust port starts with the pure
//! queue/state portion and keeps world-dependent decisions behind lightweight
//! snapshots so later runtime ports can plug in real `World`/`Building` state.

use std::collections::VecDeque;

use crate::mindustry::entities::units::BuildPlan;
use crate::mindustry::game::TEAM_DERELICT;
use crate::mindustry::game::{BlockPlan as TeamBlockPlan, TeamPlanClaim};
use crate::mindustry::io::{TeamId, TypeValue};
use crate::mindustry::r#type::UnitType;

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
}
