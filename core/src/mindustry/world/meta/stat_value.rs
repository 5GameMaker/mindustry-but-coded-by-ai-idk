use super::StatUnit;

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
    Liquid {
        name: String,
        amount: f32,
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
            StatValue::Liquid { .. } => "liquid",
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
            StatValue::Liquid {
                name,
                amount,
                per_second,
            } => vec![format!(
                "liquid:{name}:{}:{}",
                fix_value(*amount),
                if *per_second { "perSecond" } else { "raw" }
            )],
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
    }
}
