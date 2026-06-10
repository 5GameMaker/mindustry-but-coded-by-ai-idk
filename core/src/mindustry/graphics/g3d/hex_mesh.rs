//! Hex planet mesh wrapper mirroring upstream `mindustry.graphics.g3d.HexMesh`.

use crate::mindustry::graphics::shaders::ShaderId;
use crate::mindustry::r#type::PlanetMeta;

use super::{DefaultHexMesher, G3dColor, G3dVec3, HexMesher, MeshBuildPlan, MeshBuilder};

#[derive(Debug, Clone, PartialEq)]
pub struct HexMesh {
    pub planet: String,
    pub mesh: MeshBuildPlan,
    pub shader: ShaderId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexMeshPreRenderContext {
    pub planet_position: G3dVec3,
    pub solar_system_position: G3dVec3,
    pub rotation: f32,
    pub solar_system_light_color: G3dColor,
    pub generator_emissive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexMeshPreRenderPlan {
    pub shader: ShaderId,
    pub planet_uniform: &'static str,
    pub emissive: bool,
    pub light_dir: G3dVec3,
    pub ambient_color: G3dColor,
}

impl HexMesh {
    pub fn new(planet: &PlanetMeta, divisions: i32) -> Self {
        Self::with_mesher(planet, &DefaultHexMesher, divisions, ShaderId::Planet)
    }

    pub fn with_mesher<M: HexMesher>(
        planet: &PlanetMeta,
        mesher: &M,
        divisions: i32,
        shader: ShaderId,
    ) -> Self {
        Self {
            planet: planet.name.clone(),
            mesh: MeshBuilder::build_hex(mesher, divisions, planet.radius, 0.2),
            shader,
        }
    }

    pub fn from_planet_mesh(planet: &PlanetMeta, mesh: MeshBuildPlan, shader: ShaderId) -> Self {
        Self {
            planet: planet.name.clone(),
            mesh,
            shader,
        }
    }

    pub fn pre_render(&self, context: HexMeshPreRenderContext) -> HexMeshPreRenderPlan {
        HexMeshPreRenderPlan {
            shader: ShaderId::Planet,
            planet_uniform: "Shaders.planet.planet",
            emissive: context.generator_emissive,
            light_dir: context
                .solar_system_position
                .sub(context.planet_position)
                .rotate_y_degrees(context.rotation)
                .nor(),
            ambient_color: context.solar_system_light_color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_mesh_constructor_builds_planet_shader_hex_with_radius_intensity() {
        let planet = PlanetMeta::new("serpulo", 3.0);

        let mesh = HexMesh::new(&planet, 1);

        assert_eq!(mesh.planet, "serpulo");
        assert_eq!(mesh.shader, ShaderId::Planet);
        assert_eq!(mesh.mesh.divisions, 1);
        assert_eq!(mesh.mesh.radius, 3.0);
        assert_eq!(mesh.mesh.intensity, 0.2);
    }

    #[test]
    fn hex_mesh_pre_render_sets_planet_shader_state_like_java() {
        let planet = PlanetMeta::new("erekir", 4.0);
        let mesh = HexMesh::new(&planet, 0);

        let plan = mesh.pre_render(HexMeshPreRenderContext {
            planet_position: G3dVec3::new(1.0, 0.0, 0.0),
            solar_system_position: G3dVec3::new(1.0, 0.0, 2.0),
            rotation: 90.0,
            solar_system_light_color: G3dColor::rgb(0.2, 0.3, 0.4),
            generator_emissive: true,
        });

        assert_eq!(plan.shader, ShaderId::Planet);
        assert!(plan.emissive);
        assert!((plan.light_dir.x - 1.0).abs() < 0.0001);
        assert!(plan.light_dir.z.abs() < 0.0001);
        assert_eq!(plan.ambient_color, G3dColor::rgb(0.2, 0.3, 0.4));
    }
}
