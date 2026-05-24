use std::collections::{HashSet, VecDeque};

use crate::mindustry::{
    ai::base_registry::{BasePart, BasePartTile, BasePartTileKind},
    ctype::ContentId,
    entities::units::BuildPlan,
    game::{BlockPlan as TeamBlockPlan, TeamData, TeamPlanClaim},
    r#type::Category,
    vars::TILE_SIZE,
    world::{footprint_tiles, get_edges, point2_pack},
};

pub const ATTEMPTS: usize = 6;
pub const CORE_UNIT_MULTIPLIER: usize = 2;
pub const EMPTY_CHANCE: f32 = 0.01;
pub const TIMER_STEP: usize = 0;
pub const TIMER_SPAWN: usize = 1;
pub const TIMER_REFRESH_PATH: usize = 2;
pub const PLACE_INTERVAL_MIN: f32 = 12.0;
pub const PLACE_INTERVAL_MAX: f32 = 2.0;
pub const PATH_STEP: usize = 50;
pub const BUILDER_AI_DEFAULT_REBUILD_PERIOD: f32 = 60.0 * 2.0;
pub const BUILDER_AI_BUILD_AI_REBUILD_PERIOD: f32 = 10.0;
pub const PREBUILD_AI_PRODUCTION_PRIORITY: f32 = 11.0;
pub const PREBUILD_AI_DISTRIBUTION_PRIORITY: f32 = 10.0;
pub const PREBUILD_AI_LIQUID_PRIORITY: f32 = 9.0;
pub const PREBUILD_AI_CRAFTING_PRIORITY: f32 = 8.0;
pub const PREBUILD_AI_PRIORITY_DST_SCALE: f32 = 200.0;

const D4: [TilePoint; 4] = [
    TilePoint { x: 1, y: 0 },
    TilePoint { x: 0, y: 1 },
    TilePoint { x: -1, y: 0 },
    TilePoint { x: 0, y: -1 },
];

