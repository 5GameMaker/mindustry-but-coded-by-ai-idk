//! Damage component mirroring upstream `mindustry.entities.comp.DamageComp`.

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DamageComp {
    pub damage: f32,
}

impl DamageComp {
    pub const fn new(damage: f32) -> Self {
        Self { damage }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_component_is_a_plain_damage_state_like_java() {
        assert_eq!(DamageComp::default().damage, 0.0);
        assert_eq!(DamageComp::new(12.5).damage, 12.5);
    }
}
