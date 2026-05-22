use super::{Attribute, StatUnit, StatValue};
use crate::mindustry::r#type::{ItemStack, LiquidStack};

pub struct StatValues;

impl StatValues {
    pub fn string(value: impl Into<String>) -> StatValue {
        StatValue::Text(value.into())
    }

    pub fn string_args(
        value: impl AsRef<str>,
        args: impl IntoIterator<Item = impl ToString>,
    ) -> StatValue {
        StatValue::Text(format_at_placeholders(value.as_ref(), args))
    }

    pub fn bool(value: bool) -> StatValue {
        StatValue::Bool(value)
    }

    pub fn squared(value: f32, unit: StatUnit) -> StatValue {
        StatValue::Squared { value, unit }
    }

    pub fn number(value: f32, unit: StatUnit) -> StatValue {
        Self::number_merge(value, unit, false)
    }

    pub fn number_merge(value: f32, unit: StatUnit, merge: bool) -> StatValue {
        StatValue::Number { value, unit, merge }
    }

    pub fn multiplier_modifier(value: f32) -> StatValue {
        Self::multiplier_modifier_unit(value, StatUnit::Multiplier, true)
    }

    pub fn multiplier_modifier_unit(value: f32, unit: StatUnit, merge: bool) -> StatValue {
        StatValue::MultiplierModifier { value, unit, merge }
    }

    pub fn percent_modifier(value: f32) -> StatValue {
        Self::percent_modifier_unit(value, StatUnit::Percent, true)
    }

    pub fn percent_modifier_unit(value: f32, unit: StatUnit, merge: bool) -> StatValue {
        StatValue::PercentModifier { value, unit, merge }
    }

    pub fn item(name: impl Into<String>, amount: i32) -> StatValue {
        StatValue::Item {
            name: name.into(),
            amount,
            display_name: true,
        }
    }

    pub fn item_with_display(
        name: impl Into<String>,
        amount: i32,
        display_name: bool,
    ) -> StatValue {
        StatValue::Item {
            name: name.into(),
            amount,
            display_name,
        }
    }

    pub fn items(stacks: Vec<ItemStack>) -> StatValue {
        Self::items_with_display(true, stacks)
    }

    pub fn items_with_display(display_name: bool, stacks: Vec<ItemStack>) -> StatValue {
        StatValue::Items {
            stacks,
            display_name,
            time_period: None,
        }
    }

    pub fn items_per_time(time_period: f32, stacks: Vec<ItemStack>) -> StatValue {
        StatValue::Items {
            stacks,
            display_name: true,
            time_period: Some(time_period),
        }
    }

    pub fn liquid(name: impl Into<String>, amount: f32, per_second: bool) -> StatValue {
        StatValue::Liquid {
            name: name.into(),
            amount,
            per_second,
        }
    }

    pub fn liquids(time_period: f32, stacks: Vec<LiquidStack>) -> StatValue {
        Self::liquids_with(time_period, true, stacks)
    }

    pub fn liquids_with(time_period: f32, per_second: bool, stacks: Vec<LiquidStack>) -> StatValue {
        StatValue::Liquids {
            stacks,
            time_period,
            per_second,
        }
    }

    pub fn blocks(
        attribute: &Attribute,
        floating: bool,
        scale: f32,
        start_zero: bool,
    ) -> StatValue {
        StatValue::Blocks {
            attribute: attribute.name.clone(),
            floating,
            scale,
            start_zero,
        }
    }
}

fn format_at_placeholders(value: &str, args: impl IntoIterator<Item = impl ToString>) -> String {
    let mut result = value.to_string();
    for arg in args {
        if let Some(index) = result.find('@') {
            result.replace_range(index..index + 1, &arg.to_string());
        } else {
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_values_constructors_match_java_helper_intent() {
        assert_eq!(StatValues::string("hello"), StatValue::Text("hello".into()));
        assert_eq!(
            StatValues::string_args("hello @ @", ["copper", "lead"]),
            StatValue::Text("hello copper lead".into())
        );
        assert_eq!(StatValues::bool(true), StatValue::Bool(true));
        assert_eq!(
            StatValues::number(2.0, StatUnit::Seconds),
            StatValue::Number {
                value: 2.0,
                unit: StatUnit::Seconds,
                merge: false
            }
        );
        assert_eq!(
            StatValues::multiplier_modifier(1.2).kind(),
            "multiplierModifier"
        );
        assert_eq!(StatValues::percent_modifier(0.8).kind(), "percentModifier");
        assert_eq!(StatValues::item("copper", 3).kind(), "item");
        assert_eq!(
            StatValues::items(vec![ItemStack::new("copper", 3), ItemStack::new("lead", 4)])
                .display_tokens(),
            vec!["item:copper:3:name", "item:lead:4:name"]
        );
        assert_eq!(
            StatValues::items_with_display(false, vec![ItemStack::new("graphite", 5)])
                .display_tokens(),
            vec!["item:graphite:5:icon"]
        );
        assert_eq!(
            StatValues::items_per_time(120.0, vec![ItemStack::new("silicon", 30)]).display_tokens(),
            vec!["item:silicon:30:perSecond:15:name"]
        );
        assert_eq!(StatValues::liquid("water", 1.5, true).kind(), "liquid");
        assert_eq!(
            StatValues::liquids(
                30.0,
                vec![
                    LiquidStack::new("water", 1.5),
                    LiquidStack::new("slag", 2.0)
                ]
            )
            .display_tokens(),
            vec!["liquid:water:3:perSecond", "liquid:slag:4:perSecond"]
        );
        assert_eq!(
            StatValues::liquids_with(60.0, false, vec![LiquidStack::new("oil", 2.25)])
                .display_tokens(),
            vec!["liquid:oil:2.25:raw"]
        );
        let attr = Attribute::new(0, "heat");
        assert_eq!(
            StatValues::blocks(&attr, false, 1.0, false).kind(),
            "blocks"
        );
    }

    #[test]
    fn stat_values_string_args_replace_at_markers_like_arc_strings_format() {
        assert_eq!(format_at_placeholders("@x @ @", ["A", "B", "C"]), "Ax B C");
        assert_eq!(format_at_placeholders("no args", ["ignored"]), "no args");
        assert_eq!(format_at_placeholders("@ @ @", ["one"]), "one @ @");
    }
}
