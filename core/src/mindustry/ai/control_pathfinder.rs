use std::collections::HashMap;

use crate::mindustry::ai::{
    pathfind_queue::PathfindQueue,
    pathfinder::{PathTile, IMPASSABLE},
};

pub const WALL_IMPASSABLE_CAP: i32 = 1_000_000;
pub const SOLID_CAP: i32 = 7000;
pub const CLUSTER_SIZE: i32 = 12;

pub const COST_ID_GROUND: i32 = 0;
pub const COST_ID_HOVER: i32 = 1;
pub const COST_ID_LEGS: i32 = 2;
pub const COST_ID_NAVAL: i32 = 3;

pub const UPDATE_STEP_INTERVAL: i32 = 200;
pub const UPDATE_FPS: i32 = 30;
pub const UPDATE_INTERVAL_MS: i32 = 1000 / UPDATE_FPS;
pub const INVALIDATE_CHECK_INTERVAL_MS: i32 = 1000;

const D4: [GridPoint; 4] = [
    GridPoint { x: 1, y: 0 },
    GridPoint { x: 0, y: 1 },
    GridPoint { x: -1, y: 0 },
    GridPoint { x: 0, y: -1 },
];

const OFFSETS: [i32; 8] = [
    1, 0, // right: bottom to top
    0, 1, // top: left to right
    0, 0, // left: bottom to top
    0, 0, // bottom: left to right
];

const MOVE_DIRS: [i32; 8] = [
    0, 1, // right
    1, 0, // top
    0, 1, // left
    1, 0, // bottom
];

