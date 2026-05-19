// Mirrors upstream core/src/mindustry/logic. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

use std::fmt;

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
