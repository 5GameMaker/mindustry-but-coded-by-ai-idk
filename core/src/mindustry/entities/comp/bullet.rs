//! Bullet state shell mirroring the stable core of upstream `BulletComp`.
//!
//! This file keeps the runtime bullet state and a set of pure helpers that can
//! be exercised without the world/collision systems. The goal is to make the
//! bullet pipeline incrementally movable while preserving the upstream data
//! shape as much as possible.

use crate::mindustry::core::world::World;
use crate::mindustry::ctype::ContentId;
use crate::mindustry::io::{EntityRef, TeamId, TypeValue, Vec2};
use crate::mindustry::logic::{rgba_u32_to_double_bits, LAccess};

const TILE_SIZE: f32 = 8.0;
const AIM_UNSET: f32 = -1.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BulletSpec {
    pub damage: f32,
    pub range: f32,
    pub speed: f32,
    pub hit_size: f32,
    pub draw_size: f32,
    pub layer: f32,
    pub drag: f32,
    pub accel: f32,
    pub damage_multiplier: f32,
    pub collides: bool,
    pub collides_air: bool,
    pub collides_ground: bool,
    pub collides_tiles: bool,
    pub collides_floor: bool,
    pub collide_terrain: bool,
    pub collides_team: bool,
    pub pierce: bool,
    pub pierce_building: bool,
    pub pierce_cap: i32,
    pub remove_after_pierce: bool,
    pub sticky: bool,
    pub sticky_extra_lifetime: f32,
    pub hit_under: bool,
    pub underwater: bool,
}

impl BulletSpec {
    pub const fn new(damage: f32, range: f32, speed: f32) -> Self {
        Self {
            damage,
            range,
            speed,
            hit_size: 4.0,
            draw_size: 40.0,
            layer: 0.0,
            drag: 0.0,
            accel: 0.0,
            damage_multiplier: 1.0,
            collides: true,
            collides_air: true,
            collides_ground: true,
            collides_tiles: true,
            collides_floor: false,
            collide_terrain: false,
            collides_team: false,
            pierce: false,
            pierce_building: false,
            pierce_cap: -1,
            remove_after_pierce: false,
            sticky: false,
            sticky_extra_lifetime: 0.0,
            hit_under: false,
            underwater: false,
        }
    }
}

impl Default for BulletSpec {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BulletComp {
    pub bullet_type_id: ContentId,
    pub team: TeamId,
    pub owner: EntityRef,
    pub x: f32,
    pub y: f32,
    pub last_x: f32,
    pub last_y: f32,
    pub damage: f32,
    pub time: f32,
    pub lifetime: f32,
    pub rotation: f32,
    pub velocity: Vec2,
    pub aim_x: f32,
    pub aim_y: f32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub data: TypeValue,
    pub fdata: f32,
    pub collided_ids: Vec<i32>,
    pub keep_alive: bool,
    pub just_spawned: bool,
    pub absorbed: bool,
    pub hit: bool,
    pub removed: bool,
    pub sticky_target: EntityRef,
    pub sticky_x: f32,
    pub sticky_y: f32,
    pub sticky_rotation: f32,
    pub sticky_offset: f32,
    pub frags: i32,
    pub building_damage_multiplier: f32,
}

impl BulletComp {
    pub fn new(bullet_type_id: ContentId, team: TeamId, owner: EntityRef, x: f32, y: f32) -> Self {
        Self {
            bullet_type_id,
            team,
            owner,
            x,
            y,
            last_x: x,
            last_y: y,
            damage: 0.0,
            time: 0.0,
            lifetime: 0.0,
            rotation: 0.0,
            velocity: Vec2 { x: 0.0, y: 0.0 },
            aim_x: AIM_UNSET,
            aim_y: AIM_UNSET,
            origin_x: x,
            origin_y: y,
            data: TypeValue::Null,
            fdata: 0.0,
            collided_ids: Vec::new(),
            keep_alive: false,
            just_spawned: true,
            absorbed: false,
            hit: false,
            removed: false,
            sticky_target: EntityRef::null(),
            sticky_x: 0.0,
            sticky_y: 0.0,
            sticky_rotation: 0.0,
            sticky_offset: 0.0,
            frags: 0,
            building_damage_multiplier: 1.0,
        }
    }

