//! Shared tree node arena for upstream `mindustry.ui.layout.TreeLayout`.

#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub children: Vec<usize>,
    pub parent: Option<usize>,
    pub mode: f32,
    pub prelim: f32,
    pub change: f32,
    pub shift: f32,
    pub cached_width: f32,
    pub number: i32,
    pub leaves: i32,
    pub thread: Option<usize>,
    pub ancestor: Option<usize>,
}

impl TreeNode {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            x: 0.0,
            y: 0.0,
            children: Vec::new(),
            parent: None,
            mode: 0.0,
            prelim: 0.0,
            change: 0.0,
            shift: 0.0,
            cached_width: -1.0,
            number: -1,
            leaves: 0,
            thread: None,
            ancestor: None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TreeArena {
    nodes: Vec<TreeNode>,
}

impl TreeArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, width: f32, height: f32) -> usize {
        let index = self.nodes.len();
        self.nodes.push(TreeNode::new(width, height));
        index
    }

    pub fn add_child(&mut self, parent: usize, child: usize) {
        self.nodes[child].parent = Some(parent);
        self.nodes[parent].children.push(child);
    }

    pub fn node(&self, index: usize) -> &TreeNode {
        &self.nodes[index]
    }

    pub fn node_mut(&mut self, index: usize) -> &mut TreeNode {
        &mut self.nodes[index]
    }

    pub fn nodes(&self) -> &[TreeNode] {
        &self.nodes
    }

    pub fn calc_width(&mut self, index: usize) -> f32 {
        if self.nodes[index].children.is_empty() {
            return self.nodes[index].width;
        }
        if self.nodes[index].cached_width > 0.0 {
            return self.nodes[index].cached_width;
        }

        let children = self.nodes[index].children.clone();
        let mut children_width = 0.0;
        for child in children {
            children_width += self.calc_width(child);
        }
        let width = self.nodes[index].width.max(children_width);
        self.nodes[index].cached_width = width;
        width
    }
}

pub trait TreeLayout {
    fn layout(&mut self, arena: &mut TreeArena, root: usize);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_width_matches_java_recursive_cache() {
        let mut arena = TreeArena::new();
        let root = arena.add_node(30.0, 10.0);
        let left = arena.add_node(12.0, 10.0);
        let right = arena.add_node(20.0, 10.0);
        arena.add_child(root, left);
        arena.add_child(root, right);

        assert_eq!(arena.calc_width(root), 32.0);
        assert_eq!(arena.node(root).cached_width, 32.0);
        assert!(!arena.node(root).is_leaf());
        assert!(arena.node(left).is_leaf());
    }
}
