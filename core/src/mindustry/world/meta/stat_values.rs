use super::{Attribute, StatUnit, StatValue};

pub struct StatValues;

impl StatValues {
    pub fn string(value: impl Into<String>) -> StatValue {
        StatValue::Text(value.into())
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

    pub fn liquid(name: impl Into<String>, amount: f32, per_second: bool) -> StatValue {
        StatValue::Liquid {
            name: name.into(),
            amount,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_values_constructors_match_java_helper_intent() {
        assert_eq!(StatValues::string("hello"), StatValue::Text("hello".into()));
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
        assert_eq!(StatValues::liquid("water", 1.5, true).kind(), "liquid");
        let attr = Attribute::new(0, "heat");
        assert_eq!(
            StatValues::blocks(&attr, false, 1.0, false).kind(),
            "blocks"
        );
    }
}