const D8: [TilePoint; 8] = [
    TilePoint { x: -1, y: -1 },
    TilePoint { x: 0, y: -1 },
    TilePoint { x: 1, y: -1 },
    TilePoint { x: -1, y: 0 },
    TilePoint { x: 1, y: 0 },
    TilePoint { x: -1, y: 1 },
    TilePoint { x: 0, y: 1 },
    TilePoint { x: 1, y: 1 },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TilePoint {
    pub x: i32,
    pub y: i32,
}

impl TilePoint {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn packed(self) -> i32 {
        point2_pack(self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedPositionSource {
    Core,
    Spawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartPoolChoice {
    Resource,
    Generic,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathCalculationOutcome {
    Idle,
    Seeded,
    WaitingWeights,
    Advanced,
    Found,
    NoStart,
    Exhausted,
    Stuck,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BaseBuilderPathState {
    pub path: HashSet<i32>,
    pub calc_path: HashSet<i32>,
    pub calc_tile: Option<TilePoint>,
    pub calculating: bool,
    pub started_calculating: bool,
    pub calc_count: i32,
    pub total_calcs: i32,
    pub found_path: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseBuildTile {
    pub x: i32,
    pub y: i32,
    pub rotation: i32,
    pub block_name: String,
    pub config: Option<String>,
    pub kind: BasePartTileKind,
    pub block_size: i32,
    pub block_offset: f32,
    pub solid: bool,
    pub requires_payload_clearance: bool,
    pub taken_offsets: Vec<TilePoint>,
}

impl BaseBuildTile {
    pub fn new(x: i32, y: i32, block_name: impl Into<String>, kind: BasePartTileKind) -> Self {
        Self {
            x,
            y,
            rotation: 0,
            block_name: block_name.into(),
            config: None,
            kind,
            block_size: 1,
            block_offset: 0.0,
            solid: false,
            requires_payload_clearance: false,
            taken_offsets: vec![TilePoint::new(0, 0)],
        }
    }

    pub fn from_base_part_tile(tile: &BasePartTile) -> Self {
        let mut out = Self::new(tile.x, tile.y, tile.block_name.clone(), tile.kind);
        out.config = tile.config.clone();
        out.block_offset = tile.offset;
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseBuildPart {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub center_x: i32,
    pub center_y: i32,
    pub required: Option<String>,
    pub tiles: Vec<BaseBuildTile>,
}

impl BaseBuildPart {
    pub fn from_base_part(part: &BasePart) -> Self {
        Self {
            name: part.name.clone(),
            width: part.width,
            height: part.height,
            center_x: part.center_x,
            center_y: part.center_y,
            required: part.required.clone(),
            tiles: part
                .tiles
                .iter()
                .map(BaseBuildTile::from_base_part_tile)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockPlan {
    pub x: i32,
    pub y: i32,
    pub rotation: i32,
    pub block_name: String,
    pub config: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrebuildAiRequirement {
    pub item: ContentId,
    pub amount: i32,
}

impl PrebuildAiRequirement {
    pub const fn new(item: ContentId, amount: i32) -> Self {
        Self { item, amount }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrebuildAiBlockInfo {
    pub name: String,
    pub category: Category,
    pub build_time: f32,
    pub offset: f32,
    pub size: i32,
    pub requirements: Vec<PrebuildAiRequirement>,
}

impl PrebuildAiBlockInfo {
    pub fn new(name: impl Into<String>, category: Category) -> Self {
        Self {
            name: name.into(),
            category,
            build_time: 0.0,
            offset: 0.0,
            size: 1,
            requirements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrebuildAiPlanSnapshot {
    pub plan: TeamBlockPlan,
    pub block: PrebuildAiBlockInfo,
}

impl PrebuildAiPlanSnapshot {
    pub fn new(plan: TeamBlockPlan, block: PrebuildAiBlockInfo) -> Self {
        Self { plan, block }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrebuildAiTreeQuery {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrebuildAiMiningTarget {
    pub target_item: Option<ContentId>,
    pub last_target_item: Option<ContentId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrebuildAiFullCoreAction {
    pub handled: bool,
    pub clear_item: bool,
    pub clear_mine_tile: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrebuildAiOreTarget {
    Existing(Option<i32>),
    Refreshed(Option<i32>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrebuildAiReturnToCoreAction {
    pub mining: bool,
    pub collecting_items: bool,
    pub clear_mine_tile: bool,
    pub clear_item: bool,
    pub transfer_to_core: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrebuildAiBuildMove {
    pub range: f32,
    pub move_range: f32,
    pub moving: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrebuildAiAcceptPlanAction {
    pub accepted: bool,
    pub collecting_items: bool,
    pub collect_block: Option<PrebuildAiBlockInfo>,
    pub last_target_item: Option<ContentId>,
    pub ore: Option<i32>,
    pub reset_target_timer: bool,
    pub last_plan: Option<TeamBlockPlan>,
    pub build_plan: Option<BuildPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderAiPlanAction {
    NoPlan,
    Keep,
    DropConflictingBreak,
    DropHoldOutOfRange,
    DropInvalid,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuilderAiPlanValidation {
    pub action: BuilderAiPlanAction,
    pub removed_plan: Option<BuildPlan>,
    pub remove_team_plan_at: Option<(i32, i32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderAiFollowAction {
    NoFollower,
    ClearInvalidFollower,
    CopyFollowerPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuilderAiFollowSync {
    pub action: BuilderAiFollowAction,
    pub clear_following: bool,
    pub reset_retreat_timer: bool,
    pub copied_plan: Option<BuildPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuilderAiRetreatDecision {
    pub retreat_timer: f32,
    pub clear_building: bool,
    pub move_to_core: bool,
    pub moving: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuilderAiEnemySearchRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderAiFallbackController {
    Prebuild,
    Flying,
    Ground,
}

pub fn should_spawn_core_unit(
    ai_core_spawn: bool,
    timer_ready: bool,
    has_core: bool,
    editor: bool,
    current_core_units: usize,
    core_count: usize,
) -> bool {
    ai_core_spawn
        && timer_ready
        && has_core
        && !editor
        && current_core_units < core_count * CORE_UNIT_MULTIPLIER
}

pub fn place_interval(build_ai_tier: f32) -> f32 {
    PLACE_INTERVAL_MIN + (PLACE_INTERVAL_MAX - PLACE_INTERVAL_MIN) * build_ai_tier
}

pub fn random_position_source(
    has_core: bool,
    is_wave_team: bool,
    spawn_count: usize,
) -> Option<SeedPositionSource> {
    if has_core {
        Some(SeedPositionSource::Core)
    } else if is_wave_team && spawn_count > 0 {
        Some(SeedPositionSource::Spawn)
    } else {
        None
    }
}

pub fn choose_part_pool(
    tile_drop: Option<&str>,
    resource_parts_available: bool,
    empty_roll: f32,
) -> PartPoolChoice {
    if tile_drop.is_some() && resource_parts_available {
        PartPoolChoice::Resource
    } else if empty_roll < EMPTY_CHANCE {
        PartPoolChoice::Generic
    } else {
        PartPoolChoice::None
    }
}

pub fn claim_builder_ai_rebuild_plan<FPlaced, FValid, FNearEnemy>(
    team_data: &mut TeamData,
    always_flee: bool,
    already_placed: FPlaced,
    mut valid_place: FValid,
    mut near_enemy: FNearEnemy,
) -> TeamPlanClaim
where
    FPlaced: FnMut(&TeamBlockPlan) -> bool,
    FValid: FnMut(&TeamBlockPlan) -> bool,
    FNearEnemy: FnMut(&TeamBlockPlan) -> bool,
{
    team_data.claim_front_plan(already_placed, |plan| {
        valid_place(plan) && (!always_flee || !near_enemy(plan))
    })
}

pub fn claim_builder_ai_hold_plan<FWithin, FValid>(
    team_data: &mut TeamData,
    infinite_resources: bool,
    mut within_build_range: FWithin,
    mut valid_place: FValid,
) -> TeamPlanClaim
where
    FWithin: FnMut(&TeamBlockPlan) -> bool,
    FValid: FnMut(&TeamBlockPlan) -> bool,
{
    team_data.claim_first_usable_plan(|plan| {
        (infinite_resources || within_build_range(plan)) && valid_place(plan)
    })
}

pub const fn prebuild_ai_category_priority(category: Category) -> f32 {
    match category {
        Category::Production => PREBUILD_AI_PRODUCTION_PRIORITY,
        Category::Distribution => PREBUILD_AI_DISTRIBUTION_PRIORITY,
        Category::Liquid => PREBUILD_AI_LIQUID_PRIORITY,
        Category::Crafting => PREBUILD_AI_CRAFTING_PRIORITY,
        _ => 0.0,
    }
}

pub fn prebuild_ai_sort_plans(plans: &mut Vec<PrebuildAiPlanSnapshot>) {
    plans.sort_by(|left, right| {
        prebuild_ai_category_priority(left.block.category)
            .total_cmp(&prebuild_ai_category_priority(right.block.category))
            .then_with(|| left.block.build_time.total_cmp(&right.block.build_time))
    });
    plans.reverse();
}

pub fn prebuild_ai_can_build<FCoreItems>(
    rules_infinite_resources: bool,
    team_infinite_resources: bool,
    requirements: &[PrebuildAiRequirement],
    build_cost_multiplier: f32,
    mut core_has_item: FCoreItems,
) -> bool
where
    FCoreItems: FnMut(ContentId, i32) -> bool,
{
    rules_infinite_resources
        || team_infinite_resources
        || requirements.iter().all(|requirement| {
            core_has_item(
                requirement.item,
                scaled_requirement_amount(requirement.amount, build_cost_multiplier),
            )
        })
}

pub fn prebuild_ai_all_requirements_have_ore<FHasOre>(
    requirements: &[PrebuildAiRequirement],
    mut has_ore: FHasOre,
) -> bool
where
    FHasOre: FnMut(ContentId) -> bool,
{
    requirements
        .iter()
        .all(|requirement| has_ore(requirement.item))
}

pub fn prebuild_ai_tree_query(
    plan: &TeamBlockPlan,
    block: &PrebuildAiBlockInfo,
) -> PrebuildAiTreeQuery {
    let size = block.size as f32 * TILE_SIZE as f32 + 1.0;
    let half = size / 2.0;
    PrebuildAiTreeQuery {
        x: plan.x as f32 * TILE_SIZE as f32 + block.offset - half,
        y: plan.y as f32 * TILE_SIZE as f32 + block.offset - half,
        width: size,
        height: size,
    }
}

pub fn prebuild_ai_plan_reachable_from_tree<FBuildingTree>(
    plan: &TeamBlockPlan,
    block: &PrebuildAiBlockInfo,
    mut building_tree_any: FBuildingTree,
) -> bool
where
    FBuildingTree: FnMut(PrebuildAiTreeQuery) -> bool,
{
    block.category == Category::Production || building_tree_any(prebuild_ai_tree_query(plan, block))
}

pub fn prebuild_ai_find_next_plan<FCanBuild, FHasOre, FBuildingTree>(
    plans: &[PrebuildAiPlanSnapshot],
    core_available: bool,
    building_tree_available: bool,
    unit_x: f32,
    unit_y: f32,
    mut can_build: FCanBuild,
    mut has_ore: FHasOre,
    mut building_tree_any: FBuildingTree,
) -> Option<PrebuildAiPlanSnapshot>
where
    FCanBuild: FnMut(&PrebuildAiPlanSnapshot) -> bool,
    FHasOre: FnMut(ContentId) -> bool,
    FBuildingTree: FnMut(PrebuildAiTreeQuery) -> bool,
{
    if !core_available || !building_tree_available || plans.is_empty() {
        return None;
    }

    let mut best_index = None;
    let mut best_score = 0.0;

    for (index, plan) in plans.iter().enumerate() {
        let can_supply = can_build(plan)
            || prebuild_ai_all_requirements_have_ore(&plan.block.requirements, &mut has_ore);
        if !can_supply
            || !prebuild_ai_plan_reachable_from_tree(
                &plan.plan,
                &plan.block,
                &mut building_tree_any,
            )
        {
            continue;
        }

        let score = tile_world_dst(unit_x, unit_y, plan.plan.x as i32, plan.plan.y as i32)
            - prebuild_ai_category_priority(plan.block.category) * PREBUILD_AI_PRIORITY_DST_SCALE;
        if best_index.is_none() || score < best_score {
            best_index = Some(index);
            best_score = score;
        }
    }

    best_index
        .map(|index| plans[index].clone())
        .or_else(|| plans.first().cloned())
}

pub fn prebuild_ai_pick_missing_collect_target_item<FCoreHas>(
    requirements: &[PrebuildAiRequirement],
    build_cost_multiplier: f32,
    last_target_item: Option<ContentId>,
    mut core_has_item: FCoreHas,
) -> PrebuildAiMiningTarget
where
    FCoreHas: FnMut(ContentId, i32) -> bool,
{
    let target_item = requirements
        .iter()
        .find(|requirement| {
            !core_has_item(
                requirement.item,
                scaled_requirement_amount(requirement.amount, build_cost_multiplier),
            )
        })
        .map(|requirement| requirement.item);

    PrebuildAiMiningTarget {
        target_item: target_item.or(last_target_item),
        last_target_item: target_item.or(last_target_item),
    }
}

pub fn prebuild_ai_handle_full_core_for_target_item(
    target_item: Option<ContentId>,
    core_accept_stack_one: i32,
) -> PrebuildAiFullCoreAction {
    let handled = target_item.is_some() && core_accept_stack_one == 0;
    PrebuildAiFullCoreAction {
        handled,
        clear_item: handled,
        clear_mine_tile: handled,
    }
}

pub fn prebuild_ai_should_stop_mining_for_carry_limit_or_acceptance(
    target_item: Option<ContentId>,
    stack_amount: i32,
    item_capacity: i32,
    accepts_target_item: bool,
) -> bool {
    target_item.is_none() || stack_amount >= item_capacity || !accepts_target_item
}

pub fn prebuild_ai_refresh_mining_ore_target<FFloor, FWall>(
    current_ore: Option<i32>,
    timer_ready: bool,
    target_item: Option<ContentId>,
    mine_floor: bool,
    mine_walls: bool,
    mut find_floor_ore: FFloor,
    mut find_wall_ore: FWall,
) -> PrebuildAiOreTarget
where
    FFloor: FnMut(ContentId) -> Option<i32>,
    FWall: FnMut(ContentId) -> Option<i32>,
{
    let Some(target_item) = target_item else {
        return PrebuildAiOreTarget::Existing(current_ore);
    };
    if !timer_ready {
        return PrebuildAiOreTarget::Existing(current_ore);
    }

    let floor = mine_floor.then(|| find_floor_ore(target_item)).flatten();
    let ore = floor.or_else(|| mine_walls.then(|| find_wall_ore(target_item)).flatten());
    PrebuildAiOreTarget::Refreshed(ore)
}

pub fn prebuild_ai_handle_return_to_core_with_items(
    stack_amount: i32,
    within_core_range: bool,
    core_accept_stack_amount: i32,
    can_build_after_deposit: bool,
) -> PrebuildAiReturnToCoreAction {
    if stack_amount <= 0 {
        return PrebuildAiReturnToCoreAction {
            mining: true,
            collecting_items: !can_build_after_deposit,
            clear_mine_tile: true,
            clear_item: false,
            transfer_to_core: false,
        };
    }

    let at_core = within_core_range;
    PrebuildAiReturnToCoreAction {
        mining: at_core,
        collecting_items: if at_core {
            !can_build_after_deposit
        } else {
            true
        },
        clear_mine_tile: true,
        clear_item: at_core,
        transfer_to_core: at_core && core_accept_stack_amount > 0,
    }
}

pub fn validate_prebuild_ai_current_plan<FValidBreak, FValidPlace>(
    unit_plans: &mut VecDeque<BuildPlan>,
    last_plan: &mut Option<TeamBlockPlan>,
    construct_current_matches: bool,
    mut valid_break: FValidBreak,
    mut valid_place: FValidPlace,
) -> BuilderAiPlanValidation
where
    FValidBreak: FnMut(&BuildPlan) -> bool,
    FValidPlace: FnMut(&BuildPlan) -> bool,
{
    validate_builder_ai_current_plan(
        unit_plans,
        last_plan,
        false,
        true,
        true,
        false,
        construct_current_matches,
        &mut valid_break,
        &mut valid_place,
    )
}

pub fn prebuild_ai_build_move(build_range: f32, within_range: bool) -> PrebuildAiBuildMove {
    let range = (build_range - 20.0).min(100.0);
    PrebuildAiBuildMove {
        range,
        move_range: range - 10.0,
        moving: !within_range,
    }
}

pub fn prebuild_ai_update_building(collecting_items: bool) -> bool {
    !collecting_items
}

pub fn prebuild_ai_accept_plan<FValidPlace, FCanBuild>(
    plan: Option<PrebuildAiPlanSnapshot>,
    mut valid_place: FValidPlace,
    mut can_build: FCanBuild,
) -> PrebuildAiAcceptPlanAction
where
    FValidPlace: FnMut(&PrebuildAiPlanSnapshot) -> bool,
    FCanBuild: FnMut(&PrebuildAiPlanSnapshot) -> bool,
{
    let Some(plan) = plan else {
        return PrebuildAiAcceptPlanAction {
            accepted: false,
            collecting_items: false,
            collect_block: None,
            last_target_item: None,
            ore: None,
            reset_target_timer: false,
            last_plan: None,
            build_plan: None,
        };
    };

    if !valid_place(&plan) {
        return PrebuildAiAcceptPlanAction {
            accepted: false,
            collecting_items: false,
            collect_block: None,
            last_target_item: None,
            ore: None,
            reset_target_timer: false,
            last_plan: None,
            build_plan: None,
        };
    }

    let can_build_now = can_build(&plan);
    let mut build_plan = BuildPlan::new_place(
        plan.plan.x as i32,
        plan.plan.y as i32,
        plan.plan.rotation as i32,
        plan.plan.block.clone(),
    );
    if let Some(config) = &plan.plan.config {
        build_plan.config = crate::mindustry::io::TypeValue::String(config.clone());
    }

    PrebuildAiAcceptPlanAction {
        accepted: true,
        collecting_items: !can_build_now,
        collect_block: (!can_build_now).then_some(plan.block),
        last_target_item: None,
        ore: None,
        reset_target_timer: !can_build_now,
        last_plan: Some(plan.plan),
        build_plan: Some(build_plan),
    }
}

pub fn builder_ai_init_rebuild_period(
    rebuild_period: f32,
    default_rebuild_period: f32,
    team_build_ai: bool,
) -> f32 {
    if rebuild_period == default_rebuild_period && team_build_ai {
        BUILDER_AI_BUILD_AI_REBUILD_PERIOD
    } else {
        rebuild_period
    }
}

pub fn builder_ai_should_promote_assist_following(
    assist_valid: bool,
    assist_actively_building: bool,
) -> bool {
    assist_valid && assist_actively_building
}

pub fn builder_ai_should_fire(command_ai_should_fire: Option<bool>) -> bool {
    command_ai_should_fire.unwrap_or(true)
}

pub fn builder_ai_should_shoot(unit_is_building: bool, unit_can_attack: bool) -> bool {
    !unit_is_building && unit_can_attack
}

pub fn builder_ai_enemy_search_rect(
    tile_x: i32,
    tile_y: i32,
    flee_range: f32,
) -> BuilderAiEnemySearchRect {
    let half = flee_range / 2.0;
    BuilderAiEnemySearchRect {
        x: tile_x as f32 * TILE_SIZE as f32 - half,
        y: tile_y as f32 * TILE_SIZE as f32 - half,
        width: flee_range,
        height: flee_range,
    }
}

pub fn builder_ai_near_enemy<FNearEnemy>(
    tile_x: i32,
    tile_y: i32,
    flee_range: f32,
    mut near_enemy: FNearEnemy,
) -> bool
where
    FNearEnemy: FnMut(BuilderAiEnemySearchRect) -> bool,
{
    near_enemy(builder_ai_enemy_search_rect(tile_x, tile_y, flee_range))
}

pub fn builder_ai_fallback_controller(
    team_is_ai: bool,
    team_prebuild_ai: bool,
    unit_flying: bool,
) -> BuilderAiFallbackController {
    if team_is_ai && team_prebuild_ai {
        BuilderAiFallbackController::Prebuild
    } else if unit_flying {
        BuilderAiFallbackController::Flying
    } else {
        BuilderAiFallbackController::Ground
    }
}

pub fn builder_ai_use_fallback(
    team_is_ai: bool,
    team_prebuild_ai: bool,
    state_waves: bool,
    unit_is_wave_team: bool,
    team_rts_ai: bool,
) -> bool {
    if team_is_ai && team_prebuild_ai {
        true
    } else {
        state_waves && unit_is_wave_team && !team_rts_ai
    }
}

pub fn builder_ai_idle_retreat(
    retreat_timer: f32,
    delta: f32,
    retreat_delay: f32,
    always_flee: bool,
    has_enemy: bool,
    has_core: bool,
    within_retreat_distance: bool,
) -> BuilderAiRetreatDecision {
    let next_timer = retreat_timer + delta;
    let should_retreat = next_timer >= retreat_delay || always_flee;
    let clear_building = should_retreat && has_enemy;
    let move_to_core = clear_building && has_core && !within_retreat_distance;

    BuilderAiRetreatDecision {
        retreat_timer: next_timer,
        clear_building,
        move_to_core,
        moving: move_to_core,
    }
}

pub fn sync_builder_ai_follow_plan(
    unit_plans: &mut VecDeque<BuildPlan>,
    last_plan: &mut Option<TeamBlockPlan>,
    following_present: bool,
    following_valid: bool,
    following_actively_building: bool,
    following_plan: Option<BuildPlan>,
) -> BuilderAiFollowSync {
    if !following_present {
        return BuilderAiFollowSync {
            action: BuilderAiFollowAction::NoFollower,
            clear_following: false,
            reset_retreat_timer: false,
            copied_plan: None,
        };
    }

    if !following_valid || !following_actively_building {
        unit_plans.clear();
        return BuilderAiFollowSync {
            action: BuilderAiFollowAction::ClearInvalidFollower,
            clear_following: true,
            reset_retreat_timer: true,
            copied_plan: None,
        };
    }

    unit_plans.clear();
    if let Some(plan) = &following_plan {
        unit_plans.push_front(plan.clone());
    }
    *last_plan = None;

    BuilderAiFollowSync {
        action: BuilderAiFollowAction::CopyFollowerPlan,
        clear_following: false,
        reset_retreat_timer: true,
        copied_plan: following_plan,
    }
}

pub fn validate_builder_ai_current_plan<FValidBreak, FValidPlace>(
    unit_plans: &mut VecDeque<BuildPlan>,
    last_plan: &mut Option<TeamBlockPlan>,
    hold: bool,
    infinite_resources: bool,
    within_hold_range: bool,
    conflicting_breaker: bool,
    construct_current_matches: bool,
    mut valid_break: FValidBreak,
    mut valid_place: FValidPlace,
) -> BuilderAiPlanValidation
where
    FValidBreak: FnMut(&BuildPlan) -> bool,
    FValidPlace: FnMut(&BuildPlan) -> bool,
{
    let Some(request) = unit_plans.front().cloned() else {
        return BuilderAiPlanValidation {
            action: BuilderAiPlanAction::NoPlan,
            removed_plan: None,
            remove_team_plan_at: None,
        };
    };

    if !request.breaking && conflicting_breaker {
        let removed_plan = unit_plans.pop_front();
        return BuilderAiPlanValidation {
            action: BuilderAiPlanAction::DropConflictingBreak,
            removed_plan,
            remove_team_plan_at: Some((request.x, request.y)),
        };
    }

    let last_plan_removed = last_plan
        .as_ref()
        .is_some_and(|last_plan| last_plan.removed);
    let valid = !last_plan_removed
        && (construct_current_matches
            || if request.breaking {
                valid_break(&request)
            } else {
                valid_place(&request)
            });

    if !valid {
        let removed_plan = unit_plans.pop_front();
        *last_plan = None;
        return BuilderAiPlanValidation {
            action: BuilderAiPlanAction::DropInvalid,
            removed_plan,
            remove_team_plan_at: None,
        };
    }

    if hold && !within_hold_range && !infinite_resources {
        let removed_plan = unit_plans.pop_front();
        *last_plan = None;
        return BuilderAiPlanValidation {
            action: BuilderAiPlanAction::DropHoldOutOfRange,
            removed_plan,
            remove_team_plan_at: None,
        };
    }

    BuilderAiPlanValidation {
        action: BuilderAiPlanAction::Keep,
        removed_plan: None,
        remove_team_plan_at: None,
    }
}

pub fn begin_path_refresh(state: &mut BaseBuilderPathState) {
    state.calculating = true;
    state.started_calculating = true;
    state.calc_tile = None;
    state.calc_path.clear();
}

pub fn step_core_path<F>(
    state: &mut BaseBuilderPathState,
    start: Option<TilePoint>,
    weights: &[i32],
    world_width: i32,
    world_height: i32,
    max_steps: usize,
    mut is_enemy_core: F,
) -> PathCalculationOutcome
where
    F: FnMut(TilePoint) -> bool,
{
    if !state.calculating {
        return PathCalculationOutcome::Idle;
    }

    if state.calc_count >= world_width.saturating_mul(world_height) {
        state.calculating = false;
        state.calc_count = 0;
        state.calc_path.clear();
        state.calc_tile = None;
        state.total_calcs += 1;
        return PathCalculationOutcome::Exhausted;
    }

    if state.calc_tile.is_none() {
        if let Some(start) = start {
            state.calc_tile = Some(start);
            return PathCalculationOutcome::Seeded;
        }

        state.calculating = false;
        return PathCalculationOutcome::NoStart;
    }

    if weights.len() < world_width.saturating_mul(world_height).max(0) as usize {
        return PathCalculationOutcome::WaitingWeights;
    }

    let mut advanced = false;
    for _ in 0..max_steps {
        let current = state.calc_tile.expect("calc_tile checked above");
        let mut min_cost = i32::MAX;
        let mut best = None;

        for delta in D4 {
            let next = TilePoint::new(current.x + delta.x, current.y + delta.y);
            if let Some(index) = world_index(next.x, next.y, world_width, world_height) {
                let cost = weights[index];
                if cost != -1 && cost < min_cost {
                    min_cost = cost;
                    best = Some(next);
                }
            }
        }

        let Some(next) = best else {
            state.calc_count = i32::MAX;
            return PathCalculationOutcome::Stuck;
        };

        state.calc_tile = Some(next);
        state.calc_path.insert(next.packed());
        for delta in D8 {
            state
                .calc_path
                .insert(TilePoint::new(next.x + delta.x, next.y + delta.y).packed());
        }

        if is_enemy_core(next) {
            state.calculating = false;
            state.calc_count = 0;
            state.path.clear();
            state.path.extend(state.calc_path.iter().copied());
            state.calc_path.clear();
            state.calc_tile = None;
            state.total_calcs += 1;
            state.found_path = true;
            return PathCalculationOutcome::Found;
        }

        state.calc_count += 1;
        advanced = true;
    }

    if advanced {
        PathCalculationOutcome::Advanced
    } else {
        PathCalculationOutcome::Idle
    }
}

pub fn rotate_center(
    center_x: i32,
    center_y: i32,
    width: i32,
    height: i32,
    steps: i32,
) -> TilePoint {
    let axis_x = width / 2;
    let axis_y = height / 2;
    match steps.rem_euclid(4) {
        0 => TilePoint::new(center_x, center_y),
        1 => TilePoint::new(axis_x + axis_y - center_y, axis_y - axis_x + center_x),
        2 => TilePoint::new(axis_x * 2 - center_x, axis_y * 2 - center_y),
        _ => TilePoint::new(axis_x - axis_y + center_y, axis_y + axis_x - center_x),
    }
}

pub fn rotate_build_tile(
    tile: &BaseBuildTile,
    schematic_width: i32,
    schematic_height: i32,
    steps: i32,
) -> BaseBuildTile {
    let mut out = tile.clone();
    let mut width = schematic_width;
    let mut height = schematic_height;

    for _ in 0..steps.rem_euclid(4) {
        let ox = width / 2;
        let oy = height / 2;
        let wx = (out.x - ox) as f32 * TILE_SIZE as f32 + out.block_offset;
        let wy = (out.y - oy) as f32 * TILE_SIZE as f32 + out.block_offset;
        let rotated_x = -wy;
        let rotated_y = wx;

        out.x = world_to_tile(rotated_x - out.block_offset) + ox;
        out.y = world_to_tile(rotated_y - out.block_offset) + oy;
        out.rotation = (out.rotation + 1).rem_euclid(4);
        std::mem::swap(&mut width, &mut height);
    }

    out
}

pub fn try_place_part<FValid, FResource, FAdjacent>(
    part: &BaseBuildPart,
    anchor_x: i32,
    anchor_y: i32,
    rotation_steps: i32,
    path: &HashSet<i32>,
    mut valid_place: FValid,
    mut resource_at: FResource,
    mut adjacent_building: FAdjacent,
) -> Option<Vec<BlockPlan>>
where
    FValid: FnMut(&BaseBuildTile, i32, i32, i32) -> bool,
    FResource: FnMut(i32, i32) -> Option<String>,
    FAdjacent: FnMut(i32, i32) -> bool,
{
    let rotation_steps = rotation_steps.rem_euclid(4);
    let rotated_center = rotate_center(
        part.center_x,
        part.center_y,
        part.width,
        part.height,
        rotation_steps,
    );
    let corner_x = anchor_x - rotated_center.x;
    let corner_y = anchor_y - rotated_center.y;
    let rotated_tiles = part
        .tiles
        .iter()
        .map(|tile| rotate_build_tile(tile, part.width, part.height, rotation_steps))
        .collect::<Vec<_>>();

    for tile in &rotated_tiles {
        let real_x = corner_x + tile.x;
        let real_y = corner_y + tile.y;
        if !valid_place(tile, real_x, real_y, tile.rotation) {
            return None;
        }

        if tile.requires_payload_clearance {
            for edge in get_edges(tile.block_size) {
                if adjacent_building(real_x + edge.x, real_y + edge.y) {
                    return None;
                }
            }
        }

        if tile.solid
            && footprint_tiles(real_x, real_y, tile.block_size)
                .into_iter()
                .any(|(x, y)| path.contains(&point2_pack(x, y)))
        {
            return None;
        }
    }

    if let Some(required) = &part.required {
        let mut correct = 0;
        let mut incorrect = 0;
        let mut any_drills = false;

        for tile in &rotated_tiles {
            if tile.kind == BasePartTileKind::Drill {
                any_drills = true;
                let real_x = corner_x + tile.x;
                let real_y = corner_y + tile.y;

                for offset in &tile.taken_offsets {
                    match resource_at(real_x + offset.x, real_y + offset.y) {
                        Some(resource) if resource == *required => correct += 1,
                        Some(_) => incorrect += 1,
                        None => {}
                    }
                }
            }
        }

        if any_drills && (incorrect != 0 || correct == 0) {
            return None;
        }
    }

    Some(
        rotated_tiles
            .into_iter()
            .map(|tile| BlockPlan {
                x: corner_x + tile.x,
                y: corner_y + tile.y,
                rotation: tile.rotation,
                block_name: tile.block_name,
                config: tile.config,
            })
            .collect(),
    )
}

fn world_index(x: i32, y: i32, world_width: i32, world_height: i32) -> Option<usize> {
    (x >= 0 && y >= 0 && x < world_width && y < world_height)
        .then_some((x + y * world_width) as usize)
}

fn scaled_requirement_amount(amount: i32, build_cost_multiplier: f32) -> i32 {
    (amount as f32 * build_cost_multiplier).ceil() as i32
}

fn tile_world_dst(unit_x: f32, unit_y: f32, tile_x: i32, tile_y: i32) -> f32 {
    let dx = unit_x - tile_x as f32 * TILE_SIZE as f32;
    let dy = unit_y - tile_y as f32 * TILE_SIZE as f32;
    (dx * dx + dy * dy).sqrt()
}

fn world_to_tile(coord: f32) -> i32 {
    (coord / TILE_SIZE as f32).round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drill(x: i32, y: i32) -> BaseBuildTile {
        let mut tile = BaseBuildTile::new(x, y, "mechanical-drill", BasePartTileKind::Drill);
        tile.solid = true;
        tile
    }

    fn prebuild_plan(
        x: i32,
        y: i32,
        name: &str,
        category: Category,
        build_time: f32,
        requirements: Vec<PrebuildAiRequirement>,
    ) -> PrebuildAiPlanSnapshot {
        let mut block = PrebuildAiBlockInfo::new(name, category);
        block.build_time = build_time;
        block.requirements = requirements;
        PrebuildAiPlanSnapshot::new(TeamBlockPlan::new(x, y, 0, name, None), block)
    }

    #[test]
    fn core_spawn_limit_intervals_and_pool_choices_match_java_rules() {
        assert!(should_spawn_core_unit(true, true, true, false, 1, 1));
        assert!(!should_spawn_core_unit(true, true, true, false, 2, 1));
        assert!(!should_spawn_core_unit(true, true, true, true, 0, 1));

        assert_eq!(place_interval(0.0), PLACE_INTERVAL_MIN);
        assert_eq!(place_interval(1.0), PLACE_INTERVAL_MAX);
        assert_eq!(
            random_position_source(true, false, 0),
            Some(SeedPositionSource::Core)
        );
        assert_eq!(
            random_position_source(false, true, 2),
            Some(SeedPositionSource::Spawn)
        );
        assert_eq!(random_position_source(false, false, 2), None);

        assert_eq!(
            choose_part_pool(Some("copper"), true, 0.9),
            PartPoolChoice::Resource
        );
        assert_eq!(choose_part_pool(None, false, 0.0), PartPoolChoice::Generic);
        assert_eq!(choose_part_pool(None, false, 0.5), PartPoolChoice::None);
    }

    #[test]
    fn prebuild_ai_sort_plans_reverses_java_sorted_copy_into_priority_queue() {
        assert_eq!(
            prebuild_ai_category_priority(Category::Production),
            PREBUILD_AI_PRODUCTION_PRIORITY
        );
        assert_eq!(
            prebuild_ai_category_priority(Category::Distribution),
            PREBUILD_AI_DISTRIBUTION_PRIORITY
        );
        assert_eq!(
            prebuild_ai_category_priority(Category::Liquid),
            PREBUILD_AI_LIQUID_PRIORITY
        );
        assert_eq!(
            prebuild_ai_category_priority(Category::Crafting),
            PREBUILD_AI_CRAFTING_PRIORITY
        );
        assert_eq!(prebuild_ai_category_priority(Category::Turret), 0.0);

        let mut plans = vec![
            prebuild_plan(1, 1, "graphite-press", Category::Crafting, 20.0, vec![]),
            prebuild_plan(2, 2, "router", Category::Distribution, 8.0, vec![]),
            prebuild_plan(3, 3, "junction", Category::Distribution, 18.0, vec![]),
            prebuild_plan(4, 4, "mechanical-drill", Category::Production, 12.0, vec![]),
        ];

        prebuild_ai_sort_plans(&mut plans);

        assert_eq!(
            plans
                .iter()
                .map(|plan| plan.plan.block.as_str())
                .collect::<Vec<_>>(),
            vec!["mechanical-drill", "junction", "router", "graphite-press"]
        );
    }

    #[test]
    fn prebuild_ai_can_build_uses_infinite_flags_or_scaled_core_items() {
        let requirements = [
            PrebuildAiRequirement::new(1, 2),
            PrebuildAiRequirement::new(2, 5),
        ];

        assert!(prebuild_ai_can_build(
            true,
            false,
            &requirements,
            10.0,
            |_, _| false,
        ));
        assert!(prebuild_ai_can_build(
            false,
            true,
            &requirements,
            10.0,
            |_, _| false,
        ));
        assert!(prebuild_ai_can_build(
            false,
            false,
            &requirements,
            1.5,
            |item, amount| matches!((item, amount), (1, 3) | (2, 8)),
        ));
        assert!(!prebuild_ai_can_build(
            false,
            false,
            &requirements,
            1.5,
            |item, amount| item == 1 && amount == 3,
        ));
    }

    #[test]
    fn prebuild_ai_tree_query_and_find_next_plan_match_java_filters() {
        let mut bridge = prebuild_plan(
            2,
            3,
            "bridge-conveyor",
            Category::Distribution,
            12.0,
            vec![PrebuildAiRequirement::new(1, 1)],
        );
        bridge.block.offset = 4.0;
        bridge.block.size = 2;

        let query = prebuild_ai_tree_query(&bridge.plan, &bridge.block);
        assert_eq!(
            query,
            PrebuildAiTreeQuery {
                x: 2.0 * TILE_SIZE as f32 + 4.0 - (2.0 * TILE_SIZE as f32 + 1.0) / 2.0,
                y: 3.0 * TILE_SIZE as f32 + 4.0 - (2.0 * TILE_SIZE as f32 + 1.0) / 2.0,
                width: 2.0 * TILE_SIZE as f32 + 1.0,
                height: 2.0 * TILE_SIZE as f32 + 1.0,
            }
        );

        let far_production = prebuild_plan(
            20,
            0,
            "mechanical-drill",
            Category::Production,
            10.0,
            vec![PrebuildAiRequirement::new(2, 1)],
        );
        let near_crafting = prebuild_plan(
            1,
            0,
            "graphite-press",
            Category::Crafting,
            8.0,
            vec![PrebuildAiRequirement::new(3, 1)],
        );
        let unreachable = prebuild_plan(
            0,
            1,
            "router",
            Category::Distribution,
            4.0,
            vec![PrebuildAiRequirement::new(4, 1)],
        );
        let plans = vec![
            unreachable.clone(),
            near_crafting.clone(),
            far_production.clone(),
        ];

        let selected = prebuild_ai_find_next_plan(
            &plans,
            true,
            true,
            0.0,
            0.0,
            |_| false,
            |item| item == 2 || item == 3,
            |query| query.y == 0.0,
        )
        .expect("production plan should win by Java priority score");
        assert_eq!(selected.plan.block, "mechanical-drill");

        let no_core =
            prebuild_ai_find_next_plan(&plans, false, true, 0.0, 0.0, |_| true, |_| true, |_| true);
        assert_eq!(no_core, None);

        let fallback = prebuild_ai_find_next_plan(
            &plans,
            true,
            true,
            0.0,
            0.0,
            |_| false,
            |_| false,
            |_| false,
        )
        .expect("Java falls back to plans.first() when no min candidate matches");
        assert_eq!(fallback, unreachable);
    }

    #[test]
    fn prebuild_ai_pick_missing_collect_target_item_updates_last_target_only_on_new_gap() {
        let requirements = [
            PrebuildAiRequirement::new(1, 2),
            PrebuildAiRequirement::new(2, 5),
        ];

        let missing = prebuild_ai_pick_missing_collect_target_item(
            &requirements,
            1.5,
            Some(9),
            |item, amount| item == 1 && amount == 3,
        );
        assert_eq!(
            missing,
            PrebuildAiMiningTarget {
                target_item: Some(2),
                last_target_item: Some(2),
            }
        );

        let fallback =
            prebuild_ai_pick_missing_collect_target_item(&requirements, 1.5, Some(9), |_, _| true);
        assert_eq!(
            fallback,
            PrebuildAiMiningTarget {
                target_item: Some(9),
                last_target_item: Some(9),
            }
        );

        let none =
            prebuild_ai_pick_missing_collect_target_item(&requirements, 1.0, None, |_, _| true);
        assert_eq!(
            none,
            PrebuildAiMiningTarget {
                target_item: None,
                last_target_item: None,
            }
        );
    }

    #[test]
    fn prebuild_ai_mining_branch_handles_full_core_and_stop_conditions() {
        assert_eq!(
            prebuild_ai_handle_full_core_for_target_item(Some(1), 0),
            PrebuildAiFullCoreAction {
                handled: true,
                clear_item: true,
                clear_mine_tile: true,
            }
        );
        assert_eq!(
            prebuild_ai_handle_full_core_for_target_item(Some(1), 1),
            PrebuildAiFullCoreAction {
                handled: false,
                clear_item: false,
                clear_mine_tile: false,
            }
        );
        assert!(!prebuild_ai_handle_full_core_for_target_item(None, 0).handled);

        assert!(prebuild_ai_should_stop_mining_for_carry_limit_or_acceptance(None, 0, 10, true,));
        assert!(
            prebuild_ai_should_stop_mining_for_carry_limit_or_acceptance(Some(1), 10, 10, true,)
        );
        assert!(
            prebuild_ai_should_stop_mining_for_carry_limit_or_acceptance(Some(1), 1, 10, false,)
        );
        assert!(
            !prebuild_ai_should_stop_mining_for_carry_limit_or_acceptance(Some(1), 1, 10, true,)
        );
    }

    #[test]
    fn prebuild_ai_refresh_mining_ore_target_prefers_floor_then_wall() {
        assert_eq!(
            prebuild_ai_refresh_mining_ore_target(
                Some(7),
                false,
                Some(1),
                true,
                true,
                |_| Some(1),
                |_| Some(2)
            ),
            PrebuildAiOreTarget::Existing(Some(7))
        );
        assert_eq!(
            prebuild_ai_refresh_mining_ore_target(
                None,
                true,
                None,
                true,
                true,
                |_| Some(1),
                |_| Some(2)
            ),
            PrebuildAiOreTarget::Existing(None)
        );
        assert_eq!(
            prebuild_ai_refresh_mining_ore_target(
                None,
                true,
                Some(1),
                true,
                true,
                |_| Some(11),
                |_| Some(22)
            ),
            PrebuildAiOreTarget::Refreshed(Some(11))
        );
        assert_eq!(
            prebuild_ai_refresh_mining_ore_target(
                None,
                true,
                Some(1),
                true,
                true,
                |_| None,
                |_| Some(22)
            ),
            PrebuildAiOreTarget::Refreshed(Some(22))
        );
        assert_eq!(
            prebuild_ai_refresh_mining_ore_target(
                None,
                true,
                Some(1),
                false,
                false,
                |_| Some(11),
                |_| Some(22)
            ),
            PrebuildAiOreTarget::Refreshed(None)
        );
    }

    #[test]
    fn prebuild_ai_return_to_core_state_matches_java_deposit_branch() {
        assert_eq!(
            prebuild_ai_handle_return_to_core_with_items(0, false, 0, false),
            PrebuildAiReturnToCoreAction {
                mining: true,
                collecting_items: true,
                clear_mine_tile: true,
                clear_item: false,
                transfer_to_core: false,
            }
        );
        assert_eq!(
            prebuild_ai_handle_return_to_core_with_items(0, false, 0, true),
            PrebuildAiReturnToCoreAction {
                mining: true,
                collecting_items: false,
                clear_mine_tile: true,
                clear_item: false,
                transfer_to_core: false,
            }
        );
        assert_eq!(
            prebuild_ai_handle_return_to_core_with_items(5, true, 5, true),
            PrebuildAiReturnToCoreAction {
                mining: true,
                collecting_items: false,
                clear_mine_tile: true,
                clear_item: true,
                transfer_to_core: true,
            }
        );
        assert_eq!(
            prebuild_ai_handle_return_to_core_with_items(5, true, 0, false),
            PrebuildAiReturnToCoreAction {
                mining: true,
                collecting_items: true,
                clear_mine_tile: true,
                clear_item: true,
                transfer_to_core: false,
            }
        );
        assert_eq!(
            prebuild_ai_handle_return_to_core_with_items(5, false, 5, true),
            PrebuildAiReturnToCoreAction {
                mining: false,
                collecting_items: true,
                clear_mine_tile: true,
                clear_item: false,
                transfer_to_core: false,
            }
        );
    }

    #[test]
    fn prebuild_ai_current_plan_validation_and_move_range_match_java_update_movement() {
        let mut unit_plans = VecDeque::from([BuildPlan::new_place(4, 5, 1, "router")]);
        let mut last_plan = Some(TeamBlockPlan::new(4, 5, 1, "router", None));

        let keep = validate_prebuild_ai_current_plan(
            &mut unit_plans,
            &mut last_plan,
            false,
            |_| false,
            |plan| plan.block.as_deref() == Some("router"),
        );
        assert_eq!(keep.action, BuilderAiPlanAction::Keep);
        assert_eq!(unit_plans.len(), 1);
        assert!(last_plan.is_some());

        let move_plan = prebuild_ai_build_move(160.0, false);
        assert_eq!(
            move_plan,
            PrebuildAiBuildMove {
                range: 100.0,
                move_range: 90.0,
                moving: true,
            }
        );
        assert_eq!(prebuild_ai_build_move(70.0, true).range, 50.0);
        assert!(!prebuild_ai_build_move(70.0, true).moving);

        let mut removed_last = Some(TeamBlockPlan {
            removed: true,
            ..TeamBlockPlan::new(4, 5, 1, "router", None)
        });
        let invalid = validate_prebuild_ai_current_plan(
            &mut unit_plans,
            &mut removed_last,
            true,
            |_| true,
            |_| true,
        );
        assert_eq!(invalid.action, BuilderAiPlanAction::DropInvalid);
        assert_eq!(
            invalid.removed_plan,
            Some(BuildPlan::new_place(4, 5, 1, "router"))
        );
        assert!(unit_plans.is_empty());
        assert_eq!(removed_last, None);
    }

    #[test]
    fn prebuild_ai_accept_plan_sets_collecting_state_only_when_core_lacks_items() {
        let plan = prebuild_plan(
            6,
            7,
            "router",
            Category::Distribution,
            3.0,
            vec![PrebuildAiRequirement::new(1, 1)],
        );

        assert!(!prebuild_ai_update_building(true));
        assert!(prebuild_ai_update_building(false));

        let ready = prebuild_ai_accept_plan(Some(plan.clone()), |_| true, |_| true);
        assert!(ready.accepted);
        assert!(!ready.collecting_items);
        assert_eq!(ready.collect_block, None);
        assert_eq!(ready.last_plan, Some(plan.plan.clone()));
        assert_eq!(
            ready.build_plan,
            Some(BuildPlan::new_place(6, 7, 0, "router"))
        );
        assert!(!ready.reset_target_timer);

        let collect = prebuild_ai_accept_plan(Some(plan.clone()), |_| true, |_| false);
        assert!(collect.accepted);
        assert!(collect.collecting_items);
        assert_eq!(collect.collect_block, Some(plan.block.clone()));
        assert_eq!(collect.last_target_item, None);
        assert_eq!(collect.ore, None);
        assert!(collect.reset_target_timer);

        let invalid = prebuild_ai_accept_plan(Some(plan), |_| false, |_| false);
        assert!(!invalid.accepted);
        assert_eq!(invalid.build_plan, None);

        let none = prebuild_ai_accept_plan(None, |_| true, |_| true);
        assert!(!none.accepted);
    }

    #[test]
    fn builder_ai_rebuild_claim_removes_placed_or_rotates_invalid_front_plan() {
        let mut team_data = crate::mindustry::game::TeamData::new(1);
        team_data.plans = vec![
            TeamBlockPlan::new(1, 1, 0, "duo", None),
            TeamBlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
        ];

        let placed = claim_builder_ai_rebuild_plan(
            &mut team_data,
            false,
            |plan| plan.block == "duo",
            |_| panic!("placed blocks should be removed before validPlace"),
            |_| false,
        );
        assert_eq!(
            placed,
            TeamPlanClaim::AlreadyPlaced(TeamBlockPlan::new(1, 1, 0, "duo", None))
        );
        assert_eq!(
            team_data.plans,
            vec![TeamBlockPlan::new(2, 2, 1, "router", Some("cfg".into()))]
        );

        let rotated =
            claim_builder_ai_rebuild_plan(&mut team_data, false, |_| false, |_| false, |_| false);
        assert_eq!(
            rotated,
            TeamPlanClaim::Rotated(TeamBlockPlan::new(2, 2, 1, "router", Some("cfg".into())))
        );
        assert_eq!(
            team_data.plans,
            vec![TeamBlockPlan::new(2, 2, 1, "router", Some("cfg".into()))]
        );
    }

    #[test]
    fn builder_ai_rebuild_claim_accepts_valid_front_plan_and_respects_always_flee() {
        let mut team_data = crate::mindustry::game::TeamData::new(1);
        team_data.plans = vec![TeamBlockPlan::new(3, 3, 2, "wall", None)];

        let near_enemy = claim_builder_ai_rebuild_plan(
            &mut team_data,
            true,
            |_| false,
            |_| true,
            |plan| plan.x == 3 && plan.y == 3,
        );
        assert_eq!(
            near_enemy,
            TeamPlanClaim::Rotated(TeamBlockPlan::new(3, 3, 2, "wall", None))
        );

        let claimed =
            claim_builder_ai_rebuild_plan(&mut team_data, true, |_| false, |_| true, |_| false);
        assert_eq!(
            claimed,
            TeamPlanClaim::Claimed(TeamBlockPlan::new(3, 3, 2, "wall", None))
        );
        assert_eq!(
            team_data.plans,
            vec![TeamBlockPlan::new(3, 3, 2, "wall", None)]
        );
    }

    #[test]
    fn builder_ai_hold_claim_selects_first_reachable_valid_plan_and_rotates_to_tail() {
        let mut team_data = crate::mindustry::game::TeamData::new(1);
        team_data.plans = vec![
            TeamBlockPlan::new(1, 1, 0, "duo", None),
            TeamBlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
            TeamBlockPlan::new(3, 3, 2, "wall", None),
        ];

        let selected = claim_builder_ai_hold_plan(
            &mut team_data,
            false,
            |plan| plan.x == 2 || plan.x == 3,
            |plan| plan.block != "wall",
        );

        assert_eq!(
            selected,
            TeamPlanClaim::Claimed(TeamBlockPlan::new(2, 2, 1, "router", Some("cfg".into())))
        );
        assert_eq!(
            team_data.plans,
            vec![
                TeamBlockPlan::new(1, 1, 0, "duo", None),
                TeamBlockPlan::new(3, 3, 2, "wall", None),
                TeamBlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
            ]
        );
    }

    #[test]
    fn builder_ai_hold_claim_allows_infinite_resources_and_keeps_queue_on_no_match() {
        let mut team_data = crate::mindustry::game::TeamData::new(1);
        team_data.plans = vec![
            TeamBlockPlan::new(4, 4, 0, "duo", None),
            TeamBlockPlan::new(5, 5, 0, "router", None),
        ];
        let original = team_data.plans.clone();

        assert_eq!(
            claim_builder_ai_hold_plan(&mut team_data, false, |_| false, |_| true),
            TeamPlanClaim::NoUsablePlan
        );
        assert_eq!(team_data.plans, original);

        let selected =
            claim_builder_ai_hold_plan(&mut team_data, true, |_| false, |plan| plan.block == "duo");
        assert_eq!(
            selected,
            TeamPlanClaim::Claimed(TeamBlockPlan::new(4, 4, 0, "duo", None))
        );
        assert_eq!(
            team_data.plans,
            vec![
                TeamBlockPlan::new(5, 5, 0, "router", None),
                TeamBlockPlan::new(4, 4, 0, "duo", None),
            ]
        );
    }

    #[test]
    fn builder_ai_init_fire_shoot_and_enemy_rect_match_java_rules() {
        assert_eq!(
            builder_ai_init_rebuild_period(
                BUILDER_AI_DEFAULT_REBUILD_PERIOD,
                BUILDER_AI_DEFAULT_REBUILD_PERIOD,
                true,
            ),
            BUILDER_AI_BUILD_AI_REBUILD_PERIOD
        );
        assert_eq!(
            builder_ai_init_rebuild_period(
                BUILDER_AI_DEFAULT_REBUILD_PERIOD,
                BUILDER_AI_DEFAULT_REBUILD_PERIOD,
                false,
            ),
            BUILDER_AI_DEFAULT_REBUILD_PERIOD
        );
        assert_eq!(
            builder_ai_init_rebuild_period(30.0, BUILDER_AI_DEFAULT_REBUILD_PERIOD, true),
            30.0
        );

        assert!(builder_ai_should_fire(None));
        assert!(builder_ai_should_fire(Some(true)));
        assert!(!builder_ai_should_fire(Some(false)));

        assert!(builder_ai_should_shoot(false, true));
        assert!(!builder_ai_should_shoot(true, true));
        assert!(!builder_ai_should_shoot(false, false));

        let rect = builder_ai_enemy_search_rect(10, 20, 370.0);
        assert_eq!(
            rect,
            BuilderAiEnemySearchRect {
                x: 10.0 * TILE_SIZE as f32 - 185.0,
                y: 20.0 * TILE_SIZE as f32 - 185.0,
                width: 370.0,
                height: 370.0,
            }
        );
        assert!(builder_ai_near_enemy(10, 20, 370.0, |rect| {
            rect.x == 10.0 * TILE_SIZE as f32 - 185.0 && rect.width == 370.0
        }));
    }

    #[test]
    fn builder_ai_current_plan_drops_conflicting_break_and_invalid_front_only() {
        let mut unit_plans = VecDeque::from([
            BuildPlan::new_place(4, 4, 0, "router"),
            BuildPlan::new_place(5, 5, 1, "duo"),
        ]);
        let mut last_plan = Some(TeamBlockPlan::new(4, 4, 0, "router", None));

        let conflict = validate_builder_ai_current_plan(
            &mut unit_plans,
            &mut last_plan,
            false,
            false,
            true,
            true,
            false,
            |_| false,
            |_| true,
        );

        assert_eq!(conflict.action, BuilderAiPlanAction::DropConflictingBreak);
        assert_eq!(
            conflict.removed_plan,
            Some(BuildPlan::new_place(4, 4, 0, "router"))
        );
        assert_eq!(conflict.remove_team_plan_at, Some((4, 4)));
        assert_eq!(unit_plans.len(), 1);
        assert_eq!(unit_plans.front().unwrap().x, 5);
        assert!(last_plan.is_some());

        let mut last_plan = Some(TeamBlockPlan {
            removed: true,
            ..TeamBlockPlan::new(5, 5, 1, "duo", None)
        });
        let invalid = validate_builder_ai_current_plan(
            &mut unit_plans,
            &mut last_plan,
            false,
            false,
            true,
            false,
            true,
            |_| true,
            |_| true,
        );

        assert_eq!(invalid.action, BuilderAiPlanAction::DropInvalid);
        assert_eq!(
            invalid.removed_plan,
            Some(BuildPlan::new_place(5, 5, 1, "duo"))
        );
        assert!(unit_plans.is_empty());
        assert_eq!(last_plan, None);
    }

    #[test]
    fn builder_ai_current_plan_keeps_valid_or_drops_hold_out_of_range() {
        let mut unit_plans = VecDeque::from([BuildPlan::new_place(7, 8, 2, "scatter")]);
        let mut last_plan = Some(TeamBlockPlan::new(7, 8, 2, "scatter", None));

        let keep = validate_builder_ai_current_plan(
            &mut unit_plans,
            &mut last_plan,
            false,
            false,
            false,
            false,
            false,
            |_| false,
            |plan| plan.block.as_deref() == Some("scatter"),
        );

        assert_eq!(keep.action, BuilderAiPlanAction::Keep);
        assert_eq!(keep.removed_plan, None);
        assert_eq!(unit_plans.len(), 1);
        assert!(last_plan.is_some());

        let hold_drop = validate_builder_ai_current_plan(
            &mut unit_plans,
            &mut last_plan,
            true,
            false,
            false,
            false,
            false,
            |_| false,
            |_| true,
        );

        assert_eq!(hold_drop.action, BuilderAiPlanAction::DropHoldOutOfRange);
        assert_eq!(
            hold_drop.removed_plan,
            Some(BuildPlan::new_place(7, 8, 2, "scatter"))
        );
        assert!(unit_plans.is_empty());
        assert_eq!(last_plan, None);

        let mut infinite_plans = VecDeque::from([BuildPlan::new_place(9, 9, 0, "wall")]);
        let mut last_plan = Some(TeamBlockPlan::new(9, 9, 0, "wall", None));
        let keep_infinite = validate_builder_ai_current_plan(
            &mut infinite_plans,
            &mut last_plan,
            true,
            true,
            false,
            false,
            false,
            |_| false,
            |_| true,
        );

        assert_eq!(keep_infinite.action, BuilderAiPlanAction::Keep);
        assert_eq!(infinite_plans.len(), 1);
        assert!(last_plan.is_some());
    }

    #[test]
    fn builder_ai_follow_sync_clears_invalid_follower_and_promotes_assist() {
        assert!(builder_ai_should_promote_assist_following(true, true));
        assert!(!builder_ai_should_promote_assist_following(false, true));
        assert!(!builder_ai_should_promote_assist_following(true, false));

        let mut unit_plans = VecDeque::from([BuildPlan::new_place(1, 1, 0, "router")]);
        let mut last_plan = Some(TeamBlockPlan::new(1, 1, 0, "router", None));

        let no_follower =
            sync_builder_ai_follow_plan(&mut unit_plans, &mut last_plan, false, false, false, None);
        assert_eq!(no_follower.action, BuilderAiFollowAction::NoFollower);
        assert_eq!(unit_plans.len(), 1);
        assert!(last_plan.is_some());

        let cleared = sync_builder_ai_follow_plan(
            &mut unit_plans,
            &mut last_plan,
            true,
            false,
            true,
            Some(BuildPlan::new_place(2, 2, 0, "duo")),
        );
        assert_eq!(cleared.action, BuilderAiFollowAction::ClearInvalidFollower);
        assert!(cleared.clear_following);
        assert!(cleared.reset_retreat_timer);
        assert!(cleared.copied_plan.is_none());
        assert!(unit_plans.is_empty());
        assert!(last_plan.is_some());
    }

    #[test]
    fn builder_ai_follow_sync_copies_follower_plan_and_clears_last_plan() {
        let mut unit_plans = VecDeque::from([
            BuildPlan::new_place(3, 3, 0, "router"),
            BuildPlan::new_place(4, 4, 1, "duo"),
        ]);
        let mut last_plan = Some(TeamBlockPlan::new(3, 3, 0, "router", None));
        let follower_plan = BuildPlan::new_place(9, 10, 2, "scatter");

        let copied = sync_builder_ai_follow_plan(
            &mut unit_plans,
            &mut last_plan,
            true,
            true,
            true,
            Some(follower_plan.clone()),
        );

        assert_eq!(copied.action, BuilderAiFollowAction::CopyFollowerPlan);
        assert!(!copied.clear_following);
        assert!(copied.reset_retreat_timer);
        assert_eq!(copied.copied_plan, Some(follower_plan.clone()));
        assert_eq!(unit_plans.len(), 1);
        assert_eq!(unit_plans.front(), Some(&follower_plan));
        assert_eq!(last_plan, None);
    }

    #[test]
    fn builder_ai_idle_retreat_waits_for_delay_and_requires_enemy() {
        let waiting = builder_ai_idle_retreat(30.0, 10.0, 120.0, false, true, true, false);
        assert_eq!(
            waiting,
            BuilderAiRetreatDecision {
                retreat_timer: 40.0,
                clear_building: false,
                move_to_core: false,
                moving: false,
            }
        );

        let no_enemy = builder_ai_idle_retreat(120.0, 1.0, 120.0, false, false, true, false);
        assert_eq!(no_enemy.retreat_timer, 121.0);
        assert!(!no_enemy.clear_building);
        assert!(!no_enemy.move_to_core);
    }

    #[test]
    fn builder_ai_idle_retreat_clears_building_and_moves_to_core_when_needed() {
        let delayed = builder_ai_idle_retreat(119.0, 1.0, 120.0, false, true, true, false);
        assert_eq!(
            delayed,
            BuilderAiRetreatDecision {
                retreat_timer: 120.0,
                clear_building: true,
                move_to_core: true,
                moving: true,
            }
        );

        let already_near_core = builder_ai_idle_retreat(200.0, 1.0, 120.0, false, true, true, true);
        assert!(already_near_core.clear_building);
        assert!(!already_near_core.move_to_core);
        assert!(!already_near_core.moving);

        let always_flee = builder_ai_idle_retreat(0.0, 0.0, 120.0, true, true, false, false);
        assert!(always_flee.clear_building);
        assert!(!always_flee.move_to_core);
        assert!(!always_flee.moving);
    }

    #[test]
    fn builder_ai_fallback_controller_prefers_prebuild_then_unit_mobility() {
        assert_eq!(
            builder_ai_fallback_controller(true, true, false),
            BuilderAiFallbackController::Prebuild
        );
        assert_eq!(
            builder_ai_fallback_controller(false, true, true),
            BuilderAiFallbackController::Flying
        );
        assert_eq!(
            builder_ai_fallback_controller(false, false, false),
            BuilderAiFallbackController::Ground
        );
    }

    #[test]
    fn builder_ai_use_fallback_matches_prebuild_and_wave_team_rules() {
        assert!(builder_ai_use_fallback(true, true, false, false, true));
        assert!(builder_ai_use_fallback(false, false, true, true, false));
        assert!(!builder_ai_use_fallback(false, false, true, true, true));
        assert!(!builder_ai_use_fallback(false, false, false, true, false));
        assert!(!builder_ai_use_fallback(false, false, true, false, false));
    }

    #[test]
    fn path_calculation_follows_descending_complete_weights_and_flushes_path() {
        let mut state = BaseBuilderPathState::default();
        begin_path_refresh(&mut state);
        let weights = [4, 3, 2, 1, 0];

        assert_eq!(
            step_core_path(
                &mut state,
                Some(TilePoint::new(0, 0)),
                &weights,
                5,
                1,
                PATH_STEP,
                |point| point == TilePoint::new(4, 0),
            ),
            PathCalculationOutcome::Seeded
        );
        assert_eq!(
            step_core_path(
                &mut state,
                Some(TilePoint::new(0, 0)),
                &weights,
                5,
                1,
                PATH_STEP,
                |point| point == TilePoint::new(4, 0),
            ),
            PathCalculationOutcome::Found
        );

        assert!(!state.calculating);
        assert!(state.found_path);
        assert_eq!(state.total_calcs, 1);
        assert!(state.path.contains(&point2_pack(4, 0)));
        assert!(state.path.contains(&point2_pack(3, 0)));
        assert!(state.path.contains(&point2_pack(3, 1)));
        assert!(state.calc_path.is_empty());
    }

    #[test]
    fn path_calculation_handles_missing_start_and_stuck_weights() {
        let mut state = BaseBuilderPathState::default();
        begin_path_refresh(&mut state);
        assert_eq!(
            step_core_path(&mut state, None, &[], 4, 4, PATH_STEP, |_| false),
            PathCalculationOutcome::NoStart
        );
        assert!(!state.calculating);

        begin_path_refresh(&mut state);
        let weights = [0, -1, -1, -1];
        assert_eq!(
            step_core_path(
                &mut state,
                Some(TilePoint::new(0, 0)),
                &weights,
                2,
                2,
                PATH_STEP,
                |_| false,
            ),
            PathCalculationOutcome::Seeded
        );
        assert_eq!(
            step_core_path(
                &mut state,
                Some(TilePoint::new(0, 0)),
                &weights,
                2,
                2,
                PATH_STEP,
                |_| false,
            ),
            PathCalculationOutcome::Stuck
        );
        assert_eq!(state.calc_count, i32::MAX);
    }

    #[test]
    fn placement_rotates_validates_path_and_checks_required_drill_resource() {
        let mut belt = BaseBuildTile::new(2, 1, "conveyor", BasePartTileKind::Other);
        belt.rotation = 1;
        let part = BaseBuildPart {
            name: "copper-drill".into(),
            width: 3,
            height: 3,
            center_x: 1,
            center_y: 1,
            required: Some("copper".into()),
            tiles: vec![drill(1, 1), belt],
        };

        let mut blocked_path = HashSet::new();
        blocked_path.insert(point2_pack(10, 10));
        assert!(try_place_part(
            &part,
            10,
            10,
            0,
            &blocked_path,
            |_, _, _, _| true,
            |_, _| Some("copper".into()),
            |_, _| false,
        )
        .is_none());

        let plans = try_place_part(
            &part,
            10,
            10,
            1,
            &HashSet::new(),
            |_, _, _, _| true,
            |x, y| (x == 10 && y == 10).then_some("copper".into()),
            |_, _| false,
        )
        .expect("valid drill should be queued");

        assert_eq!(plans.len(), 2);
        assert!(plans
            .iter()
            .any(|plan| plan.block_name == "mechanical-drill"
                && plan.x == 10
                && plan.y == 10
                && plan.rotation == 1));
        assert!(plans
            .iter()
            .any(|plan| plan.block_name == "conveyor" && plan.rotation == 2));

        assert!(try_place_part(
            &part,
            10,
            10,
            0,
            &HashSet::new(),
            |_, _, _, _| true,
            |_, _| Some("lead".into()),
            |_, _| false,
        )
        .is_none());
    }

    #[test]
    fn payload_blocks_reject_adjacent_buildings() {
        let mut payload = BaseBuildTile::new(4, 4, "payload-conveyor", BasePartTileKind::Other);
        payload.requires_payload_clearance = true;
        payload.block_size = 1;
        let part = BaseBuildPart {
            name: "payload".into(),
            width: 8,
            height: 8,
            center_x: 4,
            center_y: 4,
            required: None,
            tiles: vec![payload],
        };

        assert!(try_place_part(
            &part,
            20,
            20,
            0,
            &HashSet::new(),
            |_, _, _, _| true,
            |_, _| None,
            |x, y| x == 21 && y == 20,
        )
        .is_none());
        assert!(try_place_part(
            &part,
            20,
            20,
            0,
            &HashSet::new(),
            |_, _, _, _| true,
            |_, _| None,
            |_, _| false,
        )
        .is_some());
    }
}
