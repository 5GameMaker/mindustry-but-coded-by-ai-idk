//! Placement HUD fragment state model mirroring upstream
//! `mindustry.ui.fragments.PlacementFragment`.

use std::collections::HashMap;

use crate::mindustry::r#type::Category;

pub const PLACEMENT_ROW_WIDTH: usize = 4;
pub const PLACEMENT_BLOCK_SELECT_TIMEOUT_MILLIS: u64 = 400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementBlock {
    pub name: String,
    pub category: Category,
    pub visible: bool,
    pub unlocked_now_host: bool,
    pub placeable_player: bool,
    pub environment_buildable: bool,
    pub supports_env: bool,
    pub placeable: bool,
    pub localized_name: String,
    pub requirements: Vec<PlacementRequirement>,
}

impl PlacementBlock {
    pub fn new(name: impl Into<String>, category: Category) -> Self {
        let name = name.into();
        Self {
            localized_name: name.clone(),
            name,
            category,
            visible: true,
            unlocked_now_host: true,
            placeable_player: true,
            environment_buildable: true,
            supports_env: true,
            placeable: true,
            requirements: Vec::new(),
        }
    }

    pub fn with_placeable(mut self, placeable: bool) -> Self {
        self.placeable = placeable;
        self
    }

    pub fn with_unlocked(mut self, unlocked: bool) -> Self {
        self.unlocked_now_host = unlocked;
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn with_environment_buildable(mut self, environment_buildable: bool) -> Self {
        self.environment_buildable = environment_buildable;
        self
    }

    pub fn with_supports_env(mut self, supports_env: bool) -> Self {
        self.supports_env = supports_env;
        self
    }

    pub fn with_requirement(mut self, item: impl Into<String>, amount: i32) -> Self {
        self.requirements.push(PlacementRequirement {
            item_name: item.into(),
            amount,
        });
        self
    }

    pub fn unlocked_for_player(&self) -> bool {
        self.unlocked_now_host
            && self.placeable_player
            && self.environment_buildable
            && self.supports_env
    }

    fn visible_in_category(&self, category: Category) -> bool {
        self.category == category && self.visible && self.environment_buildable
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementRequirement {
    pub item_name: String,
    pub amount: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementKey {
    BlockSelect(usize),
    BlockSelectLeft,
    BlockSelectRight,
    BlockSelectUp,
    BlockSelectDown,
    CategoryPrev,
    CategoryNext,
    BlockInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementInputContext {
    pub now_millis: u64,
    pub chat_shown: bool,
    pub console_shown: bool,
    pub keyboard_focus: bool,
}

impl PlacementInputContext {
    pub const fn new(now_millis: u64) -> Self {
        Self {
            now_millis,
            chat_shown: false,
            console_shown: false,
            keyboard_focus: false,
        }
    }

    fn blocks_grid_input(&self) -> bool {
        !(self.chat_shown || self.console_shown || self.keyboard_focus)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacementDisplayable {
    Unit { id: i32, type_name: String },
    Building { id: i32, block_name: String },
    TileDrop { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementHoverContext {
    pub scene_has_mouse: bool,
    pub top_table_hit: bool,
    pub unit: Option<PlacementDisplayable>,
    pub building: Option<PlacementDisplayable>,
    pub building_in_fog: bool,
    pub tile_drop: Option<PlacementDisplayable>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacementAction {
    RebuildPalette,
    ClearInputBlock,
    SelectCategory(Category),
    SelectBlock(String),
    DeselectBlock,
    RestorePickedBlock {
        block_name: String,
        config: Option<String>,
    },
    ShowBlockInfo(String),
    ShowUnitInfo(String),
    FireBlockInfoEvent,
    RefreshCommandTable,
    SwitchToCommandTable,
    SwitchToBlockCategoryTable,
    StopFlow(i32),
    UpdateFlow(i32),
    Handled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementPaletteCell {
    pub block_name: String,
    pub placeable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlacementPaletteModel {
    pub current_category: Category,
    pub categories: Vec<Category>,
    pub category_empty: [bool; Category::ALL.len()],
    pub blocks: Vec<PlacementPaletteCell>,
    pub restored_scroll_y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementInfoBoxModel {
    pub source_is_hovered_world: bool,
    pub display_state: PlacementInfoDisplay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacementInfoDisplay {
    Block {
        name: String,
        localized_name: String,
        requirements: Vec<PlacementRequirement>,
        placeable: bool,
    },
    Displayable(PlacementDisplayable),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlacementFragment {
    current_category: Category,
    selected_blocks: HashMap<Category, String>,
    scroll_positions: HashMap<Category, f32>,
    category_empty: [bool; Category::ALL.len()],
    menu_hover_block: Option<String>,
    hover: Option<PlacementDisplayable>,
    last_flow_build: Option<i32>,
    next_flow_build: Option<i32>,
    last_display_state: Option<PlacementInfoDisplay>,
    last_team: Option<i32>,
    was_hovered: bool,
    was_command_mode: bool,
    block_select_end: bool,
    block_select_seq: usize,
    block_select_seq_millis: u64,
    input_block: Option<String>,
}

impl Default for PlacementFragment {
    fn default() -> Self {
        Self::new()
    }
}

impl PlacementFragment {
    pub fn new() -> Self {
        Self {
            current_category: Category::Distribution,
            selected_blocks: HashMap::new(),
            scroll_positions: HashMap::new(),
            category_empty: [true; Category::ALL.len()],
            menu_hover_block: None,
            hover: None,
            last_flow_build: None,
            next_flow_build: None,
            last_display_state: None,
            last_team: None,
            was_hovered: false,
            was_command_mode: false,
            block_select_end: false,
            block_select_seq: 0,
            block_select_seq_millis: 0,
            input_block: None,
        }
    }

    pub fn current_category(&self) -> Category {
        self.current_category
    }

    pub fn input_block(&self) -> Option<&str> {
        self.input_block.as_deref()
    }

    pub fn hover(&self) -> Option<&PlacementDisplayable> {
        self.hover.as_ref()
    }

    pub fn selected_block_for(&self, category: Category) -> Option<&str> {
        self.selected_blocks.get(&category).map(String::as_str)
    }

    pub fn category_empty(&self, category: Category) -> bool {
        self.category_empty[category.ordinal()]
    }

    pub fn block_select_state(&self) -> (bool, usize, u64) {
        (
            self.block_select_end,
            self.block_select_seq,
            self.block_select_seq_millis,
        )
    }

    pub fn world_loaded(&mut self) -> Vec<PlacementAction> {
        self.current_category = Category::Distribution;
        self.input_block = None;
        self.last_display_state = None;
        vec![
            PlacementAction::ClearInputBlock,
            PlacementAction::RebuildPalette,
        ]
    }

    pub fn unlock_event(&mut self, content_is_block: bool) -> Vec<PlacementAction> {
        if content_is_block {
            self.rebuild()
        } else {
            Vec::new()
        }
    }

    pub fn reset_event(&mut self) {
        self.selected_blocks.clear();
    }

    pub fn unit_command_changed(&self) -> Vec<PlacementAction> {
        vec![PlacementAction::RefreshCommandTable]
    }

    pub fn rebuild(&mut self) -> Vec<PlacementAction> {
        self.last_display_state = None;
        vec![PlacementAction::RebuildPalette]
    }

    pub fn remember_scroll(&mut self, scroll_y: f32) {
        self.scroll_positions
            .insert(self.current_category, scroll_y);
    }

    pub fn scroll_for(&self, category: Category) -> f32 {
        self.scroll_positions.get(&category).copied().unwrap_or(0.0)
    }

    pub fn set_menu_hover_block(&mut self, block: Option<impl Into<String>>) {
        self.menu_hover_block = block.map(Into::into);
    }

    pub fn rebuild_palette(&mut self, blocks: &[PlacementBlock]) -> PlacementPaletteModel {
        for category in Category::ALL {
            self.category_empty[category.ordinal()] =
                self.unlocked_by_category(blocks, category).is_empty();
        }

        if self.category_empty[self.current_category.ordinal()] {
            for category in self.sorted_categories() {
                if !self.category_empty[category.ordinal()] {
                    self.current_category = category;
                    break;
                }
            }
        }

        let blocks = self
            .unlocked_by_category(blocks, self.current_category)
            .into_iter()
            .map(|block| PlacementPaletteCell {
                block_name: block.name.clone(),
                placeable: block.placeable,
            })
            .collect();

        PlacementPaletteModel {
            current_category: self.current_category,
            categories: self.sorted_categories(),
            category_empty: self.category_empty,
            blocks,
            restored_scroll_y: self.scroll_for(self.current_category),
        }
    }

    pub fn click_block(
        &mut self,
        block_name: &str,
        unicode: Option<char>,
        copy_modifier: bool,
    ) -> Vec<PlacementAction> {
        if copy_modifier {
            if unicode.is_some() {
                return vec![PlacementAction::Handled];
            }
            return Vec::new();
        }

        if self.input_block.as_deref() == Some(block_name) {
            self.input_block = None;
            self.selected_blocks.remove(&self.current_category);
            vec![PlacementAction::DeselectBlock]
        } else {
            self.input_block = Some(block_name.to_string());
            self.selected_blocks
                .insert(self.current_category, block_name.to_string());
            vec![PlacementAction::SelectBlock(block_name.to_string())]
        }
    }

    pub fn restore_picked_block(
        &mut self,
        block: &PlacementBlock,
        config: Option<String>,
        editor: bool,
    ) -> Vec<PlacementAction> {
        if (block.visible && block.unlocked_for_player()) || editor {
            self.input_block = Some(block.name.clone());
            if block.visible {
                self.current_category = block.category;
            }
            self.selected_blocks
                .insert(self.current_category, block.name.clone());
            vec![
                PlacementAction::RestorePickedBlock {
                    block_name: block.name.clone(),
                    config,
                },
                PlacementAction::SelectBlock(block.name.clone()),
            ]
        } else {
            Vec::new()
        }
    }

    pub fn grid_update(
        &mut self,
        key: PlacementKey,
        blocks: &[PlacementBlock],
        context: PlacementInputContext,
    ) -> Vec<PlacementAction> {
        if !context.blocks_grid_input() {
            return Vec::new();
        }

        match key {
            PlacementKey::BlockSelect(index) if index < 10 => {
                self.number_key(index, blocks, context.now_millis)
            }
            PlacementKey::BlockSelectLeft => self.directional_select(-1, 0, blocks),
            PlacementKey::BlockSelectRight => self.directional_select(1, 0, blocks),
            PlacementKey::BlockSelectUp => self.directional_select(0, -1, blocks),
            PlacementKey::BlockSelectDown => self.directional_select(0, 1, blocks),
            PlacementKey::CategoryPrev => self.step_category(false, blocks),
            PlacementKey::CategoryNext => self.step_category(true, blocks),
            PlacementKey::BlockInfo => self.show_block_info_action(blocks),
            PlacementKey::BlockSelect(_) => Vec::new(),
        }
    }

    pub fn resolve_hover(
        &mut self,
        context: PlacementHoverContext,
    ) -> Option<PlacementDisplayable> {
        if context.scene_has_mouse || context.top_table_hit {
            self.hover = None;
            return None;
        }

        let hover = if let Some(unit) = context.unit {
            Some(unit)
        } else if !context.building_in_fog {
            match context.building {
                Some(PlacementDisplayable::Building { id, block_name }) => {
                    self.next_flow_build = Some(id);
                    Some(PlacementDisplayable::Building { id, block_name })
                }
                other => other,
            }
        } else {
            None
        }
        .or(context.tile_drop);

        self.hover = hover.clone();
        hover
    }

    pub fn set_next_flow_build(&mut self, building: Option<i32>) {
        self.next_flow_build = building;
    }

    pub fn update_flow_tick(&mut self) -> Vec<PlacementAction> {
        let mut actions = Vec::new();
        if let Some(last) = self.last_flow_build {
            if self.last_flow_build != self.next_flow_build {
                actions.push(PlacementAction::StopFlow(last));
            }
        }

        self.last_flow_build = self.next_flow_build;
        if let Some(next) = self.next_flow_build {
            actions.push(PlacementAction::UpdateFlow(next));
        }
        actions
    }

    pub fn update_command_mode(&mut self, command_mode: bool) -> Vec<PlacementAction> {
        if command_mode == self.was_command_mode {
            return Vec::new();
        }

        self.was_command_mode = command_mode;
        if command_mode {
            vec![PlacementAction::SwitchToCommandTable]
        } else {
            vec![PlacementAction::SwitchToBlockCategoryTable]
        }
    }

    pub fn info_box_model(
        &mut self,
        blocks: &[PlacementBlock],
        player_team: i32,
    ) -> Option<PlacementInfoBoxModel> {
        let display_block = self
            .menu_hover_block
            .as_deref()
            .or(self.input_block.as_deref())
            .and_then(|name| blocks.iter().find(|block| block.name == name));

        let source_is_hovered_world = display_block.is_none();
        let display_state = if let Some(block) = display_block {
            PlacementInfoDisplay::Block {
                name: block.name.clone(),
                localized_name: block.localized_name.clone(),
                requirements: block.requirements.clone(),
                placeable: block.placeable,
            }
        } else {
            PlacementInfoDisplay::Displayable(self.hover.clone()?)
        };

        if self.was_hovered == source_is_hovered_world
            && self.last_display_state.as_ref() == Some(&display_state)
            && self.last_team == Some(player_team)
        {
            return None;
        }

        self.last_display_state = Some(display_state.clone());
        self.was_hovered = source_is_hovered_world;
        self.last_team = Some(player_team);

        Some(PlacementInfoBoxModel {
            source_is_hovered_world,
            display_state,
        })
    }

    fn number_key(
        &mut self,
        mut index: usize,
        blocks: &[PlacementBlock],
        now_millis: u64,
    ) -> Vec<PlacementAction> {
        if self.block_select_end
            || now_millis.saturating_sub(self.block_select_seq_millis)
                > PLACEMENT_BLOCK_SELECT_TIMEOUT_MILLIS
        {
            let category = Category::ALL[index];
            if !self.unlocked_by_category(blocks, category).is_empty() {
                self.current_category = category;
                let mut actions = vec![PlacementAction::SelectCategory(category)];
                if self.input_block.is_some() {
                    if let Some(selected) = self.selected_block_name(blocks, category) {
                        self.input_block = Some(selected.clone());
                        actions.push(PlacementAction::SelectBlock(selected));
                    } else {
                        self.input_block = None;
                        actions.push(PlacementAction::DeselectBlock);
                    }
                }
                self.block_select_end = false;
                self.block_select_seq = 0;
                self.block_select_seq_millis = now_millis;
                actions
            } else {
                Vec::new()
            }
        } else {
            if self.block_select_seq == 0 {
                self.block_select_seq = index + 1;
            } else {
                index += (self.block_select_seq - usize::from(index == 9)) * 10;
                self.block_select_end = true;
            }

            let by_category = self.by_category(blocks, self.current_category);
            if index >= by_category.len() || !by_category[index].unlocked_for_player() {
                return vec![PlacementAction::Handled];
            }

            let block_name = by_category[index].name.clone();
            self.input_block = Some(block_name.clone());
            self.selected_blocks
                .insert(self.current_category, block_name.clone());
            self.block_select_seq_millis = now_millis;
            vec![PlacementAction::SelectBlock(block_name)]
        }
    }

    fn directional_select(
        &mut self,
        dx: i32,
        dy: i32,
        blocks: &[PlacementBlock],
    ) -> Vec<PlacementAction> {
        let unlocked = self.unlocked_by_category(blocks, self.current_category);
        let Some(current) = self.selected_block_name(blocks, self.current_category) else {
            return Vec::new();
        };
        let Some(mut index) = unlocked.iter().position(|block| block.name == current) else {
            return Vec::new();
        };

        if dx < 0 {
            index = (index + unlocked.len() - 1) % unlocked.len();
        } else if dx > 0 {
            index = (index + 1) % unlocked.len();
        } else if dy < 0 {
            index = if index > PLACEMENT_ROW_WIDTH - 1 {
                index - PLACEMENT_ROW_WIDTH
            } else {
                unlocked.len() - unlocked.len() % PLACEMENT_ROW_WIDTH + index
            };
            if index >= unlocked.len() {
                index -= PLACEMENT_ROW_WIDTH;
            }
        } else if dy > 0 {
            index = if index < unlocked.len().saturating_sub(PLACEMENT_ROW_WIDTH) {
                index + PLACEMENT_ROW_WIDTH
            } else {
                index % PLACEMENT_ROW_WIDTH
            };
        }

        let block_name = unlocked[index].name.clone();
        self.input_block = Some(block_name.clone());
        self.selected_blocks
            .insert(self.current_category, block_name.clone());
        vec![PlacementAction::SelectBlock(block_name)]
    }

    fn step_category(&mut self, next: bool, blocks: &[PlacementBlock]) -> Vec<PlacementAction> {
        self.rebuild_palette(blocks);
        for _ in 0..self.category_empty.len() {
            self.current_category = if next {
                self.current_category.next()
            } else {
                self.current_category.prev()
            };
            if !self.category_empty[self.current_category.ordinal()] {
                break;
            }
        }

        let mut actions = vec![PlacementAction::SelectCategory(self.current_category)];
        if let Some(selected) = self.selected_block_name(blocks, self.current_category) {
            self.input_block = Some(selected.clone());
            actions.push(PlacementAction::SelectBlock(selected));
        } else {
            self.input_block = None;
            actions.push(PlacementAction::DeselectBlock);
        }
        actions
    }

    fn show_block_info_action(&self, blocks: &[PlacementBlock]) -> Vec<PlacementAction> {
        if let Some(PlacementDisplayable::Unit { type_name, .. }) = &self.hover {
            return vec![PlacementAction::ShowUnitInfo(type_name.clone())];
        }

        let block = self
            .menu_hover_block
            .as_deref()
            .or(self.input_block.as_deref())
            .and_then(|name| blocks.iter().find(|block| block.name == name));

        if let Some(block) = block {
            if block.unlocked_now_host {
                return vec![
                    PlacementAction::ShowBlockInfo(block.name.clone()),
                    PlacementAction::FireBlockInfoEvent,
                ];
            }
        }
        Vec::new()
    }

    fn sorted_categories(&self) -> Vec<Category> {
        let mut categories = Category::ALL.to_vec();
        categories.sort_by_key(|category| self.category_empty[category.ordinal()]);
        categories
    }

    fn by_category<'a>(
        &self,
        blocks: &'a [PlacementBlock],
        category: Category,
    ) -> Vec<&'a PlacementBlock> {
        blocks
            .iter()
            .filter(|block| block.visible_in_category(category))
            .collect()
    }

    fn unlocked_by_category<'a>(
        &self,
        blocks: &'a [PlacementBlock],
        category: Category,
    ) -> Vec<&'a PlacementBlock> {
        let mut selected = blocks
            .iter()
            .filter(|block| {
                block.category == category && block.visible && block.unlocked_for_player()
            })
            .collect::<Vec<_>>();
        selected.sort_by_key(|block| !block.placeable);
        selected
    }

    fn selected_block_name(&self, blocks: &[PlacementBlock], category: Category) -> Option<String> {
        self.selected_blocks.get(&category).cloned().or_else(|| {
            self.by_category(blocks, category)
                .into_iter()
                .find(|block| block.unlocked_for_player())
                .map(|block| block.name.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocks() -> Vec<PlacementBlock> {
        vec![
            PlacementBlock::new("duo", Category::Turret),
            PlacementBlock::new("scatter", Category::Turret),
            PlacementBlock::new("drill", Category::Production),
            PlacementBlock::new("router", Category::Distribution),
            PlacementBlock::new("junction", Category::Distribution),
            PlacementBlock::new("bridge", Category::Distribution).with_placeable(false),
            PlacementBlock::new("conduit", Category::Liquid),
            PlacementBlock::new("graphite-press", Category::Crafting),
        ]
    }

    #[test]
    fn world_load_resets_distribution_and_requests_rebuild_like_java_event() {
        let mut fragment = PlacementFragment::new();
        fragment.current_category = Category::Turret;
        fragment.input_block = Some("duo".into());

        let actions = fragment.world_loaded();

        assert_eq!(fragment.current_category(), Category::Distribution);
        assert_eq!(fragment.input_block(), None);
        assert_eq!(
            actions,
            vec![
                PlacementAction::ClearInputBlock,
                PlacementAction::RebuildPalette
            ]
        );
    }

    #[test]
    fn rebuild_palette_marks_empty_categories_and_sorts_unplaceable_last() {
        let mut fragment = PlacementFragment::new();
        fragment.remember_scroll(42.0);

        let model = fragment.rebuild_palette(&blocks());

        assert_eq!(model.current_category, Category::Distribution);
        assert_eq!(model.restored_scroll_y, 42.0);
        assert!(!model.category_empty[Category::Distribution.ordinal()]);
        assert!(model.category_empty[Category::Power.ordinal()]);
        assert_eq!(
            model
                .blocks
                .iter()
                .map(|cell| cell.block_name.as_str())
                .collect::<Vec<_>>(),
            vec!["router", "junction", "bridge"]
        );
        assert!(!model.blocks[2].placeable);
        assert_eq!(model.categories.last(), Some(&Category::Logic));
    }

    #[test]
    fn reset_clears_per_category_selected_blocks() {
        let mut fragment = PlacementFragment::new();
        fragment.click_block("router", None, false);
        assert_eq!(
            fragment.selected_block_for(Category::Distribution),
            Some("router")
        );

        fragment.reset_event();

        assert_eq!(fragment.selected_block_for(Category::Distribution), None);
    }

    #[test]
    fn numeric_keys_select_category_then_block_with_java_timeout_rules() {
        let mut fragment = PlacementFragment::new();
        let blocks = blocks();

        let actions = fragment.grid_update(
            PlacementKey::BlockSelect(Category::Turret.ordinal()),
            &blocks,
            PlacementInputContext::new(1000),
        );
        assert_eq!(
            actions,
            vec![PlacementAction::SelectCategory(Category::Turret)]
        );
        assert_eq!(fragment.current_category(), Category::Turret);

        let actions = fragment.grid_update(
            PlacementKey::BlockSelect(1),
            &blocks,
            PlacementInputContext::new(1100),
        );

        assert_eq!(
            actions,
            vec![PlacementAction::SelectBlock("scatter".into())]
        );
        assert_eq!(fragment.input_block(), Some("scatter"));
        assert_eq!(fragment.block_select_state(), (false, 2, 1100));
    }

    #[test]
    fn third_numeric_key_uses_java_two_digit_block_formula() {
        let mut fragment = PlacementFragment::new();
        let mut many = blocks();
        for i in 0..20 {
            many.push(PlacementBlock::new(
                format!("dist-{i:02}"),
                Category::Distribution,
            ));
        }

        fragment.grid_update(
            PlacementKey::BlockSelect(Category::Distribution.ordinal()),
            &many,
            PlacementInputContext::new(1000),
        );
        fragment.grid_update(
            PlacementKey::BlockSelect(0),
            &many,
            PlacementInputContext::new(1100),
        );
        fragment.grid_update(
            PlacementKey::BlockSelect(9),
            &many,
            PlacementInputContext::new(1200),
        );

        assert_eq!(fragment.input_block(), Some("dist-06"));
        assert_eq!(fragment.block_select_state().0, true);
    }

    #[test]
    fn directional_select_moves_on_four_column_grid() {
        let mut fragment = PlacementFragment::new();
        let mut many = Vec::new();
        for i in 0..9 {
            many.push(PlacementBlock::new(
                format!("dist-{i}"),
                Category::Distribution,
            ));
        }
        fragment.click_block("dist-0", None, false);

        fragment.grid_update(
            PlacementKey::BlockSelectDown,
            &many,
            PlacementInputContext::new(1000),
        );
        assert_eq!(fragment.input_block(), Some("dist-4"));

        fragment.grid_update(
            PlacementKey::BlockSelectUp,
            &many,
            PlacementInputContext::new(1100),
        );
        assert_eq!(fragment.input_block(), Some("dist-0"));

        fragment.grid_update(
            PlacementKey::BlockSelectLeft,
            &many,
            PlacementInputContext::new(1200),
        );
        assert_eq!(fragment.input_block(), Some("dist-8"));
    }

    #[test]
    fn category_next_and_prev_skip_empty_categories() {
        let mut fragment = PlacementFragment::new();
        let blocks = blocks();
        fragment.rebuild_palette(&blocks);

        fragment.grid_update(
            PlacementKey::CategoryNext,
            &blocks,
            PlacementInputContext::new(1000),
        );
        assert_eq!(fragment.current_category(), Category::Liquid);
        assert_eq!(fragment.input_block(), Some("conduit"));

        fragment.grid_update(
            PlacementKey::CategoryPrev,
            &blocks,
            PlacementInputContext::new(1100),
        );
        assert_eq!(fragment.current_category(), Category::Distribution);
        assert_eq!(fragment.input_block(), Some("router"));
    }

    #[test]
    fn hover_priority_prefers_unit_then_building_then_tile_drop_and_ignores_ui_mouse() {
        let mut fragment = PlacementFragment::new();
        let hover = fragment.resolve_hover(PlacementHoverContext {
            scene_has_mouse: false,
            top_table_hit: false,
            unit: Some(PlacementDisplayable::Unit {
                id: 1,
                type_name: "dagger".into(),
            }),
            building: Some(PlacementDisplayable::Building {
                id: 2,
                block_name: "router".into(),
            }),
            building_in_fog: false,
            tile_drop: Some(PlacementDisplayable::TileDrop {
                name: "copper".into(),
            }),
        });
        assert!(matches!(hover, Some(PlacementDisplayable::Unit { .. })));

        let hover = fragment.resolve_hover(PlacementHoverContext {
            scene_has_mouse: false,
            top_table_hit: false,
            unit: None,
            building: Some(PlacementDisplayable::Building {
                id: 2,
                block_name: "router".into(),
            }),
            building_in_fog: false,
            tile_drop: Some(PlacementDisplayable::TileDrop {
                name: "copper".into(),
            }),
        });
        assert!(matches!(hover, Some(PlacementDisplayable::Building { .. })));

        let hover = fragment.resolve_hover(PlacementHoverContext {
            scene_has_mouse: true,
            top_table_hit: false,
            unit: None,
            building: None,
            building_in_fog: false,
            tile_drop: Some(PlacementDisplayable::TileDrop {
                name: "copper".into(),
            }),
        });
        assert_eq!(hover, None);
    }

    #[test]
    fn command_mode_and_flow_updates_emit_java_like_actions() {
        let mut fragment = PlacementFragment::new();
        assert_eq!(
            fragment.update_command_mode(true),
            vec![PlacementAction::SwitchToCommandTable]
        );
        assert!(fragment.update_command_mode(true).is_empty());
        assert_eq!(
            fragment.update_command_mode(false),
            vec![PlacementAction::SwitchToBlockCategoryTable]
        );

        fragment.set_next_flow_build(Some(10));
        assert_eq!(
            fragment.update_flow_tick(),
            vec![PlacementAction::UpdateFlow(10)]
        );
        fragment.set_next_flow_build(Some(11));
        assert_eq!(
            fragment.update_flow_tick(),
            vec![
                PlacementAction::StopFlow(10),
                PlacementAction::UpdateFlow(11)
            ]
        );
    }

    #[test]
    fn info_box_uses_menu_hover_then_input_block_then_world_hover() {
        let mut fragment = PlacementFragment::new();
        let blocks = blocks();
        fragment.input_block = Some("router".into());
        fragment.set_menu_hover_block(Some("duo"));

        let model = fragment.info_box_model(&blocks, 1).unwrap();
        assert_eq!(
            model.display_state,
            PlacementInfoDisplay::Block {
                name: "duo".into(),
                localized_name: "duo".into(),
                requirements: vec![],
                placeable: true,
            }
        );
        assert!(!model.source_is_hovered_world);

        fragment.set_menu_hover_block(None::<String>);
        fragment.input_block = None;
        fragment.hover = Some(PlacementDisplayable::TileDrop {
            name: "copper".into(),
        });
        let model = fragment.info_box_model(&blocks, 1).unwrap();
        assert!(model.source_is_hovered_world);
        assert_eq!(
            model.display_state,
            PlacementInfoDisplay::Displayable(PlacementDisplayable::TileDrop {
                name: "copper".into()
            })
        );
    }
}
