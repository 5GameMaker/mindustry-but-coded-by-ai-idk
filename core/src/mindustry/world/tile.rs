use crate::mindustry::vars::TILE_SIZE;

use super::{footprint_tiles, Block, BlockId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingRef {
    pub tile_pos: i32,
    pub block: BlockId,
    pub team: i32,
    pub rotation: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildingBaseWire {
    pub block_revision: u8,
    pub health: f32,
    pub rotation: i32,
    pub team: i32,
}

impl BuildingBaseWire {
    pub const MIN_SAVE_PAYLOAD_LEN: usize = 7;

    pub fn from_save_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() < Self::MIN_SAVE_PAYLOAD_LEN {
            return None;
        }

        let health = f32::from_bits(u32::from_be_bytes([
            payload[1], payload[2], payload[3], payload[4],
        ]));
        Some(Self {
            block_revision: payload[0],
            health,
            rotation: (payload[5] & 0b0111_1111) as i32,
            team: payload[6] as i32,
        })
    }
}

impl BuildingRef {
    pub fn from_save_payload(tile_pos: i32, block: BlockId, payload: &[u8]) -> Option<Self> {
        let base = BuildingBaseWire::from_save_payload(payload)?;
        Some(Self {
            tile_pos,
            block,
            team: base.team,
            rotation: base.rotation,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pub data: u8,
    pub floor_data: u8,
    pub overlay_data: u8,
    pub extra_data: i32,
    pub build: Option<BuildingRef>,
    pub x: i16,
    pub y: i16,
    pub block: BlockId,
    pub floor: BlockId,
    pub overlay: BlockId,
    pub changing: bool,
}

impl Tile {
    pub const AIR: BlockId = 0;

    pub fn new(x: i32, y: i32) -> Self {
        Self {
            data: 0,
            floor_data: 0,
            overlay_data: 0,
            extra_data: 0,
            build: None,
            x: x as i16,
            y: y as i16,
            block: Self::AIR,
            floor: Self::AIR,
            overlay: Self::AIR,
            changing: false,
        }
    }

    pub fn with_blocks(x: i32, y: i32, floor: BlockId, overlay: BlockId, block: BlockId) -> Self {
        let mut tile = Self::new(x, y);
        tile.floor = floor;
        tile.overlay = overlay;
        tile.block = block;
        tile
    }

    pub fn pos(&self) -> i32 {
        point2_pack(self.x as i32, self.y as i32)
    }

    pub fn array(&self, width: usize) -> usize {
        self.x as usize + self.y as usize * width
    }

    pub fn relative_to_tile(&self, tile: &Tile) -> i8 {
        self.relative_to(tile.x as i32, tile.y as i32)
    }

    pub fn relative_to(&self, cx: i32, cy: i32) -> i8 {
        relative_to(self.x as i32, self.y as i32, cx, cy)
    }

    pub fn block_id(&self) -> BlockId {
        self.block
    }

    pub fn floor_id(&self) -> BlockId {
        self.floor
    }

    pub fn overlay_id(&self) -> BlockId {
        self.overlay
    }

    pub fn is_center(&self) -> bool {
        self.build
            .map_or(true, |build| build.tile_pos == self.pos())
    }

    pub fn center_pos(&self) -> i32 {
        self.build
            .map_or_else(|| self.pos(), |build| build.tile_pos)
    }

    pub fn center_x(&self) -> i32 {
        point2_x(self.center_pos()) as i32
    }

    pub fn center_y(&self) -> i32 {
        point2_y(self.center_pos()) as i32
    }

    pub fn team(&self) -> i32 {
        self.build.map_or(0, |build| build.team)
    }

    pub fn set_team(&mut self, team: i32) {
        if let Some(build) = &mut self.build {
            build.team = team;
        }
    }

    pub fn world_x(&self) -> f32 {
        self.x as f32 * TILE_SIZE as f32
    }

    pub fn world_y(&self) -> f32 {
        self.y as f32 * TILE_SIZE as f32
    }

    pub fn draw_x(&self, block: &Block) -> f32 {
        self.world_x() + block.offset
    }

    pub fn draw_y(&self, block: &Block) -> f32 {
        self.world_y() + block.offset
    }

    pub fn absolute_relative_to(&self, block: &Block, cx: i32, cy: i32) -> i8 {
        absolute_relative_to(self.x as i32, self.y as i32, block.size, cx, cy)
    }

    pub fn adjacent_to(&self, tile: &Tile) -> bool {
        self.relative_to_tile(tile) != -1
    }

    pub fn passable(&self, floor: &Block, block: &Block) -> bool {
        !((floor.solid && (block.id == Self::AIR || block.solidifies))
            || (block.solid && !block.destructible && !block.update))
    }

    pub fn synthetic(&self, block: &Block) -> bool {
        block.synthetic()
    }

    pub fn is_darkened(&self, block: &Block) -> bool {
        block.is_darkened()
    }

    pub fn breakable(&self, block: &Block) -> bool {
        block.destructible || block.breakable || block.update
    }

    pub fn linked_positions(&self, block: &Block) -> Vec<(i32, i32)> {
        footprint_tiles(self.center_x(), self.center_y(), block.size)
    }

    pub fn linked_positions_as(&self, block: &Block) -> Vec<(i32, i32)> {
        footprint_tiles(self.x as i32, self.y as i32, block.size)
    }

    pub fn static_darkness(&self, block: &Block) -> u8 {
        if block.solid && block.fills_tile && !block.synthetic() {
            self.data
        } else {
            0
        }
    }

    pub fn leg_solid(&self, floor: &Block, block: &Block) -> bool {
        self.static_darkness(block) >= 2 || (floor.solid && block.id == Self::AIR)
    }

    pub fn floor_color_rgba_with<F>(&self, mut floor_color: F) -> u32
    where
        F: FnMut(BlockId, &Tile) -> u32,
    {
        floor_color(self.floor_id(), self)
    }

    pub fn set_block_ref(&mut self, block: &Block, team: i32, rotation: i32) {
        self.block = block.id;
        self.build = block.has_building().then(|| BuildingRef {
            tile_pos: self.pos(),
            block: block.id,
            team: block.force_team.unwrap_or(team),
            rotation: block.plan_rotation(rotation),
        });
    }

    pub fn set_proxy_block_ref(&mut self, block: &Block, build: Option<BuildingRef>) {
        self.block = block.id;
        self.build = build;
    }

    pub fn clear_block_ref(&mut self) {
        self.block = Self::AIR;
        self.build = None;
    }

    pub fn should_save_data(&self, floor: &Block, overlay: &Block, block: &Block) -> bool {
        floor.save_data || overlay.save_data || block.save_data
    }

    pub fn get_packed_data(&self) -> u64 {
        packed_tile_data_get(
            self.extra_data,
            self.data,
            self.floor_data,
            self.overlay_data,
        )
    }

    pub fn set_packed_data(&mut self, packed: u64) {
        self.extra_data = packed_tile_extra_data(packed);
        self.data = packed_tile_data(packed);
        self.floor_data = packed_tile_floor_data(packed);
        self.overlay_data = packed_tile_overlay_data(packed);
    }
}

pub fn relative_to(x: i32, y: i32, cx: i32, cy: i32) -> i8 {
    if x == cx && y == cy - 1 {
        1
    } else if x == cx && y == cy + 1 {
        3
    } else if x == cx - 1 && y == cy {
        0
    } else if x == cx + 1 && y == cy {
        2
    } else {
        -1
    }
}

pub fn absolute_relative_to(x: i32, y: i32, block_size: i32, cx: i32, cy: i32) -> i8 {
    if block_size % 2 == 1 {
        if (x - cx).abs() > (y - cy).abs() {
            if x <= cx - 1 {
                return 0;
            }
            if x >= cx + 1 {
                return 2;
            }
        } else {
            if y <= cy - 1 {
                return 1;
            }
            if y >= cy + 1 {
                return 3;
            }
        }
    } else {
        let x = x as f32 + 0.5;
        let y = y as f32 + 0.5;
        let cx = cx as f32;
        let cy = cy as f32;
        if (x - cx).abs() > (y - cy).abs() {
            if x <= cx - 1.0 {
                return 0;
            }
            if x >= cx + 1.0 {
                return 2;
            }
        } else {
            if y <= cy - 1.0 {
                return 1;
            }
            if y >= cy + 1.0 {
                return 3;
            }
        }
    }
    -1
}

pub fn point2_pack(x: i32, y: i32) -> i32 {
    ((x as i16 as i32) << 16) | (y as i16 as u16 as i32)
}

pub fn point2_x(pos: i32) -> i16 {
    ((pos as u32) >> 16) as u16 as i16
}

pub fn point2_y(pos: i32) -> i16 {
    (pos as u32 & 0xffff) as u16 as i16
}

pub fn packed_tile_data_get(extra_data: i32, data: u8, floor_data: u8, overlay_data: u8) -> u64 {
    (extra_data as u32 as u64)
        | ((data as u64) << 32)
        | ((floor_data as u64) << 40)
        | ((overlay_data as u64) << 48)
}

pub fn packed_tile_extra_data(packed: u64) -> i32 {
    (packed as u32) as i32
}

pub fn packed_tile_data(packed: u64) -> u8 {
    ((packed >> 32) & 0xff) as u8
}

pub fn packed_tile_floor_data(packed: u64) -> u8 {
    ((packed >> 40) & 0xff) as u8
}

pub fn packed_tile_overlay_data(packed: u64) -> u8 {
    ((packed >> 48) & 0xff) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point2_pack_matches_arc_layout() {
        let packed = point2_pack(-1, 2);
        assert_eq!(point2_x(packed), -1);
        assert_eq!(point2_y(packed), 2);
        assert_eq!(point2_pack(3, 4), 0x0003_0004);
    }

    #[test]
    fn relative_to_matches_java_rotation_values() {
        let tile = Tile::new(5, 5);
        assert_eq!(tile.relative_to(6, 5), 0);
        assert_eq!(tile.relative_to(5, 6), 1);
        assert_eq!(tile.relative_to(4, 5), 2);
        assert_eq!(tile.relative_to(5, 4), 3);
        assert_eq!(tile.relative_to(7, 7), -1);
    }

    #[test]
    fn packed_tile_data_roundtrips_generated_struct_layout() {
        let mut tile = Tile::new(1, 2);
        tile.extra_data = -1234567;
        tile.data = 0xaa;
        tile.floor_data = 0xbb;
        tile.overlay_data = 0xcc;

        let packed = tile.get_packed_data();
        assert_eq!(packed_tile_data(packed), 0xaa);
        assert_eq!(packed_tile_floor_data(packed), 0xbb);
        assert_eq!(packed_tile_overlay_data(packed), 0xcc);
        assert_eq!(packed_tile_extra_data(packed), -1234567);

        let mut decoded = Tile::new(0, 0);
        decoded.set_packed_data(packed);
        assert_eq!(decoded.extra_data, tile.extra_data);
        assert_eq!(decoded.data, tile.data);
        assert_eq!(decoded.floor_data, tile.floor_data);
        assert_eq!(decoded.overlay_data, tile.overlay_data);
    }

    #[test]
    fn building_ref_decodes_java_save_map_base_prefix() {
        let mut payload = vec![2];
        payload.extend_from_slice(&42.5f32.to_bits().to_be_bytes());
        payload.push(0b1000_0000 | 3);
        payload.push(6);

        let base = BuildingBaseWire::from_save_payload(&payload).unwrap();
        assert_eq!(base.block_revision, 2);
        assert_eq!(base.health, 42.5);
        assert_eq!(base.rotation, 3);
        assert_eq!(base.team, 6);

        assert_eq!(
            BuildingRef::from_save_payload(point2_pack(4, 5), 12, &payload),
            Some(BuildingRef {
                tile_pos: point2_pack(4, 5),
                block: 12,
                team: 6,
                rotation: 3,
            })
        );
        assert!(BuildingBaseWire::from_save_payload(&payload[..6]).is_none());
    }

    #[test]
    fn tile_floor_color_resolver_matches_java_get_floor_color_call_shape() {
        let tile = Tile::with_blocks(3, 4, 42, Tile::AIR, Tile::AIR);
        let color = tile.floor_color_rgba_with(|floor_id, tile_arg| {
            assert_eq!(floor_id, 42);
            assert_eq!(tile_arg.pos(), tile.pos());
            0xaabbccdd
        });

        assert_eq!(color, 0xaabbccdd);
    }

    #[test]
    fn tile_is_darkened_uses_block_darkening_rules() {
        let tile = Tile::new(0, 0);
        let mut wall = Block::new(7, "wall");
        wall.solid = true;
        wall.fills_tile = true;
        assert!(tile.is_darkened(&wall));

        wall.destructible = true;
        assert!(!tile.is_darkened(&wall));

        wall.force_dark = true;
        assert!(tile.is_darkened(&wall));
    }
}
