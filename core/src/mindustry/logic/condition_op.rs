//! Mirrors upstream `mindustry.logic.ConditionOp`.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConditionValue<'a> {
    Number(f64),
    Object(&'a str),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConditionOp {
    Equal,
    NotEqual,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    StrictEqual,
    Always,
}

impl ConditionOp {
    pub const ALL: [ConditionOp; 8] = [
        ConditionOp::Equal,
        ConditionOp::NotEqual,
        ConditionOp::LessThan,
        ConditionOp::LessThanEq,
        ConditionOp::GreaterThan,
        ConditionOp::GreaterThanEq,
        ConditionOp::StrictEqual,
        ConditionOp::Always,
    ];

    pub const SYMBOLS: [&'static str; 8] = ["==", "not", "<", "<=", ">", ">=", "===", "always"];

    pub const JAVA_NAMES: [&'static str; 8] = [
        "equal",
        "notEqual",
        "lessThan",
        "lessThanEq",
        "greaterThan",
        "greaterThanEq",
        "strictEqual",
        "always",
    ];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn symbol(self) -> &'static str {
        Self::SYMBOLS[self.ordinal() as usize]
    }

    pub fn java_name(self) -> &'static str {
        Self::JAVA_NAMES[self.ordinal() as usize]
    }

    pub fn by_java_name(name: &str) -> Option<Self> {
        match name {
            "equal" => Some(ConditionOp::Equal),
            "notEqual" => Some(ConditionOp::NotEqual),
            "lessThan" => Some(ConditionOp::LessThan),
            "lessThanEq" => Some(ConditionOp::LessThanEq),
            "greaterThan" => Some(ConditionOp::GreaterThan),
            "greaterThanEq" => Some(ConditionOp::GreaterThanEq),
            "strictEqual" => Some(ConditionOp::StrictEqual),
            "always" => Some(ConditionOp::Always),
            _ => None,
        }
    }

    pub fn test_numbers(self, a: f64, b: f64) -> bool {
        match self {
            ConditionOp::Equal => (a - b).abs() < 0.000001,
            ConditionOp::NotEqual => (a - b).abs() >= 0.000001,
            ConditionOp::LessThan => a < b,
            ConditionOp::LessThanEq => a <= b,
            ConditionOp::GreaterThan => a > b,
            ConditionOp::GreaterThanEq => a >= b,
            ConditionOp::StrictEqual => a == b,
            ConditionOp::Always => true,
        }
    }

    pub fn test_values(self, a: ConditionValue<'_>, b: ConditionValue<'_>) -> bool {
        match self {
            ConditionOp::StrictEqual => a == b,
            ConditionOp::Equal => match (a, b) {
                (ConditionValue::Object(a), ConditionValue::Object(b)) => a == b,
                (ConditionValue::Number(a), ConditionValue::Number(b)) => self.test_numbers(a, b),
                _ => false,
            },
            ConditionOp::NotEqual => match (a, b) {
                (ConditionValue::Object(a), ConditionValue::Object(b)) => a != b,
                (ConditionValue::Number(a), ConditionValue::Number(b)) => self.test_numbers(a, b),
                _ => true,
            },
            ConditionOp::Always => true,
            _ => match (a, b) {
                (ConditionValue::Number(a), ConditionValue::Number(b)) => self.test_numbers(a, b),
                _ => false,
            },
        }
    }
}

impl fmt::Display for ConditionOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}

#[cfg(test)]
mod tests {
    use super::{ConditionOp, ConditionValue};

    #[test]
    fn condition_op_order_symbols_and_java_names_match_java_enum() {
        assert_eq!(
            ConditionOp::ALL,
            [
                ConditionOp::Equal,
                ConditionOp::NotEqual,
                ConditionOp::LessThan,
                ConditionOp::LessThanEq,
                ConditionOp::GreaterThan,
                ConditionOp::GreaterThanEq,
                ConditionOp::StrictEqual,
                ConditionOp::Always,
            ]
        );
        assert_eq!(
            ConditionOp::SYMBOLS,
            ["==", "not", "<", "<=", ">", ">=", "===", "always"]
        );
        assert_eq!(ConditionOp::LessThanEq.ordinal(), 3);
        assert_eq!(ConditionOp::from_ordinal(7), Some(ConditionOp::Always));
        assert_eq!(ConditionOp::from_ordinal(8), None);
        assert_eq!(
            ConditionOp::by_java_name("greaterThanEq"),
            Some(ConditionOp::GreaterThanEq)
        );
        assert_eq!(ConditionOp::GreaterThan.to_string(), ">");
    }

    #[test]
    fn condition_op_number_and_object_tests_follow_java_lvar_semantics() {
        assert!(ConditionOp::Equal.test_numbers(1.0, 1.0 + 0.0000005));
        assert!(ConditionOp::NotEqual.test_numbers(1.0, 1.0 + 0.000002));
        assert!(ConditionOp::LessThan.test_numbers(1.0, 2.0));
        assert!(ConditionOp::LessThanEq.test_numbers(2.0, 2.0));
        assert!(ConditionOp::GreaterThan.test_numbers(3.0, 2.0));
        assert!(ConditionOp::GreaterThanEq.test_numbers(2.0, 2.0));
        assert!(ConditionOp::Always.test_numbers(f64::NAN, f64::NAN));

        assert!(ConditionOp::StrictEqual.test_values(
            ConditionValue::Object("core"),
            ConditionValue::Object("core")
        ));
        assert!(!ConditionOp::StrictEqual
            .test_values(ConditionValue::Number(2.0), ConditionValue::Object("2")));
        assert!(ConditionOp::Equal.test_values(
            ConditionValue::Object("core"),
            ConditionValue::Object("core")
        ));
        assert!(ConditionOp::NotEqual.test_values(
            ConditionValue::Object("core"),
            ConditionValue::Object("vault")
        ));
        assert!(!ConditionOp::LessThan.test_values(
            ConditionValue::Object("core"),
            ConditionValue::Object("vault")
        ));
    }
}
