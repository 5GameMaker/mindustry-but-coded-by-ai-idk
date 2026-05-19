// Mirrors upstream core/src/mindustry/logic. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

use crate::mindustry::{content::ContentCatalog, ctype::ContentType, world::meta::BlockFlag};

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub const LOGIC_CTRL_PROCESSOR: i32 = 1;
pub const LOGIC_CTRL_PLAYER: i32 = 2;
pub const LOGIC_CTRL_COMMAND: i32 = 3;
pub const LOOKABLE_CONTENT: [&str; 5] = ["block", "unit", "item", "liquid", "team"];
pub const WRITABLE_LOOKABLE_CONTENT: [&str; 4] = ["block", "unit", "item", "liquid"];
pub const LOOKABLE_CONTENT_TYPES: [ContentType; 5] = [
    ContentType::Block,
    ContentType::Unit,
    ContentType::Item,
    ContentType::Liquid,
    ContentType::Team,
];
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicStatement {
    Invalid,
    Read {
        output: String,
        target: String,
        address: String,
    },
    Write {
        input: String,
        target: String,
        address: String,
    },
    Draw {
        type_: GraphicsType,
        x: String,
        y: String,
        p1: String,
        p2: String,
        p3: String,
        p4: String,
    },
    Print {
        value: String,
    },
    PrintChar {
        value: String,
    },
    Format {
        value: String,
    },
    LocalePrint {
        value: String,
    },
    DrawFlush {
        target: String,
    },
    PrintFlush {
        target: String,
    },
    GetLink {
        output: String,
        address: String,
    },
    SetRate {
        amount: String,
    },
    Sync {
        variable: String,
    },
    Set {
        to: String,
        from: String,
    },
    Operation {
        op: LogicOp,
        dest: String,
        a: String,
        b: String,
    },
    Select {
        result: String,
        op: ConditionOp,
        comp0: String,
        comp1: String,
        a: String,
        b: String,
    },
    Wait {
        value: String,
    },
    Stop,
    End,
    PackColor {
        result: String,
        r: String,
        g: String,
        b: String,
        a: String,
    },
    UnpackColor {
        r: String,
        g: String,
        b: String,
        a: String,
        value: String,
    },
    Lookup {
        type_: ContentType,
        result: String,
        id: String,
    },
    Jump {
        dest_index: i32,
        op: ConditionOp,
        value: String,
        compare: String,
    },
    Control {
        type_: LAccess,
        target: String,
        p1: String,
        p2: String,
        p3: String,
        p4: String,
    },
    Radar {
        target1: RadarTarget,
        target2: RadarTarget,
        target3: RadarTarget,
        sort: RadarSort,
        radar: String,
        sort_order: String,
        output: String,
    },
    Sensor {
        to: String,
        from: String,
        type_: String,
    },
    UnitBind {
        type_: String,
    },
    UnitControl {
        type_: LUnitControl,
        p1: String,
        p2: String,
        p3: String,
        p4: String,
        p5: String,
    },
    UnitRadar {
        target1: RadarTarget,
        target2: RadarTarget,
        target3: RadarTarget,
        sort: RadarSort,
        radar: String,
        sort_order: String,
        output: String,
    },
    UnitLocate {
        locate: LLocate,
        flag: BlockFlag,
        enemy: String,
        ore: String,
        out_x: String,
        out_y: String,
        out_found: String,
        out_build: String,
    },
    Query {
        shape: QueryShape,
        type_: QueryType,
        team: String,
        x: String,
        y: String,
        w: String,
        h: String,
    },
    GetBlock {
        layer: TileLayer,
        result: String,
        x: String,
        y: String,
    },
    SetBlock {
        layer: TileLayer,
        block: String,
        x: String,
        y: String,
        team: String,
        rotation: String,
    },
    SpawnUnit {
        type_: String,
        x: String,
        y: String,
        rotation: String,
        team: String,
        result: String,
    },
    ApplyStatus {
        clear: bool,
        effect: String,
        unit: String,
        duration: String,
    },
    SpawnWave {
        x: String,
        y: String,
        natural: String,
    },
    SpawnBullet {
        result: String,
        from: String,
        index: String,
        x: String,
        y: String,
        rotation: String,
        team: String,
        owner: String,
        damage: String,
        velocity_scl: String,
        life_scl: String,
        aim_x: String,
        aim_y: String,
    },
    WeatherSense {
        to: String,
        weather: String,
    },
    WeatherSet {
        weather: String,
        state: String,
    },
    Effect {
        type_: String,
        x: String,
        y: String,
        sizerot: String,
        color: String,
        data: String,
    },
    Explosion {
        team: String,
        x: String,
        y: String,
        radius: String,
        damage: String,
        air: String,
        ground: String,
        pierce: String,
        effect: String,
    },
    SetRule {
        rule: LogicRule,
        value: String,
        p1: String,
        p2: String,
        p3: String,
        p4: String,
    },
    Fetch {
        type_: FetchType,
        result: String,
        team: String,
        index: String,
        extra: String,
    },
    GetFlag {
        result: String,
        flag: String,
    },
    SetFlag {
        flag: String,
        value: String,
    },
    SetProp {
        type_: String,
        of: String,
        value: String,
    },
    FlushMessage {
        type_: MessageType,
        duration: String,
        out_success: String,
    },
    Cutscene {
        action: CutsceneAction,
        p1: String,
        p2: String,
        p3: String,
        p4: String,
    },
    ClientData {
        channel: String,
        value: String,
        reliable: String,
    },
    PlaySound {
        positional: bool,
        id: String,
        volume: String,
        pitch: String,
        pan: String,
        x: String,
        y: String,
        limit: String,
    },
    SetMarker {
        type_: LMarkerControl,
        id: String,
        p1: String,
        p2: String,
        p3: String,
    },
    MakeMarker {
        type_: String,
        id: String,
        x: String,
        y: String,
        replace: String,
    },
}

impl LogicStatement {
    pub fn invalid() -> Self {
        Self::Invalid
    }

    pub fn read() -> Self {
        Self::Read {
            output: "result".into(),
            target: "cell1".into(),
            address: "0".into(),
        }
    }

    pub fn write() -> Self {
        Self::Write {
            input: "result".into(),
            target: "cell1".into(),
            address: "0".into(),
        }
    }

    pub fn draw() -> Self {
        Self::Draw {
            type_: GraphicsType::Clear,
            x: "0".into(),
            y: "0".into(),
            p1: "0".into(),
            p2: "0".into(),
            p3: "0".into(),
            p4: "0".into(),
        }
    }

    pub fn print() -> Self {
        Self::Print {
            value: "\"frog\"".into(),
        }
    }

    pub fn print_char() -> Self {
        Self::PrintChar { value: "65".into() }
    }

    pub fn format() -> Self {
        Self::Format {
            value: "\"frog\"".into(),
        }
    }

    pub fn locale_print() -> Self {
        Self::LocalePrint {
            value: "\"name\"".into(),
        }
    }

    pub fn draw_flush() -> Self {
        Self::DrawFlush {
            target: "display1".into(),
        }
    }

    pub fn print_flush() -> Self {
        Self::PrintFlush {
            target: "message1".into(),
        }
    }

    pub fn get_link() -> Self {
        Self::GetLink {
            output: "result".into(),
            address: "0".into(),
        }
    }

    pub fn set_rate() -> Self {
        Self::SetRate {
            amount: "10".into(),
        }
    }

    pub fn sync() -> Self {
        Self::Sync {
            variable: "var".into(),
        }
    }

    pub fn set() -> Self {
        Self::Set {
            to: "result".into(),
            from: "0".into(),
        }
    }

    pub fn operation() -> Self {
        Self::Operation {
            op: LogicOp::Add,
            dest: "result".into(),
            a: "a".into(),
            b: "b".into(),
        }
    }

    pub fn select() -> Self {
        Self::Select {
            result: "result".into(),
            op: ConditionOp::NotEqual,
            comp0: "x".into(),
            comp1: "false".into(),
            a: "a".into(),
            b: "b".into(),
        }
    }

    pub fn wait() -> Self {
        Self::Wait {
            value: "0.5".into(),
        }
    }

    pub fn stop() -> Self {
        Self::Stop
    }

    pub fn end() -> Self {
        Self::End
    }

    pub fn pack_color() -> Self {
        Self::PackColor {
            result: "result".into(),
            r: "1".into(),
            g: "0".into(),
            b: "0".into(),
            a: "1".into(),
        }
    }

    pub fn unpack_color() -> Self {
        Self::UnpackColor {
            r: "r".into(),
            g: "g".into(),
            b: "b".into(),
            a: "a".into(),
            value: "color".into(),
        }
    }

    pub fn lookup() -> Self {
        Self::Lookup {
            type_: ContentType::Item,
            result: "result".into(),
            id: "0".into(),
        }
    }

    pub fn jump() -> Self {
        Self::Jump {
            dest_index: 0,
            op: ConditionOp::NotEqual,
            value: "x".into(),
            compare: "false".into(),
        }
    }

    pub fn control() -> Self {
        Self::Control {
            type_: LAccess::Enabled,
            target: "block1".into(),
            p1: "0".into(),
            p2: "0".into(),
            p3: "0".into(),
            p4: "0".into(),
        }
    }

    pub fn radar() -> Self {
        Self::Radar {
            target1: RadarTarget::Enemy,
            target2: RadarTarget::Any,
            target3: RadarTarget::Any,
            sort: RadarSort::Distance,
            radar: "turret1".into(),
            sort_order: "1".into(),
            output: "result".into(),
        }
    }

    pub fn sensor() -> Self {
        Self::Sensor {
            to: "result".into(),
            from: "block1".into(),
            type_: "@copper".into(),
        }
    }

    pub fn unit_bind() -> Self {
        Self::UnitBind {
            type_: "@poly".into(),
        }
    }

    pub fn unit_control() -> Self {
        Self::UnitControl {
            type_: LUnitControl::Move,
            p1: "0".into(),
            p2: "0".into(),
            p3: "0".into(),
            p4: "0".into(),
            p5: "0".into(),
        }
    }

    pub fn unit_radar() -> Self {
        Self::UnitRadar {
            target1: RadarTarget::Enemy,
            target2: RadarTarget::Any,
            target3: RadarTarget::Any,
            sort: RadarSort::Distance,
            radar: "0".into(),
            sort_order: "1".into(),
            output: "result".into(),
        }
    }

    pub fn unit_locate() -> Self {
        Self::UnitLocate {
            locate: LLocate::Building,
            flag: BlockFlag::Core,
            enemy: "true".into(),
            ore: "@copper".into(),
            out_x: "outx".into(),
            out_y: "outy".into(),
            out_found: "found".into(),
            out_build: "building".into(),
        }
    }

    pub fn query() -> Self {
        Self::Query {
            shape: QueryShape::Circle,
            type_: QueryType::Unit,
            team: "null".into(),
            x: "0".into(),
            y: "0".into(),
            w: "10".into(),
            h: "10".into(),
        }
    }

    pub fn get_block() -> Self {
        Self::GetBlock {
            layer: TileLayer::Block,
            result: "result".into(),
            x: "0".into(),
            y: "0".into(),
        }
    }

    pub fn set_block() -> Self {
        Self::SetBlock {
            layer: TileLayer::Block,
            block: "@air".into(),
            x: "0".into(),
            y: "0".into(),
            team: "@derelict".into(),
            rotation: "0".into(),
        }
    }

    pub fn spawn_unit() -> Self {
        Self::SpawnUnit {
            type_: "@dagger".into(),
            x: "10".into(),
            y: "10".into(),
            rotation: "90".into(),
            team: "@sharded".into(),
            result: "result".into(),
        }
    }

    pub fn apply_status() -> Self {
        Self::ApplyStatus {
            clear: false,
            effect: "wet".into(),
            unit: "unit".into(),
            duration: "10".into(),
        }
    }

    pub fn spawn_wave() -> Self {
        Self::SpawnWave {
            x: "10".into(),
            y: "10".into(),
            natural: "false".into(),
        }
    }

    pub fn spawn_bullet() -> Self {
        Self::SpawnBullet {
            result: "result".into(),
            from: "@dagger".into(),
            index: "0".into(),
            x: "x".into(),
            y: "y".into(),
            rotation: "angle".into(),
            team: "null".into(),
            owner: "null".into(),
            damage: "-1".into(),
            velocity_scl: "1".into(),
            life_scl: "1".into(),
            aim_x: "-1".into(),
            aim_y: "-1".into(),
        }
    }

    pub fn weather_sense() -> Self {
        Self::WeatherSense {
            to: "result".into(),
            weather: "@rain".into(),
        }
    }

    pub fn weather_set() -> Self {
        Self::WeatherSet {
            weather: "@rain".into(),
            state: "true".into(),
        }
    }

    pub fn effect() -> Self {
        Self::Effect {
            type_: "warn".into(),
            x: "0".into(),
            y: "0".into(),
            sizerot: "2".into(),
            color: "%ffaaff".into(),
            data: "".into(),
        }
    }

    pub fn explosion() -> Self {
        Self::Explosion {
            team: "@crux".into(),
            x: "0".into(),
            y: "0".into(),
            radius: "5".into(),
            damage: "50".into(),
            air: "true".into(),
            ground: "true".into(),
            pierce: "false".into(),
            effect: "true".into(),
        }
    }

    pub fn set_rule() -> Self {
        Self::SetRule {
            rule: LogicRule::WaveSpacing,
            value: "10".into(),
            p1: "0".into(),
            p2: "0".into(),
            p3: "100".into(),
            p4: "100".into(),
        }
    }

    pub fn fetch() -> Self {
        Self::Fetch {
            type_: FetchType::Unit,
            result: "result".into(),
            team: "@sharded".into(),
            index: "0".into(),
            extra: "@conveyor".into(),
        }
    }

    pub fn get_flag() -> Self {
        Self::GetFlag {
            result: "result".into(),
            flag: "\"flag\"".into(),
        }
    }

    pub fn set_flag() -> Self {
        Self::SetFlag {
            flag: "\"flag\"".into(),
            value: "true".into(),
        }
    }

    pub fn set_prop() -> Self {
        Self::SetProp {
            type_: "@copper".into(),
            of: "block1".into(),
            value: "0".into(),
        }
    }

    pub fn flush_message() -> Self {
        Self::FlushMessage {
            type_: MessageType::Announce,
            duration: "3".into(),
            out_success: "@wait".into(),
        }
    }

    pub fn cutscene() -> Self {
        Self::Cutscene {
            action: CutsceneAction::Pan,
            p1: "100".into(),
            p2: "100".into(),
            p3: "0.06".into(),
            p4: "0".into(),
        }
    }

    pub fn client_data() -> Self {
        Self::ClientData {
            channel: "\"frog\"".into(),
            value: "\"bar\"".into(),
            reliable: "0".into(),
        }
    }

