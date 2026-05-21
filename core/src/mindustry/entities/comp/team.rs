//! Team component mirroring upstream `mindustry.entities.comp.TeamComp`.

use crate::mindustry::entities::comp::PosComp;
use crate::mindustry::game::TEAM_DERELICT;
use crate::mindustry::io::TeamId;
use crate::mindustry::world::BuildingRef;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TeamRulesView {
    pub cheat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TeamComp {
    pub pos: PosComp,
    pub team: TeamId,
}

impl TeamComp {
    pub const fn new(x: f32, y: f32, team: TeamId) -> Self {
        Self {
            pos: PosComp::new(x, y),
            team,
        }
    }

    pub const fn derelict(x: f32, y: f32) -> Self {
        Self::new(x, y, TeamId(TEAM_DERELICT))
    }

    pub fn cheating(&self, rules: TeamRulesView) -> bool {
        rules.cheat
    }

    /// Java: `this.team != viewer && !fogControl.isVisible(viewer, x, y)`.
    pub fn in_fog_to(&self, viewer: TeamId, visible_to_viewer: bool) -> bool {
        self.team != viewer && !visible_to_viewer
    }

    pub fn core_with<F>(&self, lookup: F) -> Option<BuildingRef>
    where
        F: FnOnce(TeamId) -> Option<BuildingRef>,
    {
        lookup(self.team)
    }

    pub fn closest_core_with<F>(&self, lookup: F) -> Option<BuildingRef>
    where
        F: FnOnce(f32, f32, TeamId) -> Option<BuildingRef>,
    {
        lookup(self.pos.x, self.pos.y, self.team)
    }

    pub fn closest_enemy_core_with<F>(&self, lookup: F) -> Option<BuildingRef>
    where
        F: FnOnce(f32, f32, TeamId) -> Option<BuildingRef>,
    {
        lookup(self.pos.x, self.pos.y, self.team)
    }
}

impl Default for TeamComp {
    fn default() -> Self {
        Self::derelict(0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::world::point2_pack;

    #[test]
    fn team_component_defaults_to_derelict_team() {
        let team = TeamComp::default();

        assert_eq!(team.team, TeamId(TEAM_DERELICT));
        assert_eq!(team.pos, PosComp::new(0.0, 0.0));
    }

    #[test]
    fn team_component_cheat_and_fog_checks_follow_java_conditions() {
        let team = TeamComp::new(10.0, 20.0, TeamId(2));

        assert!(team.cheating(TeamRulesView { cheat: true }));
        assert!(!team.cheating(TeamRulesView { cheat: false }));
        assert!(!team.in_fog_to(TeamId(2), false));
        assert!(!team.in_fog_to(TeamId(3), true));
        assert!(team.in_fog_to(TeamId(3), false));
    }

    #[test]
    fn team_component_core_queries_are_explicit_runtime_callbacks() {
        let team = TeamComp::new(8.0, 16.0, TeamId(1));
        let core = BuildingRef {
            tile_pos: point2_pack(3, 4),
            block: 9,
            team: 1,
            rotation: 0,
        };

        assert_eq!(
            team.core_with(|id| (id == TeamId(1)).then_some(core)),
            Some(core)
        );
        assert_eq!(
            team.closest_core_with(|x, y, id| {
                (x == 8.0 && y == 16.0 && id == TeamId(1)).then_some(core)
            }),
            Some(core)
        );
        assert_eq!(
            team.closest_enemy_core_with(|_, _, id| (id != TeamId(2)).then_some(core)),
            Some(core)
        );
    }
}
