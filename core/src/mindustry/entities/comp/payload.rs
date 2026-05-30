//! Payload component shell mirroring upstream `mindustry.entities.comp.PayloadComp`.

use crate::mindustry::{io::TeamId, r#type::PayloadKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadKind {
    Unit,
    Build,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PayloadState {
    pub kind: PayloadKind,
    pub size: f32,
    pub key: Option<PayloadKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PayloadComp {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub team: TeamId,
    pub payload_capacity: f32,
    pub pickup_units: bool,
    pub unit_payloads_explode: bool,
    pub payloads: Vec<PayloadState>,
    pub removed_payloads: usize,
    pub destroyed_payloads: usize,
}

impl PayloadComp {
    pub fn new(team: TeamId, payload_capacity: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            team,
            payload_capacity,
            pickup_units: false,
            unit_payloads_explode: false,
            payloads: Vec::new(),
            removed_payloads: 0,
            destroyed_payloads: 0,
        }
    }

    pub fn update(&mut self) {
        // Real payload position/power updates are delegated to payload systems later.
    }

    pub fn remove(&mut self) {
        self.removed_payloads += self.payloads.len();
        self.payloads.clear();
    }

    pub fn destroy(&mut self) {
        if self.unit_payloads_explode {
            self.destroyed_payloads += self.payloads.len();
        }
    }

    pub fn payload_used(&self) -> f32 {
        self.payloads.iter().map(|p| p.size * p.size).sum()
    }

    pub fn can_pickup_unit(
        &self,
        hit_size: f32,
        unit_team: TeamId,
        is_ai: bool,
        allowed_in_payloads: bool,
    ) -> bool {
        self.pickup_units
            && self.payload_used() + hit_size * hit_size <= self.payload_capacity + 0.001
            && unit_team == self.team
            && is_ai
            && allowed_in_payloads
    }

    pub fn can_pickup_build(&self, block_size: f32, build_team: TeamId, can_pickup: bool) -> bool {
        self.payload_used() + block_size * block_size * 8.0 * 8.0 <= self.payload_capacity + 0.001
            && can_pickup
            && build_team == self.team
    }

    pub fn can_pickup_payload(&self, payload: &PayloadState) -> bool {
        self.payload_used() + payload.size * payload.size <= self.payload_capacity + 0.001
            && (self.pickup_units || payload.kind != PayloadKind::Unit)
    }

    pub fn has_payload(&self) -> bool {
        !self.payloads.is_empty()
    }

    pub fn first_draw_payload(&self) -> Option<&PayloadState> {
        self.payloads.first()
    }

    pub fn add_payload(&mut self, load: PayloadState) {
        self.payloads.push(load);
    }

    pub fn drop_last_payload<F>(&mut self, try_drop: F) -> bool
    where
        F: FnOnce(&PayloadState) -> bool,
    {
        let Some(load) = self.payloads.last() else {
            return false;
        };
        if try_drop(load) {
            self.payloads.pop();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_component_tracks_used_area_and_pickup_rules() {
        let mut comp = PayloadComp::new(TeamId(1), 100.0);
        comp.pickup_units = true;
        comp.add_payload(PayloadState {
            kind: PayloadKind::Build,
            size: 4.0,
            key: None,
        });

        assert_eq!(comp.payload_used(), 16.0);
        assert!(comp.can_pickup_unit(5.0, TeamId(1), true, true));
        assert!(!comp.can_pickup_unit(10.0, TeamId(1), true, true));
        assert!(comp.can_pickup_payload(&PayloadState {
            kind: PayloadKind::Unit,
            size: 2.0,
            key: None,
        }));
    }

    #[test]
    fn payload_component_build_pickup_uses_tile_area_like_java() {
        let comp = PayloadComp::new(TeamId(2), 300.0);

        assert!(comp.can_pickup_build(2.0, TeamId(2), true));
        assert!(!comp.can_pickup_build(3.0, TeamId(2), true));
        assert!(!comp.can_pickup_build(2.0, TeamId(1), true));
    }

    #[test]
    fn payload_component_drop_remove_and_destroy_follow_container_shape() {
        let mut comp = PayloadComp::new(TeamId(1), 100.0);
        comp.unit_payloads_explode = true;
        comp.add_payload(PayloadState {
            kind: PayloadKind::Unit,
            size: 2.0,
            key: None,
        });
        comp.add_payload(PayloadState {
            kind: PayloadKind::Build,
            size: 3.0,
            key: None,
        });

        assert!(comp.has_payload());
        assert!(comp.drop_last_payload(|payload| payload.kind == PayloadKind::Build));
        assert_eq!(comp.payloads.len(), 1);

        comp.destroy();
        assert_eq!(comp.destroyed_payloads, 1);
        comp.remove();
        assert_eq!(comp.removed_payloads, 1);
        assert!(!comp.has_payload());
    }

    #[test]
    fn payload_component_draw_uses_first_payload_like_java() {
        let mut comp = PayloadComp::new(TeamId(1), 100.0);
        comp.add_payload(PayloadState {
            kind: PayloadKind::Unit,
            size: 2.0,
            key: None,
        });
        comp.add_payload(PayloadState {
            kind: PayloadKind::Build,
            size: 3.0,
            key: None,
        });

        let draw_payload = comp
            .first_draw_payload()
            .expect("drawPayload should use payloads().first()");
        assert_eq!(draw_payload.kind, PayloadKind::Unit);
        assert_eq!(draw_payload.size, 2.0);
        assert!(comp.drop_last_payload(|payload| payload.kind == PayloadKind::Build));
        assert_eq!(
            comp.first_draw_payload()
                .expect("first payload remains after dropping last")
                .kind,
            PayloadKind::Unit
        );
    }
}