    pub fn is_active(&self) -> bool {
        !self.removed
    }

    pub fn damage_multiplier(&self, spec: &BulletSpec) -> f32 {
        spec.damage_multiplier * self.building_damage_multiplier
    }

    pub fn clip_size(&self, spec: &BulletSpec) -> f32 {
        spec.draw_size
    }

    pub fn has_collided(&self, id: i32) -> bool {
        self.collided_ids.contains(&id)
    }

    pub fn record_collision(&mut self, id: i32) {
        self.collided_ids.push(id);
    }

    pub fn drop_last_collision(&mut self) -> Option<i32> {
        self.collided_ids.pop()
    }

    pub fn clear_collisions(&mut self) {
        self.collided_ids.clear();
    }

    pub fn init_vel(&mut self, angle: f32, amount: f32) {
        self.velocity = vector_from_angle(angle, amount);
        self.rotation = angle;
    }

    pub fn rotation(&self) -> f32 {
        if vector_length_sq(self.velocity) <= 0.001 * 0.001 {
            self.rotation
        } else {
            vector_angle(self.velocity)
        }
    }

    pub fn set_rotation(&mut self, angle: f32) {
        self.rotation = angle;
        self.velocity = vector_with_angle(self.velocity, angle);
    }

    pub fn move_relative(&mut self, delta: f32, x: f32, y: f32) {
        self.last_x = self.x;
        self.last_y = self.y;
        let (dx, dy) = rotate_offset(self.rotation(), x * delta, y * delta);
        self.x += dx;
        self.y += dy;
    }

    pub fn turn(&mut self, delta: f32, x: f32, y: f32, spec: &BulletSpec) {
        let angle = vector_angle(self.velocity);
        let (dx, dy) = rotate_offset(angle, x * delta, y * delta);
        self.velocity.x += dx;
        self.velocity.y += dy;
        self.velocity = vector_limited(self.velocity, spec.speed);
    }

    pub fn step_motion(&mut self, delta: f32, spec: &BulletSpec) {
        if !self.just_spawned {
            self.last_x = self.x;
            self.last_y = self.y;
            self.x += self.velocity.x * delta;
            self.y += self.velocity.y * delta;

            let scale = (1.0 - spec.drag * delta).max(0.0);
            self.velocity.x *= scale;
            self.velocity.y *= scale;
        }
        self.just_spawned = false;

        if spec.accel != 0.0 {
            self.velocity = vector_with_length(
                self.velocity,
                vector_length(self.velocity) + spec.accel * delta,
            );
        }

        if self.keep_alive {
            self.time -= delta;
            self.keep_alive = false;
        }
    }

    pub fn sense(&self, sensor: LAccess, spec: &BulletSpec) -> f64 {
        match sensor {
            LAccess::Rotation => self.rotation() as f64,
            LAccess::Health => self.damage as f64,
            LAccess::MaxHealth => spec.damage as f64,
            LAccess::X => World::conv(self.x) as f64,
            LAccess::Y => World::conv(self.y) as f64,
            LAccess::VelocityX => (self.velocity.x * 60.0 / TILE_SIZE) as f64,
            LAccess::VelocityY => (self.velocity.y * 60.0 / TILE_SIZE) as f64,
            LAccess::Dead => {
                if self.is_active() {
                    0.0
                } else {
                    1.0
                }
            }
            LAccess::Team => self.team.0 as f64,
            LAccess::Range => spec.range as f64,
            LAccess::ShootX => World::conv(self.aim_x) as f64,
            LAccess::ShootY => World::conv(self.aim_y) as f64,
            LAccess::Speed => (spec.speed * 60.0 / TILE_SIZE) as f64,
            LAccess::Size => (spec.hit_size / TILE_SIZE) as f64,
            LAccess::Color => self.team_color_bits(),
            LAccess::BulletLifetime => self.lifetime as f64,
            LAccess::BulletTime => self.time as f64,
            _ => f64::NAN,
        }
    }

