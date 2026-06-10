//! Transform wrapper mirroring upstream `mindustry.graphics.g3d.MatMesh`.

use super::{Disposable, GenericMesh, Mat3D, PlanetParams};

#[derive(Debug, Clone, PartialEq)]
pub struct MatMesh<M> {
    pub mesh: M,
    pub mat: Mat3D,
}

impl<M> MatMesh<M> {
    pub fn new(mesh: M, mat: Mat3D) -> Self {
        Self { mesh, mat }
    }
}

impl<M: Disposable> Disposable for MatMesh<M> {
    fn dispose(&mut self) {
        self.mesh.dispose();
    }
}

impl<M: GenericMesh> GenericMesh for MatMesh<M> {
    fn render(&mut self, params: &PlanetParams, projection: &Mat3D, transform: &Mat3D) {
        self.mesh
            .render(params, projection, &(*transform).mul(self.mat));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, Clone, PartialEq)]
    struct CapturingMesh {
        transforms: Vec<Mat3D>,
        disposed: bool,
    }

    impl Disposable for CapturingMesh {
        fn dispose(&mut self) {
            self.disposed = true;
        }
    }

    impl GenericMesh for CapturingMesh {
        fn render(&mut self, _params: &PlanetParams, projection: &Mat3D, transform: &Mat3D) {
            assert_eq!(*projection, Mat3D::identity());
            self.transforms.push(*transform);
        }
    }

    #[test]
    fn mat_mesh_applies_local_matrix_after_parent_transform() {
        let base = CapturingMesh::default();
        let local = Mat3D::translation(3.0, 4.0, 5.0);
        let parent = Mat3D::translation(10.0, 20.0, 30.0);
        let mut mesh = MatMesh::new(base, local);

        mesh.render(&PlanetParams::new("serpulo"), &Mat3D::identity(), &parent);

        assert_eq!(
            mesh.mesh.transforms,
            vec![Mat3D::translation(13.0, 24.0, 35.0)]
        );
    }

    #[test]
    fn mat_mesh_disposes_wrapped_mesh() {
        let mut mesh = MatMesh::new(CapturingMesh::default(), Mat3D::identity());

        mesh.dispose();

        assert!(mesh.mesh.disposed);
    }
}
