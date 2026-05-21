//! Child component mirroring upstream `mindustry.entities.comp.ChildComp`.

use crate::mindustry::entities::EntityPosition;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChildParent {
    pub x: f32,
    pub y: f32,
    /// Covers both Java `Rotc.rotation()` and `RotBlock.buildRotation()`.
    pub rotation: Option<f32>,
}

impl EntityPosition for ChildParent {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ChildComp {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub parent: Option<ChildParent>,
    pub rot_with_parent: bool,
    pub offset_x: f32,
    pub offset_y: f32,
    pub offset_pos: f32,
    pub offset_rot: f32,
}

impl ChildComp {
    pub const fn new(x: f32, y: f32, rotation: f32) -> Self {
        Self {
            x,
            y,
            rotation,
            parent: None,
            rot_with_parent: false,
            offset_x: 0.0,
            offset_y: 0.0,
            offset_pos: 0.0,
            offset_rot: 0.0,
        }
    }

    pub fn add(&mut self) {
        if let Some(parent) = self.parent {
            self.offset_x = self.x - parent.x;
            self.offset_y = self.y - parent.y;
            if self.rot_with_parent {
                if let Some(rotation) = parent.rotation {
                    self.offset_pos = -rotation;
                    self.offset_rot = self.rotation - rotation;
                }
            }
        }
    }

    pub fn update(&mut self) {
        if let Some(parent) = self.parent {
            if self.rot_with_parent {
                if let Some(rotation) = parent.rotation {
                    self.x =
                        parent.x + trnsx(rotation + self.offset_pos, self.offset_x, self.offset_y);
                    self.y =
                        parent.y + trnsy(rotation + self.offset_pos, self.offset_x, self.offset_y);
                    self.rotation = rotation + self.offset_rot;
                }
            } else {
                self.x = parent.x + self.offset_x;
                self.y = parent.y + self.offset_y;
            }
        }
    }
}

impl EntityPosition for ChildComp {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

fn trnsx(angle_degrees: f32, x: f32, y: f32) -> f32 {
    let radians = angle_degrees.to_radians();
    radians.cos() * x - radians.sin() * y
}

fn trnsy(angle_degrees: f32, x: f32, y: f32) -> f32 {
    let radians = angle_degrees.to_radians();
    radians.sin() * x + radians.cos() * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_component_captures_plain_parent_offset_on_add_and_update() {
        let mut child = ChildComp::new(12.0, 13.0, 45.0);
        child.parent = Some(ChildParent {
            x: 10.0,
            y: 10.0,
            rotation: None,
        });

        child.add();
        child.parent = Some(ChildParent {
            x: 20.0,
            y: 30.0,
            rotation: None,
        });
        child.update();

        assert_eq!((child.offset_x, child.offset_y), (2.0, 3.0));
        assert_eq!((child.x, child.y, child.rotation), (22.0, 33.0, 45.0));
    }

    #[test]
    fn child_component_rotates_with_rotating_parent_like_java_angles() {
        let mut child = ChildComp::new(12.0, 10.0, 30.0);
        child.rot_with_parent = true;
        child.parent = Some(ChildParent {
            x: 10.0,
            y: 10.0,
            rotation: Some(0.0),
        });

        child.add();
        child.parent = Some(ChildParent {
            x: 10.0,
            y: 10.0,
            rotation: Some(90.0),
        });
        child.update();

        assert!((child.x - 10.0).abs() < 0.0001);
        assert!((child.y - 12.0).abs() < 0.0001);
        assert_eq!(child.rotation, 120.0);
    }
}
