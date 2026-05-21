//! Shader sphere mesh shell mirroring upstream `mindustry.graphics.g3d.ShaderSphereMesh`.

use crate::mindustry::r#type::PlanetMeta;

#[derive(Debug, Clone, PartialEq)]
pub struct MeshGeometry {
    pub kind: String,
    pub divisions: i32,
    pub radius: f32,
}

impl MeshGeometry {
    pub fn icosphere(divisions: i32, radius: f32) -> Self {
        Self {
            kind: "icosphere".into(),
            divisions,
            radius,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetMesh {
    pub planet: String,
    pub mesh: MeshGeometry,
    pub shader: String,
}

impl PlanetMesh {
    pub fn new(planet: &PlanetMeta, mesh: MeshGeometry, shader: impl Into<String>) -> Self {
        Self {
            planet: planet.name.clone(),
            mesh,
            shader: shader.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShaderSphereMesh {
    pub base: PlanetMesh,
}

impl ShaderSphereMesh {
    pub fn new(planet: &PlanetMeta, shader: impl Into<String>, divisions: i32) -> Self {
        Self {
            base: PlanetMesh::new(
                planet,
                MeshGeometry::icosphere(divisions, planet.radius),
                shader,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_sphere_mesh_builds_icosphere_with_planet_radius() {
        let planet = PlanetMeta::new("serpulo", 3.5);

        let mesh = ShaderSphereMesh::new(&planet, "atmosphere", 4);

        assert_eq!(mesh.base.planet, "serpulo");
        assert_eq!(mesh.base.shader, "atmosphere");
        assert_eq!(
            mesh.base.mesh,
            MeshGeometry {
                kind: "icosphere".into(),
                divisions: 4,
                radius: 3.5
            }
        );
    }
}
