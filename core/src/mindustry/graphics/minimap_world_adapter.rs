use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
#[path = "minimap_renderer.rs"]
mod minimap_renderer;

#[cfg(test)]
use self::minimap_renderer::{
    color_for, MinimapPixelUpdate, MinimapTilePos, MinimapTileSnapshot, MinimapTileUpdatePlan,
    MinimapWorldSize,
};

#[cfg(not(test))]
use super::{
    color_for, MinimapPixelUpdate, MinimapTilePos, MinimapTileSnapshot, MinimapTileUpdatePlan,
    MinimapWorldSize,
};

/// 轻量化的 minimap tile 快照输入，适合从任意 tile-like 数据源归一化后再喂给渲染层。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapTileSnapshotInput {
    pub pos: MinimapTilePos,
    pub real_block_color: u32,
    pub fallback_map_color: u32,
    pub floor_color: u32,
    pub block_is_air: bool,
    pub overlay_is_air: bool,
    pub above_real_block_solid: bool,
    pub floor_is_liquid: bool,
    pub above_floor_is_liquid: bool,
    pub darkness: f32,
}

impl MinimapTileSnapshotInput {
    pub const fn new(
        pos: MinimapTilePos,
        real_block_color: u32,
        fallback_map_color: u32,
        floor_color: u32,
        block_is_air: bool,
        overlay_is_air: bool,
        above_real_block_solid: bool,
        floor_is_liquid: bool,
        above_floor_is_liquid: bool,
        darkness: f32,
    ) -> Self {
        Self {
            pos,
            real_block_color,
            fallback_map_color,
            floor_color,
            block_is_air,
            overlay_is_air,
            above_real_block_solid,
            floor_is_liquid,
            above_floor_is_liquid,
            darkness,
        }
    }

    pub const fn from_snapshot(snapshot: MinimapTileSnapshot) -> Self {
        Self {
            pos: snapshot.pos,
            real_block_color: snapshot.real_block_color,
            fallback_map_color: snapshot.fallback_map_color,
            floor_color: snapshot.floor_color,
            block_is_air: snapshot.block_is_air,
            overlay_is_air: snapshot.overlay_is_air,
            above_real_block_solid: snapshot.above_real_block_solid,
            floor_is_liquid: snapshot.floor_is_liquid,
            above_floor_is_liquid: snapshot.above_floor_is_liquid,
            darkness: snapshot.darkness,
        }
    }

    pub const fn into_snapshot(self) -> MinimapTileSnapshot {
        MinimapTileSnapshot {
            pos: self.pos,
            real_block_color: self.real_block_color,
            fallback_map_color: self.fallback_map_color,
            floor_color: self.floor_color,
            block_is_air: self.block_is_air,
            overlay_is_air: self.overlay_is_air,
            above_real_block_solid: self.above_real_block_solid,
            floor_is_liquid: self.floor_is_liquid,
            above_floor_is_liquid: self.above_floor_is_liquid,
            darkness: self.darkness,
        }
    }
}

impl From<MinimapTileSnapshotInput> for MinimapTileSnapshot {
    fn from(value: MinimapTileSnapshotInput) -> Self {
        value.into_snapshot()
    }
}

impl From<MinimapTileSnapshot> for MinimapTileSnapshotInput {
    fn from(value: MinimapTileSnapshot) -> Self {
        Self::from_snapshot(value)
    }
}

/// 任意 tile-like 输入都可以通过这个 trait 归一化为 minimap 快照输入。
pub trait MinimapTileSnapshotSource {
    fn minimap_snapshot_input(&self) -> MinimapTileSnapshotInput;
}

impl MinimapTileSnapshotSource for MinimapTileSnapshotInput {
    fn minimap_snapshot_input(&self) -> MinimapTileSnapshotInput {
        *self
    }
}

impl MinimapTileSnapshotSource for MinimapTileSnapshot {
    fn minimap_snapshot_input(&self) -> MinimapTileSnapshotInput {
        (*self).into()
    }
}

/// 轻量 world/tile adapter：保存世界尺寸和按位置索引后的 minimap 快照输入，
/// 并可直接导出 `MinimapTileSnapshot` 或 `MinimapTileUpdatePlan` 所需数据。
#[derive(Debug, Clone, PartialEq)]
pub struct MinimapWorldTileSnapshotAdapter {
    pub world: MinimapWorldSize,
    snapshots: BTreeMap<MinimapTilePos, MinimapTileSnapshotInput>,
}

impl MinimapWorldTileSnapshotAdapter {
    pub fn new(world: MinimapWorldSize) -> Self {
        Self {
            world,
            snapshots: BTreeMap::new(),
        }
    }

    pub fn from_tiles<T>(world: MinimapWorldSize, tiles: impl IntoIterator<Item = T>) -> Self
    where
        T: MinimapTileSnapshotSource,
    {
        let mut adapter = Self::new(world);
        for tile in tiles {
            adapter.insert(tile);
        }
        adapter
    }

