use std::collections::HashSet;

use crate::mindustry::{
    ai::base_registry::{BasePart, BasePartTile, BasePartTileKind},
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
