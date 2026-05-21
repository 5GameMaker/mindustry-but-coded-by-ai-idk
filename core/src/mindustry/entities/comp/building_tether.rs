//! Building tether component mirroring upstream
//! `mindustry.entities.comp.BuildingTetherComp`.

use crate::mindustry::io::TeamId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingTetherRef {
    pub team: TeamId,
    pub valid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingTetherAction {
    Keep,
    Despawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingTetherComp {
    pub team: TeamId,
    pub building: Option<BuildingTetherRef>,
}

impl BuildingTetherComp {
    pub const fn new(team: TeamId) -> Self {
        Self {
            team,
            building: None,
        }
    }

    pub fn update(&self) -> BuildingTetherAction {
        if let Some(building) = self.building {
            if building.valid && building.team == self.team {
                return BuildingTetherAction::Keep;
            }
        }

        BuildingTetherAction::Despawn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_tether_keeps_only_valid_same_team_buildings() {
        let mut tether = BuildingTetherComp::new(TeamId(1));

        assert_eq!(tether.update(), BuildingTetherAction::Despawn);

        tether.building = Some(BuildingTetherRef {
            team: TeamId(1),
            valid: true,
        });
        assert_eq!(tether.update(), BuildingTetherAction::Keep);

        tether.building = Some(BuildingTetherRef {
            team: TeamId(2),
            valid: true,
        });
        assert_eq!(tether.update(), BuildingTetherAction::Despawn);

        tether.building = Some(BuildingTetherRef {
            team: TeamId(1),
            valid: false,
        });
        assert_eq!(tether.update(), BuildingTetherAction::Despawn);
    }
}
