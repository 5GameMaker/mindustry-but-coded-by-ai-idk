use super::{point2_x, point2_y, Tile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tiles {
    pub width: usize,
    pub height: usize,
    array: Vec<Tile>,
    puddles: Vec<Option<i32>>,
    fires: Vec<Option<i32>>,
    tmp_floor_state: Option<Vec<u64>>,
    tmp_block_state: Option<Vec<u64>>,
}

impl Tiles {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = Self {
            width,
            height,
            array: Vec::with_capacity(width * height),
            puddles: vec![None; width * height],
            fires: vec![None; width * height],
            tmp_floor_state: None,
            tmp_block_state: None,
        };
        tiles.fill();
        tiles
    }

    pub fn len(&self) -> usize {
        self.array.len()
    }

    pub fn is_empty(&self) -> bool {
        self.array.is_empty()
    }

    pub fn fill(&mut self) {
        self.array.clear();
        for i in 0..self.width * self.height {
            self.array
                .push(Tile::new((i % self.width) as i32, (i / self.width) as i32));
        }
    }

    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32
    }

    pub fn set(&mut self, x: usize, y: usize, tile: Tile) {
        let index = self.index(x, y);
        self.array[index] = tile;
    }

    pub fn seti(&mut self, index: usize, tile: Tile) {
        self.array[index] = tile;
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&Tile> {
        if self.in_bounds(x, y) {
            Some(&self.array[self.index(x as usize, y as usize)])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut Tile> {
        if self.in_bounds(x, y) {
            let index = self.index(x as usize, y as usize);
            Some(&mut self.array[index])
        } else {
            None
        }
    }

    pub fn getn(&self, x: i32, y: i32) -> Result<&Tile, String> {
        self.get(x, y).ok_or_else(|| {
            format!(
                "{x}, {y} out of bounds: width={}, height={}",
                self.width, self.height
            )
        })
    }

    pub fn getc(&self, x: i32, y: i32) -> &Tile {
        let x = x.clamp(0, self.width as i32 - 1) as usize;
        let y = y.clamp(0, self.height as i32 - 1) as usize;
        &self.array[self.index(x, y)]
    }

    pub fn geti(&self, index: usize) -> Option<&Tile> {
        self.array.get(index)
    }

    pub fn getp(&self, pos: i32) -> Option<&Tile> {
        self.get(point2_x(pos) as i32, point2_y(pos) as i32)
    }

    pub fn each<F: FnMut(usize, usize)>(&self, mut f: F) {
        for x in 0..self.width {
            for y in 0..self.height {
                f(x, y);
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tile> {
        self.array.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Tile> {
        self.array.iter_mut()
    }

    pub fn get_tmp_floor_state(&self, pos: usize) -> u64 {
        self.tmp_floor_state.as_ref().map_or(0, |state| state[pos])
    }

    pub fn set_tmp_floor_state(&mut self, pos: usize, value: u64) {
        ensure_state_len(&mut self.tmp_floor_state, self.array.len());
        self.tmp_floor_state.as_mut().unwrap()[pos] = value;
    }

    pub fn get_tmp_block_state(&self, pos: usize) -> u64 {
        self.tmp_block_state.as_ref().map_or(0, |state| state[pos])
    }

    pub fn set_tmp_block_state(&mut self, pos: usize, value: u64) {
        ensure_state_len(&mut self.tmp_block_state, self.array.len());
        self.tmp_block_state.as_mut().unwrap()[pos] = value;
    }

    pub fn get_puddle(&self, pos: usize) -> Option<i32> {
        self.puddles[pos]
    }

    pub fn set_puddle(&mut self, pos: usize, puddle: Option<i32>) {
        self.puddles[pos] = puddle;
    }

    pub fn get_fire(&self, pos: usize) -> Option<i32> {
        self.fires[pos]
    }

    pub fn set_fire(&mut self, pos: usize, fire: Option<i32>) {
        self.fires[pos] = fire;
    }
}

fn ensure_state_len(state: &mut Option<Vec<u64>>, len: usize) {
    if state.as_ref().map_or(true, |state| state.len() != len) {
        *state = Some(vec![0; len]);
    }
}

impl IntoIterator for Tiles {
    type Item = Tile;
    type IntoIter = std::vec::IntoIter<Tile>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::world::point2_pack;

    #[test]
    fn tiles_index_bounds_and_clamped_access_match_java_container() {
        let mut tiles = Tiles::new(3, 2);
        assert_eq!(tiles.len(), 6);
        assert!(tiles.in_bounds(2, 1));
        assert!(!tiles.in_bounds(3, 1));
        assert_eq!(tiles.get(1, 1).unwrap().array(3), 4);
        assert!(tiles.get(-1, 0).is_none());
        assert_eq!(tiles.getc(99, 99).pos(), point2_pack(2, 1));

        let replacement = Tile::with_blocks(1, 1, 2, 3, 4);
        tiles.set(1, 1, replacement.clone());
        assert_eq!(tiles.get(1, 1), Some(&replacement));
        assert_eq!(tiles.getp(point2_pack(1, 1)), Some(&replacement));
    }

    #[test]
    fn temp_states_allocate_lazily() {
        let mut tiles = Tiles::new(2, 2);
        assert_eq!(tiles.get_tmp_floor_state(1), 0);
        tiles.set_tmp_floor_state(1, 77);
        tiles.set_tmp_block_state(2, 99);
        assert_eq!(tiles.get_tmp_floor_state(1), 77);
        assert_eq!(tiles.get_tmp_block_state(2), 99);
    }
}
