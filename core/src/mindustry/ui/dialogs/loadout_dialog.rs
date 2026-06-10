//! Loadout dialog model mirroring upstream `mindustry.ui.dialogs.LoadoutDialog`.

use crate::mindustry::{
    r#type::{Item, ItemStack},
    ui::upstream_menu_bundle_format_for_locale,
};

pub const LOADOUT_DIALOG_TITLE: &str = "@configure";
pub const LOADOUT_BACK_BUTTON_TEXT: &str = "@back";
pub const LOADOUT_BACK_ICON: &str = "left";
pub const LOADOUT_MAX_BUTTON_TEXT: &str = "@max";
pub const LOADOUT_MAX_ICON: &str = "export";
pub const LOADOUT_RESET_BUTTON_TEXT: &str = "@settings.reset";
pub const LOADOUT_RESET_ICON: &str = "refresh";
pub const LOADOUT_BUTTON_SIZE: (f32, f32) = (210.0, 64.0);
pub const LOADOUT_ITEMS_MARGIN: f32 = 10.0;
pub const LOADOUT_ITEM_BACKGROUND: &str = "Tex.pane";
pub const LOADOUT_ITEM_MARGIN: f32 = 4.0;
pub const LOADOUT_ITEM_MARGIN_RIGHT: f32 = 8.0;
pub const LOADOUT_ITEM_OUTER_PAD: f32 = 2.0;
pub const LOADOUT_ITEM_BUTTON_SIZE: f32 = 40.0;
pub const LOADOUT_ITEM_BUTTON_STYLE: &str = "Styles.flatt";
pub const LOADOUT_EDIT_BUTTON_STYLE: &str = "Styles.flati";
pub const LOADOUT_EDIT_ICON: &str = "pencil";
pub const LOADOUT_ITEM_ICON_SIZE: f32 = 24.0;
pub const LOADOUT_ITEM_ICON_PAD: f32 = 4.0;
pub const LOADOUT_AMOUNT_LABEL_WIDTH: f32 = 90.0;
pub const LOADOUT_TEXT_INPUT_MAX_LENGTH: usize = 10;
pub const LOADOUT_INVALID_KEY: &str = "configure.invalid";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadoutDialogContext {
    pub mobile: bool,
    pub portrait: bool,
}

