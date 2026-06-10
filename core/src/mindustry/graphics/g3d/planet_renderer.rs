//! Backend-neutral scene plan mirroring upstream `mindustry.graphics.g3d.PlanetRenderer`.

use crate::mindustry::graphics::cubemap_mesh::{CubemapMesh, CubemapRenderPlan};

use super::Mat3D;

pub const PLANET_RENDERER_OUTLINE_RAD: f32 = 1.17;
pub const PLANET_RENDERER_CAM_LENGTH: f32 = 4.0;
pub const PLANET_RENDERER_DEFAULT_FOV: f32 = 60.0;
pub const PLANET_RENDERER_DEFAULT_FAR: f32 = 150.0;
pub const PLANET_RENDERER_PROJECTOR_SCALING: f32 = 1.0 / 150.0;

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetRenderNode {
    pub name: String,
    pub position: [f32; 3],
    pub rotation: f32,
    pub radius: f32,
    pub cam_radius: f32,
    pub clip_radius: f32,
    pub orbit_radius: f32,
    pub visible: bool,
    pub has_grid: bool,
    pub has_atmosphere: bool,
    pub draw_orbit: bool,
    pub children: Vec<PlanetRenderNode>,
}

impl PlanetRenderNode {
    pub fn new(name: impl Into<String>, radius: f32) -> Self {
        Self {
            name: name.into(),
            position: [0.0, 0.0, 0.0],
            rotation: 0.0,
            radius,
            cam_radius: 0.0,
            clip_radius: radius,
            orbit_radius: 0.0,
            visible: true,
            has_grid: false,
            has_atmosphere: false,
            draw_orbit: true,
            children: Vec::new(),
        }
    }

    pub fn at(mut self, position: [f32; 3]) -> Self {
        self.position = position;
        self
    }

    pub fn cam_radius(mut self, cam_radius: f32) -> Self {
        self.cam_radius = cam_radius;
        self
    }

    pub fn orbit_radius(mut self, orbit_radius: f32) -> Self {
        self.orbit_radius = orbit_radius;
        self
    }

    pub fn grid(mut self, has_grid: bool) -> Self {
        self.has_grid = has_grid;
        self
    }

