//! Block-unit proxy component mirroring upstream
//! `mindustry.entities.comp.BlockUnitComp`.

use crate::mindustry::io::TeamId;

#[derive(Debug, Clone, PartialEq)]
pub struct BlockUnitBuilding {
    pub team: TeamId,
    pub health: f32,
    pub block_health: f32,
    pub block_size: i32,
    pub ui_icon: String,
    pub x: f32,
    pub y: f32,
    pub valid: bool,
    pub dead: bool,
    pub killed: bool,
    pub damage_events: Vec<(f32, bool)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockUnitComp {
    pub team: TeamId,
    pub tile: Option<BlockUnitBuilding>,
    pub max_health: f32,
    pub health: f32,
    pub hit_size: f32,
    pub x: f32,
    pub y: f32,
}

impl BlockUnitComp {
    pub const TILE_SIZE: f32 = 8.0;

    pub fn new(team: TeamId) -> Self {
        Self {
            team,
            tile: None,
            max_health: 0.0,
            health: 0.0,
            hit_size: 0.0,
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn tile(&mut self, tile: BlockUnitBuilding) {
        self.max_health = tile.block_health;
        self.health = tile.health;
        self.hit_size = tile.block_size as f32 * Self::TILE_SIZE * 0.7;
        self.x = tile.x;
        self.y = tile.y;
        self.team = tile.team;
        self.tile = Some(tile);
    }

    pub fn add(&self) -> Result<(), &'static str> {
        if self.tile.is_none() {
            Err("Do not add BlockUnit entities to the game, they will simply crash. Internal use only.")
        } else {
            Ok(())
        }
    }

    pub fn update(&mut self) {
        if let Some(tile) = &self.tile {
            self.team = tile.team;
        }
    }

    pub fn icon(&self) -> Option<&str> {
        self.tile.as_ref().map(|tile| tile.ui_icon.as_str())
    }

    pub fn killed(&mut self) {
        if let Some(tile) = &mut self.tile {
            tile.killed = true;
            tile.dead = true;
            tile.valid = false;
        }
    }

    pub fn damage(&mut self, amount: f32, with_effect: bool) {
        if let Some(tile) = &mut self.tile {
            tile.damage_events.push((amount, with_effect));
            tile.health -= amount;
            self.health = tile.health;
            if tile.health <= 0.0 {
                tile.dead = true;
                tile.valid = false;
            }
        }
    }

    pub fn dead(&self) -> bool {
        self.tile.as_ref().map(|tile| tile.dead).unwrap_or(true)
    }

    pub fn is_valid(&self) -> bool {
        self.tile.as_ref().map(|tile| tile.valid).unwrap_or(false)
    }

    pub fn is_added(&self) -> bool {
        self.is_valid()
    }

    pub fn set_team(&mut self, team: TeamId) {
        if let Some(tile) = &mut self.tile {
            if self.team != team {
                self.team = team;
                if tile.team != team {
                    tile.team = team;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn building() -> BlockUnitBuilding {
        BlockUnitBuilding {
            team: TeamId(1),
            health: 80.0,
            block_health: 100.0,
            block_size: 2,
            ui_icon: "router".into(),
            x: 16.0,
            y: 24.0,
            valid: true,
            dead: false,
            killed: false,
            damage_events: Vec::new(),
        }
    }

    #[test]
    fn block_unit_tile_sets_stats_from_building_proxy() {
        let mut unit = BlockUnitComp::new(TeamId(0));
        unit.tile(building());

        assert_eq!(unit.max_health, 100.0);
        assert_eq!(unit.health, 80.0);
        assert_eq!(unit.hit_size, 2.0 * BlockUnitComp::TILE_SIZE * 0.7);
        assert_eq!((unit.x, unit.y), (16.0, 24.0));
        assert_eq!(unit.icon(), Some("router"));
        assert!(unit.add().is_ok());
    }

    #[test]
    fn block_unit_without_tile_rejects_add_like_java_runtime_exception() {
        let unit = BlockUnitComp::new(TeamId(1));

        assert!(unit.add().is_err());
        assert!(unit.dead());
        assert!(!unit.is_valid());
    }

    #[test]
    fn block_unit_proxies_damage_kill_and_team_changes_to_building() {
        let mut unit = BlockUnitComp::new(TeamId(0));
        unit.tile(building());

        unit.damage(10.0, false);
        assert_eq!(unit.health, 70.0);
        assert_eq!(
            unit.tile.as_ref().unwrap().damage_events,
            vec![(10.0, false)]
        );

        unit.set_team(TeamId(2));
        assert_eq!(unit.team, TeamId(2));
        assert_eq!(unit.tile.as_ref().unwrap().team, TeamId(2));

        unit.killed();
        assert!(unit.dead());
        assert!(unit.tile.as_ref().unwrap().killed);
    }
}
