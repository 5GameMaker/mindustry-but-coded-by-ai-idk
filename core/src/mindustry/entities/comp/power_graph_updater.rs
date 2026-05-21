//! Power graph updater component mirroring upstream
//! `mindustry.entities.comp.PowerGraphUpdaterComp`.

pub trait PowerGraphUpdate {
    fn update(&mut self);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerGraphUpdaterComp<G> {
    /// Java: `public transient PowerGraph graph`.
    pub graph: G,
}

impl<G> PowerGraphUpdaterComp<G> {
    pub const ENTITY_COMPONENT: &'static str = "PowerGraphUpdaterc";
    pub const SERIALIZE: bool = false;
    pub const GENIO: bool = false;

    pub fn new(graph: G) -> Self {
        Self { graph }
    }
}

impl<G: PowerGraphUpdate> PowerGraphUpdaterComp<G> {
    /// Java: `public void update(){ graph.update(); }`.
    pub fn update(&mut self) {
        self.graph.update();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    struct Graph {
        updates: usize,
    }

    impl PowerGraphUpdate for Graph {
        fn update(&mut self) {
            self.updates += 1;
        }
    }

    #[test]
    fn power_graph_updater_forwards_update_to_graph() {
        let mut updater = PowerGraphUpdaterComp::new(Graph::default());

        updater.update();
        updater.update();

        assert_eq!(updater.graph.updates, 2);
    }

    #[test]
    fn power_graph_updater_preserves_entity_definition_metadata() {
        assert_eq!(
            PowerGraphUpdaterComp::<Graph>::ENTITY_COMPONENT,
            "PowerGraphUpdaterc"
        );
        assert!(!PowerGraphUpdaterComp::<Graph>::SERIALIZE);
        assert!(!PowerGraphUpdaterComp::<Graph>::GENIO);
    }
}