    pub fn play_sound() -> Self {
        Self::PlaySound {
            positional: false,
            id: "@sfx-shoot".into(),
            volume: "1".into(),
            pitch: "1".into(),
            pan: "0".into(),
            x: "@thisx".into(),
            y: "@thisy".into(),
            limit: "true".into(),
        }
    }

    pub fn set_marker() -> Self {
        Self::SetMarker {
            type_: LMarkerControl::Pos,
            id: "0".into(),
            p1: "0".into(),
            p2: "0".into(),
            p3: "0".into(),
        }
    }

    pub fn make_marker() -> Self {
        Self::MakeMarker {
            type_: "shape".into(),
            id: "0".into(),
            x: "0".into(),
            y: "0".into(),
            replace: "true".into(),
        }
    }

    pub fn opcode(&self) -> &'static str {
        match self {
            LogicStatement::Invalid => "noop",
            LogicStatement::Read { .. } => "read",
            LogicStatement::Write { .. } => "write",
            LogicStatement::Draw { .. } => "draw",
            LogicStatement::Print { .. } => "print",
            LogicStatement::PrintChar { .. } => "printchar",
            LogicStatement::Format { .. } => "format",
            LogicStatement::LocalePrint { .. } => "localeprint",
            LogicStatement::DrawFlush { .. } => "drawflush",
            LogicStatement::PrintFlush { .. } => "printflush",
            LogicStatement::GetLink { .. } => "getlink",
            LogicStatement::SetRate { .. } => "setrate",
            LogicStatement::Sync { .. } => "sync",
            LogicStatement::Set { .. } => "set",
            LogicStatement::Operation { .. } => "op",
            LogicStatement::Select { .. } => "select",
            LogicStatement::Wait { .. } => "wait",
            LogicStatement::Stop => "stop",
            LogicStatement::End => "end",
            LogicStatement::PackColor { .. } => "packcolor",
            LogicStatement::UnpackColor { .. } => "unpackcolor",
            LogicStatement::Lookup { .. } => "lookup",
            LogicStatement::Jump { .. } => "jump",
            LogicStatement::Control { .. } => "control",
            LogicStatement::Radar { .. } => "radar",
            LogicStatement::Sensor { .. } => "sensor",
            LogicStatement::UnitBind { .. } => "ubind",
            LogicStatement::UnitControl { .. } => "ucontrol",
            LogicStatement::UnitRadar { .. } => "uradar",
            LogicStatement::UnitLocate { .. } => "ulocate",
            LogicStatement::Query { .. } => "query",
            LogicStatement::GetBlock { .. } => "getblock",
            LogicStatement::SetBlock { .. } => "setblock",
            LogicStatement::SpawnUnit { .. } => "spawn",
            LogicStatement::ApplyStatus { .. } => "status",
            LogicStatement::SpawnWave { .. } => "spawnwave",
            LogicStatement::SpawnBullet { .. } => "bullet",
            LogicStatement::WeatherSense { .. } => "weathersense",
            LogicStatement::WeatherSet { .. } => "weatherset",
            LogicStatement::Effect { .. } => "effect",
            LogicStatement::Explosion { .. } => "explosion",
            LogicStatement::SetRule { .. } => "setrule",
            LogicStatement::Fetch { .. } => "fetch",
            LogicStatement::GetFlag { .. } => "getflag",
            LogicStatement::SetFlag { .. } => "setflag",
            LogicStatement::SetProp { .. } => "setprop",
            LogicStatement::FlushMessage { .. } => "message",
            LogicStatement::Cutscene { .. } => "cutscene",
            LogicStatement::ClientData { .. } => "clientdata",
            LogicStatement::PlaySound { .. } => "playsound",
            LogicStatement::SetMarker { .. } => "setmarker",
            LogicStatement::MakeMarker { .. } => "makemarker",
        }
    }

    pub fn category(&self) -> &'static LCategory {
        match self {
            LogicStatement::Invalid => LCategory::by_name("unknown").unwrap(),
            LogicStatement::Read { .. }
            | LogicStatement::Write { .. }
            | LogicStatement::Draw { .. }
            | LogicStatement::Print { .. }
            | LogicStatement::PrintChar { .. }
            | LogicStatement::Format { .. } => LCategory::by_name("io").unwrap(),
            LogicStatement::DrawFlush { .. }
            | LogicStatement::PrintFlush { .. }
            | LogicStatement::GetLink { .. } => LCategory::by_name("block").unwrap(),
            LogicStatement::SetRate { .. }
            | LogicStatement::Sync { .. }
            | LogicStatement::LocalePrint { .. }
            | LogicStatement::Query { .. }
            | LogicStatement::GetBlock { .. }
            | LogicStatement::SetBlock { .. }
            | LogicStatement::SpawnUnit { .. }
            | LogicStatement::ApplyStatus { .. }
            | LogicStatement::SpawnWave { .. }
            | LogicStatement::SpawnBullet { .. }
            | LogicStatement::WeatherSense { .. }
            | LogicStatement::WeatherSet { .. }
            | LogicStatement::Effect { .. }
            | LogicStatement::Explosion { .. }
            | LogicStatement::SetRule { .. }
            | LogicStatement::Fetch { .. }
            | LogicStatement::GetFlag { .. }
            | LogicStatement::SetFlag { .. }
            | LogicStatement::SetProp { .. }
            | LogicStatement::FlushMessage { .. }
            | LogicStatement::Cutscene { .. }
            | LogicStatement::ClientData { .. }
            | LogicStatement::PlaySound { .. }
            | LogicStatement::SetMarker { .. }
            | LogicStatement::MakeMarker { .. } => LCategory::by_name("world").unwrap(),
            LogicStatement::Set { .. }
            | LogicStatement::Operation { .. }
            | LogicStatement::Select { .. }
            | LogicStatement::Lookup { .. }
            | LogicStatement::PackColor { .. }
            | LogicStatement::UnpackColor { .. } => LCategory::by_name("operation").unwrap(),
            LogicStatement::Wait { .. }
            | LogicStatement::Stop
            | LogicStatement::End
            | LogicStatement::Jump { .. } => LCategory::by_name("control").unwrap(),
            LogicStatement::Control { .. }
            | LogicStatement::Radar { .. }
            | LogicStatement::Sensor { .. } => LCategory::by_name("block").unwrap(),
            LogicStatement::UnitBind { .. }
            | LogicStatement::UnitControl { .. }
            | LogicStatement::UnitRadar { .. }
            | LogicStatement::UnitLocate { .. } => LCategory::by_name("unit").unwrap(),
        }
    }

    pub fn privileged(&self) -> bool {
        matches!(
            self,
            LogicStatement::SetRate { .. }
                | LogicStatement::Sync { .. }
                | LogicStatement::LocalePrint { .. }
                | LogicStatement::Query { .. }
                | LogicStatement::GetBlock { .. }
                | LogicStatement::SetBlock { .. }
                | LogicStatement::SpawnUnit { .. }
                | LogicStatement::ApplyStatus { .. }
                | LogicStatement::SpawnWave { .. }
                | LogicStatement::SpawnBullet { .. }
                | LogicStatement::WeatherSense { .. }
                | LogicStatement::WeatherSet { .. }
                | LogicStatement::Effect { .. }
                | LogicStatement::Explosion { .. }
                | LogicStatement::SetRule { .. }
                | LogicStatement::Fetch { .. }
                | LogicStatement::GetFlag { .. }
                | LogicStatement::SetFlag { .. }
                | LogicStatement::SetProp { .. }
                | LogicStatement::FlushMessage { .. }
                | LogicStatement::Cutscene { .. }
                | LogicStatement::ClientData { .. }
                | LogicStatement::PlaySound { .. }
                | LogicStatement::SetMarker { .. }
                | LogicStatement::MakeMarker { .. }
        )
    }

    pub fn tokens(&self) -> Vec<String> {
        match self {
            LogicStatement::Invalid => vec!["noop".into()],
            LogicStatement::Read {
                output,
                target,
                address,
            } => vec![
                "read".into(),
                output.clone(),
                target.clone(),
                address.clone(),
            ],
            LogicStatement::Write {
                input,
                target,
                address,
            } => vec![
                "write".into(),
                input.clone(),
                target.clone(),
                address.clone(),
            ],
            LogicStatement::Draw {
                type_,
                x,
                y,
                p1,
                p2,
                p3,
                p4,
            } => vec![
                "draw".into(),
                type_.wire_name().into(),
                x.clone(),
                y.clone(),
                p1.clone(),
                p2.clone(),
                p3.clone(),
                p4.clone(),
            ],
            LogicStatement::Print { value } => vec!["print".into(), value.clone()],
            LogicStatement::PrintChar { value } => vec!["printchar".into(), value.clone()],
            LogicStatement::Format { value } => vec!["format".into(), value.clone()],
            LogicStatement::LocalePrint { value } => vec!["localeprint".into(), value.clone()],
            LogicStatement::DrawFlush { target } => vec!["drawflush".into(), target.clone()],
            LogicStatement::PrintFlush { target } => vec!["printflush".into(), target.clone()],
            LogicStatement::GetLink { output, address } => {
                vec!["getlink".into(), output.clone(), address.clone()]
            }
            LogicStatement::SetRate { amount } => vec!["setrate".into(), amount.clone()],
            LogicStatement::Sync { variable } => vec!["sync".into(), variable.clone()],
            LogicStatement::Set { to, from } => vec!["set".into(), to.clone(), from.clone()],
            LogicStatement::Operation { op, dest, a, b } => vec![
                "op".into(),
                op.java_name().into(),
                dest.clone(),
                a.clone(),
                b.clone(),
            ],
            LogicStatement::Select {
                result,
                op,
                comp0,
                comp1,
                a,
                b,
            } => vec![
                "select".into(),
                result.clone(),
                op.java_name().into(),
                comp0.clone(),
                comp1.clone(),
                a.clone(),
                b.clone(),
            ],
            LogicStatement::Wait { value } => vec!["wait".into(), value.clone()],
            LogicStatement::Stop => vec!["stop".into()],
            LogicStatement::End => vec!["end".into()],
            LogicStatement::PackColor { result, r, g, b, a } => vec![
                "packcolor".into(),
                result.clone(),
                r.clone(),
                g.clone(),
                b.clone(),
                a.clone(),
            ],
            LogicStatement::UnpackColor { r, g, b, a, value } => vec![
                "unpackcolor".into(),
                r.clone(),
                g.clone(),
                b.clone(),
                a.clone(),
                value.clone(),
            ],
            LogicStatement::Lookup { type_, result, id } => vec![
                "lookup".into(),
                type_.wire_name().into(),
                result.clone(),
                id.clone(),
            ],
            LogicStatement::Jump {
                dest_index,
                op,
                value,
                compare,
            } => vec![
                "jump".into(),
                dest_index.to_string(),
                op.java_name().into(),
                value.clone(),
                compare.clone(),
            ],
            LogicStatement::Control {
                type_,
                target,
                p1,
                p2,
                p3,
                p4,
            } => vec![
                "control".into(),
                type_.wire_name().into(),
                target.clone(),
                p1.clone(),
                p2.clone(),
                p3.clone(),
                p4.clone(),
            ],
            LogicStatement::Radar {
                target1,
                target2,
                target3,
                sort,
                radar,
                sort_order,
                output,
            } => vec![
                "radar".into(),
                target1.wire_name().into(),
                target2.wire_name().into(),
                target3.wire_name().into(),
                sort.wire_name().into(),
                radar.clone(),
                sort_order.clone(),
                output.clone(),
            ],
            LogicStatement::Sensor { to, from, type_ } => {
                vec!["sensor".into(), to.clone(), from.clone(), type_.clone()]
            }
            LogicStatement::UnitBind { type_ } => vec!["ubind".into(), type_.clone()],
            LogicStatement::UnitControl {
                type_,
                p1,
                p2,
                p3,
                p4,
                p5,
            } => vec![
                "ucontrol".into(),
                type_.wire_name().into(),
                p1.clone(),
                p2.clone(),
                p3.clone(),
                p4.clone(),
                p5.clone(),
            ],
            LogicStatement::UnitRadar {
                target1,
                target2,
                target3,
                sort,
                radar,
                sort_order,
                output,
            } => vec![
                "uradar".into(),
                target1.wire_name().into(),
                target2.wire_name().into(),
                target3.wire_name().into(),
                sort.wire_name().into(),
                radar.clone(),
                sort_order.clone(),
                output.clone(),
            ],
            LogicStatement::UnitLocate {
                locate,
                flag,
                enemy,
                ore,
                out_x,
                out_y,
                out_found,
                out_build,
            } => vec![
                "ulocate".into(),
                locate.wire_name().into(),
                flag.wire_name().into(),
                enemy.clone(),
                ore.clone(),
                out_x.clone(),
                out_y.clone(),
                out_found.clone(),
                out_build.clone(),
            ],
            LogicStatement::Query {
                shape,
                type_,
                team,
                x,
                y,
                w,
                h,
            } => vec![
                "query".into(),
                shape.wire_name().into(),
                type_.wire_name().into(),
                team.clone(),
                x.clone(),
                y.clone(),
                w.clone(),
                h.clone(),
            ],
            LogicStatement::GetBlock {
                layer,
                result,
                x,
                y,
            } => vec![
                "getblock".into(),
                layer.wire_name().into(),
                result.clone(),
                x.clone(),
                y.clone(),
            ],
            LogicStatement::SetBlock {
                layer,
                block,
                x,
                y,
                team,
                rotation,
            } => vec![
                "setblock".into(),
                layer.wire_name().into(),
                block.clone(),
                x.clone(),
                y.clone(),
                team.clone(),
                rotation.clone(),
            ],
            LogicStatement::SpawnUnit {
                type_,
                x,
                y,
                rotation,
                team,
                result,
            } => vec![
                "spawn".into(),
                type_.clone(),
                x.clone(),
                y.clone(),
                rotation.clone(),
                team.clone(),
                result.clone(),
            ],
            LogicStatement::ApplyStatus {
                clear,
                effect,
                unit,
                duration,
            } => vec![
                "status".into(),
                clear.to_string(),
                effect.clone(),
                unit.clone(),
                duration.clone(),
            ],
            LogicStatement::SpawnWave { x, y, natural } => {
                vec!["spawnwave".into(), x.clone(), y.clone(), natural.clone()]
            }
            LogicStatement::SpawnBullet {
                result,
                from,
                index,
                x,
                y,
                rotation,
                team,
                owner,
                damage,
                velocity_scl,
                life_scl,
                aim_x,
                aim_y,
            } => vec![
                "bullet".into(),
                result.clone(),
                from.clone(),
                index.clone(),
                x.clone(),
                y.clone(),
                rotation.clone(),
                team.clone(),
                owner.clone(),
                damage.clone(),
                velocity_scl.clone(),
                life_scl.clone(),
                aim_x.clone(),
                aim_y.clone(),
            ],
            LogicStatement::WeatherSense { to, weather } => {
                vec!["weathersense".into(), to.clone(), weather.clone()]
            }
            LogicStatement::WeatherSet { weather, state } => {
                vec!["weatherset".into(), weather.clone(), state.clone()]
            }
            LogicStatement::Effect {
                type_,
                x,
                y,
                sizerot,
                color,
                data,
            } => vec![
                "effect".into(),
                type_.clone(),
                x.clone(),
                y.clone(),
                sizerot.clone(),
                color.clone(),
                data.clone(),
            ],
            LogicStatement::Explosion {
                team,
                x,
                y,
                radius,
                damage,
                air,
                ground,
                pierce,
                effect,
            } => vec![
                "explosion".into(),
                team.clone(),
                x.clone(),
                y.clone(),
                radius.clone(),
                damage.clone(),
                air.clone(),
                ground.clone(),
                pierce.clone(),
                effect.clone(),
            ],
            LogicStatement::SetRule {
                rule,
                value,
                p1,
                p2,
                p3,
                p4,
            } => vec![
                "setrule".into(),
                rule.wire_name().into(),
                value.clone(),
                p1.clone(),
                p2.clone(),
                p3.clone(),
                p4.clone(),
            ],
            LogicStatement::Fetch {
                type_,
                result,
                team,
                index,
                extra,
            } => vec![
                "fetch".into(),
                type_.wire_name().into(),
                result.clone(),
                team.clone(),
                index.clone(),
                extra.clone(),
            ],
            LogicStatement::GetFlag { result, flag } => {
                vec!["getflag".into(), result.clone(), flag.clone()]
            }
            LogicStatement::SetFlag { flag, value } => {
                vec!["setflag".into(), flag.clone(), value.clone()]
            }
            LogicStatement::SetProp { type_, of, value } => {
                vec!["setprop".into(), type_.clone(), of.clone(), value.clone()]
            }
            LogicStatement::FlushMessage {
                type_,
                duration,
                out_success,
            } => vec![
                "message".into(),
                type_.wire_name().into(),
                duration.clone(),
                out_success.clone(),
            ],
            LogicStatement::Cutscene {
                action,
                p1,
                p2,
                p3,
                p4,
            } => vec![
                "cutscene".into(),
                action.wire_name().into(),
                p1.clone(),
                p2.clone(),
                p3.clone(),
                p4.clone(),
            ],
            LogicStatement::ClientData {
                channel,
                value,
                reliable,
            } => vec![
                "clientdata".into(),
                channel.clone(),
                value.clone(),
                reliable.clone(),
            ],
            LogicStatement::PlaySound {
                positional,
                id,
                volume,
                pitch,
                pan,
                x,
                y,
                limit,
            } => vec![
                "playsound".into(),
                positional.to_string(),
                id.clone(),
                volume.clone(),
                pitch.clone(),
                pan.clone(),
                x.clone(),
                y.clone(),
                limit.clone(),
            ],
            LogicStatement::SetMarker {
                type_,
                id,
                p1,
                p2,
                p3,
            } => vec![
                "setmarker".into(),
                type_.wire_name().into(),
                id.clone(),
                p1.clone(),
                p2.clone(),
                p3.clone(),
            ],
            LogicStatement::MakeMarker {
                type_,
                id,
                x,
                y,
                replace,
            } => vec![
                "makemarker".into(),
                type_.clone(),
                id.clone(),
                x.clone(),
                y.clone(),
                replace.clone(),
            ],
        }
    }

    pub fn write_line(&self) -> String {
        self.tokens().join(" ")
    }

    pub fn read_tokens(tokens: &[String]) -> Option<Self> {
        let opcode = tokens.first()?.as_str();
        Some(match opcode {
            "noop" => Self::Invalid,
            "read" => {
                let mut statement = Self::read();
                if let LogicStatement::Read {
                    output,
                    target,
                    address,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *output = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *target = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *address = tokens[3].clone();
                    }
                }
                statement
            }
            "write" => {
                let mut statement = Self::write();
                if let LogicStatement::Write {
                    input,
                    target,
                    address,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *input = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *target = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *address = tokens[3].clone();
                    }
                }
                statement
            }
            "draw" => {
                let mut statement = Self::draw();
                if let LogicStatement::Draw {
                    type_,
                    x,
                    y,
                    p1,
                    p2,
                    p3,
                    p4,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = GraphicsType::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *x = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *y = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *p1 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *p2 = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *p3 = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *p4 = tokens[7].clone();
                    }

                    if *type_ == GraphicsType::Color && p2 == "0" {
                        *p2 = "255".into();
                    }

                    if *type_ == GraphicsType::Print && LogicAlign::by_name(p1).is_some() {
                        p1.insert(0, '@');
                    }
                }
                statement
            }
            "print" => Self::Print {
                value: tokens.get(1).cloned().unwrap_or_else(|| "\"frog\"".into()),
            },
            "printchar" => Self::PrintChar {
                value: tokens.get(1).cloned().unwrap_or_else(|| "65".into()),
            },
            "format" => Self::Format {
                value: tokens.get(1).cloned().unwrap_or_else(|| "\"frog\"".into()),
            },
            "localeprint" => Self::LocalePrint {
                value: tokens.get(1).cloned().unwrap_or_else(|| "\"name\"".into()),
            },
            "drawflush" => Self::DrawFlush {
                target: tokens.get(1).cloned().unwrap_or_else(|| "display1".into()),
            },
            "printflush" => Self::PrintFlush {
                target: tokens.get(1).cloned().unwrap_or_else(|| "message1".into()),
            },
            "getlink" => {
                let mut statement = Self::get_link();
                if let LogicStatement::GetLink { output, address } = &mut statement {
                    if tokens.len() > 1 {
                        *output = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *address = tokens[2].clone();
                    }
                }
                statement
            }
            "setrate" => Self::SetRate {
                amount: tokens.get(1).cloned().unwrap_or_else(|| "10".into()),
            },
            "sync" => Self::Sync {
                variable: tokens.get(1).cloned().unwrap_or_else(|| "var".into()),
            },
            "set" => {
                let mut statement = Self::set();
                if let LogicStatement::Set { to, from } = &mut statement {
                    if tokens.len() > 1 {
                        *to = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *from = tokens[2].clone();
                    }
                }
                statement
            }
            "op" => {
                let mut statement = Self::operation();
                if let LogicStatement::Operation { op, dest, a, b } = &mut statement {
                    if tokens.len() > 1 {
                        *op = LogicOp::by_java_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *dest = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *a = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *b = tokens[4].clone();
                    }
                }
                statement
            }
            "select" => {
                let mut statement = Self::select();
                if let LogicStatement::Select {
                    result,
                    op,
                    comp0,
                    comp1,
                    a,
                    b,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *result = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *op = ConditionOp::by_java_name(&tokens[2])?;
                    }
                    if tokens.len() > 3 {
                        *comp0 = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *comp1 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *a = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *b = tokens[6].clone();
                    }
                }
                statement
            }
            "wait" => Self::Wait {
                value: tokens.get(1).cloned().unwrap_or_else(|| "0.5".into()),
            },
            "stop" => Self::Stop,
            "end" => Self::End,
            "packcolor" => {
                let mut statement = Self::pack_color();
                if let LogicStatement::PackColor { result, r, g, b, a } = &mut statement {
                    if tokens.len() > 1 {
                        *result = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *r = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *g = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *b = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *a = tokens[5].clone();
                    }
                }
                statement
            }
            "unpackcolor" => {
                let mut statement = Self::unpack_color();
                if let LogicStatement::UnpackColor { r, g, b, a, value } = &mut statement {
                    if tokens.len() > 1 {
                        *r = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *g = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *b = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *a = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *value = tokens[5].clone();
                    }
                }
                statement
            }
            "lookup" => {
                let mut statement = Self::lookup();
                if let LogicStatement::Lookup { type_, result, id } = &mut statement {
                    if tokens.len() > 1 {
                        let value = ContentType::from_wire_name(&tokens[1])?;
                        if !LOOKABLE_CONTENT_TYPES.contains(&value) {
                            return None;
                        }
                        *type_ = value;
                    }
                    if tokens.len() > 2 {
                        *result = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *id = tokens[3].clone();
                    }
                }
                statement
            }
            "jump" => {
                let mut statement = Self::jump();
                if let LogicStatement::Jump {
                    dest_index,
                    op,
                    value,
                    compare,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *dest_index = tokens[1].parse().ok()?;
                    }
                    if tokens.len() > 2 {
                        *op = ConditionOp::by_java_name(&tokens[2])?;
                    }
                    if tokens.len() > 3 {
                        *value = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *compare = tokens[4].clone();
                    }
                }
                statement
            }
            "control" => {
                let mut statement = Self::control();
                if let LogicStatement::Control {
                    type_,
                    target,
                    p1,
                    p2,
                    p3,
                    p4,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        let value = LAccess::by_wire_name(&tokens[1])?;
                        if !value.is_control() {
                            return None;
                        }
                        *type_ = value;
                    }
                    if tokens.len() > 2 {
                        *target = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *p1 = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *p2 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *p3 = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *p4 = tokens[6].clone();
                    }
                }
                statement
            }
            "radar" => {
                let mut statement = Self::radar();
                if let LogicStatement::Radar {
                    target1,
                    target2,
                    target3,
                    sort,
                    radar,
                    sort_order,
                    output,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *target1 = RadarTarget::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *target2 = RadarTarget::by_wire_name(&tokens[2])?;
                    }
                    if tokens.len() > 3 {
                        *target3 = RadarTarget::by_wire_name(&tokens[3])?;
                    }
                    if tokens.len() > 4 {
                        *sort = RadarSort::by_wire_name(&tokens[4])?;
                    }
                    if tokens.len() > 5 {
                        *radar = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *sort_order = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *output = tokens[7].clone();
                    }
                }
                statement
            }
            "sensor" => {
                let mut statement = Self::sensor();
                if let LogicStatement::Sensor { to, from, type_ } = &mut statement {
                    if tokens.len() > 1 {
                        *to = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *from = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *type_ = tokens[3].clone();
                    }
                }
                statement
            }
            "ubind" => Self::UnitBind {
                type_: tokens.get(1).cloned().unwrap_or_else(|| "@poly".into()),
            },
            "ucontrol" => {
                let mut statement = Self::unit_control();
                if let LogicStatement::UnitControl {
                    type_,
                    p1,
                    p2,
                    p3,
                    p4,
                    p5,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = LUnitControl::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *p1 = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *p2 = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *p3 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *p4 = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *p5 = tokens[6].clone();
                    }
                }
                statement
            }
            "uradar" => {
                let mut statement = Self::unit_radar();
                if let LogicStatement::UnitRadar {
                    target1,
                    target2,
                    target3,
                    sort,
                    radar,
                    sort_order,
                    output,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *target1 = RadarTarget::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *target2 = RadarTarget::by_wire_name(&tokens[2])?;
                    }
                    if tokens.len() > 3 {
                        *target3 = RadarTarget::by_wire_name(&tokens[3])?;
                    }
                    if tokens.len() > 4 {
                        *sort = RadarSort::by_wire_name(&tokens[4])?;
                    }
                    if tokens.len() > 5 {
                        *radar = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *sort_order = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *output = tokens[7].clone();
                    }
                }
                statement
            }
            "ulocate" => {
                let mut statement = Self::unit_locate();
                if let LogicStatement::UnitLocate {
                    locate,
                    flag,
                    enemy,
                    ore,
                    out_x,
                    out_y,
                    out_found,
                    out_build,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *locate = LLocate::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        let value = BlockFlag::by_wire_name(&tokens[2])?;
                        if !value.is_logic() {
                            return None;
                        }
                        *flag = value;
                    }
                    if tokens.len() > 3 {
                        *enemy = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *ore = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *out_x = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *out_y = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *out_found = tokens[7].clone();
                    }
                    if tokens.len() > 8 {
                        *out_build = tokens[8].clone();
                    }
                }
                statement
            }
            "query" => {
                let mut statement = Self::query();
                if let LogicStatement::Query {
                    shape,
                    type_,
                    team,
                    x,
                    y,
                    w,
                    h,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *shape = QueryShape::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *type_ = QueryType::by_wire_name(&tokens[2])?;
                    }
                    if tokens.len() > 3 {
                        *team = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *x = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *y = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *w = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *h = tokens[7].clone();
                    }
                }
                statement
            }
            "getblock" => {
                let mut statement = Self::get_block();
                if let LogicStatement::GetBlock {
                    layer,
                    result,
                    x,
                    y,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *layer = TileLayer::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *result = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *x = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *y = tokens[4].clone();
                    }
                }
                statement
            }
            "setblock" => {
                let mut statement = Self::set_block();
                if let LogicStatement::SetBlock {
                    layer,
                    block,
                    x,
                    y,
                    team,
                    rotation,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *layer = TileLayer::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *block = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *x = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *y = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *team = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *rotation = tokens[6].clone();
                    }
                }
                statement
            }
            "spawn" => {
                let mut statement = Self::spawn_unit();
                if let LogicStatement::SpawnUnit {
                    type_,
                    x,
                    y,
                    rotation,
                    team,
                    result,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *x = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *y = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *rotation = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *team = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *result = tokens[6].clone();
                    }
                }
                statement
            }
            "status" => {
                let mut statement = Self::apply_status();
                if let LogicStatement::ApplyStatus {
                    clear,
                    effect,
                    unit,
                    duration,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *clear = java_boolean_value_of(&tokens[1]);
                    }
                    if tokens.len() > 2 {
                        *effect = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *unit = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *duration = tokens[4].clone();
                    }
                }
                statement
            }
            "spawnwave" => {
                let mut statement = Self::spawn_wave();
                if let LogicStatement::SpawnWave { x, y, natural } = &mut statement {
                    if tokens.len() > 1 {
                        *x = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *y = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *natural = tokens[3].clone();
                    }
                }
                statement
            }
            "bullet" => {
                let mut statement = Self::spawn_bullet();
                if let LogicStatement::SpawnBullet {
                    result,
                    from,
                    index,
                    x,
                    y,
                    rotation,
                    team,
                    owner,
                    damage,
                    velocity_scl,
                    life_scl,
                    aim_x,
                    aim_y,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *result = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *from = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *index = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *x = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *y = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *rotation = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *team = tokens[7].clone();
                    }
                    if tokens.len() > 8 {
                        *owner = tokens[8].clone();
                    }
                    if tokens.len() > 9 {
                        *damage = tokens[9].clone();
                    }
                    if tokens.len() > 10 {
                        *velocity_scl = tokens[10].clone();
                    }
                    if tokens.len() > 11 {
                        *life_scl = tokens[11].clone();
                    }
                    if tokens.len() > 12 {
                        *aim_x = tokens[12].clone();
                    }
                    if tokens.len() > 13 {
                        *aim_y = tokens[13].clone();
                    }
                }
                statement
            }
            "weathersense" => {
                let mut statement = Self::weather_sense();
                if let LogicStatement::WeatherSense { to, weather } = &mut statement {
                    if tokens.len() > 1 {
                        *to = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *weather = tokens[2].clone();
                    }
                }
                statement
            }
            "weatherset" => {
                let mut statement = Self::weather_set();
                if let LogicStatement::WeatherSet { weather, state } = &mut statement {
                    if tokens.len() > 1 {
                        *weather = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *state = tokens[2].clone();
                    }
                }
                statement
            }
            "effect" => {
                let mut statement = Self::effect();
                if let LogicStatement::Effect {
                    type_,
                    x,
                    y,
                    sizerot,
                    color,
                    data,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *x = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *y = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *sizerot = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *color = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *data = tokens[6].clone();
                    }
                }
                statement
            }
            "explosion" => {
                let mut statement = Self::explosion();
                if let LogicStatement::Explosion {
                    team,
                    x,
                    y,
                    radius,
                    damage,
                    air,
                    ground,
                    pierce,
                    effect,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *team = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *x = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *y = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *radius = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *damage = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *air = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *ground = tokens[7].clone();
                    }
                    if tokens.len() > 8 {
                        *pierce = tokens[8].clone();
                    }
                    if tokens.len() > 9 {
                        *effect = tokens[9].clone();
                    }
                }
                statement
            }
            "setrule" => {
                let mut statement = Self::set_rule();
                if let LogicStatement::SetRule {
                    rule,
                    value,
                    p1,
                    p2,
                    p3,
                    p4,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *rule = LogicRule::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *value = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *p1 = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *p2 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *p3 = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *p4 = tokens[6].clone();
                    }
                }
                statement
            }
            "fetch" => {
                let mut statement = Self::fetch();
                if let LogicStatement::Fetch {
                    type_,
                    result,
                    team,
                    index,
                    extra,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = FetchType::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *result = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *team = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *index = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *extra = tokens[5].clone();
                    }
                }
                statement
            }
            "getflag" => {
                let mut statement = Self::get_flag();
                if let LogicStatement::GetFlag { result, flag } = &mut statement {
                    if tokens.len() > 1 {
                        *result = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *flag = tokens[2].clone();
                    }
                }
                statement
            }
            "setflag" => {
                let mut statement = Self::set_flag();
                if let LogicStatement::SetFlag { flag, value } = &mut statement {
                    if tokens.len() > 1 {
                        *flag = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *value = tokens[2].clone();
                    }
                }
                statement
            }
            "setprop" => {
                let mut statement = Self::set_prop();
                if let LogicStatement::SetProp { type_, of, value } = &mut statement {
                    if tokens.len() > 1 {
                        *type_ = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *of = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *value = tokens[3].clone();
                    }
                }
                statement
            }
            "message" => {
                let mut statement = Self::flush_message();
                if let LogicStatement::FlushMessage {
                    type_,
                    duration,
                    out_success,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = MessageType::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *duration = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *out_success = tokens[3].clone();
                    }
                }
                statement
            }
            "cutscene" => {
                let mut statement = Self::cutscene();
                if let LogicStatement::Cutscene {
                    action,
                    p1,
                    p2,
                    p3,
                    p4,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *action = CutsceneAction::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *p1 = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *p2 = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *p3 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *p4 = tokens[5].clone();
                    }
                }
                statement
            }
            "clientdata" => {
                let mut statement = Self::client_data();
                if let LogicStatement::ClientData {
                    channel,
                    value,
                    reliable,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *channel = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *value = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *reliable = tokens[3].clone();
                    }
                }
                statement
            }
            "playsound" => {
                let mut statement = Self::play_sound();
                if let LogicStatement::PlaySound {
                    positional,
                    id,
                    volume,
                    pitch,
                    pan,
                    x,
                    y,
                    limit,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *positional = java_boolean_value_of(&tokens[1]);
                    }
                    if tokens.len() > 2 {
                        *id = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *volume = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *pitch = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *pan = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *x = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *y = tokens[7].clone();
                    }
                    if tokens.len() > 8 {
                        *limit = tokens[8].clone();
                    }
                }
                statement
            }
            "setmarker" => {
                let mut statement = Self::set_marker();
                if let LogicStatement::SetMarker {
                    type_,
                    id,
                    p1,
                    p2,
                    p3,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = LMarkerControl::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *id = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *p1 = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *p2 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *p3 = tokens[5].clone();
                    }
                }
                statement
            }
            "makemarker" => {
                let mut statement = Self::make_marker();
                if let LogicStatement::MakeMarker {
                    type_,
                    id,
                    x,
                    y,
                    replace,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *id = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *x = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *y = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *replace = tokens[5].clone();
                    }
                }
                statement
            }
            _ => return None,
        })
    }
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

