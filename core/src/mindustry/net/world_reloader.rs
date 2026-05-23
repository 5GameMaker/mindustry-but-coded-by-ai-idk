//! Rust-side state planner for upstream `mindustry.net.WorldReloader`.
//!
//! Java `WorldReloader` is tightly coupled to global `Vars`, `Groups.player`
//! and generated `Call` helpers.  This module keeps the same begin/end
//! semantics as pure state transitions plus explicit actions so the migrated
//! server/client layers can wire the actions to their runtime transport.

use crate::mindustry::io::TeamId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldReloadPlayer {
    pub connection_id: i32,
    pub local: bool,
    pub connected: bool,
    pub admin: bool,
    pub team: TeamId,
    pub has_unit: bool,
}

impl WorldReloadPlayer {
    pub const fn remote(connection_id: i32, team: TeamId) -> Self {
        Self {
            connection_id,
            local: false,
            connected: true,
            admin: false,
            team,
            has_unit: true,
        }
    }

    pub const fn local(connection_id: i32, team: TeamId) -> Self {
        Self {
            connection_id,
            local: true,
            connected: true,
            admin: false,
            team,
            has_unit: true,
        }
    }

    pub fn clear_unit(&mut self) {
        self.has_unit = false;
    }

    pub fn reset_like_java_player_reset(&mut self) {
        self.has_unit = false;
        self.team = TeamId(0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoredReloadPlayer {
    pub connection_id: i32,
    pub admin: bool,
    pub team: TeamId,
}

impl From<&WorldReloadPlayer> for StoredReloadPlayer {
    fn from(player: &WorldReloadPlayer) -> Self {
        Self {
            connection_id: player.connection_id,
            admin: player.admin,
            team: player.team,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldReloadBeginAction {
    ClearPlayerUnit(i32),
    ResetLogic,
    ResetNet,
    SendWorldDataBegin,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldReloadBeginPlan {
    pub already_began: bool,
    pub was_server: bool,
    pub actions: Vec<WorldReloadBeginAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldReloadEndAction {
    ResetPlayer {
        connection_id: i32,
        preserved_admin: bool,
    },
    AssignTeam {
        connection_id: i32,
        team: TeamId,
    },
    SendWorldData {
        connection_id: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldReloadEndPlan {
    pub was_server: bool,
    pub actions: Vec<WorldReloadEndAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldReloader {
    players: Vec<StoredReloadPlayer>,
    was_server: bool,
    began: bool,
}

impl WorldReloader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn began(&self) -> bool {
        self.began
    }

    pub fn was_server(&self) -> bool {
        self.was_server
    }

    pub fn stored_players(&self) -> &[StoredReloadPlayer] {
        &self.players
    }

    pub fn begin(
        &mut self,
        net_server: bool,
        net_client: bool,
        players: &mut [WorldReloadPlayer],
    ) -> WorldReloadBeginPlan {
        if self.began {
            return WorldReloadBeginPlan {
                already_began: true,
                was_server: self.was_server,
                actions: Vec::new(),
            };
        }

        self.was_server = net_server;
        let mut actions = Vec::new();

        if self.was_server {
            self.players.clear();

            for player in players.iter_mut().filter(|player| !player.local) {
                self.players.push(StoredReloadPlayer::from(&*player));
                player.clear_unit();
                actions.push(WorldReloadBeginAction::ClearPlayerUnit(
                    player.connection_id,
                ));
            }

            actions.push(WorldReloadBeginAction::ResetLogic);
            actions.push(WorldReloadBeginAction::SendWorldDataBegin);
        } else {
            if net_client {
                actions.push(WorldReloadBeginAction::ResetNet);
            }
            actions.push(WorldReloadBeginAction::ResetLogic);
        }

        self.began = true;
        WorldReloadBeginPlan {
            already_began: false,
            was_server: self.was_server,
            actions,
        }
    }

    pub fn end<F>(
        &self,
        players: &mut [WorldReloadPlayer],
        pvp: bool,
        mut assign_team: F,
    ) -> WorldReloadEndPlan
    where
        F: FnMut(&StoredReloadPlayer, &[StoredReloadPlayer]) -> TeamId,
    {
        if !self.was_server {
            return WorldReloadEndPlan {
                was_server: false,
                actions: Vec::new(),
            };
        }

        let mut actions = Vec::new();
        for stored in &self.players {
            let Some(player) = players
                .iter_mut()
                .find(|player| player.connection_id == stored.connection_id)
            else {
                continue;
            };
            if !player.connected {
                continue;
            }

            let was_admin = player.admin;
            player.reset_like_java_player_reset();
            player.admin = was_admin;
            actions.push(WorldReloadEndAction::ResetPlayer {
                connection_id: stored.connection_id,
                preserved_admin: was_admin,
            });

            if pvp {
                let team = assign_team(stored, &self.players);
                player.team = team;
                actions.push(WorldReloadEndAction::AssignTeam {
                    connection_id: stored.connection_id,
                    team,
                });
            }

            actions.push(WorldReloadEndAction::SendWorldData {
                connection_id: stored.connection_id,
            });
        }

        WorldReloadEndPlan {
            was_server: true,
            actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_as_server_stores_remote_players_clears_units_and_broadcasts_begin() {
        let mut reloader = WorldReloader::new();
        let mut players = vec![
            WorldReloadPlayer::local(1, TeamId(1)),
            WorldReloadPlayer::remote(2, TeamId(2)),
            WorldReloadPlayer {
                admin: true,
                ..WorldReloadPlayer::remote(3, TeamId(3))
            },
        ];

        let plan = reloader.begin(true, false, &mut players);

        assert!(reloader.began());
        assert!(reloader.was_server());
        assert_eq!(
            reloader.stored_players(),
            &[
                StoredReloadPlayer {
                    connection_id: 2,
                    admin: false,
                    team: TeamId(2)
                },
                StoredReloadPlayer {
                    connection_id: 3,
                    admin: true,
                    team: TeamId(3)
                }
            ]
        );
        assert!(players[0].has_unit);
        assert!(!players[1].has_unit);
        assert!(!players[2].has_unit);
        assert_eq!(
            plan.actions,
            vec![
                WorldReloadBeginAction::ClearPlayerUnit(2),
                WorldReloadBeginAction::ClearPlayerUnit(3),
                WorldReloadBeginAction::ResetLogic,
                WorldReloadBeginAction::SendWorldDataBegin
            ]
        );
    }

    #[test]
    fn begin_is_single_shot_like_java_guard() {
        let mut reloader = WorldReloader::new();
        let mut players = vec![WorldReloadPlayer::remote(2, TeamId(2))];

        assert!(!reloader.begin(true, false, &mut players).already_began);
        let second = reloader.begin(true, false, &mut players);

        assert!(second.already_began);
        assert!(second.actions.is_empty());
        assert_eq!(reloader.stored_players().len(), 1);
    }

    #[test]
    fn begin_as_client_resets_net_then_logic_without_storing_players() {
        let mut reloader = WorldReloader::new();
        let mut players = vec![WorldReloadPlayer::remote(2, TeamId(2))];

        let plan = reloader.begin(false, true, &mut players);

        assert!(!plan.was_server);
        assert!(reloader.stored_players().is_empty());
        assert_eq!(
            plan.actions,
            vec![
                WorldReloadBeginAction::ResetNet,
                WorldReloadBeginAction::ResetLogic
            ]
        );
        assert!(players[0].has_unit);
    }

    #[test]
    fn end_as_server_resets_connected_players_preserves_admin_and_sends_world_data() {
        let mut reloader = WorldReloader::new();
        let mut players = vec![
            WorldReloadPlayer::remote(2, TeamId(2)),
            WorldReloadPlayer {
                admin: true,
                ..WorldReloadPlayer::remote(3, TeamId(3))
            },
            WorldReloadPlayer {
                connected: false,
                ..WorldReloadPlayer::remote(4, TeamId(4))
            },
        ];
        reloader.begin(true, false, &mut players);

        let plan = reloader.end(&mut players, false, |stored, _| stored.team);

        assert_eq!(players[0].team, TeamId(0));
        assert_eq!(players[1].team, TeamId(0));
        assert!(players[1].admin);
        assert_eq!(players[2].team, TeamId(4));
        assert_eq!(
            plan.actions,
            vec![
                WorldReloadEndAction::ResetPlayer {
                    connection_id: 2,
                    preserved_admin: false,
                },
                WorldReloadEndAction::SendWorldData { connection_id: 2 },
                WorldReloadEndAction::ResetPlayer {
                    connection_id: 3,
                    preserved_admin: true,
                },
                WorldReloadEndAction::SendWorldData { connection_id: 3 },
            ]
        );
    }

    #[test]
    fn end_as_server_reassigns_pvp_team_before_world_data() {
        let mut reloader = WorldReloader::new();
        let mut players = vec![
            WorldReloadPlayer::remote(2, TeamId(2)),
            WorldReloadPlayer::remote(3, TeamId(3)),
        ];
        reloader.begin(true, false, &mut players);

        let plan = reloader.end(&mut players, true, |stored, all| {
            TeamId(stored.connection_id as u8 + all.len() as u8)
        });

        assert_eq!(players[0].team, TeamId(4));
        assert_eq!(players[1].team, TeamId(5));
        assert_eq!(
            plan.actions,
            vec![
                WorldReloadEndAction::ResetPlayer {
                    connection_id: 2,
                    preserved_admin: false,
                },
                WorldReloadEndAction::AssignTeam {
                    connection_id: 2,
                    team: TeamId(4),
                },
                WorldReloadEndAction::SendWorldData { connection_id: 2 },
                WorldReloadEndAction::ResetPlayer {
                    connection_id: 3,
                    preserved_admin: false,
                },
                WorldReloadEndAction::AssignTeam {
                    connection_id: 3,
                    team: TeamId(5),
                },
                WorldReloadEndAction::SendWorldData { connection_id: 3 },
            ]
        );
    }

    #[test]
    fn end_without_server_begin_does_nothing_like_java() {
        let mut reloader = WorldReloader::new();
        let mut players = vec![WorldReloadPlayer::remote(2, TeamId(2))];
        reloader.begin(false, false, &mut players);

        let plan = reloader.end(&mut players, true, |stored, _| stored.team);

        assert!(!plan.was_server);
        assert!(plan.actions.is_empty());
        assert_eq!(players[0].team, TeamId(2));
    }
}
