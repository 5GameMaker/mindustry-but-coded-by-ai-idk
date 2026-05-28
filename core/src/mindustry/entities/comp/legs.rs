//! Legs component shell mirroring upstream `mindustry.entities.comp.LegsComp`.

use crate::mindustry::entities::{Leg, LegDestroyData, TextureRegionRef};
use crate::mindustry::io::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegsSolidPred {
    Solid,
    LegsSolid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegsType {
    pub leg_count: i32,
    pub leg_length: f32,
    pub rotate_speed: f32,
    pub lock_leg_base: bool,
    pub allow_leg_step: bool,
    pub leg_base_offset: f32,
    pub leg_straightness: f32,
    pub leg_straight_length: f32,
    pub base_leg_straightness: f32,
    pub leg_group_size: i32,
    pub leg_move_space: f32,
    pub leg_continuous_move: bool,
    pub leg_forward_scl: f32,
    pub leg_pair_offset: f32,
    pub leg_length_scl: f32,
    pub leg_extension: f32,
    pub leg_min_length: f32,
    pub leg_max_length: f32,
    pub flip_back_legs: bool,
    pub flip_leg_side: bool,
    pub speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegsUpdateInput {
    pub delta: f32,
    pub delta_x: f32,
    pub delta_y: f32,
    pub deep_feet: i32,
    pub floor_on_deep: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegsDestroyRegions {
    pub leg: TextureRegionRef,
    pub leg_base: TextureRegionRef,
}

impl LegsDestroyRegions {
    pub fn new(leg: TextureRegionRef, leg_base: TextureRegionRef) -> Self {
        Self { leg, leg_base }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegsDestroyEffectEvent {
    pub x: f32,
    pub y: f32,
    pub data: LegDestroyData,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegsDynamicExplosionEvent {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub effect: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LegsDestroyPlan {
    pub effects: Vec<LegsDestroyEffectEvent>,
    pub explosions: Vec<LegsDynamicExplosionEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegsComp {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub speed_multiplier: f32,
    pub type_info: LegsType,
    pub legs: Vec<Leg>,
    pub total_length: f32,
    pub move_space: f32,
    pub base_rotation: f32,
    pub last_deep_floor: bool,
    pub cur_move_offset: Vec2,
    pub ignore_solids: bool,
}

impl LegsComp {
    pub fn new(type_info: LegsType) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            speed_multiplier: 1.0,
            type_info,
            legs: Vec::new(),
            total_length: 0.0,
            move_space: 0.0,
            base_rotation: 0.0,
            last_deep_floor: false,
            cur_move_offset: Vec2::new(0.0, 0.0),
            ignore_solids: false,
        }
    }

    pub fn solidity(&self) -> Option<LegsSolidPred> {
        if self.ignore_solids {
            None
        } else if self.type_info.allow_leg_step {
            Some(LegsSolidPred::LegsSolid)
        } else {
            Some(LegsSolidPred::Solid)
        }
    }

    pub fn drown_floor(&self) -> bool {
        self.last_deep_floor
    }

    pub fn add(&mut self) {
        self.reset_legs(self.type_info.leg_length);
    }

    pub fn unloaded(&mut self) {
        self.reset_legs(1.0);
    }

    pub fn destroy_plan(
        &self,
        regions: &LegsDestroyRegions,
        death_explosion_effect: &str,
        is_added: bool,
        headless: bool,
    ) -> LegsDestroyPlan {
        if !is_added || headless {
            return LegsDestroyPlan::default();
        }

        let mut plan = LegsDestroyPlan {
            effects: Vec::with_capacity(self.legs.len() * 2),
            explosions: Vec::with_capacity(self.legs.len() * 3),
        };
        let leg_explode_rad = regions.leg.height.max(0) as f32 / 4.0 / 1.45;

        for (index, leg) in self.legs.iter().enumerate() {
            let base = add(self.leg_offset(index), Vec2::new(self.x, self.y));
            let extension = set_length(
                Vec2::new(leg.joint.x - leg.base.x, leg.joint.y - leg.base.y),
                self.type_info.leg_extension,
            );
            let extended_joint = add(leg.joint, extension);

            for point in [base, leg.joint, leg.base] {
                plan.explosions.push(LegsDynamicExplosionEvent {
                    x: point.x,
                    y: point.y,
                    radius: leg_explode_rad,
                    effect: death_explosion_effect.to_string(),
                });
            }

            plan.effects.push(LegsDestroyEffectEvent {
                x: base.x,
                y: base.y,
                data: LegDestroyData::new(base, leg.joint, regions.leg.clone()),
            });
            plan.effects.push(LegsDestroyEffectEvent {
                x: leg.joint.x,
                y: leg.joint.y,
                data: LegDestroyData::new(extended_joint, leg.base, regions.leg_base.clone()),
            });
        }

        plan
    }

    pub fn reset_legs(&mut self, leg_length: f32) {
        self.legs.clear();
        if self.type_info.lock_leg_base {
            self.base_rotation = self.rotation;
        }
        let count = self.type_info.leg_count.max(0) as usize;
        for i in 0..count {
            let dst_rot = self.leg_angle(i);
            let base_offset = add(self.leg_offset(i), Vec2::new(self.x, self.y));
            let joint = add(trns(dst_rot, leg_length / 2.0), base_offset);
            let base = add(trns(dst_rot, leg_length), base_offset);
            self.legs.push(Leg {
                joint,
                base,
                ..Leg::default()
            });
        }
        self.total_length = 0.0;
    }

    pub fn update(&mut self, input: LegsUpdateInput) {
        if len(input.delta_x, input.delta_y) > 0.001 {
            self.base_rotation = move_toward(
                self.base_rotation,
                angle(input.delta_x, input.delta_y),
                self.type_info.rotate_speed,
            );
        }
        if self.type_info.lock_leg_base {
            self.base_rotation = self.rotation;
        }
        if self.legs.len() != self.type_info.leg_count.max(0) as usize {
            self.reset_legs(self.type_info.leg_length);
        }
        let div = (self.legs.len() as i32 / self.type_info.leg_group_size).max(2);
        self.move_space =
            self.type_info.leg_length / 1.6 / (div as f32 / 2.0) * self.type_info.leg_move_space;
        self.total_length += if self.type_info.leg_continuous_move {
            self.type_info.speed * self.speed_multiplier * input.delta
        } else {
            len(input.delta_x, input.delta_y)
        };

        let trns_len = self.move_space * 0.85 * self.type_info.leg_forward_scl;
        let moving = self.moving(input.delta_x, input.delta_y);
        let move_target = if moving {
            trns(angle(input.delta_x, input.delta_y), trns_len)
        } else {
            Vec2::new(0.0, 0.0)
        };
        self.cur_move_offset = lerp_vec(self.cur_move_offset, move_target, 0.1);

        let leg_length = self.type_info.leg_length;
        let leg_min = self.type_info.leg_min_length * leg_length;
        let leg_max = self.type_info.leg_max_length * leg_length;
        for index in 0..self.legs.len() {
            let dst_rot = self.leg_angle(index);
            let base_offset = add(self.leg_offset(index), Vec2::new(self.x, self.y));
            let stage_f = (self.total_length + index as f32 * self.type_info.leg_pair_offset)
                / self.move_space.max(f32::EPSILON);
            let stage = stage_f as i32;
            let group = stage.rem_euclid(div);
            let should_move = index as i32 % div == group;

            let leg = &mut self.legs[index];
            leg.joint = clamp_around(
                leg.joint,
                base_offset,
                self.type_info.leg_min_length * leg_length / 2.0,
                self.type_info.leg_max_length * leg_length / 2.0,
            );
            leg.base = clamp_around(leg.base, base_offset, leg_min, leg_max);
            leg.moving = should_move;
            leg.stage = if moving {
                stage_f.rem_euclid(1.0)
            } else {
                lerp_scalar(leg.stage, 0.0, 0.1)
            };
            leg.group = group;

            if should_move {
                let leg_dest = add(
                    add(
                        trns(dst_rot, leg_length * self.type_info.leg_length_scl),
                        base_offset,
                    ),
                    self.cur_move_offset,
                );
                leg.base = lerp_vec(leg.base, leg_dest, leg.stage);
            }
        }

        self.last_deep_floor = input.deep_feet == self.legs.len() as i32 && input.floor_on_deep;
    }

    pub fn moving(&self, delta_x: f32, delta_y: f32) -> bool {
        len(delta_x, delta_y) > 0.001
    }

    pub fn leg_offset(&self, index: usize) -> Vec2 {
        let mut out = trns(
            self.default_leg_angle(index),
            self.type_info.leg_base_offset,
        );
        if self.type_info.leg_straightness > 0.0 {
            let mut straight = trns(
                self.default_leg_angle(index) - self.base_rotation,
                self.type_info.leg_base_offset,
            );
            straight.y = straight.y.signum()
                * self.type_info.leg_base_offset
                * self.type_info.leg_straight_length;
            straight = rotate(straight, self.base_rotation);
            out = lerp_vec(out, straight, self.type_info.base_leg_straightness);
        }
        out
    }

    pub fn leg_angle(&self, index: usize) -> f32 {
        if self.type_info.leg_straightness > 0.0 {
            let target = if index >= self.legs.len().max(1) / 2 {
                -90.0
            } else {
                90.0
            } + self.base_rotation;
            slerp(
                self.default_leg_angle(index),
                target,
                self.type_info.leg_straightness,
            )
        } else {
            self.default_leg_angle(index)
        }
    }

    pub fn default_leg_angle(&self, index: usize) -> f32 {
        let count = self
            .legs
            .len()
            .max(self.type_info.leg_count.max(1) as usize) as f32;
        self.base_rotation + 360.0 / count * index as f32 + (360.0 / count / 2.0)
    }
}

fn trns(angle_degrees: f32, length: f32) -> Vec2 {
    let radians = angle_degrees.to_radians();
    Vec2::new(radians.cos() * length, radians.sin() * length)
}

fn rotate(v: Vec2, angle_degrees: f32) -> Vec2 {
    let radians = angle_degrees.to_radians();
    Vec2::new(
        radians.cos() * v.x - radians.sin() * v.y,
        radians.sin() * v.x + radians.cos() * v.y,
    )
}

fn add(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x + b.x, a.y + b.y)
}

fn len(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
}

fn len_vec(v: Vec2) -> f32 {
    len(v.x, v.y)
}

fn set_length(v: Vec2, target: f32) -> Vec2 {
    let length = len_vec(v);
    if length <= f32::EPSILON || target.abs() <= f32::EPSILON {
        Vec2::new(0.0, 0.0)
    } else {
        Vec2::new(v.x / length * target, v.y / length * target)
    }
}

fn angle(x: f32, y: f32) -> f32 {
    y.atan2(x).to_degrees().rem_euclid(360.0)
}

fn lerp_vec(a: Vec2, b: Vec2, alpha: f32) -> Vec2 {
    Vec2::new(a.x + (b.x - a.x) * alpha, a.y + (b.y - a.y) * alpha)
}

fn lerp_scalar(a: f32, b: f32, alpha: f32) -> f32 {
    a + (b - a) * alpha
}

fn clamp_around(point: Vec2, center: Vec2, min_len: f32, max_len: f32) -> Vec2 {
    let delta = Vec2::new(point.x - center.x, point.y - center.y);
    let length = len_vec(delta);
    if length <= f32::EPSILON {
        return Vec2::new(center.x + max_len.max(min_len), center.y);
    }
    let clamped = length.clamp(min_len, max_len);
    Vec2::new(
        center.x + delta.x / length * clamped,
        center.y + delta.y / length * clamped,
    )
}

fn move_toward(from: f32, to: f32, amount: f32) -> f32 {
    let delta = (to - from + 540.0).rem_euclid(360.0) - 180.0;
    if delta.abs() <= amount {
        to
    } else {
        (from + amount * delta.signum()).rem_euclid(360.0)
    }
}

fn slerp(from: f32, to: f32, alpha: f32) -> f32 {
    let delta = (to - from + 540.0).rem_euclid(360.0) - 180.0;
    (from + delta * alpha).rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legs_type() -> LegsType {
        LegsType {
            leg_count: 4,
            leg_length: 10.0,
            rotate_speed: 20.0,
            lock_leg_base: false,
            allow_leg_step: true,
            leg_base_offset: 2.0,
            leg_straightness: 0.0,
            leg_straight_length: 1.0,
            base_leg_straightness: 0.0,
            leg_group_size: 2,
            leg_move_space: 1.0,
            leg_continuous_move: false,
            leg_forward_scl: 1.0,
            leg_pair_offset: 0.0,
            leg_length_scl: 1.0,
            leg_extension: 3.0,
            leg_min_length: 0.0,
            leg_max_length: 1.75,
            flip_back_legs: true,
            flip_leg_side: false,
            speed: 1.0,
        }
    }

    #[test]
    fn legs_component_solidity_and_reset_legs_match_java_shape() {
        let mut legs = LegsComp::new(legs_type());
        assert_eq!(legs.solidity(), Some(LegsSolidPred::LegsSolid));
        legs.add();
        assert_eq!(legs.legs.len(), 4);
        assert_eq!(legs.default_leg_angle(0), 45.0);
        assert_eq!(legs.default_leg_angle(1), 135.0);
    }

    #[test]
    fn legs_component_unloaded_resets_with_length_one() {
        let mut legs = LegsComp::new(legs_type());
        legs.unloaded();
        assert_eq!(legs.legs.len(), 4);
        assert!((len(legs.legs[0].base.x, legs.legs[0].base.y) - 3.0).abs() < 0.0001);
    }

    #[test]
    fn legs_component_update_tracks_move_space_total_length_and_drown_floor() {
        let mut legs = LegsComp::new(legs_type());
        legs.add();
        legs.update(LegsUpdateInput {
            delta: 1.0,
            delta_x: 3.0,
            delta_y: 4.0,
            deep_feet: 4,
            floor_on_deep: true,
        });
        assert_eq!(legs.total_length, 5.0);
        assert!(legs.move_space > 0.0);
        assert!(legs.drown_floor());
        assert_eq!(legs.base_rotation, 20.0);
        assert!(legs.moving(3.0, 4.0));
        assert!((legs.cur_move_offset.x - 0.31875).abs() < 0.0001);
        assert!((legs.cur_move_offset.y - 0.425).abs() < 0.0001);
        assert_eq!(legs.legs[0].group, 0);
        assert!(legs.legs[0].moving);
        assert!((legs.legs[0].stage - 0.8).abs() < 0.0001);
        assert_eq!(legs.legs[1].group, 0);
        assert!(!legs.legs[1].moving);
    }

    #[test]
    fn legs_component_destroy_plan_matches_java_leg_destroy_payload_shape() {
        let mut legs = LegsComp::new(legs_type());
        legs.x = 10.0;
        legs.y = 20.0;
        legs.add();

        let regions = LegsDestroyRegions::new(
            TextureRegionRef::with_size("crawler-leg", 16, 8),
            TextureRegionRef::with_size("crawler-leg-base", 12, 6),
        );
        let plan = legs.destroy_plan(&regions, "dynamicExplosion", true, false);

        assert_eq!(plan.effects.len(), 8);
        assert_eq!(plan.explosions.len(), 12);
        assert_eq!(plan.explosions[0].effect, "dynamicExplosion");
        assert!((plan.explosions[0].radius - (8.0 / 4.0 / 1.45)).abs() < 0.0001);

        let base = legs.leg_offset(0);
        let base = Vec2::new(base.x + legs.x, base.y + legs.y);
        let leg = &legs.legs[0];
        assert_eq!(plan.effects[0].x, base.x);
        assert_eq!(plan.effects[0].y, base.y);
        assert_eq!(plan.effects[0].data.a, base);
        assert_eq!(plan.effects[0].data.b, leg.joint);
        assert_eq!(plan.effects[0].data.region, regions.leg.clone());

        assert_eq!(plan.effects[1].x, leg.joint.x);
        assert_eq!(plan.effects[1].y, leg.joint.y);
        assert_eq!(plan.effects[1].data.b, leg.base);
        assert_eq!(plan.effects[1].data.region, regions.leg_base.clone());
        let joint_offset = Vec2::new(
            plan.effects[1].data.a.x - leg.joint.x,
            plan.effects[1].data.a.y - leg.joint.y,
        );
        assert!((len_vec(joint_offset) - legs.type_info.leg_extension).abs() < 0.0001);

        assert!(legs
            .destroy_plan(&regions, "dynamicExplosion", false, false)
            .effects
            .is_empty());
        assert!(legs
            .destroy_plan(&regions, "dynamicExplosion", true, true)
            .effects
            .is_empty());
    }
}
