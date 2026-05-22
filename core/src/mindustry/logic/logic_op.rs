//! Mirrors upstream `mindustry.logic.LogicOp`.

use std::fmt;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicOp {
    Add,
    Sub,
    Mul,
    Div,
    Idiv,
    Mod,
    Emod,
    Pow,
    Equal,
    NotEqual,
    Land,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    StrictEqual,
    Shl,
    Shr,
    Ushr,
    Or,
    And,
    Xor,
    Not,
    Max,
    Min,
    Angle,
    AngleDiff,
    Len,
    Noise,
    Abs,
    Sign,
    Log,
    Logn,
    Log10,
    Floor,
    Ceil,
    Round,
    Sqrt,
    Rand,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
}

impl LogicOp {
    pub const ALL: [LogicOp; 45] = [
        LogicOp::Add,
        LogicOp::Sub,
        LogicOp::Mul,
        LogicOp::Div,
        LogicOp::Idiv,
        LogicOp::Mod,
        LogicOp::Emod,
        LogicOp::Pow,
        LogicOp::Equal,
        LogicOp::NotEqual,
        LogicOp::Land,
        LogicOp::LessThan,
        LogicOp::LessThanEq,
        LogicOp::GreaterThan,
        LogicOp::GreaterThanEq,
        LogicOp::StrictEqual,
        LogicOp::Shl,
        LogicOp::Shr,
        LogicOp::Ushr,
        LogicOp::Or,
        LogicOp::And,
        LogicOp::Xor,
        LogicOp::Not,
        LogicOp::Max,
        LogicOp::Min,
        LogicOp::Angle,
        LogicOp::AngleDiff,
        LogicOp::Len,
        LogicOp::Noise,
        LogicOp::Abs,
        LogicOp::Sign,
        LogicOp::Log,
        LogicOp::Logn,
        LogicOp::Log10,
        LogicOp::Floor,
        LogicOp::Ceil,
        LogicOp::Round,
        LogicOp::Sqrt,
        LogicOp::Rand,
        LogicOp::Sin,
        LogicOp::Cos,
        LogicOp::Tan,
        LogicOp::Asin,
        LogicOp::Acos,
        LogicOp::Atan,
    ];

    pub const SYMBOLS: [&'static str; 45] = [
        "+",
        "-",
        "*",
        "/",
        "//",
        "%",
        "%%",
        "^",
        "==",
        "not",
        "and",
        "<",
        "<=",
        ">",
        ">=",
        "===",
        "<<",
        ">>",
        ">>>",
        "or",
        "b-and",
        "xor",
        "flip",
        "max",
        "min",
        "angle",
        "anglediff",
        "len",
        "noise",
        "abs",
        "sign",
        "log",
        "logn",
        "log10",
        "floor",
        "ceil",
        "round",
        "sqrt",
        "rand",
        "sin",
        "cos",
        "tan",
        "asin",
        "acos",
        "atan",
    ];