fn java_boolean_value_of(value: &str) -> bool {
    value.eq_ignore_ascii_case("true")
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

#[derive(Debug, Clone, PartialEq)]
pub struct LogicMemoryObject {
    pub memory: Vec<f64>,
    pub team: u8,
    pub block_privileged: bool,
    pub valid: bool,
}

impl LogicMemoryObject {
    pub fn new(capacity: usize, team: u8) -> Self {
        Self {
            memory: vec![0.0; capacity],
            team,
            block_privileged: false,
            valid: true,
        }
    }

    pub fn readable_by(&self, exec_privileged: bool, exec_team: u8) -> bool {
        self.valid && (exec_privileged || (self.team == exec_team && !self.block_privileged))
    }

    pub fn read(&self, position: &LVar, output: &mut LVar) {
        let address = position.numi();
        if address < 0 || address as usize >= self.memory.len() {
            output.set_num(f64::NAN);
        } else {
            output.set_num(self.memory[address as usize]);
        }
    }

    pub fn write(&mut self, position: &LVar, value: &LVar) {
        let address = position.numi();
        if address < 0 || address as usize >= self.memory.len() {
            return;
        }
        self.memory[address as usize] = value.num();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicSenseObject {
    pub numeric_senses: BTreeMap<LAccess, f64>,
    pub object_senses: BTreeMap<LAccess, Option<String>>,
    pub content_senses: BTreeMap<String, f64>,
}

impl Default for LogicSenseObject {
    fn default() -> Self {
        Self {
            numeric_senses: BTreeMap::new(),
            object_senses: BTreeMap::new(),
            content_senses: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicRuntimeObject {
    Text(String),
    Sequence(Vec<LVarValue>),
    Memory(LogicMemoryObject),
    Senseable(LogicSenseObject),
}

impl LogicRuntimeObject {
    fn read_runtime(
        &self,
        exec_privileged: bool,
        exec_team: u8,
        position: &LVar,
        output: &mut LVar,
    ) -> bool {
        match self {
            LogicRuntimeObject::Text(value) => {
                read_logic_text(value, position, output);
                true
            }
            LogicRuntimeObject::Sequence(values) => {
                read_logic_sequence(values, position, output);
                true
            }
            LogicRuntimeObject::Memory(memory) => {
                if memory.readable_by(exec_privileged, exec_team) {
                    memory.read(position, output);
                } else {
                    output.set_obj(None);
                }
                true
            }
            LogicRuntimeObject::Senseable(_) => false,
        }
    }

    fn write_runtime(
        &mut self,
        exec_privileged: bool,
        exec_team: u8,
        position: &LVar,
        value: &LVar,
    ) -> bool {
        match self {
            LogicRuntimeObject::Memory(memory) => {
                if memory.readable_by(exec_privileged, exec_team) {
                    memory.write(position, value);
                }
                true
            }
            _ => false,
        }
    }

    fn sense_access(&self, access: LAccess) -> Option<LVarValue> {
        match self {
            LogicRuntimeObject::Text(value) => match access {
                LAccess::Size | LAccess::BufferSize => {
                    Some(LVarValue::Number(logic_utf16_len(value) as f64))
                }
                _ => None,
            },
            LogicRuntimeObject::Sequence(values) => match access {
                LAccess::Size | LAccess::BufferSize => Some(LVarValue::Number(values.len() as f64)),
                _ => None,
            },
            LogicRuntimeObject::Memory(memory) => match access {
                LAccess::MemoryCapacity => Some(LVarValue::Number(memory.memory.len() as f64)),
                _ => None,
            },
            LogicRuntimeObject::Senseable(senseable) => {
                if let Some(value) = senseable.object_senses.get(&access) {
                    Some(LVarValue::Object(value.clone()))
                } else {
                    Some(LVarValue::Number(
                        *senseable.numeric_senses.get(&access).unwrap_or(&0.0),
                    ))
                }
            }
        }
    }

    fn sense_content(&self, content_name: &str) -> Option<f64> {
        match self {
            LogicRuntimeObject::Senseable(senseable) => {
                Some(*senseable.content_senses.get(content_name).unwrap_or(&0.0))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicExecutor {
    pub instructions: Vec<LogicInstruction>,
    pub vars: Vec<LVar>,
    pub counter: LVar,
    pub yield_: bool,
    pub privileged: bool,
    pub team: u8,
    pub links: Vec<String>,
    pub objects: BTreeMap<String, LogicRuntimeObject>,
    pub headless: bool,
    pub graphics_buffer: Vec<u64>,
    pub text_buffer: String,
}

impl Default for LogicExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl LogicExecutor {
    pub const MAX_TEXT_BUFFER: usize = 400;
    pub const MAX_GRAPHICS_BUFFER: usize = 256;

    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            vars: Vec::new(),
            counter: {
                let mut counter = LVar::new("@counter");
                counter.is_obj = false;
                counter
            },
            yield_: false,
            privileged: false,
            team: 0,
            links: Vec::new(),
            objects: BTreeMap::new(),
            headless: false,
            graphics_buffer: Vec::new(),
            text_buffer: String::new(),
        }
    }

    pub fn run_once(&mut self) {
        if self.counter.numval >= self.instructions.len() as f64 || self.counter.numval < 0.0 {
            self.counter.numval = 0.0;
        }

        if self.counter.numval < self.instructions.len() as f64 {
            self.counter.is_obj = false;
            let index = self.counter.numval as usize;
            self.counter.numval += 1.0;
            let mut instruction = self.instructions[index].clone();
            instruction.run(self);
            self.instructions[index] = instruction;
        }
    }

    pub fn push_text_bounded(&mut self, value: &str) {
        if self.text_buffer.len() >= Self::MAX_TEXT_BUFFER {
            return;
        }

        let remaining = Self::MAX_TEXT_BUFFER - self.text_buffer.len();
        if value.len() <= remaining {
            self.text_buffer.push_str(value);
            return;
        }

        let mut end = remaining;
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        self.text_buffer.push_str(&value[..end]);
    }

    pub fn register_object(&mut self, name: impl Into<String>, object: LogicRuntimeObject) {
        self.objects.insert(name.into(), object);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicDisplayCommand {
    pub type_: u8,
    pub x: u16,
    pub y: u16,
    pub p1: u16,
    pub p2: u16,
    pub p3: u16,
    pub p4: u16,
}

impl LogicDisplayCommand {
    pub const DISPLAY_DRAW_TYPE: i32 = 30;
    pub const SCALE_STEP: f32 = 0.05;

    pub const fn get(type_: u8, x: u16, y: u16, p1: u16, p2: u16, p3: u16, p4: u16) -> u64 {
        (type_ as u64 & 0x0f)
            | ((x as u64 & 0x03ff) << 4)
            | ((y as u64 & 0x03ff) << 14)
            | ((p1 as u64 & 0x03ff) << 24)
            | ((p2 as u64 & 0x03ff) << 34)
            | ((p3 as u64 & 0x03ff) << 44)
            | ((p4 as u64 & 0x03ff) << 54)
    }

    pub const fn unpack(value: u64) -> Self {
        Self {
            type_: (value & 0x0f) as u8,
            x: ((value >> 4) & 0x03ff) as u16,
            y: ((value >> 14) & 0x03ff) as u16,
            p1: ((value >> 24) & 0x03ff) as u16,
            p2: ((value >> 34) & 0x03ff) as u16,
            p3: ((value >> 44) & 0x03ff) as u16,
            p4: ((value >> 54) & 0x03ff) as u16,
        }
    }

    pub const fn pack(value: i32) -> u16 {
        (value & 0b0111111111) as u16
    }

    pub fn pack_sign(value: i32) -> u16 {
        ((value.abs() & 0b0111111111) | if value < 0 { 0b1000000000 } else { 0 }) as u16
    }

    pub const fn unpack_sign(value: u16) -> i32 {
        ((value & 0b0111111111) as i32) * if (value & 0b1000000000) != 0 { -1 } else { 1 }
    }

    pub fn from_draw_instruction(
        type_: GraphicsType,
        x: &LVar,
        y: &LVar,
        p1: &LVar,
        p2: &LVar,
        p3: &LVar,
        p4: &LVar,
    ) -> Option<u64> {
        let type_id = type_.ordinal();
        if type_ == GraphicsType::Col {
            let rgba = double_bits_to_rgba(x.num());
            return Some(Self::get(
                GraphicsType::Color.ordinal(),
                Self::pack(((rgba >> 24) & 0xff) as i32),
                Self::pack(((rgba >> 16) & 0xff) as i32),
                Self::pack(((rgba >> 8) & 0xff) as i32),
                Self::pack((rgba & 0xff) as i32),
                0,
                0,
            ));
        }

        let mut num1 = Self::pack_sign(p1.numi());
        let mut num4 = Self::pack_sign(p4.numi());
        let mut xval = Self::pack_sign(x.numi());
        let mut yval = Self::pack_sign(y.numi());

        if type_ == GraphicsType::Image {
            let packed = -1;
            num1 = (packed & 0x3ff) as u16;
            num4 = ((packed >> 10) & 0x3ff) as u16;
        } else if type_ == GraphicsType::Scale {
            xval = Self::pack_sign((x.numf() / Self::SCALE_STEP) as i32);
            yval = Self::pack_sign((y.numf() / Self::SCALE_STEP) as i32);
        } else if type_ == GraphicsType::Print {
            return None;
        }

        Some(Self::get(
            type_id,
            xval,
            yval,
            num1,
            Self::pack_sign(p2.numi()),
            Self::pack_sign(p3.numi()),
            num4,
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicInstruction {
    Draw {
        type_: GraphicsType,
        x: LVar,
        y: LVar,
        p1: LVar,
        p2: LVar,
        p3: LVar,
        p4: LVar,
    },
    Set {
        from: LVar,
        to: LVar,
    },
    Op {
        op: LogicOp,
        a: LVar,
        b: LVar,
        dest: LVar,
    },
    Select {
        op: ConditionOp,
        result: LVar,
        comp0: LVar,
        comp1: LVar,
        a: LVar,
        b: LVar,
    },
    End,
    Noop,
    Print {
        value: LVar,
    },
    PrintChar {
        value: LVar,
    },
    Format {
        value: LVar,
    },
    Jump {
        op: ConditionOp,
        value: LVar,
        compare: LVar,
        address: i32,
    },
    Wait {
        value: LVar,
        cur_time: f32,
    },
    Stop,
    GetLink {
        output: LVar,
        index: LVar,
    },
    Read {
        target: LVar,
        position: LVar,
        output: LVar,
    },
    Write {
        target: LVar,
        position: LVar,
        value: LVar,
    },
    Sense {
        from: LVar,
        to: LVar,
        type_: LVar,
    },
    Lookup {
        dest: LVar,
        from: LVar,
        type_: ContentType,
    },
    PackColor {
        result: LVar,
        r: LVar,
        g: LVar,
        b: LVar,
        a: LVar,
    },
    UnpackColor {
        r: LVar,
        g: LVar,
        b: LVar,
        a: LVar,
        value: LVar,
    },
}

impl LogicInstruction {
    pub fn run(&mut self, exec: &mut LogicExecutor) {
        match self {
            LogicInstruction::Draw {
                type_,
                x,
                y,
                p1,
                p2,
                p3,
                p4,
            } => {
                if exec.headless || exec.graphics_buffer.len() >= LogicExecutor::MAX_GRAPHICS_BUFFER
                {
                    return;
                }

                if *type_ == GraphicsType::Print {
                    exec.text_buffer.clear();
                    return;
                }

                if let Some(command) =
                    LogicDisplayCommand::from_draw_instruction(*type_, x, y, p1, p2, p3, p4)
                {
                    exec.graphics_buffer.push(command);
                }
            }
            LogicInstruction::Set { from, to } => {
                if !to.constant {
                    to.set_from(from);
                }
            }
            LogicInstruction::Op { op, a, b, dest } => {
                if dest.constant {
                    return;
                }

                if *op == LogicOp::StrictEqual {
                    dest.set_num(logic_var_strict_equal(a, b) as u8 as f64);
                } else if op.unary() {
                    if let Some(value) = op.eval_unary(a.num()) {
                        dest.set_num(value);
                    }
                } else if let Some(value) = op.eval_binary(a.num(), b.num()) {
                    dest.set_num(value);
                }
            }
            LogicInstruction::Select {
                op,
                result,
                comp0,
                comp1,
                a,
                b,
            } => {
                if result.constant {
                    return;
                }

                if condition_op_test_vars(*op, comp0, comp1) {
                    result.set_from(a);
                } else {
                    result.set_from(b);
                }
            }
            LogicInstruction::End => {
                exec.counter.numval = exec.instructions.len() as f64;
            }
            LogicInstruction::Noop => {}
            LogicInstruction::Print { value } => {
                let text = print_logic_value(value);
                exec.push_text_bounded(&text);
            }
            LogicInstruction::PrintChar { value } => {
                if exec.text_buffer.len() >= LogicExecutor::MAX_TEXT_BUFFER {
                    return;
                }

                if value.is_obj {
                    return;
                }

                let code = value.numval.floor() as u32;
                if let Some(ch) = char::from_u32(code) {
                    exec.push_text_bounded(&ch.to_string());
                }
            }
            LogicInstruction::Format { value } => {
                if exec.text_buffer.len() >= LogicExecutor::MAX_TEXT_BUFFER {
                    return;
                }

                if let Some((index, _number)) = first_logic_placeholder(&exec.text_buffer) {
                    let text = print_logic_value(value);
                    exec.text_buffer.replace_range(index..index + 3, &text);
                }
            }
            LogicInstruction::Jump {
                op,
                value,
                compare,
                address,
            } => {
                if *address != -1 && condition_op_test_vars(*op, value, compare) {
                    exec.counter.numval = *address as f64;
                }
            }
            LogicInstruction::Wait { value, cur_time } => {
                let seconds = value.num();
                if seconds <= 0.0 {
                    exec.yield_ = true;
                    *cur_time = 0.0;
                } else if *cur_time as f64 >= seconds {
                    *cur_time = 0.0;
                } else {
                    exec.counter.numval -= 1.0;
                    exec.yield_ = true;
                    *cur_time += 1.0 / 60.0;
                }
            }
            LogicInstruction::Stop => {
                exec.counter.numval -= 1.0;
                exec.yield_ = true;
            }
            LogicInstruction::GetLink { output, index } => {
                let address = index.numi();
                output.set_obj(
                    (address >= 0)
                        .then(|| exec.links.get(address as usize).cloned())
                        .flatten(),
                );
            }
            LogicInstruction::Read {
                target,
                position,
                output,
            } => {
                exec_read_runtime(exec, target, position, output);
            }
            LogicInstruction::Write {
                target,
                position,
                value,
            } => {
                exec_write_runtime(exec, target, position, value);
            }
            LogicInstruction::Sense { from, to, type_ } => {
                exec_sense_runtime(exec, from, type_, to);
            }
            LogicInstruction::Lookup { dest, from, type_ } => {
                let value = lookup_logic_content_name(*type_, from.numi());
                dest.set_obj(value.map(str::to_string));
            }
            LogicInstruction::PackColor { result, r, g, b, a } => {
                result.set_num(rgba_to_double_bits(
                    logic_color_channel_to_byte(r.num()),
                    logic_color_channel_to_byte(g.num()),
                    logic_color_channel_to_byte(b.num()),
                    logic_color_channel_to_byte(a.num()),
                ));
            }
            LogicInstruction::UnpackColor { r, g, b, a, value } => {
                let (rv, gv, bv, av) = unpack_double_color(value.num());
                r.set_num(rv);
                g.set_num(gv);
                b.set_num(bv);
                a.set_num(av);
            }
        }
    }
}

pub fn logic_var_strict_equal(a: &LVar, b: &LVar) -> bool {
    a.is_obj == b.is_obj
        && if a.is_obj {
            a.objval == b.objval
        } else {
            a.numval == b.numval
        }
}

pub fn condition_op_test_vars(op: ConditionOp, a: &LVar, b: &LVar) -> bool {
    if a.is_obj {
        if b.is_obj {
            let left = a.objval.as_deref().unwrap_or("");
            let right = b.objval.as_deref().unwrap_or("");
            op.test_values(ConditionValue::Object(left), ConditionValue::Object(right))
        } else {
            op.test_values(
                ConditionValue::Object(a.objval.as_deref().unwrap_or("")),
                ConditionValue::Number(b.num()),
            )
        }
    } else if b.is_obj {
        op.test_values(
            ConditionValue::Number(a.num()),
            ConditionValue::Object(b.objval.as_deref().unwrap_or("")),
        )
    } else {
        op.test_values(
            ConditionValue::Number(a.num()),
            ConditionValue::Number(b.num()),
        )
    }
}

pub fn print_logic_value(value: &LVar) -> String {
    if value.is_obj {
        value.objval.clone().unwrap_or_else(|| "null".into())
    } else if (value.numval - value.numval.round()).abs() < 0.00001 {
        (value.numval.round() as i64).to_string()
    } else {
        value.numval.to_string()
    }
}

pub fn first_logic_placeholder(buffer: &str) -> Option<(usize, u8)> {
    let bytes = buffer.as_bytes();
    let mut best: Option<(usize, u8)> = None;
    for index in 0..bytes.len().saturating_sub(2) {
        if bytes[index] == b'{' && bytes[index + 2] == b'}' {
            let digit = bytes[index + 1];
            if digit.is_ascii_digit() {
                let number = digit - b'0';
                if best.is_none_or(|(_, best_number)| number < best_number) {
                    best = Some((index, number));
                }
            }
        }
    }
    best
}

pub fn logic_utf16_len(value: &str) -> usize {
    value.encode_utf16().count()
}

pub fn logic_utf16_char_code_at(value: &str, index: i32) -> Option<u16> {
    if index < 0 {
        return None;
    }
    value.encode_utf16().nth(index as usize)
}

pub fn set_lvar_value(target: &mut LVar, value: &LVarValue) {
    match value {
        LVarValue::Number(value) => target.set_num(*value),
        LVarValue::Object(value) => target.set_obj(value.clone()),
    }
}

pub fn read_logic_text(value: &str, position: &LVar, output: &mut LVar) {
    if let Some(code) = logic_utf16_char_code_at(value, position.numi()) {
        output.set_num(code as f64);
    } else {
        output.set_num(f64::NAN);
    }
}

pub fn read_logic_sequence(values: &[LVarValue], position: &LVar, output: &mut LVar) {
    let address = position.numi();
    if address < 0 || address as usize >= values.len() {
        output.set_obj(None);
    } else {
        set_lvar_value(output, &values[address as usize]);
    }
}

pub fn exec_read_runtime(exec: &LogicExecutor, target: &LVar, position: &LVar, output: &mut LVar) {
    if let Some(name) = target.obj() {
        if let Some(object) = exec.objects.get(name) {
            if object.read_runtime(exec.privileged, exec.team, position, output) {
                return;
            }
        }
    }

    output.set_obj(None);
}

pub fn exec_write_runtime(exec: &mut LogicExecutor, target: &LVar, position: &LVar, value: &LVar) {
    let privileged = exec.privileged;
    let team = exec.team;
    if let Some(name) = target.obj() {
        if let Some(object) = exec.objects.get_mut(name) {
            object.write_runtime(privileged, team, position, value);
        }
    }
}

pub fn exec_sense_runtime(exec: &LogicExecutor, from: &LVar, type_: &LVar, to: &mut LVar) {
    let target_name = from.obj();
    let sense_obj = type_.obj();

    if target_name.is_none() && sense_obj == Some("@dead") {
        to.set_num(1.0);
        return;
    }

    if let Some(name) = target_name {
        if let Some(object) = exec.objects.get(name) {
            if let Some(access) = sense_obj.and_then(logic_access_from_object_name) {
                if let Some(value) = object.sense_access(access) {
                    set_lvar_value(to, &value);
                } else {
                    to.set_obj(None);
                }
                return;
            }

            if let Some(content_name) = sense_obj.and_then(logic_content_name_from_object_name) {
                if let Some(value) = object.sense_content(content_name) {
                    to.set_num(value);
                    return;
                }
            }
        }
    }

    to.set_obj(None);
}

pub fn logic_access_from_object_name(name: &str) -> Option<LAccess> {
    name.strip_prefix('@').and_then(LAccess::by_wire_name)
}

pub fn logic_content_name_from_object_name(name: &str) -> Option<&str> {
    name.strip_prefix('@')
}

pub fn logic_color_channel_to_byte(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0) as u8
}

pub fn lookup_logic_content_name(type_: ContentType, id: i32) -> Option<&'static str> {
    if id < 0 {
        return None;
    }

    let id = id as i16;
    let catalog = ContentCatalog::load_base_content();
    let name = match type_ {
        ContentType::Item => catalog
            .item_by_id(id)
            .map(|item| item.base.mappable.name.clone()),
        ContentType::Block => catalog
            .blocks
            .get(id)
            .map(|block| block.base().name.clone()),
        ContentType::Liquid => catalog
            .liquid_by_id(id)
            .map(|liquid| liquid.base.mappable.name.clone()),
        ContentType::Status => catalog
            .status_effect_by_id(id)
            .map(|status| status.base.mappable.name.clone()),
        _ => None,
    }?;
    Some(Box::leak(name.into_boxed_str()))
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphicsType {
    Clear,
    Color,
    Col,
    Stroke,
    Line,
    Rect,
    LineRect,
    Poly,
    LinePoly,
    Triangle,
    Image,
    Print,
    Translate,
    Scale,
    Rotate,
    Reset,
}

impl GraphicsType {
    pub const ALL: [GraphicsType; 16] = [
        GraphicsType::Clear,
        GraphicsType::Color,
        GraphicsType::Col,
        GraphicsType::Stroke,
        GraphicsType::Line,
        GraphicsType::Rect,
        GraphicsType::LineRect,
        GraphicsType::Poly,
        GraphicsType::LinePoly,
        GraphicsType::Triangle,
        GraphicsType::Image,
        GraphicsType::Print,
        GraphicsType::Translate,
        GraphicsType::Scale,
        GraphicsType::Rotate,
        GraphicsType::Reset,
    ];

    pub const WIRE_NAMES: [&'static str; 16] = [
        "clear",
        "color",
        "col",
        "stroke",
        "line",
        "rect",
        "lineRect",
        "poly",
        "linePoly",
        "triangle",
        "image",
        "print",
        "translate",
        "scale",
        "rotate",
        "reset",
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
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
    fn simple_logic_statements_keep_java_generated_field_order_defaults_and_privilege() {
        assert_eq!(LogicStatement::invalid().write_line(), "noop");
        assert_eq!(LogicStatement::read().write_line(), "read result cell1 0");
        assert_eq!(LogicStatement::write().write_line(), "write result cell1 0");
        assert_eq!(
            LogicStatement::draw().write_line(),
            "draw clear 0 0 0 0 0 0"
        );
        assert_eq!(LogicStatement::print().write_line(), "print \"frog\"");
        assert_eq!(LogicStatement::print_char().write_line(), "printchar 65");
        assert_eq!(LogicStatement::format().write_line(), "format \"frog\"");
        assert_eq!(
            LogicStatement::locale_print().write_line(),
            "localeprint \"name\""
        );
        assert_eq!(
            LogicStatement::draw_flush().write_line(),
            "drawflush display1"
        );
        assert_eq!(
            LogicStatement::print_flush().write_line(),
            "printflush message1"
        );
        assert_eq!(LogicStatement::get_link().write_line(), "getlink result 0");
        assert_eq!(LogicStatement::set_rate().write_line(), "setrate 10");
        assert_eq!(LogicStatement::sync().write_line(), "sync var");
        assert_eq!(LogicStatement::set().write_line(), "set result 0");
        assert_eq!(
            LogicStatement::operation().write_line(),
            "op add result a b"
        );
        assert_eq!(
            LogicStatement::select().write_line(),
            "select result notEqual x false a b"
        );
        assert_eq!(LogicStatement::wait().write_line(), "wait 0.5");
        assert_eq!(LogicStatement::stop().write_line(), "stop");
        assert_eq!(LogicStatement::end().write_line(), "end");
        assert_eq!(
            LogicStatement::pack_color().write_line(),
            "packcolor result 1 0 0 1"
        );
        assert_eq!(
            LogicStatement::unpack_color().write_line(),
            "unpackcolor r g b a color"
        );
        assert_eq!(
            LogicStatement::lookup().write_line(),
            "lookup item result 0"
        );
        assert_eq!(
            LogicStatement::jump().write_line(),
            "jump 0 notEqual x false"
        );
        assert_eq!(
            LogicStatement::control().write_line(),
            "control enabled block1 0 0 0 0"
        );
        assert_eq!(
            LogicStatement::radar().write_line(),
            "radar enemy any any distance turret1 1 result"
        );
        assert_eq!(
            LogicStatement::sensor().write_line(),
            "sensor result block1 @copper"
        );
        assert_eq!(LogicStatement::unit_bind().write_line(), "ubind @poly");
        assert_eq!(
            LogicStatement::unit_control().write_line(),
            "ucontrol move 0 0 0 0 0"
        );
        assert_eq!(
            LogicStatement::unit_radar().write_line(),
            "uradar enemy any any distance 0 1 result"
        );
        assert_eq!(
            LogicStatement::unit_locate().write_line(),
            "ulocate building core true @copper outx outy found building"
        );
        assert_eq!(
            LogicStatement::query().write_line(),
            "query circle unit null 0 0 10 10"
        );
        assert_eq!(
            LogicStatement::get_block().write_line(),
            "getblock block result 0 0"
        );
        assert_eq!(
            LogicStatement::set_block().write_line(),
            "setblock block @air 0 0 @derelict 0"
        );
        assert_eq!(
            LogicStatement::spawn_unit().write_line(),
            "spawn @dagger 10 10 90 @sharded result"
        );
        assert_eq!(
            LogicStatement::apply_status().write_line(),
            "status false wet unit 10"
        );
        assert_eq!(
            LogicStatement::spawn_wave().write_line(),
            "spawnwave 10 10 false"
        );
        assert_eq!(
            LogicStatement::spawn_bullet().write_line(),
            "bullet result @dagger 0 x y angle null null -1 1 1 -1 -1"
        );
        assert_eq!(
            LogicStatement::weather_sense().write_line(),
            "weathersense result @rain"
        );
        assert_eq!(
            LogicStatement::weather_set().write_line(),
            "weatherset @rain true"
        );
        assert_eq!(
            LogicStatement::effect().write_line(),
            "effect warn 0 0 2 %ffaaff "
        );
        assert_eq!(
            LogicStatement::explosion().write_line(),
            "explosion @crux 0 0 5 50 true true false true"
        );
        assert_eq!(
            LogicStatement::set_rule().write_line(),
            "setrule waveSpacing 10 0 0 100 100"
        );
        assert_eq!(
            LogicStatement::fetch().write_line(),
            "fetch unit result @sharded 0 @conveyor"
        );
        assert_eq!(
            LogicStatement::get_flag().write_line(),
            "getflag result \"flag\""
        );
        assert_eq!(
            LogicStatement::set_flag().write_line(),
            "setflag \"flag\" true"
        );
        assert_eq!(
            LogicStatement::set_prop().write_line(),
            "setprop @copper block1 0"
        );
        assert_eq!(
            LogicStatement::flush_message().write_line(),
            "message announce 3 @wait"
        );
        assert_eq!(
            LogicStatement::cutscene().write_line(),
            "cutscene pan 100 100 0.06 0"
        );
        assert_eq!(
            LogicStatement::client_data().write_line(),
            "clientdata \"frog\" \"bar\" 0"
        );
        assert_eq!(
            LogicStatement::play_sound().write_line(),
            "playsound false @sfx-shoot 1 1 0 @thisx @thisy true"
        );
        assert_eq!(
            LogicStatement::set_marker().write_line(),
            "setmarker pos 0 0 0 0"
        );
        assert_eq!(
            LogicStatement::make_marker().write_line(),
            "makemarker shape 0 0 0 true"
        );

        assert_eq!(LogicStatement::read().category().name, "io");
        assert_eq!(LogicStatement::draw().category().name, "io");
        assert_eq!(LogicStatement::print_char().category().name, "io");
        assert_eq!(LogicStatement::draw_flush().category().name, "block");
        assert_eq!(LogicStatement::set_rate().category().name, "world");
        assert_eq!(LogicStatement::operation().category().name, "operation");
        assert_eq!(LogicStatement::select().category().name, "operation");
        assert_eq!(LogicStatement::lookup().category().name, "operation");
        assert_eq!(LogicStatement::wait().category().name, "control");
        assert_eq!(LogicStatement::jump().category().name, "control");
        assert_eq!(LogicStatement::control().category().name, "block");
        assert_eq!(LogicStatement::radar().category().name, "block");
        assert_eq!(LogicStatement::sensor().category().name, "block");
        assert_eq!(LogicStatement::unit_bind().category().name, "unit");
        assert_eq!(LogicStatement::unit_control().category().name, "unit");
        assert_eq!(LogicStatement::unit_radar().category().name, "unit");
        assert_eq!(LogicStatement::unit_locate().category().name, "unit");
        assert_eq!(LogicStatement::query().category().name, "world");
        assert_eq!(LogicStatement::get_block().category().name, "world");
        assert_eq!(LogicStatement::set_block().category().name, "world");
        assert_eq!(LogicStatement::spawn_unit().category().name, "world");
        assert_eq!(LogicStatement::apply_status().category().name, "world");
        assert_eq!(LogicStatement::spawn_wave().category().name, "world");
        assert_eq!(LogicStatement::spawn_bullet().category().name, "world");
        assert_eq!(LogicStatement::weather_sense().category().name, "world");
        assert_eq!(LogicStatement::weather_set().category().name, "world");
        assert_eq!(LogicStatement::effect().category().name, "world");
        assert_eq!(LogicStatement::explosion().category().name, "world");
        assert_eq!(LogicStatement::set_rule().category().name, "world");
        assert_eq!(LogicStatement::fetch().category().name, "world");
        assert_eq!(LogicStatement::get_flag().category().name, "world");
        assert_eq!(LogicStatement::set_flag().category().name, "world");
        assert_eq!(LogicStatement::set_prop().category().name, "world");
        assert_eq!(LogicStatement::flush_message().category().name, "world");
        assert_eq!(LogicStatement::cutscene().category().name, "world");
        assert_eq!(LogicStatement::client_data().category().name, "world");
        assert_eq!(LogicStatement::play_sound().category().name, "world");
        assert_eq!(LogicStatement::set_marker().category().name, "world");
        assert_eq!(LogicStatement::make_marker().category().name, "world");
        assert!(!LogicStatement::read().privileged());
        assert!(!LogicStatement::draw().privileged());
        assert!(!LogicStatement::print_char().privileged());
        assert!(!LogicStatement::operation().privileged());
        assert!(!LogicStatement::select().privileged());
        assert!(!LogicStatement::stop().privileged());
        assert!(!LogicStatement::lookup().privileged());
        assert!(!LogicStatement::jump().privileged());
        assert!(!LogicStatement::sensor().privileged());
        assert!(!LogicStatement::unit_bind().privileged());
        assert!(!LogicStatement::unit_control().privileged());
        assert!(!LogicStatement::unit_radar().privileged());
        assert!(!LogicStatement::unit_locate().privileged());
        assert!(LogicStatement::query().privileged());
        assert!(LogicStatement::set_rate().privileged());
        assert!(LogicStatement::sync().privileged());
        assert!(LogicStatement::locale_print().privileged());
        assert!(LogicStatement::get_block().privileged());
        assert!(LogicStatement::set_block().privileged());
        assert!(LogicStatement::spawn_unit().privileged());
        assert!(LogicStatement::apply_status().privileged());
        assert!(LogicStatement::spawn_wave().privileged());
        assert!(LogicStatement::spawn_bullet().privileged());
        assert!(LogicStatement::weather_sense().privileged());
        assert!(LogicStatement::weather_set().privileged());
        assert!(LogicStatement::effect().privileged());
        assert!(LogicStatement::explosion().privileged());
        assert!(LogicStatement::set_rule().privileged());
        assert!(LogicStatement::fetch().privileged());
        assert!(LogicStatement::get_flag().privileged());
        assert!(LogicStatement::set_flag().privileged());
        assert!(LogicStatement::set_prop().privileged());
        assert!(LogicStatement::flush_message().privileged());
        assert!(LogicStatement::cutscene().privileged());
        assert!(LogicStatement::client_data().privileged());
        assert!(LogicStatement::play_sound().privileged());
        assert!(LogicStatement::set_marker().privileged());
        assert!(LogicStatement::make_marker().privileged());
    }

    #[test]
    fn simple_logic_statements_parse_tokens_like_generated_logic_io() {
        assert_eq!(
            LogicStatement::read_tokens(&["read", "out", "cell2", "7"].map(String::from)),
            Some(LogicStatement::Read {
                output: "out".into(),
                target: "cell2".into(),
                address: "7".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["read", "out"].map(String::from)),
            Some(LogicStatement::Read {
                output: "out".into(),
                target: "cell1".into(),
                address: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["write", "value", "cell3", "2"].map(String::from))
                .unwrap()
                .write_line(),
            "write value cell3 2"
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["draw", "line", "1", "2", "3", "4", "5", "6"].map(String::from)
            ),
            Some(LogicStatement::Draw {
                type_: GraphicsType::Line,
                x: "1".into(),
                y: "2".into(),
                p1: "3".into(),
                p2: "4".into(),
                p3: "5".into(),
                p4: "6".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["draw", "color", "1", "2", "3", "0"].map(String::from))
                .unwrap()
                .write_line(),
            "draw color 1 2 3 255 0 0"
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["draw", "print", "1", "2", "bottomLeft"].map(String::from)
            )
            .unwrap()
            .write_line(),
            "draw print 1 2 @bottomLeft 0 0 0"
        );
        assert_eq!(
            LogicStatement::read_tokens(&["draw", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(&["print", "\"hello world\""].map(String::from))
                .unwrap()
                .tokens(),
            vec!["print".to_string(), "\"hello world\"".to_string()]
        );
        assert_eq!(
            LogicStatement::read_tokens(&["printchar", "97"].map(String::from)),
            Some(LogicStatement::PrintChar { value: "97".into() })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["format", "\"x=%d\""].map(String::from)),
            Some(LogicStatement::Format {
                value: "\"x=%d\"".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["localeprint", "\"unit.name\""].map(String::from)),
            Some(LogicStatement::LocalePrint {
                value: "\"unit.name\"".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["drawflush", "display2"].map(String::from)),
            Some(LogicStatement::DrawFlush {
                target: "display2".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["printflush", "message2"].map(String::from)),
            Some(LogicStatement::PrintFlush {
                target: "message2".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["getlink", "linked", "3"].map(String::from)),
            Some(LogicStatement::GetLink {
                output: "linked".into(),
                address: "3".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setrate", "20"].map(String::from)),
            Some(LogicStatement::SetRate {
                amount: "20".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["sync", "flag"].map(String::from)),
            Some(LogicStatement::Sync {
                variable: "flag".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["set", "x"].map(String::from)),
            Some(LogicStatement::Set {
                to: "x".into(),
                from: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["op", "lessThanEq", "out", "a", "b"].map(String::from)),
            Some(LogicStatement::Operation {
                op: LogicOp::LessThanEq,
                dest: "out".into(),
                a: "a".into(),
                b: "b".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["op", "angleDiff", "result", "x"].map(String::from)),
            Some(LogicStatement::Operation {
                op: LogicOp::AngleDiff,
                dest: "result".into(),
                a: "x".into(),
                b: "b".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["op", "missing", "out", "a", "b"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "select",
                    "out",
                    "greaterThanEq",
                    "hp",
                    "10",
                    "alive",
                    "dead"
                ]
                .map(String::from)
            ),
            Some(LogicStatement::Select {
                result: "out".into(),
                op: ConditionOp::GreaterThanEq,
                comp0: "hp".into(),
                comp1: "10".into(),
                a: "alive".into(),
                b: "dead".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["select", "out"].map(String::from)),
            Some(LogicStatement::Select {
                result: "out".into(),
                op: ConditionOp::NotEqual,
                comp0: "x".into(),
                comp1: "false".into(),
                a: "a".into(),
                b: "b".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["select", "out", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(&["wait", "2"].map(String::from)),
            Some(LogicStatement::Wait { value: "2".into() })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["stop".into()]),
            Some(LogicStatement::Stop)
        );
        assert_eq!(
            LogicStatement::read_tokens(&["end".into()]),
            Some(LogicStatement::End)
        );
        assert_eq!(
            LogicStatement::read_tokens(&["packcolor", "c", "0.1", "0.2"].map(String::from)),
            Some(LogicStatement::PackColor {
                result: "c".into(),
                r: "0.1".into(),
                g: "0.2".into(),
                b: "0".into(),
                a: "1".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["unpackcolor", "red", "green", "blue", "alpha", "packed"].map(String::from)
            ),
            Some(LogicStatement::UnpackColor {
                r: "red".into(),
                g: "green".into(),
                b: "blue".into(),
                a: "alpha".into(),
                value: "packed".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["lookup", "block", "out", "7"].map(String::from)),
            Some(LogicStatement::Lookup {
                type_: ContentType::Block,
                result: "out".into(),
                id: "7".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["lookup", "bullet", "out", "7"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["jump", "-1", "always", "ignored", "alsoIgnored"].map(String::from)
            ),
            Some(LogicStatement::Jump {
                dest_index: -1,
                op: ConditionOp::Always,
                value: "ignored".into(),
                compare: "alsoIgnored".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["control", "shootp", "turret", "@unit", "true"].map(String::from)
            ),
            Some(LogicStatement::Control {
                type_: LAccess::Shootp,
                target: "turret".into(),
                p1: "@unit".into(),
                p2: "true".into(),
                p3: "0".into(),
                p4: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["control", "health", "block1", "1"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "radar",
                    "enemy",
                    "flying",
                    "boss",
                    "maxHealth",
                    "radar1",
                    "-1",
                    "target"
                ]
                .map(String::from)
            ),
            Some(LogicStatement::Radar {
                target1: RadarTarget::Enemy,
                target2: RadarTarget::Flying,
                target3: RadarTarget::Boss,
                sort: RadarSort::MaxHealth,
                radar: "radar1".into(),
                sort_order: "-1".into(),
                output: "target".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["sensor", "out", "turret", "@health"].map(String::from)),
            Some(LogicStatement::Sensor {
                to: "out".into(),
                from: "turret".into(),
                type_: "@health".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["ubind", "@dagger"].map(String::from)),
            Some(LogicStatement::UnitBind {
                type_: "@dagger".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["ucontrol", "build", "1", "2", "@copper", "0", "config"].map(String::from)
            ),
            Some(LogicStatement::UnitControl {
                type_: LUnitControl::Build,
                p1: "1".into(),
                p2: "2".into(),
                p3: "@copper".into(),
                p4: "0".into(),
                p5: "config".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "uradar",
                    "enemy",
                    "flying",
                    "boss",
                    "maxHealth",
                    "0",
                    "-1",
                    "out"
                ]
                .map(String::from)
            ),
            Some(LogicStatement::UnitRadar {
                target1: RadarTarget::Enemy,
                target2: RadarTarget::Flying,
                target3: RadarTarget::Boss,
                sort: RadarSort::MaxHealth,
                radar: "0".into(),
                sort_order: "-1".into(),
                output: "out".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["ulocate", "ore", "core", "false", "@thorium", "x", "y", "found", "build"]
                    .map(String::from)
            ),
            Some(LogicStatement::UnitLocate {
                locate: LLocate::Ore,
                flag: BlockFlag::Core,
                enemy: "false".into(),
                ore: "@thorium".into(),
                out_x: "x".into(),
                out_y: "y".into(),
                out_found: "found".into(),
                out_build: "build".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "ulocate",
                    "building",
                    "launchPad",
                    "true",
                    "@copper",
                    "x",
                    "y",
                    "found",
                    "build"
                ]
                .map(String::from)
            ),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["query", "rect", "building", "@sharded", "1", "2", "3", "4"].map(String::from)
            ),
            Some(LogicStatement::Query {
                shape: QueryShape::Rect,
                type_: QueryType::Building,
                team: "@sharded".into(),
                x: "1".into(),
                y: "2".into(),
                w: "3".into(),
                h: "4".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["query", "circle", "bullet"].map(String::from)),
            Some(LogicStatement::Query {
                shape: QueryShape::Circle,
                type_: QueryType::Bullet,
                team: "null".into(),
                x: "0".into(),
                y: "0".into(),
                w: "10".into(),
                h: "10".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["query", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(&["getblock", "ore", "oreOut", "4", "5"].map(String::from)),
            Some(LogicStatement::GetBlock {
                layer: TileLayer::Ore,
                result: "oreOut".into(),
                x: "4".into(),
                y: "5".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["getblock", "floor"].map(String::from)),
            Some(LogicStatement::GetBlock {
                layer: TileLayer::Floor,
                result: "result".into(),
                x: "0".into(),
                y: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["getblock", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["setblock", "floor", "@sand", "1", "2", "@blue", "3"].map(String::from)
            ),
            Some(LogicStatement::SetBlock {
                layer: TileLayer::Floor,
                block: "@sand".into(),
                x: "1".into(),
                y: "2".into(),
                team: "@blue".into(),
                rotation: "3".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setblock", "ore", "@thorium"].map(String::from)),
            Some(LogicStatement::SetBlock {
                layer: TileLayer::Ore,
                block: "@thorium".into(),
                x: "0".into(),
                y: "0".into(),
                team: "@derelict".into(),
                rotation: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["spawn", "@flare", "7", "8", "45", "@crux", "spawned"].map(String::from)
            ),
            Some(LogicStatement::SpawnUnit {
                type_: "@flare".into(),
                x: "7".into(),
                y: "8".into(),
                rotation: "45".into(),
                team: "@crux".into(),
                result: "spawned".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["spawn", "@mono"].map(String::from)),
            Some(LogicStatement::SpawnUnit {
                type_: "@mono".into(),
                x: "10".into(),
                y: "10".into(),
                rotation: "90".into(),
                team: "@sharded".into(),
                result: "result".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["status", "true", "burning", "@unit", "3"].map(String::from)
            ),
            Some(LogicStatement::ApplyStatus {
                clear: true,
                effect: "burning".into(),
                unit: "@unit".into(),
                duration: "3".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["status", "1"].map(String::from)),
            Some(LogicStatement::ApplyStatus {
                clear: false,
                effect: "wet".into(),
                unit: "unit".into(),
                duration: "10".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["status", "TRUE"].map(String::from))
                .unwrap()
                .write_line(),
            "status true wet unit 10"
        );
        assert_eq!(
            LogicStatement::read_tokens(&["spawnwave", "20", "30", "true"].map(String::from)),
            Some(LogicStatement::SpawnWave {
                x: "20".into(),
                y: "30".into(),
                natural: "true".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["spawnwave"].map(String::from)),
            Some(LogicStatement::SpawnWave {
                x: "10".into(),
                y: "10".into(),
                natural: "false".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "bullet", "out", "@duo", "2", "10", "20", "90", "@sharded", "@unit", "50",
                    "1.5", "0.5", "30", "40"
                ]
                .map(String::from)
            ),
            Some(LogicStatement::SpawnBullet {
                result: "out".into(),
                from: "@duo".into(),
                index: "2".into(),
                x: "10".into(),
                y: "20".into(),
                rotation: "90".into(),
                team: "@sharded".into(),
                owner: "@unit".into(),
                damage: "50".into(),
                velocity_scl: "1.5".into(),
                life_scl: "0.5".into(),
                aim_x: "30".into(),
                aim_y: "40".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["bullet", "out", "@foreshadow"].map(String::from)),
            Some(LogicStatement::SpawnBullet {
                result: "out".into(),
                from: "@foreshadow".into(),
                index: "0".into(),
                x: "x".into(),
                y: "y".into(),
                rotation: "angle".into(),
                team: "null".into(),
                owner: "null".into(),
                damage: "-1".into(),
                velocity_scl: "1".into(),
                life_scl: "1".into(),
                aim_x: "-1".into(),
                aim_y: "-1".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["weathersense", "out", "@sandstorm"].map(String::from)),
            Some(LogicStatement::WeatherSense {
                to: "out".into(),
                weather: "@sandstorm".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["weatherset", "@rain", "false"].map(String::from)),
            Some(LogicStatement::WeatherSet {
                weather: "@rain".into(),
                state: "false".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["effect", "spark", "1", "2", "3", "%ffffff", "payload"].map(String::from)
            ),
            Some(LogicStatement::Effect {
                type_: "spark".into(),
                x: "1".into(),
                y: "2".into(),
                sizerot: "3".into(),
                color: "%ffffff".into(),
                data: "payload".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["effect", "smoke"].map(String::from))
                .unwrap()
                .write_line(),
            "effect smoke 0 0 2 %ffaaff "
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "explosion",
                    "@crux",
                    "5",
                    "6",
                    "7",
                    "8",
                    "false",
                    "true",
                    "true",
                    "false"
                ]
                .map(String::from)
            ),
            Some(LogicStatement::Explosion {
                team: "@crux".into(),
                x: "5".into(),
                y: "6".into(),
                radius: "7".into(),
                damage: "8".into(),
                air: "false".into(),
                ground: "true".into(),
                pierce: "true".into(),
                effect: "false".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["setrule", "mapArea", "1", "2", "3", "4", "5"].map(String::from)
            ),
            Some(LogicStatement::SetRule {
                rule: LogicRule::MapArea,
                value: "1".into(),
                p1: "2".into(),
                p2: "3".into(),
                p3: "4".into(),
                p4: "5".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setrule", "unitCost"].map(String::from)),
            Some(LogicStatement::SetRule {
                rule: LogicRule::UnitCost,
                value: "10".into(),
                p1: "0".into(),
                p2: "0".into(),
                p3: "100".into(),
                p4: "100".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setrule", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["fetch", "build", "out", "@crux", "2", "@duo"].map(String::from)
            ),
            Some(LogicStatement::Fetch {
                type_: FetchType::Build,
                result: "out".into(),
                team: "@crux".into(),
                index: "2".into(),
                extra: "@duo".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["fetch", "unitCount", "count"].map(String::from)),
            Some(LogicStatement::Fetch {
                type_: FetchType::UnitCount,
                result: "count".into(),
                team: "@sharded".into(),
                index: "0".into(),
                extra: "@conveyor".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["fetch", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(&["getflag", "out", "\"waves\""].map(String::from)),
            Some(LogicStatement::GetFlag {
                result: "out".into(),
                flag: "\"waves\"".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setflag", "\"done\"", "false"].map(String::from)),
            Some(LogicStatement::SetFlag {
                flag: "\"done\"".into(),
                value: "false".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setprop", "@health", "@unit", "100"].map(String::from)),
            Some(LogicStatement::SetProp {
                type_: "@health".into(),
                of: "@unit".into(),
                value: "100".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setprop", "@x"].map(String::from)),
            Some(LogicStatement::SetProp {
                type_: "@x".into(),
                of: "block1".into(),
                value: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["message", "toast", "5", "ok"].map(String::from)),
            Some(LogicStatement::FlushMessage {
                type_: MessageType::Toast,
                duration: "5".into(),
                out_success: "ok".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["message", "mission"].map(String::from)),
            Some(LogicStatement::FlushMessage {
                type_: MessageType::Mission,
                duration: "3".into(),
                out_success: "@wait".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["message", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["cutscene", "zoom", "2.5", "ignoredY", "0.1", "extra"].map(String::from)
            ),
            Some(LogicStatement::Cutscene {
                action: CutsceneAction::Zoom,
                p1: "2.5".into(),
                p2: "ignoredY".into(),
                p3: "0.1".into(),
                p4: "extra".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["cutscene", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["clientdata", "\"chan\"", "\"payload\"", "1"].map(String::from)
            ),
            Some(LogicStatement::ClientData {
                channel: "\"chan\"".into(),
                value: "\"payload\"".into(),
                reliable: "1".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["clientdata", "\"chan\""].map(String::from)),
            Some(LogicStatement::ClientData {
                channel: "\"chan\"".into(),
                value: "\"bar\"".into(),
                reliable: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "playsound",
                    "true",
                    "@sfx-explosion",
                    "0.5",
                    "2",
                    "-1",
                    "10",
                    "20",
                    "false"
                ]
                .map(String::from)
            ),
            Some(LogicStatement::PlaySound {
                positional: true,
                id: "@sfx-explosion".into(),
                volume: "0.5".into(),
                pitch: "2".into(),
                pan: "-1".into(),
                x: "10".into(),
                y: "20".into(),
                limit: "false".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["playsound", "1", "@sfx-pew"].map(String::from)),
            Some(LogicStatement::PlaySound {
                positional: false,
                id: "@sfx-pew".into(),
                volume: "1".into(),
                pitch: "1".into(),
                pan: "0".into(),
                x: "@thisx".into(),
                y: "@thisy".into(),
                limit: "true".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["setmarker", "color", "7", "%ff00aa", "0", "0"].map(String::from)
            ),
            Some(LogicStatement::SetMarker {
                type_: LMarkerControl::Color,
                id: "7".into(),
                p1: "%ff00aa".into(),
                p2: "0".into(),
                p3: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setmarker", "shape"].map(String::from)),
            Some(LogicStatement::SetMarker {
                type_: LMarkerControl::Shape,
                id: "0".into(),
                p1: "0".into(),
                p2: "0".into(),
                p3: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setmarker", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["makemarker", "text", "3", "10", "20", "false"].map(String::from)
            ),
            Some(LogicStatement::MakeMarker {
                type_: "text".into(),
                id: "3".into(),
                x: "10".into(),
                y: "20".into(),
                replace: "false".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["makemarker", "line"].map(String::from)),
            Some(LogicStatement::MakeMarker {
                type_: "line".into(),
                id: "0".into(),
                x: "0".into(),
                y: "0".into(),
                replace: "true".into()
            })
        );
        assert_eq!(LogicStatement::read_tokens(&["missing".into()]), None);
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
        assert_eq!(
            LMarkerControl::by_wire_name("drawLayer"),
            Some(LMarkerControl::DrawLayer)
        );
        assert_eq!(LMarkerControl::by_wire_name("missing"), None);
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
        assert_eq!(LAccess::by_wire_name("enabled"), Some(LAccess::Enabled));
        assert_eq!(
            LAccess::by_wire_name("currentAmmoType"),
            Some(LAccess::CurrentAmmoType)
        );
        assert_eq!(LAccess::by_wire_name("missing"), None);

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
        assert_eq!(LogicOp::Add.java_name(), "add");
        assert_eq!(LogicOp::NotEqual.java_name(), "notEqual");
        assert_eq!(LogicOp::AngleDiff.java_name(), "angleDiff");
        assert_eq!(LogicOp::by_java_name("lessThan"), Some(LogicOp::LessThan));
        assert_eq!(LogicOp::by_java_name("angleDiff"), Some(LogicOp::AngleDiff));
        assert_eq!(LogicOp::by_java_name("+"), None);

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
    fn basic_logic_executor_instructions_follow_java_l_executor_semantics() {
        let mut set = LogicInstruction::Set {
            from: {
                let mut from = LVar::new("from");
                from.set_num(7.0);
                from
            },
            to: LVar::new("to"),
        };
        set.run(&mut LogicExecutor::new());
        match set {
            LogicInstruction::Set { to, .. } => assert_eq!(to.value(), LVarValue::Number(7.0)),
            _ => unreachable!(),
        }

        let mut op = LogicInstruction::Op {
            op: LogicOp::Add,
            a: {
                let mut value = LVar::new("a");
                value.set_num(2.0);
                value
            },
            b: {
                let mut value = LVar::new("b");
                value.set_num(3.0);
                value
            },
            dest: LVar::new("dest"),
        };
        op.run(&mut LogicExecutor::new());
        match op {
            LogicInstruction::Op { dest, .. } => assert_eq!(dest.value(), LVarValue::Number(5.0)),
            _ => unreachable!(),
        }

        let mut select = LogicInstruction::Select {
            op: ConditionOp::GreaterThan,
            result: LVar::new("result"),
            comp0: {
                let mut value = LVar::new("hp");
                value.set_num(11.0);
                value
            },
            comp1: {
                let mut value = LVar::new("limit");
                value.set_num(10.0);
                value
            },
            a: {
                let mut value = LVar::new("alive");
                value.set_obj(Some("alive".into()));
                value
            },
            b: {
                let mut value = LVar::new("dead");
                value.set_obj(Some("dead".into()));
                value
            },
        };
        select.run(&mut LogicExecutor::new());
        match select {
            LogicInstruction::Select { result, .. } => {
                assert_eq!(result.value(), LVarValue::Object(Some("alive".into())));
            }
            _ => unreachable!(),
        }

        assert!(logic_var_strict_equal(
            &{
                let mut value = LVar::new("a");
                value.set_obj(Some("same".into()));
                value
            },
            &{
                let mut value = LVar::new("b");
                value.set_obj(Some("same".into()));
                value
            }
        ));
        assert!(!condition_op_test_vars(
            ConditionOp::Equal,
            &{
                let mut value = LVar::new("a");
                value.set_obj(Some("1".into()));
                value
            },
            &{
                let mut value = LVar::new("b");
                value.set_num(1.0);
                value
            }
        ));
    }

    #[test]
    fn text_and_flow_logic_executor_instructions_follow_java_l_executor_semantics() {
        let mut exec = LogicExecutor::new();
        LogicInstruction::Print {
            value: {
                let mut value = LVar::new("n");
                value.set_num(4.0);
                value
            },
        }
        .run(&mut exec);
        LogicInstruction::Print {
            value: {
                let mut value = LVar::new("s");
                value.set_obj(Some(" frogs".into()));
                value
            },
        }
        .run(&mut exec);
        LogicInstruction::PrintChar {
            value: {
                let mut value = LVar::new("bang");
                value.set_num(33.0);
                value
            },
        }
        .run(&mut exec);
        assert_eq!(exec.text_buffer, "4 frogs!");

        exec.text_buffer = "{1} before {0}".into();
        LogicInstruction::Format {
            value: {
                let mut value = LVar::new("value");
                value.set_obj(Some("first".into()));
                value
            },
        }
        .run(&mut exec);
        assert_eq!(exec.text_buffer, "{1} before first");

        assert_eq!(
            print_logic_value(&{
                let mut value = LVar::new("fraction");
                value.set_num(1.25);
                value
            }),
            "1.25"
        );
        assert_eq!(first_logic_placeholder("x {3} {1} {2}"), Some((6, 1)));

        let mut jump_exec = LogicExecutor::new();
        jump_exec.counter.set_num(1.0);
        LogicInstruction::Jump {
            op: ConditionOp::Always,
            value: LVar::new("a"),
            compare: LVar::new("b"),
            address: 5,
        }
        .run(&mut jump_exec);
        assert_eq!(jump_exec.counter.numval, 5.0);

        let mut stop_exec = LogicExecutor::new();
        stop_exec.counter.set_num(3.0);
        LogicInstruction::Stop.run(&mut stop_exec);
        assert_eq!(stop_exec.counter.numval, 2.0);
        assert!(stop_exec.yield_);

        let mut wait = LogicInstruction::Wait {
            value: {
                let mut value = LVar::new("seconds");
                value.set_num(0.5);
                value
            },
            cur_time: 0.0,
        };
        let mut wait_exec = LogicExecutor::new();
        wait_exec.counter.set_num(2.0);
        wait.run(&mut wait_exec);
        assert_eq!(wait_exec.counter.numval, 1.0);
        assert!(wait_exec.yield_);
        match wait {
            LogicInstruction::Wait { cur_time, .. } => {
                assert!((cur_time - 1.0 / 60.0).abs() < 0.000001)
            }
            _ => unreachable!(),
        }

        let mut end_exec = LogicExecutor {
            instructions: vec![LogicInstruction::Noop, LogicInstruction::Noop],
            ..LogicExecutor::new()
        };
        LogicInstruction::End.run(&mut end_exec);
        assert_eq!(end_exec.counter.numval, 2.0);
    }

    #[test]
    fn display_command_packing_and_draw_instruction_follow_java_logic_display_bits() {
        let packed = LogicDisplayCommand::get(4, 1, 2, 3, 4, 5, 6);
        assert_eq!(
            LogicDisplayCommand::unpack(packed),
            LogicDisplayCommand {
                type_: 4,
                x: 1,
                y: 2,
                p1: 3,
                p2: 4,
                p3: 5,
                p4: 6
            }
        );
        assert_eq!(LogicDisplayCommand::pack(1025), 1);
        assert_eq!(LogicDisplayCommand::pack_sign(-12), 0b1000001100);
        assert_eq!(LogicDisplayCommand::unpack_sign(0b1000001100), -12);

        let mut exec = LogicExecutor::new();
        let mut draw = LogicInstruction::Draw {
            type_: GraphicsType::Line,
            x: {
                let mut value = LVar::new("x");
                value.set_num(-1.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(2.0);
                value
            },
            p1: {
                let mut value = LVar::new("p1");
                value.set_num(3.0);
                value
            },
            p2: {
                let mut value = LVar::new("p2");
                value.set_num(4.0);
                value
            },
            p3: {
                let mut value = LVar::new("p3");
                value.set_num(5.0);
                value
            },
            p4: {
                let mut value = LVar::new("p4");
                value.set_num(-6.0);
                value
            },
        };
        draw.run(&mut exec);
        assert_eq!(exec.graphics_buffer.len(), 1);
        let command = LogicDisplayCommand::unpack(exec.graphics_buffer[0]);
        assert_eq!(command.type_, GraphicsType::Line.ordinal());
        assert_eq!(LogicDisplayCommand::unpack_sign(command.x), -1);
        assert_eq!(LogicDisplayCommand::unpack_sign(command.y), 2);
        assert_eq!(LogicDisplayCommand::unpack_sign(command.p1), 3);
        assert_eq!(LogicDisplayCommand::unpack_sign(command.p4), -6);

        let mut color_exec = LogicExecutor::new();
        LogicInstruction::Draw {
            type_: GraphicsType::Col,
            x: {
                let mut value = LVar::new("packed");
                value.set_num(rgba_to_double_bits(0xff, 0x00, 0xaa, 0x80));
                value
            },
            y: LVar::new("y"),
            p1: LVar::new("p1"),
            p2: LVar::new("p2"),
            p3: LVar::new("p3"),
            p4: LVar::new("p4"),
        }
        .run(&mut color_exec);
        let color = LogicDisplayCommand::unpack(color_exec.graphics_buffer[0]);
        assert_eq!(color.type_, GraphicsType::Color.ordinal());
        assert_eq!((color.x, color.y, color.p1, color.p2), (255, 0, 170, 128));
    }

    #[test]
    fn color_and_lookup_executor_instructions_follow_java_l_executor_semantics() {
        let mut pack = LogicInstruction::PackColor {
            result: LVar::new("result"),
            r: {
                let mut value = LVar::new("r");
                value.set_num(1.0);
                value
            },
            g: {
                let mut value = LVar::new("g");
                value.set_num(0.0);
                value
            },
            b: {
                let mut value = LVar::new("b");
                value.set_num(0.5);
                value
            },
            a: {
                let mut value = LVar::new("a");
                value.set_num(2.0);
                value
            },
        };
        pack.run(&mut LogicExecutor::new());
        let packed = match pack {
            LogicInstruction::PackColor { result, .. } => {
                assert_eq!(double_bits_to_rgba(result.numval), 0xff007fff);
                result.numval
            }
            _ => unreachable!(),
        };

        let mut unpack = LogicInstruction::UnpackColor {
            r: LVar::new("r"),
            g: LVar::new("g"),
            b: LVar::new("b"),
            a: LVar::new("a"),
            value: {
                let mut value = LVar::new("value");
                value.set_num(packed);
                value
            },
        };
        unpack.run(&mut LogicExecutor::new());
        match unpack {
            LogicInstruction::UnpackColor { r, g, b, a, .. } => {
                assert_eq!(r.value(), LVarValue::Number(1.0));
                assert_eq!(g.value(), LVarValue::Number(0.0));
                assert!((b.numval - 127.0 / 255.0).abs() < 0.000001);
                assert_eq!(a.value(), LVarValue::Number(1.0));
            }
            _ => unreachable!(),
        }

        assert_eq!(
            lookup_logic_content_name(ContentType::Item, 0),
            Some("copper")
        );
        assert_eq!(
            lookup_logic_content_name(ContentType::Liquid, 0),
            Some("water")
        );
        assert_eq!(
            lookup_logic_content_name(ContentType::Status, 0),
            Some("none")
        );
        assert_eq!(lookup_logic_content_name(ContentType::Item, -1), None);

        let mut lookup = LogicInstruction::Lookup {
            dest: LVar::new("dest"),
            from: {
                let mut value = LVar::new("from");
                value.set_num(0.0);
                value
            },
            type_: ContentType::Item,
        };
        lookup.run(&mut LogicExecutor::new());
        match lookup {
            LogicInstruction::Lookup { dest, .. } => {
                assert_eq!(dest.value(), LVarValue::Object(Some("copper".into())));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn linked_read_write_and_sense_executor_instructions_follow_java_l_executor_semantics() {
        let mut exec = LogicExecutor::new();
        exec.team = 1;
        exec.links = vec!["cell1".into(), "message1".into()];
        exec.register_object(
            "cell1",
            LogicRuntimeObject::Memory(LogicMemoryObject::new(4, 1)),
        );
        exec.register_object(
            "enemy-cell",
            LogicRuntimeObject::Memory(LogicMemoryObject::new(2, 2)),
        );
        exec.register_object("message1", LogicRuntimeObject::Text("ab💥".into()));
        exec.register_object(
            "seq1",
            LogicRuntimeObject::Sequence(vec![
                LVarValue::Object(Some("copper".into())),
                LVarValue::Number(7.0),
            ]),
        );
        let mut sensor = LogicSenseObject::default();
        sensor.numeric_senses.insert(LAccess::Health, 12.5);
        sensor
            .object_senses
            .insert(LAccess::CurrentAmmoType, Some("copper".into()));
        sensor.content_senses.insert("copper".into(), 42.0);
        exec.register_object("turret", LogicRuntimeObject::Senseable(sensor));

        let mut getlink = LogicInstruction::GetLink {
            output: LVar::new("out"),
            index: {
                let mut value = LVar::new("index");
                value.set_num(1.0);
                value
            },
        };
        getlink.run(&mut exec);
        match getlink {
            LogicInstruction::GetLink { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(Some("message1".into())));
            }
            _ => unreachable!(),
        }

        let mut missing_link = LogicInstruction::GetLink {
            output: {
                let mut value = LVar::new("out");
                value.set_obj(Some("old".into()));
                value
            },
            index: {
                let mut value = LVar::new("index");
                value.set_num(-1.0);
                value
            },
        };
        missing_link.run(&mut exec);
        match missing_link {
            LogicInstruction::GetLink { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(None));
            }
            _ => unreachable!(),
        }

        let mut write = LogicInstruction::Write {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("cell1".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(2.0);
                value
            },
            value: {
                let mut value = LVar::new("value");
                value.set_num(9.0);
                value
            },
        };
        write.run(&mut exec);
        assert_eq!(
            match exec.objects.get("cell1").unwrap() {
                LogicRuntimeObject::Memory(memory) => memory.memory[2],
                _ => unreachable!(),
            },
            9.0
        );

        let mut read_memory = LogicInstruction::Read {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("cell1".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(2.0);
                value
            },
            output: LVar::new("output"),
        };
        read_memory.run(&mut exec);
        match read_memory {
            LogicInstruction::Read { output, .. } => {
                assert_eq!(output.value(), LVarValue::Number(9.0));
            }
            _ => unreachable!(),
        }

        let mut read_oob = LogicInstruction::Read {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("cell1".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(99.0);
                value
            },
            output: LVar::new("output"),
        };
        read_oob.run(&mut exec);
        match read_oob {
            LogicInstruction::Read { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(None));
            }
            _ => unreachable!(),
        }

        let mut read_denied = LogicInstruction::Read {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("enemy-cell".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(0.0);
                value
            },
            output: {
                let mut value = LVar::new("output");
                value.set_num(1.0);
                value
            },
        };
        read_denied.run(&mut exec);
        match read_denied {
            LogicInstruction::Read { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(None));
            }
            _ => unreachable!(),
        }

        let mut read_text = LogicInstruction::Read {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("message1".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(1.0);
                value
            },
            output: LVar::new("output"),
        };
        read_text.run(&mut exec);
        match read_text {
            LogicInstruction::Read { output, .. } => {
                assert_eq!(output.value(), LVarValue::Number('b' as u32 as f64));
            }
            _ => unreachable!(),
        }

        let mut read_text_oob = LogicInstruction::Read {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("message1".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(4.0);
                value
            },
            output: LVar::new("output"),
        };
        read_text_oob.run(&mut exec);
        match read_text_oob {
            LogicInstruction::Read { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(None));
            }
            _ => unreachable!(),
        }

        let mut read_seq = LogicInstruction::Read {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("seq1".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(0.0);
                value
            },
            output: LVar::new("output"),
        };
        read_seq.run(&mut exec);
        match read_seq {
            LogicInstruction::Read { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(Some("copper".into())));
            }
            _ => unreachable!(),
        }

        let mut sense_dead = LogicInstruction::Sense {
            from: {
                let mut value = LVar::new("from");
                value.set_obj(None);
                value
            },
            to: LVar::new("to"),
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@dead".into()));
                value
            },
        };
        sense_dead.run(&mut exec);
        match sense_dead {
            LogicInstruction::Sense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Number(1.0));
            }
            _ => unreachable!(),
        }

        let mut sense_text_size = LogicInstruction::Sense {
            from: {
                let mut value = LVar::new("from");
                value.set_obj(Some("message1".into()));
                value
            },
            to: LVar::new("to"),
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@size".into()));
                value
            },
        };
        sense_text_size.run(&mut exec);
        match sense_text_size {
            LogicInstruction::Sense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Number(4.0));
            }
            _ => unreachable!(),
        }

        let mut sense_memory_capacity = LogicInstruction::Sense {
            from: {
                let mut value = LVar::new("from");
                value.set_obj(Some("cell1".into()));
                value
            },
            to: LVar::new("to"),
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@memoryCapacity".into()));
                value
            },
        };
        sense_memory_capacity.run(&mut exec);
        match sense_memory_capacity {
            LogicInstruction::Sense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Number(4.0));
            }
            _ => unreachable!(),
        }

        let mut sense_numeric = LogicInstruction::Sense {
            from: {
                let mut value = LVar::new("from");
                value.set_obj(Some("turret".into()));
                value
            },
            to: LVar::new("to"),
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@health".into()));
                value
            },
        };
        sense_numeric.run(&mut exec);
        match sense_numeric {
            LogicInstruction::Sense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Number(12.5));
            }
            _ => unreachable!(),
        }

        let mut sense_object = LogicInstruction::Sense {
            from: {
                let mut value = LVar::new("from");
                value.set_obj(Some("turret".into()));
                value
            },
            to: LVar::new("to"),
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@currentAmmoType".into()));
                value
            },
        };
        sense_object.run(&mut exec);
        match sense_object {
            LogicInstruction::Sense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Object(Some("copper".into())));
            }
            _ => unreachable!(),
        }

        let mut sense_content = LogicInstruction::Sense {
            from: {
                let mut value = LVar::new("from");
                value.set_obj(Some("turret".into()));
                value
            },
            to: LVar::new("to"),
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@copper".into()));
                value
            },
        };
        sense_content.run(&mut exec);
        match sense_content {
            LogicInstruction::Sense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Number(42.0));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn condition_ops_match_java_order_symbols_and_tests() {
        assert_eq!(ConditionOp::ALL.len(), 8);
        assert_eq!(ConditionOp::Equal.ordinal(), 0);
        assert_eq!(ConditionOp::StrictEqual.ordinal(), 6);
        assert_eq!(ConditionOp::Always.ordinal(), 7);
        assert_eq!(ConditionOp::from_ordinal(7), Some(ConditionOp::Always));
        assert_eq!(ConditionOp::from_ordinal(8), None);
        assert_eq!(ConditionOp::NotEqual.java_name(), "notEqual");
        assert_eq!(
            ConditionOp::by_java_name("lessThanEq"),
            Some(ConditionOp::LessThanEq)
        );
        assert_eq!(ConditionOp::by_java_name("<="), None);

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
        assert_eq!(
            RadarSort::by_wire_name("maxHealth"),
            Some(RadarSort::MaxHealth)
        );
        assert_eq!(RadarSort::by_wire_name("missing"), None);

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
        assert_eq!(
            RadarTarget::by_wire_name("attacker"),
            Some(RadarTarget::Attacker)
        );
        assert_eq!(RadarTarget::by_wire_name("missing"), None);
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

        assert_eq!(GraphicsType::ALL.len(), 16);
        assert_eq!(GraphicsType::Clear.ordinal(), 0);
        assert_eq!(GraphicsType::LineRect.ordinal(), 6);
        assert_eq!(GraphicsType::Reset.ordinal(), 15);
        assert_eq!(GraphicsType::from_ordinal(15), Some(GraphicsType::Reset));
        assert_eq!(GraphicsType::from_ordinal(16), None);
        assert_eq!(GraphicsType::LineRect.wire_name(), "lineRect");
        assert_eq!(
            GraphicsType::by_wire_name("triangle"),
            Some(GraphicsType::Triangle)
        );
        assert_eq!(GraphicsType::by_wire_name("missing"), None);

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
