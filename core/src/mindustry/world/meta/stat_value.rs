use super::StatUnit;
use crate::mindustry::r#type::{ItemStack, LiquidStack};

#[derive(Debug, Clone, PartialEq)]
pub enum StatValue {
    Text(String),
    Bool(bool),
    Number {
        value: f32,
        unit: StatUnit,
        merge: bool,
    },
    Squared {
        value: f32,
        unit: StatUnit,
    },
    MultiplierModifier {
        value: f32,
        unit: StatUnit,
        merge: bool,
    },
    PercentModifier {
        value: f32,
        unit: StatUnit,
        merge: bool,
    },
    Item {
        name: String,
        amount: i32,
        display_name: bool,
    },
    Items {
        stacks: Vec<ItemStack>,
        display_name: bool,
        time_period: Option<f32>,
    },
    FilteredItems {
        stacks: Vec<ItemStack>,
        time_period: Option<f32>,
    },
    Liquid {
        name: String,
        amount: f32,
        per_second: bool,
    },
    Liquids {
        stacks: Vec<LiquidStack>,
        time_period: f32,
        per_second: bool,
    },
    FilteredLiquids {
        stacks: Vec<LiquidStack>,
        per_second: bool,
    },
    Blocks {
        attribute: String,
        floating: bool,
        scale: f32,
        start_zero: bool,
    },
}

impl StatValue {
    pub fn kind(&self) -> &'static str {
        match self {
            StatValue::Text(_) => "string",
            StatValue::Bool(_) => "bool",
            StatValue::Number { .. } => "number",
            StatValue::Squared { .. } => "squared",
            StatValue::MultiplierModifier { .. } => "multiplierModifier",
            StatValue::PercentModifier { .. } => "percentModifier",
            StatValue::Item { .. } => "item",
            StatValue::Items { .. } => "items",
            StatValue::FilteredItems { .. } => "items",
            StatValue::Liquid { .. } => "liquid",
            StatValue::Liquids { .. } => "liquids",
            StatValue::FilteredLiquids { .. } => "liquids",
            StatValue::Blocks { .. } => "blocks",
        }
    }

    pub fn display_tokens(&self) -> Vec<String> {
        match self {
            StatValue::Text(text) => vec![text.clone()],
            StatValue::Bool(value) => vec![if *value { "@yes" } else { "@no" }.to_string()],
            StatValue::Number { value, unit, merge } => {
                let left = format!(
                    "{}{}",
                    unit.icon()
                        .map(|icon| format!("{icon} "))
                        .unwrap_or_default(),
                    fix_value(*value)
                );
                let right = unit
                    .bundle_key()
                    .map(|key| format!("{}@{}", if unit.space() { " " } else { "" }, key))
                    .unwrap_or_default();
                if *merge {
                    vec![format!("{left}{right}")]
                } else {
                    vec![left, right]
                }
            }
            StatValue::Squared { value, unit } => vec![
                format!("{}x{}", fix_value(*value), fix_value(*value)),
                unit.bundle_key()
                    .map(|key| format!("{}@{}", if unit.space() { " " } else { "" }, key))
                    .unwrap_or_default(),
            ],
            StatValue::MultiplierModifier { value, unit, merge } => {
                let left = format!(
                    "{}{}",
                    unit.icon()
                        .map(|icon| format!("{icon} "))
                        .unwrap_or_default(),
                    mult_stat(*value)
                );
                let right = unit
                    .bundle_key()
                    .map(|key| format!("{}@{}", if unit.space() { " " } else { "" }, key))
                    .unwrap_or_default();
                if *merge {
                    vec![format!("{left}{right}")]
                } else {
                    vec![left, right]
                }
            }
            StatValue::PercentModifier { value, unit, merge } => {
                let left = format!(
                    "{}{}",
                    unit.icon()
                        .map(|icon| format!("{icon} "))
                        .unwrap_or_default(),
                    ammo_stat((*value - 1.0) * 100.0)
                );
                let right = unit
                    .bundle_key()
                    .map(|key| format!("{}@{}", if unit.space() { " " } else { "" }, key))
                    .unwrap_or_default();
                if *merge {
                    vec![format!("{left}{right}")]
                } else {
                    vec![left, right]
                }
            }
            StatValue::Item {
                name,
                amount,
                display_name,
            } => vec![format!(
                "item:{name}:{}:{}",
                amount,
                if *display_name { "name" } else { "icon" }
            )],
            StatValue::Items {
                stacks,
                display_name,
                time_period,
            } => stacks
                .iter()
                .map(|stack| {
                    if let Some(time_period) = time_period {
                        format!(
                            "item:{}:{}:perSecond:{}:{}",
                            stack.item,
                            stack.amount,
                            fix_value(stack.amount as f32 / (*time_period / 60.0)),
                            if *display_name { "name" } else { "icon" }
                        )
                    } else {
                        format!(
                            "item:{}:{}:{}",
                            stack.item,
                            stack.amount,
                            if *display_name { "name" } else { "icon" }
                        )
                    }
                })
                .collect(),
            StatValue::FilteredItems {
                stacks,
                time_period,
            } => {
                let mut tokens = Vec::new();
                for (index, stack) in stacks.iter().enumerate() {
                    if let Some(time_period) = time_period {
                        tokens.push(format!(
                            "item:{}:{}:perSecond:{}:name",
                            stack.item,
                            stack.amount,
                            fix_value(stack.amount as f32 / (*time_period / 60.0))
                        ));
                    } else {
                        tokens.push(format!("item:{}:{}:name", stack.item, stack.amount));
                    }
                    if index != stacks.len() - 1 {
                        tokens.push("/".to_string());
                    }
                }
                tokens
            }
            StatValue::Liquid {
                name,
                amount,
                per_second,
            } => vec![format!(
                "liquid:{name}:{}:{}",
                fix_value(*amount),
                if *per_second { "perSecond" } else { "raw" }
            )],
            StatValue::Liquids {
                stacks,
                time_period,
                per_second,
            } => stacks
                .iter()
                .map(|stack| {
                    format!(
                        "liquid:{}:{}:{}",
                        stack.liquid,
                        fix_value(stack.amount * (60.0 / *time_period)),
                        if *per_second { "perSecond" } else { "raw" }
                    )
                })
                .collect(),
            StatValue::FilteredLiquids { stacks, per_second } => {
                let mut tokens = Vec::new();
                for (index, stack) in stacks.iter().enumerate() {
                    tokens.push(format!(
                        "liquid:{}:{}:{}",
                        stack.liquid,
                        fix_value(stack.amount),
                        if *per_second { "perSecond" } else { "raw" }
                    ));
                    if index != stacks.len() - 1 {
                        tokens.push("/".to_string());
                    }
                }
                tokens
            }
            StatValue::Blocks {
                attribute,
                floating,
                scale,
                start_zero,
            } => vec![format!(
                "blocks:{attribute}:{}:{}:{}",
                floating,
                fix_value(*scale),
                start_zero
            )],
        }
    }
}

