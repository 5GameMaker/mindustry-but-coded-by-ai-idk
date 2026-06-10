//! Block inventory popup model mirroring upstream `BlockInventoryFragment`.

use crate::mindustry::ui::upstream_menu_bundle_value_for_locale;

pub const BLOCK_INVENTORY_HOLD_WITHDRAW: f32 = 20.0;
pub const BLOCK_INVENTORY_HOLD_SHRINK: f32 = 120.0;
pub const BLOCK_INVENTORY_COLUMNS: usize = 3;
pub const BLOCK_INVENTORY_MARGIN: f32 = 4.0;
pub const BLOCK_INVENTORY_CELL_SIZE: f32 = 40.0;
pub const BLOCK_INVENTORY_CELL_PAD: f32 = 4.0;

#[derive(Debug, Clone, PartialEq)]
pub struct BlockInventoryBuildingRef {
    pub id: i32,
    pub pos: i32,
    pub x: f32,
    pub y: f32,
    pub block_size: i32,
    pub accessible: bool,
    pub has_items: bool,
    pub valid: bool,
    pub can_withdraw: bool,
    pub item_amounts: Vec<i32>,
}

impl BlockInventoryBuildingRef {
    pub fn total(&self) -> i32 {
        self.item_amounts.iter().sum()
    }

