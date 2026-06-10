//! Row layout mirror of upstream `mindustry.ui.layout.RowTreeLayout`.

use super::{TreeArena, TreeLayout};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RowTreeLayout;

impl RowTreeLayout {
    fn layout_node(arena: &mut TreeArena, node: usize, depth: usize, nexts: &mut Vec<i32>) {
        let size = arena.node(node).height * 5.0;
        if nexts.len() < depth + 1 {
            nexts.resize(depth + 1, 0);
        }

        let x = size * nexts[depth] as f32;
        let y = size * depth as f32;
        arena.node_mut(node).x = x;
        arena.node_mut(node).y = y;
        nexts[depth] += 1;

        let children = arena.node(node).children.clone();
        for child in children {
            Self::layout_node(arena, child, depth + 1, nexts);
        }
    }
}

impl TreeLayout for RowTreeLayout {
    fn layout(&mut self, arena: &mut TreeArena, root: usize) {
        Self::layout_node(arena, root, 0, &mut Vec::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_tree_layout_uses_height_times_five_per_depth_like_java() {
        let mut arena = TreeArena::new();
        let root = arena.add_node(10.0, 4.0);
        let a = arena.add_node(10.0, 4.0);
        let b = arena.add_node(10.0, 4.0);
        let c = arena.add_node(10.0, 4.0);
        arena.add_child(root, a);
        arena.add_child(root, b);
        arena.add_child(a, c);

        RowTreeLayout.layout(&mut arena, root);

        assert_eq!((arena.node(root).x, arena.node(root).y), (0.0, 0.0));
        assert_eq!((arena.node(a).x, arena.node(a).y), (0.0, 20.0));
        assert_eq!((arena.node(b).x, arena.node(b).y), (20.0, 20.0));
        assert_eq!((arena.node(c).x, arena.node(c).y), (0.0, 40.0));
    }
}
