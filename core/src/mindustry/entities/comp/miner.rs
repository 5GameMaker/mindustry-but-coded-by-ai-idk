//! Miner component mirroring upstream `mindustry.entities.comp.MinerComp`.

use crate::mindustry::{
    ai::{
        prebuild_ai_handle_full_core_for_target_item, prebuild_ai_handle_return_to_core_with_items,
        prebuild_ai_pick_missing_collect_target_item, prebuild_ai_refresh_mining_ore_target,
        prebuild_ai_should_stop_mining_for_carry_limit_or_acceptance, PrebuildAiOreTarget,
        PrebuildAiRequirement,
    },
    ctype::ContentId,
    entities::comp::builder::PrebuildAiRuntimeState,
};

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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PrebuildMiningRuntimeStep {
    pub target_item: Option<ContentId>,
    pub set_mine_tile: Option<MineTile>,
    pub cleared_mine_tile: bool,
    pub cleared_item: bool,
    pub transfer_to_core: bool,
    pub mining: bool,
    pub collecting_items: bool,
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

    #[allow(clippy::too_many_arguments)]
    pub fn apply_prebuild_mining_tick<
        FCoreHas,
        FCoreAcceptOne,
        FAcceptsItem,
        FFindFloorOre,
        FFindWallOre,
        FOreTile,
        FCanBuild,
    >(
        &mut self,
        state: &mut PrebuildAiRuntimeState,
        requirements: &[PrebuildAiRequirement],
        build_cost_multiplier: f32,
        timer_ore_ready: bool,
        core_accept_stack_one: FCoreAcceptOne,
        core_accept_stack_amount: i32,
        within_core_range: bool,
        can_build_after_deposit: FCanBuild,
        core_has_item: FCoreHas,
        accepts_item: FAcceptsItem,
        find_floor_ore: FFindFloorOre,
        find_wall_ore: FFindWallOre,
        ore_tile: FOreTile,
    ) -> PrebuildMiningRuntimeStep
    where
        FCoreHas: FnMut(ContentId, i32) -> bool,
        FCoreAcceptOne: FnMut(ContentId) -> i32,
        FAcceptsItem: FnMut(ContentId) -> bool,
        FFindFloorOre: FnMut(ContentId) -> Option<i32>,
        FFindWallOre: FnMut(ContentId) -> Option<i32>,
        FOreTile: FnMut(i32) -> Option<MineTile>,
        FCanBuild: FnMut() -> bool,
    {
        apply_prebuild_mining_tick(
            self,
            state,
            requirements,
            build_cost_multiplier,
            timer_ore_ready,
            core_accept_stack_one,
            core_accept_stack_amount,
            within_core_range,
            can_build_after_deposit,
            core_has_item,
            accepts_item,
            find_floor_ore,
            find_wall_ore,
            ore_tile,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn apply_prebuild_mining_tick<
    FCoreHas,
    FCoreAcceptOne,
    FAcceptsItem,
    FFindFloorOre,
    FFindWallOre,
    FOreTile,
    FCanBuild,
>(
    miner: &mut MinerComp,
    state: &mut PrebuildAiRuntimeState,
    requirements: &[PrebuildAiRequirement],
    build_cost_multiplier: f32,
    timer_ore_ready: bool,
    mut core_accept_stack_one: FCoreAcceptOne,
    core_accept_stack_amount: i32,
    within_core_range: bool,
    mut can_build_after_deposit: FCanBuild,
    core_has_item: FCoreHas,
    mut accepts_item: FAcceptsItem,
    mut find_floor_ore: FFindFloorOre,
    mut find_wall_ore: FFindWallOre,
    mut ore_tile: FOreTile,
) -> PrebuildMiningRuntimeStep
where
    FCoreHas: FnMut(ContentId, i32) -> bool,
    FCoreAcceptOne: FnMut(ContentId) -> i32,
    FAcceptsItem: FnMut(ContentId) -> bool,
    FFindFloorOre: FnMut(ContentId) -> Option<i32>,
    FFindWallOre: FnMut(ContentId) -> Option<i32>,
    FOreTile: FnMut(i32) -> Option<MineTile>,
    FCanBuild: FnMut() -> bool,
{
    let mut step = PrebuildMiningRuntimeStep {
        mining: state.mining,
        collecting_items: state.collecting_items,
        ..PrebuildMiningRuntimeStep::default()
    };

    if state.mining {
        let target = prebuild_ai_pick_missing_collect_target_item(
            requirements,
            build_cost_multiplier,
            state.last_target_item,
            core_has_item,
        );
        state.last_target_item = target.last_target_item;
        step.target_item = target.target_item;

        if let Some(target_item) = target.target_item {
            let full_core = prebuild_ai_handle_full_core_for_target_item(
                Some(target_item),
                core_accept_stack_one(target_item),
            );
            if full_core.handled {
                if full_core.clear_item {
                    miner.stack_item = None;
                    miner.stack_amount = 0;
                    step.cleared_item = true;
                }
                if full_core.clear_mine_tile {
                    miner.mine_tile = None;
                    state.ore = None;
                    step.cleared_mine_tile = true;
                }
                step.mining = state.mining;
                step.collecting_items = state.collecting_items;
                return step;
            }
        }

        if prebuild_ai_should_stop_mining_for_carry_limit_or_acceptance(
            target.target_item,
            miner.stack_amount,
            miner.item_capacity,
            target.target_item.is_some_and(|item| accepts_item(item)),
        ) {
            state.mining = false;
            step.mining = false;
            return step;
        }

        match prebuild_ai_refresh_mining_ore_target(
            state.ore,
            timer_ore_ready,
            target.target_item,
            miner.type_info.mine_floor,
            miner.type_info.mine_walls,
            &mut find_floor_ore,
            &mut find_wall_ore,
        ) {
            PrebuildAiOreTarget::Existing(ore) | PrebuildAiOreTarget::Refreshed(ore) => {
                state.ore = ore;
            }
        }

        if let Some(ore) = state.ore.and_then(&mut ore_tile) {
            miner.mine_tile = Some(ore.clone());
            step.set_mine_tile = Some(ore);
        }
    } else {
        miner.mine_tile = None;
        step.cleared_mine_tile = true;

        let return_action = prebuild_ai_handle_return_to_core_with_items(
            miner.stack_amount,
            within_core_range,
            core_accept_stack_amount,
            can_build_after_deposit(),
        );
        state.mining = return_action.mining;
        state.collecting_items = return_action.collecting_items;
        step.mining = return_action.mining;
        step.collecting_items = return_action.collecting_items;
        step.transfer_to_core = return_action.transfer_to_core;

        if return_action.clear_item {
            miner.stack_item = None;
            miner.stack_amount = 0;
            step.cleared_item = true;
        }
    }

    step
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

    #[test]
    fn prebuild_mining_runtime_selects_missing_item_and_sets_ore_tile() {
        let mut miner = MinerComp::new(miner_type());
        miner.item_capacity = 10;
        let mut state = PrebuildAiRuntimeState {
            collecting_items: true,
            mining: true,
            ..PrebuildAiRuntimeState::default()
        };
        let requirements = [
            PrebuildAiRequirement::new(1, 2),
            PrebuildAiRequirement::new(2, 5),
        ];

        let step = miner.apply_prebuild_mining_tick(
            &mut state,
            &requirements,
            1.5,
            true,
            |_| 1,
            0,
            false,
            || false,
            |item, amount| item == 1 && amount == 3,
            |item| item == 2,
            |_| Some(42),
            |_| None,
            |pos| {
                (pos == 42).then_some(MineTile {
                    world_x: 16,
                    world_y: 32,
                    block_air: true,
                    floor_drop: Some(MineItem::new("lead", 1)),
                    wall_drop: None,
                })
            },
        );

        assert_eq!(step.target_item, Some(2));
        assert_eq!(state.last_target_item, Some(2));
        assert_eq!(state.ore, Some(42));
        assert!(state.mining);
        assert!(state.collecting_items);
        assert_eq!(miner.mine_tile, step.set_mine_tile);
        assert_eq!(miner.mine_tile.as_ref().unwrap().world_x, 16);
    }

    #[test]
    fn prebuild_mining_runtime_switches_to_delivery_and_returns_to_core() {
        let mut miner = MinerComp::new(miner_type());
        miner.item_capacity = 3;
        miner.stack_item = Some("lead".into());
        miner.stack_amount = 3;
        let mut state = PrebuildAiRuntimeState {
            collecting_items: true,
            mining: true,
            last_target_item: Some(2),
            ..PrebuildAiRuntimeState::default()
        };
        let requirements = [PrebuildAiRequirement::new(2, 5)];

        let full = miner.apply_prebuild_mining_tick(
            &mut state,
            &requirements,
            1.0,
            true,
            |_| 1,
            0,
            false,
            || false,
            |_, _| false,
            |_| true,
            |_| Some(1),
            |_| None,
            |_| None,
        );
        assert!(!full.mining);
        assert!(!state.mining);
        assert_eq!(miner.stack_amount, 3);

        let delivered = miner.apply_prebuild_mining_tick(
            &mut state,
            &requirements,
            1.0,
            false,
            |_| 1,
            3,
            true,
            || true,
            |_, _| true,
            |_| true,
            |_| None,
            |_| None,
            |_| None,
        );
        assert!(delivered.mining);
        assert!(!delivered.collecting_items);
        assert!(delivered.transfer_to_core);
        assert!(delivered.cleared_item);
        assert_eq!(miner.stack_amount, 0);
        assert!(miner.stack_item.is_none());
        assert!(state.mining);
        assert!(!state.collecting_items);
    }

    #[test]
    fn prebuild_mining_runtime_handles_full_core_by_clearing_item_and_mine_tile() {
        let mut miner = MinerComp::new(miner_type());
        miner.item_capacity = 10;
        miner.stack_item = Some("copper".into());
        miner.stack_amount = 2;
        miner.mine_tile = Some(tile());
        let mut state = PrebuildAiRuntimeState {
            collecting_items: true,
            mining: true,
            last_target_item: Some(1),
            ore: Some(7),
            ..PrebuildAiRuntimeState::default()
        };
        let requirements = [PrebuildAiRequirement::new(1, 1)];

        let step = miner.apply_prebuild_mining_tick(
            &mut state,
            &requirements,
            1.0,
            true,
            |_| 0,
            0,
            false,
            || false,
            |_, _| true,
            |_| true,
            |_| Some(7),
            |_| None,
            |_| Some(tile()),
        );

        assert!(step.cleared_item);
        assert!(step.cleared_mine_tile);
        assert_eq!(miner.stack_amount, 0);
        assert!(miner.stack_item.is_none());
        assert!(miner.mine_tile.is_none());
        assert_eq!(state.ore, None);
        assert!(state.mining);
        assert!(state.collecting_items);
    }
}