    pub fn set_prop<V>(&mut self, prop: LAccess, value: V)
    where
        V: Into<BulletPropValue>,
    {
        let value = value.into();
        match prop {
            LAccess::Health => {
                if let Some(value) = value.as_number() {
                    self.damage = value as f32;
                }
            }
            LAccess::X => {
                if let Some(value) = value.as_number() {
                    self.x = World::unconv(value as f32);
                }
            }
            LAccess::Y => {
                if let Some(value) = value.as_number() {
                    self.y = World::unconv(value as f32);
                }
            }
            LAccess::VelocityX => {
                if let Some(value) = value.as_number() {
                    self.velocity.x = (value * TILE_SIZE as f64 / 60.0) as f32;
                }
            }
            LAccess::VelocityY => {
                if let Some(value) = value.as_number() {
                    self.velocity.y = (value * TILE_SIZE as f64 / 60.0) as f32;
                }
            }
            LAccess::Rotation => {
                if let Some(value) = value.as_number() {
                    self.set_rotation(value as f32);
                }
            }
            LAccess::Team => {
                if let Some(team) = value.as_team_id() {
                    self.team = team;
                }
            }
            LAccess::Speed => {
                if let Some(value) = value.as_number() {
                    self.velocity = vector_with_length(self.velocity, value as f32 * TILE_SIZE);
                }
            }
            LAccess::BulletLifetime => {
                if let Some(value) = value.as_number() {
                    self.lifetime = value as f32;
                }
            }
            LAccess::BulletTime => {
                if let Some(value) = value.as_number() {
                    self.time = value as f32;
                }
            }
            _ => {}
        }
    }

    pub fn can_hit_under_build(
        &self,
        spec: &BulletSpec,
        build_under_bullets: bool,
        build_team: TeamId,
        direct_hit: bool,
        overshot_aim: bool,
    ) -> bool {
        if !build_under_bullets {
            return true;
        }

        let no_aim = self.aim_x == AIM_UNSET && self.aim_y == AIM_UNSET;
        direct_hit
            || spec.hit_under
            || build_team == self.team
            || (spec.pierce && overshot_aim && !no_aim)
            || no_aim
    }

