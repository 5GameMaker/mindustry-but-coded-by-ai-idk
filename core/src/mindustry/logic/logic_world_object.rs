//! Mirrors the lightweight world and tile snapshots used by logic instructions.

use std::collections::BTreeMap;

use super::{RadarTarget, TileLayer};

#[derive(Debug, Clone, PartialEq)]
pub struct LogicTileObject {
    pub floor: Option<String>,
    pub ore: Option<String>,
    pub block: Option<String>,
    pub building: Option<String>,
    pub team: u8,
    pub rotation: i32,
}

impl Default for LogicTileObject {
    fn default() -> Self {
        Self {
            floor: Some("@air".into()),
            ore: Some("@air".into()),
            block: Some("@air".into()),
            building: None,
            team: RadarTarget::DERELICT_TEAM,
            rotation: 0,
        }
    }
}

impl LogicTileObject {
    pub fn get_layer(&self, layer: TileLayer) -> Option<String> {
        match layer {
            TileLayer::Floor => self.floor.clone(),
            TileLayer::Ore => self.ore.clone(),
            TileLayer::Block => self.block.clone(),
            TileLayer::Building => self.building.clone(),
        }
    }

    pub fn set_layer(&mut self, layer: TileLayer, value: Option<String>, team: u8, rotation: i32) {
        match layer {
            TileLayer::Floor => self.floor = value,
            TileLayer::Ore => self.ore = value,
            TileLayer::Block => {
                self.block = value;
                self.team = team;
                self.rotation = rotation.clamp(0, 3);
            }
            TileLayer::Building => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicWorldObject {
    pub tiles: BTreeMap<(i32, i32), LogicTileObject>,
    pub spawns: Vec<(f32, f32)>,
}

impl Default for LogicWorldObject {
    fn default() -> Self {
        Self::new()
    }
}

impl LogicWorldObject {
    pub fn new() -> Self {
        Self {
            tiles: BTreeMap::new(),
            spawns: Vec::new(),
        }
    }

    pub fn tile(&self, x: i32, y: i32) -> Option<&LogicTileObject> {
        self.tiles.get(&(x, y))
    }

    pub fn tile_mut(&mut self, x: i32, y: i32) -> Option<&mut LogicTileObject> {
        self.tiles.get_mut(&(x, y))
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: LogicTileObject) {
        self.tiles.insert((x, y), tile);
    }
}

#[cfg(test)]
mod tests {
    use super::{LogicTileObject, LogicWorldObject};
    use crate::mindustry::logic::{RadarTarget, TileLayer};

    #[test]
    fn logic_tile_object_layers_follow_java_tile_layer_contract() {
        let mut tile = LogicTileObject::default();
        assert_eq!(tile.floor.as_deref(), Some("@air"));
        assert_eq!(tile.ore.as_deref(), Some("@air"));
        assert_eq!(tile.block.as_deref(), Some("@air"));
        assert!(tile.building.is_none());
        assert_eq!(tile.team, RadarTarget::DERELICT_TEAM);
        assert_eq!(tile.rotation, 0);

        tile.set_layer(TileLayer::Floor, Some("@stone".into()), 1, 2);
        tile.set_layer(TileLayer::Ore, Some("@ore-copper".into()), 1, 2);
        tile.set_layer(TileLayer::Block, Some("@duo".into()), 3, 9);
        tile.set_layer(TileLayer::Building, Some("@ignored".into()), 4, 1);

        assert_eq!(tile.get_layer(TileLayer::Floor).as_deref(), Some("@stone"));
        assert_eq!(
            tile.get_layer(TileLayer::Ore).as_deref(),
            Some("@ore-copper")
        );
        assert_eq!(tile.get_layer(TileLayer::Block).as_deref(), Some("@duo"));
        assert!(tile.get_layer(TileLayer::Building).is_none());
        assert_eq!(tile.team, 3);
        assert_eq!(tile.rotation, 3);
    }

    #[test]
    fn logic_world_object_stores_tiles_and_spawns_by_java_query_shape() {
        let mut world = LogicWorldObject::new();
        assert!(world.tiles.is_empty());
        assert!(world.spawns.is_empty());
        assert!(world.tile(1, 2).is_none());

        world.spawns.push((8.0, 16.0));
        world.set_tile(1, 2, LogicTileObject::default());
        assert!(world.tile(1, 2).is_some());
        assert_eq!(world.tile(1, 2).unwrap().block.as_deref(), Some("@air"));

        world
            .tile_mut(1, 2)
            .unwrap()
            .set_layer(TileLayer::Block, Some("@router".into()), 1, -2);
        let tile = world.tile(1, 2).unwrap();
        assert_eq!(tile.block.as_deref(), Some("@router"));
        assert_eq!(tile.rotation, 0);
        assert_eq!(world.spawns, vec![(8.0, 16.0)]);
    }
}
