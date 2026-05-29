//! Data-only port of upstream `ParticleRenderer`.
//!
//! The Java class owns an OpenGL point-sprite mesh and may update particles on
//! a background executor.  This Rust core module keeps the queue/update/vertex
//! math deterministic and backend-neutral; render backends can upload
//! `ParticleVertex` values to their preferred GPU abstraction.

use crate::mindustry::world::draw::{
    DrawBlockParticleBlendMode, DrawBlockParticleConfig, DrawBlockParticleRenderKind,
    DrawBlockParticleSizeInterp,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleCamera {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ParticleCamera {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    pub time: f32,
    pub lifetime: f32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size_from: f32,
    pub size_to: f32,
    pub color: f32,
}

impl Particle {
    pub fn new(
        x: f32,
        y: f32,
        lifetime: f32,
        vx: f32,
        vy: f32,
        size_from_radius: f32,
        size_to_radius: f32,
        color: f32,
    ) -> Self {
        Self {
            time: 0.0,
            lifetime,
            x,
            y,
            vx,
            vy,
            size_from: size_from_radius * 2.0,
            size_to: size_to_radius * 2.0,
            color,
        }
    }

    pub fn size(&self) -> f32 {
        let alpha = if self.lifetime <= 0.0 {
            1.0
        } else {
            (self.time / self.lifetime).clamp(0.0, 1.0)
        };
        self.size_from + (self.size_to - self.size_from) * alpha
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleVertex {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub color: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ParticleColor {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockDrawerParticlePlanConfig {
    pub build_id_seed: i32,
    pub warmup: f32,
    pub time: f32,
    pub x: f32,
    pub y: f32,
    pub alpha: f32,
    pub sides: usize,
    pub particle_count: usize,
    pub particle_rotation: f32,
    pub particle_life: f32,
    pub particle_radius: f32,
    pub particle_size: f32,
    pub fade_margin: f32,
    pub rotate_scl: f32,
    pub reverse: bool,
    pub random_life_range: f32,
    pub invert_life: bool,
    pub size_interp: DrawBlockParticleSizeInterp,
    pub blend_mode: DrawBlockParticleBlendMode,
    pub render_kind: DrawBlockParticleRenderKind,
    pub layer: f32,
    pub color: ParticleColor,
    pub secondary_color: Option<ParticleColor>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockDrawerParticleSample {
    pub fin: f32,
    pub fout: f32,
    pub alpha: f32,
    pub angle: f32,
    pub length: f32,
    pub size: f32,
    pub color_t: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockDrawerParticlePlan {
    pub build_id_seed: i32,
    pub warmup: f32,
    pub time: f32,
    pub x: f32,
    pub y: f32,
    pub alpha: f32,
    pub sides: usize,
    pub particle_count: usize,
    pub particle_rotation: f32,
    pub particle_life: f32,
    pub particle_radius: f32,
    pub particle_size: f32,
    pub fade_margin: f32,
    pub rotate_scl: f32,
    pub reverse: bool,
    pub random_life_range: f32,
    pub invert_life: bool,
    pub size_interp: DrawBlockParticleSizeInterp,
    pub blend_mode: DrawBlockParticleBlendMode,
    pub render_kind: DrawBlockParticleRenderKind,
    pub layer: f32,
    pub color: ParticleColor,
    pub secondary_color: Option<ParticleColor>,
}

impl BlockDrawerParticlePlan {
    pub const fn new(config: BlockDrawerParticlePlanConfig) -> Self {
        Self {
            build_id_seed: config.build_id_seed,
            warmup: config.warmup,
            time: config.time,
            x: config.x,
            y: config.y,
            alpha: config.alpha,
            sides: config.sides,
            particle_count: config.particle_count,
            particle_rotation: config.particle_rotation,
            particle_life: config.particle_life,
            particle_radius: config.particle_radius,
            particle_size: config.particle_size,
            fade_margin: config.fade_margin,
            rotate_scl: config.rotate_scl,
            reverse: config.reverse,
            random_life_range: config.random_life_range,
            invert_life: config.invert_life,
            size_interp: config.size_interp,
            blend_mode: config.blend_mode,
            render_kind: config.render_kind,
            layer: config.layer,
            color: config.color,
            secondary_color: config.secondary_color,
        }
    }

    pub fn is_noop(&self) -> bool {
        self.warmup <= 0.0 || self.alpha <= 0.0 || self.particle_count == 0
    }

    pub fn effective_alpha(&self) -> f32 {
        self.alpha * self.warmup
    }

    pub fn base_time(&self) -> f32 {
        if self.particle_life == 0.0 {
            0.0
        } else {
            self.time / self.particle_life
        }
    }

    pub fn particle_seed(&self, index: usize) -> u64 {
        seeded_u64(self.build_id_seed, self.time, index)
    }

    pub fn particle_fin(&self, random_life: f32) -> Option<f32> {
        if self.warmup <= 0.0 || self.particle_life == 0.0 {
            return None;
        }

        let mut fin = (random_life * self.random_life_range.max(0.0) + self.base_time()) % 1.0;
        if self.invert_life {
            fin = 1.0 - fin;
        }
        if self.reverse {
            fin = 1.0 - fin;
        }
        Some(fin)
    }

    pub fn particle_fout(&self, random_life: f32) -> Option<f32> {
        self.particle_fin(random_life).map(|fin| 1.0 - fin)
    }

    pub fn particle_angle(&self, random_angle: f32) -> Option<f32> {
        if self.warmup <= 0.0 || self.rotate_scl == 0.0 {
            return None;
        }

        Some(random_angle + (self.time / self.rotate_scl) % 360.0)
    }

    pub fn particle_alpha(&self, fin: f32) -> f32 {
        self.effective_alpha() * (1.0 - curve(fin, 1.0 - self.fade_margin, 1.0))
    }

    pub fn particle_length(&self, fout: f32) -> f32 {
        self.particle_radius * fout.clamp(0.0, 1.0).powf(1.5)
    }

    pub fn particle_size_regular(&self, fin: f32) -> f32 {
        self.particle_size * slope(fin) * self.warmup
    }

    pub fn particle_size_soft(&self, fin: f32) -> f32 {
        self.particle_size * fin.clamp(0.0, 1.0) * self.warmup * 2.0
    }

    pub fn particle_size_for_fin(&self, fin: f32) -> f32 {
        match self.size_interp {
            DrawBlockParticleSizeInterp::Slope => self.particle_size_regular(fin),
            DrawBlockParticleSizeInterp::One => self.particle_size * self.warmup,
            DrawBlockParticleSizeInterp::Linear => self.particle_size_soft(fin),
        }
    }

    pub fn sample_for_index(&self, index: usize) -> Option<BlockDrawerParticleSample> {
        if index >= self.particle_count {
            return None;
        }

        let seed = self.particle_seed(index);
        let random_life = seed01(seed ^ 0xA53A_9E37_79B9_7F4A);
        let random_angle = seed01(seed ^ 0xC3A5_C85C_97CB_3127) * 360.0;
        let random_color = seed01(seed ^ 0x9E37_79B9_7F4A_7C15);

        let fin = self.particle_fin(random_life)?;
        let fout = 1.0 - fin;
        let angle = self.particle_angle(random_angle)?;
        let alpha = self.particle_alpha(fin);
        let length = self.particle_length(fout);
        let size = self.particle_size_for_fin(fin);

        Some(BlockDrawerParticleSample {
            fin,
            fout,
            alpha,
            angle,
            length,
            size,
            color_t: self.secondary_color.map(|_| random_color),
        })
    }
}

impl BlockDrawerParticlePlanConfig {
    pub fn from_draw_config(
        config: DrawBlockParticleConfig,
        build_id_seed: i32,
        warmup: f32,
        time: f32,
        layer: f32,
    ) -> Self {
        Self {
            build_id_seed,
            warmup,
            time,
            x: config.x,
            y: config.y,
            alpha: config.alpha,
            sides: config.sides,
            particle_count: config.particle_count,
            particle_rotation: config.particle_rotation,
            particle_life: config.particle_life,
            particle_radius: config.particle_radius,
            particle_size: config.particle_size,
            fade_margin: config.fade_margin,
            rotate_scl: config.rotate_scl,
            reverse: config.reverse,
            random_life_range: config.random_life_range,
            invert_life: config.invert_life,
            size_interp: config.size_interp,
            blend_mode: config.blend_mode,
            render_kind: config.render_kind,
            layer,
            color: rgba_to_particle_color(config.color_rgba),
            secondary_color: config.secondary_color_rgba.map(rgba_to_particle_color),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleRenderPlan {
    pub vertices: Vec<ParticleVertex>,
    pub shader_name: &'static str,
    pub point_size_enabled: bool,
    pub camera_scaling: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleRendererState {
    pub particles: Vec<Particle>,
    pub pending: Vec<Particle>,
    pub vertex_buffer: Vec<ParticleVertex>,
    pub max_particles: usize,
    pub max_particles_per_frame: usize,
    pub global_drag: f32,
    pub cull_padding: f32,
}

impl Default for ParticleRendererState {
    fn default() -> Self {
        Self {
            particles: Vec::new(),
            pending: Vec::new(),
            vertex_buffer: Vec::new(),
            max_particles: Self::MAX_PARTICLES,
            max_particles_per_frame: Self::MAX_PARTICLES_PER_FRAME,
            global_drag: Self::GLOBAL_DRAG,
            cull_padding: Self::CULL_PADDING,
        }
    }
}

impl ParticleRendererState {
    pub const MAX_PARTICLES: usize = 100_000;
    pub const MAX_PARTICLES_PER_FRAME: usize = 25_000;
    pub const PARTICLE_SIZE: usize = 9;
    pub const PARTICLE_VERTEX_SIZE: usize = 4;
    pub const GLOBAL_DRAG: f32 = 0.05;
    pub const CULL_PADDING: f32 = 8.0 * 3.0;
    pub const SHADER_NAME: &'static str = "particle-point-sprite";

    pub fn count(&self) -> usize {
        self.particles.len()
    }

    pub fn add(
        &mut self,
        camera: ParticleCamera,
        x: f32,
        y: f32,
        lifetime: f32,
        vx: f32,
        vy: f32,
        size_from: f32,
        size_to: f32,
        color: f32,
    ) -> bool {
        if self.pending.len() >= self.max_particles_per_frame {
            return false;
        }

        if !self.contains_with_padding(camera, x, y, size_from) {
            return false;
        }

        self.pending.push(Particle::new(
            x, y, lifetime, vx, vy, size_from, size_to, color,
        ));
        true
    }

    pub fn update(&mut self, delta: f32) {
        self.append_pending_like_java();
        update_particles(&mut self.particles, delta, self.global_drag);
        self.vertex_buffer = build_vertices(&self.particles);
    }

    pub fn render_plan(&self, camera: ParticleCamera, graphics_width: f32) -> ParticleRenderPlan {
        ParticleRenderPlan {
            vertices: self.vertex_buffer.clone(),
            shader_name: Self::SHADER_NAME,
            point_size_enabled: true,
            camera_scaling: if camera.width == 0.0 {
                0.0
            } else {
                graphics_width / camera.width
            },
        }
    }

    pub fn block_drawer_particle_plan(
        config: BlockDrawerParticlePlanConfig,
    ) -> BlockDrawerParticlePlan {
        BlockDrawerParticlePlan::new(config)
    }

    pub fn block_drawer_particle_plan_from_draw_config(
        config: DrawBlockParticleConfig,
        build_id_seed: i32,
        warmup: f32,
        time: f32,
        layer: f32,
    ) -> BlockDrawerParticlePlan {
        BlockDrawerParticlePlan::new(BlockDrawerParticlePlanConfig::from_draw_config(
            config,
            build_id_seed,
            warmup,
            time,
            layer,
        ))
    }

    pub fn block_drawer_particle_plans_from_drawer(
        block_name: &str,
        drawer: &str,
        build_id_seed: i32,
        warmup: f32,
        time: f32,
        layer: f32,
    ) -> Vec<BlockDrawerParticlePlan> {
        crate::mindustry::world::draw::draw_block_dispatch_particle_configs(block_name, drawer)
            .into_iter()
            .map(|config| {
                Self::block_drawer_particle_plan_from_draw_config(
                    config,
                    build_id_seed,
                    warmup,
                    time,
                    layer,
                )
            })
            .collect()
    }

    fn append_pending_like_java(&mut self) {
        let space = self.max_particles.saturating_sub(self.particles.len());
        let max_added = space.min(self.pending.len());
        if max_added == 0 {
            self.pending.clear();
            return;
        }

        // Java prioritizes particles at the end of addBuffer when overflowed.
        let start = self.pending.len() - max_added;
        self.particles.extend_from_slice(&self.pending[start..]);
        self.pending.clear();
    }

    fn contains_with_padding(
        &self,
        camera: ParticleCamera,
        x: f32,
        y: f32,
        size_from: f32,
    ) -> bool {
        let rect_x = camera.x - camera.width / 2.0 - size_from - self.cull_padding;
        let rect_y = camera.y - camera.height / 2.0 - size_from - self.cull_padding;
        let rect_w = camera.width + size_from * 2.0 + self.cull_padding * 2.0;
        let rect_h = camera.height + size_from * 2.0 + self.cull_padding * 2.0;
        x >= rect_x && x <= rect_x + rect_w && y >= rect_y && y <= rect_y + rect_h
    }
}

pub fn update_particles(particles: &mut Vec<Particle>, delta: f32, global_drag: f32) {
    let drag_value = (1.0 - global_drag * delta).max(0.0);
    let mut i = 0;
    while i < particles.len() {
        particles[i].time += delta;
        if particles[i].time >= particles[i].lifetime {
            particles.swap_remove(i);
        } else {
            particles[i].x += particles[i].vx * delta;
            particles[i].y += particles[i].vy * delta;
            particles[i].vx *= drag_value;
            particles[i].vy *= drag_value;
            i += 1;
        }
    }
}

pub fn build_vertices(particles: &[Particle]) -> Vec<ParticleVertex> {
    particles
        .iter()
        .map(|particle| ParticleVertex {
            x: particle.x,
            y: particle.y,
            size: particle.size(),
            color: particle.color,
        })
        .collect()
}

fn seeded_u64(build_id_seed: i32, time: f32, index: usize) -> u64 {
    let mut state = build_id_seed as u64
        ^ time.to_bits() as u64
        ^ (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    state = splitmix64(state);
    state
}

fn splitmix64(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut result = state;
    result = (result ^ (result >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    result = (result ^ (result >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    result ^ (result >> 31)
}

fn seed01(seed: u64) -> f32 {
    ((seed >> 40) as u32 as f32) / ((1u32 << 24) as f32)
}

fn slope(value: f32) -> f32 {
    1.0 - (value - 0.5).abs() * 2.0
}

fn curve(value: f32, start: f32, end: f32) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return if value >= end { 1.0 } else { 0.0 };
    }

    ((value - start) / (end - start)).clamp(0.0, 1.0)
}

fn rgba_to_particle_color(rgba: u32) -> ParticleColor {
    ParticleColor::new(
        ((rgba >> 24) & 0xff) as f32 / 255.0,
        ((rgba >> 16) & 0xff) as f32 / 255.0,
        ((rgba >> 8) & 0xff) as f32 / 255.0,
        (rgba & 0xff) as f32 / 255.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::world::draw::{
        draw_block_dispatch_icons, draw_block_dispatch_particle_configs, DrawBlockParticleConfig,
    };

    #[test]
    fn particle_constants_match_upstream_shape() {
        assert_eq!(ParticleRendererState::MAX_PARTICLES, 100_000);
        assert_eq!(ParticleRendererState::MAX_PARTICLES_PER_FRAME, 25_000);
        assert_eq!(ParticleRendererState::PARTICLE_SIZE, 9);
        assert_eq!(ParticleRendererState::PARTICLE_VERTEX_SIZE, 4);
        assert_eq!(ParticleRendererState::GLOBAL_DRAG, 0.05);
        assert_eq!(ParticleRendererState::CULL_PADDING, 24.0);
    }

    #[test]
    fn add_culls_outside_camera_and_stores_diameter_sizes() {
        let mut state = ParticleRendererState::default();
        let camera = ParticleCamera::new(0.0, 0.0, 100.0, 80.0);

        assert!(state.add(camera, 0.0, 0.0, 10.0, 1.0, 2.0, 3.0, 4.0, 7.0));
        assert!(!state.add(camera, 1000.0, 1000.0, 10.0, 1.0, 2.0, 3.0, 4.0, 7.0));

        assert_eq!(state.pending.len(), 1);
        assert_eq!(state.pending[0].size_from, 6.0);
        assert_eq!(state.pending[0].size_to, 8.0);
    }

    #[test]
    fn update_moves_drags_expires_and_builds_vertices() {
        let mut state = ParticleRendererState::default();
        let camera = ParticleCamera::new(0.0, 0.0, 100.0, 100.0);
        state.add(camera, 0.0, 0.0, 10.0, 10.0, 0.0, 1.0, 3.0, 1.0);
        state.add(camera, 1.0, 1.0, 0.5, 0.0, 10.0, 2.0, 2.0, 2.0);

        state.update(1.0);

        assert_eq!(state.count(), 1);
        assert_eq!(state.particles[0].x, 10.0);
        assert_eq!(state.particles[0].vx, 9.5);
        assert_eq!(state.vertex_buffer.len(), 1);
        assert_eq!(state.vertex_buffer[0].size, 2.4);
    }

    #[test]
    fn append_pending_prioritizes_most_recent_when_full() {
        let mut state = ParticleRendererState {
            max_particles: 2,
            ..Default::default()
        };
        let camera = ParticleCamera::new(0.0, 0.0, 1000.0, 1000.0);
        state.add(camera, 1.0, 0.0, 10.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        state.add(camera, 2.0, 0.0, 10.0, 0.0, 0.0, 1.0, 1.0, 2.0);
        state.add(camera, 3.0, 0.0, 10.0, 0.0, 0.0, 1.0, 1.0, 3.0);

        state.update(0.1);

        assert_eq!(state.particles.len(), 2);
        assert_eq!(state.particles[0].color, 2.0);
        assert_eq!(state.particles[1].color, 3.0);
    }

    #[test]
    fn block_drawer_particle_plan_is_deterministic_for_same_build_id() {
        let config = BlockDrawerParticlePlanConfig {
            build_id_seed: 42,
            warmup: 0.5,
            time: 70.0,
            x: 0.0,
            y: 0.0,
            alpha: 0.6,
            sides: 12,
            particle_count: 3,
            particle_rotation: 0.0,
            particle_life: 70.0,
            particle_radius: 7.0,
            particle_size: 3.0,
            fade_margin: 0.4,
            rotate_scl: 3.0,
            reverse: false,
            random_life_range: 1.0,
            invert_life: false,
            size_interp: DrawBlockParticleSizeInterp::Slope,
            blend_mode: DrawBlockParticleBlendMode::Normal,
            render_kind: DrawBlockParticleRenderKind::Circle,
            layer: 12.0,
            color: ParticleColor::new(0.2, 0.4, 0.6, 1.0),
            secondary_color: Some(ParticleColor::new(0.8, 0.1, 0.2, 0.9)),
        };

        let first = ParticleRendererState::block_drawer_particle_plan(config);
        let second = ParticleRendererState::block_drawer_particle_plan(config);

        assert_eq!(first, second);
        assert_eq!(first.sample_for_index(0), second.sample_for_index(0));
        assert_eq!(first.sample_for_index(1), second.sample_for_index(1));
    }

    #[test]
    fn block_drawer_particle_plan_from_draw_config_with_zero_warmup_is_noop() {
        let plan = ParticleRendererState::block_drawer_particle_plan_from_draw_config(
            DrawBlockParticleConfig {
                x: 0.0,
                y: 0.0,
                color_rgba: 0xff8040ff,
                secondary_color_rgba: None,
                alpha: 0.7,
                sides: 12,
                particle_count: 5,
                particle_rotation: 0.0,
                particle_life: 70.0,
                particle_radius: 7.0,
                particle_size: 3.0,
                fade_margin: 0.4,
                rotate_scl: 1.5,
                reverse: true,
                random_life_range: 2.0,
                invert_life: false,
                size_interp: DrawBlockParticleSizeInterp::Slope,
                blend_mode: DrawBlockParticleBlendMode::Normal,
                render_kind: DrawBlockParticleRenderKind::Circle,
            },
            7,
            0.0,
            10.0,
            9.0,
        );

        assert!(plan.is_noop());
        assert_eq!(plan.effective_alpha(), 0.0);
        assert_eq!(plan.sample_for_index(0), None);
    }

    #[test]
    fn block_drawer_particle_plans_from_drawer_collects_configs_and_is_deterministic() {
        assert_eq!(
            draw_block_dispatch_icons("surge-crucible", "DrawParticles"),
            Vec::<String>::new()
        );
        assert_eq!(
            draw_block_dispatch_icons("surge-crucible", "DrawSoftParticles"),
            Vec::<String>::new()
        );

        let configs = draw_block_dispatch_particle_configs(
            "surge-crucible",
            "DrawMulti(DrawParticles, DrawSoftParticles)",
        );
        assert_eq!(configs.len(), 2);
        assert_eq!(
            configs[0],
            crate::mindustry::world::draw::draw_particles_block_config()
        );
        assert_eq!(
            configs[1],
            crate::mindustry::world::draw::draw_soft_particles_block_config()
        );

        let first = ParticleRendererState::block_drawer_particle_plans_from_drawer(
            "surge-crucible",
            "DrawMulti(DrawParticles, DrawSoftParticles)",
            1,
            0.8,
            90.0,
            7.0,
        );
        let second = ParticleRendererState::block_drawer_particle_plans_from_drawer(
            "surge-crucible",
            "DrawMulti(DrawParticles, DrawSoftParticles)",
            1,
            0.8,
            90.0,
            7.0,
        );

        assert_eq!(first, second);
        assert_eq!(first.len(), 2);
        assert!(first[0].secondary_color.is_none());
        assert!(first[1].secondary_color.is_some());
        assert_eq!(first[0].sample_for_index(0), second[0].sample_for_index(0));
        assert_eq!(first[1].sample_for_index(0), second[1].sample_for_index(0));
    }

    #[test]
    fn block_drawer_particle_plan_preserves_java_draw_particle_shape_fields() {
        let mut config = crate::mindustry::world::draw::draw_particles_block_config();
        config.x = 1.25;
        config.y = -2.5;
        config.sides = 6;
        config.particle_rotation = 45.0;
        config.render_kind = DrawBlockParticleRenderKind::Polygon;
        config.blend_mode = DrawBlockParticleBlendMode::Normal;

        let plan = ParticleRendererState::block_drawer_particle_plan_from_draw_config(
            config, 17, 0.75, 33.0, 8.0,
        );

        assert_eq!(plan.x, 1.25);
        assert_eq!(plan.y, -2.5);
        assert_eq!(plan.sides, 6);
        assert_eq!(plan.particle_rotation, 45.0);
        assert_eq!(plan.render_kind, DrawBlockParticleRenderKind::Polygon);
        assert_eq!(plan.blend_mode, DrawBlockParticleBlendMode::Normal);
        assert_eq!(plan.random_life_range, 2.0);

        let sample = plan.sample_for_index(0).unwrap();
        assert_eq!(sample.color_t, None);
        assert!((sample.size - plan.particle_size_for_fin(sample.fin)).abs() < 0.00001);
    }

    #[test]
    fn block_drawer_soft_particle_plan_uses_java_soft_sprite_life_and_size_semantics() {
        let config = crate::mindustry::world::draw::draw_soft_particles_block_config();
        let plan = ParticleRendererState::block_drawer_particle_plan_from_draw_config(
            config, 17, 0.5, 21.0, 8.0,
        );

        assert_eq!(plan.render_kind, DrawBlockParticleRenderKind::SoftSprite);
        assert_eq!(plan.blend_mode, DrawBlockParticleBlendMode::Additive);
        assert_eq!(plan.size_interp, DrawBlockParticleSizeInterp::Linear);
        assert!(plan.invert_life);
        assert_eq!(plan.random_life_range, 1.0);

        let sample = plan.sample_for_index(0).unwrap();
        assert!(sample.color_t.is_some());
        assert!((sample.fout - (1.0 - sample.fin)).abs() < 0.00001);
        assert!(
            (sample.size - config.particle_size * sample.fin * plan.warmup * 2.0).abs() < 0.00001
        );
    }
}
