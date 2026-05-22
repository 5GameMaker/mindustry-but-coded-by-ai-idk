use crate::mindustry::io::Vec2;

use super::AsyncProcess;

pub const PHYSICS_LAYERS: i32 = 4;
pub const PHYSICS_LAYER_GROUND: i32 = 0;
pub const PHYSICS_LAYER_LEGS: i32 = 1;
pub const PHYSICS_LAYER_FLYING: i32 = 2;
pub const PHYSICS_LAYER_UNDERWATER: i32 = 3;
pub const PHYSICS_SOFTEN: f32 = 1.25;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl PhysicsRect {
    pub fn centered(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x: x - width / 2.0,
            y: y - height / 2.0,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsBody {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub mass: f32,
    pub layer: i32,
    pub collided: bool,
    pub local: bool,
}

impl PhysicsBody {
    pub fn hitbox(&self) -> PhysicsRect {
        PhysicsRect::centered(self.x, self.y, self.radius * 2.0, self.radius * 2.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsWorld {
    pub bounds: PhysicsRect,
    pub bodies: Vec<PhysicsBody>,
    pub zero_separation_angle: f32,
}

impl PhysicsWorld {
    pub fn new(bounds: PhysicsRect) -> Self {
        Self {
            bounds,
            bodies: Vec::new(),
            zero_separation_angle: 0.0,
        }
    }

    pub fn add(&mut self, body: PhysicsBody) {
        self.bodies.push(body);
    }

    pub fn remove(&mut self, id: i32) -> Option<PhysicsBody> {
        let index = self.bodies.iter().position(|body| body.id == id)?;
        Some(self.bodies.remove(index))
    }

    pub fn body_mut(&mut self, id: i32) -> Option<&mut PhysicsBody> {
        self.bodies.iter_mut().find(|body| body.id == id)
    }

    pub fn update(&mut self) {
        for body in &mut self.bodies {
            if body.layer >= 0 {
                body.collided = false;
            }
        }

        let len = self.bodies.len();
        for i in 0..len {
            if !self.bodies[i].local || self.bodies[i].layer < 0 {
                continue;
            }

            let layer = self.bodies[i].layer;
            for j in 0..len {
                if i == j || self.bodies[j].collided || self.bodies[j].layer != layer {
                    continue;
                }

                let rs = self.bodies[i].radius + self.bodies[j].radius;
                let dx = self.bodies[i].x - self.bodies[j].x;
                let dy = self.bodies[i].y - self.bodies[j].y;
                let dst = (dx * dx + dy * dy).sqrt();
                if dst >= rs {
                    continue;
                }

                let push = if dst <= f32::EPSILON {
                    vec_from_angle(self.zero_separation_angle, rs - dst)
                } else {
                    set_length(Vec2::new(dx, dy), rs - dst)
                };
                let mass_sum = self.bodies[i].mass + self.bodies[j].mass;
                if mass_sum <= f32::EPSILON {
                    continue;
                }
                let m1 = self.bodies[j].mass / mass_sum;
                let m2 = self.bodies[i].mass / mass_sum;

                self.bodies[i].x += push.x * m1 / PHYSICS_SOFTEN;
                self.bodies[i].y += push.y * m1 / PHYSICS_SOFTEN;

                if self.bodies[j].local {
                    self.bodies[j].x -= push.x * m2 / PHYSICS_SOFTEN;
                    self.bodies[j].y -= push.y * m2 / PHYSICS_SOFTEN;
                }
            }

            self.bodies[i].collided = true;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsEntitySnapshot {
    pub id: i32,
    pub added: bool,
    pub has_physics: bool,
    pub x: f32,
    pub y: f32,
    pub mass: f32,
    pub hit_size: f32,
    pub collision_layer: i32,
    pub local: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsRef {
    pub entity_id: i32,
    pub body_id: i32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsMovePlan {
    pub entity_id: i32,
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsProcess {
    pub physics: Option<PhysicsWorld>,
    pub refs: Vec<PhysicsRef>,
}

impl PhysicsProcess {
    pub fn new(bounds: PhysicsRect) -> Self {
        Self {
            physics: Some(PhysicsWorld::new(bounds)),
            refs: Vec::new(),
        }
    }

    pub fn init_world(&mut self, bounds: PhysicsRect) {
        self.reset();
        self.physics = Some(PhysicsWorld::new(bounds));
    }

    pub fn begin_with_entities(
        &mut self,
        entities: &[PhysicsEntitySnapshot],
        net_client: bool,
        unit_collision_radius_scale: f32,
    ) {
        let Some(world) = self.physics.as_mut() else {
            return;
        };

        self.refs.retain(|reference| {
            let keep = entities
                .iter()
                .any(|entity| entity.id == reference.entity_id && entity.added);
            if !keep {
                world.remove(reference.body_id);
            }
            keep
        });

        for entity in entities {
            if !entity.added || !entity.has_physics {
                continue;
            }

            if self
                .refs
                .iter()
                .all(|reference| reference.entity_id != entity.id)
            {
                world.add(PhysicsBody {
                    id: entity.id,
                    x: entity.x,
                    y: entity.y,
                    mass: entity.mass,
                    radius: entity.hit_size * unit_collision_radius_scale,
                    layer: entity.collision_layer,
                    collided: false,
                    local: !net_client || entity.local,
                });
                self.refs.push(PhysicsRef {
                    entity_id: entity.id,
                    body_id: entity.id,
                    x: entity.x,
                    y: entity.y,
                });
            }

            if let Some(reference) = self
                .refs
                .iter_mut()
                .find(|reference| reference.entity_id == entity.id)
            {
                reference.x = entity.x;
                reference.y = entity.y;
            }

            if let Some(body) = world.body_mut(entity.id) {
                body.layer = entity.collision_layer;
                body.local = !net_client || entity.local;
            }
        }
    }

    pub fn process_world(&mut self) {
        let Some(world) = self.physics.as_mut() else {
            return;
        };

        for reference in &self.refs {
            if let Some(body) = world.body_mut(reference.body_id) {
                body.x = reference.x;
                body.y = reference.y;
            }
        }

        world.update();
    }

    pub fn end_moves(&self) -> Vec<PhysicsMovePlan> {
        let Some(world) = self.physics.as_ref() else {
            return Vec::new();
        };

        self.refs
            .iter()
            .filter_map(|reference| {
                let body = world
                    .bodies
                    .iter()
                    .find(|body| body.id == reference.body_id)?;
                Some(PhysicsMovePlan {
                    entity_id: reference.entity_id,
                    dx: body.x - reference.x,
                    dy: body.y - reference.y,
                })
            })
            .collect()
    }
}

impl AsyncProcess for PhysicsProcess {
    fn reset(&mut self) {
        self.refs.clear();
        self.physics = None;
    }

    fn process(&mut self) {
        self.process_world();
    }
}

fn vec_from_angle(angle: f32, length: f32) -> Vec2 {
    let rad = angle.to_radians();
    Vec2::new(rad.cos() * length, rad.sin() * length)
}

fn set_length(vec: Vec2, length: f32) -> Vec2 {
    let current = (vec.x * vec.x + vec.y * vec.y).sqrt();
    if current <= f32::EPSILON {
        Vec2::new(0.0, 0.0)
    } else {
        let scale = length / current;
        Vec2::new(vec.x * scale, vec.y * scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_body_hitbox_is_centered_on_radius_like_quad_tree_object() {
        let body = PhysicsBody {
            id: 1,
            x: 10.0,
            y: 20.0,
            radius: 3.0,
            mass: 1.0,
            layer: PHYSICS_LAYER_GROUND,
            collided: false,
            local: true,
        };

        assert_eq!(
            body.hitbox(),
            PhysicsRect {
                x: 7.0,
                y: 17.0,
                width: 6.0,
                height: 6.0,
            }
        );
    }

    #[test]
    fn physics_world_separates_overlapping_local_bodies_by_mass() {
        let mut world = PhysicsWorld::new(PhysicsRect::centered(0.0, 0.0, 100.0, 100.0));
        world.add(PhysicsBody {
            id: 1,
            x: 0.0,
            y: 0.0,
            radius: 5.0,
            mass: 1.0,
            layer: PHYSICS_LAYER_GROUND,
            collided: false,
            local: true,
        });
        world.add(PhysicsBody {
            id: 2,
            x: 8.0,
            y: 0.0,
            radius: 5.0,
            mass: 3.0,
            layer: PHYSICS_LAYER_GROUND,
            collided: false,
            local: true,
        });

        world.update();

        let first = world.bodies.iter().find(|body| body.id == 1).unwrap();
        let second = world.bodies.iter().find(|body| body.id == 2).unwrap();
        assert!((first.x + 1.2).abs() < 0.0001);
        assert!((second.x - 8.4).abs() < 0.0001);
        assert!(first.collided);
    }

    #[test]
    fn physics_world_only_local_bodies_initiate_collision_resolution() {
        let mut world = PhysicsWorld::new(PhysicsRect::centered(0.0, 0.0, 100.0, 100.0));
        world.add(PhysicsBody {
            id: 1,
            x: 0.0,
            y: 0.0,
            radius: 5.0,
            mass: 1.0,
            layer: PHYSICS_LAYER_GROUND,
            collided: false,
            local: false,
        });
        world.add(PhysicsBody {
            id: 2,
            x: 8.0,
            y: 0.0,
            radius: 5.0,
            mass: 1.0,
            layer: PHYSICS_LAYER_GROUND,
            collided: false,
            local: false,
        });

        world.update();

        assert_eq!(world.bodies[0].x, 0.0);
        assert_eq!(world.bodies[1].x, 8.0);
    }

    #[test]
    fn physics_process_creates_refs_updates_world_and_returns_move_plan() {
        let mut process = PhysicsProcess::new(PhysicsRect::centered(0.0, 0.0, 100.0, 100.0));
        process.begin_with_entities(
            &[
                PhysicsEntitySnapshot {
                    id: 1,
                    added: true,
                    has_physics: true,
                    x: 0.0,
                    y: 0.0,
                    mass: 1.0,
                    hit_size: 10.0,
                    collision_layer: PHYSICS_LAYER_GROUND,
                    local: true,
                },
                PhysicsEntitySnapshot {
                    id: 2,
                    added: true,
                    has_physics: true,
                    x: 8.0,
                    y: 0.0,
                    mass: 1.0,
                    hit_size: 10.0,
                    collision_layer: PHYSICS_LAYER_GROUND,
                    local: true,
                },
            ],
            false,
            0.5,
        );

        assert_eq!(process.refs.len(), 2);
        process.process_world();
        let moves = process.end_moves();
        assert_eq!(moves.len(), 2);
        assert!(moves
            .iter()
            .any(|plan| plan.entity_id == 1 && plan.dx < 0.0));
        assert!(moves
            .iter()
            .any(|plan| plan.entity_id == 2 && plan.dx > 0.0));
    }

    #[test]
    fn physics_process_removes_stale_refs_and_reset_drops_world() {
        let mut process = PhysicsProcess::new(PhysicsRect::centered(0.0, 0.0, 100.0, 100.0));
        process.begin_with_entities(
            &[PhysicsEntitySnapshot {
                id: 1,
                added: true,
                has_physics: true,
                x: 0.0,
                y: 0.0,
                mass: 1.0,
                hit_size: 10.0,
                collision_layer: PHYSICS_LAYER_GROUND,
                local: true,
            }],
            false,
            0.5,
        );
        process.begin_with_entities(
            &[PhysicsEntitySnapshot {
                id: 1,
                added: false,
                has_physics: true,
                x: 0.0,
                y: 0.0,
                mass: 1.0,
                hit_size: 10.0,
                collision_layer: PHYSICS_LAYER_GROUND,
                local: true,
            }],
            false,
            0.5,
        );

        assert!(process.refs.is_empty());
        assert!(process.physics.as_ref().unwrap().bodies.is_empty());

        process.reset();
        assert!(process.refs.is_empty());
        assert!(process.physics.is_none());
    }
}