    pub fn atmosphere(mut self, has_atmosphere: bool) -> Self {
        self.has_atmosphere = has_atmosphere;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn child(mut self, child: PlanetRenderNode) -> Self {
        self.children.push(child);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetRendererParams {
    pub planet: String,
    pub solar_system: PlanetRenderNode,
    pub view_w: i32,
    pub view_h: i32,
    pub graphics_width: i32,
    pub graphics_height: i32,
    pub zoom: f32,
    pub cam_pos: [f32; 3],
    pub other_cam_pos: Option<[f32; 3]>,
    pub other_cam_alpha: f32,
    pub ui_alpha: f32,
    pub draw_skybox: bool,
    pub draw_ui: bool,
    pub always_draw_atmosphere: bool,
    pub atmosphere_enabled: bool,
    pub has_interface_renderer: bool,
}

impl PlanetRendererParams {
    pub fn new(planet: impl Into<String>, solar_system: PlanetRenderNode) -> Self {
        Self {
            planet: planet.into(),
            solar_system,
            view_w: 0,
            view_h: 0,
            graphics_width: 0,
            graphics_height: 0,
            zoom: 1.0,
            cam_pos: [0.0, 0.0, 1.0],
            other_cam_pos: None,
            other_cam_alpha: 0.0,
            ui_alpha: 1.0,
            draw_skybox: true,
            draw_ui: true,
            always_draw_atmosphere: false,
            atmosphere_enabled: true,
            has_interface_renderer: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanetCameraPlan {
    pub width: i32,
    pub height: i32,
    pub fov: f32,
    pub far: f32,
    pub position: [f32; 3],
    pub look_at: [f32; 3],
    pub up: [f32; 3],
    pub direction: [f32; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlanetSceneStep {
    FlushDraw,
    ClearDepth,
    EnableDepthTest,
    DepthMask(bool),
    EnableBackfaceCulling,
    UniverseDrawBegin,
    BloomCapture {
        width: i32,
        height: i32,
        blending: bool,
    },
    DrawSkybox(CubemapRenderPlan),
    UniverseDraw,
    DrawPlanet {
        planet: String,
        transform: Mat3D,
    },
    DrawClouds {
        planet: String,
        transform: Mat3D,
    },
    DrawSectors {
        planet: String,
    },
    DrawAtmosphere {
        planet: String,
    },
    DrawOrbit {
        planet: String,
        points: i32,
        alpha: f32,
    },
    RenderOverProjections {
        planet: String,
    },
    BloomRender,
    EnableBlend,
    RenderProjections {
        planet: String,
    },
    DisableBackfaceCulling,
    DisableDepthTest,
    CameraUpdate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetScenePlan {
    pub camera: PlanetCameraPlan,
    pub projector_scaling: f32,
    pub steps: Vec<PlanetSceneStep>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetRenderer {
    pub skybox: CubemapMesh,
    pub fov: f32,
    pub far: f32,
    pub projector_scaling: f32,
}

impl Default for PlanetRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanetRenderer {
    pub fn new() -> Self {
        Self {
            skybox: CubemapMesh::new("cubemaps/stars/"),
            fov: PLANET_RENDERER_DEFAULT_FOV,
            far: PLANET_RENDERER_DEFAULT_FAR,
            projector_scaling: PLANET_RENDERER_PROJECTOR_SCALING,
        }
    }

    pub fn render(&self, params: &PlanetRendererParams) -> PlanetScenePlan {
        let width = if params.view_w <= 0 {
            params.graphics_width
        } else {
            params.view_w
        };
        let height = if params.view_h <= 0 {
            params.graphics_height
        } else {
            params.view_h
        };
        let selected = find_planet(&params.solar_system, &params.planet).unwrap();
        let cam_distance = (selected.radius + selected.cam_radius) * PLANET_RENDERER_CAM_LENGTH
            + (params.zoom - 1.0) * (selected.radius + selected.cam_radius) * 2.0;
        let cam_offset = set_length(params.cam_pos, cam_distance);
        let base = if let Some(other) = params.other_cam_pos {
            lerp(other, selected.position, params.other_cam_alpha)
        } else {
            selected.position
        };
        let camera_position = add(base, cam_offset);
        let direction = normalize(sub(selected.position, camera_position));
        let camera = PlanetCameraPlan {
            width,
            height,
            fov: self.fov,
            far: self.far,
            position: camera_position,
            look_at: selected.position,
            up: [0.0, 1.0, 0.0],
            direction,
        };

        let mut steps = vec![
            PlanetSceneStep::FlushDraw,
            PlanetSceneStep::ClearDepth,
            PlanetSceneStep::EnableDepthTest,
            PlanetSceneStep::DepthMask(true),
            PlanetSceneStep::EnableBackfaceCulling,
            PlanetSceneStep::UniverseDrawBegin,
            PlanetSceneStep::BloomCapture {
                width,
                height,
                blending: !params.draw_skybox,
            },
        ];

        if params.draw_skybox {
            steps.push(PlanetSceneStep::DepthMask(false));
            steps.push(PlanetSceneStep::DrawSkybox(
                self.skybox.render(&Mat3D::identity()),
            ));
            steps.push(PlanetSceneStep::DepthMask(true));
        }

        steps.push(PlanetSceneStep::UniverseDraw);
        render_planet_recursive(&params.solar_system, &mut steps);
        render_transparent_recursive(&params.solar_system, params, false, &mut steps);

        if params.has_interface_renderer {
            steps.push(PlanetSceneStep::RenderOverProjections {
                planet: params.planet.clone(),
            });
        }

        steps.push(PlanetSceneStep::BloomRender);
        steps.push(PlanetSceneStep::EnableBlend);

        if params.has_interface_renderer {
            steps.push(PlanetSceneStep::RenderProjections {
                planet: params.planet.clone(),
            });
        }

        steps.push(PlanetSceneStep::DisableBackfaceCulling);
        steps.push(PlanetSceneStep::DisableDepthTest);
        steps.push(PlanetSceneStep::CameraUpdate);

        PlanetScenePlan {
            camera,
            projector_scaling: self.projector_scaling,
            steps,
        }
    }
}

fn render_planet_recursive(planet: &PlanetRenderNode, steps: &mut Vec<PlanetSceneStep>) {
    if !planet.visible {
        return;
    }

    steps.push(PlanetSceneStep::DrawPlanet {
        planet: planet.name.clone(),
        transform: planet_transform(planet),
    });

    for child in &planet.children {
        render_planet_recursive(child, steps);
    }
}

fn render_transparent_recursive(
    planet: &PlanetRenderNode,
    params: &PlanetRendererParams,
    has_parent: bool,
    steps: &mut Vec<PlanetSceneStep>,
) {
    if !planet.visible {
        return;
    }

    steps.push(PlanetSceneStep::DrawClouds {
        planet: planet.name.clone(),
        transform: planet_transform(planet),
    });

    if planet.has_grid && planet.name == params.planet && params.draw_ui && params.ui_alpha > 0.02 {
        steps.push(PlanetSceneStep::DrawSectors {
            planet: planet.name.clone(),
        });
    }

    if has_parent
        && planet.has_atmosphere
        && (params.always_draw_atmosphere || params.atmosphere_enabled)
    {
        steps.push(PlanetSceneStep::DrawAtmosphere {
            planet: planet.name.clone(),
        });
    }

    for child in &planet.children {
        render_transparent_recursive(child, params, true, steps);
    }

    if params.draw_ui && has_parent && planet.draw_orbit && params.ui_alpha > 0.02 {
        steps.push(PlanetSceneStep::DrawOrbit {
            planet: planet.name.clone(),
            points: (planet.orbit_radius * 10.0) as i32,
            alpha: params.ui_alpha,
        });
    }
}

fn planet_transform(planet: &PlanetRenderNode) -> Mat3D {
    Mat3D::translation(planet.position[0], planet.position[1], planet.position[2])
}

fn find_planet<'a>(planet: &'a PlanetRenderNode, name: &str) -> Option<&'a PlanetRenderNode> {
    if planet.name == name {
        return Some(planet);
    }
    planet
        .children
        .iter()
        .find_map(|child| find_planet(child, name))
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn lerp(a: [f32; 3], b: [f32; 3], alpha: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * alpha,
        a[1] + (b[1] - a[1]) * alpha,
        a[2] + (b[2] - a[2]) * alpha,
    ]
}

fn len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let length = len(v);
    [v[0] / length, v[1] / length, v[2] / length]
}

fn set_length(v: [f32; 3], length: f32) -> [f32; 3] {
    let normalized = normalize(v);
    [
        normalized[0] * length,
        normalized[1] * length,
        normalized[2] * length,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solar_system() -> PlanetRenderNode {
        PlanetRenderNode::new("sun", 10.0).child(
            PlanetRenderNode::new("serpulo", 3.0)
                .at([1.0, 2.0, 3.0])
                .cam_radius(0.5)
                .orbit_radius(12.0)
                .grid(true)
                .atmosphere(true),
        )
    }

    #[test]
    fn planet_renderer_initializes_camera_and_projector_like_upstream() {
        let renderer = PlanetRenderer::new();

        assert_eq!(renderer.fov, PLANET_RENDERER_DEFAULT_FOV);
        assert_eq!(renderer.far, PLANET_RENDERER_DEFAULT_FAR);
        assert_eq!(
            renderer.projector_scaling,
            PLANET_RENDERER_PROJECTOR_SCALING
        );
        assert_eq!(renderer.skybox.cubemap, "cubemaps/stars/");
    }

    #[test]
    fn planet_renderer_scene_plan_matches_upstream_stage_order_with_skybox() {
        let renderer = PlanetRenderer::new();
        let params = PlanetRendererParams {
            graphics_width: 1920,
            graphics_height: 1080,
            ui_alpha: 0.6,
            has_interface_renderer: true,
            ..PlanetRendererParams::new("serpulo", solar_system())
        };

        let plan = renderer.render(&params);

        assert_eq!(plan.camera.width, 1920);
        assert_eq!(plan.camera.height, 1080);
        assert_eq!(plan.camera.position, [1.0, 2.0, 17.0]);
        assert_eq!(plan.camera.look_at, [1.0, 2.0, 3.0]);
        assert_eq!(plan.projector_scaling, 1.0 / 150.0);
        assert_eq!(
            &plan.steps[..10],
            &[
                PlanetSceneStep::FlushDraw,
                PlanetSceneStep::ClearDepth,
                PlanetSceneStep::EnableDepthTest,
                PlanetSceneStep::DepthMask(true),
                PlanetSceneStep::EnableBackfaceCulling,
                PlanetSceneStep::UniverseDrawBegin,
                PlanetSceneStep::BloomCapture {
                    width: 1920,
                    height: 1080,
                    blending: false,
                },
                PlanetSceneStep::DepthMask(false),
                PlanetSceneStep::DrawSkybox(renderer.skybox.render(&Mat3D::identity())),
                PlanetSceneStep::DepthMask(true),
            ]
        );
        assert!(plan.steps.contains(&PlanetSceneStep::DrawPlanet {
            planet: "sun".into(),
            transform: Mat3D::identity(),
        }));
        assert!(plan.steps.contains(&PlanetSceneStep::DrawSectors {
            planet: "serpulo".into(),
        }));
        assert!(plan.steps.contains(&PlanetSceneStep::DrawAtmosphere {
            planet: "serpulo".into(),
        }));
        assert!(plan.steps.contains(&PlanetSceneStep::DrawOrbit {
            planet: "serpulo".into(),
            points: 120,
            alpha: 0.6,
        }));
        assert_eq!(
            &plan.steps[plan.steps.len() - 7..],
            &[
                PlanetSceneStep::RenderOverProjections {
                    planet: "serpulo".into(),
                },
                PlanetSceneStep::BloomRender,
                PlanetSceneStep::EnableBlend,
                PlanetSceneStep::RenderProjections {
                    planet: "serpulo".into(),
                },
                PlanetSceneStep::DisableBackfaceCulling,
                PlanetSceneStep::DisableDepthTest,
                PlanetSceneStep::CameraUpdate,
            ]
        );
    }

    #[test]
    fn planet_renderer_disables_skybox_and_sectors_from_params() {
        let renderer = PlanetRenderer::new();
        let params = PlanetRendererParams {
            view_w: 800,
            view_h: 600,
            graphics_width: 1920,
            graphics_height: 1080,
            draw_skybox: false,
            draw_ui: false,
            ..PlanetRendererParams::new("serpulo", solar_system())
        };

        let plan = renderer.render(&params);

        assert_eq!(plan.camera.width, 800);
        assert_eq!(plan.camera.height, 600);
        assert!(plan.steps.contains(&PlanetSceneStep::BloomCapture {
            width: 800,
            height: 600,
            blending: true,
        }));
        assert!(!plan
            .steps
            .iter()
            .any(|step| matches!(step, PlanetSceneStep::DrawSkybox(_))));
        assert!(!plan
            .steps
            .iter()
            .any(|step| matches!(step, PlanetSceneStep::DrawSectors { .. })));
        assert!(!plan
            .steps
            .iter()
            .any(|step| matches!(step, PlanetSceneStep::DrawOrbit { .. })));
    }
}
