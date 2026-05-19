#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumeFlags {
    pub optional: bool,
    pub booster: bool,
    pub update: bool,
}

impl Default for ConsumeFlags {
    fn default() -> Self {
        Self {
            optional: false,
            booster: false,
            update: true,
        }
    }
}

pub fn consume_optional(mut flags: ConsumeFlags, optional: bool, boost: bool) -> ConsumeFlags {
    flags.optional = optional;
    flags.booster = boost;
    flags
}

pub fn consume_boost(flags: ConsumeFlags) -> ConsumeFlags {
    consume_optional(flags, true, true)
}

pub fn consume_update(mut flags: ConsumeFlags, update: bool) -> ConsumeFlags {
    flags.update = update;
    flags
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumePowerSpec {
    pub usage: f32,
    pub capacity: f32,
    pub buffered: bool,
}

impl ConsumePowerSpec {
    pub const fn new(usage: f32, capacity: f32, buffered: bool) -> Self {
        Self {
            usage,
            capacity,
            buffered,
        }
    }
}

pub fn consume_power_ignore(buffered: bool) -> bool {
    buffered
}

pub fn consume_power_efficiency(power_status: f32) -> f32 {
    power_status
}

pub fn consume_power_requested_power(
    spec: ConsumePowerSpec,
    power_status: f32,
    should_consume: bool,
) -> f32 {
    if spec.buffered {
        (1.0 - power_status) * spec.capacity
    } else {
        spec.usage * if should_consume { 1.0 } else { 0.0 }
    }
}

pub fn consume_power_condition_requested(usage: f32, condition: bool) -> f32 {
    if condition {
        usage
    } else {
        0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemStackSpec {
    pub item_id: i16,
    pub amount: i32,
}

pub fn consume_items_trigger_amount(stack_amount: i32, multiplier: f32) -> i32 {
    (stack_amount as f32 * multiplier).round() as i32
}

pub fn consume_items_efficiency(
    stacks: &[ItemStackSpec],
    multiplier: f32,
    trigger_valid: bool,
    has_item: impl Fn(i16, i32) -> bool,
) -> f32 {
    if trigger_valid
        || stacks.iter().all(|stack| {
            has_item(
                stack.item_id,
                consume_items_trigger_amount(stack.amount, multiplier),
            )
        })
    {
        1.0
    } else {
        0.0
    }
}

pub fn consume_item_filter_get_consumed(
    item_ids_in_content_order: &[i16],
    has_item: impl Fn(i16) -> bool,
    filter: impl Fn(i16) -> bool,
) -> Option<i16> {
    item_ids_in_content_order
        .iter()
        .copied()
        .find(|item_id| has_item(*item_id) && filter(*item_id))
}

pub fn consume_item_filter_efficiency(consumed: Option<i16>, trigger_valid: bool) -> f32 {
    if trigger_valid || consumed.is_some() {
        1.0
    } else {
        0.0
    }
}

pub fn consume_item_filter_efficiency_multiplier(
    consumed: Option<i16>,
    item_multiplier: impl Fn(i16) -> f32,
) -> f32 {
    consumed.map(item_multiplier).unwrap_or(0.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemProperties {
    pub flammability: f32,
    pub explosiveness: f32,
    pub radioactivity: f32,
    pub charge: f32,
}

pub fn consume_item_flammable_filter(item: ItemProperties, min_flammability: f32) -> bool {
    item.flammability >= min_flammability
}

pub fn consume_item_explosive_filter(item: ItemProperties, min_explosiveness: f32) -> bool {
    item.explosiveness >= min_explosiveness
}

pub fn consume_item_radioactive_filter(item: ItemProperties, min_radioactivity: f32) -> bool {
    item.radioactivity >= min_radioactivity
}

pub fn consume_item_charged_filter(item: ItemProperties, min_charge: f32) -> bool {
    item.charge >= min_charge
}

pub fn consume_item_explode_chance(
    reactor_explosions: bool,
    delta: f32,
    base_chance: f32,
    explosiveness: f32,
    threshold: f32,
) -> f32 {
    if reactor_explosions {
        delta * base_chance * (explosiveness - threshold).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub fn consume_item_explode_should_damage(chance: f32, random: f32) -> bool {
    random < chance
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidStackSpec {
    pub liquid_id: i16,
    pub amount: f32,
}

pub fn consume_liquid_update_amount(amount: f32, edelta: f32, multiplier: f32) -> f32 {
    amount * edelta * multiplier
}

pub fn consume_liquid_efficiency(
    stored: f32,
    amount: f32,
    edelta: f32,
    efficiency_scale: f32,
    multiplier: f32,
) -> f32 {
    let ed = edelta * efficiency_scale;
    if ed <= 0.00000001 {
        0.0
    } else {
        (stored / (amount * ed * multiplier)).min(1.0)
    }
}

pub fn consume_liquids_efficiency(
    stacks: &[LiquidStackSpec],
    edelta: f32,
    efficiency_scale: f32,
    multiplier: f32,
    get_liquid: impl Fn(i16) -> f32,
) -> f32 {
    let ed = edelta * efficiency_scale;
    if ed <= 0.00000001 {
        return 0.0;
    }
    stacks.iter().fold(1.0, |min, stack| {
        min.min(get_liquid(stack.liquid_id) / (stack.amount * ed * multiplier))
    })
}

pub fn consume_liquid_filter_get_consumed(
    current: Option<i16>,
    current_amount: f32,
    liquid_ids_in_content_order: &[i16],
    get_liquid: impl Fn(i16) -> f32,
    filter: impl Fn(i16) -> bool,
) -> Option<i16> {
    if let Some(id) = current {
        if filter(id) && current_amount > 0.0 {
            return Some(id);
        }
    }
    liquid_ids_in_content_order
        .iter()
        .copied()
        .find(|liquid_id| filter(*liquid_id) && get_liquid(*liquid_id) > 0.0)
}

pub fn consume_liquid_filter_efficiency(
    consumed: Option<i16>,
    stored: f32,
    amount: f32,
    edelta: f32,
    multiplier: f32,
) -> f32 {
    if edelta <= 0.00000001 {
        0.0
    } else if consumed.is_some() {
        (stored / (amount * edelta * multiplier)).min(1.0)
    } else {
        0.0
    }
}

pub fn consume_liquid_filter_efficiency_multiplier(
    consumed: Option<i16>,
    liquid_multiplier: impl Fn(i16) -> f32,
) -> f32 {
    consumed.map(liquid_multiplier).unwrap_or(0.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidProperties {
    pub coolant: bool,
    pub gas: bool,
    pub temperature: f32,
    pub flammability: f32,
}

pub fn consume_coolant_filter(
    liquid: LiquidProperties,
    allow_liquid: bool,
    allow_gas: bool,
    max_temp: f32,
    max_flammability: f32,
) -> bool {
    liquid.coolant
        && ((allow_liquid && !liquid.gas) || (allow_gas && liquid.gas))
        && liquid.temperature <= max_temp
        && liquid.flammability < max_flammability
}

pub fn consume_liquid_flammable_filter(liquid: LiquidProperties, min_flammability: f32) -> bool {
    liquid.flammability >= min_flammability
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayloadStackSpec {
    pub content_id: i16,
    pub amount: i32,
}

pub fn consume_item_dynamic_efficiency(
    stacks: &[ItemStackSpec],
    multiplier: f32,
    trigger_valid: bool,
    has_item: impl Fn(i16, i32) -> bool,
) -> f32 {
    consume_items_efficiency(stacks, multiplier, trigger_valid, has_item)
}

pub fn consume_item_dynamic_trigger_amounts(
    stacks: &[ItemStackSpec],
    multiplier: f32,
) -> Vec<(i16, i32)> {
    stacks
        .iter()
        .map(|stack| {
            (
                stack.item_id,
                consume_items_trigger_amount(stack.amount, multiplier),
            )
        })
        .collect()
}

pub fn consume_liquids_dynamic_update_amounts(
    stacks: &[LiquidStackSpec],
    edelta: f32,
    multiplier: f32,
) -> Vec<(i16, f32)> {
    stacks
        .iter()
        .map(|stack| (stack.liquid_id, stack.amount * edelta * multiplier))
        .collect()
}

pub fn consume_liquids_dynamic_efficiency(
    stacks: &[LiquidStackSpec],
    edelta: f32,
    multiplier: f32,
    get_liquid: impl Fn(i16) -> f32,
) -> f32 {
    if edelta <= 0.00000001 {
        return 0.0;
    }
    stacks.iter().fold(1.0, |min, stack| {
        min.min(get_liquid(stack.liquid_id) / (stack.amount * edelta * multiplier))
    })
}

pub fn consume_payloads_efficiency(
    stacks: &[PayloadStackSpec],
    multiplier: f32,
    contains_payload: impl Fn(i16, i32) -> bool,
) -> f32 {
    if stacks.iter().all(|stack| {
        contains_payload(
            stack.content_id,
            (stack.amount as f32 * multiplier).round() as i32,
        )
    }) {
        1.0
    } else {
        0.0
    }
}

pub fn consume_payloads_trigger_amounts(
    stacks: &[PayloadStackSpec],
    multiplier: f32,
) -> Vec<(i16, i32)> {
    stacks
        .iter()
        .map(|stack| {
            (
                stack.content_id,
                (stack.amount as f32 * multiplier).round() as i32,
            )
        })
        .collect()
}

pub fn consume_payload_filter_efficiency(
    fitting_content_ids: &[i16],
    contains_payload: impl Fn(i16, i32) -> bool,
) -> f32 {
    if fitting_content_ids
        .iter()
        .any(|content_id| contains_payload(*content_id, 1))
    {
        1.0
    } else {
        0.0
    }
}

pub fn consume_payload_filter_get_consumed(
    fitting_content_ids: &[i16],
    contains_payload: impl Fn(i16, i32) -> bool,
) -> Option<i16> {
    fitting_content_ids
        .iter()
        .copied()
        .find(|content_id| contains_payload(*content_id, 1))
}

pub fn consume_power_dynamic_requested(usage: f32) -> f32 {
    usage
}

pub fn consume_power_dynamic_display_per_second(displayed_power_usage: f32) -> Option<f32> {
    if displayed_power_usage != 0.0 {
        Some(displayed_power_usage * 60.0)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_base_and_power_follow_upstream_contracts() {
        let flags = consume_boost(ConsumeFlags::default());
        assert!(flags.optional);
        assert!(flags.booster);
        assert!(!consume_update(flags, false).update);

        let buffered = ConsumePowerSpec::new(2.0, 10.0, true);
        assert!(consume_power_ignore(buffered.buffered));
        assert_eq!(consume_power_efficiency(0.75), 0.75);
        assert_eq!(consume_power_requested_power(buffered, 0.25, false), 7.5);
        let unbuffered = ConsumePowerSpec::new(2.0, 0.0, false);
        assert_eq!(consume_power_requested_power(unbuffered, 0.0, true), 2.0);
        assert_eq!(consume_power_requested_power(unbuffered, 0.0, false), 0.0);
        assert_eq!(consume_power_condition_requested(3.0, false), 0.0);
    }

    #[test]
    fn item_consumers_follow_java_filter_and_multiplier_rules() {
        let stacks = [
            ItemStackSpec {
                item_id: 1,
                amount: 2,
            },
            ItemStackSpec {
                item_id: 2,
                amount: 3,
            },
        ];
        assert_eq!(consume_items_trigger_amount(3, 1.5), 5);
        assert_eq!(
            consume_items_efficiency(&stacks, 1.0, false, |id, amount| id == 1 && amount <= 2
                || id == 2 && amount <= 3),
            1.0
        );
        assert_eq!(
            consume_item_filter_get_consumed(&[0, 1, 2], |id| id == 2, |id| id > 1),
            Some(2)
        );
        assert_eq!(consume_item_filter_efficiency(Some(2), false), 1.0);
        assert_eq!(
            consume_item_filter_efficiency_multiplier(Some(2), |id| id as f32 * 0.5),
            1.0
        );
        let item = ItemProperties {
            flammability: 0.3,
            explosiveness: 0.4,
            radioactivity: 0.1,
            charge: 0.5,
        };
        assert!(consume_item_flammable_filter(item, 0.2));
        assert!(consume_item_explosive_filter(item, 0.2));
        assert!(!consume_item_radioactive_filter(item, 0.2));
        assert!(consume_item_charged_filter(item, 0.2));
        assert_eq!(
            consume_item_explode_chance(true, 2.0, 0.06, 0.75, 0.5),
            0.03
        );
        assert!(consume_item_explode_should_damage(0.5, 0.49));
    }

    #[test]
    fn liquid_consumers_follow_java_efficiency_and_selection_rules() {
        assert_eq!(consume_liquid_update_amount(2.0, 3.0, 0.5), 3.0);
        assert_eq!(consume_liquid_efficiency(5.0, 2.0, 2.0, 1.0, 1.0), 1.0);
        assert_eq!(consume_liquid_efficiency(1.0, 2.0, 2.0, 1.0, 1.0), 0.25);
        let stacks = [
            LiquidStackSpec {
                liquid_id: 1,
                amount: 2.0,
            },
            LiquidStackSpec {
                liquid_id: 2,
                amount: 4.0,
            },
        ];
        assert_eq!(
            consume_liquids_efficiency(&stacks, 1.0, 1.0, 1.0, |id| if id == 1 {
                1.0
            } else {
                8.0
            }),
            0.5
        );
        assert_eq!(
            consume_liquid_filter_get_consumed(Some(2), 1.0, &[1, 2], |_| 0.0, |id| id == 2),
            Some(2)
        );
        assert_eq!(
            consume_liquid_filter_get_consumed(
                None,
                0.0,
                &[1, 2],
                |id| if id == 1 { 0.0 } else { 1.0 },
                |id| id == 2
            ),
            Some(2)
        );
        assert_eq!(
            consume_liquid_filter_efficiency(Some(2), 1.0, 2.0, 1.0, 1.0),
            0.5
        );
        assert_eq!(
            consume_liquid_filter_efficiency_multiplier(Some(2), |id| id as f32),
            2.0
        );

        let water = LiquidProperties {
            coolant: true,
            gas: false,
            temperature: 0.4,
            flammability: 0.0,
        };
        let gas = LiquidProperties { gas: true, ..water };
        assert!(consume_coolant_filter(water, true, false, 0.5, 0.1));
        assert!(!consume_coolant_filter(gas, true, false, 0.5, 0.1));
        assert!(consume_coolant_filter(gas, false, true, 0.5, 0.1));
        assert!(consume_liquid_flammable_filter(
            LiquidProperties {
                flammability: 0.3,
                ..water
            },
            0.2
        ));
    }

    #[test]
    fn dynamic_payload_and_power_consumers_follow_upstream_rules() {
        let item_stacks = [ItemStackSpec {
            item_id: 3,
            amount: 4,
        }];
        assert_eq!(
            consume_item_dynamic_trigger_amounts(&item_stacks, 1.25),
            vec![(3, 5)]
        );
        assert_eq!(
            consume_item_dynamic_efficiency(&item_stacks, 1.25, false, |id, amount| {
                id == 3 && amount == 5
            }),
            1.0
        );

        let liquid_stacks = [LiquidStackSpec {
            liquid_id: 2,
            amount: 3.0,
        }];
        assert_eq!(
            consume_liquids_dynamic_update_amounts(&liquid_stacks, 2.0, 0.5),
            vec![(2, 3.0)]
        );
        assert_eq!(
            consume_liquids_dynamic_efficiency(&liquid_stacks, 2.0, 0.5, |id| if id == 2 {
                1.5
            } else {
                0.0
            }),
            0.5
        );

        let payloads = [
            PayloadStackSpec {
                content_id: 10,
                amount: 2,
            },
            PayloadStackSpec {
                content_id: 11,
                amount: 1,
            },
        ];
        assert_eq!(
            consume_payloads_efficiency(&payloads, 1.5, |id, amount| {
                (id == 10 && amount == 3) || (id == 11 && amount == 2)
            }),
            1.0
        );
        assert_eq!(
            consume_payloads_trigger_amounts(&payloads, 1.5),
            vec![(10, 3), (11, 2)]
        );
        assert_eq!(
            consume_payload_filter_get_consumed(&[5, 6, 7], |id, amount| id == 6 && amount == 1),
            Some(6)
        );
        assert_eq!(
            consume_payload_filter_efficiency(&[5, 6, 7], |id, _| id == 4),
            0.0
        );

        assert_eq!(consume_power_dynamic_requested(12.0), 12.0);
        assert_eq!(consume_power_dynamic_display_per_second(2.0), Some(120.0));
        assert_eq!(consume_power_dynamic_display_per_second(0.0), None);
    }
}