    pub fn has_item(&self, index: usize) -> bool {
        self.item_amounts.get(index).copied().unwrap_or(0) > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInventoryItemCell {
    pub item_index: usize,
    pub item_name: String,
    pub amount: i32,
    pub label: String,
    pub can_pick: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockInventoryModel {
    pub visible: bool,
    pub cols: usize,
    pub margin: f32,
    pub cell_size: f32,
    pub cell_pad: f32,
    pub position_top_left: (f32, f32),
    pub cells: Vec<BlockInventoryItemCell>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockInventoryAction {
    RequestBlockSnapshot(i32),
    RequestItem {
        building_id: i32,
        item_index: usize,
        amount: i32,
    },
    WithdrawEvent {
        building_id: i32,
        item_index: usize,
        amount: i32,
    },
    Hide,
    Rebuild {
        with_actions: bool,
    },
    SetPositionTopLeft {
        x: f32,
        y: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockInventoryUpdateContext {
    pub state_is_menu: bool,
    pub state_is_paused: bool,
    pub player_dead: bool,
    pub player_within_transfer_range: bool,
    pub player_max_accepted: i32,
    pub net_client: bool,
    pub mouse_screen: (f32, f32),
    pub tile_size: f32,
    pub delta: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockInventoryFragment {
    build: Option<i32>,
    hold_time: f32,
    empty_time: f32,
    holding: bool,
    held: bool,
    shrink_hold_times: Vec<f32>,
    last_item: Option<usize>,
    table_visible: bool,
}

impl BlockInventoryFragment {
    pub fn new(item_count: usize) -> Self {
        Self {
            build: None,
            hold_time: 0.0,
            empty_time: 0.0,
            holding: false,
            held: false,
            shrink_hold_times: vec![0.0; item_count],
            last_item: None,
            table_visible: false,
        }
    }

    pub fn build(&mut self) {
        self.table_visible = false;
    }

    pub fn show_for(
        &mut self,
        building: &BlockInventoryBuildingRef,
        item_names: &[String],
        context: &BlockInventoryUpdateContext,
    ) -> (Option<BlockInventoryModel>, Vec<BlockInventoryAction>) {
        if self.build == Some(building.id) {
            return (None, self.hide());
        }

        self.build = Some(building.id);
        let mut actions = vec![BlockInventoryAction::RequestBlockSnapshot(building.pos)];
        if !building.accessible || building.total() == 0 {
            return (None, actions);
        }

        let model = self.rebuild(building, item_names, context);
        actions.push(BlockInventoryAction::Rebuild { with_actions: true });
        (Some(model), actions)
    }

    pub fn hide(&mut self) -> Vec<BlockInventoryAction> {
        self.table_visible = false;
        self.build = None;
        self.holding = false;
        self.last_item = None;
        vec![BlockInventoryAction::Hide]
    }

    pub fn touch_down(&mut self, item_index: usize, valid_click: bool) {
        self.held = false;
        if valid_click {
            self.last_item = Some(item_index);
            self.holding = true;
        }
    }

    pub fn clicked(
        &mut self,
        building: &BlockInventoryBuildingRef,
        item_index: usize,
        context: &BlockInventoryUpdateContext,
        valid_click: bool,
    ) -> Vec<BlockInventoryAction> {
        if !valid_click || self.held {
            return Vec::new();
        }
        self.last_item = Some(item_index);
        self.take_item(
            building,
            item_index,
            building.item_amounts[item_index],
            context.player_max_accepted,
            context.net_client,
        )
    }

    pub fn touch_up(&mut self) {
        self.holding = false;
        self.last_item = None;
    }

    pub fn update(
        &mut self,
        building: Option<&BlockInventoryBuildingRef>,
        item_names: &[String],
        context: &BlockInventoryUpdateContext,
    ) -> (Option<BlockInventoryModel>, Vec<BlockInventoryAction>) {
        let Some(building) = building else {
            return (None, self.hide());
        };

        if context.state_is_menu
            || self.build != Some(building.id)
            || !building.valid
            || !building.accessible
            || self.empty_time >= BLOCK_INVENTORY_HOLD_SHRINK
        {
            return (None, self.hide());
        }

        if building.total() == 0 {
            self.empty_time += context.delta;
        } else {
            self.empty_time = 0.0;
        }

        let mut actions = Vec::new();
        if self.holding {
            if let Some(item) = self.last_item {
                self.hold_time += context.delta;
                if self.hold_time >= BLOCK_INVENTORY_HOLD_WITHDRAW {
                    self.hold_time = 0.0;
                    actions.extend(self.take_item(
                        building,
                        item,
                        1,
                        context.player_max_accepted,
                        context.net_client,
                    ));
                }
            }
        }

        actions.push(BlockInventoryAction::SetPositionTopLeft {
            x: context.mouse_screen.0,
            y: context.mouse_screen.1,
        });

        let model = self.rebuild(building, item_names, context);
        if model.cells.is_empty() {
            return (None, self.hide());
        }
        (Some(model), actions)
    }

    fn take_item(
        &mut self,
        building: &BlockInventoryBuildingRef,
        item_index: usize,
        requested: i32,
        max_accepted: i32,
        net_client: bool,
    ) -> Vec<BlockInventoryAction> {
        if !building.can_withdraw {
            return Vec::new();
        }
        let amount = requested.min(max_accepted);
        if amount > 0 {
            self.holding = false;
            self.hold_time = 0.0;
            self.held = true;
            let mut actions = vec![BlockInventoryAction::RequestItem {
                building_id: building.id,
                item_index,
                amount,
            }];
            if net_client {
                actions.push(BlockInventoryAction::WithdrawEvent {
                    building_id: building.id,
                    item_index,
                    amount,
                });
            }
            actions
        } else {
            Vec::new()
        }
    }

    fn rebuild(
        &mut self,
        building: &BlockInventoryBuildingRef,
        item_names: &[String],
        context: &BlockInventoryUpdateContext,
    ) -> BlockInventoryModel {
        self.hold_time = 0.0;
        self.empty_time = 0.0;
        self.table_visible = true;

        let cells = item_names
            .iter()
            .enumerate()
            .filter_map(|(index, name)| {
                let amount = building.item_amounts.get(index).copied().unwrap_or(0);
                (amount > 0).then(|| BlockInventoryItemCell {
                    item_index: index,
                    item_name: name.clone(),
                    amount,
                    label: round_amount(amount as f32),
                    can_pick: !context.player_dead
                        && !context.state_is_paused
                        && context.player_within_transfer_range,
                })
            })
            .collect::<Vec<_>>();

        BlockInventoryModel {
            visible: true,
            cols: BLOCK_INVENTORY_COLUMNS,
            margin: BLOCK_INVENTORY_MARGIN,
            cell_size: BLOCK_INVENTORY_CELL_SIZE,
            cell_pad: BLOCK_INVENTORY_CELL_PAD,
            position_top_left: context.mouse_screen,
            cells,
        }
    }

    pub fn selected_building(&self) -> Option<i32> {
        self.build
    }
}

pub fn round_amount(amount: f32) -> String {
    let value = amount as i32;
    let thousands = upstream_menu_bundle_value_for_locale("en", "unit.thousands").unwrap_or("k");
    let millions = upstream_menu_bundle_value_for_locale("en", "unit.millions").unwrap_or("mil");
    if amount >= 1_000_000.0 {
        format!("{}[gray]{millions}", (amount / 1_000_000.0) as i32)
    } else if amount >= 1000.0 {
        format!("{}{thousands}", value / 1000)
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> BlockInventoryUpdateContext {
        BlockInventoryUpdateContext {
            state_is_menu: false,
            state_is_paused: false,
            player_dead: false,
            player_within_transfer_range: true,
            player_max_accepted: 10,
            net_client: true,
            mouse_screen: (100.0, 200.0),
            tile_size: 8.0,
            delta: 1.0,
        }
    }

    fn building() -> BlockInventoryBuildingRef {
        BlockInventoryBuildingRef {
            id: 7,
            pos: 123,
            x: 0.0,
            y: 0.0,
            block_size: 2,
            accessible: true,
            has_items: true,
            valid: true,
            can_withdraw: true,
            item_amounts: vec![1500, 3],
        }
    }

    #[test]
    fn show_for_requests_snapshot_and_builds_inventory_cells() {
        let mut fragment = BlockInventoryFragment::new(2);
        let names = vec!["copper".to_string(), "lead".to_string()];
        let (model, actions) = fragment.show_for(&building(), &names, &context());

        assert_eq!(fragment.selected_building(), Some(7));
        assert!(actions.contains(&BlockInventoryAction::RequestBlockSnapshot(123)));
        let model = model.unwrap();
        assert_eq!(model.cols, 3);
        assert_eq!(model.cells[0].label, "1k");
        assert_eq!(model.cells[1].label, "3");
    }

    #[test]
    fn clicking_valid_item_requests_all_and_emits_withdraw_event_on_client() {
        let mut fragment = BlockInventoryFragment::new(2);
        let actions = fragment.clicked(&building(), 0, &context(), true);

        assert_eq!(
            actions[0],
            BlockInventoryAction::RequestItem {
                building_id: 7,
                item_index: 0,
                amount: 10
            }
        );
        assert!(matches!(
            actions[1],
            BlockInventoryAction::WithdrawEvent { .. }
        ));
    }

    #[test]
    fn round_amount_matches_java_inventory_rounding() {
        assert_eq!(round_amount(999.0), "999");
        assert_eq!(round_amount(1500.0), "1k");
        assert_eq!(round_amount(2_300_000.0), "2[gray]mil");
    }
}