impl Default for LoadoutDialogContext {
    fn default() -> Self {
        Self {
            mobile: false,
            portrait: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadoutDialogModel {
    pub title: &'static str,
    pub fill_parent: bool,
    pub items_margin: f32,
    pub items_left_aligned: bool,
    pub buttons: Vec<LoadoutDialogButton>,
    pub rows: Vec<LoadoutItemRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadoutDialogButton {
    pub text: &'static str,
    pub icon: &'static str,
    pub size: (f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadoutItemRow {
    pub item: String,
    pub item_id: i16,
    pub localized_name: String,
    pub amount: i32,
    pub background: &'static str,
    pub margin: f32,
    pub margin_right: f32,
    pub outer_pad: f32,
    pub left_aligned: bool,
    pub fill_x: bool,
    pub minus_button: LoadoutItemButton,
    pub plus_button: LoadoutItemButton,
    pub edit_button: LoadoutItemButton,
    pub icon: String,
    pub icon_size: f32,
    pub icon_pad_left: f32,
    pub icon_pad_right: f32,
    pub amount_label: String,
    pub amount_label_width: f32,
    pub row_after: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadoutItemButton {
    pub text: Option<&'static str>,
    pub icon: Option<&'static str>,
    pub style: &'static str,
    pub size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadoutDialogKey {
    Escape,
    Back,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadoutDialogAction {
    PostHideDialog,
    HideDialog,
    RunResetter,
    Reseed,
    Rebuild,
    RunUpdater,
    RunHider,
    UpdateOriginalStacks(Vec<ItemStack>),
    ShowTextInput {
        title: &'static str,
        message: String,
        max_length: usize,
        text: String,
        numeric: bool,
        item: String,
    },
    ShowInfo {
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadoutDialog {
    pub stacks: Vec<ItemStack>,
    pub original_stacks: Vec<ItemStack>,
    pub capacity: i32,
    pub total: Option<Vec<ItemStack>>,
    pub hider_present: bool,
}

impl Default for LoadoutDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadoutDialog {
    pub fn new() -> Self {
        Self {
            stacks: Vec::new(),
            original_stacks: Vec::new(),
            capacity: 0,
            total: None,
            hider_present: false,
        }
    }

    pub fn show<F>(
        &mut self,
        capacity: i32,
        total: Option<&[ItemStack]>,
        stacks: &[ItemStack],
        items: &[Item],
        mut validator: F,
        hider_present: bool,
        context: &LoadoutDialogContext,
    ) -> LoadoutDialogModel
    where
        F: FnMut(&Item) -> bool,
    {
        self.original_stacks = stacks.to_vec();
        self.capacity = capacity;
        self.total = total.map(ItemStack::copy_all);
        self.hider_present = hider_present;
        self.reseed(items, &mut validator);
        self.model(items, context)
    }

    pub fn model(&self, items: &[Item], context: &LoadoutDialogContext) -> LoadoutDialogModel {
        LoadoutDialogModel {
            title: LOADOUT_DIALOG_TITLE,
            fill_parent: true,
            items_margin: LOADOUT_ITEMS_MARGIN,
            items_left_aligned: true,
            buttons: loadout_buttons(),
            rows: self
                .stacks
                .iter()
                .enumerate()
                .map(|(index, stack)| item_row(index, stack, items, context))
                .collect(),
        }
    }

    pub fn key_down_plan(key: LoadoutDialogKey) -> Vec<LoadoutDialogAction> {
        match key {
            LoadoutDialogKey::Escape | LoadoutDialogKey::Back => {
                vec![LoadoutDialogAction::PostHideDialog]
            }
            LoadoutDialogKey::Other => Vec::new(),
        }
    }

    pub fn back_plan() -> Vec<LoadoutDialogAction> {
        vec![LoadoutDialogAction::HideDialog]
    }

    pub fn reset_plan() -> Vec<LoadoutDialogAction> {
        vec![
            LoadoutDialogAction::RunResetter,
            LoadoutDialogAction::Reseed,
            LoadoutDialogAction::RunUpdater,
            LoadoutDialogAction::Rebuild,
        ]
    }

    pub fn reseed<F>(&mut self, items: &[Item], validator: &mut F)
    where
        F: FnMut(&Item) -> bool,
    {
        self.stacks = ItemStack::copy_all(&self.original_stacks);
        let existing: Vec<String> = self.stacks.iter().map(|stack| stack.item.clone()).collect();
        let missing: Vec<ItemStack> = items
            .iter()
            .filter(|item| validator(item) && !item.is_hidden())
            .filter(|item| !existing.iter().any(|existing| existing == item.name()))
            .map(|item| ItemStack::new(item.name(), 0))
            .collect();
        self.stacks.extend(missing);
        self.sort_stacks_by_item_id(items);
    }

    pub fn max_items_plan(&mut self) -> Vec<LoadoutDialogAction> {
        for stack in &mut self.stacks {
            stack.amount = self
                .total
                .as_ref()
                .map(|total| get_stack(total, &stack.item).min(self.capacity).max(0))
                .unwrap_or(self.capacity);
        }
        Vec::new()
    }

    pub fn decrement_plan(&mut self, item: &str) -> Vec<LoadoutDialogAction> {
        let stack = self.stack_mut(item);
        stack.amount = (stack.amount - step(stack.amount)).max(0);
        vec![LoadoutDialogAction::RunUpdater]
    }

    pub fn increment_plan(&mut self, item: &str) -> Vec<LoadoutDialogAction> {
        let capacity = self.capacity;
        let stack = self.stack_mut(item);
        stack.amount = (stack.amount + step(stack.amount)).min(capacity);
        vec![LoadoutDialogAction::RunUpdater]
    }

    pub fn edit_amount_plan(&self, item: &str, items: &[Item]) -> LoadoutDialogAction {
        let stack = self.stack(item);
        let item_def = item_def(items, item);
        LoadoutDialogAction::ShowTextInput {
            title: LOADOUT_DIALOG_TITLE,
            message: item_def.localized_name().to_string(),
            max_length: LOADOUT_TEXT_INPUT_MAX_LENGTH,
            text: stack.amount.to_string(),
            numeric: true,
            item: item.to_string(),
        }
    }

    pub fn submit_amount_text(
        &mut self,
        item: &str,
        text: &str,
        locale: &str,
    ) -> Vec<LoadoutDialogAction> {
        if can_parse_positive_int(text) {
            let amount = text.parse::<i32>().unwrap();
            if amount >= 0 && amount <= self.capacity {
                self.stack_mut(item).amount = amount;
                return vec![LoadoutDialogAction::RunUpdater];
            }
        }

        vec![LoadoutDialogAction::ShowInfo {
            text: bundle_format(locale, LOADOUT_INVALID_KEY, &[&self.capacity.to_string()]),
        }]
    }

    pub fn hidden_plan(&mut self) -> Vec<LoadoutDialogAction> {
        self.original_stacks = self
            .stacks
            .iter()
            .filter(|stack| stack.amount > 0)
            .cloned()
            .collect();
        let mut actions = vec![
            LoadoutDialogAction::UpdateOriginalStacks(self.original_stacks.clone()),
            LoadoutDialogAction::RunUpdater,
        ];
        if self.hider_present {
            actions.push(LoadoutDialogAction::RunHider);
        }
        actions
    }

    fn sort_stacks_by_item_id(&mut self, items: &[Item]) {
        self.stacks
            .sort_by_key(|stack| item_def(items, &stack.item).base.mappable.base.id);
    }

    fn stack(&self, item: &str) -> &ItemStack {
        self.stacks
            .iter()
            .find(|stack| stack.item == item)
            .expect("loadout stack item must exist")
    }

    fn stack_mut(&mut self, item: &str) -> &mut ItemStack {
        self.stacks
            .iter_mut()
            .find(|stack| stack.item == item)
            .expect("loadout stack item must exist")
    }
}

pub fn step(amount: i32) -> i32 {
    if amount < 1000 {
        100
    } else if amount < 2000 {
        200
    } else if amount < 5000 {
        500
    } else {
        1000
    }
}

fn loadout_buttons() -> Vec<LoadoutDialogButton> {
    vec![
        LoadoutDialogButton {
            text: LOADOUT_BACK_BUTTON_TEXT,
            icon: LOADOUT_BACK_ICON,
            size: LOADOUT_BUTTON_SIZE,
        },
        LoadoutDialogButton {
            text: LOADOUT_MAX_BUTTON_TEXT,
            icon: LOADOUT_MAX_ICON,
            size: LOADOUT_BUTTON_SIZE,
        },
        LoadoutDialogButton {
            text: LOADOUT_RESET_BUTTON_TEXT,
            icon: LOADOUT_RESET_ICON,
            size: LOADOUT_BUTTON_SIZE,
        },
    ]
}

fn item_row(
    index: usize,
    stack: &ItemStack,
    items: &[Item],
    context: &LoadoutDialogContext,
) -> LoadoutItemRow {
    let item = item_def(items, &stack.item);
    LoadoutItemRow {
        item: stack.item.clone(),
        item_id: item.base.mappable.base.id,
        localized_name: item.localized_name().to_string(),
        amount: stack.amount,
        background: LOADOUT_ITEM_BACKGROUND,
        margin: LOADOUT_ITEM_MARGIN,
        margin_right: LOADOUT_ITEM_MARGIN_RIGHT,
        outer_pad: LOADOUT_ITEM_OUTER_PAD,
        left_aligned: true,
        fill_x: true,
        minus_button: LoadoutItemButton {
            text: Some("-"),
            icon: None,
            style: LOADOUT_ITEM_BUTTON_STYLE,
            size: LOADOUT_ITEM_BUTTON_SIZE,
        },
        plus_button: LoadoutItemButton {
            text: Some("+"),
            icon: None,
            style: LOADOUT_ITEM_BUTTON_STYLE,
            size: LOADOUT_ITEM_BUTTON_SIZE,
        },
        edit_button: LoadoutItemButton {
            text: None,
            icon: Some(LOADOUT_EDIT_ICON),
            style: LOADOUT_EDIT_BUTTON_STYLE,
            size: LOADOUT_ITEM_BUTTON_SIZE,
        },
        icon: stack.item.clone(),
        icon_size: LOADOUT_ITEM_ICON_SIZE,
        icon_pad_left: LOADOUT_ITEM_ICON_PAD,
        icon_pad_right: LOADOUT_ITEM_ICON_PAD,
        amount_label: stack.amount.to_string(),
        amount_label_width: LOADOUT_AMOUNT_LABEL_WIDTH,
        row_after: (index + 1) % 2 == 0 || (context.mobile && context.portrait),
    }
}

fn item_def<'a>(items: &'a [Item], name: &str) -> &'a Item {
    items
        .iter()
        .find(|item| item.name() == name)
        .expect("loadout item must exist in content")
}

fn get_stack(stacks: &[ItemStack], item: &str) -> i32 {
    stacks
        .iter()
        .find_map(|stack| (stack.item == item).then_some(stack.amount))
        .unwrap_or(0)
}

fn can_parse_positive_int(text: &str) -> bool {
    !text.is_empty()
        && text.bytes().all(|byte| byte.is_ascii_digit())
        && text.parse::<i32>().is_ok()
}

fn bundle_format(locale: &str, key: &str, args: &[&str]) -> String {
    upstream_menu_bundle_format_for_locale(locale, key, args)
        .or_else(|| upstream_menu_bundle_format_for_locale("en", key, args))
        .unwrap_or_else(|| format!("{key}:{}", args.join(",")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: i16, name: &str) -> Item {
        Item::new(id, name)
    }

    fn localized_item(id: i16, name: &str, localized: &str) -> Item {
        let mut item = Item::new(id, name);
        item.base.localized_name = Some(localized.into());
        item
    }

    fn hidden_item(id: i16, name: &str) -> Item {
        let mut item = Item::new(id, name);
        item.hidden = true;
        item
    }

    fn items() -> Vec<Item> {
        vec![
            item(2, "lead"),
            item(0, "scrap"),
            localized_item(1, "copper", "Copper"),
            item(3, "graphite"),
            hidden_item(4, "fissile-matter"),
        ]
    }

    fn stack(item: &str, amount: i32) -> ItemStack {
        ItemStack::new(item, amount)
    }

    #[test]
    fn show_reseeds_existing_stacks_adds_valid_visible_items_and_sorts_by_item_id() {
        let mut dialog = LoadoutDialog::new();
        let model = dialog.show(
            500,
            None,
            &[stack("copper", 250)],
            &items(),
            |item| item.name() != "graphite",
            false,
            &LoadoutDialogContext::default(),
        );

        assert_eq!(
            dialog.stacks,
            vec![stack("scrap", 0), stack("copper", 250), stack("lead", 0)]
        );
        assert_eq!(model.title, "@configure");
        assert!(model.fill_parent);
        assert_eq!(model.items_margin, 10.0);
        assert_eq!(
            model
                .rows
                .iter()
                .map(|row| (row.item.as_str(), row.item_id, row.amount))
                .collect::<Vec<_>>(),
            vec![("scrap", 0, 0), ("copper", 1, 250), ("lead", 2, 0)]
        );
    }

    #[test]
    fn show_keeps_original_hidden_or_invalid_stack_but_does_not_add_hidden_items() {
        let mut dialog = LoadoutDialog::new();
        dialog.show(
            500,
            None,
            &[stack("fissile-matter", 7)],
            &items(),
            |item| item.name() != "graphite",
            false,
            &LoadoutDialogContext::default(),
        );

        assert_eq!(
            dialog.stacks,
            vec![
                stack("scrap", 0),
                stack("copper", 0),
                stack("lead", 0),
                stack("fissile-matter", 7)
            ]
        );
    }

    #[test]
    fn model_builds_upstream_buttons_and_rows_with_mobile_portrait_row_breaks() {
        let mut dialog = LoadoutDialog::new();
        let model = dialog.show(
            500,
            None,
            &[stack("copper", 250), stack("lead", 10)],
            &items(),
            |_| true,
            true,
            &LoadoutDialogContext {
                mobile: true,
                portrait: true,
            },
        );

        assert_eq!(
            model.buttons,
            vec![
                LoadoutDialogButton {
                    text: "@back",
                    icon: "left",
                    size: (210.0, 64.0),
                },
                LoadoutDialogButton {
                    text: "@max",
                    icon: "export",
                    size: (210.0, 64.0),
                },
                LoadoutDialogButton {
                    text: "@settings.reset",
                    icon: "refresh",
                    size: (210.0, 64.0),
                },
            ]
        );
        assert!(model.rows.iter().all(|row| row.row_after));
        let copper = model.rows.iter().find(|row| row.item == "copper").unwrap();
        assert_eq!(copper.localized_name, "Copper");
        assert_eq!(copper.background, "Tex.pane");
        assert_eq!(copper.minus_button.text, Some("-"));
        assert_eq!(copper.plus_button.text, Some("+"));
        assert_eq!(copper.edit_button.icon, Some("pencil"));
        assert_eq!(copper.icon_size, 24.0);
        assert_eq!(copper.amount_label_width, 90.0);
    }

    #[test]
    fn desktop_rows_break_after_every_second_item_like_setup_loop() {
        let mut dialog = LoadoutDialog::new();
        let model = dialog.show(
            500,
            None,
            &[],
            &items(),
            |item| !item.is_hidden(),
            false,
            &LoadoutDialogContext::default(),
        );

        assert_eq!(
            model
                .rows
                .iter()
                .map(|row| row.row_after)
                .collect::<Vec<_>>(),
            vec![false, true, false, true]
        );
    }

    #[test]
    fn max_items_sets_capacity_without_total_and_clamps_each_total_amount_without_updater() {
        let mut dialog = LoadoutDialog::new();
        dialog.show(
            500,
            None,
            &[stack("copper", 1), stack("lead", 2)],
            &items(),
            |_| true,
            false,
            &LoadoutDialogContext::default(),
        );
        assert!(dialog.max_items_plan().is_empty());
        assert_eq!(dialog.stack("copper").amount, 500);
        assert_eq!(dialog.stack("lead").amount, 500);

        dialog.total = Some(vec![stack("copper", 1200), stack("lead", -10)]);
        assert!(dialog.max_items_plan().is_empty());
        assert_eq!(dialog.stack("copper").amount, 500);
        assert_eq!(dialog.stack("lead").amount, 0);
    }

    #[test]
    fn plus_minus_buttons_use_java_step_thresholds_and_run_updater() {
        let mut dialog = LoadoutDialog::new();
        dialog.show(
            3000,
            None,
            &[
                stack("copper", 999),
                stack("lead", 1000),
                stack("graphite", 5000),
            ],
            &items(),
            |_| true,
            false,
            &LoadoutDialogContext::default(),
        );

        assert_eq!(step(999), 100);
        assert_eq!(step(1000), 200);
        assert_eq!(step(2000), 500);
        assert_eq!(step(5000), 1000);
        assert_eq!(
            dialog.increment_plan("copper"),
            vec![LoadoutDialogAction::RunUpdater]
        );
        assert_eq!(dialog.stack("copper").amount, 1099);
        assert_eq!(
            dialog.decrement_plan("lead"),
            vec![LoadoutDialogAction::RunUpdater]
        );
        assert_eq!(dialog.stack("lead").amount, 800);
        assert_eq!(
            dialog.increment_plan("graphite"),
            vec![LoadoutDialogAction::RunUpdater]
        );
        assert_eq!(dialog.stack("graphite").amount, 3000);
    }

    #[test]
    fn edit_amount_plan_opens_numeric_text_input_with_localized_item_name() {
        let mut dialog = LoadoutDialog::new();
        dialog.show(
            500,
            None,
            &[stack("copper", 250)],
            &items(),
            |_| true,
            false,
            &LoadoutDialogContext::default(),
        );

        assert_eq!(
            dialog.edit_amount_plan("copper", &items()),
            LoadoutDialogAction::ShowTextInput {
                title: "@configure",
                message: "Copper".into(),
                max_length: 10,
                text: "250".into(),
                numeric: true,
                item: "copper".into(),
            }
        );
    }

    #[test]
    fn text_input_accepts_zero_to_capacity_and_shows_bundle_info_for_invalid_values() {
        let mut dialog = LoadoutDialog::new();
        dialog.show(
            500,
            None,
            &[stack("copper", 250)],
            &items(),
            |_| true,
            false,
            &LoadoutDialogContext::default(),
        );

        assert_eq!(
            dialog.submit_amount_text("copper", "0", "en"),
            vec![LoadoutDialogAction::RunUpdater]
        );
        assert_eq!(dialog.stack("copper").amount, 0);
        assert_eq!(
            dialog.submit_amount_text("copper", "501", "en"),
            vec![LoadoutDialogAction::ShowInfo {
                text: "Amount must be a number between 0 and 500.".into(),
            }]
        );
        assert_eq!(
            dialog.submit_amount_text("copper", "-1", "zh_CN"),
            vec![LoadoutDialogAction::ShowInfo {
                text: "数量必须是 0 到 500 之间的数字。".into(),
            }]
        );
    }

    #[test]
    fn hidden_plan_selects_positive_stacks_runs_updater_then_optional_hider() {
        let mut dialog = LoadoutDialog::new();
        dialog.show(
            500,
            None,
            &[stack("copper", 0), stack("lead", 20)],
            &items(),
            |_| true,
            true,
            &LoadoutDialogContext::default(),
        );
        dialog.stack_mut("scrap").amount = 5;
        dialog.stack_mut("lead").amount = 0;

        assert_eq!(
            dialog.hidden_plan(),
            vec![
                LoadoutDialogAction::UpdateOriginalStacks(vec![stack("scrap", 5)]),
                LoadoutDialogAction::RunUpdater,
                LoadoutDialogAction::RunHider,
            ]
        );
        assert_eq!(dialog.original_stacks, vec![stack("scrap", 5)]);
    }

    #[test]
    fn key_back_and_reset_plans_preserve_upstream_callback_order() {
        assert_eq!(
            LoadoutDialog::key_down_plan(LoadoutDialogKey::Escape),
            vec![LoadoutDialogAction::PostHideDialog]
        );
        assert_eq!(
            LoadoutDialog::key_down_plan(LoadoutDialogKey::Back),
            vec![LoadoutDialogAction::PostHideDialog]
        );
        assert!(LoadoutDialog::key_down_plan(LoadoutDialogKey::Other).is_empty());
        assert_eq!(
            LoadoutDialog::back_plan(),
            vec![LoadoutDialogAction::HideDialog]
        );
        assert_eq!(
            LoadoutDialog::reset_plan(),
            vec![
                LoadoutDialogAction::RunResetter,
                LoadoutDialogAction::Reseed,
                LoadoutDialogAction::RunUpdater,
                LoadoutDialogAction::Rebuild,
            ]
        );
    }
}
