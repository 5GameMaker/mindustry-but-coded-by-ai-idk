//! Incremental Rust counterpart of upstream `mindustry.core.World`.
//!
//! This module ports the pure tile-container and coordinate portions first.
//! Heavy Java side effects (`Events`, `Groups`, legacy block removal, sector
//! generators) are represented as explicit state updates so the rest of the
//! Rust runtime can depend on a stable world shell while those systems are
//! migrated separately.

use crate::mindustry::{
    vars::TILE_SIZE,
    world::{point2_x, point2_y, BlockId, BuildingRef, Tile, Tiles},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorldContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldLoadEventKind {
    Begin,
    End,
    Loaded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockSolidity {
    pub solid: bool,
    pub fills_tile: bool,
}

impl BlockSolidity {
    pub const fn new(solid: bool, fills_tile: bool) -> Self {
        Self { solid, fills_tile }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
    pub context: WorldContext,
    pub tiles: Tiles,
    /// Number of tile changes in this session. Starts at 1 like Java.
    pub tile_changes: i32,
    /// Number of floor changes in this session. Starts at 1 like Java.
    pub floor_changes: i32,
    generating: bool,
    invalid_map: bool,
    load_events: Vec<WorldLoadEventKind>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            context: WorldContext,
            tiles: Tiles::new(0, 0),
            tile_changes: 1,
            floor_changes: 1,
            generating: false,
            invalid_map: false,
            load_events: Vec::new(),
        }
    }

    pub fn is_invalid_map(&self) -> bool {
        self.invalid_map
    }

    pub fn set_invalid_map(&mut self, invalid_map: bool) {
        self.invalid_map = invalid_map;
    }

    pub fn solid(&self, x: i32, y: i32) -> bool {
        self.wall_solid(x, y)
    }

    pub fn passable(&self, x: i32, y: i32) -> bool {
        self.tile(x, y).is_some() && !self.wall_solid(x, y)
    }

    pub fn wall_solid(&self, x: i32, y: i32) -> bool {
        self.tile(x, y)
            .map(|tile| tile.block != Tile::AIR)
            .unwrap_or(true)
    }

    pub fn wall_solid_full(&self, x: i32, y: i32) -> bool {
        self.tile(x, y)
            .map(|tile| tile.block != Tile::AIR)
            .unwrap_or(true)
    }

    pub fn wall_solid_with<F>(&self, x: i32, y: i32, block: F) -> bool
    where
        F: Fn(BlockId) -> Option<BlockSolidity>,
    {
        self.tile(x, y)
            .map(|tile| {
                block(tile.block)
                    .map(|block| block.solid)
                    .unwrap_or(tile.block != Tile::AIR)
            })
            .unwrap_or(true)
    }

    pub fn wall_solid_full_with<F>(&self, x: i32, y: i32, block: F) -> bool
    where
        F: Fn(BlockId) -> Option<BlockSolidity>,
    {
        self.tile(x, y)
            .map(|tile| {
                block(tile.block)
                    .map(|block| block.solid && block.fills_tile)
                    .unwrap_or(tile.block != Tile::AIR)
            })
            .unwrap_or(true)
    }

    pub fn is_accessible(&self, x: i32, y: i32) -> bool {
        !self.wall_solid(x, y - 1)
            || !self.wall_solid(x, y + 1)
            || !self.wall_solid(x - 1, y)
            || !self.wall_solid(x + 1, y)
    }

    pub fn width(&self) -> usize {
        self.tiles.width
    }

    pub fn height(&self) -> usize {
        self.tiles.height
    }

    pub fn unit_width(&self) -> i32 {
        self.width() as i32 * TILE_SIZE
    }

    pub fn unit_height(&self) -> i32 {
        self.height() as i32 * TILE_SIZE
    }

    pub fn floor_id(&self, x: i32, y: i32) -> BlockId {
        self.tile(x, y).map(Tile::floor_id).unwrap_or(Tile::AIR)
    }

    pub fn floor_world_id(&self, x: f32, y: f32) -> BlockId {
        self.tile_world(x, y)
            .map(Tile::floor_id)
            .unwrap_or(Tile::AIR)
    }

    pub fn tile_pos(&self, pos: i32) -> Option<&Tile> {
        self.tile(point2_x(pos) as i32, point2_y(pos) as i32)
    }

    pub fn tile(&self, x: i32, y: i32) -> Option<&Tile> {
        self.tiles.get(x, y)
    }

    pub fn tile_mut(&mut self, x: i32, y: i32) -> Option<&mut Tile> {
        self.tiles.get_mut(x, y)
    }

    pub fn tile_building(&self, x: i32, y: i32) -> Option<&Tile> {
        let tile = self.tiles.get(x, y)?;
        if let Some(build) = tile.build {
            self.tile_pos(build.tile_pos).or(Some(tile))
        } else {
            Some(tile)
        }
    }

    pub fn build(&self, x: i32, y: i32) -> Option<BuildingRef> {
        self.tile(x, y).and_then(|tile| tile.build)
    }

    pub fn build_pos(&self, pos: i32) -> Option<BuildingRef> {
        self.tile_pos(pos).and_then(|tile| tile.build)
    }

    pub fn raw_tile(&self, x: i32, y: i32) -> Result<&Tile, String> {
        self.tiles.getn(x, y)
    }

    pub fn tile_world(&self, x: f32, y: f32) -> Option<&Tile> {
        self.tile(Self::to_tile(x), Self::to_tile(y))
    }

    pub fn build_world(&self, x: f32, y: f32) -> Option<BuildingRef> {
        self.build(Self::to_tile(x), Self::to_tile(y))
    }

    /// Convert from world to logic tile coordinates. Whole numbers are tile centers.
    pub fn conv(coord: f32) -> f32 {
        coord / TILE_SIZE as f32
    }

    /// Convert from tile to world coordinates.
    pub fn unconv(coord: f32) -> f32 {
        coord * TILE_SIZE as f32
    }

    pub fn to_tile(coord: f32) -> i32 {
        (coord / TILE_SIZE as f32).round() as i32
    }

    pub fn pack_array(&self, x: i32, y: i32) -> i32 {
        x + y * self.tiles.width as i32
    }

    pub fn clear_buildings(&mut self) {
        for tile in self.tiles.iter_mut() {
            tile.build = None;
        }
    }

    /// Resizes the tile array and returns the resulting tile container.
    /// Only use for loading saves, matching the Java method contract.
    pub fn resize(&mut self, width: usize, height: usize) -> &mut Tiles {
        self.clear_buildings();
        if self.tiles.width != width || self.tiles.height != height {
            self.tiles = Tiles::new(width, height);
        }
        &mut self.tiles
    }

    /// Signifies the beginning of map loading.
    pub fn begin_map_load(&mut self) {
        self.generating = true;
        self.load_events.push(WorldLoadEventKind::Begin);
    }

    /// Signifies the end of map loading.
    pub fn end_map_load(&mut self) {
        self.load_events.push(WorldLoadEventKind::End);
        self.generating = false;
        // Java's WorldLoadEvent listener resets both counters to -1.
        self.tile_changes = -1;
        self.floor_changes = -1;
        self.load_events.push(WorldLoadEventKind::Loaded);
    }

    pub fn load_generator<F>(&mut self, width: usize, height: usize, generator: F)
    where
        F: FnOnce(&mut Tiles),
    {
        self.begin_map_load();
        self.resize(width, height);
        generator(&mut self.tiles);
        self.end_map_load();
    }

    pub fn set_generating(&mut self, generating: bool) {
        self.generating = generating;
    }

    pub fn is_generating(&self) -> bool {
        self.generating
    }

    pub fn load_events(&self) -> &[WorldLoadEventKind] {
        &self.load_events
    }

    pub fn clear_load_events(&mut self) {
        self.load_events.clear();
    }

    pub fn note_tile_change(&mut self) {
        self.tile_changes += 1;
    }

    pub fn note_floor_change(&mut self) {
        self.floor_changes += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::world::point2_pack;

    #[test]
    fn world_defaults_match_java_field_initializers() {
        let world = World::new();

        assert_eq!(world.width(), 0);
        assert_eq!(world.height(), 0);
        assert_eq!(world.tile_changes, 1);
        assert_eq!(world.floor_changes, 1);
        assert!(!world.is_generating());
        assert!(!world.is_invalid_map());
        assert!(world.load_events().is_empty());
    }

    #[test]
    fn world_coordinates_follow_java_tilesize_rounding() {
        assert_eq!(World::conv(16.0), 2.0);
        assert_eq!(World::unconv(3.0), 24.0);
        assert_eq!(World::to_tile(0.0), 0);
        assert_eq!(World::to_tile(3.9), 0);
        assert_eq!(World::to_tile(4.0), 1);
        assert_eq!(World::to_tile(12.0), 2);
    }

    #[test]
    fn resize_tile_lookup_and_world_lookup_match_java_helpers() {
        let mut world = World::new();
        world.resize(3, 2);

        assert_eq!(world.width(), 3);
        assert_eq!(world.height(), 2);
        assert_eq!(world.unit_width(), 24);
        assert_eq!(world.unit_height(), 16);
        assert_eq!(world.pack_array(2, 1), 5);
        assert!(world.tile(2, 1).is_some());
        assert!(world.tile(3, 1).is_none());
        assert_eq!(world.tile_pos(point2_pack(2, 1)).unwrap().x, 2);
        assert_eq!(
            world.tile_world(16.0, 8.0).unwrap().pos(),
            point2_pack(2, 1)
        );
        assert!(world.raw_tile(99, 0).is_err());
    }

    #[test]
    fn building_helpers_follow_center_tile_reference() {
        let mut world = World::new();
        world.resize(2, 2);
        let center = point2_pack(1, 1);
        let build = BuildingRef {
            tile_pos: center,
            block: 9,
            team: 1,
            rotation: 2,
        };

        world.tile_mut(1, 1).unwrap().build = Some(build);
        world.tile_mut(0, 0).unwrap().build = Some(build);

        assert_eq!(world.build(0, 0), Some(build));
        assert_eq!(world.build_pos(point2_pack(1, 1)), Some(build));
        assert_eq!(world.tile_building(0, 0).unwrap().pos(), center);

        world.clear_buildings();
        assert_eq!(world.build(1, 1), None);
    }

    #[test]
    fn solidity_helpers_treat_out_of_bounds_as_solid_and_allow_metadata_lookup() {
        let mut world = World::new();
        world.resize(2, 1);

        assert!(world.wall_solid(-1, 0));
        assert!(world.passable(0, 0));
        assert!(world.is_accessible(0, 0));

        world.tile_mut(0, 0).unwrap().block = 7;
        assert!(world.wall_solid(0, 0));
        assert!(!world.passable(0, 0));

        assert!(!world.wall_solid_with(0, 0, |_| Some(BlockSolidity::new(false, false))));
        assert!(!world.wall_solid_full_with(0, 0, |_| Some(BlockSolidity::new(true, false))));
        assert!(world.wall_solid_full_with(0, 0, |_| Some(BlockSolidity::new(true, true))));
    }

    #[test]
    fn map_load_brackets_generator_and_resets_change_counters_like_world_load_event() {
        let mut world = World::new();
        world.note_tile_change();
        world.note_floor_change();
        assert_eq!(world.tile_changes, 2);
        assert_eq!(world.floor_changes, 2);

        world.load_generator(2, 2, |tiles| {
            tiles.get_mut(1, 1).unwrap().block = 5;
        });

        assert!(!world.is_generating());
        assert_eq!(world.width(), 2);
        assert_eq!(world.height(), 2);
        assert_eq!(world.tile(1, 1).unwrap().block, 5);
        assert_eq!(world.tile_changes, -1);
        assert_eq!(world.floor_changes, -1);
        assert_eq!(
            world.load_events(),
            &[
                WorldLoadEventKind::Begin,
                WorldLoadEventKind::End,
                WorldLoadEventKind::Loaded
            ]
        );
    }
}
