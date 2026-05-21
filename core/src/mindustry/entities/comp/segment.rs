//! Segment component mirroring upstream `mindustry.entities.comp.SegmentComp`.

use crate::mindustry::io::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentType {
    pub player_controllable: bool,
    pub allow_leg_step: bool,
    pub leg_physics_layer: bool,
    pub grounded: bool,
    pub speed: f32,
    pub segment_rotation_range: f32,
    pub base_rotate_speed: f32,
    pub segment_spacing: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentRef {
    pub id: i32,
    pub valid: bool,
    pub rotation: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentComp {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub vel: Vec2,
    pub type_info: SegmentType,
    pub parent_segment: Option<SegmentRef>,
    pub child_segment: Option<SegmentRef>,
    pub head_segment: Option<SegmentRef>,
    pub segment_index: i32,
    pub parent_id: i32,
    pub flying: bool,
    pub command_ai: bool,
}

impl SegmentComp {
    pub const LAYER_GROUND: i32 = 0;
    pub const LAYER_LEGS: i32 = 1;
    pub const LAYER_FLYING: i32 = 2;

    pub const fn new(id: i32, type_info: SegmentType) -> Self {
        Self {
            id,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            vel: Vec2 { x: 0.0, y: 0.0 },
            type_info,
            parent_segment: None,
            child_segment: None,
            head_segment: None,
            segment_index: 0,
            parent_id: -1,
            flying: false,
            command_ai: false,
        }
    }

    pub fn is_head(&self) -> bool {
        self.parent_segment.is_none()
    }

    pub fn add_child(&mut self, other: SegmentRef) {
        if other.id == self.id {
            return;
        }
        self.child_segment = Some(other);
    }

    pub fn ignore_solids(&self) -> bool {
        self.flying || self.parent_segment.is_some()
    }

    pub fn update(&mut self) {
        if self.child_segment.is_some_and(|child| !child.valid) {
            self.child_segment = None;
        }
        if self.parent_segment.is_some_and(|parent| !parent.valid) {
            self.parent_segment = None;
        }
        if self.parent_segment.is_none() {
            self.segment_index = 0;
            if self.child_segment.is_some() {
                self.head_segment = Some(SegmentRef {
                    id: self.id,
                    valid: true,
                    rotation: self.rotation as i32,
                });
            }
        }
    }

    pub fn player_controllable(&self) -> bool {
        self.type_info.player_controllable && self.is_head()
    }

    pub fn should_update_controller(&self) -> bool {
        self.is_head()
    }

    pub fn moving(&self, delta_len: f32, delta: f32) -> bool {
        if self.is_head() {
            self.vel.x * self.vel.x + self.vel.y * self.vel.y > 0.01 * 0.01
        } else {
            delta_len / delta >= 0.01
        }
    }

    pub fn collision_layer(&self) -> i32 {
        if self.parent_segment.is_some() {
            -1
        } else if self.type_info.allow_leg_step && self.type_info.leg_physics_layer {
            Self::LAYER_LEGS
        } else if self.type_info.grounded {
            Self::LAYER_GROUND
        } else {
            Self::LAYER_FLYING
        }
    }

    pub fn is_commandable(&self) -> bool {
        self.parent_segment.is_none() && self.command_ai
    }

    pub fn before_write(&mut self) {
        self.parent_id = self.parent_segment.map(|parent| parent.id).unwrap_or(-1);
    }

    pub fn check_parent<F>(&mut self, resolve: F)
    where
        F: FnOnce(i32) -> Option<SegmentRef>,
    {
        if self.parent_id != -1 {
            if let Some(parent) = resolve(self.parent_id) {
                self.parent_segment = Some(parent);
                return;
            }
            self.parent_id = -1;
        }
        self.parent_segment = None;
    }

    pub fn update_segment(
        &mut self,
        head: SegmentRef,
        parent: SegmentRef,
        index: i32,
        head_delta: f32,
        delta: f32,
    ) {
        self.rotation = clamp_range(
            self.rotation,
            parent.rotation as f32,
            self.type_info.segment_rotation_range,
        );
        self.segment_index = index;
        self.head_segment = Some(head);
        if head_delta > 0.001 && self.type_info.speed != 0.0 && delta != 0.0 {
            self.rotation = slerp(
                self.rotation,
                parent.rotation as f32,
                self.type_info.base_rotate_speed
                    * (head_delta / self.type_info.speed / delta).clamp(0.0, 1.0),
            );
        }
    }
}

fn clamp_range(value: f32, target: f32, range: f32) -> f32 {
    let delta = (value - target + 540.0).rem_euclid(360.0) - 180.0;
    if delta.abs() <= range {
        value
    } else {
        (target + range * delta.signum()).rem_euclid(360.0)
    }
}

fn slerp(from: f32, to: f32, alpha: f32) -> f32 {
    let delta = (to - from + 540.0).rem_euclid(360.0) - 180.0;
    (from + delta * alpha.clamp(0.0, 1.0)).rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg_type() -> SegmentType {
        SegmentType {
            player_controllable: true,
            allow_leg_step: true,
            leg_physics_layer: true,
            grounded: true,
            speed: 5.0,
            segment_rotation_range: 30.0,
            base_rotate_speed: 0.5,
            segment_spacing: 8.0,
        }
    }

    #[test]
    fn segment_component_head_child_and_solidity_rules_match_java() {
        let mut seg = SegmentComp::new(1, seg_type());
        assert!(seg.is_head());
        assert!(!seg.ignore_solids());

        seg.add_child(SegmentRef {
            id: 2,
            valid: true,
            rotation: 0,
        });
        seg.update();
        assert_eq!(seg.segment_index, 0);
        assert!(seg.head_segment.is_some());

        seg.parent_segment = Some(SegmentRef {
            id: 9,
            valid: true,
            rotation: 0,
        });
        assert!(seg.ignore_solids());
        assert!(!seg.player_controllable());
    }

    #[test]
    fn segment_component_moving_collision_and_commandable_follow_head_state() {
        let mut seg = SegmentComp::new(1, seg_type());
        seg.vel = Vec2 { x: 1.0, y: 0.0 };
        seg.command_ai = true;

        assert!(seg.moving(0.0, 1.0));
        assert_eq!(seg.collision_layer(), SegmentComp::LAYER_LEGS);
        assert!(seg.is_commandable());

        seg.parent_segment = Some(SegmentRef {
            id: 2,
            valid: true,
            rotation: 0,
        });
        assert_eq!(seg.collision_layer(), -1);
        assert!(seg.moving(1.0, 10.0));
        assert!(!seg.is_commandable());
    }

    #[test]
    fn segment_component_parent_id_roundtrip_and_update_segment() {
        let mut seg = SegmentComp::new(3, seg_type());
        seg.parent_segment = Some(SegmentRef {
            id: 7,
            valid: true,
            rotation: 90,
        });
        seg.before_write();
        assert_eq!(seg.parent_id, 7);

        seg.parent_segment = None;
        seg.check_parent(|id| {
            Some(SegmentRef {
                id,
                valid: true,
                rotation: 90,
            })
        });
        assert!(seg.parent_segment.is_some());

        seg.rotation = 180.0;
        seg.update_segment(
            SegmentRef {
                id: 1,
                valid: true,
                rotation: 90,
            },
            SegmentRef {
                id: 7,
                valid: true,
                rotation: 90,
            },
            2,
            5.0,
            1.0,
        );
        assert_eq!(seg.segment_index, 2);
        assert!(seg.rotation <= 120.0);
    }
}
