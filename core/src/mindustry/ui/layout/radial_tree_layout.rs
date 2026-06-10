//! Radial layout mirror of upstream `mindustry.ui.layout.RadialTreeLayout`.

use std::collections::{BTreeSet, VecDeque};

use super::{TreeArena, TreeLayout};

const DEG_RAD: f32 = std::f32::consts::PI / 180.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadialTreeLayout {
    pub start_radius: f32,
    pub delta: f32,
}

impl Default for RadialTreeLayout {
    fn default() -> Self {
        Self {
            start_radius: 0.0,
            delta: 0.0,
        }
    }
}

impl RadialTreeLayout {
    pub fn new() -> Self {
        Self::default()
    }

    fn bfs(arena: &mut TreeArena, node: usize, assign: bool) -> i32 {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        if assign {
            arena.node_mut(node).number = 0;
        }

        let mut leaves = 0;
        visited.insert(node);
        queue.push_front(node);

        while let Some(current) = queue.pop_front() {
            if arena.node(current).children.is_empty() {
                leaves += 1;
            }

            let children = arena.node(current).children.clone();
            for child in children {
                if assign {
                    arena.node_mut(child).number = arena.node(current).number + 1;
                }
                if visited.insert(child) {
                    queue.push_back(child);
                }
            }
        }

        leaves
    }

    fn bfs_indices(arena: &TreeArena, node: usize) -> Vec<usize> {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        let mut out = Vec::new();

        visited.insert(node);
        queue.push_front(node);
        while let Some(current) = queue.pop_front() {
            out.push(current);
            for child in arena.node(current).children.iter().copied() {
                if visited.insert(child) {
                    queue.push_back(child);
                }
            }
        }
        out
    }

    fn radialize(&self, arena: &mut TreeArena, root: usize, radius: f32, from: f32, to: f32) {
        let mut angle = from;
        let children = arena.node(root).children.clone();
        for child in children {
            let next_angle = angle
                + (arena.node(child).leaves as f32 / arena.node(root).leaves as f32) * (to - from);
            let middle = (angle + next_angle) / 2.0 * DEG_RAD;
            arena.node_mut(child).x = radius * middle.cos();
            arena.node_mut(child).y = radius * middle.sin();

            if !arena.node(child).children.is_empty() {
                self.radialize(arena, child, radius + self.delta, angle, next_angle);
            }
            angle = next_angle;
        }
    }
}

impl TreeLayout for RadialTreeLayout {
    fn layout(&mut self, arena: &mut TreeArena, root: usize) {
        self.start_radius = arena.node(root).height * 2.4;
        self.delta = arena.node(root).height * 20.4;

        Self::bfs(arena, root, true);
        let all = Self::bfs_indices(arena, root);
        for node in all {
            let leaves = Self::bfs(arena, node, false);
            arena.node_mut(node).leaves = leaves;
        }

        self.radialize(arena, root, self.start_radius, 0.0, 360.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radial_layout_assigns_bfs_numbers_leaves_and_cardinal_children() {
        let mut arena = TreeArena::new();
        let root = arena.add_node(10.0, 10.0);
        let a = arena.add_node(10.0, 10.0);
        let b = arena.add_node(10.0, 10.0);
        arena.add_child(root, a);
        arena.add_child(root, b);

        let mut layout = RadialTreeLayout::new();
        layout.layout(&mut arena, root);

        assert_eq!(layout.start_radius, 24.0);
        assert_eq!(layout.delta, 204.0);
        assert_eq!(arena.node(root).leaves, 2);
        assert_eq!(arena.node(a).number, 1);
        assert!((arena.node(a).x - 0.0).abs() < 0.0001);
        assert!((arena.node(a).y - 24.0).abs() < 0.0001);
        assert!((arena.node(b).x - 0.0).abs() < 0.0001);
        assert!((arena.node(b).y + 24.0).abs() < 0.0001);
    }
}