    fn team_color_bits(&self) -> f64 {
        rgba_u32_to_double_bits(team_color_rgba(self.team))
    }
}

impl Default for BulletComp {
    fn default() -> Self {
        Self::new(0, TeamId(0), EntityRef::null(), 0.0, 0.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BulletPropValue {
    Number(f64),
    Value(TypeValue),
}

impl BulletPropValue {
    fn as_number(&self) -> Option<f64> {
        match self {
            BulletPropValue::Number(value) => Some(*value),
            BulletPropValue::Value(value) => match value {
                TypeValue::Int(v) => Some(*v as f64),
                TypeValue::Long(v) => Some(*v as f64),
                TypeValue::Float(v) => Some(*v as f64),
                TypeValue::Double(v) => Some(*v),
                TypeValue::Team(v) => Some(*v as f64),
                TypeValue::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
                _ => None,
            },
        }
    }

    fn as_team_id(&self) -> Option<TeamId> {
        match self {
            BulletPropValue::Number(value) => Some(TeamId((*value as i32) as u8)),
            BulletPropValue::Value(value) => match value {
                TypeValue::Team(v) => Some(TeamId(*v)),
                TypeValue::Int(v) => Some(TeamId((*v as i32) as u8)),
                TypeValue::Long(v) => Some(TeamId((*v as i64) as i32 as u8)),
                TypeValue::Float(v) => Some(TeamId((*v as i32) as u8)),
                TypeValue::Double(v) => Some(TeamId((*v as i32) as u8)),
                TypeValue::Bool(v) => Some(TeamId(if *v { 1 } else { 0 })),
                _ => None,
            },
        }
    }
}

impl From<f64> for BulletPropValue {
    fn from(value: f64) -> Self {
        BulletPropValue::Number(value)
    }
}

impl From<f32> for BulletPropValue {
    fn from(value: f32) -> Self {
        BulletPropValue::Number(value as f64)
    }
}

impl From<i32> for BulletPropValue {
    fn from(value: i32) -> Self {
        BulletPropValue::Number(value as f64)
    }
}

impl From<i64> for BulletPropValue {
    fn from(value: i64) -> Self {
        BulletPropValue::Number(value as f64)
    }
}

impl From<u8> for BulletPropValue {
    fn from(value: u8) -> Self {
        BulletPropValue::Number(value as f64)
    }
}

impl From<TeamId> for BulletPropValue {
    fn from(value: TeamId) -> Self {
        BulletPropValue::Value(TypeValue::Team(value.0))
    }
}

impl From<TypeValue> for BulletPropValue {
    fn from(value: TypeValue) -> Self {
        BulletPropValue::Value(value)
    }
}

fn vector_length(vec: Vec2) -> f32 {
    (vec.x * vec.x + vec.y * vec.y).sqrt()
}

fn vector_length_sq(vec: Vec2) -> f32 {
    vec.x * vec.x + vec.y * vec.y
}

fn vector_angle(vec: Vec2) -> f32 {
    if vector_length_sq(vec) <= 0.0 {
        0.0
    } else {
        vec.y.atan2(vec.x).to_degrees().rem_euclid(360.0)
    }
}

fn vector_from_angle(angle: f32, length: f32) -> Vec2 {
    let rad = angle.to_radians();
    Vec2 {
        x: rad.cos() * length,
        y: rad.sin() * length,
    }
}

fn vector_with_angle(vec: Vec2, angle: f32) -> Vec2 {
    vector_from_angle(angle, vector_length(vec))
}

fn vector_with_length(vec: Vec2, length: f32) -> Vec2 {
    if vector_length_sq(vec) <= 0.0 {
        Vec2 { x: length, y: 0.0 }
    } else {
        vector_from_angle(vector_angle(vec), length.max(0.0))
    }
}

fn vector_limited(vec: Vec2, max_len: f32) -> Vec2 {
    let len = vector_length(vec);
    if len <= max_len || len <= 0.0 {
        vec
    } else {
        vector_with_length(vec, max_len)
    }
}

fn rotate_offset(angle: f32, x: f32, y: f32) -> (f32, f32) {
    let rad = angle.to_radians();
    let cos = rad.cos();
    let sin = rad.sin();
    (x * cos - y * sin, x * sin + y * cos)
}

fn team_color_rgba(team: TeamId) -> u32 {
    match team.0 {
        0 => 0x4d4e58ff,
        1 => 0xffd37fff,
        2 => 0xf25555ff,
        3 => 0xa27ce5ff,
        4 => 0x54d67dff,
        5 => 0x6c87fdff,
        6 => 0xe05438ff,
        other => placeholder_team_color(other),
    }
}

fn placeholder_team_color(id: u8) -> u32 {
    let hash = splitmix32(id as u32 + 0x9e37_79b9);
    let r = 0x60 + ((hash >> 16) & 0x7f);
    let g = 0x60 + ((hash >> 8) & 0x7f);
    let b = 0x60 + (hash & 0x7f);
    (r << 24) | (g << 16) | (b << 8) | 0xff
}

fn splitmix32(mut value: u32) -> u32 {
    value = value.wrapping_add(0x9e37_79b9);
    value = (value ^ (value >> 16)).wrapping_mul(0x85eb_ca6b);
    value = (value ^ (value >> 13)).wrapping_mul(0xc2b2_ae35);
    value ^ (value >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::logic::{double_bits_to_rgba, rgba_u32_to_double_bits};

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.0001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn bullet_component_motion_helpers_follow_rotation_and_velocity_rules() {
        let spec = BulletSpec {
            speed: 5.0,
            drag: 0.25,
            accel: 1.0,
            ..BulletSpec::default()
        };
        let mut bullet = BulletComp::new(3, TeamId(1), EntityRef::new(42), 0.0, 0.0);

        bullet.init_vel(90.0, 2.0);
        assert_close(bullet.rotation(), 90.0);
        assert_close(bullet.velocity.x, 0.0);
        assert_close(bullet.velocity.y, 2.0);

        bullet.move_relative(2.0, 1.0, 0.0);
        assert_close(bullet.last_x, 0.0);
        assert_close(bullet.last_y, 0.0);
        assert_close(bullet.x, 0.0);
        assert_close(bullet.y, 2.0);

        bullet.turn(1.0, 1.0, 0.0, &spec);
        assert_close(bullet.velocity.x, 0.0);
        assert_close(bullet.velocity.y, 3.0);

        bullet.just_spawned = false;
        bullet.step_motion(1.0, &spec);
        assert_close(bullet.last_x, 0.0);
        assert_close(bullet.last_y, 2.0);
        assert_close(bullet.x, 0.0);
        assert_close(bullet.y, 5.0);
        assert_close(bullet.velocity.x, 0.0);
        assert_close(bullet.velocity.y, 3.25);
        assert!(!bullet.just_spawned);

        bullet.time = 5.0;
        bullet.keep_alive = true;
        bullet.step_motion(2.0, &BulletSpec::default());
        assert_close(bullet.time, 3.0);
        assert!(!bullet.keep_alive);
    }

    #[test]
    fn bullet_component_sense_and_set_prop_roundtrip_cover_stable_sensors() {
        let mut spec = BulletSpec::new(12.0, 120.0, 3.0);
        spec.hit_size = 16.0;
        spec.draw_size = 48.0;
        spec.damage_multiplier = 2.0;

        let mut bullet = BulletComp::new(9, TeamId(0), EntityRef::null(), 16.0, 24.0);
        bullet.aim_x = 40.0;
        bullet.aim_y = 48.0;
        bullet.building_damage_multiplier = 0.5;

        bullet.set_prop(LAccess::Health, 25.0);
        bullet.set_prop(LAccess::X, 3.0);
        bullet.set_prop(LAccess::Y, 4.0);
        bullet.set_prop(LAccess::VelocityX, 6.0);
        bullet.set_prop(LAccess::VelocityY, -3.0);
        bullet.set_prop(LAccess::Team, TypeValue::Team(4));
        bullet.set_prop(LAccess::BulletLifetime, 99.0);
        bullet.set_prop(LAccess::BulletTime, 12.0);

        assert_close(bullet.damage, 25.0);
        assert_close(bullet.x, 24.0);
        assert_close(bullet.y, 32.0);
        assert_close(bullet.velocity.x, 0.8);
        assert_close(bullet.velocity.y, -0.4);
        assert_eq!(bullet.team, TeamId(4));

        assert_close(bullet.sense(LAccess::Health, &spec) as f32, 25.0);
        assert_close(bullet.sense(LAccess::MaxHealth, &spec) as f32, 12.0);
        assert_close(bullet.sense(LAccess::X, &spec) as f32, 3.0);
        assert_close(bullet.sense(LAccess::Y, &spec) as f32, 4.0);
        assert_close(bullet.sense(LAccess::VelocityX, &spec) as f32, 6.0);
        assert_close(bullet.sense(LAccess::VelocityY, &spec) as f32, -3.0);
        assert_eq!(bullet.sense(LAccess::Dead, &spec), 0.0);
        assert_close(bullet.sense(LAccess::Team, &spec) as f32, 4.0);
        assert_close(bullet.sense(LAccess::Range, &spec) as f32, 120.0);
        assert_close(bullet.sense(LAccess::ShootX, &spec) as f32, 5.0);
        assert_close(bullet.sense(LAccess::ShootY, &spec) as f32, 6.0);
        assert_close(bullet.sense(LAccess::Speed, &spec) as f32, 22.5);
        assert_close(bullet.sense(LAccess::Size, &spec) as f32, 2.0);
        assert_eq!(
            double_bits_to_rgba(bullet.sense(LAccess::Color, &spec)),
            0x54d67dff
        );
        assert_close(bullet.sense(LAccess::BulletLifetime, &spec) as f32, 99.0);
        assert_close(bullet.sense(LAccess::BulletTime, &spec) as f32, 12.0);

        bullet.set_prop(LAccess::Team, TypeValue::Int(5));
        assert_eq!(bullet.team, TeamId(5));

        bullet.removed = true;
        assert_eq!(bullet.sense(LAccess::Dead, &spec), 1.0);

        assert_close(bullet.damage_multiplier(&spec), 1.0);
        assert_close(bullet.clip_size(&spec), 48.0);
        assert_eq!(
            bullet.sense(LAccess::Color, &spec),
            rgba_u32_to_double_bits(0x6c87fdff)
        );

        let mut speed_bullet = BulletComp::new(10, TeamId(6), EntityRef::null(), 0.0, 0.0);
        speed_bullet.set_prop(LAccess::Speed, 2.0);
        assert_close(speed_bullet.velocity.x, 16.0);
        assert_close(speed_bullet.velocity.y, 0.0);

        speed_bullet.set_prop(LAccess::Rotation, 90.0);
        assert_close(speed_bullet.rotation(), 90.0);
        assert_close(speed_bullet.velocity.x, 0.0);
        assert_close(speed_bullet.velocity.y, 16.0);
        assert_close(speed_bullet.sense(LAccess::Rotation, &spec) as f32, 90.0);

        speed_bullet.set_prop(LAccess::Rotation, 270.0);
        assert_close(speed_bullet.rotation(), 270.0);
    }

    #[test]
    fn bullet_component_collision_log_tracks_recent_ids() {
        let mut bullet = BulletComp::default();

        assert!(!bullet.has_collided(7));
        assert_eq!(bullet.drop_last_collision(), None);

        bullet.record_collision(7);
        bullet.record_collision(9);

        assert!(bullet.has_collided(7));
        assert!(bullet.has_collided(9));
        assert_eq!(bullet.drop_last_collision(), Some(9));
        assert!(bullet.has_collided(7));
        assert!(!bullet.has_collided(9));

        bullet.clear_collisions();
        assert!(!bullet.has_collided(7));
        assert_eq!(bullet.drop_last_collision(), None);
    }

    #[test]
    fn bullet_component_can_hit_under_build_matches_pure_boolean_rules() {
        let spec = BulletSpec {
            hit_under: false,
            pierce: true,
            ..BulletSpec::default()
        };
        let mut bullet = BulletComp::default();
        bullet.team = TeamId(2);
        bullet.aim_x = 10.0;
        bullet.aim_y = 20.0;

        assert!(!bullet.can_hit_under_build(&spec, true, TeamId(3), false, false));
        assert!(bullet.can_hit_under_build(&spec, false, TeamId(3), false, false));
        assert!(bullet.can_hit_under_build(&spec, true, TeamId(3), true, false));
        assert!(bullet.can_hit_under_build(&spec, true, TeamId(3), false, true));
        assert!(bullet.can_hit_under_build(&spec, true, TeamId(2), false, false));

        bullet.aim_x = AIM_UNSET;
        bullet.aim_y = AIM_UNSET;
        assert!(bullet.can_hit_under_build(&spec, true, TeamId(3), false, false));
    }
}