    pub fn insert<T>(&mut self, tile: T)
    where
        T: MinimapTileSnapshotSource,
    {
        let snapshot = tile.minimap_snapshot_input();
        self.snapshots.insert(snapshot.pos, snapshot);
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    pub fn snapshot_input(&self, pos: MinimapTilePos) -> Option<MinimapTileSnapshotInput> {
        self.snapshots.get(&pos).copied()
    }

    pub fn snapshot(&self, pos: MinimapTilePos) -> Option<MinimapTileSnapshot> {
        self.snapshot_input(pos).map(Into::into)
    }

    pub fn snapshots(&self) -> Vec<MinimapTileSnapshot> {
        self.snapshots.values().copied().map(Into::into).collect()
    }

    pub fn update_plan_from_positions<I>(&self, positions: I) -> MinimapTileUpdatePlan
    where
        I: IntoIterator<Item = MinimapTilePos>,
    {
        self.update_plan_from_packed_positions(positions.into_iter().map(MinimapTilePos::pack))
    }

    pub fn update_plan_from_packed_positions<I>(&self, positions: I) -> MinimapTileUpdatePlan
    where
        I: IntoIterator<Item = i32>,
    {
        if self.world.height <= 0 {
            return MinimapTileUpdatePlan {
                updates: Vec::new(),
            };
        }

        let mut packed_positions = BTreeSet::new();
        packed_positions.extend(positions);

        let mut updates = Vec::new();
        for packed in packed_positions {
            let pos = MinimapTilePos::unpack(packed);
            if let Some(snapshot) = self.snapshots.get(&pos) {
                updates.push(MinimapPixelUpdate {
                    pos,
                    pixmap_x: pos.x,
                    pixmap_y: self.world.height - 1 - pos.y,
                    rgba: color_for((*snapshot).into()),
                });
            }
        }

        MinimapTileUpdatePlan { updates }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(pos: MinimapTilePos, color: u32) -> MinimapTileSnapshotInput {
        MinimapTileSnapshotInput::new(
            pos, color, 0x01020304, 0x05060708, true, true, false, false, false, 1.5,
        )
    }

    #[test]
    fn snapshot_input_roundtrips_into_minimap_snapshot() {
        let input = make_input(MinimapTilePos::new(-2, 7), 0x11223344);
        let snapshot = input.into_snapshot();

        assert_eq!(snapshot.pos, MinimapTilePos::new(-2, 7));
        assert_eq!(snapshot.real_block_color, 0x11223344);
        assert_eq!(snapshot.fallback_map_color, 0x01020304);
        assert_eq!(snapshot.floor_color, 0x05060708);
        assert!(snapshot.block_is_air);
        assert!(snapshot.overlay_is_air);
        assert!((snapshot.darkness - 1.5).abs() < f32::EPSILON);

        let normalized: MinimapTileSnapshotInput = snapshot.into();
        assert_eq!(normalized, input);
    }

    #[test]
    fn adapter_keeps_last_snapshot_for_duplicate_positions() {
        let world = MinimapWorldSize::new(16, 9);
        let pos = MinimapTilePos::new(3, 4);
        let adapter = MinimapWorldTileSnapshotAdapter::from_tiles(
            world,
            [
                make_input(pos, 0x111111ff),
                make_input(pos, 0x222222ff),
                make_input(MinimapTilePos::new(1, 1), 0x333333ff),
            ],
        );

        assert_eq!(adapter.world, world);
        assert_eq!(adapter.len(), 2);
        assert_eq!(adapter.snapshot(pos).unwrap().real_block_color, 0x222222ff);
        assert_eq!(
            adapter
                .snapshot(MinimapTilePos::new(1, 1))
                .unwrap()
                .real_block_color,
            0x333333ff
        );
    }

    #[test]
    fn update_plan_uses_known_tiles_and_flips_pixmap_y() {
        let world = MinimapWorldSize::new(12, 10);
        let adapter = MinimapWorldTileSnapshotAdapter::from_tiles(
            world,
            [
                make_input(MinimapTilePos::new(4, 1), 0x102030ff),
                make_input(MinimapTilePos::new(2, 3), 0x405060ff),
            ],
        );

        let plan = adapter.update_plan_from_positions([
            MinimapTilePos::new(4, 1),
            MinimapTilePos::new(9, 9),
            MinimapTilePos::new(2, 3),
            MinimapTilePos::new(2, 3),
        ]);

        assert_eq!(
            plan.updates,
            vec![
                MinimapPixelUpdate {
                    pos: MinimapTilePos::new(2, 3),
                    pixmap_x: 2,
                    pixmap_y: 6,
                    rgba: color_for(MinimapTileSnapshot::from(make_input(
                        MinimapTilePos::new(2, 3),
                        0x405060ff,
                    ))),
                },
                MinimapPixelUpdate {
                    pos: MinimapTilePos::new(4, 1),
                    pixmap_x: 4,
                    pixmap_y: 8,
                    rgba: color_for(MinimapTileSnapshot::from(make_input(
                        MinimapTilePos::new(4, 1),
                        0x102030ff,
                    ))),
                },
            ]
        );
    }

    #[test]
    fn update_plan_from_packed_positions_ignores_missing_tiles() {
        let world = MinimapWorldSize::new(8, 8);
        let adapter = MinimapWorldTileSnapshotAdapter::from_tiles(
            world,
            [make_input(MinimapTilePos::new(0, 0), 0xabcdef12)],
        );

        let plan = adapter.update_plan_from_packed_positions([
            MinimapTilePos::new(0, 0).pack(),
            MinimapTilePos::new(7, 7).pack(),
        ]);

        assert_eq!(plan.updates.len(), 1);
        assert_eq!(plan.updates[0].pos, MinimapTilePos::new(0, 0));
        assert_eq!(plan.updates[0].pixmap_y, 7);
    }
}
