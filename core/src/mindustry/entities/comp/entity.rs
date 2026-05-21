//! Base entity component mirroring upstream `mindustry.entities.comp.EntityComp`.

use std::sync::atomic::{AtomicI32, Ordering};

static NEXT_ENTITY_ID: AtomicI32 = AtomicI32::new(1);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EntityLocality {
    /// Java branch: `((Object)this) == player`.
    pub is_player_object: bool,
    /// Java branch: `this instanceof Unitc u && u.controller() == player`.
    pub unit_controller_is_player: bool,
    /// Java branch: `this instanceof Unitc u && u.isPlayer()`.
    pub unit_is_player: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EntityIoState {
    pub before_write_calls: usize,
    pub write_calls: usize,
    pub after_read_calls: usize,
    pub after_read_all_calls: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityComp {
    added: bool,
    pub id: i32,
    pub class_id: i32,
    pub serialize: bool,
    pub io: EntityIoState,
}

impl EntityComp {
    pub fn new(class_id: i32, serialize: bool) -> Self {
        Self::with_id(
            NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed),
            class_id,
            serialize,
        )
    }

    pub const fn with_id(id: i32, class_id: i32, serialize: bool) -> Self {
        Self {
            added: false,
            id,
            class_id,
            serialize,
            io: EntityIoState {
                before_write_calls: 0,
                write_calls: 0,
                after_read_calls: 0,
                after_read_all_calls: 0,
            },
        }
    }

    pub fn is_added(&self) -> bool {
        self.added
    }

    pub fn update(&mut self) {}

    pub fn remove(&mut self) {
        self.added = false;
    }

    pub fn add(&mut self) {
        self.added = true;
    }

    pub fn is_local(&self, locality: EntityLocality) -> bool {
        locality.is_player_object || locality.unit_controller_is_player
    }

    pub fn is_remote(&self, locality: EntityLocality) -> bool {
        locality.unit_is_player && !self.is_local(locality)
    }

    pub fn class_id(&self) -> i32 {
        self.class_id
    }

    pub fn serialize(&self) -> bool {
        self.serialize
    }

    pub fn read(&mut self) {
        self.after_read();
    }

    pub fn write(&mut self) {
        self.io.write_calls += 1;
    }

    pub fn before_write(&mut self) {
        self.io.before_write_calls += 1;
    }

    pub fn after_read(&mut self) {
        self.io.after_read_calls += 1;
    }

    pub fn after_read_all(&mut self) {
        self.io.after_read_all_calls += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_component_add_remove_and_id_state_match_java_base_shape() {
        let mut entity = EntityComp::with_id(42, 7, true);

        assert_eq!(entity.id, 42);
        assert_eq!(entity.class_id(), 7);
        assert!(entity.serialize());
        assert!(!entity.is_added());

        entity.add();
        assert!(entity.is_added());
        entity.remove();
        assert!(!entity.is_added());
    }

    #[test]
    fn entity_component_local_and_remote_checks_match_java_branches() {
        let entity = EntityComp::with_id(1, 0, false);

        assert!(entity.is_local(EntityLocality {
            is_player_object: true,
            ..EntityLocality::default()
        }));
        assert!(entity.is_local(EntityLocality {
            unit_controller_is_player: true,
            ..EntityLocality::default()
        }));
        assert!(entity.is_remote(EntityLocality {
            unit_is_player: true,
            ..EntityLocality::default()
        }));
        assert!(!entity.is_remote(EntityLocality {
            unit_is_player: true,
            unit_controller_is_player: true,
            ..EntityLocality::default()
        }));
    }

    #[test]
    fn entity_component_io_hooks_are_noops_with_observable_order_points() {
        let mut entity = EntityComp::with_id(1, 0, true);

        entity.before_write();
        entity.write();
        entity.read();
        entity.after_read_all();

        assert_eq!(
            entity.io,
            EntityIoState {
                before_write_calls: 1,
                write_calls: 1,
                after_read_calls: 1,
                after_read_all_calls: 1,
            }
        );
    }
}
