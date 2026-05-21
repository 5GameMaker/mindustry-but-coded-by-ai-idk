//! Owner component mirroring upstream `mindustry.entities.comp.OwnerComp`.

use crate::mindustry::io::EntityRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnerComp {
    pub owner: EntityRef,
}

impl OwnerComp {
    pub const fn new(owner: EntityRef) -> Self {
        Self { owner }
    }

    pub const fn without_owner() -> Self {
        Self {
            owner: EntityRef::null(),
        }
    }

    pub fn has_owner(&self) -> bool {
        self.owner.id.is_some()
    }
}

impl Default for OwnerComp {
    fn default() -> Self {
        Self::without_owner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_component_defaults_to_java_null_owner() {
        let owner = OwnerComp::default();

        assert_eq!(owner.owner, EntityRef::null());
        assert!(!owner.has_owner());
    }

    #[test]
    fn owner_component_stores_entity_reference() {
        let owner = OwnerComp::new(EntityRef::new(42));

        assert_eq!(owner.owner, EntityRef::new(42));
        assert!(owner.has_owner());
    }
}
