use std::collections::HashMap;

use crate::mindustry::vars::TILE_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasePartTileKind {
    Core,
    ItemSource,
    LiquidSource,
    Drill,
    Pump,
    SandboxOnly,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasePartTile {
    pub x: i32,
    pub y: i32,
    pub block_name: String,
    pub build_time: f32,
    pub build_cost_multiplier: f32,
    pub offset: f32,
    pub kind: BasePartTileKind,
    pub config: Option<String>,
}

impl BasePartTile {
    pub fn new(x: i32, y: i32, block_name: impl Into<String>, kind: BasePartTileKind) -> Self {
        Self {
            x,
            y,
            block_name: block_name.into(),
            build_time: 1.0,
            build_cost_multiplier: 1.0,
            offset: 0.0,
            kind,
            config: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasePart {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<BasePartTile>,
    pub center_x: i32,
    pub center_y: i32,
    pub required: Option<String>,
    pub core: Option<String>,
    pub tier: f32,
}

impl BasePart {
    pub fn from_tiles(
        name: impl Into<String>,
        width: i32,
        height: i32,
        tiles: Vec<BasePartTile>,
    ) -> Self {
        let name = name.into();
        let mut core = None;
        let mut required = None;
        let mut drill_x = 0.0;
        let mut drill_y = 0.0;
        let mut drills = 0;
        let mut filtered = Vec::new();
        let mut tier = 0.0;

        for tile in tiles {
            match tile.kind {
                BasePartTileKind::Core => core = Some(tile.block_name.clone()),
                BasePartTileKind::ItemSource | BasePartTileKind::LiquidSource => {
                    if let Some(config) = &tile.config {
                        required = Some(config.clone());
                    }
                }
                BasePartTileKind::Drill | BasePartTileKind::Pump => {
                    drill_x += tile.x as f32 * TILE_SIZE as f32 + tile.offset;
                    drill_y += tile.y as f32 * TILE_SIZE as f32 + tile.offset;
                    drills += 1;
                }
                _ => {}
            }

            if tile.kind != BasePartTileKind::SandboxOnly {
                tier += (tile.build_time / tile.build_cost_multiplier).powf(1.4);
                filtered.push(tile);
            }
        }

        let (center_x, center_y) = if drills > 0 {
            (
                (drill_x / drills as f32 / TILE_SIZE as f32) as i32,
                (drill_y / drills as f32 / TILE_SIZE as f32) as i32,
            )
        } else {
            (width / 2, height / 2)
        };

        Self {
            name,
            width,
            height,
            tiles: filtered,
            center_x,
            center_y,
            required,
            core,
            tier,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BaseRegistry {
    pub cores: Vec<BasePart>,
    pub parts: Vec<BasePart>,
    pub req_parts: HashMap<String, Vec<BasePart>>,
}

impl BaseRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_parts(&mut self, parts: Vec<BasePart>) {
        self.cores.clear();
        self.parts.clear();
        self.req_parts.clear();

        for part in parts {
            if part.core.is_some() {
                self.cores.push(part.clone());
            } else if part.required.is_none() {
                self.parts.push(part.clone());
            }

            if let (None, Some(required)) = (&part.core, &part.required) {
                self.req_parts
                    .entry(required.clone())
                    .or_default()
                    .push(part.clone());
            }
        }

        self.cores.sort_by(|a, b| a.tier.total_cmp(&b.tier));
        self.parts.sort_by(|a, b| a.tier.total_cmp(&b.tier));
        for parts in self.req_parts.values_mut() {
            parts.sort_by(|a, b| a.tier.total_cmp(&b.tier));
        }
    }

    pub fn for_resource(&self, resource: &str) -> &[BasePart] {
        self.req_parts
            .get(resource)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile(x: i32, y: i32, name: &str, kind: BasePartTileKind, build_time: f32) -> BasePartTile {
        let mut tile = BasePartTile::new(x, y, name, kind);
        tile.build_time = build_time;
        tile
    }

    #[test]
    fn base_part_extracts_core_requirement_tier_and_drill_center() {
        let mut source = tile(0, 0, "item-source", BasePartTileKind::ItemSource, 1.0);
        source.config = Some("copper".into());
        let mut drill = tile(4, 6, "mechanical-drill", BasePartTileKind::Drill, 4.0);
        drill.offset = 4.0;

        let part = BasePart::from_tiles(
            "copper-drill",
            10,
            12,
            vec![
                source,
                drill,
                tile(1, 1, "sandbox", BasePartTileKind::SandboxOnly, 1000.0),
            ],
        );

        assert_eq!(part.required.as_deref(), Some("copper"));
        assert_eq!(part.core, None);
        assert_eq!((part.center_x, part.center_y), (4, 6));
        assert_eq!(part.tiles.len(), 2);
        assert!((part.tier - (1.0f32.powf(1.4) + 4.0f32.powf(1.4))).abs() < 0.0001);
    }

    #[test]
    fn base_part_without_drills_uses_schematic_center() {
        let part = BasePart::from_tiles(
            "core",
            11,
            9,
            vec![tile(5, 4, "core-shard", BasePartTileKind::Core, 10.0)],
        );

        assert_eq!(part.core.as_deref(), Some("core-shard"));
        assert_eq!((part.center_x, part.center_y), (5, 4));
    }

    #[test]
    fn base_registry_classifies_and_sorts_cores_parts_and_required_parts() {
        let core_low = BasePart::from_tiles(
            "core-low",
            5,
            5,
            vec![tile(0, 0, "core-shard", BasePartTileKind::Core, 1.0)],
        );
        let core_high = BasePart::from_tiles(
            "core-high",
            5,
            5,
            vec![tile(0, 0, "core-nucleus", BasePartTileKind::Core, 9.0)],
        );
        let free = BasePart::from_tiles(
            "free",
            5,
            5,
            vec![tile(0, 0, "router", BasePartTileKind::Other, 2.0)],
        );
        let mut source = tile(0, 0, "liquid-source", BasePartTileKind::LiquidSource, 1.0);
        source.config = Some("water".into());
        let required = BasePart::from_tiles("water-pump", 5, 5, vec![source]);

        let mut registry = BaseRegistry::new();
        registry.load_parts(vec![
            core_high.clone(),
            required.clone(),
            free.clone(),
            core_low.clone(),
        ]);

        assert_eq!(registry.cores[0].name, "core-low");
        assert_eq!(registry.cores[1].name, "core-high");
        assert_eq!(registry.parts, vec![free]);
        assert_eq!(registry.for_resource("water"), &[required]);
        assert!(registry.for_resource("copper").is_empty());
    }
}
