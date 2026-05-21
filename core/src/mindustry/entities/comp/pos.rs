//! Position component mirroring upstream `mindustry.entities.comp.PosComp`.

use crate::mindustry::core::world::World;
use crate::mindustry::entities::EntityPosition;
use crate::mindustry::world::{BlockId, BuildingRef, Tile};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PosComp {
    pub x: f32,
    pub y: f32,
}

impl PosComp {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn set(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_position(&mut self, pos: impl EntityPosition) {
        self.set(pos.x(), pos.y());
    }

    pub fn trns(&mut self, x: f32, y: f32) {
        self.set(self.x + x, self.y + y);
    }

    pub fn trns_position(&mut self, pos: impl EntityPosition) {
        self.trns(pos.x(), pos.y());
    }

    pub fn tile_x(&self) -> i32 {
        World::to_tile(self.x)
    }

    pub fn tile_y(&self) -> i32 {
        World::to_tile(self.y)
    }

    /// Java returns air if this unit is on a non-air top block.
    pub fn floor_on(&self, world: &World) -> BlockId {
        self.tile_on(world)
            .map(|tile| {
                if tile.block != Tile::AIR {
                    Tile::AIR
                } else {
                    tile.floor
                }
            })
            .unwrap_or(Tile::AIR)
    }

    pub fn block_on(&self, world: &World) -> BlockId {
        self.tile_on(world)
            .map(|tile| tile.block)
            .unwrap_or(Tile::AIR)
    }

    pub fn build_on(&self, world: &World) -> Option<BuildingRef> {
        world.build_world(self.x, self.y)
    }

    pub fn tile_on<'a>(&self, world: &'a World) -> Option<&'a Tile> {
        world.tile_world(self.x, self.y)
    }

    pub fn on_solid(&self, world: &World) -> bool {
        self.tile_on(world)
            .map(|tile| tile.block != Tile::AIR)
            .unwrap_or(true)
    }
}

impl EntityPosition for PosComp {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::world::BuildingRef as TileBuildingRef;

    #[derive(Debug, Clone, Copy)]
    struct Offset {
        x: f32,
        y: f32,
    }

    impl EntityPosition for Offset {
        fn x(&self) -> f32 {
            self.x
        }

        fn y(&self) -> f32 {
            self.y
        }
    }

    #[test]
    fn pos_component_sets_translates_and_converts_to_tile_coordinates() {
        let mut pos = PosComp::new(1.0, 2.0);
        pos.set(4.0, 5.0);
        pos.trns(4.0, 7.0);

        assert_eq!((pos.x(), pos.y()), (8.0, 12.0));
        assert_eq!((pos.tile_x(), pos.tile_y()), (1, 2));

        pos.set_position(Offset { x: 16.0, y: 24.0 });
        pos.trns_position(Offset { x: 8.0, y: -8.0 });
        assert_eq!((pos.tile_x(), pos.tile_y()), (3, 2));
    }

    #[test]
    fn pos_component_world_queries_follow_java_air_and_solid_branches() {
        let mut world = World::new();
        world.resize(4, 4);
        {
            let tile = world.tile_mut(1, 1).unwrap();
            tile.floor = 5;
            tile.block = Tile::AIR;
        }
        {
            let tile = world.tile_mut(2, 2).unwrap();
            tile.floor = 6;
            tile.block = 9;
            tile.build = Some(TileBuildingRef {
                tile_pos: tile.pos(),
                block: 9,
                team: 1,
                rotation: 0,
            });
        }

        let pos = PosComp::new(8.0, 8.0);
        assert_eq!(pos.floor_on(&world), 5);
        assert_eq!(pos.block_on(&world), Tile::AIR);
        assert!(!pos.on_solid(&world));

        let pos = PosComp::new(16.0, 16.0);
        assert_eq!(pos.floor_on(&world), Tile::AIR);
        assert_eq!(pos.block_on(&world), 9);
        assert!(pos.on_solid(&world));
        assert!(pos.build_on(&world).is_some());

        let out_of_world = PosComp::new(-100.0, -100.0);
        assert_eq!(out_of_world.floor_on(&world), Tile::AIR);
        assert!(out_of_world.on_solid(&world));
    }
}
