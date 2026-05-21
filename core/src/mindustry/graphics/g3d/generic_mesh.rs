//! Generic mesh render contract mirroring upstream `mindustry.graphics.g3d.GenericMesh`.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3D {
    pub values: [[f32; 4]; 4],
}

impl Mat3D {
    pub const fn identity() -> Self {
        Self {
            values: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetParams {
    pub planet: String,
}

impl PlanetParams {
    pub fn new(planet: impl Into<String>) -> Self {
        Self {
            planet: planet.into(),
        }
    }
}

pub trait Disposable {
    fn dispose(&mut self) {}
}

pub trait GenericMesh: Disposable {
    fn render(&mut self, params: &PlanetParams, projection: &Mat3D, transform: &Mat3D);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct Mesh {
        renders: Vec<String>,
        disposed: bool,
    }

    impl Disposable for Mesh {
        fn dispose(&mut self) {
            self.disposed = true;
        }
    }

    impl GenericMesh for Mesh {
        fn render(&mut self, params: &PlanetParams, projection: &Mat3D, transform: &Mat3D) {
            assert_eq!(*projection, Mat3D::identity());
            assert_eq!(*transform, Mat3D::identity());
            self.renders.push(params.planet.clone());
        }
    }

    #[test]
    fn generic_mesh_render_receives_planet_params_and_matrices() {
        let mut mesh = Mesh::default();
        let matrix = Mat3D::identity();

        mesh.render(&PlanetParams::new("serpulo"), &matrix, &matrix);

        assert_eq!(mesh.renders, vec!["serpulo"]);
    }

    #[test]
    fn generic_mesh_extends_disposable_contract() {
        let mut mesh = Mesh::default();

        mesh.dispose();

        assert!(mesh.disposed);
    }
}