    pub const JAVA_NAMES: [&'static str; 45] = [
        "add",
        "sub",
        "mul",
        "div",
        "idiv",
        "mod",
        "emod",
        "pow",
        "equal",
        "notEqual",
        "land",
        "lessThan",
        "lessThanEq",
        "greaterThan",
        "greaterThanEq",
        "strictEqual",
        "shl",
        "shr",
        "ushr",
        "or",
        "and",
        "xor",
        "not",
        "max",
        "min",
        "angle",
        "angleDiff",
        "len",
        "noise",
        "abs",
        "sign",
        "log",
        "logn",
        "log10",
        "floor",
        "ceil",
        "round",
        "sqrt",
        "rand",
        "sin",
        "cos",
        "tan",
        "asin",
        "acos",
        "atan",
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
            "add" => Some(LogicOp::Add),
            "sub" => Some(LogicOp::Sub),
            "mul" => Some(LogicOp::Mul),
            "div" => Some(LogicOp::Div),
            "idiv" => Some(LogicOp::Idiv),
            "mod" => Some(LogicOp::Mod),
            "emod" => Some(LogicOp::Emod),
            "pow" => Some(LogicOp::Pow),
            "equal" => Some(LogicOp::Equal),
            "notEqual" => Some(LogicOp::NotEqual),
            "land" => Some(LogicOp::Land),
            "lessThan" => Some(LogicOp::LessThan),
            "lessThanEq" => Some(LogicOp::LessThanEq),
            "greaterThan" => Some(LogicOp::GreaterThan),
            "greaterThanEq" => Some(LogicOp::GreaterThanEq),
            "strictEqual" => Some(LogicOp::StrictEqual),
            "shl" => Some(LogicOp::Shl),
            "shr" => Some(LogicOp::Shr),
            "ushr" => Some(LogicOp::Ushr),
            "or" => Some(LogicOp::Or),
            "and" => Some(LogicOp::And),
            "xor" => Some(LogicOp::Xor),
            "not" => Some(LogicOp::Not),
            "max" => Some(LogicOp::Max),
            "min" => Some(LogicOp::Min),
            "angle" => Some(LogicOp::Angle),
            "angleDiff" => Some(LogicOp::AngleDiff),
            "len" => Some(LogicOp::Len),
            "noise" => Some(LogicOp::Noise),
            "abs" => Some(LogicOp::Abs),
            "sign" => Some(LogicOp::Sign),
            "log" => Some(LogicOp::Log),
            "logn" => Some(LogicOp::Logn),
            "log10" => Some(LogicOp::Log10),
            "floor" => Some(LogicOp::Floor),
            "ceil" => Some(LogicOp::Ceil),
            "round" => Some(LogicOp::Round),
            "sqrt" => Some(LogicOp::Sqrt),
            "rand" => Some(LogicOp::Rand),
            "sin" => Some(LogicOp::Sin),
            "cos" => Some(LogicOp::Cos),
            "tan" => Some(LogicOp::Tan),
            "asin" => Some(LogicOp::Asin),
            "acos" => Some(LogicOp::Acos),
            "atan" => Some(LogicOp::Atan),
            _ => None,
        }
    }

    pub const fn unary(self) -> bool {
        matches!(
            self,
            LogicOp::Not
                | LogicOp::Abs
                | LogicOp::Sign
                | LogicOp::Log
                | LogicOp::Log10
                | LogicOp::Floor
                | LogicOp::Ceil
                | LogicOp::Round
                | LogicOp::Sqrt
                | LogicOp::Rand
                | LogicOp::Sin
                | LogicOp::Cos
                | LogicOp::Tan
                | LogicOp::Asin
                | LogicOp::Acos
                | LogicOp::Atan
        )
    }

    pub const fn func(self) -> bool {
        matches!(
            self,
            LogicOp::Max
                | LogicOp::Min
                | LogicOp::Angle
                | LogicOp::AngleDiff
                | LogicOp::Len
                | LogicOp::Noise
        )
    }

    pub fn eval_binary(self, a: f64, b: f64) -> Option<f64> {
        let value = match self {
            LogicOp::Add => a + b,
            LogicOp::Sub => a - b,
            LogicOp::Mul => a * b,
            LogicOp::Div => a / b,
            LogicOp::Idiv => (a / b).floor(),
            LogicOp::Mod => a % b,
            LogicOp::Emod => ((a % b) + b) % b,
            LogicOp::Pow => a.powf(b),
            LogicOp::Equal => ((a - b).abs() < 0.000001) as u8 as f64,
            LogicOp::NotEqual => ((a - b).abs() >= 0.000001) as u8 as f64,
            LogicOp::Land => (a != 0.0 && b != 0.0) as u8 as f64,
            LogicOp::LessThan => (a < b) as u8 as f64,
            LogicOp::LessThanEq => (a <= b) as u8 as f64,
            LogicOp::GreaterThan => (a > b) as u8 as f64,
            LogicOp::GreaterThanEq => (a >= b) as u8 as f64,
            LogicOp::StrictEqual => 0.0,
            LogicOp::Shl => (java_long(a).wrapping_shl((java_long(b) as u32) & 63)) as f64,
            LogicOp::Shr => (java_long(a).wrapping_shr((java_long(b) as u32) & 63)) as f64,
            LogicOp::Ushr => {
                ((java_long(a) as u64).wrapping_shr((java_long(b) as u32) & 63)) as f64
            }
            LogicOp::Or => (java_long(a) | java_long(b)) as f64,
            LogicOp::And => (java_long(a) & java_long(b)) as f64,
            LogicOp::Xor => (java_long(a) ^ java_long(b)) as f64,
            LogicOp::Max => a.max(b),
            LogicOp::Min => a.min(b),
            LogicOp::Angle => java_angle(a, b),
            LogicOp::AngleDiff => java_angle_diff(a, b),
            LogicOp::Len => a.hypot(b),
            LogicOp::Logn => a.ln() / b.ln(),
            LogicOp::Noise => return None,
            _ => return None,
        };
        Some(value)
    }

    pub fn eval_unary(self, a: f64) -> Option<f64> {
        let value = match self {
            LogicOp::Not => (!java_long(a)) as f64,
            LogicOp::Abs => a.abs(),
            LogicOp::Sign => a.signum(),
            LogicOp::Log => a.ln(),
            LogicOp::Log10 => a.log10(),
            LogicOp::Floor => a.floor(),
            LogicOp::Ceil => a.ceil(),
            LogicOp::Round => (a + 0.5).floor(),
            LogicOp::Sqrt => a.sqrt(),
            LogicOp::Sin => a.to_radians().sin(),
            LogicOp::Cos => a.to_radians().cos(),
            LogicOp::Tan => a.to_radians().tan(),
            LogicOp::Asin => a.asin().to_degrees(),
            LogicOp::Acos => a.acos().to_degrees(),
            LogicOp::Atan => a.atan().to_degrees(),
            LogicOp::Rand => return None,
            _ => return None,
        };
        Some(value)
    }
}

