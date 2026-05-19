// Mirrors upstream core/src/mindustry/logic. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub const LOGIC_CTRL_PROCESSOR: i32 = 1;
pub const LOGIC_CTRL_PLAYER: i32 = 2;
pub const LOGIC_CTRL_COMMAND: i32 = 3;
pub const LOOKABLE_CONTENT: [&str; 5] = ["block", "unit", "item", "liquid", "team"];
pub const WRITABLE_LOOKABLE_CONTENT: [&str; 4] = ["block", "unit", "item", "liquid"];
pub const LOGIC_PARSER_MAX_TOKENS: usize = 16;
pub const LOGIC_PARSER_MAX_JUMPS: usize = 500;
pub const LOGIC_CANVAS_INVALID_JUMP: i32 = i32::MAX;

const INVALID_NUM_NEGATIVE: i64 = i64::MIN;
const INVALID_NUM_POSITIVE: i64 = i64::MAX;

#[derive(Debug, Clone, PartialEq)]
pub enum LogicValue {
    Number(f64),
    Object(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicParseError {
    pub message: String,
}

impl LogicParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for LogicParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid code. {}", self.message)
    }
}

impl std::error::Error for LogicParseError {}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicStatementKind {
    Label {
        name: String,
        line: usize,
    },
    Instruction {
        tokens: Vec<String>,
        line: usize,
        jump_label: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicParserOutput {
    pub statements: Vec<LogicStatementKind>,
    pub jump_locations: BTreeMap<String, usize>,
}

pub fn parse_logic_statements(text: &str) -> Result<LogicParserOutput, LogicParseError> {
    LogicParser::new(text).parse()
}

struct LogicParser<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    statements: Vec<LogicStatementKind>,
    jump_locations: BTreeMap<String, usize>,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> LogicParser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            chars: text.chars().collect(),
            pos: 0,
            line: 0,
            statements: Vec::new(),
            jump_locations: BTreeMap::new(),
            _marker: std::marker::PhantomData,
        }
    }

    fn parse(mut self) -> Result<LogicParserOutput, LogicParseError> {
        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                '\n' | ';' | ' ' => self.pos += 1,
                '\r' => self.pos = (self.pos + 2).min(self.chars.len()),
                _ => self.statement()?,
            }
        }

        Ok(LogicParserOutput {
            statements: self.statements,
            jump_locations: self.jump_locations,
        })
    }

    fn statement(&mut self) -> Result<(), LogicParseError> {
        let mut expect_next = false;
        let mut tokens = Vec::new();

        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if tokens.len() >= LOGIC_PARSER_MAX_TOKENS {
                return Err(LogicParseError::new(format!(
                    "Line too long; may only contain {} tokens",
                    LOGIC_PARSER_MAX_TOKENS
                )));
            }
            if c == '\n' || c == ';' {
                break;
            }
            if expect_next && c != ' ' && c != '#' && c != '\t' {
                return Err(LogicParseError::new("Expected space after string/token."));
            }

            expect_next = false;
            if c == '#' {
                self.comment();
                break;
            } else if c == '"' {
                tokens.push(self.string()?);
                expect_next = true;
            } else if c != ' ' && c != '\t' {
                tokens.push(self.token());
                expect_next = true;
            } else {
                self.pos += 1;
            }
        }

        if !tokens.is_empty() {
            check_logic_tokens(&mut tokens);
            if tokens.len() == 1 && tokens[0].ends_with(':') {
                if self.jump_locations.len() >= LOGIC_PARSER_MAX_JUMPS {
                    return Err(LogicParseError::new(format!(
                        "Too many jump locations. Max jumps: {}",
                        LOGIC_PARSER_MAX_JUMPS
                    )));
                }
                let label = tokens[0][..tokens[0].len() - 1].to_string();
                self.jump_locations.insert(label.clone(), self.line);
                self.statements.push(LogicStatementKind::Label {
                    name: label,
                    line: self.line,
                });
            } else {
                let mut jump_label = None;
                if tokens[0] == "jump" && tokens.len() > 1 && !can_parse_i32(&tokens[1]) {
                    jump_label = Some(tokens[1].clone());
                    tokens[1] = "-1".to_string();
                }

                for token in tokens.iter_mut().skip(1) {
                    if token == "@configure" {
                        *token = "@config".to_string();
                    }
                    if token == "configure" {
                        *token = "config".to_string();
                    }
                }

                self.statements.push(LogicStatementKind::Instruction {
                    tokens,
                    line: self.line,
                    jump_label,
                });
                self.line += 1;
            }
        }
        Ok(())
    }

    fn comment(&mut self) {
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            self.pos += 1;
            if c == '\n' {
                break;
            }
        }
    }

    fn string(&mut self) -> Result<String, LogicParseError> {
        let from = self.pos;
        let mut utflen = 0usize;

        self.pos += 1;
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c == '\n' {
                return Err(LogicParseError::new(
                    "Missing closing quote \" before end of line.",
                ));
            } else if c == '"' {
                break;
            }
            utflen += java_modified_utf_char_len(c);
            self.pos += 1;
        }

        if self.pos >= self.chars.len() || self.chars[self.pos] != '"' {
            return Err(LogicParseError::new(
                "Missing closing quote \" before end of file.",
            ));
        }
        if utflen > 65535 {
            return Err(LogicParseError::new("String value too long."));
        }

        self.pos += 1;
        Ok(self.chars[from..self.pos].iter().collect())
    }

    fn token(&mut self) -> String {
        let from = self.pos;
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c == '\n' || c == ' ' || c == '#' || c == '\t' || c == ';' {
                break;
            }
            self.pos += 1;
        }
        self.chars[from..self.pos].iter().collect()
    }
}

pub fn check_logic_tokens(tokens: &mut [String]) {
    if tokens.first().is_some_and(|token| token == "op") && tokens.len() > 1 {
        if tokens[1] == "atan2" {
            tokens[1] = "angle".to_string();
        } else if tokens[1] == "dst" {
            tokens[1] = "len".to_string();
        }
    }
}

fn can_parse_i32(value: &str) -> bool {
    value.parse::<i32>().is_ok()
}

fn java_modified_utf_char_len(c: char) -> usize {
    let code = c as u32;
    if code != 0 && code <= 0x7f {
        1
    } else if code <= 0x7ff {
        2
    } else if code <= 0xffff {
        3
    } else {
        // Java source parser works with UTF-16 chars. A supplementary Unicode scalar
        // is two surrogate chars, each encoded as three bytes by modified UTF-8.
        6
    }
}

pub fn sanitize_logic_value(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    } else if value.chars().count() == 1 {
        let c = value.chars().next().unwrap();
        if c == '"' || c == ';' || c == ' ' {
            return "invalid".to_string();
        }
    } else {
        let mut res = String::with_capacity(value.len());
        if value.starts_with('"') && value.ends_with('"') {
            res.push('"');
            for c in value[1..value.len() - 1].chars() {
                res.push(if c == '"' { '\'' } else { c });
            }
            res.push('"');
        } else {
            for c in value.chars() {
                res.push(match c {
                    ';' => 's',
                    '"' => '\'',
                    ' ' => '_',
                    _ => c,
                });
            }
        }
        return res;
    }

    value.to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicAlign {
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl LogicAlign {
    pub const ALL: [LogicAlign; 9] = [
        LogicAlign::TopLeft,
        LogicAlign::Top,
        LogicAlign::TopRight,
        LogicAlign::Left,
        LogicAlign::Center,
        LogicAlign::Right,
        LogicAlign::BottomLeft,
        LogicAlign::Bottom,
        LogicAlign::BottomRight,
    ];

    pub const fn java_bits(self) -> i32 {
        match self {
            LogicAlign::Center => 1,
            LogicAlign::Top => 2,
            LogicAlign::Bottom => 4,
            LogicAlign::Left => 8,
            LogicAlign::Right => 16,
            LogicAlign::TopLeft => 10,
            LogicAlign::TopRight => 18,
            LogicAlign::BottomLeft => 12,
            LogicAlign::BottomRight => 20,
        }
    }

    pub const fn wire_name(self) -> &'static str {
        match self {
            LogicAlign::Center => "center",
            LogicAlign::Top => "top",
            LogicAlign::Bottom => "bottom",
            LogicAlign::Left => "left",
            LogicAlign::Right => "right",
            LogicAlign::TopLeft => "topLeft",
            LogicAlign::TopRight => "topRight",
            LogicAlign::BottomLeft => "bottomLeft",
            LogicAlign::BottomRight => "bottomRight",
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "center" => Some(LogicAlign::Center),
            "top" => Some(LogicAlign::Top),
            "bottom" => Some(LogicAlign::Bottom),
            "left" => Some(LogicAlign::Left),
            "right" => Some(LogicAlign::Right),
            "topLeft" => Some(LogicAlign::TopLeft),
            "topRight" => Some(LogicAlign::TopRight),
            "bottomLeft" => Some(LogicAlign::BottomLeft),
            "bottomRight" => Some(LogicAlign::BottomRight),
            _ => None,
        }
    }

    pub fn by_java_bits(bits: i32) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|align| align.java_bits() == bits)
    }

    pub const fn is_center_horizontal(self) -> bool {
        self.java_bits() & 8 == 0 && self.java_bits() & 16 == 0
    }

    pub const fn is_center_vertical(self) -> bool {
        self.java_bits() & 2 == 0 && self.java_bits() & 4 == 0
    }
}

