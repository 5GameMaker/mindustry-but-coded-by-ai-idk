//! Pure placement helpers mirroring upstream `mindustry.input.Placement`.
//!
//! The Java class also owns global scratch buffers and pathfinding over
//! `Vars.world`.  This module keeps the deterministic geometry and plan
//! rewrites as explicit value-returning helpers so `DesktopInput` and
//! `MobileInput` can be ported on top without depending on global state.

use crate::mindustry::{
    entities::units::BuildPlan,
    io::{Point2 as PlacementPoint, TypeValue},
    vars::TILE_SIZE,
    world::tile::relative_to,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizeResult {
    pub x: i32,
    pub y: i32,
    pub x2: i32,
    pub y2: i32,
    pub rotation: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizeDrawResult {
    pub x: f32,
    pub y: f32,
    pub x2: f32,
    pub y2: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacementBlockDraw {
    pub size: i32,
    pub offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementPlanState {
    pub placeable: bool,
    pub same_block: bool,
    pub build_rotation: Option<i32>,
    pub avoid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgePlacementConfig {
    pub range: i32,
    pub has_junction: bool,
    pub unlocked: bool,
    pub rotated: bool,
    pub write_item_bridge_config: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Placement;

impl Placement {
    pub fn pathfind_line(
        conveyors: bool,
        conveyor_pathfinding: bool,
        astar_path: Option<Vec<PlacementPoint>>,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    ) -> Vec<PlacementPoint> {
        if conveyors && conveyor_pathfinding {
            astar_path.unwrap_or_else(|| normalize_line(start_x, start_y, end_x, end_y))
        } else {
            bresenham_line_no_diagonal(start_x, start_y, end_x, end_y)
        }
    }

    pub fn normalize_line(
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    ) -> Vec<PlacementPoint> {
        normalize_line(start_x, start_y, end_x, end_y)
    }

    pub fn normalize_rectangle(
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        block_size: i32,
    ) -> Vec<PlacementPoint> {
        normalize_rectangle(start_x, start_y, end_x, end_y, block_size)
    }

    pub fn calculate_nodes<F>(
        points: &[PlacementPoint],
        mut valid_place: F,
        mut overlapper: impl FnMut(PlacementPoint, PlacementPoint) -> bool,
    ) -> Vec<PlacementPoint>
    where
        F: FnMut(PlacementPoint) -> bool,
    {
        calculate_nodes(points, &mut valid_place, &mut overlapper)
    }

    pub fn is_side_place(plans: &[BuildPlan]) -> bool {
        is_side_place(plans)
    }

    pub fn calculate_bridge_plans(
        plans: &[BuildPlan],
        states: &[PlacementPlanState],
        bridge_block: &str,
        config: BridgePlacementConfig,
    ) -> Vec<BuildPlan> {
        calculate_bridge_plans(plans, states, bridge_block, config)
    }

    pub fn normalize_area(
        tile_x: i32,
        tile_y: i32,
        end_x: i32,
        end_y: i32,
        rotation: i32,
        snap: bool,
        max_length: i32,
    ) -> NormalizeResult {
        normalize_area(tile_x, tile_y, end_x, end_y, rotation, snap, max_length)
    }

    pub fn normalize_draw_area(
        block: PlacementBlockDraw,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        snap: bool,
        max_length: i32,
        scaling: f32,
    ) -> NormalizeDrawResult {
        normalize_draw_area(
            block, start_x, start_y, end_x, end_y, snap, max_length, scaling,
        )
    }
}

pub fn pathfind_line(
    conveyors: bool,
    conveyor_pathfinding: bool,
    astar_path: Option<Vec<PlacementPoint>>,
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
) -> Vec<PlacementPoint> {
    Placement::pathfind_line(
        conveyors,
        conveyor_pathfinding,
        astar_path,
        start_x,
        start_y,
        end_x,
        end_y,
    )
}

pub fn normalize_line(start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> Vec<PlacementPoint> {
    let mut points = Vec::new();
    if (start_x - end_x).abs() > (start_y - end_y).abs() {
        let sign = signum(end_x - start_x);
        for i in 0..=(start_x - end_x).abs() {
            points.push(PlacementPoint::new(start_x + i * sign, start_y));
        }
    } else {
        let sign = signum(end_y - start_y);
        for i in 0..=(start_y - end_y).abs() {
            points.push(PlacementPoint::new(start_x, start_y + i * sign));
        }
    }
    points
}

pub fn bresenham_line_no_diagonal(
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
) -> Vec<PlacementPoint> {
    let mut points = Vec::new();
    let mut x = start_x;
    let mut y = start_y;
    points.push(PlacementPoint::new(x, y));

    while x != end_x || y != end_y {
        let dx = (end_x - x).abs();
        let dy = (end_y - y).abs();
        if dx > dy {
            x += signum(end_x - x);
        } else {
            y += signum(end_y - y);
        }
        points.push(PlacementPoint::new(x, y));
    }

    points
}

pub fn normalize_rectangle(
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    block_size: i32,
) -> Vec<PlacementPoint> {
    let step = block_size.max(1);
    let min_x = start_x.min(end_x);
    let min_y = start_y.min(end_y);
    let max_x = start_x.max(end_x);
    let max_y = start_y.max(end_y);
    let x_sign = signum(end_x - start_x);
    let y_sign = signum(end_y - start_y);
    let mut points = Vec::new();

    let mut y = 0;
    while y <= max_y - min_y {
        let mut x = 0;
        while x <= max_x - min_x {
            points.push(PlacementPoint::new(
                start_x + x * x_sign,
                start_y + y * y_sign,
            ));
            x += step;
        }
        y += step;
    }

    points
}

pub fn calculate_nodes(
    points: &[PlacementPoint],
    valid_place: &mut impl FnMut(PlacementPoint) -> bool,
    overlapper: &mut impl FnMut(PlacementPoint, PlacementPoint) -> bool,
) -> Vec<PlacementPoint> {
    if points.is_empty() {
        return Vec::new();
    }

    let first = points[0];
    let last = *points.last().unwrap();
    let base: Vec<PlacementPoint> = points
        .iter()
        .copied()
        .filter(|point| *point == first || *point == last || valid_place(*point))
        .collect();

    let mut result = Vec::new();
    let mut added_last = false;
    let mut i = 0usize;

    'outer: while i < base.len() {
        let point = base[i];
        result.push(point);
        if i == base.len() - 1 {
            added_last = true;
        }

        for j in (i + 1..base.len()).rev() {
            let other = base[j];
            if overlapper(point, other) {
                i = j;
                continue 'outer;
            }
        }

        i += 1;
    }

    if !added_last && !base.is_empty() {
        result.push(*base.last().unwrap());
    }

    result
}

pub fn is_side_place(plans: &[BuildPlan]) -> bool {
    plans.len() > 1
        && (relative_to(plans[0].x, plans[0].y, plans[1].x, plans[1].y) as i32 - plans[0].rotation)
            .rem_euclid(2)
            == 1
}

pub fn calculate_bridge_plans(
    plans: &[BuildPlan],
    states: &[PlacementPlanState],
    bridge_block: &str,
    config: BridgePlacementConfig,
) -> Vec<BuildPlan> {
    if plans.is_empty()
        || states.len() != plans.len()
        || is_side_place(plans)
        || !config.unlocked
        || !(plans[0].x == plans[plans.len() - 1].x || plans[0].y == plans[plans.len() - 1].y)
    {
        return plans.to_vec();
    }

    let mut result = Vec::new();
    let mut i = 0usize;

    'outer: while i < plans.len() {
        let mut cur = plans[i].clone();
        result.push(cur.clone());

        if i < plans.len() - 1
            && bridge_placeable(i, plans, states)
            && !bridge_placeable(i + 1, plans, states)
        {
            let mut were_same = true;

            for j in i + 1..plans.len() {
                let other = &plans[j];

                if !positions_valid_line(cur.x, cur.y, other.x, other.y, config.range) {
                    for missed in plans.iter().take(j).skip(i + 1) {
                        result.push(missed.clone());
                    }
                    i = j;
                    continue 'outer;
                } else if bridge_placeable(j, plans, states) {
                    if were_same && config.has_junction {
                        i += 1;
                        continue 'outer;
                    } else {
                        cur.block = Some(bridge_block.to_string());
                        if config.write_item_bridge_config {
                            let point = if config.rotated {
                                PlacementPoint::new(cur.x - other.x, cur.y - other.y)
                            } else {
                                PlacementPoint::new(other.x - cur.x, other.y - cur.y)
                            };
                            if config.rotated {
                                let mut linked = other.clone();
                                linked.block = Some(bridge_block.to_string());
                                linked.config = TypeValue::Point2(point);
                                replace_last(&mut result, cur);
                                result.push(linked);
                            } else {
                                cur.config = TypeValue::Point2(point);
                                replace_last(&mut result, cur);
                                let mut linked = other.clone();
                                linked.block = Some(bridge_block.to_string());
                                result.push(linked);
                            }
                        } else {
                            replace_last(&mut result, cur);
                            let mut linked = other.clone();
                            linked.block = Some(bridge_block.to_string());
                            result.push(linked);
                        }
                        i = j + 1;
                        continue 'outer;
                    }
                }

                if !states[j].avoid {
                    were_same = false;
                }
            }

            for tail in plans.iter().skip(i + 1) {
                result.push(tail.clone());
            }
            break;
        } else {
            i += 1;
        }
    }

    result
}

pub fn normalize_draw_area(
    block: PlacementBlockDraw,
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    snap: bool,
    max_length: i32,
    scaling: f32,
) -> NormalizeDrawResult {
    let result = normalize_area(start_x, start_y, end_x, end_y, 0, snap, max_length);
    let tile = TILE_SIZE as f32;
    let half = block.size as f32 * scaling * tile / 2.0;

    NormalizeDrawResult {
        x: result.x as f32 * tile - half + block.offset,
        y: result.y as f32 * tile - half + block.offset,
        x2: result.x2 as f32 * tile + half + block.offset,
        y2: result.y2 as f32 * tile + half + block.offset,
    }
}

pub fn normalize_area(
    mut tile_x: i32,
    mut tile_y: i32,
    mut end_x: i32,
    mut end_y: i32,
    mut rotation: i32,
    snap: bool,
    max_length: i32,
) -> NormalizeResult {
    if snap {
        if (tile_x - end_x).abs() > (tile_y - end_y).abs() {
            end_y = tile_y;
        } else {
            end_x = tile_x;
        }
    }

    if max_length > 0 {
        if (end_x - tile_x).abs() > max_length {
            end_x = signum(end_x - tile_x) * max_length + tile_x;
        }

        if (end_y - tile_y).abs() > max_length {
            end_y = signum(end_y - tile_y) * max_length + tile_y;
        }
    }

    let dx = end_x - tile_x;
    let dy = end_y - tile_y;

    if dx.abs() > dy.abs() {
        rotation = if dx >= 0 { 0 } else { 2 };
    } else if dx.abs() < dy.abs() {
        rotation = if dy >= 0 { 1 } else { 3 };
    }

    if end_x < tile_x {
        std::mem::swap(&mut end_x, &mut tile_x);
    }
    if end_y < tile_y {
        std::mem::swap(&mut end_y, &mut tile_y);
    }

    NormalizeResult {
        x: tile_x,
        y: tile_y,
        x2: end_x,
        y2: end_y,
        rotation,
    }
}

pub fn distance_heuristic(x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {
    ((x1 - x2).abs() + (y1 - y2).abs()) as f32
}

fn bridge_placeable(index: usize, plans: &[BuildPlan], states: &[PlacementPlanState]) -> bool {
    let state = states[index];
    (state.placeable || state.same_block)
        && !(index != 0
            && state.build_rotation.is_some()
            && state.build_rotation != Some(plans[index].rotation)
            && state.avoid)
}

fn positions_valid_line(x1: i32, y1: i32, x2: i32, y2: i32, range: i32) -> bool {
    if x1 == x2 {
        (y1 - y2).abs() <= range
    } else if y1 == y2 {
        (x1 - x2).abs() <= range
    } else {
        false
    }
}

fn replace_last(plans: &mut Vec<BuildPlan>, plan: BuildPlan) {
    if let Some(last) = plans.last_mut() {
        *last = plan;
    }
}

fn signum(value: i32) -> i32 {
    value.signum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: i32, y: i32) -> PlacementPoint {
        PlacementPoint::new(x, y)
    }

    fn placeable() -> PlacementPlanState {
        PlacementPlanState {
            placeable: true,
            same_block: false,
            build_rotation: None,
            avoid: false,
        }
    }

    fn blocked() -> PlacementPlanState {
        PlacementPlanState {
            placeable: false,
            same_block: false,
            build_rotation: None,
            avoid: true,
        }
    }

    #[test]
    fn normalize_line_prefers_major_axis_like_java() {
        assert_eq!(
            normalize_line(0, 0, 3, 1),
            vec![point(0, 0), point(1, 0), point(2, 0), point(3, 0)]
        );
        assert_eq!(
            normalize_line(3, 3, 1, -1),
            vec![
                point(3, 3),
                point(3, 2),
                point(3, 1),
                point(3, 0),
                point(3, -1)
            ]
        );
        assert_eq!(normalize_line(2, 2, 2, 2), vec![point(2, 2)]);
    }

    #[test]
    fn pathfind_line_uses_astar_only_when_conveyors_setting_and_path_exist() {
        let astar = vec![point(0, 0), point(0, 1), point(1, 1)];

        assert_eq!(
            pathfind_line(true, true, Some(astar.clone()), 0, 0, 5, 0),
            astar
        );
        assert_eq!(
            pathfind_line(true, true, None, 0, 0, 2, 1),
            normalize_line(0, 0, 2, 1)
        );
        assert_eq!(
            pathfind_line(false, true, Some(vec![point(9, 9)]), 0, 0, 2, 1),
            bresenham_line_no_diagonal(0, 0, 2, 1)
        );
    }

    #[test]
    fn normalize_rectangle_preserves_java_iteration_order_and_direction() {
        assert_eq!(
            normalize_rectangle(0, 0, 2, 2, 1),
            vec![
                point(0, 0),
                point(1, 0),
                point(2, 0),
                point(0, 1),
                point(1, 1),
                point(2, 1),
                point(0, 2),
                point(1, 2),
                point(2, 2),
            ]
        );
        assert_eq!(
            normalize_rectangle(4, 4, 0, 0, 2),
            vec![
                point(4, 4),
                point(2, 4),
                point(0, 4),
                point(4, 2),
                point(2, 2),
                point(0, 2),
                point(4, 0),
                point(2, 0),
                point(0, 0),
            ]
        );
    }

    #[test]
    fn calculate_nodes_keeps_endpoints_and_furthest_overlap() {
        let points = vec![
            point(0, 0),
            point(1, 0),
            point(2, 0),
            point(3, 0),
            point(4, 0),
        ];
        let result = calculate_nodes(&points, &mut |point| point.x != 2, &mut |left, right| {
            right.x - left.x <= 2
        });

        assert_eq!(
            result,
            vec![point(0, 0), point(1, 0), point(3, 0), point(4, 0)]
        );
    }

    #[test]
    fn is_side_place_matches_relative_rotation_guard() {
        let plans = vec![
            BuildPlan::new_place(0, 0, 0, "conveyor"),
            BuildPlan::new_place(0, 1, 0, "conveyor"),
        ];
        assert!(is_side_place(&plans));

        let forward = vec![
            BuildPlan::new_place(0, 0, 0, "conveyor"),
            BuildPlan::new_place(1, 0, 0, "conveyor"),
        ];
        assert!(!is_side_place(&forward));
        assert!(!is_side_place(&forward[..1]));
    }

    #[test]
    fn calculate_bridge_plans_links_gap_and_writes_item_config() {
        let plans = vec![
            BuildPlan::new_place(0, 0, 0, "conveyor"),
            BuildPlan::new_place(1, 0, 0, "conveyor"),
            BuildPlan::new_place(2, 0, 0, "conveyor"),
            BuildPlan::new_place(3, 0, 0, "conveyor"),
        ];
        let states = vec![placeable(), blocked(), blocked(), placeable()];

        let result = calculate_bridge_plans(
            &plans,
            &states,
            "item-bridge",
            BridgePlacementConfig {
                range: 4,
                has_junction: false,
                unlocked: true,
                rotated: false,
                write_item_bridge_config: true,
            },
        );

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].block.as_deref(), Some("item-bridge"));
        assert_eq!(result[0].config, TypeValue::Point2(point(3, 0)));
        assert_eq!(result[1].block.as_deref(), Some("item-bridge"));
    }

    #[test]
    fn calculate_bridge_plans_preserves_missed_gap_when_out_of_range() {
        let plans = vec![
            BuildPlan::new_place(0, 0, 0, "conveyor"),
            BuildPlan::new_place(1, 0, 0, "conveyor"),
            BuildPlan::new_place(2, 0, 0, "conveyor"),
            BuildPlan::new_place(5, 0, 0, "conveyor"),
        ];
        let states = vec![placeable(), blocked(), blocked(), placeable()];

        let result = calculate_bridge_plans(
            &plans,
            &states,
            "item-bridge",
            BridgePlacementConfig {
                range: 2,
                has_junction: false,
                unlocked: true,
                rotated: false,
                write_item_bridge_config: true,
            },
        );

        assert_eq!(result, plans);
    }

    #[test]
    fn normalize_area_snaps_clamps_and_rotates_like_java() {
        assert_eq!(
            normalize_area(0, 0, 5, 3, 9, true, 0),
            NormalizeResult {
                x: 0,
                y: 0,
                x2: 5,
                y2: 0,
                rotation: 0,
            }
        );
        assert_eq!(
            normalize_area(5, 5, 1, 12, 0, false, 3),
            NormalizeResult {
                x: 2,
                y: 5,
                x2: 5,
                y2: 8,
                rotation: 0,
            }
        );
        assert_eq!(
            normalize_area(5, 5, 5, 1, 0, false, 0),
            NormalizeResult {
                x: 5,
                y: 1,
                x2: 5,
                y2: 5,
                rotation: 3,
            }
        );
    }

    #[test]
    fn normalize_draw_area_applies_tile_size_block_size_scaling_and_offset() {
        let result = normalize_draw_area(
            PlacementBlockDraw {
                size: 2,
                offset: 4.0,
            },
            1,
            2,
            3,
            4,
            false,
            0,
            1.5,
        );

        assert_eq!(
            result,
            NormalizeDrawResult {
                x: 0.0,
                y: 8.0,
                x2: 40.0,
                y2: 48.0,
            }
        );
    }

    #[test]
    fn distance_heuristic_is_manhattan_distance_like_java() {
        assert_eq!(distance_heuristic(1, 2, -3, 7), 9.0);
    }
}
