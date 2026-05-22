//! Mirrors upstream `mindustry.logic.LAssembler` value and variable assembly helpers.

use std::collections::BTreeMap;

use super::{global_vars::logic_global_value, LVar};

const INVALID_NUM_NEGATIVE: i64 = i64::MIN;
const INVALID_NUM_POSITIVE: i64 = i64::MAX;

#[derive(Debug, Clone, PartialEq)]
pub enum LogicValue {
    Number(f64),
    Object(Option<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicAssembler {
    pub privileged: bool,
    pub vars: BTreeMap<String, LVar>,
}

impl Default for LogicAssembler {
    fn default() -> Self {
        Self::new()
    }
}

impl LogicAssembler {
    pub fn new() -> Self {
        let mut assembler = Self {
            privileged: false,
            vars: BTreeMap::new(),
        };
        assembler.put_var("@counter").is_obj = false;
        assembler.put_const("@unit", LogicValue::Object(None));
        assembler.put_const("@this", LogicValue::Object(None));
        assembler
    }

    pub fn var(&mut self, symbol: &str) -> &mut LVar {
        let mut symbol = symbol.trim().to_string();

        if self.vars.contains_key(&symbol) {
            return self.vars.get_mut(&symbol).expect("checked above");
        }

        if let Some(value) = logic_global_value(&symbol, self.privileged) {
            return self.put_const(symbol, value);
        }

        if symbol.starts_with('"') && symbol.ends_with('"') && !symbol.is_empty() {
            let value = symbol[1..symbol.len() - 1].replace("\\n", "\n");
            return self.put_const(format!("___{}", symbol), LogicValue::Object(Some(value)));
        }

        symbol = symbol.replace(' ', "_");
        let value = parse_logic_double(&symbol);

        if value.is_nan() {
            self.put_var(symbol)
        } else {
            self.put_const(
                format!("___{}", if value.is_infinite() { 0.0 } else { value }),
                LogicValue::Number(if value.is_infinite() { 0.0 } else { value }),
            )
        }
    }

    pub fn instruction_var(&mut self, symbol: &str) -> LVar {
        self.var(symbol).clone()
    }

    pub fn put_const(&mut self, name: impl Into<String>, value: LogicValue) -> &mut LVar {
        let var = self.put_var(name);
        match value {
            LogicValue::Number(value) => {
                var.is_obj = false;
                var.numval = value;
                var.objval = None;
            }
            LogicValue::Object(value) => {
                var.is_obj = true;
                var.objval = value;
            }
        }
        var.constant = true;
        var
    }

    pub fn put_var(&mut self, name: impl Into<String>) -> &mut LVar {
        let name = name.into();
        self.vars.entry(name.clone()).or_insert_with(|| {
            let mut var = LVar::new(name);
            var.is_obj = true;
            var
        })
    }

    pub fn get_var(&self, name: &str) -> Option<&LVar> {
        self.vars.get(name)
    }
}

pub fn parse_logic_double(symbol: &str) -> f64 {
    if symbol.starts_with("0b") {
        return parse_logic_long(false, symbol, 2, 2, symbol.len());
    }
    if symbol.starts_with("+0b") {
        return parse_logic_long(false, symbol, 2, 3, symbol.len());
    }
    if symbol.starts_with("-0b") {
        return parse_logic_long(true, symbol, 2, 3, symbol.len());
    }
    if symbol.starts_with("0x") {
        return parse_logic_long(false, symbol, 16, 2, symbol.len());
    }
    if symbol.starts_with("+0x") {
        return parse_logic_long(false, symbol, 16, 3, symbol.len());
    }
    if symbol.starts_with("-0x") {
        return parse_logic_long(true, symbol, 16, 3, symbol.len());
    }
    if symbol.starts_with("%[") && symbol.ends_with(']') && symbol.len() > 3 {
        return parse_named_logic_color(symbol);
    }
    if symbol.starts_with('%') && (symbol.len() == 7 || symbol.len() == 9) {
        return parse_logic_color(symbol);
    }

    parse_arc_double(symbol).unwrap_or(f64::NAN)
}

pub fn parse_logic_long(negative: bool, s: &str, radix: u32, start: usize, end: usize) -> f64 {
    let used_invalid = if negative {
        INVALID_NUM_POSITIVE
    } else {
        INVALID_NUM_NEGATIVE
    };
    let Some(slice) = s.get(start..end) else {
        return f64::NAN;
    };
    match i64::from_str_radix(slice, radix) {
        Ok(value) if value != used_invalid => {
            if negative {
                -(value as f64)
            } else {
                value as f64
            }
        }
        _ => f64::NAN,
    }
}

pub fn parse_logic_color(symbol: &str) -> f64 {
    let r = parse_hex_byte_or_zero(symbol, 1, 3);
    let g = parse_hex_byte_or_zero(symbol, 3, 5);
    let b = parse_hex_byte_or_zero(symbol, 5, 7);
    let a = if symbol.len() == 9 {
        parse_hex_byte_or_zero(symbol, 7, 9)
    } else {
        255
    };

    rgba_to_double_bits(r, g, b, a)
}

pub fn parse_named_logic_color(symbol: &str) -> f64 {
    let name = &symbol[2..symbol.len() - 1];
    named_logic_color_rgba(name)
        .map(rgba_u32_to_double_bits)
        .unwrap_or(f64::NAN)
}

pub fn rgba_to_double_bits(r: u8, g: u8, b: u8, a: u8) -> f64 {
    let bits = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32;
    rgba_u32_to_double_bits(bits)
}

pub fn rgba_u32_to_double_bits(rgba: u32) -> f64 {
    f64::from_bits(rgba as u64)
}

pub fn double_bits_to_rgba(value: f64) -> u32 {
    value.to_bits() as u32
}

pub fn unpack_double_color(value: f64) -> (f64, f64, f64, f64) {
    let rgba = double_bits_to_rgba(value);
    (
        ((rgba >> 24) & 0xff) as f64 / 255.0,
        ((rgba >> 16) & 0xff) as f64 / 255.0,
        ((rgba >> 8) & 0xff) as f64 / 255.0,
        (rgba & 0xff) as f64 / 255.0,
    )
}

pub fn logic_color_channel_to_byte(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0) as u8
}

fn parse_hex_byte_or_zero(symbol: &str, start: usize, end: usize) -> u8 {
    symbol
        .get(start..end)
        .and_then(|slice| u8::from_str_radix(slice, 16).ok())
        .unwrap_or(0)
}

fn parse_arc_double(symbol: &str) -> Option<f64> {
    if symbol.is_empty() || symbol.eq_ignore_ascii_case("nan") || symbol.contains("Infinity") {
        return None;
    }
    symbol.parse::<f64>().ok()
}

pub fn named_logic_color_rgba(name: &str) -> Option<u32> {
    let normalized = name.replace('_', "").to_ascii_lowercase();
    match normalized.as_str() {
        "clear" => Some(0x00000000),
        "black" => Some(0x000000ff),
        "white" => Some(0xffffffff),
        "lightgray" | "lightgrey" => Some(0xbfbfbfff),
        "gray" | "grey" => Some(0x7f7f7fff),
        "darkgray" | "darkgrey" => Some(0x3f3f3fff),
        "blue" => Some(0x4169e1ff),
        "navy" => Some(0x00007fff),
        "royal" => Some(0x4169e1ff),
        "slate" => Some(0x708090ff),
        "sky" => Some(0x87ceebff),
        "cyan" => Some(0x00ffffff),
        "teal" => Some(0x007f7fff),
        "green" => Some(0x00ff00ff),
        "acid" => Some(0x7fff00ff),
        "lime" => Some(0x32cd32ff),
        "forest" => Some(0x228b22ff),
        "olive" => Some(0x6b8e23ff),
        "yellow" => Some(0xffff00ff),
        "gold" => Some(0xffd700ff),
        "goldenrod" => Some(0xdaa520ff),
        "orange" => Some(0xffa500ff),
        "brown" => Some(0x8b4513ff),
        "tan" => Some(0xd2b48cff),
        "brick" => Some(0xb22222ff),
        "red" => Some(0xff0000ff),
        "scarlet" => Some(0xff341cff),
        "crimson" => Some(0xdc143cff),
        "coral" => Some(0xff7f50ff),
        "salmon" => Some(0xfa8072ff),
        "pink" => Some(0xff69b4ff),
        "magenta" => Some(0xff00ffff),
        "purple" => Some(0xa020f0ff),
        "violet" => Some(0xee82eeff),
        "maroon" => Some(0xb03060ff),
        "accent" | "stat" => Some(0xffd37fff),
        "unlaunched" => Some(0x8982edff),
        "negstat" => Some(0xe55454ff),
        // `highlight` is Pal.accent lerped 30% toward white in UI.loadColors().
        "highlight" => Some(0xffdea5ff),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::logic::LVarValue;

    #[test]
    fn assembler_parse_double_matches_java_numeric_and_color_rules() {
        assert_eq!(parse_logic_double("0b101"), 5.0);
        assert_eq!(parse_logic_double("+0b101"), 5.0);
        assert_eq!(parse_logic_double("-0b101"), -5.0);
        assert_eq!(parse_logic_double("0x10"), 16.0);
        assert_eq!(parse_logic_double("+0x10"), 16.0);
        assert_eq!(parse_logic_double("-0x10"), -16.0);
        assert_eq!(parse_logic_double("1e3"), 1000.0);
        assert!(parse_logic_double("NaN").is_nan());
        assert!(parse_logic_double("Infinity").is_nan());
        assert!(parse_logic_double("0b102").is_nan());
        assert!(parse_logic_double("0xg1").is_nan());

        assert_eq!(
            rgba_to_double_bits(0xff, 0x00, 0xaa, 0xff).to_bits(),
            0xff00aaff
        );
        assert_eq!(parse_logic_color("%ff00aa").to_bits(), 0xff00aaff);
        assert_eq!(parse_logic_color("%ff00aa80").to_bits(), 0xff00aa80);
        // Arc Strings.parseInt(..., default=0, ...) falls back to 0 for invalid slices.
        assert_eq!(parse_logic_color("%zz00aa").to_bits(), 0x0000aaff);

        assert_eq!(parse_named_logic_color("%[scarlet]").to_bits(), 0xff341cff);
        assert_eq!(
            parse_named_logic_color("%[LIGHT_GRAY]").to_bits(),
            0xbfbfbfff
        );
        assert_eq!(parse_named_logic_color("%[accent]").to_bits(), 0xffd37fff);
        assert!(parse_named_logic_color("%[missing]").is_nan());

        let (r, g, b, a) = unpack_double_color(parse_logic_color("%ff00aa80"));
        assert_eq!(logic_color_channel_to_byte(r), 255);
        assert_eq!(logic_color_channel_to_byte(g), 0);
        assert_eq!(logic_color_channel_to_byte(b), 170);
        assert_eq!(logic_color_channel_to_byte(a), 128);
    }

    #[test]
    fn assembler_var_put_var_and_put_const_follow_java_rules() {
        let mut asm = LogicAssembler::new();
        assert_eq!(
            asm.get_var("@counter").unwrap().value(),
            LVarValue::Number(0.0)
        );
        assert_eq!(
            asm.get_var("@unit").unwrap().value(),
            LVarValue::Object(None)
        );
        assert_eq!(
            asm.get_var("@this").unwrap().value(),
            LVarValue::Object(None)
        );

        let text = asm.var("\"hello\\nworld\"");
        assert_eq!(text.name, "___\"hello\\nworld\"");
        assert_eq!(text.value(), LVarValue::Object(Some("hello\nworld".into())));
        assert!(text.constant);

        let spaced = asm.var(" a b ");
        assert_eq!(spaced.name, "a_b");
        assert!(spaced.is_obj);
        assert_eq!(spaced.value(), LVarValue::Object(None));
        assert!(!spaced.constant);

        let number = asm.var("0x10");
        assert_eq!(number.name, "___16");
        assert_eq!(number.value(), LVarValue::Number(16.0));
        assert!(number.constant);

        let overflow = asm.var("1e309");
        assert_eq!(overflow.name, "___0");
        assert_eq!(overflow.value(), LVarValue::Number(0.0));
        assert!(overflow.constant);

        let existing = asm.put_var("same") as *mut LVar;
        asm.put_var("same").set_num(5.0);
        assert_eq!(asm.get_var("same").unwrap().numval, 5.0);
        assert_eq!(existing, asm.put_var("same") as *mut LVar);
    }
}
