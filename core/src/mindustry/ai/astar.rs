use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub const D4: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct QueueNode {
    pos: usize,
    priority_bits: u32,
}

impl QueueNode {
    fn new(pos: usize, priority: f32) -> Self {
        Self {
            pos,
            priority_bits: priority.to_bits(),
        }
    }

    fn priority(self) -> f32 {
        f32::from_bits(self.priority_bits)
    }
}

impl Ord for QueueNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .priority()
            .total_cmp(&self.priority())
            .then_with(|| other.pos.cmp(&self.pos))
    }
}

impl PartialOrd for QueueNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn manhattan(x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {
    (x1 - x2).abs() as f32 + (y1 - y2).abs() as f32
}

pub fn pathfind_grid<C, P, H>(
    width: i32,
    height: i32,
    start: (i32, i32),
    end: (i32, i32),
    mut tile_cost: C,
    mut heuristic: H,
    mut passable: P,
) -> Vec<(i32, i32)>
where
    C: FnMut((i32, i32), (i32, i32)) -> f32,
    P: FnMut((i32, i32)) -> bool,
    H: FnMut(i32, i32, i32, i32) -> f32,
{
    if !in_bounds(start.0, start.1, width, height) || !in_bounds(end.0, end.1, width, height) {
        return Vec::new();
    }

    let len = (width * height) as usize;
    let start_pos = index(start.0, start.1, width);
    let end_pos = index(end.0, end.1, width);
    let mut costs = vec![0.0; len];
    let mut closed = vec![false; len];
    let mut parents = vec![None; len];
    let mut queue = BinaryHeap::new();
    queue.push(QueueNode::new(
        start_pos,
        heuristic(start.0, start.1, end.0, end.1),
    ));

    let mut found = false;
    while let Some(next) = queue.pop() {
        let (x, y) = coords(next.pos, width);
        let base_cost = costs[next.pos];
        if next.pos == end_pos {
            found = true;
            break;
        }
        closed[next.pos] = true;

        for (dx, dy) in D4 {
            let child = (x + dx, y + dy);
            if !in_bounds(child.0, child.1, width, height) || !passable(child) {
                continue;
            }

            let child_pos = index(child.0, child.1, width);
            let new_cost = tile_cost((x, y), child) + base_cost;
            if !closed[child_pos] {
                closed[child_pos] = true;
                parents[child_pos] = Some(next.pos);
                costs[child_pos] = new_cost;
                queue.push(QueueNode::new(
                    child_pos,
                    new_cost + heuristic(child.0, child.1, end.0, end.1),
                ));
            }
        }
    }

    if !found {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut current = end_pos;
    while current != start_pos {
        out.push(coords(current, width));
        let Some(parent) = parents[current] else {
            return Vec::new();
        };
        current = parent;
    }
    out.reverse();
    out
}

pub fn pathfind_grid_manhattan<C, P>(
    width: i32,
    height: i32,
    start: (i32, i32),
    end: (i32, i32),
    tile_cost: C,
    passable: P,
) -> Vec<(i32, i32)>
where
    C: FnMut((i32, i32), (i32, i32)) -> f32,
    P: FnMut((i32, i32)) -> bool,
{
    pathfind_grid(width, height, start, end, tile_cost, manhattan, passable)
}

fn in_bounds(x: i32, y: i32, width: i32, height: i32) -> bool {
    x >= 0 && y >= 0 && x < width && y < height
}

fn index(x: i32, y: i32, width: i32) -> usize {
    (x + y * width) as usize
}

fn coords(pos: usize, width: i32) -> (i32, i32) {
    (pos as i32 % width, pos as i32 / width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manhattan_distance_matches_java_default_heuristic() {
        assert_eq!(manhattan(0, 0, 3, 4), 7.0);
        assert_eq!(manhattan(5, 5, 2, 9), 7.0);
    }

    #[test]
    fn pathfind_grid_returns_empty_when_unreachable_or_out_of_bounds() {
        assert!(pathfind_grid_manhattan(4, 4, (0, 0), (3, 3), |_, _| 1.0, |_| false).is_empty());
        assert!(pathfind_grid_manhattan(4, 4, (-1, 0), (3, 3), |_, _| 1.0, |_| true).is_empty());
    }

    #[test]
    fn pathfind_grid_returns_path_without_start_including_end() {
        let path = pathfind_grid_manhattan(4, 4, (0, 0), (2, 0), |_, _| 1.0, |_| true);

        assert_eq!(path, vec![(1, 0), (2, 0)]);
    }

    #[test]
    fn pathfind_grid_routes_around_blocked_tiles() {
        let blocked = (1, 0);
        let path =
            pathfind_grid_manhattan(4, 4, (0, 0), (2, 0), |_, _| 1.0, |tile| tile != blocked);

        assert_eq!(path, vec![(0, 1), (1, 1), (2, 1), (2, 0)]);
    }
}
