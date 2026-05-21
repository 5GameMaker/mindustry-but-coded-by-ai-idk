//! Mech component mirroring upstream `mindustry.entities.comp.MechComp`.

use crate::mindustry::io::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MechType {
    pub base_rotate_speed: f32,
    pub speed: f32,
    pub mech_stride: f32,
    pub rotate_speed: f32,
    pub step_shake: f32,
    pub mech_step_particles: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MechStepPlan {
    pub stepped: bool,
    pub side: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MechComp {
    pub x: f32,
    pub y: f32,
    pub hit_size: f32,
    pub type_info: MechType,
    pub base_rotation: f32,
    pub walk_time: f32,
    pub walk_extension: f32,
    walked: bool,
}

impl MechComp {
    pub const fn new(type_info: MechType) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            hit_size: 0.0,
            type_info,
            base_rotation: 0.0,
            walk_time: 0.0,
            walk_extension: 0.0,
            walked: false,
        }
    }

    pub fn walked(&self) -> bool {
        self.walked
    }

    pub fn update(
        &mut self,
        delta_len: f32,
        delta_angle: f32,
        delta: f32,
        net_client: bool,
        is_remote: bool,
        can_step_effect: bool,
    ) -> MechStepPlan {
        if self.walked || net_client || is_remote {
            let speed_scale = if self.type_info.speed == 0.0 || delta == 0.0 {
                0.0
            } else {
                (delta_len / self.type_info.speed / delta).clamp(0.0, 1.0)
            };
            self.base_rotation = move_toward(
                self.base_rotation,
                delta_angle,
                self.type_info.base_rotate_speed * speed_scale * delta,
            );
            self.walk_time += delta_len;
            self.walked = false;
        }

        let extend = self.walk_extend(false);
        let base = self.walk_extend(true);
        let extend_scl = base % 1.0;
        let last_extend = self.walk_extension;
        let stepped = can_step_effect && extend_scl < last_extend && base % 2.0 > 1.0;
        self.walk_extension = extend_scl;

        MechStepPlan {
            stepped,
            side: -signum(extend),
        }
    }

    pub fn drown_floor(
        &self,
        can_drown: bool,
        all_nearby_deep: bool,
        floor_on_deep: bool,
    ) -> Option<bool> {
        if self.hit_size >= 12.0 && can_drown && !all_nearby_deep {
            None
        } else {
            Some(floor_on_deep)
        }
    }

    pub fn walk_extend(&self, scaled: bool) -> f32 {
        let stride = self.type_info.mech_stride;
        let mut raw = self.walk_time % (stride * 4.0);
        if scaled {
            return raw / stride;
        }
        if raw > stride * 3.0 {
            raw -= stride * 4.0;
        } else if raw > stride * 2.0 {
            raw = stride * 2.0 - raw;
        } else if raw > stride {
            raw = stride * 2.0 - raw;
        }
        raw
    }

    pub fn rotate_move(&mut self, vec: Vec2, delta: f32) {
        if !is_zero(vec, 0.0) {
            self.walked = true;
            self.base_rotation = move_toward(
                self.base_rotation,
                angle(vec),
                self.type_info.rotate_speed * delta.max(1.0),
            );
        }
    }

    pub fn move_at(&mut self, vector: Vec2) {
        if !is_zero(vector, 0.0) {
            self.walked = true;
        }
    }

    pub fn approach(&mut self, vector: Vec2) {
        if !is_zero(vector, 0.001) {
            self.walked = true;
        }
    }
}

fn signum(value: f32) -> i32 {
    if value > 0.0 {
        1
    } else if value < 0.0 {
        -1
    } else {
        0
    }
}

fn is_zero(vec: Vec2, tolerance: f32) -> bool {
    vec.x * vec.x + vec.y * vec.y <= tolerance * tolerance
}

fn angle(vec: Vec2) -> f32 {
    vec.y.atan2(vec.x).to_degrees().rem_euclid(360.0)
}

fn move_toward(from: f32, to: f32, amount: f32) -> f32 {
    let delta = (to - from + 540.0).rem_euclid(360.0) - 180.0;
    if delta.abs() <= amount {
        to
    } else {
        (from + amount * delta.signum()).rem_euclid(360.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mech_type() -> MechType {
        MechType {
            base_rotate_speed: 10.0,
            speed: 5.0,
            mech_stride: 4.0,
            rotate_speed: 20.0,
            step_shake: 1.0,
            mech_step_particles: true,
        }
    }

    #[test]
    fn mech_walk_extend_matches_java_piecewise_stride_wave() {
        let mut mech = MechComp::new(mech_type());
        mech.walk_time = 5.0;
        assert_eq!(mech.walk_extend(true), 1.25);
        assert_eq!(mech.walk_extend(false), 3.0);

        mech.walk_time = 13.0;
        assert_eq!(mech.walk_extend(false), -3.0);
    }

    #[test]
    fn mech_update_rotates_base_and_advances_walk_time_when_walked() {
        let mut mech = MechComp::new(mech_type());
        mech.move_at(Vec2 { x: 1.0, y: 0.0 });
        assert!(mech.walked());

        mech.update(5.0, 90.0, 1.0, false, false, false);

        assert_eq!(mech.base_rotation, 10.0);
        assert_eq!(mech.walk_time, 5.0);
        assert!(!mech.walked());
    }

    #[test]
    fn mech_rotate_move_and_approach_mark_controlled_walking() {
        let mut mech = MechComp::new(mech_type());
        mech.rotate_move(Vec2 { x: 0.0, y: 1.0 }, 1.0);
        assert!(mech.walked());
        assert_eq!(mech.base_rotation, 20.0);

        mech.update(0.0, 0.0, 1.0, false, false, false);
        mech.approach(Vec2 { x: 0.01, y: 0.0 });
        assert!(mech.walked());
    }

    #[test]
    fn mech_drown_floor_requires_all_nearby_deep_for_large_mechs() {
        let mut mech = MechComp::new(mech_type());
        mech.hit_size = 12.0;

        assert_eq!(mech.drown_floor(true, false, true), None);
        assert_eq!(mech.drown_floor(true, true, true), Some(true));
        mech.hit_size = 8.0;
        assert_eq!(mech.drown_floor(true, false, false), Some(false));
    }
}
