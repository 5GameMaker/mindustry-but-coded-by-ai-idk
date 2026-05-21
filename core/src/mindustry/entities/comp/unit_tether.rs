//! Unit tether component mirroring upstream `mindustry.entities.comp.UnitTetherComp`.

use crate::mindustry::io::TeamId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitTetherRef {
    pub id: i32,
    pub team: TeamId,
    pub valid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitTetherAction {
    Keep { spawner_unit_id: i32 },
    Despawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitTetherComp {
    pub team: TeamId,
    pub spawner: Option<UnitTetherRef>,
    pub spawner_unit_id: i32,
}

impl UnitTetherComp {
    pub const fn new(team: TeamId) -> Self {
        Self {
            team,
            spawner: None,
            spawner_unit_id: -1,
        }
    }

    pub fn after_read<F>(&mut self, resolve: F)
    where
        F: FnOnce(i32) -> Option<UnitTetherRef>,
    {
        self.resolve_spawner(resolve);
    }

    pub fn after_sync<F>(&mut self, resolve: F)
    where
        F: FnOnce(i32) -> Option<UnitTetherRef>,
    {
        self.resolve_spawner(resolve);
    }

    fn resolve_spawner<F>(&mut self, resolve: F)
    where
        F: FnOnce(i32) -> Option<UnitTetherRef>,
    {
        if self.spawner_unit_id != -1 {
            self.spawner = resolve(self.spawner_unit_id);
        }
        self.spawner_unit_id = -1;
    }

    pub fn update(&mut self) -> UnitTetherAction {
        if let Some(spawner) = self.spawner {
            if spawner.valid && spawner.team == self.team {
                self.spawner_unit_id = spawner.id;
                return UnitTetherAction::Keep {
                    spawner_unit_id: spawner.id,
                };
            }
        }

        UnitTetherAction::Despawn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_tether_after_read_resolves_spawner_id_and_clears_wire_id() {
        let mut tether = UnitTetherComp::new(TeamId(1));
        tether.spawner_unit_id = 42;

        tether.after_read(|id| {
            Some(UnitTetherRef {
                id,
                team: TeamId(1),
                valid: true,
            })
        });

        assert_eq!(
            tether.spawner,
            Some(UnitTetherRef {
                id: 42,
                team: TeamId(1),
                valid: true,
            })
        );
        assert_eq!(tether.spawner_unit_id, -1);
    }

    #[test]
    fn unit_tether_update_keeps_valid_same_team_spawner_and_despawns_otherwise() {
        let mut tether = UnitTetherComp::new(TeamId(1));
        tether.spawner = Some(UnitTetherRef {
            id: 7,
            team: TeamId(1),
            valid: true,
        });

        assert_eq!(
            tether.update(),
            UnitTetherAction::Keep { spawner_unit_id: 7 }
        );
        assert_eq!(tether.spawner_unit_id, 7);

        tether.spawner = Some(UnitTetherRef {
            id: 8,
            team: TeamId(2),
            valid: true,
        });
        assert_eq!(tether.update(), UnitTetherAction::Despawn);

        tether.spawner = None;
        assert_eq!(tether.update(), UnitTetherAction::Despawn);
    }
}