impl fmt::Display for LogicOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}

fn java_long(value: f64) -> i64 {
    value as i64
}

fn java_angle(x: f64, y: f64) -> f64 {
    let angle = y.atan2(x).to_degrees();
    if angle < 0.0 {
        angle + 360.0
    } else {
        angle
    }
}

fn java_angle_diff(a: f64, b: f64) -> f64 {
    ((a - b + 180.0).rem_euclid(360.0) - 180.0).abs()
}

#[cfg(test)]
mod tests {
    use super::LogicOp;

    #[test]
    fn logic_op_order_symbols_flags_and_core_math_match_java_enum() {
        assert_eq!(LogicOp::ALL.len(), 45);
        assert_eq!(LogicOp::Add.ordinal(), 0);
        assert_eq!(LogicOp::StrictEqual.ordinal(), 15);
        assert_eq!(LogicOp::Not.ordinal(), 22);
        assert_eq!(LogicOp::Atan.ordinal(), 44);
        assert_eq!(LogicOp::from_ordinal(44), Some(LogicOp::Atan));
        assert_eq!(LogicOp::from_ordinal(45), None);
        assert_eq!(LogicOp::NotEqual.symbol(), "not");
        assert_eq!(LogicOp::Land.symbol(), "and");
        assert_eq!(LogicOp::And.symbol(), "b-and");
        assert_eq!(LogicOp::Not.symbol(), "flip");
        assert_eq!(LogicOp::AngleDiff.symbol(), "anglediff");
        assert_eq!(LogicOp::Add.to_string(), "+");

        assert_eq!(LogicOp::Add.java_name(), "add");
        assert_eq!(LogicOp::NotEqual.java_name(), "notEqual");
        assert_eq!(LogicOp::AngleDiff.java_name(), "angleDiff");
        assert_eq!(LogicOp::by_java_name("lessThan"), Some(LogicOp::LessThan));
        assert_eq!(LogicOp::by_java_name("angleDiff"), Some(LogicOp::AngleDiff));
        assert_eq!(LogicOp::by_java_name("+"), None);

        assert_eq!(
            LogicOp::ALL
                .iter()
                .map(|op| op.symbol())
                .collect::<Vec<_>>(),
            LogicOp::SYMBOLS.to_vec()
        );
        assert_eq!(
            LogicOp::ALL
                .iter()
                .map(|op| op.java_name())
                .collect::<Vec<_>>(),
            LogicOp::JAVA_NAMES.to_vec()
        );

        assert!(LogicOp::Not.unary());
        assert!(LogicOp::Sin.unary());
        assert!(!LogicOp::Add.unary());
        assert!(LogicOp::Max.func());
        assert!(LogicOp::Angle.func());
        assert!(!LogicOp::Logn.func());

        assert_eq!(LogicOp::Add.eval_binary(2.0, 3.0), Some(5.0));
        assert_eq!(LogicOp::Idiv.eval_binary(7.0, 2.0), Some(3.0));
        assert_eq!(LogicOp::Emod.eval_binary(-1.0, 5.0), Some(4.0));
        assert_eq!(LogicOp::Equal.eval_binary(1.0, 1.0 + 0.0000005), Some(1.0));
        assert_eq!(LogicOp::Land.eval_binary(1.0, 0.0), Some(0.0));
        assert_eq!(LogicOp::Shl.eval_binary(3.0, 2.0), Some(12.0));
        assert_eq!(LogicOp::And.eval_binary(6.0, 3.0), Some(2.0));
        assert_eq!(LogicOp::Not.eval_unary(0.0), Some(-1.0));
        assert_eq!(LogicOp::Abs.eval_unary(-3.5), Some(3.5));
        assert!((LogicOp::Angle.eval_binary(0.0, 1.0).unwrap() - 90.0).abs() < 0.000001);
        assert!((LogicOp::AngleDiff.eval_binary(350.0, 10.0).unwrap() - 20.0).abs() < 0.000001);
        assert!((LogicOp::Len.eval_binary(3.0, 4.0).unwrap() - 5.0).abs() < 0.000001);
        assert!((LogicOp::Sin.eval_unary(90.0).unwrap() - 1.0).abs() < 0.000001);
        assert_eq!(LogicOp::Noise.eval_binary(1.0, 2.0), None);
        assert_eq!(LogicOp::Rand.eval_unary(10.0), None);
    }
}
