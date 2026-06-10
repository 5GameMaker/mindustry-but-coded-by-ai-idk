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

    pub const fn from_values(values: [[f32; 4]; 4]) -> Self {
        Self { values }
    }

    pub const fn translation(x: f32, y: f32, z: f32) -> Self {
        Self {
            values: [
                [1.0, 0.0, 0.0, x],
                [0.0, 1.0, 0.0, y],
                [0.0, 0.0, 1.0, z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_y_degrees(degrees: f32) -> Self {
        let radians = degrees.to_radians();
        let cos = radians.cos();
        let sin = radians.sin();
        Self {
            values: [
                [cos, 0.0, sin, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-sin, 0.0, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn mul(self, rhs: Self) -> Self {
        let mut values = [[0.0; 4]; 4];
        for row in 0..4 {
            for col in 0..4 {
                values[row][col] = self.values[row][0] * rhs.values[0][col]
                    + self.values[row][1] * rhs.values[1][col]
                    + self.values[row][2] * rhs.values[2][col]
                    + self.values[row][3] * rhs.values[3][col];
            }
        }
        Self { values }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetParams {
    pub planet: String,
    pub ui_alpha: f32,
}

impl PlanetParams {
    pub fn new(planet: impl Into<String>) -> Self {
        Self {
            planet: planet.into(),
            ui_alpha: 1.0,
        }
    }

    pub fn with_ui_alpha(mut self, ui_alpha: f32) -> Self {
        self.ui_alpha = ui_alpha;
        self
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

    #[test]
    fn mat3d_mul_uses_transform_then_local_matrix_order() {
        let transform = Mat3D::translation(2.0, 3.0, 4.0);
        let local = Mat3D::from_values([
            [2.0, 0.0, 0.0, 5.0],
            [0.0, 3.0, 0.0, 7.0],
            [0.0, 0.0, 4.0, 11.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let combined = transform.mul(local);

        assert_eq!(
            combined,
            Mat3D::from_values([
                [2.0, 0.0, 0.0, 7.0],
                [0.0, 3.0, 0.0, 10.0],
                [0.0, 0.0, 4.0, 15.0],
                [0.0, 0.0, 0.0, 1.0],
            ])
        );
    }

    #[test]
    fn mat3d_rotation_y_uses_degrees() {
        let rotated = Mat3D::rotation_y_degrees(90.0);

        assert!(rotated.values[0][0].abs() < 0.0001);
        assert!((rotated.values[0][2] - 1.0).abs() < 0.0001);
        assert!((rotated.values[2][0] + 1.0).abs() < 0.0001);
    }

    #[test]
    fn planet_params_default_ui_alpha_matches_visible_ui() {
        assert_eq!(PlanetParams::new("serpulo").ui_alpha, 1.0);
        assert_eq!(
            PlanetParams::new("serpulo").with_ui_alpha(0.25).ui_alpha,
            0.25
        );
    }
}
