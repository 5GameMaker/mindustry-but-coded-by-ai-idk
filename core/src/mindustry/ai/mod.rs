// Mirrors upstream core/src/mindustry/ai. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.
pub mod astar;
pub mod base_builder_ai;
pub mod base_registry;
pub mod block_indexer;
pub mod control_pathfinder;
pub mod item_unit_stance;
pub mod pathfind_queue;
pub mod pathfinder;
pub mod rts_ai;
pub mod unit_command;
pub mod unit_group;
pub mod unit_stance;
pub mod wave_spawner;

pub use astar::{manhattan, pathfind_grid, pathfind_grid_manhattan, D4};
pub use base_builder_ai::{
    begin_path_refresh, builder_ai_enemy_search_rect, builder_ai_fallback_controller,
    builder_ai_idle_retreat, builder_ai_init_rebuild_period, builder_ai_near_enemy,
    builder_ai_should_fire, builder_ai_should_promote_assist_following, builder_ai_should_shoot,
    builder_ai_use_fallback, choose_part_pool, claim_builder_ai_hold_plan,
    claim_builder_ai_rebuild_plan, place_interval, prebuild_ai_all_requirements_have_ore,
    prebuild_ai_can_build, prebuild_ai_category_priority, prebuild_ai_find_next_plan,
    prebuild_ai_handle_full_core_for_target_item, prebuild_ai_handle_return_to_core_with_items,
    prebuild_ai_look_action, prebuild_ai_pick_missing_collect_target_item,
    prebuild_ai_plan_reachable_from_tree, prebuild_ai_refresh_mining_ore_target,
    prebuild_ai_should_boost, prebuild_ai_should_stop_mining_for_carry_limit_or_acceptance,
    prebuild_ai_sort_plans, prebuild_ai_tree_query, prebuild_ai_update_building,
    prebuild_ai_update_movement_tick, random_position_source, rotate_build_tile, rotate_center,
    should_spawn_core_unit, step_core_path, sync_builder_ai_follow_plan, try_place_part,
    validate_builder_ai_current_plan, validate_prebuild_ai_current_plan, BaseBuildPart,
    BaseBuildTile, BaseBuilderPathState, BlockPlan, BuilderAiEnemySearchRect,
    BuilderAiFallbackController, BuilderAiFollowAction, BuilderAiFollowSync, BuilderAiPlanAction,
    BuilderAiPlanValidation, BuilderAiRetreatDecision, PartPoolChoice, PathCalculationOutcome,
    PrebuildAiAcceptPlanAction, PrebuildAiBlockInfo, PrebuildAiBuildMove, PrebuildAiFullCoreAction,
    PrebuildAiLookAction, PrebuildAiMiningTarget, PrebuildAiOreTarget, PrebuildAiPlanSnapshot,
    PrebuildAiRequirement, PrebuildAiReturnToCoreAction, PrebuildAiTickBranch,
    PrebuildAiTickDecision, PrebuildAiTreeQuery, SeedPositionSource, TilePoint, ATTEMPTS,
    BUILDER_AI_BUILD_AI_REBUILD_PERIOD, BUILDER_AI_DEFAULT_REBUILD_PERIOD, CORE_UNIT_MULTIPLIER,
    EMPTY_CHANCE, PATH_STEP as BASE_BUILDER_PATH_STEP, PLACE_INTERVAL_MAX, PLACE_INTERVAL_MIN,
    PREBUILD_AI_CRAFTING_PRIORITY, PREBUILD_AI_DISTRIBUTION_PRIORITY, PREBUILD_AI_LIQUID_PRIORITY,
    PREBUILD_AI_PRIORITY_DST_SCALE, PREBUILD_AI_PRODUCTION_PRIORITY, TIMER_REFRESH_PATH,
    TIMER_SPAWN, TIMER_STEP,
};
pub use base_registry::{BasePart, BasePartTile, BasePartTileKind, BaseRegistry};
pub use block_indexer::{
    quadrant_dimensions, quadrant_for_tile, BlockIndexer, IndexedTile, QUADRANT_SIZE,
};
pub use control_pathfinder::{
    avoid_cost, cluster_astar, control_ground_cost, control_hover_cost, control_legs_cost,
    control_naval_cost, control_path_cost, inner_astar, make_node_index, near_passable_cost,
    passable_cost, portal_position, raycast_fast, raycast_fast_avoid, raycast_rect,
    rebuild_inner_edges, scan_cluster_portals, solid_cost, Cluster, FieldIndex, GridPoint,
    IntraEdge, NodeIndex, PortalRange, CLUSTER_SIZE, COST_ID_GROUND, COST_ID_HOVER, COST_ID_LEGS,
    COST_ID_NAVAL, INVALIDATE_CHECK_INTERVAL_MS, SOLID_CAP, UPDATE_FPS, UPDATE_INTERVAL_MS,
    UPDATE_STEP_INTERVAL, WALL_IMPASSABLE_CAP,
};
pub use item_unit_stance::{ItemUnitStance, ItemUnitStanceRegistry};
pub use pathfind_queue::PathfindQueue;
pub use pathfinder::{
    ground_cost, hover_cost, legs_cost, naval_cost, neoplasm_cost, none_cost, path_cost, Flowfield,
    PathTile, BIT_ALL_DEEP, BIT_DAMAGES, BIT_DEEP, BIT_LEG_SOLID, BIT_LIQUID, BIT_NEAR_DEEP,
    BIT_NEAR_GROUND, BIT_NEAR_LEG_SOLID, BIT_NEAR_LIQUID, BIT_NEAR_SOLID, BIT_SOLID,
    BIT_TEAM_PASSABLE, COST_GROUND, COST_HOVER, COST_LEGS, COST_NAVAL, COST_NEOPLASM, COST_NONE,
    FIELD_CORE, IMPASSABLE, MAX_COSTS, MAX_FIELDS,
};
pub use rts_ai::{
    battle_yield, candidate_score, compare_candidates, point_segment_distance,
    select_target_candidate, RtsAiConfig, RtsRulesSnapshot, SquadMemberSnapshot, SquadSummary,
    TargetCandidate, TargetPriority, DEFAULT_TARGET_PRIORITIES, RTS_BATTLE_EPSILON,
    RTS_BATTLE_YIELD_IMPOSSIBLE, RTS_DEFEND_CHECK_RANGE, RTS_DEFEND_WITHIN_RANGE,
    RTS_MAX_TARGETS_CHECKED, RTS_SQUAD_RADIUS, RTS_TIMER_SPAWN, RTS_TIME_UPDATE,
    RTS_TURRET_SCAN_PADDING,
};
pub use unit_group::{
    calculate_relative_positions, update_raycast_position, UnitGroup, UnitGroupMember,
    LAYER_FLYING, LAYER_GROUND, LAYER_LEGS, LAYER_UNDERWATER, PHYSICS_LAYERS,
};
pub use wave_spawner::{
    count_flyer_spawns, count_ground_spawns, flyer_edge_spawn, flyer_spawns, ground_spawns,
    is_spawning, player_near, spawn_effect_plan, spawn_rotation, FlyerSpawn, GroundSpawn,
    SpawnEffectPlan, SpawnTile, CORE_MARGIN, MAX_CORE_SPAWN_STEPS, SPAWN_MARGIN,
};
