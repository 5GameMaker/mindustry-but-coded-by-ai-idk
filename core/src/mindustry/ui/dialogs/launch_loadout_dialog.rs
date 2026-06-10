//! Launch-loadout dialog model mirroring upstream `mindustry.ui.dialogs.LaunchLoadoutDialog`.

use crate::mindustry::r#type::ItemStack;

pub const LAUNCH_LOADOUT_DIALOG_TITLE: &str = "@configure";
pub const LAUNCH_LOADOUT_BACK_BUTTON_TEXT: &str = "@back";
pub const LAUNCH_LOADOUT_BACK_ICON: &str = "left";
pub const LAUNCH_LOADOUT_BUTTON_SIZE: (f32, f32) = (160.0, 64.0);
pub const LAUNCH_LOADOUT_MAX_BUTTON_TEXT: &str = "@resources.max";
pub const LAUNCH_LOADOUT_MAX_ICON: &str = "add";
pub const LAUNCH_LOADOUT_RESOURCES_BUTTON_TEXT: &str = "@resources";
pub const LAUNCH_LOADOUT_RESOURCES_ICON: &str = "edit";
pub const LAUNCH_LOADOUT_CONFIRM_TEXT: &str = "@launch.text";
pub const LAUNCH_LOADOUT_CONFIRM_ICON: &str = "ok";
pub const LAUNCH_LOADOUT_MISSING_RESOURCES_TEXT: &str = "@sector.missingresources";
pub const LAUNCH_LOADOUT_SCHEMATIC_BUTTON_SIZE: f32 = 200.0;
pub const LAUNCH_LOADOUT_SCHEMATIC_BUTTON_PAD: f32 = 4.0;
pub const LAUNCH_LOADOUT_ITEM_ICON_SIZE: f32 = 32.0;
pub const LAUNCH_LOADOUT_PORTRAIT_CONFIRM_WIDTH: f32 = 160.0 + 160.0 + 4.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchCore {
    pub name: String,
    pub size: i32,
    pub item_capacity: i32,
}

