/// Data-only port of upstream `ParticleRenderer`.
///
/// The Java class owns an OpenGL point-sprite mesh and may update particles on
/// a background executor.  This Rust core module keeps the queue/update/vertex
/// math deterministic and backend-neutral; render backends can upload
/// `ParticleVertex` values to their preferred GPU abstraction.

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