pub const NEXT_OFFSETS: [i32; 8] = [
    1, 0, // right
    0, 1, // top
    -1, 0, // left
    0, -1, // bottom
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPoint {
    pub x: i32,
    pub y: i32,
}

impl GridPoint {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn array(self, width: i32) -> i32 {
        self.x + self.y * width
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalRange {
    pub from: i32,
    pub to: i32,
}

impl PortalRange {
    pub const fn new(from: i32, to: i32) -> Self {
        Self { from, to }
    }

    pub fn midpoint(self) -> i32 {
        (self.from + self.to) / 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntraEdge {
    pub dir: u8,
    pub portal: u8,
    pub cost: f32,
}

impl IntraEdge {
    pub const fn new(dir: u8, portal: u8, cost: f32) -> Self {
        Self { dir, portal, cost }
    }

    pub fn pack(self) -> u64 {
        ((self.dir as u64) & 0xff)
            | (((self.portal as u64) & 0xff) << 8)
            | ((self.cost.to_bits() as u64) << 16)
    }

    pub fn unpack(bits: u64) -> Self {
        Self {
            dir: (bits & 0xff) as u8,
            portal: ((bits >> 8) & 0xff) as u8,
            cost: f32::from_bits(((bits >> 16) & 0xffff_ffff) as u32),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeIndex {
    pub cluster: u32,
    pub dir: u8,
    pub portal: u8,
}

impl NodeIndex {
    pub const CLUSTER_MASK: u32 = 0x003f_ffff;
    pub const DIR_MASK: u32 = 0x00c0_0000;
    pub const PORTAL_MASK: u32 = 0xff00_0000;

    pub const fn new(cluster: u32, dir: u8, portal: u8) -> Self {
        Self {
            cluster,
            dir,
            portal,
        }
    }

    pub fn pack(self) -> i32 {
        (((self.cluster & Self::CLUSTER_MASK) << 0)
            | (((self.dir as u32) & 0x03) << 22)
            | (((self.portal as u32) & 0xff) << 24)) as i32
    }

    pub fn unpack(bits: i32) -> Self {
        let bits = bits as u32;
        Self {
            cluster: bits & Self::CLUSTER_MASK,
            dir: ((bits & Self::DIR_MASK) >> 22) as u8,
            portal: ((bits & Self::PORTAL_MASK) >> 24) as u8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldIndex {
    pub pos: u32,
    pub cost_id: u8,
    pub team: u8,
}

impl FieldIndex {
    pub const fn new(pos: u32, cost_id: u8, team: u8) -> Self {
        Self { pos, cost_id, team }
    }

    pub fn pack(self) -> u64 {
        self.pos as u64 | ((self.cost_id as u64) << 32) | ((self.team as u64) << 40)
    }

    pub fn unpack(bits: u64) -> Self {
        Self {
            pos: (bits & 0xffff_ffff) as u32,
            cost_id: ((bits >> 32) & 0xff) as u8,
            team: ((bits >> 40) & 0xff) as u8,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cluster {
    pub portals: [Vec<PortalRange>; 4],
    pub portal_connections: [Vec<Vec<IntraEdge>>; 4],
}

impl Cluster {
    pub fn new() -> Self {
        Self {
            portals: std::array::from_fn(|_| Vec::new()),
            portal_connections: std::array::from_fn(|_| Vec::new()),
        }
    }

    pub fn clear_connections(&mut self) {
        self.portal_connections = std::array::from_fn(|_| Vec::new());
    }
}

impl Default for Cluster {
    fn default() -> Self {
        Self::new()
    }
}

pub fn control_path_cost(cost_id: i32, team: u8, tile: i32) -> i32 {
    match cost_id {
        COST_ID_GROUND => control_ground_cost(team, tile),
        COST_ID_HOVER => control_hover_cost(team, tile),
        COST_ID_LEGS => control_legs_cost(team, tile),
        COST_ID_NAVAL => control_naval_cost(team, tile),
        _ => control_ground_cost(team, tile),
    }
}

pub fn control_ground_cost(team: u8, tile: i32) -> i32 {
    let tile = PathTile::unpack(tile);
    if tile.all_deep
        || (tile.solid && ((tile.team == team && !tile.team_passable) || tile.team == 0))
    {
        IMPASSABLE
    } else {
        enemy_wall_penalty(team, tile)
            + 1
            + if tile.near_solid { 6 } else { 0 }
            + if tile.near_liquid { 8 } else { 0 }
            + if tile.deep { 6000 } else { 0 }
            + if tile.damages { 50 } else { 0 }
    }
}

pub fn control_hover_cost(team: u8, tile: i32) -> i32 {
    let tile = PathTile::unpack(tile);
    if tile.solid && ((tile.team == team && !tile.team_passable) || tile.team == 0) {
        IMPASSABLE
    } else {
        enemy_wall_penalty(team, tile) + 1 + if tile.near_solid { 6 } else { 0 }
    }
}

pub fn control_legs_cost(_team: u8, tile: i32) -> i32 {
    let tile = PathTile::unpack(tile);
    if tile.leg_solid {
        IMPASSABLE
    } else {
        1 + if tile.near_deep { 8 } else { 0 }
            + if tile.deep { 6000 } else { 0 }
            + if tile.near_leg_solid { 3 } else { 0 }
    }
}

pub fn control_naval_cost(team: u8, tile: i32) -> i32 {
    let tile = PathTile::unpack(tile);
    if tile.solid && ((tile.team == team && !tile.team_passable) || tile.team == 0) {
        IMPASSABLE
    } else {
        (if !tile.liquid { 6000 } else { 1 })
            + enemy_wall_penalty(team, tile)
            + if tile.near_ground || tile.near_solid {
                6
            } else {
                0
            }
    }
}

pub fn passable_cost(cost: i32) -> bool {
    cost != IMPASSABLE && cost < SOLID_CAP
}

pub fn near_passable_cost(initial_cost: i32, cost: i32) -> bool {
    cost != IMPASSABLE && cost < (initial_cost + 1).max(50).min(SOLID_CAP)
}

pub fn solid_cost(cost: i32) -> bool {
    cost == IMPASSABLE || cost >= SOLID_CAP
}

pub fn avoid_cost(cost: i32) -> bool {
    cost == IMPASSABLE || cost >= 2
}

pub fn make_node_index(cx: i32, cy: i32, dir: i32, portal: i32, cluster_width: i32) -> i32 {
    let mut cx = cx;
    let mut cy = cy;
    let mut dir = dir;

    if dir == 2 && cx != 0 {
        dir = 0;
        cx -= 1;
    }

    if dir == 3 && cy != 0 {
        dir = 1;
        cy -= 1;
    }

    NodeIndex::new((cx + cy * cluster_width) as u32, dir as u8, portal as u8).pack()
}

pub fn scan_cluster_portals<F>(
    cx: i32,
    cy: i32,
    cluster_width: i32,
    cluster_height: i32,
    mut solid: F,
) -> Cluster
where
    F: FnMut(i32, i32) -> bool,
{
    let mut cluster = Cluster::new();

    for direction in 0..4usize {
        let other_x = cx + D4[direction].x;
        let other_y = cy + D4[direction].y;
        if other_x < 0 || other_y < 0 || other_x >= cluster_width || other_y >= cluster_height {
            continue;
        }

        let add_x = MOVE_DIRS[direction * 2];
        let add_y = MOVE_DIRS[direction * 2 + 1];
        let base_x = cx * CLUSTER_SIZE + OFFSETS[direction * 2] * (CLUSTER_SIZE - 1);
        let base_y = cy * CLUSTER_SIZE + OFFSETS[direction * 2 + 1] * (CLUSTER_SIZE - 1);
        let next_base_x = base_x + D4[direction].x;
        let next_base_y = base_y + D4[direction].y;

        let mut last_portal = -1;
        let mut prev_solid = true;

        for i in 0..CLUSTER_SIZE {
            let x = base_x + add_x * i;
            let y = base_y + add_y * i;
            let blocked = solid(x, y) || solid(next_base_x + add_x * i, next_base_y + add_y * i);

            if blocked {
                let previous = i - 1;
                if !prev_solid && previous >= last_portal {
                    cluster.portals[direction].push(PortalRange::new(last_portal, previous));
                }
                prev_solid = true;
            } else {
                if prev_solid {
                    last_portal = i;
                }
                prev_solid = false;
            }
        }

        let previous = CLUSTER_SIZE - 1;
        if !prev_solid && previous >= last_portal {
            cluster.portals[direction].push(PortalRange::new(last_portal, previous));
        }
    }

    cluster
}

pub fn portal_position(cx: i32, cy: i32, direction: usize, portal: PortalRange) -> GridPoint {
    let average = portal.midpoint();
    let ox = cx * CLUSTER_SIZE + OFFSETS[direction * 2] * (CLUSTER_SIZE - 1);
    let oy = cy * CLUSTER_SIZE + OFFSETS[direction * 2 + 1] * (CLUSTER_SIZE - 1);
    GridPoint::new(
        MOVE_DIRS[direction * 2] * average + ox,
        MOVE_DIRS[direction * 2 + 1] * average + oy,
    )
}

pub fn rebuild_inner_edges<F>(
    cluster: &mut Cluster,
    cx: i32,
    cy: i32,
    world_width: i32,
    world_height: i32,
    cost_at: F,
) where
    F: FnMut(i32) -> i32 + Copy,
{
    let min_x = cx * CLUSTER_SIZE;
    let min_y = cy * CLUSTER_SIZE;
    let max_x = (min_x + CLUSTER_SIZE - 1).min(world_width - 1);
    let max_y = (min_y + CLUSTER_SIZE - 1).min(world_height - 1);

    cluster.portal_connections =
        std::array::from_fn(|direction| vec![Vec::new(); cluster.portals[direction].len()]);

    for direction in 0..4usize {
        for i in 0..cluster.portals[direction].len() {
            let start = portal_position(cx, cy, direction, cluster.portals[direction][i]);

            for other_dir in 0..4usize {
                for j in 0..cluster.portals[other_dir].len() {
                    if other_dir < direction || (other_dir == direction && j <= i) {
                        continue;
                    }

                    let target_portal = cluster.portals[other_dir][j];
                    let target = portal_position(cx, cy, other_dir, target_portal);
                    if start == target {
                        continue;
                    }

                    let goal_start = portal_endpoint(cx, cy, other_dir, target_portal.from);
                    let goal_end = portal_endpoint(cx, cy, other_dir, target_portal.to);
                    let connection_cost = inner_astar(
                        world_width,
                        min_x,
                        min_y,
                        max_x,
                        max_y,
                        start.array(world_width),
                        target.array(world_width),
                        goal_start,
                        goal_end,
                        cost_at,
                    );

                    if let Some(cost) = connection_cost {
                        cluster.portal_connections[direction][i].push(IntraEdge::new(
                            other_dir as u8,
                            j as u8,
                            cost,
                        ));
                        cluster.portal_connections[other_dir][j].push(IntraEdge::new(
                            direction as u8,
                            i as u8,
                            cost,
                        ));
                    }
                }
            }
        }
    }
}

pub fn inner_astar<F>(
    world_width: i32,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
    start_pos: i32,
    goal_pos: i32,
    goal_a: GridPoint,
    goal_b: GridPoint,
    mut cost_at: F,
) -> Option<f32>
where
    F: FnMut(i32) -> i32,
{
    let mut frontier = PathfindQueue::new();
    let mut costs = HashMap::<i32, f32>::new();
    let (goal_x1, goal_x2) = ordered(goal_a.x, goal_b.x);
    let (goal_y1, goal_y2) = ordered(goal_a.y, goal_b.y);

    costs.insert(start_pos, 0.0);
    frontier.add(start_pos, 0.0);

    while !frontier.empty() {
        let current = frontier.poll();
        let cx = current % world_width;
        let cy = current / world_width;

        if (cx >= goal_x1 && cy >= goal_y1 && cx <= goal_x2 && cy <= goal_y2) || current == goal_pos
        {
            return costs.get(&current).copied();
        }

        for point in D4 {
            let new_x = cx + point.x;
            let new_y = cy + point.y;
            let next = new_x + world_width * new_y;

            if new_x > max_x
                || new_y > max_y
                || new_x < min_x
                || new_y < min_y
                || cost_at(next) == IMPASSABLE
            {
                continue;
            }

            let add = cost_at(next);
            if add < 0 {
                continue;
            }

            let new_cost = costs[&current] + add as f32;
            if new_cost < costs.get(&next).copied().unwrap_or(f32::INFINITY) {
                costs.insert(next, new_cost);
                frontier.add(
                    next,
                    new_cost + manhattan_index(next, goal_pos, world_width),
                );
            }
        }
    }

    None
}

pub fn cluster_astar<FEdges, FHeuristic>(
    start_node: i32,
    end_node: i32,
    mut edges: FEdges,
    mut heuristic: FHeuristic,
) -> Option<Vec<i32>>
where
    FEdges: FnMut(i32) -> Vec<(i32, f32)>,
    FHeuristic: FnMut(i32, i32) -> f32,
{
    if start_node == end_node {
        return Some(vec![start_node]);
    }

    let mut frontier = PathfindQueue::new();
    let mut costs = HashMap::<i32, f32>::new();
    let mut came_from = HashMap::<i32, i32>::new();

    came_from.insert(start_node, start_node);
    costs.insert(start_node, 0.0);
    frontier.add(start_node, 0.0);

    while !frontier.empty() {
        let current = frontier.poll();
        if current == end_node {
            let mut result = Vec::new();
            let mut cur = end_node;
            while cur != start_node {
                result.push(cur);
                cur = came_from[&cur];
            }
            result.reverse();
            return Some(result);
        }

        for (next, edge_cost) in edges(current) {
            let new_cost = costs[&current] + edge_cost;
            if new_cost < costs.get(&next).copied().unwrap_or(f32::INFINITY) {
                costs.insert(next, new_cost);
                frontier.add(next, new_cost + heuristic(next, end_node));
                came_from.insert(next, current);
            }
        }
    }

    None
}

pub fn raycast_fast<F>(
    world_width: i32,
    world_height: i32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    mut blocked: F,
) -> Option<GridPoint>
where
    F: FnMut(i32, i32) -> bool,
{
    raycast_no_diagonal(world_width, world_height, x1, y1, x2, y2, |x, y| {
        blocked(x, y)
    })
}

pub fn raycast_fast_avoid<F>(
    world_width: i32,
    world_height: i32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    mut avoid: F,
) -> Option<GridPoint>
where
    F: FnMut(i32, i32) -> bool,
{
    raycast_no_diagonal(world_width, world_height, x1, y1, x2, y2, |x, y| {
        avoid(x, y)
    })
}

pub fn raycast_rect<F>(
    world_width: i32,
    world_height: i32,
    start: (f32, f32),
    end: (f32, f32),
    tile_start: GridPoint,
    tile_end: GridPoint,
    rect_size: f32,
    mut near_passable: F,
) -> bool
where
    F: FnMut(i32, i32) -> bool,
{
    let mut x = tile_start.x;
    let dx = (tile_end.x - x).abs();
    let sx = if x < tile_end.x { 1 } else { -1 };
    let mut y = tile_start.y;
    let dy = (tile_end.y - y).abs();
    let sy = if y < tile_end.y { 1 } else { -1 };
    let mut err = dx - dy;

    while x >= 0 && y >= 0 && x < world_width && y < world_height {
        if !near_passable(x, y)
            || overlap(
                world_width,
                world_height,
                x + 1,
                y,
                start,
                end,
                rect_size,
                &mut near_passable,
            )
            || overlap(
                world_width,
                world_height,
                x - 1,
                y,
                start,
                end,
                rect_size,
                &mut near_passable,
            )
            || overlap(
                world_width,
                world_height,
                x,
                y + 1,
                start,
                end,
                rect_size,
                &mut near_passable,
            )
            || overlap(
                world_width,
                world_height,
                x,
                y - 1,
                start,
                end,
                rect_size,
                &mut near_passable,
            )
        {
            return true;
        }

        if x == tile_end.x && y == tile_end.y {
            return false;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }

    true
}

fn enemy_wall_penalty(team: u8, tile: PathTile) -> i32 {
    if tile.team != team && tile.team != 0 && tile.solid {
        WALL_IMPASSABLE_CAP
    } else {
        0
    }
}

fn portal_endpoint(cx: i32, cy: i32, direction: usize, portal_index: i32) -> GridPoint {
    let ox = cx * CLUSTER_SIZE + OFFSETS[direction * 2] * (CLUSTER_SIZE - 1);
    let oy = cy * CLUSTER_SIZE + OFFSETS[direction * 2 + 1] * (CLUSTER_SIZE - 1);
    GridPoint::new(
        MOVE_DIRS[direction * 2] * portal_index + ox,
        MOVE_DIRS[direction * 2 + 1] * portal_index + oy,
    )
}

fn ordered(a: i32, b: i32) -> (i32, i32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn manhattan_index(a: i32, b: i32, world_width: i32) -> f32 {
    let ax = a % world_width;
    let ay = a / world_width;
    let bx = b % world_width;
    let by = b / world_width;
    ((ax - bx).abs() + (ay - by).abs()) as f32
}

fn raycast_no_diagonal<F>(
    world_width: i32,
    world_height: i32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    mut blocked: F,
) -> Option<GridPoint>
where
    F: FnMut(i32, i32) -> bool,
{
    let mut x = x1;
    let dx = (x2 - x).abs();
    let sx = if x < x2 { 1 } else { -1 };
    let mut y = y1;
    let dy = (y2 - y).abs();
    let sy = if y < y2 { 1 } else { -1 };
    let mut err = dx - dy;

    while x >= 0 && y >= 0 && x < world_width && y < world_height {
        if blocked(x, y) {
            return Some(GridPoint::new(x, y));
        }
        if x == x2 && y == y2 {
            return None;
        }

        if 2 * err + dy > dx - 2 * err {
            err -= dy;
            x += sx;
        } else {
            err += dx;
            y += sy;
        }
    }

    None
}

fn overlap<F>(
    world_width: i32,
    world_height: i32,
    x: i32,
    y: i32,
    start: (f32, f32),
    end: (f32, f32),
    rect_size: f32,
    near_passable: &mut F,
) -> bool
where
    F: FnMut(i32, i32) -> bool,
{
    if x < 0 || y < 0 || x >= world_width || y >= world_height || near_passable(x, y) {
        return false;
    }

    let half = rect_size / 2.0;
    segment_intersects_rect(
        start,
        end,
        (x as f32 - half, y as f32 - half, rect_size, rect_size),
    )
}

fn segment_intersects_rect(start: (f32, f32), end: (f32, f32), rect: (f32, f32, f32, f32)) -> bool {
    let (rx, ry, rw, rh) = rect;
    let min_x = rx;
    let max_x = rx + rw;
    let min_y = ry;
    let max_y = ry + rh;

    point_in_rect(start, min_x, min_y, max_x, max_y)
        || point_in_rect(end, min_x, min_y, max_x, max_y)
        || segments_intersect(start, end, (min_x, min_y), (max_x, min_y))
        || segments_intersect(start, end, (max_x, min_y), (max_x, max_y))
        || segments_intersect(start, end, (max_x, max_y), (min_x, max_y))
        || segments_intersect(start, end, (min_x, max_y), (min_x, min_y))
}

fn point_in_rect(point: (f32, f32), min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> bool {
    point.0 >= min_x && point.0 <= max_x && point.1 >= min_y && point.1 <= max_y
}

fn segments_intersect(a: (f32, f32), b: (f32, f32), c: (f32, f32), d: (f32, f32)) -> bool {
    let o1 = orientation(a, b, c);
    let o2 = orientation(a, b, d);
    let o3 = orientation(c, d, a);
    let o4 = orientation(c, d, b);

    (o1 * o2 <= 0.0) && (o3 * o4 <= 0.0)
}

fn orientation(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_struct_packing_matches_java_bit_layout() {
        let node = NodeIndex::new(0x003f_fffe, 2, 17);
        let packed = node.pack();
        assert_eq!(NodeIndex::unpack(packed), node);
        assert_eq!(NodeIndex::unpack(packed).cluster, 0x003f_fffe);

        let field = FieldIndex::new(0xfeed_beef, 3, 42);
        assert_eq!(FieldIndex::unpack(field.pack()), field);

        let edge = IntraEdge::new(1, 7, 42.5);
        assert_eq!(IntraEdge::unpack(edge.pack()), edge);
    }

    #[test]
    fn control_costs_match_control_pathfinder_thresholds() {
        let same_solid = PathTile {
            team: 1,
            solid: true,
            ..Default::default()
        }
        .pack();
        assert_eq!(control_ground_cost(1, same_solid), IMPASSABLE);

        let enemy_solid = PathTile {
            team: 2,
            solid: true,
            near_solid: true,
            ..Default::default()
        }
        .pack();
        assert_eq!(
            control_hover_cost(1, enemy_solid),
            WALL_IMPASSABLE_CAP + 1 + 6
        );
        assert!(solid_cost(SOLID_CAP));
        assert!(passable_cost(SOLID_CAP - 1));
        assert!(avoid_cost(2));
        assert!(near_passable_cost(80, 80));
        assert!(!near_passable_cost(80, 81));
    }

    #[test]
    fn scan_cluster_portals_groups_contiguous_open_edges() {
        let cluster = scan_cluster_portals(0, 0, 2, 1, |x, y| {
            // right edge between x=11 and x=12: block rows 0..1 and 5, leave 2..4 and 6..11 open.
            (x == 11 || x == 12) && (y <= 1 || y == 5)
        });

        assert_eq!(
            cluster.portals[0],
            vec![PortalRange::new(2, 4), PortalRange::new(6, 11)]
        );
        assert!(cluster.portals[1].is_empty());
        assert!(cluster.portals[2].is_empty());
        assert!(cluster.portals[3].is_empty());
    }

    #[test]
    fn inner_astar_stays_inside_bounds_and_returns_weighted_cost() {
        let result = inner_astar(
            5,
            0,
            0,
            4,
            4,
            GridPoint::new(0, 0).array(5),
            GridPoint::new(4, 0).array(5),
            GridPoint::new(4, 0),
            GridPoint::new(4, 0),
            |pos| {
                let x = pos % 5;
                let y = pos / 5;
                if x == 2 && y == 0 {
                    IMPASSABLE
                } else {
                    1
                }
            },
        );

        assert_eq!(result, Some(6.0));
    }

    #[test]
    fn rebuild_inner_edges_connects_portals_with_astar_cost() {
        let mut cluster = Cluster::new();
        cluster.portals[0].push(PortalRange::new(3, 3));
        cluster.portals[1].push(PortalRange::new(3, 3));

        rebuild_inner_edges(&mut cluster, 0, 0, 24, 24, |_| 1);

        assert_eq!(cluster.portal_connections[0][0].len(), 1);
        assert_eq!(cluster.portal_connections[0][0][0].dir, 1);
        assert_eq!(cluster.portal_connections[1][0][0].dir, 0);
        assert!(cluster.portal_connections[0][0][0].cost > 0.0);
    }

    #[test]
    fn node_canonicalization_mirrors_left_and_bottom_edges() {
        let left = make_node_index(2, 1, 2, 5, 10);
        assert_eq!(NodeIndex::unpack(left), NodeIndex::new(11, 0, 5));

        let bottom = make_node_index(2, 1, 3, 7, 10);
        assert_eq!(NodeIndex::unpack(bottom), NodeIndex::new(2, 1, 7));

        let map_left = make_node_index(0, 1, 2, 9, 10);
        assert_eq!(NodeIndex::unpack(map_left), NodeIndex::new(10, 2, 9));
    }

    #[test]
    fn cluster_astar_returns_path_without_start_node_like_java_result() {
        let path = cluster_astar(
            1,
            4,
            |node| match node {
                1 => vec![(2, 1.0), (3, 10.0)],
                2 => vec![(4, 1.0)],
                3 => vec![(4, 1.0)],
                _ => Vec::new(),
            },
            |_, _| 0.0,
        );

        assert_eq!(path, Some(vec![2, 4]));
    }

    #[test]
    fn raycast_helpers_match_control_pathfinder_hit_shapes() {
        let hit = raycast_fast(8, 8, 0, 0, 5, 0, |x, y| x == 3 && y == 0);
        assert_eq!(hit, Some(GridPoint::new(3, 0)));
        assert_eq!(raycast_fast(8, 8, 0, 0, 2, 0, |_, _| false), None);

        let blocked = raycast_rect(
            8,
            8,
            (0.0, 0.0),
            (5.0, 0.0),
            GridPoint::new(0, 0),
            GridPoint::new(5, 0),
            1.0,
            |x, y| !(x == 3 && y == 0),
        );
        assert!(blocked);

        let clear = raycast_rect(
            8,
            8,
            (0.0, 0.0),
            (2.0, 0.0),
            GridPoint::new(0, 0),
            GridPoint::new(2, 0),
            1.0,
            |_, _| true,
        );
        assert!(!clear);
    }
}