pub fn fix_value(value: f32) -> String {
    let mut out = format!("{value:.3}");
    while out.contains('.') && out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    if out == "-0" {
        "0".to_string()
    } else {
        out
    }
}

pub fn ammo_stat(value: f32) -> String {
    format!(
        "{}{}",
        if value > 0.0 { "[stat]+" } else { "[negstat]" },
        fix_decimals(value, 1)
    )
}

pub fn mult_stat(value: f32) -> String {
    format!(
        "{}{}",
        if value >= 1.0 { "[stat]" } else { "[negstat]" },
        fix_decimals(value, 2)
    )
}

fn fix_decimals(value: f32, decimals: usize) -> String {
    let mut out = format!("{value:.decimals$}");
    while out.contains('.') && out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    if out == "-0" {
        "0".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_values_render_lightweight_tokens_like_java_helpers() {
        assert_eq!(fix_value(1.2301), "1.23");
        assert_eq!(fix_value(1.0), "1");
        assert_eq!(ammo_stat(12.0), "[stat]+12");
        assert_eq!(ammo_stat(-2.5), "[negstat]-2.5");
        assert_eq!(mult_stat(1.25), "[stat]1.25");
        assert_eq!(mult_stat(0.5), "[negstat]0.5");
        assert_eq!(StatValue::Bool(true).display_tokens(), vec!["@yes"]);
        assert_eq!(StatValue::Bool(false).display_tokens(), vec!["@no"]);
        assert_eq!(
            StatValue::Number {
                value: 4.5,
                unit: StatUnit::Seconds,
                merge: false
            }
            .display_tokens(),
            vec!["4.5", " @unit.seconds"]
        );
        assert_eq!(
            StatValue::Number {
                value: 8.0,
                unit: StatUnit::Percent,
                merge: true
            }
            .display_tokens(),
            vec!["8@unit.percent"]
        );
        assert_eq!(
            StatValue::Items {
                stacks: vec![ItemStack::new("copper", 2), ItemStack::new("lead", 3)],
                display_name: false,
                time_period: None
            }
            .display_tokens(),
            vec!["item:copper:2:icon", "item:lead:3:icon"]
        );
        assert_eq!(
            StatValue::FilteredItems {
                stacks: vec![ItemStack::new("copper", 0), ItemStack::new("lead", 0)],
                time_period: None
            }
            .display_tokens(),
            vec!["item:copper:0:name", "/", "item:lead:0:name"]
        );
        assert_eq!(
            StatValue::Liquids {
                stacks: vec![LiquidStack::new("water", 30.0)],
                time_period: 120.0,
                per_second: true
            }
            .display_tokens(),
            vec!["liquid:water:15:perSecond"]
        );
        assert_eq!(
            StatValue::FilteredLiquids {
                stacks: vec![LiquidStack::new("water", 1.5), LiquidStack::new("oil", 0.5)],
                per_second: true
            }
            .display_tokens(),
            vec![
                "liquid:water:1.5:perSecond",
                "/",
                "liquid:oil:0.5:perSecond"
            ]
        );
    }
}
