use std::collections::HashMap;

use crate::mindustry::entities::comp::{FireComp, FireTile};
use crate::mindustry::vars::TILE_SIZE;

pub const BASE_FIRE_LIFETIME: f32 = 1000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FireRules {
    pub net_client: bool,
    pub fire_enabled: bool,
    pub has_oxygen: bool,
}

impl Default for FireRules {
    fn default() -> Self {
        Self {
            net_client: false,
            fire_enabled: true,
            has_oxygen: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireCreateResult {
    Ignored,
    Created,
    Refreshed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExtinguishResult {
    pub found: bool,
    pub steam: bool,
    pub extinguished: bool,
    pub time: f32,
}

impl Default for ExtinguishResult {
    fn default() -> Self {
        Self {
            found: false,
            steam: false,
            extinguished: false,
            time: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fires {
    width: i32,
    height: i32,
    fires: HashMap<(i32, i32), FireComp>,
}

impl Fires {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width: width.max(0),
            height: height.max(0),
            fires: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.fires.len()
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn is_empty(&self) -> bool {
        self.fires.is_empty()
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    /// Mirrors `Fires.create(Tile)`: clients, disabled fire rules, missing
    /// oxygen, null tiles and out-of-bounds tiles are ignored.
    pub fn create(&mut self, tile: Option<FireTile>, rules: FireRules) -> FireCreateResult {
        if rules.net_client || !rules.fire_enabled || !rules.has_oxygen {
            return FireCreateResult::Ignored;
        }

        let Some(tile) = tile else {
            return FireCreateResult::Ignored;
        };
        if !self.in_bounds(tile.x, tile.y) {
            return FireCreateResult::Ignored;
        }

        if let Some(fire) = self.fires.get_mut(&(tile.x, tile.y)) {
            fire.lifetime = BASE_FIRE_LIFETIME;
            fire.time = 0.0;
            fire.tile = Some(tile);
            FireCreateResult::Refreshed
        } else {
            let mut fire = FireComp::new(
                tile.x as f32 * TILE_SIZE as f32,
                tile.y as f32 * TILE_SIZE as f32,
                BASE_FIRE_LIFETIME,
            );
            fire.tile = Some(tile);
            fire.registered = true;
            self.fires.insert((tile.x, tile.y), fire);
            FireCreateResult::Created
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&FireComp> {
        self.in_bounds(x, y)
            .then(|| self.fires.get(&(x, y)))
            .flatten()
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut FireComp> {
        if self.in_bounds(x, y) {
            self.fires.get_mut(&(x, y))
        } else {
            None
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = (&(i32, i32), &FireComp)> {
        self.fires.iter()
    }

    pub fn get_tile(&self, tile: Option<FireTile>) -> Option<&FireComp> {
        tile.and_then(|tile| self.get(tile.x, tile.y))
    }

    pub fn has(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }

        self.fires.get(&(x, y)).is_some_and(|fire| {
            !fire.removed
                && fire.registered
                && fire.lifetime > 0.0
                && fire.time / fire.lifetime < 1.0
                && fire.tile.is_some_and(|tile| tile.x == x && tile.y == y)
        })
    }

    /// Mirrors `Fires.extinguish(Tile, intensity)` and returns side-effect
    /// intents instead of firing events or playing effects directly.
    pub fn extinguish(
        &mut self,
        tile: Option<FireTile>,
        intensity: f32,
        delta: f32,
    ) -> ExtinguishResult {
        let Some(tile) = tile else {
            return ExtinguishResult::default();
        };

        let Some(fire) = self.get_mut(tile.x, tile.y) else {
            return ExtinguishResult::default();
        };

        fire.time += intensity * delta;
        ExtinguishResult {
            found: true,
            steam: true,
            extinguished: fire.time >= fire.lifetime,
            time: fire.time,
        }
    }

    pub fn remove(&mut self, tile: Option<FireTile>) -> Option<FireComp> {
        tile.and_then(|tile| self.fires.remove(&(tile.x, tile.y)))
    }

    pub fn register(&mut self, fire: FireComp) -> bool {
        let Some(tile) = fire.tile else {
            return false;
        };
        if !self.in_bounds(tile.x, tile.y) {
            return false;
        }

        self.fires.insert((tile.x, tile.y), fire);
        true
    }
}

impl Default for Fires {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile(x: i32, y: i32) -> FireTile {
        FireTile {
            x,
            y,
            build_present: true,
            flammability: 2.0,
        }
    }

    #[test]
    fn create_adds_fire_and_refreshes_existing_lifetime() {
        let mut fires = Fires::new(10, 10);

        assert_eq!(fires.width(), 10);
        assert_eq!(fires.height(), 10);
        assert_eq!(
            fires.create(Some(tile(2, 3)), FireRules::default()),
            FireCreateResult::Created
        );
        assert!(fires.has(2, 3));
        assert_eq!(fires.len(), 1);
        assert_eq!(fires.entries().count(), 1);
        let fire = fires.get(2, 3).unwrap();
        assert_eq!(fire.lifetime, BASE_FIRE_LIFETIME);
        assert_eq!((fire.x, fire.y), (16.0, 24.0));

        let fire = fires.get_mut(2, 3).unwrap();
        fire.time = 500.0;
        fire.lifetime = 600.0;
        assert_eq!(
            fires.create(Some(tile(2, 3)), FireRules::default()),
            FireCreateResult::Refreshed
        );
        let fire = fires.get(2, 3).unwrap();
        assert_eq!(fire.time, 0.0);
        assert_eq!(fire.lifetime, BASE_FIRE_LIFETIME);
        assert_eq!(fires.len(), 1);
    }

    #[test]
    fn create_ignores_client_disabled_rules_missing_tile_and_bounds() {
        let mut fires = Fires::new(2, 2);

        assert_eq!(
            fires.create(
                Some(tile(0, 0)),
                FireRules {
                    net_client: true,
                    ..FireRules::default()
                }
            ),
            FireCreateResult::Ignored
        );
        assert_eq!(
            fires.create(
                Some(tile(0, 0)),
                FireRules {
                    fire_enabled: false,
                    ..FireRules::default()
                }
            ),
            FireCreateResult::Ignored
        );
        assert_eq!(
            fires.create(
                Some(tile(0, 0)),
                FireRules {
                    has_oxygen: false,
                    ..FireRules::default()
                }
            ),
            FireCreateResult::Ignored
        );
        assert_eq!(
            fires.create(None, FireRules::default()),
            FireCreateResult::Ignored
        );
        assert_eq!(
            fires.create(Some(tile(3, 0)), FireRules::default()),
            FireCreateResult::Ignored
        );
        assert!(fires.is_empty());
    }

    #[test]
    fn has_requires_bounds_registered_active_fire_and_matching_tile() {
        let mut fires = Fires::new(4, 4);
        fires.create(Some(tile(1, 1)), FireRules::default());
        assert!(fires.has(1, 1));
        assert!(!fires.has(-1, 1));

        fires.get_mut(1, 1).unwrap().registered = false;
        assert!(!fires.has(1, 1));
        fires.get_mut(1, 1).unwrap().registered = true;
        fires.get_mut(1, 1).unwrap().time = BASE_FIRE_LIFETIME;
        assert!(!fires.has(1, 1));
        fires.get_mut(1, 1).unwrap().time = 0.0;
        fires.get_mut(1, 1).unwrap().tile = Some(tile(2, 1));
        assert!(!fires.has(1, 1));
    }

    #[test]
    fn extinguish_advances_time_and_reports_extinguish_trigger() {
        let mut fires = Fires::new(4, 4);
        fires.create(Some(tile(1, 1)), FireRules::default());

        let result = fires.extinguish(Some(tile(1, 1)), 2.0, 10.0);
        assert_eq!(
            result,
            ExtinguishResult {
                found: true,
                steam: true,
                extinguished: false,
                time: 20.0,
            }
        );

        let result = fires.extinguish(Some(tile(1, 1)), 200.0, 5.0);
        assert!(result.extinguished);
        assert_eq!(fires.get(1, 1).unwrap().time, 1020.0);
        assert_eq!(
            fires.extinguish(None, 1.0, 1.0),
            ExtinguishResult::default()
        );
    }

    #[test]
    fn remove_and_register_update_tile_slots() {
        let mut fires = Fires::new(4, 4);
        fires.create(Some(tile(1, 2)), FireRules::default());

        let removed = fires.remove(Some(tile(1, 2))).unwrap();
        assert!(!fires.has(1, 2));
        assert!(fires.register(removed));
        assert!(fires.has(1, 2));

        let mut no_tile = FireComp::new(0.0, 0.0, BASE_FIRE_LIFETIME);
        no_tile.tile = None;
        assert!(!fires.register(no_tile));
    }
}