pub fn logic_canvas_use_rows(viewport_width: f32, ui_scale: f32) -> bool {
    viewport_width < ui_scale * 900.0 * 1.2
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LogicJumpRange {
    pub begin: i32,
    pub end: i32,
    pub flipped: bool,
}

impl LogicJumpRange {
    pub const fn invalid() -> Self {
        Self {
            begin: LOGIC_CANVAS_INVALID_JUMP,
            end: LOGIC_CANVAS_INVALID_JUMP,
            flipped: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LogicJumpPlacement {
    pub begin: i32,
    pub end: i32,
    pub flipped: bool,
    pub pred_height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LogicJumpRepresentative {
    input_index: usize,
    range: LogicJumpRange,
    pred_height: usize,
}

pub fn normalize_logic_jump_range(from: i32, to: Option<i32>) -> Option<LogicJumpRange> {
    to.map(|to| LogicJumpRange {
        begin: from.min(to),
        end: from.max(to),
        flipped: from >= to,
    })
}

pub fn representative_logic_jumps(ranges: &[Option<LogicJumpRange>]) -> Vec<usize> {
    let mut repr_before: BTreeMap<i32, usize> = BTreeMap::new();
    let mut repr_after: BTreeMap<i32, usize> = BTreeMap::new();

    for (index, range) in ranges.iter().enumerate() {
        let Some(range) = range else {
            continue;
        };
        if range.begin == LOGIC_CANVAS_INVALID_JUMP {
            continue;
        }

        if range.flipped {
            if let Some(prev) = repr_after.get(&range.begin).and_then(|idx| ranges[*idx]) {
                if prev.end >= range.end {
                    continue;
                }
            }
            repr_after.insert(range.begin, index);
        } else {
            if let Some(prev) = repr_before.get(&range.end).and_then(|idx| ranges[*idx]) {
                if prev.begin <= range.begin {
                    continue;
                }
            }
            repr_before.insert(range.end, index);
        }
    }

    let mut reps: Vec<_> = repr_before
        .values()
        .chain(repr_after.values())
        .copied()
        .collect();
    reps.sort_by_key(|index| ranges[*index].unwrap().begin);
    reps
}

pub fn assign_logic_jump_heights(
    ranges: &[Option<LogicJumpRange>],
) -> Vec<Option<LogicJumpPlacement>> {
    let representatives = representative_logic_jumps(ranges);
    let mut repr_before: BTreeMap<i32, usize> = BTreeMap::new();
    let mut repr_after: BTreeMap<i32, usize> = BTreeMap::new();
    let mut processed = Vec::with_capacity(representatives.len());

    for input_index in representatives {
        let range = ranges[input_index].unwrap();
        let rep_index = processed.len();
        if range.flipped {
            repr_after.insert(range.begin, rep_index);
        } else {
            repr_before.insert(range.end, rep_index);
        }
        processed.push(LogicJumpRepresentative {
            input_index,
            range,
            pred_height: 0,
        });
    }

    let mut marked_done = vec![false; processed.len()];
    let mut occupiers: Vec<usize> = Vec::new();
    let mut occupied: BTreeSet<usize> = BTreeSet::new();

    for index in 0..processed.len() {
        let begin = processed[index].range.begin;
        occupiers.retain(|occupier| {
            if processed[*occupier].range.end > begin {
                true
            } else {
                occupied.remove(&processed[*occupier].pred_height);
                false
            }
        });
        let height = logic_jump_height(
            index,
            &mut processed,
            &mut marked_done,
            &occupiers,
            &occupied,
        );
        occupiers.push(index);
        occupied.insert(height);
    }

    let mut output = vec![None; ranges.len()];
    for (index, range) in ranges.iter().enumerate() {
        let Some(range) = range else {
            continue;
        };
        if range.begin == LOGIC_CANVAS_INVALID_JUMP {
            continue;
        }

        let rep_index = if range.flipped {
            repr_after.get(&range.begin)
        } else {
            repr_before.get(&range.end)
        };
        if let Some(rep_index) = rep_index {
            let rep = processed[*rep_index];
            output[index] = Some(LogicJumpPlacement {
                begin: range.begin,
                end: range.end,
                flipped: range.flipped,
                pred_height: rep.pred_height,
            });
        }
    }

    // Keep representative source indices observable by ensuring every representative
    // produced a placement at its original index. This mirrors the Java pass that
    // recalculates representative `JumpCurve`s first, then copies their height back
    // to every duplicate curve.
    debug_assert!(processed
        .iter()
        .all(|rep| output[rep.input_index].is_some()));

    output
}

fn logic_jump_height(
    index: usize,
    processed: &mut [LogicJumpRepresentative],
    marked_done: &mut [bool],
    occupiers: &[usize],
    occupied: &BTreeSet<usize>,
) -> usize {
    if marked_done[index] {
        return processed[index].pred_height;
    }

    let jmp_end = processed[index].range.end;
    let mut tmp_occupiers = occupiers.to_vec();
    let mut tmp_occupied = occupied.clone();

    let mut max_nested: Option<usize> = None;
    for next in index + 1..processed.len() {
        let cur = processed[next].range;
        if cur.end > jmp_end {
            continue;
        }

        tmp_occupiers.retain(|occupier| {
            if processed[*occupier].range.end > cur.begin {
                true
            } else {
                tmp_occupied.remove(&processed[*occupier].pred_height);
                false
            }
        });

        let height = logic_jump_height(next, processed, marked_done, &tmp_occupiers, &tmp_occupied);
        tmp_occupiers.push(next);
        tmp_occupied.insert(height);
        max_nested = Some(max_nested.map_or(height, |max| max.max(height)));
    }

    let mut height = max_nested.map_or(0, |max| max + 1);
    while occupied.contains(&height) {
        height += 1;
    }

    processed[index].pred_height = height;
    marked_done[index] = true;
    height
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicEffectEntry {
    pub name: String,
    pub effect: &'static str,
    pub size: bool,
    pub rotate: bool,
    pub color: bool,
    pub data: Option<&'static str>,
    pub bounds: f32,
}

impl LogicEffectEntry {
    pub fn new(name: impl Into<String>, effect: &'static str) -> Self {
        Self {
            name: name.into(),
            effect,
            size: false,
            rotate: false,
            color: false,
            data: None,
            bounds: -1.0,
        }
    }

    pub fn size(mut self) -> Self {
        self.size = true;
        self
    }

    pub fn rotate(mut self) -> Self {
        self.rotate = true;
        self
    }

    pub fn color(mut self) -> Self {
        self.color = true;
        self
    }

    pub fn data(mut self, data: &'static str) -> Self {
        self.data = Some(data);
        self
    }

    pub fn bounds(mut self, bounds: f32) -> Self {
        self.bounds = bounds;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicEffectSpec {
    pub name: &'static str,
    pub effect: &'static str,
    pub size: bool,
    pub rotate: bool,
    pub color: bool,
    pub data: Option<&'static str>,
    pub bounds: f32,
}

impl LogicEffectSpec {
    pub fn to_entry(&self) -> LogicEffectEntry {
        LogicEffectEntry {
            name: self.name.to_string(),
            effect: self.effect,
            size: self.size,
            rotate: self.rotate,
            color: self.color,
            data: self.data,
            bounds: self.bounds,
        }
    }
}

pub const LOGIC_EFFECTS: [LogicEffectSpec; 33] = [
    LogicEffectSpec {
        name: "warn",
        effect: "unitCapKill",
        size: false,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "cross",
        effect: "unitEnvKill",
        size: false,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "blockFall",
        effect: "blockCrash",
        size: false,
        rotate: false,
        color: false,
        data: Some("Block"),
        bounds: 100.0,
    },
    LogicEffectSpec {
        name: "placeBlock",
        effect: "placeBlock",
        size: true,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "placeBlockSpark",
        effect: "coreLaunchConstruct",
        size: true,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "breakBlock",
        effect: "breakBlock",
        size: true,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "spawn",
        effect: "spawn",
        size: false,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "trail",
        effect: "colorTrail",
        size: true,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "breakProp",
        effect: "breakProp",
        size: true,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokeCloud",
        effect: "missileTrailSmoke",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "vapor",
        effect: "vapor",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "hit",
        effect: "hitBulletColor",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "hitSquare",
        effect: "hitSquaresColor",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "shootSmall",
        effect: "shootSmall",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "shootBig",
        effect: "shootTitan",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokeSmall",
        effect: "shootSmallSmoke",
        size: false,
        rotate: true,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokeBig",
        effect: "shootBigSmoke",
        size: false,
        rotate: true,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokeColor",
        effect: "shootSmokeTitan",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokeSquare",
        effect: "shootSmokeSquare",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokeSquareBig",
        effect: "shootSmokeSquareBig",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "spark",
        effect: "hitLaserBlast",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "sparkBig",
        effect: "circleColorSpark",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "sparkShoot",
        effect: "colorSpark",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "sparkShootBig",
        effect: "randLifeSpark",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "drill",
        effect: "mine",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "drillBig",
        effect: "mineHuge",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "lightBlock",
        effect: "lightBlock",
        size: true,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "explosion",
        effect: "dynamicExplosion",
        size: true,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokePuff",
        effect: "smokePuff",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "sparkExplosion",
        effect: "titanExplosion",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "crossExplosion",
        effect: "dynamicSpikes",
        size: true,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "wave",
        effect: "dynamicWave",
        size: true,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "bubble",
        effect: "airBubble",
        size: false,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
];

pub fn logic_effect_names() -> Vec<&'static str> {
    LOGIC_EFFECTS.iter().map(|entry| entry.name).collect()
}

pub fn get_logic_effect(name: &str) -> Option<&'static LogicEffectSpec> {
    LOGIC_EFFECTS.iter().find(|entry| entry.name == name)
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicEffectRegistry {
    entries: Vec<LogicEffectEntry>,
}

impl Default for LogicEffectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LogicEffectRegistry {
    pub fn new() -> Self {
        Self {
            entries: LOGIC_EFFECTS
                .iter()
                .map(LogicEffectSpec::to_entry)
                .collect(),
        }
    }

    pub fn entries(&self) -> &[LogicEffectEntry] {
        &self.entries
    }

    pub fn all(&self) -> Vec<&str> {
        self.entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect()
    }

    pub fn get(&self, name: &str) -> Option<&LogicEffectEntry> {
        self.entries.iter().find(|entry| entry.name == name)
    }

    pub fn add(&mut self, name: impl Into<String>, mut entry: LogicEffectEntry) {
        let name = name.into();
        entry.name = name.clone();
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|existing| existing.name == name)
        {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LVarValue {
    Number(f64),
    Object(Option<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LVar {
    pub name: String,
    pub id: i32,
    pub is_obj: bool,
    pub constant: bool,
    pub objval: Option<String>,
    pub numval: f64,
    pub sync_time: i64,
}

impl LVar {
    pub fn new(name: impl Into<String>) -> Self {
        Self::with_id(name, -1)
    }

    pub fn with_id(name: impl Into<String>, id: i32) -> Self {
        Self::with_id_constant(name, id, false)
    }

    pub fn with_id_constant(name: impl Into<String>, id: i32, constant: bool) -> Self {
        Self {
            name: name.into(),
            id,
            is_obj: false,
            constant,
            objval: None,
            numval: 0.0,
            sync_time: 0,
        }
    }

    pub const fn invalid(value: f64) -> bool {
        value.is_nan() || value.is_infinite()
    }

    pub fn obj(&self) -> Option<&str> {
        self.is_obj.then_some(self.objval.as_deref()).flatten()
    }

    pub fn bool(&self) -> bool {
        if self.is_obj {
            self.objval.is_some()
        } else {
            self.numval.abs() >= 0.00001
        }
    }

    pub fn num(&self) -> f64 {
        if self.is_obj {
            self.objval.is_some() as u8 as f64
        } else if Self::invalid(self.numval) {
            0.0
        } else {
            self.numval
        }
    }

    pub fn num_or_nan(&self) -> f64 {
        if self.is_obj {
            if self.objval.is_some() {
                1.0
            } else {
                f64::NAN
            }
        } else if Self::invalid(self.numval) {
            0.0
        } else {
            self.numval
        }
    }

    pub fn numf(&self) -> f32 {
        self.num() as f32
    }

    pub fn numf_or_nan(&self) -> f32 {
        self.num_or_nan() as f32
    }

    pub fn numi(&self) -> i32 {
        self.num() as i32
    }

    pub fn set_bool(&mut self, value: bool) {
        self.set_num(if value { 1.0 } else { 0.0 });
    }

    pub fn set_num(&mut self, value: f64) {
        if self.constant {
            return;
        }
        if Self::invalid(value) {
            self.objval = None;
            self.is_obj = true;
        } else {
            self.numval = value;
            self.objval = None;
            self.is_obj = false;
        }
    }

    pub fn set_obj(&mut self, value: Option<String>) {
        if self.constant {
            return;
        }
        self.objval = value;
        self.is_obj = true;
    }

    pub fn set_const_obj(&mut self, value: Option<String>) {
        self.objval = value;
        self.is_obj = true;
    }

    pub fn set_from(&mut self, other: &LVar) {
        self.is_obj = other.is_obj;
        if self.is_obj {
            self.objval = other.objval.clone();
        } else {
            self.numval = if Self::invalid(other.numval) {
                0.0
            } else {
                other.numval
            };
        }
    }

    pub fn value(&self) -> LVarValue {
        if self.is_obj {
            LVarValue::Object(self.objval.clone())
        } else {
            LVarValue::Number(self.numval)
        }
    }
}

impl fmt::Display for LVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_obj {
            match &self.objval {
                Some(value) => write!(f, "{}: {}", self.name, value)?,
                None => write!(f, "{}: null", self.name)?,
            }
        } else {
            write!(f, "{}: {}", self.name, self.numval)?;
        }
        if self.constant {
            f.write_str(" [const]")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalVarEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub privileged: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVarSnapshot {
    pub entries: Vec<GlobalVarEntry>,
}

impl GlobalVarSnapshot {
    pub fn baseline() -> Self {
        let mut entries = Vec::new();
        for name in [
            "sectionProcessor",
            "@this",
            "@thisx",
            "@thisy",
            "@links",
            "@ipt",
            "sectionGeneral",
            "false",
            "true",
            "@pi",
            "@e",
            "@degToRad",
            "@radToDeg",
            "sectionMap",
            "@time",
            "@tick",
            "@second",
            "@minute",
            "@waveNumber",
            "@waveTime",
            "@mapw",
            "@maph",
            "sectionNetwork",
            "@server",
            "@client",
            "@clientLocale",
            "@clientUnit",
            "@clientName",
            "@clientTeam",
            "@clientMobile",
            "sectionLookup",
        ] {
            let privileged = matches!(
                name,
                "@server"
                    | "@client"
                    | "@clientLocale"
                    | "@clientUnit"
                    | "@clientName"
                    | "@clientTeam"
                    | "@clientMobile"
            );
            entries.push(GlobalVarEntry {
                name,
                description: "",
                icon: "",
                privileged,
            });
        }
        Self { entries }
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.entries.iter().map(|entry| entry.name).collect()
    }

    pub fn visible_to_privileged(&self, privileged: bool) -> Vec<&'static str> {
        self.entries
            .iter()
            .filter(|entry| privileged || !entry.privileged)
            .map(|entry| entry.name)
            .collect()
    }
}

/// Mirrors upstream `mindustry.logic.LAccess`.
///
/// The declaration order is observable by logic scripts and generated
/// processors, so keep it identical to Java `values()`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LAccess {
    TotalItems,
    FirstItem,
    TotalLiquids,
    TotalPower,
    ItemCapacity,
    LiquidCapacity,
    PowerCapacity,
    PowerNetStored,
    PowerNetCapacity,
    PowerNetIn,
    PowerNetOut,
    Ammo,
    AmmoCapacity,
    CurrentAmmoType,
    MemoryCapacity,
    Health,
    MaxHealth,
    Heat,
    Shield,
    Armor,
    Efficiency,
    Progress,
    Timescale,
    Rotation,
    X,
    Y,
    VelocityX,
    VelocityY,
    ShootX,
    ShootY,
    CameraX,
    CameraY,
    CameraWidth,
    CameraHeight,
    DisplayWidth,
    DisplayHeight,
    BufferSize,
    Operations,
    Size,
    Solid,
    Dead,
    Range,
    Shooting,
    Boosting,
    MineX,
    MineY,
    Mining,
    BuildX,
    BuildY,
    PingX,
    PingY,
    PingText,
    Building,
    Breaking,
    Speed,
    Team,
    Type,
    Flag,
    Controlled,
    Controller,
    Name,
    PayloadCount,
    PayloadType,
    TotalPayload,
    PayloadCapacity,
    MaxUnits,
    Id,
    SelectedBlock,
    SelectedRotation,
    BulletLifetime,
    BulletTime,
    Enabled,
    Shoot,
    Shootp,
    Config,
    Color,
}

impl LAccess {
    pub const ALL: [LAccess; 76] = [
        LAccess::TotalItems,
        LAccess::FirstItem,
        LAccess::TotalLiquids,
        LAccess::TotalPower,
        LAccess::ItemCapacity,
        LAccess::LiquidCapacity,
        LAccess::PowerCapacity,
        LAccess::PowerNetStored,
        LAccess::PowerNetCapacity,
        LAccess::PowerNetIn,
        LAccess::PowerNetOut,
        LAccess::Ammo,
        LAccess::AmmoCapacity,
        LAccess::CurrentAmmoType,
        LAccess::MemoryCapacity,
        LAccess::Health,
        LAccess::MaxHealth,
        LAccess::Heat,
        LAccess::Shield,
        LAccess::Armor,
        LAccess::Efficiency,
        LAccess::Progress,
        LAccess::Timescale,
        LAccess::Rotation,
        LAccess::X,
        LAccess::Y,
        LAccess::VelocityX,
        LAccess::VelocityY,
        LAccess::ShootX,
        LAccess::ShootY,
        LAccess::CameraX,
        LAccess::CameraY,
        LAccess::CameraWidth,
        LAccess::CameraHeight,
        LAccess::DisplayWidth,
        LAccess::DisplayHeight,
        LAccess::BufferSize,
        LAccess::Operations,
        LAccess::Size,
        LAccess::Solid,
        LAccess::Dead,
        LAccess::Range,
        LAccess::Shooting,
        LAccess::Boosting,
        LAccess::MineX,
        LAccess::MineY,
        LAccess::Mining,
        LAccess::BuildX,
        LAccess::BuildY,
        LAccess::PingX,
        LAccess::PingY,
        LAccess::PingText,
        LAccess::Building,
        LAccess::Breaking,
        LAccess::Speed,
        LAccess::Team,
        LAccess::Type,
        LAccess::Flag,
        LAccess::Controlled,
        LAccess::Controller,
        LAccess::Name,
        LAccess::PayloadCount,
        LAccess::PayloadType,
        LAccess::TotalPayload,
        LAccess::PayloadCapacity,
        LAccess::MaxUnits,
        LAccess::Id,
        LAccess::SelectedBlock,
        LAccess::SelectedRotation,
        LAccess::BulletLifetime,
        LAccess::BulletTime,
        LAccess::Enabled,
        LAccess::Shoot,
        LAccess::Shootp,
        LAccess::Config,
        LAccess::Color,
    ];

    pub const SENSEABLE: [LAccess; 74] = [
        LAccess::TotalItems,
        LAccess::FirstItem,
        LAccess::TotalLiquids,
        LAccess::TotalPower,
        LAccess::ItemCapacity,
        LAccess::LiquidCapacity,
        LAccess::PowerCapacity,
        LAccess::PowerNetStored,
        LAccess::PowerNetCapacity,
        LAccess::PowerNetIn,
        LAccess::PowerNetOut,
        LAccess::Ammo,
        LAccess::AmmoCapacity,
        LAccess::CurrentAmmoType,
        LAccess::MemoryCapacity,
        LAccess::Health,
        LAccess::MaxHealth,
        LAccess::Heat,
        LAccess::Shield,
        LAccess::Armor,
        LAccess::Efficiency,
        LAccess::Progress,
        LAccess::Timescale,
        LAccess::Rotation,
        LAccess::X,
        LAccess::Y,
        LAccess::VelocityX,
        LAccess::VelocityY,
        LAccess::ShootX,
        LAccess::ShootY,
        LAccess::CameraX,
        LAccess::CameraY,
        LAccess::CameraWidth,
        LAccess::CameraHeight,
        LAccess::DisplayWidth,
        LAccess::DisplayHeight,
        LAccess::BufferSize,
        LAccess::Operations,
        LAccess::Size,
        LAccess::Solid,
        LAccess::Dead,
        LAccess::Range,
        LAccess::Shooting,
        LAccess::Boosting,
        LAccess::MineX,
        LAccess::MineY,
        LAccess::Mining,
        LAccess::BuildX,
        LAccess::BuildY,
        LAccess::PingX,
        LAccess::PingY,
        LAccess::PingText,
        LAccess::Building,
        LAccess::Breaking,
        LAccess::Speed,
        LAccess::Team,
        LAccess::Type,
        LAccess::Flag,
        LAccess::Controlled,
        LAccess::Controller,
        LAccess::Name,
        LAccess::PayloadCount,
        LAccess::PayloadType,
        LAccess::TotalPayload,
        LAccess::PayloadCapacity,
        LAccess::MaxUnits,
        LAccess::Id,
        LAccess::SelectedBlock,
        LAccess::SelectedRotation,
        LAccess::BulletLifetime,
        LAccess::BulletTime,
        LAccess::Enabled,
        LAccess::Config,
        LAccess::Color,
    ];

    pub const CONTROLS: [LAccess; 5] = [
        LAccess::Enabled,
        LAccess::Shoot,
        LAccess::Shootp,
        LAccess::Config,
        LAccess::Color,
    ];

    pub const SETTABLE: [LAccess; 15] = [
        LAccess::X,
        LAccess::Y,
        LAccess::VelocityX,
        LAccess::VelocityY,
        LAccess::Rotation,
        LAccess::Speed,
        LAccess::Armor,
        LAccess::Health,
        LAccess::Shield,
        LAccess::Team,
        LAccess::Flag,
        LAccess::TotalPower,
        LAccess::PayloadType,
        LAccess::BulletTime,
        LAccess::BulletLifetime,
    ];

    pub const WIRE_NAMES: [&'static str; 76] = [
        "totalItems",
        "firstItem",
        "totalLiquids",
        "totalPower",
        "itemCapacity",
        "liquidCapacity",
        "powerCapacity",
        "powerNetStored",
        "powerNetCapacity",
        "powerNetIn",
        "powerNetOut",
        "ammo",
        "ammoCapacity",
        "currentAmmoType",
        "memoryCapacity",
        "health",
        "maxHealth",
        "heat",
        "shield",
        "armor",
        "efficiency",
        "progress",
        "timescale",
        "rotation",
        "x",
        "y",
        "velocityX",
        "velocityY",
        "shootX",
        "shootY",
        "cameraX",
        "cameraY",
        "cameraWidth",
        "cameraHeight",
        "displayWidth",
        "displayHeight",
        "bufferSize",
        "operations",
        "size",
        "solid",
        "dead",
        "range",
        "shooting",
        "boosting",
        "mineX",
        "mineY",
        "mining",
        "buildX",
        "buildY",
        "pingX",
        "pingY",
        "pingText",
        "building",
        "breaking",
        "speed",
        "team",
        "type",
        "flag",
        "controlled",
        "controller",
        "name",
        "payloadCount",
        "payloadType",
        "totalPayload",
        "payloadCapacity",
        "maxUnits",
        "id",
        "selectedBlock",
        "selectedRotation",
        "bulletLifetime",
        "bulletTime",
        "enabled",
        "shoot",
        "shootp",
        "config",
        "color",
    ];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }

    pub const fn params(self) -> &'static [&'static str] {
        match self {
            LAccess::Enabled => &["to"],
            LAccess::Shoot => &["x", "y", "shoot"],
            LAccess::Shootp => &["unit", "shoot"],
            LAccess::Config => &["to"],
            LAccess::Color => &["to"],
            _ => &[],
        }
    }

    pub const fn is_obj(self) -> bool {
        matches!(self, LAccess::Shootp | LAccess::Config)
    }

    pub const fn is_senseable(self) -> bool {
        self.params().len() <= 1
    }

    pub const fn is_control(self) -> bool {
        !self.params().is_empty()
    }
}

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

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn symbol(self) -> &'static str {
        Self::SYMBOLS[self.ordinal() as usize]
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

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn symbol(self) -> &'static str {
        Self::SYMBOLS[self.ordinal() as usize]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarUnitView {
    pub x: f32,
    pub y: f32,
    pub health: f32,
    pub shield: f32,
    pub armor: f32,
    pub max_health: f32,
    pub team: u8,
    pub is_player: bool,
    pub can_shoot: bool,
    pub is_flying: bool,
    pub is_boss: bool,
    pub is_grounded: bool,
}

impl RadarUnitView {
    pub const fn new(x: f32, y: f32, team: u8) -> Self {
        Self {
            x,
            y,
            health: 0.0,
            shield: 0.0,
            armor: 0.0,
            max_health: 0.0,
            team,
            is_player: false,
            can_shoot: false,
            is_flying: false,
            is_boss: false,
            is_grounded: false,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadarSort {
    Distance,
    Health,
    Shield,
    Armor,
    MaxHealth,
}

impl RadarSort {
    pub const ALL: [RadarSort; 5] = [
        RadarSort::Distance,
        RadarSort::Health,
        RadarSort::Shield,
        RadarSort::Armor,
        RadarSort::MaxHealth,
    ];

    pub const WIRE_NAMES: [&'static str; 5] =
        ["distance", "health", "shield", "armor", "maxHealth"];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }

    pub fn score(self, origin_x: f32, origin_y: f32, other: &RadarUnitView) -> f32 {
        match self {
            RadarSort::Distance => {
                let dx = origin_x - other.x;
                let dy = origin_y - other.y;
                -(dx * dx + dy * dy)
            }
            RadarSort::Health => other.health,
            RadarSort::Shield => other.shield,
            RadarSort::Armor => other.armor,
            RadarSort::MaxHealth => other.max_health,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadarTarget {
    Any,
    Enemy,
    Ally,
    Player,
    Attacker,
    Flying,
    Boss,
    Ground,
}

impl RadarTarget {
    /// Upstream Team.derelict has id 0.
    pub const DERELICT_TEAM: u8 = 0;

    pub const ALL: [RadarTarget; 8] = [
        RadarTarget::Any,
        RadarTarget::Enemy,
        RadarTarget::Ally,
        RadarTarget::Player,
        RadarTarget::Attacker,
        RadarTarget::Flying,
        RadarTarget::Boss,
        RadarTarget::Ground,
    ];

    pub const WIRE_NAMES: [&'static str; 8] = [
        "any", "enemy", "ally", "player", "attacker", "flying", "boss", "ground",
    ];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }

    pub fn matches(self, team: u8, other: &RadarUnitView) -> bool {
        match self {
            RadarTarget::Any => true,
            RadarTarget::Enemy => team != other.team && other.team != Self::DERELICT_TEAM,
            RadarTarget::Ally => team == other.team,
            RadarTarget::Player => other.is_player,
            RadarTarget::Attacker => other.can_shoot,
            RadarTarget::Flying => other.is_flying,
            RadarTarget::Boss => other.is_boss,
            RadarTarget::Ground => other.is_grounded,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicRule {
    CurrentWaveTime,
    WaveTimer,
    Waves,
    Wave,
    WaveSpacing,
    WaveSending,
    AttackMode,
    EnemyCoreBuildRadius,
    DropZoneRadius,
    UnitCap,
    MapArea,
    Lighting,
    CanGameOver,
    AmbientLight,
    SolarMultiplier,
    DragMultiplier,
    Ban,
    Unban,
    PauseDisabled,
    BuildSpeed,
    UnitHealth,
    UnitBuildSpeed,
    UnitMineSpeed,
    UnitCost,
    UnitDamage,
    BlockHealth,
    BlockDamage,
    RtsMinWeight,
    RtsMinSquad,
}

impl LogicRule {
    pub const ALL: [LogicRule; 29] = [
        LogicRule::CurrentWaveTime,
        LogicRule::WaveTimer,
        LogicRule::Waves,
        LogicRule::Wave,
        LogicRule::WaveSpacing,
        LogicRule::WaveSending,
        LogicRule::AttackMode,
        LogicRule::EnemyCoreBuildRadius,
        LogicRule::DropZoneRadius,
        LogicRule::UnitCap,
        LogicRule::MapArea,
        LogicRule::Lighting,
        LogicRule::CanGameOver,
        LogicRule::AmbientLight,
        LogicRule::SolarMultiplier,
        LogicRule::DragMultiplier,
        LogicRule::Ban,
        LogicRule::Unban,
        LogicRule::PauseDisabled,
        LogicRule::BuildSpeed,
        LogicRule::UnitHealth,
        LogicRule::UnitBuildSpeed,
        LogicRule::UnitMineSpeed,
        LogicRule::UnitCost,
        LogicRule::UnitDamage,
        LogicRule::BlockHealth,
        LogicRule::BlockDamage,
        LogicRule::RtsMinWeight,
        LogicRule::RtsMinSquad,
    ];

    pub const WIRE_NAMES: [&'static str; 29] = [
        "currentWaveTime",
        "waveTimer",
        "waves",
        "wave",
        "waveSpacing",
        "waveSending",
        "attackMode",
        "enemyCoreBuildRadius",
        "dropZoneRadius",
        "unitCap",
        "mapArea",
        "lighting",
        "canGameOver",
        "ambientLight",
        "solarMultiplier",
        "dragMultiplier",
        "ban",
        "unban",
        "pauseDisabled",
        "buildSpeed",
        "unitHealth",
        "unitBuildSpeed",
        "unitMineSpeed",
        "unitCost",
        "unitDamage",
        "blockHealth",
        "blockDamage",
        "rtsMinWeight",
        "rtsMinSquad",
    ];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FetchType {
    Unit,
    UnitCount,
    Player,
    PlayerCount,
    Core,
    CoreCount,
    Build,
    BuildCount,
}

impl FetchType {
    pub const ALL: [FetchType; 8] = [
        FetchType::Unit,
        FetchType::UnitCount,
        FetchType::Player,
        FetchType::PlayerCount,
        FetchType::Core,
        FetchType::CoreCount,
        FetchType::Build,
        FetchType::BuildCount,
    ];

    pub const WIRE_NAMES: [&'static str; 8] = [
        "unit",
        "unitCount",
        "player",
        "playerCount",
        "core",
        "coreCount",
        "build",
        "buildCount",
    ];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryType {
    Unit,
    Building,
    Bullet,
}

impl QueryType {
    pub const ALL: [QueryType; 3] = [QueryType::Unit, QueryType::Building, QueryType::Bullet];
    pub const QUERYABLE: [QueryType; 2] = [QueryType::Unit, QueryType::Building];
    pub const WIRE_NAMES: [&'static str; 3] = ["unit", "building", "bullet"];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryShape {
    Circle,
    Rect,
}

impl QueryShape {
    pub const ALL: [QueryShape; 2] = [QueryShape::Circle, QueryShape::Rect];
    pub const WIRE_NAMES: [&'static str; 2] = ["circle", "rect"];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageType {
    Notify,
    Announce,
    Toast,
    Mission,
}

impl MessageType {
    pub const ALL: [MessageType; 4] = [
        MessageType::Notify,
        MessageType::Announce,
        MessageType::Toast,
        MessageType::Mission,
    ];
    pub const WIRE_NAMES: [&'static str; 4] = ["notify", "announce", "toast", "mission"];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CutsceneAction {
    Pan,
    Zoom,
    Stop,
}

impl CutsceneAction {
    pub const ALL: [CutsceneAction; 3] = [
        CutsceneAction::Pan,
        CutsceneAction::Zoom,
        CutsceneAction::Stop,
    ];
    pub const WIRE_NAMES: [&'static str; 3] = ["pan", "zoom", "stop"];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileLayer {
    Floor,
    Ore,
    Block,
    Building,
}

impl TileLayer {
    pub const ALL: [TileLayer; 4] = [
        TileLayer::Floor,
        TileLayer::Ore,
        TileLayer::Block,
        TileLayer::Building,
    ];
    pub const SETTABLE: [TileLayer; 3] = [TileLayer::Floor, TileLayer::Ore, TileLayer::Block];
    pub const WIRE_NAMES: [&'static str; 4] = ["floor", "ore", "block", "building"];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }

    pub const fn is_settable(self) -> bool {
        matches!(self, TileLayer::Floor | TileLayer::Ore | TileLayer::Block)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LUnitControl {
    Idle,
    Stop,
    Move,
    Approach,
    Pathfind,
    AutoPathfind,
    Boost,
    Target,
    Targetp,
    ItemDrop,
    ItemTake,
    PayDrop,
    PayTake,
    PayEnter,
    Mine,
    Flag,
    Build,
    Deconstruct,
    GetBlock,
    Within,
    Unbind,
}

impl LUnitControl {
    pub const ALL: [LUnitControl; 21] = [
        LUnitControl::Idle,
        LUnitControl::Stop,
        LUnitControl::Move,
        LUnitControl::Approach,
        LUnitControl::Pathfind,
        LUnitControl::AutoPathfind,
        LUnitControl::Boost,
        LUnitControl::Target,
        LUnitControl::Targetp,
        LUnitControl::ItemDrop,
        LUnitControl::ItemTake,
        LUnitControl::PayDrop,
        LUnitControl::PayTake,
        LUnitControl::PayEnter,
        LUnitControl::Mine,
        LUnitControl::Flag,
        LUnitControl::Build,
        LUnitControl::Deconstruct,
        LUnitControl::GetBlock,
        LUnitControl::Within,
        LUnitControl::Unbind,
    ];

    pub const WIRE_NAMES: [&'static str; 21] = [
        "idle",
        "stop",
        "move",
        "approach",
        "pathfind",
        "autoPathfind",
        "boost",
        "target",
        "targetp",
        "itemDrop",
        "itemTake",
        "payDrop",
        "payTake",
        "payEnter",
        "mine",
        "flag",
        "build",
        "deconstruct",
        "getBlock",
        "within",
        "unbind",
    ];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }

    pub const fn params(self) -> &'static [&'static str] {
        match self {
            LUnitControl::Move => &["x", "y"],
            LUnitControl::Approach => &["x", "y", "radius"],
            LUnitControl::Pathfind => &["x", "y"],
            LUnitControl::Boost => &["enable"],
            LUnitControl::Target => &["x", "y", "shoot"],
            LUnitControl::Targetp => &["unit", "shoot"],
            LUnitControl::ItemDrop => &["to", "amount"],
            LUnitControl::ItemTake => &["from", "item", "amount"],
            LUnitControl::PayTake => &["takeUnits"],
            LUnitControl::Mine => &["x", "y"],
            LUnitControl::Flag => &["value"],
            LUnitControl::Build => &["x", "y", "block", "rotation", "config"],
            LUnitControl::Deconstruct => &["x", "y"],
            LUnitControl::GetBlock => &["x", "y", "type", "building", "floor"],
            LUnitControl::Within => &["x", "y", "radius", "result"],
            _ => &[],
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LLocate {
    Ore,
    Building,
    Spawn,
    Damaged,
}

impl LLocate {
    pub const ALL: [LLocate; 4] = [
        LLocate::Ore,
        LLocate::Building,
        LLocate::Spawn,
        LLocate::Damaged,
    ];
    pub const WIRE_NAMES: [&'static str; 4] = ["ore", "building", "spawn", "damaged"];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LCategory {
    pub id: u8,
    pub name: &'static str,
    pub color_rgba: u32,
    pub icon: Option<&'static str>,
}

impl LCategory {
    pub const ALL: [LCategory; 7] = [
        LCategory {
            id: 0,
            name: "unknown",
            color_rgba: 0x4c4c4cff,
            icon: None,
        },
        LCategory {
            id: 1,
            name: "io",
            color_rgba: 0xa08a8aff,
            icon: Some("logicSmall"),
        },
        LCategory {
            id: 2,
            name: "block",
            color_rgba: 0xd4816bff,
            icon: Some("effectSmall"),
        },
        LCategory {
            id: 3,
            name: "operation",
            color_rgba: 0x877badff,
            icon: Some("settingsSmall"),
        },
        LCategory {
            id: 4,
            name: "control",
            color_rgba: 0x6bb2b2ff,
            icon: Some("rotateSmall"),
        },
        LCategory {
            id: 5,
            name: "unit",
            color_rgba: 0xc7b59dff,
            icon: Some("unitsSmall"),
        },
        LCategory {
            id: 6,
            name: "world",
            color_rgba: 0x6b84d4ff,
            icon: Some("terrainSmall"),
        },
    ];

    pub fn by_name(name: &str) -> Option<&'static LCategory> {
        Self::ALL.iter().find(|category| category.name == name)
    }

    pub fn localized_key(self) -> String {
        format!("lcategory.{}", self.name)
    }

    pub fn description_key(self) -> String {
        format!("lcategory.{}.description", self.name)
    }
}

pub trait Controllable {
    type Object;

    fn control(&mut self, access: LAccess, p1: f64, p2: f64, p3: f64, p4: f64);
    fn control_object(&mut self, access: LAccess, p1: Self::Object, p2: f64, p3: f64, p4: f64);
    fn team(&self) -> u8;
}

pub trait LogicReadable<E, V> {
    fn readable(&self, exec: &E) -> bool;
    fn read(&self, position: &mut V, output: &mut V);
}

pub trait LogicWritable<E, V> {
    fn writable(&self, exec: &E) -> bool;
    fn write(&mut self, position: &mut V, value: &mut V);
}

pub trait Ranged {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn team(&self) -> u8;
    fn range(&self) -> f32;
}

pub trait Senseable {
    type Content;
    type Object;

    fn sense(&self, sensor: LAccess) -> f64;

    fn sense_content(&self, _content: &Self::Content) -> f64 {
        0.0
    }

    fn sense_object(&self, _sensor: LAccess) -> Option<Self::Object> {
        None
    }
}

pub trait Settable {
    type Content;
    type Object;

    fn set_prop(&mut self, prop: LAccess, value: f64);
    fn set_prop_object(&mut self, prop: LAccess, value: Self::Object);
    fn set_content_prop(&mut self, content: Self::Content, value: f64);
}

/// Mirrors upstream `mindustry.logic.LMarkerControl`.
///
/// The declaration order is network-visible: Java `TypeIO.writeMarkerControl`
/// writes the enum ordinal as a single byte and `readMarkerControl` indexes
/// `LMarkerControl.all` with an unsigned byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LMarkerControl {
    Remove,
    World,
    Minimap,
    Autoscale,
    Pos,
    EndPos,
    DrawLayer,
    Color,
    Radius,
    Stroke,
    Outline,
    Rotation,
    Shape,
    Arc,
    FlushText,
    FontSize,
    TextHeight,
    TextAlign,
    LineAlign,
    LabelFlags,
    Texture,
    TextureSize,
    Posi,
    Uvi,
    Colori,
}

impl LMarkerControl {
    pub const ALL: [LMarkerControl; 25] = [
        LMarkerControl::Remove,
        LMarkerControl::World,
        LMarkerControl::Minimap,
        LMarkerControl::Autoscale,
        LMarkerControl::Pos,
        LMarkerControl::EndPos,
        LMarkerControl::DrawLayer,
        LMarkerControl::Color,
        LMarkerControl::Radius,
        LMarkerControl::Stroke,
        LMarkerControl::Outline,
        LMarkerControl::Rotation,
        LMarkerControl::Shape,
        LMarkerControl::Arc,
        LMarkerControl::FlushText,
        LMarkerControl::FontSize,
        LMarkerControl::TextHeight,
        LMarkerControl::TextAlign,
        LMarkerControl::LineAlign,
        LMarkerControl::LabelFlags,
        LMarkerControl::Texture,
        LMarkerControl::TextureSize,
        LMarkerControl::Posi,
        LMarkerControl::Uvi,
        LMarkerControl::Colori,
    ];

    pub const fn ordinal(self) -> u8 {
        match self {
            LMarkerControl::Remove => 0,
            LMarkerControl::World => 1,
            LMarkerControl::Minimap => 2,
            LMarkerControl::Autoscale => 3,
            LMarkerControl::Pos => 4,
            LMarkerControl::EndPos => 5,
            LMarkerControl::DrawLayer => 6,
            LMarkerControl::Color => 7,
            LMarkerControl::Radius => 8,
            LMarkerControl::Stroke => 9,
            LMarkerControl::Outline => 10,
            LMarkerControl::Rotation => 11,
            LMarkerControl::Shape => 12,
            LMarkerControl::Arc => 13,
            LMarkerControl::FlushText => 14,
            LMarkerControl::FontSize => 15,
            LMarkerControl::TextHeight => 16,
            LMarkerControl::TextAlign => 17,
            LMarkerControl::LineAlign => 18,
            LMarkerControl::LabelFlags => 19,
            LMarkerControl::Texture => 20,
            LMarkerControl::TextureSize => 21,
            LMarkerControl::Posi => 22,
            LMarkerControl::Uvi => 23,
            LMarkerControl::Colori => 24,
        }
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub const fn wire_name(self) -> &'static str {
        match self {
            LMarkerControl::Remove => "remove",
            LMarkerControl::World => "world",
            LMarkerControl::Minimap => "minimap",
            LMarkerControl::Autoscale => "autoscale",
            LMarkerControl::Pos => "pos",
            LMarkerControl::EndPos => "endPos",
            LMarkerControl::DrawLayer => "drawLayer",
            LMarkerControl::Color => "color",
            LMarkerControl::Radius => "radius",
            LMarkerControl::Stroke => "stroke",
            LMarkerControl::Outline => "outline",
            LMarkerControl::Rotation => "rotation",
            LMarkerControl::Shape => "shape",
            LMarkerControl::Arc => "arc",
            LMarkerControl::FlushText => "flushText",
            LMarkerControl::FontSize => "fontSize",
            LMarkerControl::TextHeight => "textHeight",
            LMarkerControl::TextAlign => "textAlign",
            LMarkerControl::LineAlign => "lineAlign",
            LMarkerControl::LabelFlags => "labelFlags",
            LMarkerControl::Texture => "texture",
            LMarkerControl::TextureSize => "textureSize",
            LMarkerControl::Posi => "posi",
            LMarkerControl::Uvi => "uvi",
            LMarkerControl::Colori => "colori",
        }
    }

    pub const fn params(self) -> &'static [&'static str] {
        match self {
            LMarkerControl::Remove => &[],
            LMarkerControl::World => &["true/false"],
            LMarkerControl::Minimap => &["true/false"],
            LMarkerControl::Autoscale => &["true/false"],
            LMarkerControl::Pos => &["x", "y"],
            LMarkerControl::EndPos => &["x", "y"],
            LMarkerControl::DrawLayer => &["layer"],
            LMarkerControl::Color => &["color"],
            LMarkerControl::Radius => &["radius"],
            LMarkerControl::Stroke => &["stroke"],
            LMarkerControl::Outline => &["outline"],
            LMarkerControl::Rotation => &["rotation"],
            LMarkerControl::Shape => &["sides", "fill", "outline"],
            LMarkerControl::Arc => &["start", "end"],
            LMarkerControl::FlushText => &["fetch"],
            LMarkerControl::FontSize => &["size"],
            LMarkerControl::TextHeight => &["height"],
            LMarkerControl::TextAlign => &["align"],
            LMarkerControl::LineAlign => &["align"],
            LMarkerControl::LabelFlags => &["background", "outline"],
            LMarkerControl::Texture => &["printFlush", "name"],
            LMarkerControl::TextureSize => &["width", "height"],
            LMarkerControl::Posi => &["index", "x", "y"],
            LMarkerControl::Uvi => &["index", "x", "y"],
            LMarkerControl::Colori => &["index", "color"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn parser_tokenizes_comments_strings_labels_and_legacy_names_like_java() {
        let parsed = parse_logic_statements(
            "start:\n\
             op atan2 result x y; op dst d x y\n\
             jump start equal a b\n\
             set message \"hello world\" # trailing comment\n\
             control configure block 1\n\
             sensor @configure cell enabled\n",
        )
        .unwrap();

        assert_eq!(parsed.jump_locations.get("start"), Some(&0));
        assert_eq!(
            parsed.statements[0],
            LogicStatementKind::Label {
                name: "start".into(),
                line: 0
            }
        );
        assert_eq!(
            parsed.statements[1],
            LogicStatementKind::Instruction {
                tokens: vec!["op", "angle", "result", "x", "y"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                line: 0,
                jump_label: None
            }
        );
        assert_eq!(
            parsed.statements[2],
            LogicStatementKind::Instruction {
                tokens: vec!["op", "len", "d", "x", "y"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                line: 1,
                jump_label: None
            }
        );
        assert_eq!(
            parsed.statements[3],
            LogicStatementKind::Instruction {
                tokens: vec!["jump", "-1", "equal", "a", "b"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                line: 2,
                jump_label: Some("start".into())
            }
        );
        assert_eq!(
            parsed.statements[4],
            LogicStatementKind::Instruction {
                tokens: vec!["set", "message", "\"hello world\""]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                line: 3,
                jump_label: None
            }
        );
        assert_eq!(
            parsed.statements[5],
            LogicStatementKind::Instruction {
                tokens: vec!["control", "config", "block", "1"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                line: 4,
                jump_label: None
            }
        );
        assert_eq!(
            parsed.statements[6],
            LogicStatementKind::Instruction {
                tokens: vec!["sensor", "@config", "cell", "enabled"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                line: 5,
                jump_label: None
            }
        );
    }

    #[test]
    fn parser_reports_java_style_string_and_token_errors() {
        assert!(parse_logic_statements("set a \"unterminated")
            .unwrap_err()
            .message
            .contains("Missing closing quote \" before end of file."));
        assert!(parse_logic_statements("set a \"unterminated\n")
            .unwrap_err()
            .message
            .contains("Missing closing quote \" before end of line."));
        assert!(parse_logic_statements("set a \"ok\"next")
            .unwrap_err()
            .message
            .contains("Expected space after string/token."));

        let too_long = format!("print \"{}\"", "a".repeat(65_536));
        assert!(parse_logic_statements(&too_long)
            .unwrap_err()
            .message
            .contains("String value too long."));

        let many_tokens = (0..17)
            .map(|i| format!("t{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        assert!(parse_logic_statements(&many_tokens)
            .unwrap_err()
            .message
            .contains("Line too long"));
    }

    #[test]
    fn statement_sanitize_and_align_tables_match_java_helpers() {
        assert_eq!(sanitize_logic_value(""), "");
        assert_eq!(sanitize_logic_value("\""), "invalid");
        assert_eq!(sanitize_logic_value(";"), "invalid");
        assert_eq!(sanitize_logic_value(" "), "invalid");
        assert_eq!(sanitize_logic_value("a b;c\"d"), "a_bsc'd");
        assert_eq!(sanitize_logic_value("\"a\"b;c\""), "\"a'b;c\"");

        assert_eq!(
            LogicAlign::ALL
                .iter()
                .map(|align| align.wire_name())
                .collect::<Vec<_>>(),
            vec![
                "topLeft",
                "top",
                "topRight",
                "left",
                "center",
                "right",
                "bottomLeft",
                "bottom",
                "bottomRight"
            ]
        );
        assert_eq!(LogicAlign::Center.java_bits(), 1);
        assert_eq!(LogicAlign::Top.java_bits(), 2);
        assert_eq!(LogicAlign::Bottom.java_bits(), 4);
        assert_eq!(LogicAlign::Left.java_bits(), 8);
        assert_eq!(LogicAlign::Right.java_bits(), 16);
        assert_eq!(LogicAlign::TopLeft.java_bits(), 10);
        assert_eq!(LogicAlign::BottomRight.java_bits(), 20);
        assert_eq!(LogicAlign::by_name("topRight"), Some(LogicAlign::TopRight));
        assert_eq!(LogicAlign::by_java_bits(12), Some(LogicAlign::BottomLeft));
        assert!(LogicAlign::Top.is_center_horizontal());
        assert!(!LogicAlign::TopLeft.is_center_horizontal());
        assert!(LogicAlign::Left.is_center_vertical());
        assert!(!LogicAlign::BottomLeft.is_center_vertical());
    }

    #[test]
    fn logic_fx_registry_matches_java_order_flags_and_extension_semantics() {
        assert_eq!(LOGIC_EFFECTS.len(), 33);
        assert_eq!(
            logic_effect_names(),
            vec![
                "warn",
                "cross",
                "blockFall",
                "placeBlock",
                "placeBlockSpark",
                "breakBlock",
                "spawn",
                "trail",
                "breakProp",
                "smokeCloud",
                "vapor",
                "hit",
                "hitSquare",
                "shootSmall",
                "shootBig",
                "smokeSmall",
                "smokeBig",
                "smokeColor",
                "smokeSquare",
                "smokeSquareBig",
                "spark",
                "sparkBig",
                "sparkShoot",
                "sparkShootBig",
                "drill",
                "drillBig",
                "lightBlock",
                "explosion",
                "smokePuff",
                "sparkExplosion",
                "crossExplosion",
                "wave",
                "bubble"
            ]
        );

        let block_fall = get_logic_effect("blockFall").unwrap();
        assert_eq!(block_fall.effect, "blockCrash");
        assert_eq!(block_fall.data, Some("Block"));
        assert_eq!(block_fall.bounds, 100.0);
        assert!(!block_fall.size);
        assert!(!block_fall.rotate);
        assert!(!block_fall.color);

        let trail = get_logic_effect("trail").unwrap();
        assert_eq!(trail.effect, "colorTrail");
        assert!(trail.size);
        assert!(trail.color);
        assert!(!trail.rotate);

        let shoot_big = get_logic_effect("shootBig").unwrap();
        assert_eq!(shoot_big.effect, "shootTitan");
        assert!(shoot_big.rotate);
        assert!(shoot_big.color);
        assert!(!shoot_big.size);

        let wave = get_logic_effect("wave").unwrap();
        assert_eq!(wave.effect, "dynamicWave");
        assert!(wave.size);
        assert!(wave.color);

        assert_eq!(get_logic_effect("missing"), None);

        let mut registry = LogicEffectRegistry::new();
        assert_eq!(registry.all().first(), Some(&"warn"));
        assert_eq!(registry.all().last(), Some(&"bubble"));
        registry.add(
            "custom",
            LogicEffectEntry::new("ignored", "customFx")
                .size()
                .rotate()
                .color()
                .bounds(42.0),
        );
        let custom = registry.get("custom").unwrap();
        assert_eq!(custom.name, "custom");
        assert_eq!(custom.effect, "customFx");
        assert!(custom.size && custom.rotate && custom.color);
        assert_eq!(custom.bounds, 42.0);
        assert_eq!(registry.all().last(), Some(&"custom"));

        registry.add("warn", LogicEffectEntry::new("ignored", "replacement"));
        assert_eq!(registry.get("warn").unwrap().name, "warn");
        assert_eq!(registry.get("warn").unwrap().effect, "replacement");
        assert_eq!(registry.all().first(), Some(&"warn"));
    }

    #[test]
    fn logic_canvas_rows_and_jump_normalization_match_lcanvas_helpers() {
        assert!(logic_canvas_use_rows(1079.9, 1.0));
        assert!(!logic_canvas_use_rows(1080.0, 1.0));
        assert!(logic_canvas_use_rows(2159.0, 2.0));
        assert!(!logic_canvas_use_rows(2160.0, 2.0));

        assert_eq!(
            normalize_logic_jump_range(3, Some(8)),
            Some(LogicJumpRange {
                begin: 3,
                end: 8,
                flipped: false
            })
        );
        assert_eq!(
            normalize_logic_jump_range(8, Some(3)),
            Some(LogicJumpRange {
                begin: 3,
                end: 8,
                flipped: true
            })
        );
        assert_eq!(
            normalize_logic_jump_range(5, Some(5)),
            Some(LogicJumpRange {
                begin: 5,
                end: 5,
                flipped: true
            })
        );
        assert_eq!(normalize_logic_jump_range(1, None), None);
        assert_eq!(
            LogicJumpRange::invalid(),
            LogicJumpRange {
                begin: LOGIC_CANVAS_INVALID_JUMP,
                end: LOGIC_CANVAS_INVALID_JUMP,
                flipped: false
            }
        );
    }

    #[test]
    fn logic_canvas_jump_height_assignment_matches_lcanvas_dedup_and_layering() {
        let single = vec![normalize_logic_jump_range(0, Some(3))];
        assert_eq!(
            assign_logic_jump_heights(&single),
            vec![Some(LogicJumpPlacement {
                begin: 0,
                end: 3,
                flipped: false,
                pred_height: 0
            })]
        );

        let disjoint = vec![
            normalize_logic_jump_range(0, Some(2)),
            normalize_logic_jump_range(3, Some(5)),
        ];
        assert_eq!(
            assign_logic_jump_heights(&disjoint)
                .into_iter()
                .map(|placement| placement.unwrap().pred_height)
                .collect::<Vec<_>>(),
            vec![0, 0]
        );

        let nested = vec![
            normalize_logic_jump_range(0, Some(5)),
            normalize_logic_jump_range(1, Some(4)),
        ];
        let nested_heights = assign_logic_jump_heights(&nested)
            .into_iter()
            .map(|placement| placement.unwrap().pred_height)
            .collect::<Vec<_>>();
        assert_eq!(nested_heights, vec![1, 0]);

        // Java reprBefore de-duplicates forward curves by end index, keeping the
        // representative with the smallest begin. Both curves then copy that
        // representative height back.
        let same_forward_end = vec![
            normalize_logic_jump_range(2, Some(6)),
            normalize_logic_jump_range(1, Some(6)),
        ];
        assert_eq!(representative_logic_jumps(&same_forward_end), vec![1]);
        assert_eq!(
            assign_logic_jump_heights(&same_forward_end)
                .into_iter()
                .map(|placement| placement.unwrap().pred_height)
                .collect::<Vec<_>>(),
            vec![0, 0]
        );

        // Java reprAfter de-duplicates backward curves by begin index, keeping the
        // representative with the largest end.
        let same_backward_begin = vec![
            normalize_logic_jump_range(6, Some(2)),
            normalize_logic_jump_range(7, Some(2)),
        ];
        assert_eq!(representative_logic_jumps(&same_backward_begin), vec![1]);
        assert_eq!(
            assign_logic_jump_heights(&same_backward_begin)
                .into_iter()
                .map(|placement| placement.unwrap().pred_height)
                .collect::<Vec<_>>(),
            vec![0, 0]
        );

        let mixed = vec![
            normalize_logic_jump_range(0, Some(4)),
            normalize_logic_jump_range(4, Some(0)),
            None,
            normalize_logic_jump_range(1, Some(3)),
        ];
        let placements = assign_logic_jump_heights(&mixed);
        assert_eq!(placements[2], None);
        assert_eq!(placements[0].unwrap().pred_height, 2);
        assert_eq!(placements[1].unwrap().pred_height, 1);
        assert_eq!(placements[3].unwrap().pred_height, 0);
        assert_eq!(placements[1].unwrap().flipped, true);
    }

    #[test]
    fn lvar_matches_java_numeric_object_and_constant_semantics() {
        assert!(LVar::invalid(f64::NAN));
        assert!(LVar::invalid(f64::INFINITY));
        assert!(!LVar::invalid(42.0));

        let mut var = LVar::with_id("value", 7);
        assert_eq!(var.id, 7);
        assert_eq!(var.num(), 0.0);
        assert!(!var.bool());
        assert_eq!(var.numi(), 0);

        var.set_num(0.000001);
        assert_eq!(var.num(), 0.000001);
        assert!(!var.bool());
        var.set_num(0.00001);
        assert!(var.bool());

        var.set_num(f64::NAN);
        assert!(var.is_obj);
        assert_eq!(var.obj(), None);
        assert_eq!(var.num(), 0.0);
        assert!(var.num_or_nan().is_nan());
        assert_eq!(var.numf(), 0.0);
        assert!(var.numf_or_nan().is_nan());

        var.set_obj(Some("core".into()));
        assert_eq!(var.obj(), Some("core"));
        assert_eq!(var.num(), 1.0);
        assert!(var.bool());
        assert_eq!(var.value(), LVarValue::Object(Some("core".into())));

        var.set_bool(false);
        assert!(!var.is_obj);
        assert_eq!(var.numval, 0.0);
        assert_eq!(var.value(), LVarValue::Number(0.0));

        let mut constant = LVar::with_id_constant("const", 1, true);
        constant.set_num(9.0);
        assert_eq!(constant.numval, 0.0);
        constant.set_obj(Some("ignored".into()));
        assert_eq!(constant.obj(), None);
        constant.set_const_obj(Some("forced".into()));
        assert_eq!(constant.obj(), Some("forced"));
        assert_eq!(constant.to_string(), "const: forced [const]");

        let mut source = LVar::new("source");
        source.set_num(f64::INFINITY);
        let mut target = LVar::new("@counter");
        target.numval = 99.0;
        target.set_from(&source);
        assert!(target.is_obj);
        assert_eq!(target.obj(), None);
        assert_eq!(target.numval, 99.0);

        source.set_obj(None);
        target.numval = 77.0;
        target.set_from(&source);
        assert!(target.is_obj);
        assert_eq!(target.obj(), None);
        assert_eq!(target.numval, 77.0);
        assert_eq!(target.to_string(), "@counter: null");
    }

    #[test]
    fn global_vars_baseline_keeps_java_constants_and_entry_order() {
        assert_eq!(LOGIC_CTRL_PROCESSOR, 1);
        assert_eq!(LOGIC_CTRL_PLAYER, 2);
        assert_eq!(LOGIC_CTRL_COMMAND, 3);
        assert_eq!(
            LOOKABLE_CONTENT,
            ["block", "unit", "item", "liquid", "team"]
        );
        assert_eq!(
            WRITABLE_LOOKABLE_CONTENT,
            ["block", "unit", "item", "liquid"]
        );

        let snapshot = GlobalVarSnapshot::baseline();
        let names = snapshot.names();
        assert_eq!(
            &names[..13],
            [
                "sectionProcessor",
                "@this",
                "@thisx",
                "@thisy",
                "@links",
                "@ipt",
                "sectionGeneral",
                "false",
                "true",
                "@pi",
                "@e",
                "@degToRad",
                "@radToDeg"
            ]
        );
        assert!(names.contains(&"sectionMap"));
        assert!(names.contains(&"sectionNetwork"));
        assert_eq!(names.last(), Some(&"sectionLookup"));

        let public_names = snapshot.visible_to_privileged(false);
        assert!(!public_names.contains(&"@clientLocale"));
        assert!(!public_names.contains(&"@clientUnit"));
        assert!(snapshot
            .visible_to_privileged(true)
            .contains(&"@clientLocale"));
        assert!(
            snapshot
                .entries
                .iter()
                .find(|entry| entry.name == "@server")
                .unwrap()
                .privileged
        );
    }

    #[test]
    fn l_access_matches_java_order_params_and_derived_sets() {
        assert_eq!(LAccess::ALL.len(), 76);
        assert_eq!(LAccess::TotalItems.ordinal(), 0);
        assert_eq!(LAccess::TotalPower.ordinal(), 3);
        assert_eq!(LAccess::Health.ordinal(), 15);
        assert_eq!(LAccess::BulletTime.ordinal(), 70);
        assert_eq!(LAccess::Enabled.ordinal(), 71);
        assert_eq!(LAccess::Color.ordinal(), 75);
        assert_eq!(LAccess::from_ordinal(72), Some(LAccess::Shoot));
        assert_eq!(LAccess::from_ordinal(76), None);

        assert_eq!(
            &LAccess::ALL[LAccess::ALL.len() - 5..],
            [
                LAccess::Enabled,
                LAccess::Shoot,
                LAccess::Shootp,
                LAccess::Config,
                LAccess::Color
            ]
        );
        assert_eq!(LAccess::VelocityX.wire_name(), "velocityX");
        assert_eq!(LAccess::CurrentAmmoType.wire_name(), "currentAmmoType");
        assert_eq!(LAccess::Shoot.params(), ["x", "y", "shoot"]);
        assert_eq!(LAccess::Shootp.params(), ["unit", "shoot"]);
        assert!(LAccess::Shootp.is_obj());
        assert!(LAccess::Config.is_obj());
        assert!(!LAccess::Shoot.is_obj());

        assert_eq!(
            LAccess::SETTABLE,
            [
                LAccess::X,
                LAccess::Y,
                LAccess::VelocityX,
                LAccess::VelocityY,
                LAccess::Rotation,
                LAccess::Speed,
                LAccess::Armor,
                LAccess::Health,
                LAccess::Shield,
                LAccess::Team,
                LAccess::Flag,
                LAccess::TotalPower,
                LAccess::PayloadType,
                LAccess::BulletTime,
                LAccess::BulletLifetime
            ]
        );

        let expected_senseable: Vec<_> = LAccess::ALL
            .iter()
            .copied()
            .filter(|access| access.params().len() <= 1)
            .collect();
        assert_eq!(LAccess::SENSEABLE.as_slice(), expected_senseable.as_slice());
        assert_eq!(
            LAccess::CONTROLS,
            [
                LAccess::Enabled,
                LAccess::Shoot,
                LAccess::Shootp,
                LAccess::Config,
                LAccess::Color
            ]
        );
        assert!(LAccess::Enabled.is_senseable());
        assert!(!LAccess::Shoot.is_senseable());
        assert!(LAccess::Color.is_control());
        assert!(!LAccess::Health.is_control());
    }

    #[test]
    fn logic_ops_match_java_order_symbols_flags_and_core_math() {
        assert_eq!(LogicOp::ALL.len(), 45);
        assert_eq!(LogicOp::Add.ordinal(), 0);
        assert_eq!(LogicOp::StrictEqual.ordinal(), 15);
        assert_eq!(LogicOp::Not.ordinal(), 22);
        assert_eq!(LogicOp::Atan.ordinal(), 44);
        assert_eq!(LogicOp::from_ordinal(44), Some(LogicOp::Atan));
        assert_eq!(LogicOp::from_ordinal(45), None);

        let symbols: Vec<_> = LogicOp::ALL.iter().map(|op| op.symbol()).collect();
        assert_eq!(
            symbols,
            vec![
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
                "atan"
            ]
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
        assert_eq!(LogicOp::Add.to_string(), "+");
    }

    #[test]
    fn condition_ops_match_java_order_symbols_and_tests() {
        assert_eq!(ConditionOp::ALL.len(), 8);
        assert_eq!(ConditionOp::Equal.ordinal(), 0);
        assert_eq!(ConditionOp::StrictEqual.ordinal(), 6);
        assert_eq!(ConditionOp::Always.ordinal(), 7);
        assert_eq!(ConditionOp::from_ordinal(7), Some(ConditionOp::Always));
        assert_eq!(ConditionOp::from_ordinal(8), None);

        let symbols: Vec<_> = ConditionOp::ALL.iter().map(|op| op.symbol()).collect();
        assert_eq!(
            symbols,
            vec!["==", "not", "<", "<=", ">", ">=", "===", "always"]
        );

        assert!(ConditionOp::Equal.test_numbers(1.0, 1.0 + 0.0000005));
        assert!(!ConditionOp::NotEqual.test_numbers(1.0, 1.0 + 0.0000005));
        assert!(ConditionOp::LessThan.test_numbers(1.0, 2.0));
        assert!(ConditionOp::GreaterThanEq.test_numbers(2.0, 2.0));
        assert!(ConditionOp::Always.test_numbers(f64::NAN, f64::NAN));

        assert!(ConditionOp::StrictEqual
            .test_values(ConditionValue::Number(2.0), ConditionValue::Number(2.0)));
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
        assert_eq!(ConditionOp::Always.to_string(), "always");
    }

    #[test]
    fn radar_sort_and_target_match_java_order_and_predicates() {
        assert_eq!(RadarSort::ALL.len(), 5);
        assert_eq!(
            RadarSort::ALL,
            [
                RadarSort::Distance,
                RadarSort::Health,
                RadarSort::Shield,
                RadarSort::Armor,
                RadarSort::MaxHealth
            ]
        );
        assert_eq!(
            RadarSort::ALL
                .iter()
                .map(|sort| sort.wire_name())
                .collect::<Vec<_>>(),
            vec!["distance", "health", "shield", "armor", "maxHealth"]
        );
        assert_eq!(RadarSort::MaxHealth.ordinal(), 4);
        assert_eq!(RadarSort::from_ordinal(5), None);

        let mut unit = RadarUnitView::new(3.0, 4.0, 2);
        unit.health = 10.0;
        unit.shield = 5.0;
        unit.armor = 2.5;
        unit.max_health = 30.0;
        assert_eq!(RadarSort::Distance.score(0.0, 0.0, &unit), -25.0);
        assert_eq!(RadarSort::Health.score(0.0, 0.0, &unit), 10.0);
        assert_eq!(RadarSort::Shield.score(0.0, 0.0, &unit), 5.0);
        assert_eq!(RadarSort::Armor.score(0.0, 0.0, &unit), 2.5);
        assert_eq!(RadarSort::MaxHealth.score(0.0, 0.0, &unit), 30.0);

        assert_eq!(RadarTarget::ALL.len(), 8);
        assert_eq!(
            RadarTarget::ALL,
            [
                RadarTarget::Any,
                RadarTarget::Enemy,
                RadarTarget::Ally,
                RadarTarget::Player,
                RadarTarget::Attacker,
                RadarTarget::Flying,
                RadarTarget::Boss,
                RadarTarget::Ground
            ]
        );
        assert_eq!(
            RadarTarget::ALL
                .iter()
                .map(|target| target.wire_name())
                .collect::<Vec<_>>(),
            vec!["any", "enemy", "ally", "player", "attacker", "flying", "boss", "ground"]
        );
        assert_eq!(RadarTarget::Ground.ordinal(), 7);
        assert_eq!(RadarTarget::from_ordinal(8), None);

        assert!(RadarTarget::Any.matches(1, &unit));
        assert!(RadarTarget::Enemy.matches(1, &unit));
        assert!(!RadarTarget::Enemy.matches(1, &RadarUnitView::new(0.0, 0.0, 0)));
        assert!(RadarTarget::Ally.matches(2, &unit));
        assert!(!RadarTarget::Ally.matches(1, &unit));

        unit.is_player = true;
        unit.can_shoot = true;
        unit.is_flying = true;
        unit.is_boss = true;
        unit.is_grounded = false;
        assert!(RadarTarget::Player.matches(1, &unit));
        assert!(RadarTarget::Attacker.matches(1, &unit));
        assert!(RadarTarget::Flying.matches(1, &unit));
        assert!(RadarTarget::Boss.matches(1, &unit));
        assert!(!RadarTarget::Ground.matches(1, &unit));
    }

    #[test]
    fn pure_logic_enums_match_java_order_and_sets() {
        assert_eq!(LogicRule::ALL.len(), 29);
        assert_eq!(LogicRule::CurrentWaveTime.ordinal(), 0);
        assert_eq!(LogicRule::PauseDisabled.ordinal(), 18);
        assert_eq!(
            &LogicRule::ALL[LogicRule::ALL.len() - 2..],
            [LogicRule::RtsMinWeight, LogicRule::RtsMinSquad]
        );
        assert_eq!(LogicRule::RtsMinWeight.wire_name(), "rtsMinWeight");
        assert_eq!(LogicRule::from_ordinal(29), None);

        assert_eq!(
            FetchType::ALL,
            [
                FetchType::Unit,
                FetchType::UnitCount,
                FetchType::Player,
                FetchType::PlayerCount,
                FetchType::Core,
                FetchType::CoreCount,
                FetchType::Build,
                FetchType::BuildCount
            ]
        );
        assert_eq!(
            FetchType::ALL
                .iter()
                .map(|value| value.wire_name())
                .collect::<Vec<_>>(),
            vec![
                "unit",
                "unitCount",
                "player",
                "playerCount",
                "core",
                "coreCount",
                "build",
                "buildCount"
            ]
        );
        assert_eq!(FetchType::BuildCount.ordinal(), 7);
        assert_eq!(FetchType::from_ordinal(8), None);

        assert_eq!(
            QueryType::ALL,
            [QueryType::Unit, QueryType::Building, QueryType::Bullet]
        );
        assert_eq!(QueryType::QUERYABLE, [QueryType::Unit, QueryType::Building]);
        assert_eq!(QueryType::Bullet.wire_name(), "bullet");
        assert_eq!(QueryType::from_ordinal(3), None);

        assert_eq!(QueryShape::ALL, [QueryShape::Circle, QueryShape::Rect]);
        assert_eq!(
            QueryShape::ALL
                .iter()
                .map(|value| value.wire_name())
                .collect::<Vec<_>>(),
            vec!["circle", "rect"]
        );
        assert_eq!(QueryShape::from_ordinal(2), None);

        assert_eq!(
            MessageType::ALL,
            [
                MessageType::Notify,
                MessageType::Announce,
                MessageType::Toast,
                MessageType::Mission
            ]
        );
        assert_eq!(MessageType::Mission.ordinal(), 3);
        assert_eq!(MessageType::Toast.wire_name(), "toast");

        assert_eq!(
            CutsceneAction::ALL,
            [
                CutsceneAction::Pan,
                CutsceneAction::Zoom,
                CutsceneAction::Stop
            ]
        );
        assert_eq!(CutsceneAction::Stop.wire_name(), "stop");
        assert_eq!(CutsceneAction::from_ordinal(3), None);

        assert_eq!(
            TileLayer::ALL,
            [
                TileLayer::Floor,
                TileLayer::Ore,
                TileLayer::Block,
                TileLayer::Building
            ]
        );
        assert_eq!(
            TileLayer::SETTABLE,
            [TileLayer::Floor, TileLayer::Ore, TileLayer::Block]
        );
        assert!(TileLayer::Floor.is_settable());
        assert!(TileLayer::Block.is_settable());
        assert!(!TileLayer::Building.is_settable());
        assert_eq!(TileLayer::Building.wire_name(), "building");
    }

    #[test]
    fn unit_control_locate_and_categories_match_java_small_logic_files() {
        assert_eq!(LUnitControl::ALL.len(), 21);
        assert_eq!(LUnitControl::Idle.ordinal(), 0);
        assert_eq!(LUnitControl::AutoPathfind.ordinal(), 5);
        assert_eq!(LUnitControl::Unbind.ordinal(), 20);
        assert_eq!(LUnitControl::from_ordinal(21), None);
        assert_eq!(
            LUnitControl::ALL
                .iter()
                .map(|value| value.wire_name())
                .collect::<Vec<_>>(),
            vec![
                "idle",
                "stop",
                "move",
                "approach",
                "pathfind",
                "autoPathfind",
                "boost",
                "target",
                "targetp",
                "itemDrop",
                "itemTake",
                "payDrop",
                "payTake",
                "payEnter",
                "mine",
                "flag",
                "build",
                "deconstruct",
                "getBlock",
                "within",
                "unbind"
            ]
        );
        assert_eq!(LUnitControl::Move.params(), ["x", "y"]);
        assert_eq!(LUnitControl::Targetp.params(), ["unit", "shoot"]);
        assert_eq!(
            LUnitControl::Build.params(),
            ["x", "y", "block", "rotation", "config"]
        );
        assert_eq!(
            LUnitControl::GetBlock.params(),
            ["x", "y", "type", "building", "floor"]
        );
        assert_eq!(
            LUnitControl::Within.params(),
            ["x", "y", "radius", "result"]
        );
        assert_eq!(LUnitControl::PayDrop.params(), [] as [&str; 0]);

        assert_eq!(
            LLocate::ALL,
            [
                LLocate::Ore,
                LLocate::Building,
                LLocate::Spawn,
                LLocate::Damaged
            ]
        );
        assert_eq!(
            LLocate::ALL
                .iter()
                .map(|value| value.wire_name())
                .collect::<Vec<_>>(),
            vec!["ore", "building", "spawn", "damaged"]
        );
        assert_eq!(LLocate::Damaged.ordinal(), 3);
        assert_eq!(LLocate::from_ordinal(4), None);

        assert_eq!(LCategory::ALL.len(), 7);
        assert_eq!(
            LCategory::ALL
                .iter()
                .map(|category| (category.id, category.name, category.icon))
                .collect::<Vec<_>>(),
            vec![
                (0, "unknown", None),
                (1, "io", Some("logicSmall")),
                (2, "block", Some("effectSmall")),
                (3, "operation", Some("settingsSmall")),
                (4, "control", Some("rotateSmall")),
                (5, "unit", Some("unitsSmall")),
                (6, "world", Some("terrainSmall"))
            ]
        );
        assert_eq!(LCategory::ALL[0].color_rgba, 0x4c4c4cff);
        assert_eq!(LCategory::ALL[1].color_rgba, 0xa08a8aff);
        assert_eq!(LCategory::ALL[2].color_rgba, 0xd4816bff);
        assert_eq!(LCategory::ALL[3].color_rgba, 0x877badff);
        assert_eq!(LCategory::ALL[4].color_rgba, 0x6bb2b2ff);
        assert_eq!(LCategory::ALL[5].color_rgba, 0xc7b59dff);
        assert_eq!(LCategory::ALL[6].color_rgba, 0x6b84d4ff);

        let unit = LCategory::by_name("unit").unwrap();
        assert_eq!(unit.id, 5);
        assert_eq!(unit.localized_key(), "lcategory.unit");
        assert_eq!(unit.description_key(), "lcategory.unit.description");
        assert_eq!(LCategory::by_name("missing"), None);
    }

    #[test]
    fn marker_controls_match_java_order_and_params() {
        assert_eq!(LMarkerControl::ALL.len(), 25);
        assert_eq!(LMarkerControl::Remove.ordinal(), 0);
        assert_eq!(LMarkerControl::Shape.ordinal(), 12);
        assert_eq!(LMarkerControl::Colori.ordinal(), 24);
        assert_eq!(
            LMarkerControl::from_ordinal(20).unwrap(),
            LMarkerControl::Texture
        );
        assert_eq!(LMarkerControl::from_ordinal(25), None);
        assert_eq!(LMarkerControl::EndPos.wire_name(), "endPos");
        assert_eq!(LMarkerControl::Shape.params(), ["sides", "fill", "outline"]);
        assert_eq!(LMarkerControl::Texture.params(), ["printFlush", "name"]);
    }
}
