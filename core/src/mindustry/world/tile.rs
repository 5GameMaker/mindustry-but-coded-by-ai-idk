use super::{Block, BlockId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingRef {
    pub tile_pos: i32,
    pub block: BlockId,
    pub team: i32,
    pub rotation: i32,
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
}
