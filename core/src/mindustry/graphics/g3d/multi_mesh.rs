//! Composite mesh mirroring upstream `mindustry.graphics.g3d.MultiMesh`.

use super::{Disposable, GenericMesh, Mat3D, PlanetParams};

#[derive(Default)]
pub struct MultiMesh {
    pub meshes: Vec<Box<dyn GenericMesh>>,
}

impl MultiMesh {
    pub fn new(meshes: Vec<Box<dyn GenericMesh>>) -> Self {
        Self { meshes }
    }

    pub fn push(&mut self, mesh: Box<dyn GenericMesh>) {
        self.meshes.push(mesh);
    }
}

impl Disposable for MultiMesh {
    fn dispose(&mut self) {
        for mesh in &mut self.meshes {
            mesh.dispose();
        }
    }
}

impl GenericMesh for MultiMesh {
    fn render(&mut self, params: &PlanetParams, projection: &Mat3D, transform: &Mat3D) {
        for mesh in &mut self.meshes {
            mesh.render(params, projection, transform);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug, Clone)]
    struct TraceMesh {
        name: &'static str,
        events: Rc<RefCell<Vec<String>>>,
    }

    impl TraceMesh {
        fn new(name: &'static str, events: Rc<RefCell<Vec<String>>>) -> Self {
            Self { name, events }
        }
    }

    impl Disposable for TraceMesh {
        fn dispose(&mut self) {
            self.events
                .borrow_mut()
                .push(format!("dispose:{}", self.name));
        }
    }

    impl GenericMesh for TraceMesh {
        fn render(&mut self, params: &PlanetParams, projection: &Mat3D, transform: &Mat3D) {
            assert_eq!(*projection, Mat3D::identity());
            assert_eq!(*transform, Mat3D::translation(1.0, 2.0, 3.0));
            self.events
                .borrow_mut()
                .push(format!("render:{}:{}", self.name, params.planet));
        }
    }

    #[test]
    fn multi_mesh_renders_children_in_order() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut mesh = MultiMesh::new(vec![
            Box::new(TraceMesh::new("a", events.clone())),
            Box::new(TraceMesh::new("b", events.clone())),
        ]);

        mesh.render(
            &PlanetParams::new("erekir"),
            &Mat3D::identity(),
            &Mat3D::translation(1.0, 2.0, 3.0),
        );

        assert_eq!(
            *events.borrow(),
            vec!["render:a:erekir".to_string(), "render:b:erekir".to_string()]
        );
    }

    #[test]
    fn multi_mesh_disposes_children_in_order() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut mesh = MultiMesh::default();
        mesh.push(Box::new(TraceMesh::new("a", events.clone())));
        mesh.push(Box::new(TraceMesh::new("b", events.clone())));

        mesh.dispose();

        assert_eq!(
            *events.borrow(),
            vec!["dispose:a".to_string(), "dispose:b".to_string()]
        );
    }
}
