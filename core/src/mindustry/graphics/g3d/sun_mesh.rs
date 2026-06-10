//! Sun hex mesh variant mirroring upstream `mindustry.graphics.g3d.SunMesh`.

use crate::mindustry::graphics::shaders::ShaderId;
use crate::mindustry::r#type::PlanetMeta;

use super::{simplex_noise3d, G3dColor, G3dVec3, HexMesh, HexMesher};

#[derive(Debug, Clone, PartialEq)]
pub struct SunMesh {
    pub base: HexMesh,
    pub octaves: f64,
    pub persistence: f64,
    pub scale: f64,
    pub pow: f64,
    pub mag: f64,
    pub color_scale: f32,
    pub colors: Vec<G3dColor>,
}

#[derive(Debug, Clone, PartialEq)]
struct SunMesher {
    octaves: f64,
    persistence: f64,
    scale: f64,
    pow: f64,
    mag: f64,
    color_scale: f32,
    colors: Vec<G3dColor>,
}

impl SunMesh {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        planet: &PlanetMeta,
        divisions: i32,
        octaves: f64,
        persistence: f64,
        scale: f64,
        pow: f64,
        mag: f64,
        color_scale: f32,
        colors: Vec<G3dColor>,
    ) -> Self {
        let mesher = SunMesher {
            octaves,
            persistence,
            scale,
            pow,
            mag,
            color_scale,
            colors: colors.clone(),
        };
        Self {
            base: HexMesh::with_mesher(planet, &mesher, divisions, ShaderId::Unlit),
            octaves,
            persistence,
            scale,
            pow,
            mag,
            color_scale,
            colors,
        }
    }
}

impl HexMesher for SunMesher {
    fn get_height(&self, _position: G3dVec3) -> f32 {
        0.0
    }

    fn get_color(&self, position: G3dVec3, out: &mut G3dColor) {
        let height = (simplex_noise3d(
            0,
            self.octaves,
            self.persistence,
            self.scale,
            position.x,
            position.y,
            position.z,
        ) as f64)
            .powf(self.pow)
            * self.mag;
        let index = ((height * self.colors.len() as f64) as i32)
            .clamp(0, self.colors.len() as i32 - 1) as usize;
        *out = self.colors[index].mul(self.color_scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_mesh_uses_unlit_shader_and_zero_height_mesher() {
        let planet = PlanetMeta::new("sun", 5.0);
        let colors = vec![G3dColor::rgb(1.0, 0.4, 0.1), G3dColor::rgb(1.0, 0.8, 0.2)];

        let mesh = SunMesh::new(&planet, 0, 2.0, 0.5, 1.0, 2.0, 1.2, 0.8, colors.clone());

        assert_eq!(mesh.base.planet, "sun");
        assert_eq!(mesh.base.shader, ShaderId::Unlit);
        assert_eq!(mesh.colors, colors);
        assert_eq!(mesh.color_scale, 0.8);
        assert_eq!(mesh.base.mesh.intensity, 0.2);
    }
}
