use std::collections::BTreeMap;

use crate::mindustry::{
    ctype::ContentType,
    game::{ObjectiveContent, ObjectiveKind, SectorObjectiveState},
    r#type::ItemStack,
};

pub type TechNodeId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechContentRef {
    pub content_type: ContentType,
    pub name: String,
    pub localized_name: String,
    pub emoji: String,
    pub unlocked_host: bool,
    pub parent_unlocked_host: bool,
}

impl TechContentRef {
    pub fn new(content_type: ContentType, name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            content_type,
            localized_name: name.clone(),
            name,
            emoji: String::new(),
            unlocked_host: false,
            parent_unlocked_host: true,
        }
    }

    pub fn block(name: impl Into<String>) -> Self {
        Self::new(ContentType::Block, name)
    }

    pub fn item(name: impl Into<String>) -> Self {
        Self::new(ContentType::Item, name)
    }

    pub fn liquid(name: impl Into<String>) -> Self {
        Self::new(ContentType::Liquid, name)
    }

    pub fn unit(name: impl Into<String>) -> Self {
        Self::new(ContentType::Unit, name)
    }

    pub fn sector(name: impl Into<String>) -> Self {
        Self::new(ContentType::Sector, name)
    }

    pub fn planet(name: impl Into<String>) -> Self {
        Self::new(ContentType::Planet, name)
    }

    pub fn localized(mut self, localized_name: impl Into<String>) -> Self {
        self.localized_name = localized_name.into();
        self
    }

    pub fn emoji(mut self, emoji: impl Into<String>) -> Self {
        self.emoji = emoji.into();
        self
    }

    pub fn unlocked_host(mut self, unlocked_host: bool) -> Self {
        self.unlocked_host = unlocked_host;
        self
    }

    pub fn parent_unlocked_host(mut self, parent_unlocked_host: bool) -> Self {
        self.parent_unlocked_host = parent_unlocked_host;
        self
    }

    pub fn objective_content(&self) -> ObjectiveContent {
        ObjectiveContent::new(self.name.clone())
            .localized(self.localized_name.clone())
            .emoji(self.emoji.clone())
            .unlocked_host(self.unlocked_host)
            .parent_unlocked_host(self.parent_unlocked_host)
    }

    pub fn sector_objective_state(&self) -> SectorObjectiveState {
        SectorObjectiveState::new(self.name.clone()).localized(self.localized_name.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TechNode {
    pub id: TechNodeId,
    pub depth: usize,
    pub icon: Option<String>,
    pub name: Option<String>,
    pub requires_unlock: bool,
    pub parent: Option<TechNodeId>,
    pub research_cost_multipliers: BTreeMap<String, f32>,
    pub content: TechContentRef,
    pub requirements: Vec<ItemStack>,
    pub finished_requirements: Vec<ItemStack>,
    pub objectives: Vec<ObjectiveKind>,
    pub children: Vec<TechNodeId>,
    pub planet: Option<String>,
    pub database_tabs: Vec<TechContentRef>,
    pub shown_planets: Vec<String>,
    pub removed: bool,
}

impl TechNode {
    pub fn progress_key(&self, item: &str) -> String {
        format!("req-{}-{item}", self.content.name)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TechTree {
    context: Option<TechNodeId>,
    all: Vec<TechNode>,
    roots: Vec<TechNodeId>,
    progress: BTreeMap<String, i32>,
}

impl TechTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_progress(progress: BTreeMap<String, i32>) -> Self {
        Self {
            progress,
            ..Self::default()
        }
    }

    pub fn context(&self) -> Option<TechNodeId> {
        self.context
    }

    pub fn all(&self) -> &[TechNode] {
        &self.all
    }

    pub fn roots(&self) -> &[TechNodeId] {
        &self.roots
    }

    pub fn progress(&self) -> &BTreeMap<String, i32> {
        &self.progress
    }

    pub fn node(&self, id: TechNodeId) -> Option<&TechNode> {
        self.all.get(id).filter(|node| !node.removed)
    }

    pub fn node_mut(&mut self, id: TechNodeId) -> Option<&mut TechNode> {
        self.all.get_mut(id).filter(|node| !node.removed)
    }

    pub fn node_root(
        &mut self,
        name: impl Into<String>,
        content: TechContentRef,
        requirements: Vec<ItemStack>,
        children: impl FnOnce(&mut Self),
    ) -> TechNodeId {
        self.node_root_with_unlock(name, content, false, requirements, children)
    }

    pub fn node_root_with_unlock(
        &mut self,
        name: impl Into<String>,
        content: TechContentRef,
        requires_unlock: bool,
        requirements: Vec<ItemStack>,
        children: impl FnOnce(&mut Self),
    ) -> TechNodeId {
        let root = self.node_with_objectives(content, requirements, Vec::new(), children);
        let root_node = self
            .all
            .get_mut(root)
            .expect("newly-created root node must exist");
        root_node.name = Some(name.into());
        root_node.requires_unlock = requires_unlock;
        self.roots.push(root);
        root
    }

    pub fn node_leaf(
        &mut self,
        content: TechContentRef,
        requirements: Vec<ItemStack>,
    ) -> TechNodeId {
        self.node_with_objectives(content, requirements, Vec::new(), |_| {})
    }

    pub fn node_with_objectives(
        &mut self,
        content: TechContentRef,
        requirements: Vec<ItemStack>,
        objectives: Vec<ObjectiveKind>,
        children: impl FnOnce(&mut Self),
    ) -> TechNodeId {
        let parent = self.context;
        let node = self.push_node(parent, content, requirements, objectives);
        let previous = self.context;
        self.context = Some(node);
        children(self);
        self.context = previous;
        node
    }

    pub fn node_produce(
        &mut self,
        content: TechContentRef,
        requirements: Vec<ItemStack>,
        mut objectives: Vec<ObjectiveKind>,
        children: impl FnOnce(&mut Self),
    ) -> TechNodeId {
        objectives.push(ObjectiveKind::Produce(content.objective_content()));
        self.node_with_objectives(content, requirements, objectives, children)
    }

    pub fn set_context_research_cost_multipliers(&mut self, multipliers: BTreeMap<String, f32>) {
        let Some(context) = self.context else {
            return;
        };
        if let Some(node) = self.all.get_mut(context) {
            node.research_cost_multipliers = multipliers;
        }
    }

    pub fn set_context_planet(&mut self, planet: impl Into<String>) {
        let Some(context) = self.context else {
            return;
        };
        if let Some(node) = self.all.get_mut(context) {
            node.planet = Some(planet.into());
        }
    }

    pub fn each_from(&self, root: TechNodeId) -> Vec<TechNodeId> {
        let mut out = Vec::new();
        self.each_from_inner(root, &mut out);
        out
    }

    pub fn add_database_tab(&mut self, root: TechNodeId, tab: TechContentRef) {
        for id in self.each_from(root) {
            if let Some(node) = self.all.get_mut(id) {
                node.database_tabs.push(tab.clone());
            }
        }
    }

    pub fn add_planet(&mut self, root: TechNodeId, planet: impl Into<String>) {
        let planet = planet.into();
        for id in self.each_from(root) {
            if let Some(node) = self.all.get_mut(id) {
                node.shown_planets.push(planet.clone());
            }
        }
    }

    pub fn reset_node(&mut self, id: TechNodeId) {
        if let Some(node) = self.all.get_mut(id) {
            for stack in &mut node.finished_requirements {
                stack.amount = 0;
            }
        }
        self.save_node(id);
    }

    pub fn save_node(&mut self, id: TechNodeId) {
        let Some(node) = self.all.get(id) else {
            return;
        };
        for stack in &node.finished_requirements {
            self.progress
                .insert(node.progress_key(&stack.item), stack.amount);
        }
    }

    pub fn remove_node(&mut self, id: TechNodeId) {
        let Some(node) = self.all.get(id) else {
            return;
        };
        let parent = node.parent;
        if let Some(parent) = parent.and_then(|parent| self.all.get_mut(parent)) {
            parent.children.retain(|&child| child != id);
        }
        self.roots.retain(|&root| root != id);
        if let Some(node) = self.all.get_mut(id) {
            node.removed = true;
        }
    }

    fn push_node(
        &mut self,
        parent: Option<TechNodeId>,
        content: TechContentRef,
        requirements: Vec<ItemStack>,
        mut objectives: Vec<ObjectiveKind>,
    ) -> TechNodeId {
        let id = self.all.len();
        let (depth, planet, research_cost_multipliers) = parent
            .and_then(|parent| self.all.get(parent))
            .map(|parent| {
                (
                    parent.depth + 1,
                    parent.planet.clone(),
                    parent.research_cost_multipliers.clone(),
                )
            })
            .unwrap_or_else(|| (0, None, BTreeMap::new()));

        if let Some(parent) = parent.and_then(|parent| self.all.get(parent)) {
            if parent.content.content_type == ContentType::Sector
                && !objectives.iter().any(|objective| {
                    matches!(
                        objective,
                        ObjectiveKind::SectorComplete(sector)
                            if sector.name == parent.content.name
                    )
                })
            {
                objectives.insert(
                    0,
                    ObjectiveKind::SectorComplete(parent.content.sector_objective_state()),
                );
            }
        }

        let requirements =
            apply_research_cost_multipliers(requirements, &research_cost_multipliers);
        let finished_requirements = requirements
            .iter()
            .map(|requirement| {
                let amount = self
                    .progress
                    .get(&format!("req-{}-{}", content.name, requirement.item))
                    .copied()
                    .unwrap_or(0);
                ItemStack::new(requirement.item.clone(), amount)
            })
            .collect();

        let node = TechNode {
            id,
            depth,
            icon: None,
            name: None,
            requires_unlock: false,
            parent,
            research_cost_multipliers,
            content,
            requirements,
            finished_requirements,
            objectives,
            children: Vec::new(),
            planet,
            database_tabs: Vec::new(),
            shown_planets: Vec::new(),
            removed: false,
        };

        self.all.push(node);
        if let Some(parent) = parent.and_then(|parent| self.all.get_mut(parent)) {
            parent.children.push(id);
        }
        id
    }

    fn each_from_inner(&self, id: TechNodeId, out: &mut Vec<TechNodeId>) {
        let Some(node) = self.node(id) else {
            return;
        };
        out.push(id);
        for &child in &node.children {
            self.each_from_inner(child, out);
        }
    }
}

fn apply_research_cost_multipliers(
    requirements: Vec<ItemStack>,
    multipliers: &BTreeMap<String, f32>,
) -> Vec<ItemStack> {
    if multipliers.is_empty() {
        return requirements;
    }

    requirements
        .into_iter()
        .map(|mut requirement| {
            if let Some(multiplier) = multipliers.get(&requirement.item) {
                requirement.amount = (requirement.amount as f32 * multiplier) as i32;
            }
            requirement
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(item: &str, amount: i32) -> ItemStack {
        ItemStack::new(item, amount)
    }

    #[test]
    fn node_root_and_context_build_parent_child_depth_like_java_builder() {
        let mut tree = TechTree::new();
        let mut conveyor = None;
        let mut router = None;

        let root = tree.node_root(
            "serpulo",
            TechContentRef::block("core-shard"),
            vec![stack("copper", 10)],
            |tree| {
                assert_eq!(tree.context(), Some(0));
                conveyor = Some(tree.node_with_objectives(
                    TechContentRef::block("conveyor"),
                    vec![stack("copper", 1)],
                    Vec::new(),
                    |tree| {
                        router =
                            Some(tree.node_leaf(
                                TechContentRef::block("router"),
                                vec![stack("copper", 3)],
                            ));
                    },
                ));
            },
        );

        let conveyor = conveyor.unwrap();
        let router = router.unwrap();
        assert_eq!(tree.context(), None);
        assert_eq!(tree.roots(), &[root]);
        assert_eq!(tree.node(root).unwrap().name.as_deref(), Some("serpulo"));
        assert_eq!(tree.node(root).unwrap().depth, 0);
        assert_eq!(tree.node(conveyor).unwrap().parent, Some(root));
        assert_eq!(tree.node(conveyor).unwrap().depth, 1);
        assert_eq!(tree.node(router).unwrap().parent, Some(conveyor));
        assert_eq!(tree.node(router).unwrap().depth, 2);
        assert_eq!(tree.node(root).unwrap().children, vec![conveyor]);
        assert_eq!(tree.node(conveyor).unwrap().children, vec![router]);
        assert_eq!(tree.each_from(root), vec![root, conveyor, router]);
    }

    #[test]
    fn sector_parent_dependency_is_inserted_once_as_first_objective() {
        let mut tree = TechTree::new();
        let mut child = None;
        let root = tree.node_root(
            "serpulo",
            TechContentRef::block("core-shard"),
            Vec::new(),
            |tree| {
                tree.node_with_objectives(
                    TechContentRef::sector("groundZero").localized("Ground Zero"),
                    Vec::new(),
                    Vec::new(),
                    |tree| {
                        child = Some(tree.node_leaf(
                            TechContentRef::block("launch-pad"),
                            vec![stack("copper", 100)],
                        ));
                    },
                );
            },
        );

        let child = child.unwrap();
        assert_eq!(tree.node(root).unwrap().children.len(), 1);
        assert!(matches!(
            &tree.node(child).unwrap().objectives[..],
            [ObjectiveKind::SectorComplete(sector)] if sector.name == "groundZero"
        ));

        let mut no_duplicate = None;
        tree.node_with_objectives(
            TechContentRef::sector("frozenForest"),
            Vec::new(),
            Vec::new(),
            |tree| {
                no_duplicate = Some(tree.node_with_objectives(
                    TechContentRef::block("pneumatic-drill"),
                    Vec::new(),
                    vec![ObjectiveKind::SectorComplete(SectorObjectiveState::new(
                        "frozenForest",
                    ))],
                    |_| {},
                ));
            },
        );
        assert_eq!(
            tree.node(no_duplicate.unwrap()).unwrap().objectives.len(),
            1
        );
    }

    #[test]
    fn node_produce_appends_produce_objective_for_its_content() {
        let mut tree = TechTree::new();
        let node = tree.node_produce(
            TechContentRef::item("graphite").localized("Graphite"),
            Vec::new(),
            vec![ObjectiveKind::Research(
                TechContentRef::block("graphite-press").objective_content(),
            )],
            |_| {},
        );

        assert!(matches!(
            &tree.node(node).unwrap().objectives[..],
            [ObjectiveKind::Research(content), ObjectiveKind::Produce(produce)]
                if content.name == "graphite-press" && produce.name == "graphite"
        ));
    }

    #[test]
    fn research_cost_multipliers_are_inherited_and_truncate_like_java_cast() {
        let mut tree = TechTree::new();
        let mut child = None;

        tree.node_root(
            "erekir",
            TechContentRef::block("core-bastion"),
            vec![stack("beryllium", 10)],
            |tree| {
                tree.set_context_research_cost_multipliers(BTreeMap::from([
                    ("beryllium".into(), 0.9),
                    ("oxide".into(), 1.5),
                ]));
                child = Some(tree.node_leaf(
                    TechContentRef::block("duct"),
                    vec![
                        stack("beryllium", 7),
                        stack("oxide", 3),
                        stack("copper", 11),
                    ],
                ));
            },
        );

        assert_eq!(
            tree.node(child.unwrap()).unwrap().requirements,
            vec![
                stack("beryllium", 6),
                stack("oxide", 4),
                stack("copper", 11)
            ]
        );
    }

    #[test]
    fn finished_requirements_save_and_reset_use_java_settings_key_shape() {
        let mut tree = TechTree::with_progress(BTreeMap::from([("req-router-copper".into(), 4)]));
        let node = tree.node_leaf(TechContentRef::block("router"), vec![stack("copper", 10)]);

        assert_eq!(
            tree.node(node).unwrap().finished_requirements,
            vec![stack("copper", 4)]
        );

        tree.node_mut(node).unwrap().finished_requirements[0].amount = 7;
        tree.save_node(node);
        assert_eq!(tree.progress().get("req-router-copper"), Some(&7));

        tree.reset_node(node);
        assert_eq!(
            tree.node(node).unwrap().finished_requirements,
            vec![stack("copper", 0)]
        );
        assert_eq!(tree.progress().get("req-router-copper"), Some(&0));
    }

    #[test]
    fn remove_and_tree_wide_annotations_follow_java_utility_methods() {
        let mut tree = TechTree::new();
        let mut child = None;
        let root = tree.node_root(
            "serpulo",
            TechContentRef::block("core-shard"),
            Vec::new(),
            |tree| {
                child = Some(tree.node_leaf(TechContentRef::block("conveyor"), Vec::new()));
            },
        );
        let child = child.unwrap();

        tree.add_database_tab(root, TechContentRef::block("core-shard"));
        tree.add_planet(root, "serpulo");
        assert_eq!(tree.node(root).unwrap().database_tabs[0].name, "core-shard");
        assert_eq!(
            tree.node(child).unwrap().database_tabs[0].name,
            "core-shard"
        );
        assert_eq!(tree.node(root).unwrap().shown_planets, vec!["serpulo"]);
        assert_eq!(tree.node(child).unwrap().shown_planets, vec!["serpulo"]);

        tree.remove_node(child);
        assert!(tree.node(child).is_none());
        assert!(tree.node(root).unwrap().children.is_empty());
        assert_eq!(tree.each_from(root), vec![root]);
    }
}