impl LaunchCore {
    pub fn new(name: impl Into<String>, size: i32, item_capacity: i32) -> Self {
        Self {
            name: name.into(),
            size,
            item_capacity,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchSector {
    pub id: i32,
    pub name: String,
    pub planet_name: String,
    pub planet_launch_capacity_multiplier: f32,
    pub planet_allow_launch_loadout: bool,
    pub items: Vec<ItemStack>,
}

impl LaunchSector {
    pub fn new(id: i32, name: impl Into<String>, planet_name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            planet_name: planet_name.into(),
            planet_launch_capacity_multiplier: 0.25,
            planet_allow_launch_loadout: false,
            items: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchDestination {
    pub id: i32,
    pub name: String,
    pub allow_launch_loadout: bool,
    pub allow_launch_schematics: bool,
    pub preset_description: Option<String>,
    pub preset_rules_loadout: Vec<ItemStack>,
}

impl LaunchDestination {
    pub fn new(id: i32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            allow_launch_loadout: true,
            allow_launch_schematics: true,
            preset_description: None,
            preset_rules_loadout: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchLoadoutSchematic {
    pub name: String,
    pub file: Option<String>,
    pub core_name: String,
    pub core_size: i32,
    pub core_item_capacity: i32,
    pub requirements: Vec<ItemStack>,
    pub supports_source_planet: bool,
}

impl LaunchLoadoutSchematic {
    pub fn new(
        name: impl Into<String>,
        core_name: impl Into<String>,
        core_size: i32,
        core_item_capacity: i32,
    ) -> Self {
        Self {
            name: name.into(),
            file: None,
            core_name: core_name.into(),
            core_size,
            core_item_capacity,
            requirements: Vec::new(),
            supports_source_planet: true,
        }
    }

    pub fn file_stem(&self) -> Option<String> {
        self.file.as_deref().map(file_stem_like_java)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchLoadoutContext {
    pub item_order: Vec<String>,
    pub source_planet_items: Vec<String>,
    pub launch_resources: Vec<ItemStack>,
    pub selected_schematic_file_stem: Option<String>,
    pub max_resources: bool,
    pub mobile: bool,
    pub portrait: bool,
    pub debug_select: bool,
    pub graphics_width: i32,
}

impl Default for LaunchLoadoutContext {
    fn default() -> Self {
        Self {
            item_order: Vec::new(),
            source_planet_items: Vec::new(),
            launch_resources: Vec::new(),
            selected_schematic_file_stem: None,
            max_resources: true,
            mobile: false,
            portrait: false,
            debug_select: false,
            graphics_width: 800,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchLoadoutDialogModel {
    pub title: &'static str,
    pub from_label: String,
    pub buttons: Vec<LaunchLoadoutButton>,
    pub rows_layout: bool,
    pub schematic_columns: i32,
    pub selected_schematic_name: String,
    pub schematic_options: Vec<LaunchLoadoutSchematicOption>,
    pub show_schematics: bool,
    pub preset_description: Option<String>,
    pub capacity_label: Option<String>,
    pub last_capacity: i32,
    pub total: Vec<ItemStack>,
    pub item_rows: Vec<LaunchLoadoutItemRow>,
    pub valid: bool,
    pub missing_resources_visible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchLoadoutButton {
    pub text: &'static str,
    pub icon: &'static str,
    pub size: (f32, f32),
    pub checked: bool,
    pub disabled: bool,
    pub colspan: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchLoadoutSchematicOption {
    pub name: String,
    pub file_stem: Option<String>,
    pub checked: bool,
    pub size: f32,
    pub pad: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchLoadoutItemRow {
    pub item: String,
    pub available: i32,
    pub schematic_amount: i32,
    pub launch_amount: i32,
    pub total_amount: i32,
    pub text: String,
    pub enough: bool,
    pub icon_size: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchLoadoutDialogAction {
    SetMaxResources(bool),
    UpdateLaunchResources(Vec<ItemStack>),
    ShowLoadoutDialog {
        capacity: i32,
        available: Vec<ItemStack>,
        current: Vec<ItemStack>,
        allowed_items: Vec<String>,
    },
    SelectSchematic {
        name: String,
    },
    UpdateLoadout {
        core_name: String,
        schematic_name: String,
    },
    RunConfirm,
    HideDialog,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchLoadoutDialog {
    pub selected: Option<LaunchLoadoutSchematic>,
    pub total: Vec<ItemStack>,
    pub valid: bool,
    pub last_capacity: i32,
}

impl Default for LaunchLoadoutDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl LaunchLoadoutDialog {
    pub fn new() -> Self {
        Self {
            selected: None,
            total: Vec::new(),
            valid: false,
            last_capacity: 0,
        }
    }

    pub fn show(
        &mut self,
        core: &LaunchCore,
        source: &LaunchSector,
        destination: &LaunchDestination,
        schematics: &[LaunchLoadoutSchematic],
        context: &LaunchLoadoutContext,
    ) -> LaunchLoadoutDialogModel {
        let selected = choose_schematic(core, schematics, context).unwrap_or_else(|| {
            LaunchLoadoutSchematic::new(
                core.name.clone(),
                core.name.clone(),
                core.size,
                core.item_capacity,
            )
        });
        self.selected = Some(selected);
        self.rebuild(core, source, destination, schematics, context)
    }

    pub fn rebuild(
        &mut self,
        core: &LaunchCore,
        source: &LaunchSector,
        destination: &LaunchDestination,
        schematics: &[LaunchLoadoutSchematic],
        context: &LaunchLoadoutContext,
    ) -> LaunchLoadoutDialogModel {
        let selected = self
            .selected
            .clone()
            .unwrap_or_else(|| choose_schematic(core, schematics, context).unwrap());
        let calculation = calculate_loadout(source, destination, &selected, context);
        self.last_capacity = calculation.capacity;
        self.total = calculation.total.clone();
        self.valid = calculation.valid;

        LaunchLoadoutDialogModel {
            title: LAUNCH_LOADOUT_DIALOG_TITLE,
            from_label: format!("launch.from:{}", source.name),
            buttons: buttons(destination, context, calculation.valid),
            rows_layout: context.portrait && context.mobile,
            schematic_columns: schematic_columns(context.graphics_width),
            selected_schematic_name: selected.name.clone(),
            schematic_options: schematic_options(core, schematics, &selected, destination),
            show_schematics: destination.allow_launch_schematics,
            preset_description: (!destination.allow_launch_schematics)
                .then(|| destination.preset_description.clone())
                .flatten(),
            capacity_label: destination
                .allow_launch_schematics
                .then(|| format!("launch.capacity:{}", calculation.capacity)),
            last_capacity: calculation.capacity,
            total: calculation.total.clone(),
            item_rows: item_rows(
                &context.item_order,
                &source.items,
                &selected.requirements,
                &calculation.launch_resources,
                destination.allow_launch_loadout,
            ),
            valid: calculation.valid,
            missing_resources_visible: !calculation.valid,
        }
    }

    pub fn toggle_max_plan(max: bool) -> LaunchLoadoutDialogAction {
        LaunchLoadoutDialogAction::SetMaxResources(max)
    }

    pub fn edit_resources_plan(
        &self,
        source: &LaunchSector,
        context: &LaunchLoadoutContext,
    ) -> LaunchLoadoutDialogAction {
        let selected_requirements = self
            .selected
            .as_ref()
            .map(|schematic| schematic.requirements.clone())
            .unwrap_or_default();
        let available = subtract_stacks(&context.item_order, &source.items, &selected_requirements);
        LaunchLoadoutDialogAction::ShowLoadoutDialog {
            capacity: self.last_capacity,
            available,
            current: context.launch_resources.clone(),
            allowed_items: context.source_planet_items.clone(),
        }
    }

    pub fn select_schematic_plan(
        &mut self,
        schematic: LaunchLoadoutSchematic,
    ) -> LaunchLoadoutDialogAction {
        self.selected = Some(schematic.clone());
        LaunchLoadoutDialogAction::SelectSchematic {
            name: schematic.name,
        }
    }

    pub fn confirm_plan(&self, core: &LaunchCore) -> Vec<LaunchLoadoutDialogAction> {
        let schematic_name = self
            .selected
            .as_ref()
            .map(|schematic| schematic.name.clone())
            .unwrap_or_else(|| core.name.clone());
        vec![
            LaunchLoadoutDialogAction::UpdateLoadout {
                core_name: core.name.clone(),
                schematic_name,
            },
            LaunchLoadoutDialogAction::RunConfirm,
            LaunchLoadoutDialogAction::HideDialog,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchLoadoutCalculation {
    pub capacity: i32,
    pub launch_resources: Vec<ItemStack>,
    pub total: Vec<ItemStack>,
    pub valid: bool,
}

pub fn calculate_loadout(
    source: &LaunchSector,
    destination: &LaunchDestination,
    selected: &LaunchLoadoutSchematic,
    context: &LaunchLoadoutContext,
) -> LaunchLoadoutCalculation {
    let capacity =
        (source.planet_launch_capacity_multiplier * selected.core_item_capacity as f32) as i32;
    let mut resources = normalize_stacks(&context.item_order, &context.launch_resources);
    if source.planet_allow_launch_loadout {
        resources.retain(|stack| {
            context
                .source_planet_items
                .iter()
                .any(|item| item == &stack.item)
        });
    }
    min_stacks(&context.item_order, &mut resources, capacity);

    if !destination.allow_launch_loadout {
        resources.clear();
        for stack in &destination.preset_rules_loadout {
            if context
                .source_planet_items
                .iter()
                .any(|item| item == &stack.item)
            {
                add_stack(&context.item_order, &mut resources, stack);
            }
        }
    } else if context.max_resources {
        resources.clear();
        for item in &context.item_order {
            let available = get_stack(&source.items, item);
            let schematic = get_stack(&selected.requirements, item);
            let amount = (available - schematic).clamp(0, capacity);
            if amount != 0 {
                resources.push(ItemStack::new(item.clone(), amount));
            }
        }
    }

    let mut total = normalize_stacks(&context.item_order, &selected.requirements);
    for stack in &resources {
        add_stack(&context.item_order, &mut total, stack);
    }
    let valid = has_stacks(&source.items, &total) || context.debug_select;

    LaunchLoadoutCalculation {
        capacity,
        launch_resources: normalize_stacks(&context.item_order, &resources),
        total: normalize_stacks(&context.item_order, &total),
        valid,
    }
}

pub fn choose_schematic(
    core: &LaunchCore,
    schematics: &[LaunchLoadoutSchematic],
    context: &LaunchLoadoutContext,
) -> Option<LaunchLoadoutSchematic> {
    let compatible = schematics
        .iter()
        .filter(|schematic| schematic.core_size <= core.size)
        .cloned()
        .collect::<Vec<_>>();

    context
        .selected_schematic_file_stem
        .as_ref()
        .and_then(|stem| {
            compatible
                .iter()
                .find(|schematic| schematic.file_stem().as_deref() == Some(stem.as_str()))
                .cloned()
        })
        .or_else(|| compatible.first().cloned())
}

fn buttons(
    destination: &LaunchDestination,
    context: &LaunchLoadoutContext,
    valid: bool,
) -> Vec<LaunchLoadoutButton> {
    let rows = context.portrait && context.mobile;
    let mut buttons = vec![LaunchLoadoutButton {
        text: LAUNCH_LOADOUT_BACK_BUTTON_TEXT,
        icon: LAUNCH_LOADOUT_BACK_ICON,
        size: LAUNCH_LOADOUT_BUTTON_SIZE,
        checked: false,
        disabled: false,
        colspan: 1,
    }];

    if destination.allow_launch_loadout {
        buttons.push(LaunchLoadoutButton {
            text: LAUNCH_LOADOUT_MAX_BUTTON_TEXT,
            icon: LAUNCH_LOADOUT_MAX_ICON,
            size: LAUNCH_LOADOUT_BUTTON_SIZE,
            checked: context.max_resources,
            disabled: false,
            colspan: 1,
        });
        buttons.push(LaunchLoadoutButton {
            text: LAUNCH_LOADOUT_RESOURCES_BUTTON_TEXT,
            icon: LAUNCH_LOADOUT_RESOURCES_ICON,
            size: LAUNCH_LOADOUT_BUTTON_SIZE,
            checked: false,
            disabled: context.max_resources,
            colspan: 1,
        });
    }

    buttons.push(LaunchLoadoutButton {
        text: LAUNCH_LOADOUT_CONFIRM_TEXT,
        icon: LAUNCH_LOADOUT_CONFIRM_ICON,
        size: if rows {
            (
                LAUNCH_LOADOUT_PORTRAIT_CONFIRM_WIDTH,
                LAUNCH_LOADOUT_BUTTON_SIZE.1,
            )
        } else {
            LAUNCH_LOADOUT_BUTTON_SIZE
        },
        checked: false,
        disabled: !valid,
        colspan: if rows { 2 } else { 1 },
    });
    buttons
}

fn schematic_options(
    core: &LaunchCore,
    schematics: &[LaunchLoadoutSchematic],
    selected: &LaunchLoadoutSchematic,
    destination: &LaunchDestination,
) -> Vec<LaunchLoadoutSchematicOption> {
    if !destination.allow_launch_schematics {
        return Vec::new();
    }

    schematics
        .iter()
        .filter(|schematic| schematic.core_size <= core.size && schematic.supports_source_planet)
        .map(|schematic| LaunchLoadoutSchematicOption {
            name: schematic.name.clone(),
            file_stem: schematic.file_stem(),
            checked: schematic.name == selected.name,
            size: LAUNCH_LOADOUT_SCHEMATIC_BUTTON_SIZE,
            pad: LAUNCH_LOADOUT_SCHEMATIC_BUTTON_PAD,
        })
        .collect()
}

fn item_rows(
    item_order: &[String],
    source_items: &[ItemStack],
    schematic: &[ItemStack],
    launch: &[ItemStack],
    allow_launch_loadout: bool,
) -> Vec<LaunchLoadoutItemRow> {
    item_order
        .iter()
        .filter_map(|item| {
            let schematic_amount = get_stack(schematic, item);
            let launch_amount = get_stack(launch, item);
            let total_amount = schematic_amount + launch_amount;
            if total_amount == 0 {
                return None;
            }
            let available = get_stack(source_items, item);
            let amount = if allow_launch_loadout {
                format!("{total_amount}[gray] ({launch_amount} + {schematic_amount})")
            } else {
                total_amount.to_string()
            };
            let enough = available >= total_amount;
            let text = if enough {
                amount
            } else {
                format!(
                    "[scarlet]{}[lightgray]/{}",
                    available.min(total_amount),
                    amount
                )
            };
            Some(LaunchLoadoutItemRow {
                item: item.clone(),
                available,
                schematic_amount,
                launch_amount,
                total_amount,
                text,
                enough,
                icon_size: LAUNCH_LOADOUT_ITEM_ICON_SIZE,
            })
        })
        .collect()
}

fn schematic_columns(graphics_width: i32) -> i32 {
    (graphics_width / 230).max(1)
}

fn normalize_stacks(item_order: &[String], stacks: &[ItemStack]) -> Vec<ItemStack> {
    item_order
        .iter()
        .filter_map(|item| {
            let amount = get_stack(stacks, item);
            (amount != 0).then(|| ItemStack::new(item.clone(), amount))
        })
        .collect()
}

fn min_stacks(item_order: &[String], stacks: &mut Vec<ItemStack>, cap: i32) {
    for item in item_order {
        let amount = get_stack(stacks, item).min(cap);
        set_stack(stacks, item, amount);
    }
}

fn subtract_stacks(
    item_order: &[String],
    left: &[ItemStack],
    right: &[ItemStack],
) -> Vec<ItemStack> {
    item_order
        .iter()
        .filter_map(|item| {
            let amount = get_stack(left, item) - get_stack(right, item);
            (amount != 0).then(|| ItemStack::new(item.clone(), amount))
        })
        .collect()
}

fn has_stacks(source: &[ItemStack], required: &[ItemStack]) -> bool {
    required
        .iter()
        .all(|stack| get_stack(source, &stack.item) >= stack.amount)
}

fn add_stack(item_order: &[String], stacks: &mut Vec<ItemStack>, stack: &ItemStack) {
    if item_order.iter().any(|item| item == &stack.item) {
        let next = get_stack(stacks, &stack.item) + stack.amount;
        set_stack(stacks, &stack.item, next);
    }
}

fn set_stack(stacks: &mut Vec<ItemStack>, item: &str, amount: i32) {
    if let Some(existing) = stacks.iter_mut().find(|stack| stack.item == item) {
        existing.amount = amount;
    } else if amount != 0 {
        stacks.push(ItemStack::new(item, amount));
    }
    stacks.retain(|stack| stack.amount != 0);
}

fn get_stack(stacks: &[ItemStack], item: &str) -> i32 {
    stacks
        .iter()
        .find_map(|stack| (stack.item == item).then_some(stack.amount))
        .unwrap_or(0)
}

fn file_stem_like_java(path: &str) -> String {
    let name = path
        .rsplit(['/', '\\'])
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(path);
    name.rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or_else(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(item: &str, amount: i32) -> ItemStack {
        ItemStack::new(item, amount)
    }

    fn context() -> LaunchLoadoutContext {
        LaunchLoadoutContext {
            item_order: vec!["copper".into(), "lead".into(), "graphite".into()],
            source_planet_items: vec!["copper".into(), "lead".into(), "graphite".into()],
            max_resources: false,
            graphics_width: 460,
            ..LaunchLoadoutContext::default()
        }
    }

    fn core() -> LaunchCore {
        LaunchCore::new("core-shard", 2, 4000)
    }

    fn source() -> LaunchSector {
        let mut sector = LaunchSector::new(1, "Ground Zero", "serpulo");
        sector.planet_launch_capacity_multiplier = 0.25;
        sector.planet_allow_launch_loadout = true;
        sector.items = vec![
            stack("copper", 1000),
            stack("lead", 200),
            stack("graphite", 10),
        ];
        sector
    }

    fn destination() -> LaunchDestination {
        LaunchDestination::new(2, "Frozen Forest")
    }

    fn schematic(name: &str, file: &str, req: Vec<ItemStack>) -> LaunchLoadoutSchematic {
        let mut schematic = LaunchLoadoutSchematic::new(name, "core-shard", 1, 4000);
        schematic.file = Some(file.into());
        schematic.requirements = req;
        schematic
    }

    #[test]
    fn choose_schematic_uses_last_file_stem_then_first_compatible_like_universe_get_loadout() {
        let alpha = schematic("alpha", "schematics/alpha.msch", vec![]);
        let beta = schematic("beta", "C:/schematics/beta.msch", vec![]);
        let mut ctx = context();
        ctx.selected_schematic_file_stem = Some("beta".into());

        assert_eq!(
            choose_schematic(&core(), &[alpha.clone(), beta.clone()], &ctx)
                .unwrap()
                .name,
            "beta"
        );

        ctx.selected_schematic_file_stem = Some("missing".into());
        assert_eq!(
            choose_schematic(&core(), &[alpha.clone(), beta], &ctx)
                .unwrap()
                .name,
            "alpha"
        );
    }

    #[test]
    fn calculate_loadout_caps_launch_resources_and_sums_schematic_requirements() {
        let mut ctx = context();
        ctx.launch_resources = vec![stack("copper", 1200), stack("lead", 50)];
        let selected = schematic("alpha", "alpha.msch", vec![stack("copper", 100)]);

        let calc = calculate_loadout(&source(), &destination(), &selected, &ctx);

        assert_eq!(calc.capacity, 1000);
        assert_eq!(
            calc.launch_resources,
            vec![stack("copper", 1000), stack("lead", 50)]
        );
        assert_eq!(calc.total, vec![stack("copper", 1100), stack("lead", 50)]);
        assert!(!calc.valid);
    }

    #[test]
    fn calculate_loadout_removes_launch_resources_not_available_on_source_planet() {
        let mut ctx = context();
        ctx.source_planet_items = vec!["copper".into(), "lead".into()];
        ctx.launch_resources = vec![stack("copper", 10), stack("graphite", 10)];
        let mut src = source();
        src.planet_allow_launch_loadout = true;
        let selected = schematic("alpha", "alpha.msch", vec![]);

        let calc = calculate_loadout(&src, &destination(), &selected, &ctx);

        assert_eq!(calc.launch_resources, vec![stack("copper", 10)]);
        assert_eq!(calc.total, vec![stack("copper", 10)]);
    }

    #[test]
    fn max_resources_fills_each_item_from_source_minus_schematic_requirement() {
        let mut ctx = context();
        ctx.max_resources = true;
        let selected = schematic(
            "alpha",
            "alpha.msch",
            vec![stack("copper", 100), stack("graphite", 12)],
        );

        let calc = calculate_loadout(&source(), &destination(), &selected, &ctx);

        assert_eq!(
            calc.launch_resources,
            vec![stack("copper", 900), stack("lead", 200)]
        );
        assert_eq!(
            calc.total,
            vec![
                stack("copper", 1000),
                stack("lead", 200),
                stack("graphite", 12)
            ]
        );
        assert!(!calc.valid);
    }

    #[test]
    fn destination_without_launch_loadout_uses_preset_rules_loadout_and_description() {
        let ctx = context();
        let mut dest = destination();
        dest.allow_launch_loadout = false;
        dest.allow_launch_schematics = false;
        dest.preset_description = Some("Survive waves.".into());
        dest.preset_rules_loadout = vec![stack("copper", 50), stack("silicon", 99)];
        let selected = schematic("alpha", "alpha.msch", vec![stack("lead", 10)]);

        let calc = calculate_loadout(&source(), &dest, &selected, &ctx);

        assert_eq!(calc.launch_resources, vec![stack("copper", 50)]);
        assert_eq!(calc.total, vec![stack("copper", 50), stack("lead", 10)]);
        assert!(calc.valid);

        let mut dialog = LaunchLoadoutDialog::new();
        dialog.selected = Some(selected);
        let model = dialog.rebuild(&core(), &source(), &dest, &[], &ctx);
        assert_eq!(model.preset_description, Some("Survive waves.".into()));
        assert!(model.capacity_label.is_none());
        assert!(model.schematic_options.is_empty());
    }

    #[test]
    fn show_builds_buttons_rows_schematics_capacity_and_missing_resource_state() {
        let mut ctx = context();
        ctx.launch_resources = vec![stack("lead", 20)];
        ctx.mobile = true;
        ctx.portrait = true;
        let alpha = schematic("alpha", "alpha.msch", vec![stack("copper", 100)]);
        let beta = schematic("beta", "beta.msch", vec![stack("lead", 5)]);
        let mut dialog = LaunchLoadoutDialog::new();

        let model = dialog.show(&core(), &source(), &destination(), &[alpha, beta], &ctx);

        assert_eq!(model.title, "@configure");
        assert_eq!(model.from_label, "launch.from:Ground Zero");
        assert!(model.rows_layout);
        assert_eq!(model.schematic_columns, 2);
        assert_eq!(model.selected_schematic_name, "alpha");
        assert_eq!(model.schematic_options.len(), 2);
        assert!(model.schematic_options[0].checked);
        assert_eq!(model.capacity_label, Some("launch.capacity:1000".into()));
        assert!(model.valid);
        assert!(!model.missing_resources_visible);
        assert_eq!(model.item_rows[0].text, "100[gray] (0 + 100)");
        assert_eq!(model.item_rows[1].text, "20[gray] (20 + 0)");

        let confirm = model.buttons.last().unwrap();
        assert_eq!(confirm.text, "@launch.text");
        assert_eq!(confirm.size, (324.0, 64.0));
        assert_eq!(confirm.colspan, 2);
        assert!(!confirm.disabled);
    }

    #[test]
    fn resources_button_is_disabled_when_max_resources_is_checked() {
        let mut ctx = context();
        ctx.max_resources = true;
        let mut dialog = LaunchLoadoutDialog::new();

        let model = dialog.show(
            &core(),
            &source(),
            &destination(),
            &[schematic("alpha", "alpha.msch", vec![])],
            &ctx,
        );

        let resources = model
            .buttons
            .iter()
            .find(|button| button.text == "@resources")
            .unwrap();
        assert!(resources.disabled);
        assert!(
            model
                .buttons
                .iter()
                .find(|button| button.text == "@resources.max")
                .unwrap()
                .checked
        );
    }

    #[test]
    fn edit_resources_plan_subtracts_schematic_requirements_from_available_items() {
        let mut dialog = LaunchLoadoutDialog::new();
        dialog.selected = Some(schematic("alpha", "alpha.msch", vec![stack("copper", 100)]));
        dialog.last_capacity = 1000;
        let mut ctx = context();
        ctx.launch_resources = vec![stack("lead", 20)];

        assert_eq!(
            dialog.edit_resources_plan(&source(), &ctx),
            LaunchLoadoutDialogAction::ShowLoadoutDialog {
                capacity: 1000,
                available: vec![
                    stack("copper", 900),
                    stack("lead", 200),
                    stack("graphite", 10)
                ],
                current: vec![stack("lead", 20)],
                allowed_items: vec!["copper".into(), "lead".into(), "graphite".into()],
            }
        );
    }

    #[test]
    fn confirm_updates_loadout_runs_callback_and_hides() {
        let mut dialog = LaunchLoadoutDialog::new();
        dialog.selected = Some(schematic("alpha", "alpha.msch", vec![]));

        assert_eq!(
            dialog.confirm_plan(&core()),
            vec![
                LaunchLoadoutDialogAction::UpdateLoadout {
                    core_name: "core-shard".into(),
                    schematic_name: "alpha".into(),
                },
                LaunchLoadoutDialogAction::RunConfirm,
                LaunchLoadoutDialogAction::HideDialog,
            ]
        );
    }
}
