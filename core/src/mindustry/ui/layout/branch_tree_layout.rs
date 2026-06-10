//! Branch layout mirror of upstream `mindustry.ui.layout.BranchTreeLayout`.

use crate::mindustry::entities::entity_group::Rect;

use super::{TreeArena, TreeLayout};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeLocation {
    Top,
    Left,
    Bottom,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeAlignment {
    Center,
    TowardsRoot,
    AwayFromRoot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BranchTreeLayout {
    pub root_location: TreeLocation,
    pub alignment: TreeAlignment,
    pub gap_between_levels: f32,
    pub gap_between_nodes: f32,
    size_of_level: Vec<f32>,
    bounds_left: f32,
    bounds_right: f32,
    bounds_top: f32,
    bounds_bottom: f32,
}

impl Default for BranchTreeLayout {
    fn default() -> Self {
        Self {
            root_location: TreeLocation::Top,
            alignment: TreeAlignment::AwayFromRoot,
            gap_between_levels: 10.0,
            gap_between_nodes: 10.0,
            size_of_level: Vec::new(),
            bounds_left: f32::MAX,
            bounds_right: f32::MIN_POSITIVE,
            bounds_top: f32::MAX,
            bounds_bottom: f32::MIN_POSITIVE,
        }
    }
}

impl BranchTreeLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_bounds(&self) -> Rect {
        Rect::new(
            self.bounds_left,
            self.bounds_bottom,
            self.bounds_right - self.bounds_left,
            self.bounds_top - self.bounds_bottom,
        )
    }

    pub fn get_level_count(&self) -> usize {
        self.size_of_level.len()
    }

    pub fn get_size_of_level(&self, level: usize) -> f32 {
        self.size_of_level[level]
    }

    pub fn get_gap_between_nodes(&self, _arena: &TreeArena, _a: usize, _b: usize) -> f32 {
        self.gap_between_nodes
    }

    fn get_width_or_height_of_node(
        &self,
        arena: &mut TreeArena,
        node: usize,
        return_width: bool,
    ) -> f32 {
        if return_width {
            arena.calc_width(node)
        } else {
            arena.node(node).height
        }
    }

    fn get_node_thickness(&self, arena: &mut TreeArena, node: usize) -> f32 {
        self.get_width_or_height_of_node(arena, node, !self.is_level_change_in_y_axis())
    }

    fn get_node_size(&self, arena: &mut TreeArena, node: usize) -> f32 {
        self.get_width_or_height_of_node(arena, node, self.is_level_change_in_y_axis())
    }

    fn is_level_change_in_y_axis(&self) -> bool {
        matches!(self.root_location, TreeLocation::Top | TreeLocation::Bottom)
    }

    fn get_level_change_sign(&self) -> f32 {
        if matches!(
            self.root_location,
            TreeLocation::Bottom | TreeLocation::Right
        ) {
            -1.0
        } else {
            1.0
        }
    }

    fn update_bounds(&mut self, arena: &TreeArena, node: usize, center_x: f32, center_y: f32) {
        let width = arena.node(node).width;
        let height = arena.node(node).height;
        let left = center_x - width / 2.0;
        let right = center_x + width / 2.0;
        let top = center_y - height / 2.0;
        let bottom = center_y + height / 2.0;
        if self.bounds_left > left {
            self.bounds_left = left;
        }
        if self.bounds_right < right {
            self.bounds_right = right;
        }
        if self.bounds_top > top {
            self.bounds_top = top;
        }
        if self.bounds_bottom < bottom {
            self.bounds_bottom = bottom;
        }
    }

    fn calc_size_of_levels(&mut self, arena: &mut TreeArena, node: usize, level: usize) {
        if self.size_of_level.len() <= level {
            self.size_of_level.push(0.0);
        }
        let size = self.get_node_thickness(arena, node);
        if self.size_of_level[level] < size {
            self.size_of_level[level] = size;
        }

        let children = arena.node(node).children.clone();
        for child in children {
            self.calc_size_of_levels(arena, child, level + 1);
        }
    }

    fn get_ancestor(&self, arena: &TreeArena, node: usize) -> usize {
        arena.node(node).ancestor.unwrap_or(node)
    }

    fn get_distance(&self, arena: &mut TreeArena, v: usize, w: usize) -> f32 {
        let size_of_nodes = self.get_node_size(arena, v) + self.get_node_size(arena, w);
        size_of_nodes / 2.0 + self.get_gap_between_nodes(arena, v, w)
    }

    fn next_left(&self, arena: &TreeArena, v: usize) -> Option<usize> {
        if arena.node(v).is_leaf() {
            arena.node(v).thread
        } else {
            Some(arena.node(v).children[0])
        }
    }

    fn next_right(&self, arena: &TreeArena, v: usize) -> Option<usize> {
        if arena.node(v).is_leaf() {
            arena.node(v).thread
        } else {
            Some(*arena.node(v).children.last().unwrap())
        }
    }

    fn get_number(&self, arena: &mut TreeArena, node: usize, parent_node: usize) -> i32 {
        if arena.node(node).number == -1 {
            let mut number = 1;
            let children = arena.node(parent_node).children.clone();
            for child in children {
                arena.node_mut(child).number = number;
                number += 1;
            }
        }
        arena.node(node).number
    }

    fn ancestor(
        &self,
        arena: &TreeArena,
        v_i_minus: usize,
        parent_of_v: usize,
        default_ancestor: usize,
    ) -> usize {
        let ancestor = self.get_ancestor(arena, v_i_minus);
        if arena.node(ancestor).parent == Some(parent_of_v) {
            ancestor
        } else {
            default_ancestor
        }
    }

    fn move_subtree(
        &self,
        arena: &mut TreeArena,
        w_minus: usize,
        w_plus: usize,
        parent: usize,
        shift: f32,
    ) {
        let subtrees =
            self.get_number(arena, w_plus, parent) - self.get_number(arena, w_minus, parent);
        arena.node_mut(w_plus).change -= shift / subtrees as f32;
        arena.node_mut(w_plus).shift += shift;
        arena.node_mut(w_minus).change += shift / subtrees as f32;
        arena.node_mut(w_plus).prelim += shift;
        arena.node_mut(w_plus).mode += shift;
    }

    fn apportion(
        &self,
        arena: &mut TreeArena,
        v: usize,
        mut default_ancestor: usize,
        left_sibling: Option<usize>,
        parent_of_v: usize,
    ) -> usize {
        let Some(left_sibling) = left_sibling else {
            return default_ancestor;
        };

        let mut v_o_plus = v;
        let mut v_i_plus = v;
        let mut v_i_minus = left_sibling;
        let mut v_o_minus = arena.node(parent_of_v).children[0];

        let mut s_i_plus = arena.node(v_i_plus).mode;
        let mut s_o_plus = arena.node(v_o_plus).mode;
        let mut s_i_minus = arena.node(v_i_minus).mode;
        let mut s_o_minus = arena.node(v_o_minus).mode;

        let mut next_right_v_i_minus = self.next_right(arena, v_i_minus);
        let mut next_left_v_i_plus = self.next_left(arena, v_i_plus);

        while let (Some(next_right), Some(next_left)) = (next_right_v_i_minus, next_left_v_i_plus) {
            v_i_minus = next_right;
            v_i_plus = next_left;
            v_o_minus = self.next_left(arena, v_o_minus).unwrap();
            v_o_plus = self.next_right(arena, v_o_plus).unwrap();
            arena.node_mut(v_o_plus).ancestor = Some(v);

            let shift = (arena.node(v_i_minus).prelim + s_i_minus)
                - (arena.node(v_i_plus).prelim + s_i_plus)
                + self.get_distance(arena, v_i_minus, v_i_plus);

            if shift > 0.0 {
                let ancestor = self.ancestor(arena, v_i_minus, parent_of_v, default_ancestor);
                self.move_subtree(arena, ancestor, v, parent_of_v, shift);
                s_i_plus += shift;
                s_o_plus += shift;
            }

            s_i_minus += arena.node(v_i_minus).mode;
            s_i_plus += arena.node(v_i_plus).mode;
            s_o_minus += arena.node(v_o_minus).mode;
            s_o_plus += arena.node(v_o_plus).mode;

            next_right_v_i_minus = self.next_right(arena, v_i_minus);
            next_left_v_i_plus = self.next_left(arena, v_i_plus);
        }

        if let Some(next_right) = next_right_v_i_minus {
            if self.next_right(arena, v_o_plus).is_none() {
                arena.node_mut(v_o_plus).thread = Some(next_right);
                arena.node_mut(v_o_plus).mode += s_i_minus - s_o_plus;
            }
        }

        if let Some(next_left) = next_left_v_i_plus {
            if self.next_left(arena, v_o_minus).is_none() {
                arena.node_mut(v_o_minus).thread = Some(next_left);
                arena.node_mut(v_o_minus).mode += s_i_plus - s_o_minus;
                default_ancestor = v;
            }
        }

        default_ancestor
    }

    fn execute_shifts(&self, arena: &mut TreeArena, v: usize) {
        let mut shift = 0.0;
        let mut change = 0.0;
        let mut children = arena.node(v).children.clone();
        children.reverse();
        for w in children {
            change += arena.node(w).change;
            arena.node_mut(w).prelim += shift;
            arena.node_mut(w).mode += shift;
            shift += arena.node(w).shift + change;
        }
    }

    fn first_walk(&self, arena: &mut TreeArena, v: usize, left_sibling: Option<usize>) {
        if arena.node(v).is_leaf() {
            if let Some(left_sibling) = left_sibling {
                arena.node_mut(v).prelim =
                    arena.node(left_sibling).prelim + self.get_distance(arena, v, left_sibling);
            }
        } else {
            let children = arena.node(v).children.clone();
            let mut default_ancestor = children[0];
            let mut previous_child = None;
            for w in children.iter().copied() {
                self.first_walk(arena, w, previous_child);
                default_ancestor = self.apportion(arena, w, default_ancestor, previous_child, v);
                previous_child = Some(w);
            }
            self.execute_shifts(arena, v);

            let children = arena.node(v).children.clone();
            let midpoint = (arena.node(children[0]).prelim
                + arena.node(*children.last().unwrap()).prelim)
                / 2.0;
            if let Some(left_sibling) = left_sibling {
                let prelim =
                    arena.node(left_sibling).prelim + self.get_distance(arena, v, left_sibling);
                arena.node_mut(v).prelim = prelim;
                arena.node_mut(v).mode = prelim - midpoint;
            } else {
                arena.node_mut(v).prelim = midpoint;
            }
        }
    }

    fn second_walk(
        &mut self,
        arena: &mut TreeArena,
        v: usize,
        m: f32,
        level: usize,
        level_start: f32,
    ) {
        let level_change_sign = self.get_level_change_sign();
        let level_change_on_y_axis = self.is_level_change_in_y_axis();
        let level_size = self.get_size_of_level(level);
        let mut x = arena.node(v).prelim + m;

        let mut y = if self.alignment == TreeAlignment::Center {
            level_start + level_change_sign * (level_size / 2.0)
        } else if self.alignment == TreeAlignment::TowardsRoot {
            level_start + level_change_sign * (self.get_node_thickness(arena, v) / 2.0)
        } else {
            level_start + level_size - level_change_sign * (self.get_node_thickness(arena, v) / 2.0)
        };

        if !level_change_on_y_axis {
            std::mem::swap(&mut x, &mut y);
        }

        arena.node_mut(v).x = x;
        arena.node_mut(v).y = y;
        self.update_bounds(arena, v, x, y);

        if !arena.node(v).is_leaf() {
            let next_level_start =
                level_start + (level_size + self.gap_between_levels) * level_change_sign;
            let children = arena.node(v).children.clone();
            for w in children {
                self.second_walk(
                    arena,
                    w,
                    m + arena.node(v).mode,
                    level + 1,
                    next_level_start,
                );
            }
        }
    }
}

impl TreeLayout for BranchTreeLayout {
    fn layout(&mut self, arena: &mut TreeArena, root: usize) {
        self.size_of_level.clear();
        self.bounds_left = f32::MAX;
        self.bounds_right = f32::MIN_POSITIVE;
        self.bounds_top = f32::MAX;
        self.bounds_bottom = f32::MIN_POSITIVE;

        self.first_walk(arena, root, None);
        self.calc_size_of_levels(arena, root, 0);
        self.second_walk(arena, root, -arena.node(root).prelim, 0, 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_layout_places_children_away_from_top_root_like_java_defaults() {
        let mut arena = TreeArena::new();
        let root = arena.add_node(10.0, 10.0);
        let a = arena.add_node(10.0, 10.0);
        let b = arena.add_node(10.0, 10.0);
        arena.add_child(root, a);
        arena.add_child(root, b);

        let mut layout = BranchTreeLayout::new();
        layout.layout(&mut arena, root);

        assert_eq!(layout.get_level_count(), 2);
        assert_eq!(layout.get_size_of_level(0), 10.0);
        assert_eq!(layout.get_size_of_level(1), 10.0);
        assert_eq!((arena.node(root).x, arena.node(root).y), (0.0, 5.0));
        assert_eq!((arena.node(a).x, arena.node(a).y), (-10.0, 25.0));
        assert_eq!((arena.node(b).x, arena.node(b).y), (10.0, 25.0));
    }

    #[test]
    fn branch_layout_left_root_swaps_axes_like_java() {
        let mut arena = TreeArena::new();
        let root = arena.add_node(10.0, 10.0);
        let child = arena.add_node(10.0, 10.0);
        arena.add_child(root, child);

        let mut layout = BranchTreeLayout::new();
        layout.root_location = TreeLocation::Left;
        layout.layout(&mut arena, root);

        assert_eq!((arena.node(root).x, arena.node(root).y), (5.0, 0.0));
        assert_eq!((arena.node(child).x, arena.node(child).y), (25.0, 0.0));
    }
}
