//! Noise-based hex mesh variants mirroring upstream `mindustry.graphics.g3d.NoiseMesh`.

use crate::mindustry::graphics::shaders::ShaderId;
use crate::mindustry::r#type::PlanetMeta;

use super::{simplex_noise3d, G3dColor, G3dVec3, HexMesh, HexMesher, MeshBuilder};

#[derive(Debug, Clone, PartialEq)]
pub struct NoiseMesh {
    pub base: HexMesh,
    pub seed: i32,
    pub variant: NoiseMeshVariant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoiseMeshVariant {
    SingleColor {
        color: G3dColor,
    },
    TwoColor {
        color1: G3dColor,
        color2: G3dColor,
        color_octaves: i32,
        color_persistence: f32,
        color_scale: f32,
        color_threshold: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct NoiseMesher {
    seed: i32,
    octaves: i32,
    persistence: f32,
    scale: f32,
    mag: f32,
    variant: NoiseMesherVariant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum NoiseMesherVariant {
    SingleColor {
        color: G3dColor,
    },
    TwoColor {
        color1: G3dColor,
        color2: G3dColor,
        color_octaves: i32,
        color_persistence: f32,
        color_scale: f32,
        color_threshold: f32,
    },
}

impl NoiseMesh {
    pub fn new(
        planet: &PlanetMeta,
        seed: i32,
        divisions: i32,
        color: G3dColor,
        radius: f32,
        octaves: i32,
        persistence: f32,
        scale: f32,
        mag: f32,
    ) -> Self {
        let mesher = NoiseMesher {
            seed,
            octaves,
            persistence,
            scale,
            mag,
            variant: NoiseMesherVariant::SingleColor { color },
        };
        Self {
            base: HexMesh::from_planet_mesh(
                planet,
                MeshBuilder::build_hex(&mesher, divisions, radius, 0.2),
                ShaderId::Planet,
            ),
            seed,
            variant: NoiseMeshVariant::SingleColor { color },
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn two_color(
        planet: &PlanetMeta,
        seed: i32,
        divisions: i32,
        radius: f32,
        octaves: i32,
        persistence: f32,
        scale: f32,
        mag: f32,
        color1: G3dColor,
        color2: G3dColor,
        color_octaves: i32,
        color_persistence: f32,
        color_scale: f32,
        color_threshold: f32,
    ) -> Self {
        let mesher = NoiseMesher {
            seed,
            octaves,
            persistence,
            scale,
            mag,
            variant: NoiseMesherVariant::TwoColor {
                color1,
                color2,
                color_octaves,
                color_persistence,
                color_scale,
                color_threshold,
            },
        };
        Self {
            base: HexMesh::from_planet_mesh(
                planet,
                MeshBuilder::build_hex(&mesher, divisions, radius, 0.2),
                ShaderId::Planet,
            ),
            seed,
            variant: NoiseMeshVariant::TwoColor {
                color1,
                color2,
                color_octaves,
                color_persistence,
                color_scale,
                color_threshold,
            },
        }
    }
}

impl HexMesher for NoiseMesher {
    fn get_height(&self, position: G3dVec3) -> f32 {
        simplex_noise3d(
            7 + self.seed,
            self.octaves,
            self.persistence,
            self.scale,
            5.0 + position.x,
            5.0 + position.y,
            5.0 + position.z,
        ) * self.mag
    }

    fn get_color(&self, position: G3dVec3, out: &mut G3dColor) {
        match self.variant {
            NoiseMesherVariant::SingleColor { color } => *out = color,
            NoiseMesherVariant::TwoColor {
                color1,
                color2,
                color_octaves,
                color_persistence,
                color_scale,
                color_threshold,
            } => {
                let value = simplex_noise3d(
                    8 + self.seed,
                    color_octaves,
                    color_persistence,
                    color_scale,
                    5.0 + position.x,
                    5.0 + position.y,
                    5.0 + position.z,
                );
                *out = if value > color_threshold {
                    color2
                } else {
                    color1
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_mesh_single_color_uses_planet_shader_and_radius_argument() {
        let planet = PlanetMeta::new("asteroid", 1.0);
        let mesh = NoiseMesh::new(
            &planet,
            4,
            0,
            G3dColor::rgb(0.1, 0.2, 0.3),
            2.0,
            3,
            0.5,
            1.2,
            0.8,
        );

        assert_eq!(mesh.base.planet, "asteroid");
        assert_eq!(mesh.base.shader, ShaderId::Planet);
        assert_eq!(mesh.base.mesh.radius, 2.0);
        assert_eq!(mesh.base.mesh.intensity, 0.2);
        assert_eq!(
            mesh.variant,
            NoiseMeshVariant::SingleColor {
                color: G3dColor::rgb(0.1, 0.2, 0.3)
            }
        );
    }

    #[test]
    fn noise_mesh_two_color_keeps_color_noise_parameters() {
        let planet = PlanetMeta::new("moon", 1.0);
        let mesh = NoiseMesh::two_color(
            &planet,
            6,
            0,
            2.0,
            3,
            0.5,
            1.2,
            0.8,
            G3dColor::rgb(0.1, 0.2, 0.3),
            G3dColor::rgb(0.4, 0.5, 0.6),
            2,
            0.7,
            1.8,
            0.45,
        );

        assert_eq!(mesh.seed, 6);
        assert_eq!(
            mesh.variant,
            NoiseMeshVariant::TwoColor {
                color1: G3dColor::rgb(0.1, 0.2, 0.3),
                color2: G3dColor::rgb(0.4, 0.5, 0.6),
                color_octaves: 2,
                color_persistence: 0.7,
                color_scale: 1.8,
                color_threshold: 0.45,
            }
        );
    }
}
