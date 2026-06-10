//! Cloud layer hex mesh mirroring upstream `mindustry.graphics.g3d.HexSkyMesh`.

use crate::mindustry::graphics::shaders::ShaderId;
use crate::mindustry::r#type::PlanetMeta;

use super::{
    simplex_noise3d, G3dColor, G3dVec3, HexMesh, HexMesher, Mat3D, MeshBuilder, PlanetParams,
};

#[derive(Debug, Clone, PartialEq)]
pub struct HexSkyMesh {
    pub base: HexMesh,
    pub seed: i32,
    pub speed: f32,
    pub radius: f32,
    pub color: G3dColor,
    pub octaves: i32,
    pub persistence: f32,
    pub scale: f32,
    pub threshold: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SkyMesher {
    seed: i32,
    color: G3dColor,
    octaves: i32,
    persistence: f32,
    scale: f32,
    threshold: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexSkyPreRenderContext {
    pub planet_position: G3dVec3,
    pub solar_system_position: G3dVec3,
    pub rotation: f32,
    pub solar_system_light_color: G3dColor,
    pub global_time: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexSkyPreRenderPlan {
    pub shader: ShaderId,
    pub light_dir: G3dVec3,
    pub ambient_color: G3dColor,
    pub alpha: f32,
}

impl HexSkyMesh {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        planet: &PlanetMeta,
        seed: i32,
        speed: f32,
        radius: f32,
        divisions: i32,
        color: G3dColor,
        octaves: i32,
        persistence: f32,
        scale: f32,
        threshold: f32,
    ) -> Self {
        let mesher = SkyMesher {
            seed,
            color,
            octaves,
            persistence,
            scale,
            threshold,
        };
        Self {
            base: HexMesh::from_planet_mesh(
                planet,
                MeshBuilder::build_hex(&mesher, divisions, planet.radius, radius),
                ShaderId::Clouds,
            ),
            seed,
            speed,
            radius,
            color,
            octaves,
            persistence,
            scale,
            threshold,
        }
    }

    pub fn rel_rot(&self, global_time: f32) -> f32 {
        global_time * self.speed / 40.0
    }

    pub fn should_render(&self, params: &PlanetParams) -> bool {
        !(params.planet == self.base.planet && (1.0 - params.ui_alpha).abs() <= 0.01)
    }

    pub fn cloud_alpha(&self, params: &PlanetParams) -> f32 {
        if params.planet == self.base.planet {
            1.0 - params.ui_alpha
        } else {
            1.0
        }
    }

    pub fn pre_render(
        &self,
        params: &PlanetParams,
        context: HexSkyPreRenderContext,
    ) -> HexSkyPreRenderPlan {
        let rotation = context.rotation + self.rel_rot(context.global_time);
        HexSkyPreRenderPlan {
            shader: ShaderId::Clouds,
            light_dir: context
                .solar_system_position
                .sub(context.planet_position)
                .rotate_y_degrees(rotation)
                .nor(),
            ambient_color: context.solar_system_light_color,
            alpha: self.cloud_alpha(params),
        }
    }

    pub fn render_transform(
        &self,
        planet_position: G3dVec3,
        planet_rotation: f32,
        global_time: f32,
    ) -> Mat3D {
        Mat3D::translation(planet_position.x, planet_position.y, planet_position.z).mul(
            Mat3D::rotation_y_degrees(planet_rotation + self.rel_rot(global_time)),
        )
    }
}

impl HexMesher for SkyMesher {
    fn get_height(&self, _position: G3dVec3) -> f32 {
        1.0
    }

    fn get_color(&self, _position: G3dVec3, out: &mut G3dColor) {
        *out = self.color;
    }

    fn skip(&self, position: G3dVec3) -> bool {
        simplex_noise3d(
            7 + self.seed,
            self.octaves,
            self.persistence,
            self.scale,
            position.x,
            position.y * 3.0,
            position.z,
        ) >= self.threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_sky_mesh_uses_cloud_shader_radius_as_height_intensity_and_speed() {
        let planet = PlanetMeta::new("serpulo", 3.0);
        let mesh = HexSkyMesh::new(
            &planet,
            10,
            2.0,
            0.4,
            0,
            G3dColor::rgb(0.7, 0.8, 1.0),
            3,
            0.5,
            1.2,
            0.6,
        );

        assert_eq!(mesh.base.shader, ShaderId::Clouds);
        assert_eq!(mesh.base.mesh.radius, 3.0);
        assert_eq!(mesh.base.mesh.intensity, 0.4);
        assert_eq!(mesh.rel_rot(80.0), 4.0);
    }

    #[test]
    fn hex_sky_mesh_skips_current_planet_when_cloud_alpha_is_zero() {
        let planet = PlanetMeta::new("serpulo", 3.0);
        let mesh = HexSkyMesh::new(&planet, 10, 2.0, 0.4, 0, G3dColor::WHITE, 3, 0.5, 1.2, 0.6);

        assert!(!mesh.should_render(&PlanetParams::new("serpulo")));
        assert!(mesh.should_render(&PlanetParams::new("serpulo").with_ui_alpha(0.5)));
        assert!(mesh.should_render(&PlanetParams::new("erekir")));
        assert_eq!(
            mesh.cloud_alpha(&PlanetParams::new("serpulo").with_ui_alpha(0.25)),
            0.75
        );
    }

    #[test]
    fn hex_sky_pre_render_sets_cloud_shader_state_like_java() {
        let planet = PlanetMeta::new("serpulo", 3.0);
        let mesh = HexSkyMesh::new(&planet, 10, 2.0, 0.4, 0, G3dColor::WHITE, 3, 0.5, 1.2, 0.6);

        let plan = mesh.pre_render(
            &PlanetParams::new("serpulo").with_ui_alpha(0.25),
            HexSkyPreRenderContext {
                planet_position: G3dVec3::new(0.0, 0.0, 0.0),
                solar_system_position: G3dVec3::new(0.0, 0.0, 2.0),
                rotation: 0.0,
                solar_system_light_color: G3dColor::rgb(0.2, 0.3, 0.4),
                global_time: 80.0,
            },
        );

        assert_eq!(plan.shader, ShaderId::Clouds);
        assert_eq!(plan.alpha, 0.75);
        assert_eq!(plan.ambient_color, G3dColor::rgb(0.2, 0.3, 0.4));
    }
}
