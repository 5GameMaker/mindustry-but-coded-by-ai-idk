//! Rotation component mirroring upstream `mindustry.entities.comp.RotComp`.
//!
//! Java annotates `rotation` with `@SyncField(false)` and `@SyncLocal`. The
//! field is kept as plain state here; sync ownership is intentionally left to
//! the Rust networking/snapshot layer.

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RotComp {
    pub rotation: f32,
}

impl RotComp {
    pub const fn new(rotation: f32) -> Self {
        Self { rotation }
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn add_rotation(&mut self, amount: f32) {
        self.rotation += amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_component_defaults_and_updates_plain_rotation_field() {
        let mut rot = RotComp::default();
        assert_eq!(rot.rotation, 0.0);

        rot.set_rotation(90.0);
        rot.add_rotation(45.0);

        assert_eq!(rot.rotation, 135.0);
    }
}
