//! Miner component mirroring upstream `mindustry.entities.comp.MinerComp`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MineItem {
    pub name: String,
    pub hardness: i32,
}

impl MineItem {
    pub fn new(name: impl Into<String>, hardness: i32) -> Self {
        Self {
            name: name.into(),
            hardness,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MineTile {
    pub world_x: i32,
    pub world_y: i32,
    pub block_air: bool,
    pub floor_drop: Option<MineItem>,
    pub wall_drop: Option<MineItem>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinerType {
    pub mine_tier: i32,
    pub mine_floor: bool,
    pub mine_walls: bool,
    pub mine_range: f32,
    pub mine_speed: f32,
    pub mine_hardness_scaling: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinerUpdateContext {
    pub delta: f32,
    pub unit_mine_speed_rule: f32,
    pub net_client: bool,
    pub is_local: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MinerUpdatePlan {
    pub mined_item: Option<String>,
    pub cleared_mine_tile: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinerComp {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub hit_size: f32,
    pub type_info: MinerType,
    pub mine_timer: f32,
    pub mine_tile: Option<MineTile>,
    pub is_player: bool,
    pub actively_building: bool,
    pub stack_item: Option<String>,
    pub stack_amount: i32,
    pub item_capacity: i32,
}

impl MinerComp {
    pub fn new(type_info: MinerType) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            hit_size: 0.0,
            type_info,
            mine_timer: 0.0,
            mine_tile: None,
            is_player: false,
            actively_building: false,
            stack_item: None,
            stack_amount: 0,
            item_capacity: 0,
        }
    }

    pub fn can_mine_item(&self, item: Option<&MineItem>) -> bool {
        item.is_some_and(|item| self.type_info.mine_tier >= item.hardness)
    }

    pub fn offload_immediately(&self) -> bool {
        self.is_player
    }

    pub fn mining(&self) -> bool {
        self.mine_tile.is_some() && !self.actively_building
    }

    pub fn get_mine_result(&self, tile: Option<&MineTile>) -> Option<MineItem> {
        let tile = tile?;
        let result = if self.type_info.mine_floor && tile.block_air {
            tile.floor_drop.clone()
        } else if self.type_info.mine_walls {
            tile.wall_drop.clone()
        } else {
            None
        };
        result.filter(|item| self.can_mine_item(Some(item)))
    }

    pub fn valid_mine(&self, tile: Option<&MineTile>, check_dst: bool) -> bool {
        let Some(tile) = tile else {
            return false;
        };
        if check_dst
            && !within(
                self.x,
                self.y,
                tile.world_x as f32,
                tile.world_y as f32,
                self.type_info.mine_range,
            )
        {
            return false;
        }
        self.get_mine_result(Some(tile)).is_some()
    }

    pub fn can_mine(&self, unit_mine_speed_rule: f32) -> bool {
        self.type_info.mine_speed * unit_mine_speed_rule > 0.0 && self.type_info.mine_tier >= 0
    }

    pub fn accepts_item(&self, item: &MineItem) -> bool {
        self.stack_amount <= 0
            || (self.stack_item.as_deref() == Some(item.name.as_str())
                && self.stack_amount + 1 <= self.item_capacity)
    }

    pub fn add_item(&mut self, item: &MineItem) {
        if self.stack_item.as_deref() == Some(item.name.as_str()) {
            self.stack_amount += 1;
        } else {
            self.stack_item = Some(item.name.clone());
            self.stack_amount = 1;
        }
        self.stack_amount = self.stack_amount.clamp(0, self.item_capacity);
    }

    pub fn update(&mut self, ctx: MinerUpdateContext) -> MinerUpdatePlan {
        let mut plan = MinerUpdatePlan::default();
        let Some(tile) = self.mine_tile.clone() else {
            return plan;
        };
        let item = self.get_mine_result(Some(&tile));

        if (!ctx.net_client || ctx.is_local) && !self.valid_mine(Some(&tile), true) {
            self.mine_tile = None;
            self.mine_timer = 0.0;
            plan.cleared_mine_tile = true;
            return plan;
        }

        if self.mining() {
            if let Some(item) = item {
                self.mine_timer += ctx.delta * self.type_info.mine_speed * ctx.unit_mine_speed_rule;
                let mine_time = 50.0
                    + if self.type_info.mine_hardness_scaling {
                        item.hardness as f32 * 15.0
                    } else {
                        15.0
                    };
                if self.mine_timer >= mine_time {
                    self.mine_timer = 0.0;
                    if self.accepts_item(&item) {
                        self.add_item(&item);
                        plan.mined_item = Some(item.name);
                    } else {
                        self.mine_tile = None;
                        plan.cleared_mine_tile = true;
                    }
                }
            }
        }

        plan
    }
}

fn within(x: f32, y: f32, tx: f32, ty: f32, range: f32) -> bool {
    let dx = x - tx;
    let dy = y - ty;
    dx * dx + dy * dy <= range * range
}

#[cfg(test)]
mod tests {
    use super::*;

    fn miner_type() -> MinerType {
        MinerType {
            mine_tier: 2,
            mine_floor: true,
            mine_walls: false,
            mine_range: 20.0,
            mine_speed: 1.0,
            mine_hardness_scaling: true,
        }
    }

    fn tile() -> MineTile {
        MineTile {
            world_x: 0,
            world_y: 0,
            block_air: true,
            floor_drop: Some(MineItem::new("copper", 1)),
            wall_drop: Some(MineItem::new("lead", 1)),
        }
    }

    #[test]
    fn miner_component_selects_floor_or_wall_mine_result_by_type_rules() {
        let miner = MinerComp::new(miner_type());
        assert_eq!(miner.get_mine_result(Some(&tile())).unwrap().name, "copper");

        let mut walls = MinerComp::new(MinerType {
            mine_floor: false,
            mine_walls: true,
            ..miner_type()
        });
        assert_eq!(walls.get_mine_result(Some(&tile())).unwrap().name, "lead");

        walls.type_info.mine_tier = 0;
        assert!(walls.get_mine_result(Some(&tile())).is_none());
    }

    #[test]
    fn miner_component_validates_distance_and_capability() {
        let mut miner = MinerComp::new(miner_type());
        miner.x = 100.0;
        assert!(!miner.valid_mine(Some(&tile()), true));
        assert!(miner.valid_mine(Some(&tile()), false));
        assert!(miner.can_mine(1.0));
    }

    #[test]
    fn miner_component_update_mines_item_after_required_time() {
        let mut miner = MinerComp::new(miner_type());
        miner.item_capacity = 5;
        miner.mine_tile = Some(tile());

        let plan = miner.update(MinerUpdateContext {
            delta: 65.0,
            unit_mine_speed_rule: 1.0,
            net_client: false,
            is_local: true,
        });

        assert_eq!(plan.mined_item, Some("copper".into()));
        assert_eq!(miner.stack_item.as_deref(), Some("copper"));
        assert_eq!(miner.stack_amount, 1);
        assert_eq!(miner.mine_timer, 0.0);
    }
}
