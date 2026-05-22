use std::collections::HashMap;

use crate::mindustry::vars::TILE_SIZE;

pub const QUADRANT_SIZE: i32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexedTile {
    pub pos: i32,
    pub x: i32,
    pub y: i32,
    pub block_air: bool,
}

impl IndexedTile {
    pub const fn new(pos: i32, x: i32, y: i32, block_air: bool) -> Self {
        Self {
            pos,
            x,
            y,
            block_air,
        }
    }

    pub fn world_x(self) -> f32 {
        self.x as f32 * TILE_SIZE as f32
    }

    pub fn world_y(self) -> f32 {
        self.y as f32 * TILE_SIZE as f32
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockIndexer {
    pub quad_width: i32,
    pub quad_height: i32,
    ores: HashMap<i16, HashMap<(i32, i32), Vec<i32>>>,
    wall_ores: HashMap<i16, HashMap<(i32, i32), Vec<i32>>>,
    all_ores: HashMap<i16, i32>,
    all_wall_ores: HashMap<i16, i32>,
    active_teams: Vec<i32>,
    flagged: HashMap<(i32, i32), Vec<i32>>,
    damaged: HashMap<i32, Vec<i32>>,
    blocks_present: Vec<bool>,
}

impl BlockIndexer {
    pub fn new(world_width: i32, world_height: i32, block_count: usize) -> Self {
        let (quad_width, quad_height) = quadrant_dimensions(world_width, world_height);
        Self {
            quad_width,
            quad_height,
            ores: HashMap::new(),
            wall_ores: HashMap::new(),
            all_ores: HashMap::new(),
            all_wall_ores: HashMap::new(),
            active_teams: Vec::new(),
            flagged: HashMap::new(),
            damaged: HashMap::new(),
            blocks_present: vec![false; block_count],
        }
    }

    pub fn add_ore(&mut self, item_id: i16, tile_x: i32, tile_y: i32, pos: i32) -> bool {
        add_ore_to(
            &mut self.ores,
            &mut self.all_ores,
            item_id,
            tile_x,
            tile_y,
            pos,
        )
    }

    pub fn add_wall_ore(&mut self, item_id: i16, tile_x: i32, tile_y: i32, pos: i32) -> bool {
        add_ore_to(
            &mut self.wall_ores,
            &mut self.all_wall_ores,
            item_id,
            tile_x,
            tile_y,
            pos,
        )
    }

    pub fn has_ore(&self, item_id: i16) -> bool {
        self.all_ores.get(&item_id).copied().unwrap_or(0) > 0
    }

    pub fn has_wall_ore(&self, item_id: i16) -> bool {
        self.all_wall_ores.get(&item_id).copied().unwrap_or(0) > 0
    }

    pub fn all_present_ores(&self) -> Vec<i16> {
        let mut ores = self
            .all_ores
            .iter()
            .chain(self.all_wall_ores.iter())
            .filter_map(|(item, count)| (*count > 0).then_some(*item))
            .collect::<Vec<_>>();
        ores.sort_unstable();
        ores.dedup();
        ores
    }

    pub fn find_closest_ore<F>(
        &self,
        xp: f32,
        yp: f32,
        item_id: i16,
        wall: bool,
        mut tile_lookup: F,
    ) -> Option<i32>
    where
        F: FnMut(i32) -> Option<IndexedTile>,
    {
        let source = if wall { &self.wall_ores } else { &self.ores };
        let quadrants = source.get(&item_id)?;
        let mut closest = None;
        let mut min_dst = 0.0;

        for qx in 0..self.quad_width {
            for qy in 0..self.quad_height {
                let Some(positions) = quadrants.get(&(qx, qy)) else {
                    continue;
                };
                let Some(pos) = positions.first().copied() else {
                    continue;
                };
                let Some(tile) = tile_lookup(pos) else {
                    continue;
                };
                if tile.block_air == !wall {
                    let dst = dst2(xp, yp, tile.world_x(), tile.world_y());
                    if closest.is_none() || dst < min_dst {
                        closest = Some(tile.pos);
                        min_dst = dst;
                    }
                }
            }
        }

        closest
    }

    pub fn mark_block_present(&mut self, block_id: i16) {
        if block_id >= 0 {
            if let Some(slot) = self.blocks_present.get_mut(block_id as usize) {
                *slot = true;
            }
        }
    }

    pub fn is_block_present(&self, block_id: i16) -> bool {
        block_id >= 0
            && self
                .blocks_present
                .get(block_id as usize)
                .copied()
                .unwrap_or(false)
    }

    pub fn add_active_team(&mut self, team: i32) -> bool {
        if self.active_teams.contains(&team) {
            false
        } else {
            self.active_teams.push(team);
            true
        }
    }

    pub fn active_teams(&self) -> &[i32] {
        &self.active_teams
    }

    pub fn add_flagged(&mut self, team: i32, flag: i32, building_id: i32) -> bool {
        add_unique(self.flagged.entry((team, flag)).or_default(), building_id)
    }

    pub fn remove_flagged(&mut self, team: i32, flag: i32, building_id: i32) -> bool {
        remove_value(self.flagged.entry((team, flag)).or_default(), building_id)
    }

    pub fn get_flagged(&self, team: i32, flag: i32) -> &[i32] {
        self.flagged
            .get(&(team, flag))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn set_damaged(&mut self, team: i32, building_id: i32, damaged: bool) -> bool {
        let list = self.damaged.entry(team).or_default();
        if damaged {
            add_unique(list, building_id)
        } else {
            remove_value(list, building_id)
        }
    }

    pub fn get_damaged(&self, team: i32) -> &[i32] {
        self.damaged.get(&team).map(Vec::as_slice).unwrap_or(&[])
    }
}

pub fn quadrant_dimensions(world_width: i32, world_height: i32) -> (i32, i32) {
    (
        div_ceil(world_width.max(0), QUADRANT_SIZE),
        div_ceil(world_height.max(0), QUADRANT_SIZE),
    )
}

pub fn quadrant_for_tile(tile_x: i32, tile_y: i32) -> (i32, i32) {
    (tile_x / QUADRANT_SIZE, tile_y / QUADRANT_SIZE)
}

fn add_ore_to(
    ores: &mut HashMap<i16, HashMap<(i32, i32), Vec<i32>>>,
    counts: &mut HashMap<i16, i32>,
    item_id: i16,
    tile_x: i32,
    tile_y: i32,
    pos: i32,
) -> bool {
    let quadrant = quadrant_for_tile(tile_x, tile_y);
    let list = ores
        .entry(item_id)
        .or_default()
        .entry(quadrant)
        .or_default();
    if add_unique(list, pos) {
        *counts.entry(item_id).or_insert(0) += 1;
        true
    } else {
        false
    }
}

fn add_unique(list: &mut Vec<i32>, value: i32) -> bool {
    if list.contains(&value) {
        false
    } else {
        list.push(value);
        true
    }
}

fn remove_value(list: &mut Vec<i32>, value: i32) -> bool {
    if let Some(index) = list.iter().position(|candidate| *candidate == value) {
        list.swap_remove(index);
        true
    } else {
        false
    }
}

fn div_ceil(value: i32, divisor: i32) -> i32 {
    if value <= 0 {
        0
    } else {
        (value + divisor - 1) / divisor
    }
}

fn dst2(x: f32, y: f32, tx: f32, ty: f32) -> f32 {
    let dx = tx - x;
    let dy = ty - y;
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadrant_helpers_match_java_quadrant_size_ceiling() {
        assert_eq!(quadrant_dimensions(0, 0), (0, 0));
        assert_eq!(quadrant_dimensions(1, 20), (1, 1));
        assert_eq!(quadrant_dimensions(21, 41), (2, 3));
        assert_eq!(quadrant_for_tile(0, 0), (0, 0));
        assert_eq!(quadrant_for_tile(39, 40), (1, 2));
    }

    #[test]
    fn ore_indexes_track_floor_and_wall_ores_uniquely() {
        let mut indexer = BlockIndexer::new(100, 100, 16);

        assert!(indexer.add_ore(2, 4, 5, 100));
        assert!(!indexer.add_ore(2, 4, 5, 100));
        assert!(indexer.add_wall_ore(3, 25, 5, 200));

        assert!(indexer.has_ore(2));
        assert!(indexer.has_wall_ore(3));
        assert_eq!(indexer.all_present_ores(), vec![2, 3]);
    }

    #[test]
    fn find_closest_ore_uses_first_tile_per_quadrant_like_java() {
        let mut indexer = BlockIndexer::new(100, 100, 16);
        indexer.add_ore(1, 0, 0, 1);
        indexer.add_ore(1, 25, 0, 2);
        indexer.add_ore(1, 26, 0, 3);

        let closest = indexer.find_closest_ore(200.0, 0.0, 1, false, |pos| match pos {
            1 => Some(IndexedTile::new(1, 0, 0, true)),
            2 => Some(IndexedTile::new(2, 25, 0, true)),
            3 => Some(IndexedTile::new(3, 26, 0, true)),
            _ => None,
        });
        assert_eq!(closest, Some(2));

        assert_eq!(indexer.find_closest_ore(0.0, 0.0, 1, true, |_| None), None);
    }

    #[test]
    fn flags_damage_active_teams_and_block_presence_are_indexed() {
        let mut indexer = BlockIndexer::new(10, 10, 4);

        assert!(indexer.add_active_team(2));
        assert!(!indexer.add_active_team(2));
        assert_eq!(indexer.active_teams(), &[2]);

        assert!(indexer.add_flagged(2, 7, 99));
        assert!(!indexer.add_flagged(2, 7, 99));
        assert_eq!(indexer.get_flagged(2, 7), &[99]);
        assert!(indexer.remove_flagged(2, 7, 99));
        assert!(indexer.get_flagged(2, 7).is_empty());

        assert!(indexer.set_damaged(2, 42, true));
        assert_eq!(indexer.get_damaged(2), &[42]);
        assert!(indexer.set_damaged(2, 42, false));
        assert!(indexer.get_damaged(2).is_empty());

        indexer.mark_block_present(3);
        assert!(indexer.is_block_present(3));
        assert!(!indexer.is_block_present(9));
    }
}
