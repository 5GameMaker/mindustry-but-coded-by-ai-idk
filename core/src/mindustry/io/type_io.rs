use std::convert::TryFrom;
use std::io::{self, Read, Write};

use crate::mindustry::content::blocks::BlockDef;
use crate::mindustry::core::content_loader::{ContentLoader, ContentRecord};
use crate::mindustry::ctype::{ContentId, ContentType};
use crate::mindustry::entities::units::{BuildPlan, StatusEntry, WeaponMount};
use crate::mindustry::logic::{LAccess, LMarkerControl};
use crate::mindustry::net::{AdminAction, KickReason, TraceInfo};
use crate::mindustry::r#type::{ItemStack, LiquidStack};
use crate::mindustry::vars::MAX_PLAYER_PREVIEW_PLANS;
use crate::mindustry::world::blocks::payloads::{
    self, PayloadRef, PAYLOAD_BLOCK_TYPE, PAYLOAD_UNIT_TYPE,
};
use crate::mindustry::world::{point2_pack, point2_x, point2_y};

pub const MAX_ARRAY_SIZE: usize = 1000;
pub const MAX_OBJECT_READ_ARRAY_SIZE: usize = 200;
pub const MAX_BYTE_ARRAY_SIZE: usize = 40_000;
pub const MAX_SAFE_STRING_CHARS: usize = 1200;
pub const MAX_NET_BUILD_PLANS: usize = 20;
pub const MAX_NET_BUILD_PLAN_CONFIG_CHARS: usize = 500;
pub const MAX_RULES_BYTES: usize = 100_000;
pub const MAX_OBJECTIVES_BYTES: usize = 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point2 {
    pub x: i32,
    pub y: i32,
}

impl Point2 {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn pack(self) -> i32 {
        point2_pack(self.x, self.y)
    }

    pub fn from_packed(packed: i32) -> Self {
        Self {
            x: point2_x(packed) as i32,
            y: point2_y(packed) as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaColor(pub i32);

impl RgbaColor {
    pub const fn new(rgba: i32) -> Self {
        Self(rgba)
    }

    pub const fn rgba(self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamId(pub u8);

/// Java `TypeIO.writeEntity(...)` / `readEntity(...)` wire value.
/// The value is the sync entity id, with `-1` used as the nullable sentinel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityRef {
    pub id: Option<i32>,
}

impl EntityRef {
    pub const fn new(id: i32) -> Self {
        Self { id: Some(id) }
    }

    pub const fn null() -> Self {
        Self { id: None }
    }
}

/// Java `TypeIO.writeBuilding(...)` / `readBuilding(...)` wire value.
/// The value is the packed building tile position, with `-1` used for null.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingRef {
    pub tile_pos: Option<i32>,
}

impl BuildingRef {
    pub const fn new(tile_pos: i32) -> Self {
        Self {
            tile_pos: Some(tile_pos),
        }
    }

    pub const fn null() -> Self {
        Self { tile_pos: None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentRef {
    pub content_type: ContentType,
    pub id: ContentId,
}

impl ContentRef {
    pub const fn new(content_type: ContentType, id: ContentId) -> Self {
        Self { content_type, id }
    }

    pub fn resolve<'a>(&self, loader: &'a ContentLoader) -> Option<&'a ContentRecord> {
        loader.get_by_id(self.content_type, self.id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeValue {
    Null,
    Int(i32),
    Long(i64),
    Float(f32),
    String(String),
    Content(ContentRef),
    TechNode(ContentRef),
    Bool(bool),
    Double(f64),
    Building(i32),
    LogicAccess(LAccess),
    Unit(i32),
    Point2(Point2),
    Vec2(Vec2),
    Team(u8),
    UnitCommand(ContentId),
    IntSeq(Vec<i32>),
    IntArray(Vec<i32>),
    ByteArray(Vec<u8>),
    Point2Array(Vec<Point2>),
    BoolArray(Vec<bool>),
    Vec2Array(Vec<Vec2>),
    ObjectArray(Vec<TypeValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildPlanWire {
    pub x: i32,
    pub y: i32,
    pub rotation: i32,
    pub block: Option<String>,
    pub breaking: bool,
    pub config: TypeValue,
}

impl BuildPlanWire {
    pub fn new_place(x: i32, y: i32, rotation: i32, block: impl Into<String>) -> Self {
        Self {
            x,
            y,
            rotation,
            block: Some(block.into()),
            breaking: false,
            config: TypeValue::Null,
        }
    }

    pub fn new_place_config(
        x: i32,
        y: i32,
        rotation: i32,
        block: impl Into<String>,
        config: TypeValue,
    ) -> Self {
        Self {
            config,
            ..Self::new_place(x, y, rotation, block)
        }
    }

    pub fn new_break(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            rotation: -1,
            block: None,
            breaking: true,
            config: TypeValue::Null,
        }
    }

    pub fn from_build_plan(plan: &BuildPlan) -> Self {
        Self {
            x: plan.x,
            y: plan.y,
            rotation: plan.rotation,
            block: plan.block.clone(),
            breaking: plan.breaking,
            config: plan.config.clone(),
        }
    }

    pub fn to_build_plan(&self) -> io::Result<BuildPlan> {
        if self.breaking {
            return Ok(BuildPlan::new_break(self.x, self.y));
        }

        let block = self
            .block
            .as_ref()
            .ok_or_else(|| invalid_data("place plan missing block"))?;
        let mut plan = BuildPlan::new_place(self.x, self.y, self.rotation, block.clone());
        plan.config = self.config.clone();
        Ok(plan)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerTarget {
    BuildingPos(i32),
    UnitId(i32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandQueueEntry {
    BuildingPos(i32),
    UnitId(i32),
    Point(Vec2),
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CommandWire {
    pub target_pos: Option<Vec2>,
    pub attack_target: Option<ControllerTarget>,
    pub command_id: Option<ContentId>,
    pub command_queue: Vec<CommandQueueEntry>,
    pub stances: Vec<ContentId>,
}

impl CommandWire {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControllerWire {
    Player { player_id: i32 },
    LegacyFormation { id: i32 },
    Ground,
    Logic { controller_pos: i32 },
    Assembler,
    Command(CommandWire),
}

pub fn write_string<W: Write>(write: &mut W, string: Option<&str>) -> io::Result<()> {
    match string {
        Some(string) => {
            write.write_all(&[1])?;
            write_java_utf(write, string)
        }
        None => write.write_all(&[0]),
    }
}

pub fn read_string<R: Read>(read: &mut R) -> io::Result<Option<String>> {
    let exists = read_u8(read)?;
    if exists != 0 {
        Ok(Some(read_java_utf(read)?))
    } else {
        Ok(None)
    }
}

pub fn write_java_utf<W: Write>(write: &mut W, string: &str) -> io::Result<()> {
    let bytes = encode_java_utf(string)?;
    write_u16(write, bytes.len() as u16)?;
    write.write_all(&bytes)
}

pub fn read_java_utf<R: Read>(read: &mut R) -> io::Result<String> {
    let len = read_u16(read)? as usize;
    let mut bytes = vec![0; len];
    read.read_exact(&mut bytes)?;
    decode_java_utf(&bytes)
}

pub fn encode_java_utf(string: &str) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    for unit in string.encode_utf16() {
        match unit {
            0x0001..=0x007f => out.push(unit as u8),
            0x0000..=0x07ff => {
                out.push((0xc0 | ((unit >> 6) & 0x1f)) as u8);
                out.push((0x80 | (unit & 0x3f)) as u8);
            }
            _ => {
                out.push((0xe0 | ((unit >> 12) & 0x0f)) as u8);
                out.push((0x80 | ((unit >> 6) & 0x3f)) as u8);
                out.push((0x80 | (unit & 0x3f)) as u8);
            }
        }
        if out.len() > u16::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Java modified UTF string exceeds 65535 bytes",
            ));
        }
    }
    Ok(out)
}

pub fn decode_java_utf(bytes: &[u8]) -> io::Result<String> {
    let mut units = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b & 0x80 == 0 {
            if b == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Java modified UTF contains raw NUL byte",
                ));
            }
            units.push(b as u16);
            i += 1;
        } else if (b & 0xe0) == 0xc0 {
            if i + 1 >= bytes.len() || (bytes[i + 1] & 0xc0) != 0x80 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid two-byte Java modified UTF sequence",
                ));
            }
            let unit = (((b & 0x1f) as u16) << 6) | ((bytes[i + 1] & 0x3f) as u16);
            units.push(unit);
            i += 2;
        } else if (b & 0xf0) == 0xe0 {
            if i + 2 >= bytes.len()
                || (bytes[i + 1] & 0xc0) != 0x80
                || (bytes[i + 2] & 0xc0) != 0x80
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid three-byte Java modified UTF sequence",
                ));
            }
            let unit = (((b & 0x0f) as u16) << 12)
                | (((bytes[i + 1] & 0x3f) as u16) << 6)
                | ((bytes[i + 2] & 0x3f) as u16);
            units.push(unit);
            i += 3;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid Java modified UTF leading byte",
            ));
        }
    }
    String::from_utf16(&units)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid UTF-16 in Java UTF"))
}

pub fn write_object<W: Write>(write: &mut W, value: &TypeValue) -> io::Result<()> {
    match value {
        TypeValue::Null => write.write_all(&[0]),
        TypeValue::Int(value) => {
            write.write_all(&[1])?;
            write_i32(write, *value)
        }
        TypeValue::Long(value) => {
            write.write_all(&[2])?;
            write_i64(write, *value)
        }
        TypeValue::Float(value) => {
            write.write_all(&[3])?;
            write_u32(write, value.to_bits())
        }
        TypeValue::String(value) => {
            write.write_all(&[4])?;
            write_string(write, Some(value))
        }
        TypeValue::Content(value) => {
            write.write_all(&[5])?;
            write_content_ref(write, *value)
        }
        TypeValue::Bool(value) => {
            write.write_all(&[10])?;
            write.write_all(&[*value as u8])
        }
        TypeValue::Double(value) => {
            write.write_all(&[11])?;
            write_u64(write, value.to_bits())
        }
        TypeValue::Building(pos) => {
            write.write_all(&[12])?;
            write_i32(write, *pos)
        }
        TypeValue::LogicAccess(access) => {
            write.write_all(&[13])?;
            write_i16(write, access.ordinal() as i16)
        }
        TypeValue::Point2(value) => {
            write.write_all(&[7])?;
            write_i32(write, value.x)?;
            write_i32(write, value.y)
        }
        TypeValue::TechNode(value) => {
            write.write_all(&[9])?;
            write_content_ref(write, *value)
        }
        TypeValue::IntSeq(values) => {
            if values.len() > MAX_ARRAY_SIZE || values.len() > i16::MAX as usize {
                return Err(invalid_input("int seq too large"));
            }
            write.write_all(&[6])?;
            write_i16(write, values.len() as i16)?;
            for value in values {
                write_i32(write, *value)?;
            }
            Ok(())
        }
        TypeValue::Point2Array(values) => {
            if values.len() > u8::MAX as usize {
                return Err(invalid_input("point2 array too large"));
            }
            write.write_all(&[8])?;
            write.write_all(&[values.len() as u8])?;
            for value in values {
                write_point2_packed(write, *value)?;
            }
            Ok(())
        }
        TypeValue::Vec2(value) => {
            write.write_all(&[19])?;
            write_u32(write, value.x.to_bits())?;
            write_u32(write, value.y.to_bits())
        }
        TypeValue::ByteArray(values) => {
            if values.len() > MAX_BYTE_ARRAY_SIZE {
                return Err(invalid_input("byte array too large"));
            }
            write.write_all(&[14])?;
            write_i32(write, values.len() as i32)?;
            write.write_all(values)
        }
        TypeValue::BoolArray(values) => {
            if values.len() > MAX_ARRAY_SIZE {
                return Err(invalid_input("bool array too large"));
            }
            write.write_all(&[16])?;
            write_i32(write, values.len() as i32)?;
            for value in values {
                write.write_all(&[*value as u8])?;
            }
            Ok(())
        }
        TypeValue::Vec2Array(values) => {
            if values.len() > MAX_ARRAY_SIZE || values.len() > i16::MAX as usize {
                return Err(invalid_input("vec2 array too large"));
            }
            write.write_all(&[18])?;
            write_i16(write, values.len() as i16)?;
            for value in values {
                write_vec2(write, *value)?;
            }
            Ok(())
        }
        TypeValue::Team(value) => {
            write.write_all(&[20])?;
            write.write_all(&[*value])
        }
        TypeValue::UnitCommand(value) => {
            write.write_all(&[23])?;
            write_i16(write, *value)
        }
        TypeValue::Unit(id) => {
            write.write_all(&[17])?;
            write_i32(write, *id)
        }
        TypeValue::IntArray(values) => {
            if values.len() > MAX_ARRAY_SIZE || values.len() > i16::MAX as usize {
                return Err(invalid_input("int array too large"));
            }
            write.write_all(&[21])?;
            write_i16(write, values.len() as i16)?;
            for value in values {
                write_i32(write, *value)?;
            }
            Ok(())
        }
        TypeValue::ObjectArray(values) => {
            if values.len() > MAX_ARRAY_SIZE {
                return Err(invalid_input("object array too large"));
            }
            write.write_all(&[22])?;
            write_i32(write, values.len() as i32)?;
            for value in values {
                write_object(write, value)?;
            }
            Ok(())
        }
    }
}

pub fn read_object<R: Read>(read: &mut R) -> io::Result<TypeValue> {
    read_object_inner(read, true)
}

/// Java `TypeIO.readObjectBoxed(read, true)` wire reader.
///
/// Rust currently preserves boxed building/unit references as their stable wire
/// ids (`TypeValue::Building` / `TypeValue::Unit`) instead of eagerly unboxing
/// them into live world references. This keeps processor save-load payloads
/// relocatable until the caller can resolve them against the loaded world.
pub fn read_object_boxed<R: Read>(read: &mut R) -> io::Result<TypeValue> {
    read_object_inner_limited(read, true, MAX_OBJECT_READ_ARRAY_SIZE, None)
}

pub fn read_object_safe<R: Read>(read: &mut R) -> io::Result<TypeValue> {
    read_object_inner_limited(read, true, MAX_ARRAY_SIZE, Some(MAX_SAFE_STRING_CHARS))
}

fn read_object_inner<R: Read>(read: &mut R, allow_arrays: bool) -> io::Result<TypeValue> {
    read_object_inner_limited(read, allow_arrays, MAX_OBJECT_READ_ARRAY_SIZE, None)
}

fn read_object_inner_limited<R: Read>(
    read: &mut R,
    allow_arrays: bool,
    max_array_size: usize,
    max_string_chars: Option<usize>,
) -> io::Result<TypeValue> {
    let tag = read_u8(read)?;
    match tag {
        0 => Ok(TypeValue::Null),
        1 => Ok(TypeValue::Int(read_i32(read)?)),
        2 => Ok(TypeValue::Long(read_i64(read)?)),
        3 => Ok(TypeValue::Float(f32::from_bits(read_u32(read)?))),
        4 => {
            let value = match read_string(read)? {
                Some(value) => {
                    if max_string_chars.is_some_and(|max| value.chars().count() > max) {
                        return Err(invalid_data("safe string too long"));
                    }
                    TypeValue::String(value)
                }
                None => TypeValue::Null,
            };
            Ok(value)
        }
        5 => Ok(TypeValue::Content(read_content_ref(read)?)),
        10 => Ok(TypeValue::Bool(read_u8(read)? != 0)),
        11 => Ok(TypeValue::Double(f64::from_bits(read_u64(read)?))),
        12 => Ok(TypeValue::Building(read_i32(read)?)),
        13 => {
            let ordinal = read_i16(read)?;
            let ordinal =
                u8::try_from(ordinal).map_err(|_| invalid_data("invalid LAccess ordinal"))?;
            LAccess::from_ordinal(ordinal)
                .map(TypeValue::LogicAccess)
                .ok_or_else(|| invalid_data("invalid LAccess ordinal"))
        }
        7 => Ok(TypeValue::Point2(Point2::new(
            read_i32(read)?,
            read_i32(read)?,
        ))),
        9 => Ok(TypeValue::TechNode(read_content_ref(read)?)),
        6 => {
            ensure_arrays_allowed(allow_arrays)?;
            let len = read_i16(read)?;
            if len < 0 || len as usize > max_array_size {
                return Err(invalid_data("invalid int seq length"));
            }
            let mut values = Vec::with_capacity(len as usize);
            for _ in 0..len {
                values.push(read_i32(read)?);
            }
            Ok(TypeValue::IntSeq(values))
        }
        8 => {
            ensure_arrays_allowed(allow_arrays)?;
            let len = read_u8(read)? as usize;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push(read_point2_packed(read)?);
            }
            Ok(TypeValue::Point2Array(values))
        }
        14 => {
            ensure_arrays_allowed(allow_arrays)?;
            let len = read_i32(read)?;
            if len < 0 || len as usize > MAX_BYTE_ARRAY_SIZE {
                return Err(invalid_data("invalid byte array length"));
            }
            let mut bytes = vec![0; len as usize];
            read.read_exact(&mut bytes)?;
            Ok(TypeValue::ByteArray(bytes))
        }
        16 => {
            ensure_arrays_allowed(allow_arrays)?;
            let len = read_i32(read)?;
            if len < 0 || len as usize > max_array_size {
                return Err(invalid_data("invalid bool array length"));
            }
            let mut values = Vec::with_capacity(len as usize);
            for _ in 0..len {
                values.push(read_u8(read)? != 0);
            }
            Ok(TypeValue::BoolArray(values))
        }
        18 => {
            ensure_arrays_allowed(allow_arrays)?;
            let len = read_i16(read)?;
            if len < 0 || len as usize > max_array_size {
                return Err(invalid_data("invalid vec2 array length"));
            }
            let mut values = Vec::with_capacity(len as usize);
            for _ in 0..len {
                values.push(read_vec2(read)?);
            }
            Ok(TypeValue::Vec2Array(values))
        }
        20 => Ok(TypeValue::Team(read_u8(read)?)),
        19 => Ok(TypeValue::Vec2(Vec2::new(
            f32::from_bits(read_u32(read)?),
            f32::from_bits(read_u32(read)?),
        ))),
        21 => {
            ensure_arrays_allowed(allow_arrays)?;
            let len = read_i16(read)?;
            if len < 0 || len as usize > max_array_size {
                return Err(invalid_data("invalid int array length"));
            }
            let mut values = Vec::with_capacity(len as usize);
            for _ in 0..len {
                values.push(read_i32(read)?);
            }
            Ok(TypeValue::IntArray(values))
        }
        22 => {
            ensure_arrays_allowed(allow_arrays)?;
            let len = read_i32(read)?;
            if len < 0 || len as usize > max_array_size {
                return Err(invalid_data("invalid object array length"));
            }
            let mut values = Vec::with_capacity(len as usize);
            for _ in 0..len {
                values.push(read_object_inner_limited(
                    read,
                    false,
                    max_array_size,
                    max_string_chars,
                )?);
            }
            Ok(TypeValue::ObjectArray(values))
        }
        15 => {
            let _legacy_command_type = read_u8(read)?;
            Ok(TypeValue::Null)
        }
        17 => Ok(TypeValue::Unit(read_i32(read)?)),
        23 => Ok(TypeValue::UnitCommand(read_i16(read)?)),
        _ => Err(invalid_data("unsupported TypeIO object tag")),
    }
}

fn ensure_arrays_allowed(allow_arrays: bool) -> io::Result<()> {
    if allow_arrays {
        Ok(())
    } else {
        Err(invalid_data("nested arrays are not allowed"))
    }
}

pub fn read_u8<R: Read>(read: &mut R) -> io::Result<u8> {
    let mut b = [0; 1];
    read.read_exact(&mut b)?;
    Ok(b[0])
}

pub fn write_u8<W: Write>(write: &mut W, value: u8) -> io::Result<()> {
    write.write_all(&[value])
}

pub fn read_i16<R: Read>(read: &mut R) -> io::Result<i16> {
    let mut b = [0; 2];
    read.read_exact(&mut b)?;
    Ok(i16::from_be_bytes(b))
}

pub fn read_u16<R: Read>(read: &mut R) -> io::Result<u16> {
    let mut b = [0; 2];
    read.read_exact(&mut b)?;
    Ok(u16::from_be_bytes(b))
}

pub fn read_i32<R: Read>(read: &mut R) -> io::Result<i32> {
    let mut b = [0; 4];
    read.read_exact(&mut b)?;
    Ok(i32::from_be_bytes(b))
}

pub fn read_u32<R: Read>(read: &mut R) -> io::Result<u32> {
    let mut b = [0; 4];
    read.read_exact(&mut b)?;
    Ok(u32::from_be_bytes(b))
}

pub fn read_i64<R: Read>(read: &mut R) -> io::Result<i64> {
    let mut b = [0; 8];
    read.read_exact(&mut b)?;
    Ok(i64::from_be_bytes(b))
}

pub fn read_u64<R: Read>(read: &mut R) -> io::Result<u64> {
    let mut b = [0; 8];
    read.read_exact(&mut b)?;
    Ok(u64::from_be_bytes(b))
}

pub fn write_i16<W: Write>(write: &mut W, value: i16) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

pub fn write_u16<W: Write>(write: &mut W, value: u16) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

pub fn write_i32<W: Write>(write: &mut W, value: i32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

pub fn write_u32<W: Write>(write: &mut W, value: u32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

pub fn write_i64<W: Write>(write: &mut W, value: i64) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

pub fn write_u64<W: Write>(write: &mut W, value: u64) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

pub fn read_bool<R: Read>(read: &mut R) -> io::Result<bool> {
    Ok(read_u8(read)? != 0)
}

pub fn write_bool<W: Write>(write: &mut W, value: bool) -> io::Result<()> {
    write_u8(write, value as u8)
}

pub fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    Ok(f32::from_bits(read_u32(read)?))
}

pub fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write_u32(write, value.to_bits())
}

pub fn read_remaining_bytes<R: Read>(read: &mut R) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    read.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn write_point2<W: Write>(write: &mut W, value: Point2) -> io::Result<()> {
    write_i32(write, value.x)?;
    write_i32(write, value.y)
}

pub fn read_point2<R: Read>(read: &mut R) -> io::Result<Point2> {
    Ok(Point2::new(read_i32(read)?, read_i32(read)?))
}

pub fn write_point2_packed<W: Write>(write: &mut W, value: Point2) -> io::Result<()> {
    write_i32(write, value.pack())
}

pub fn read_point2_packed<R: Read>(read: &mut R) -> io::Result<Point2> {
    Ok(Point2::from_packed(read_i32(read)?))
}

pub fn write_vec2<W: Write>(write: &mut W, value: Vec2) -> io::Result<()> {
    write_u32(write, value.x.to_bits())?;
    write_u32(write, value.y.to_bits())
}

pub fn read_vec2<R: Read>(read: &mut R) -> io::Result<Vec2> {
    Ok(Vec2::new(
        f32::from_bits(read_u32(read)?),
        f32::from_bits(read_u32(read)?),
    ))
}

pub fn read_vec2_into<R: Read>(read: &mut R, base: &mut Vec2) -> io::Result<()> {
    base.x = f32::from_bits(read_u32(read)?);
    base.y = f32::from_bits(read_u32(read)?);
    Ok(())
}

pub fn write_vec_nullable<W: Write>(write: &mut W, value: Option<Vec2>) -> io::Result<()> {
    match value {
        Some(value) => write_vec2(write, value),
        None => {
            write_u32(write, f32::NAN.to_bits())?;
            write_u32(write, f32::NAN.to_bits())
        }
    }
}

pub fn read_vec_nullable<R: Read>(read: &mut R) -> io::Result<Option<Vec2>> {
    let x = f32::from_bits(read_u32(read)?);
    let y = f32::from_bits(read_u32(read)?);
    if x.is_nan() || y.is_nan() {
        Ok(None)
    } else {
        Ok(Some(Vec2::new(x, y)))
    }
}

pub fn write_color<W: Write>(write: &mut W, color: RgbaColor) -> io::Result<()> {
    write_i32(write, color.rgba())
}

pub fn read_color<R: Read>(read: &mut R) -> io::Result<RgbaColor> {
    Ok(RgbaColor::new(read_i32(read)?))
}

pub fn write_team_id<W: Write>(write: &mut W, value: TeamId) -> io::Result<()> {
    write.write_all(&[value.0])
}

pub fn read_team_id<R: Read>(read: &mut R) -> io::Result<TeamId> {
    Ok(TeamId(read_u8(read)?))
}

pub fn write_team<W: Write>(write: &mut W, team: Option<TeamId>) -> io::Result<()> {
    write_u8(write, team.map_or(0, |team| team.0))
}

pub fn read_team<R: Read>(read: &mut R) -> io::Result<TeamId> {
    read_team_id(read)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitRef {
    Null,
    Block { tile_pos: i32 },
    Unit { id: i32 },
}

/// Java `TypeIO.writeUnitContainer(...)` / `readUnitContainer(...)` 的原始 wire 容器。
///
/// 上游写入顺序为 `unit.id`、`unit.classId()`，之后直接接 `unit.writeSync(...)`
/// 产生的同步字节；当前 Rust 端在完整 Unit 实体系统补齐前按 raw bytes 保存。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnitSyncContainer {
    pub unit_id: i32,
    pub unit_type_id: u8,
    pub sync: Vec<u8>,
}

impl UnitSyncContainer {
    pub fn new(unit_id: i32, unit_type_id: u8, sync: Vec<u8>) -> Self {
        Self {
            unit_id,
            unit_type_id,
            sync,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MountWire {
    pub shoot: bool,
    pub rotate: bool,
    pub aim_x: f32,
    pub aim_y: f32,
}

impl From<&WeaponMount> for MountWire {
    fn from(value: &WeaponMount) -> Self {
        Self {
            shoot: value.shoot,
            rotate: value.rotate,
            aim_x: value.aim_x,
            aim_y: value.aim_y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AbilityWire {
    pub data: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FireSyncWire {
    pub lifetime: f32,
    pub tile_pos: Option<i32>,
    pub time: f32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecalSyncWire {
    pub color: RgbaColor,
    pub lifetime: f32,
    pub rotation: f32,
    pub time: f32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BulletSyncWire {
    pub collided: Vec<i32>,
    pub damage: f32,
    pub data: TypeValue,
    pub fdata: f32,
    pub lifetime: f32,
    pub owner: EntityRef,
    pub rotation: f32,
    pub team: TeamId,
    pub time: f32,
    pub bullet_type_id: ContentId,
    pub vel: Vec2,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PuddleSyncWire {
    pub amount: f32,
    pub liquid_id: Option<ContentId>,
    pub tile_pos: Option<i32>,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeatherStateSyncWire {
    pub effect_timer: f32,
    pub intensity: f32,
    pub life: f32,
    pub opacity: f32,
    pub weather_id: Option<ContentId>,
    pub wind_vector: Vec2,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectStateSyncWire {
    pub color: RgbaColor,
    pub data: TypeValue,
    pub effect_id: u16,
    pub lifetime: f32,
    pub offset_pos: f32,
    pub offset_rot: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub parent_id: Option<i32>,
    pub rot_with_parent: bool,
    pub rotation: f32,
    pub time: f32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitSyncWire {
    pub abilities: Vec<AbilityWire>,
    pub ammo: f32,
    pub controller: ControllerWire,
    pub elevation: f32,
    pub flag: f64,
    pub health: f32,
    pub is_shooting: bool,
    pub mine_tile: Option<i32>,
    pub mounts: Vec<MountWire>,
    pub plans: Option<Vec<BuildPlanWire>>,
    pub rotation: f32,
    pub shield: f32,
    pub spawned_by_core: bool,
    pub stack: ItemStack,
    pub statuses: Vec<StatusEntry>,
    pub team: TeamId,
    pub type_id: ContentId,
    pub update_building: bool,
    pub vel: Vec2,
    pub x: f32,
    pub y: f32,
}

pub fn write_unit_ref<W: Write>(write: &mut W, unit: UnitRef) -> io::Result<()> {
    match unit {
        UnitRef::Null => {
            write_u8(write, 0)?;
            write_i32(write, 0)
        }
        UnitRef::Block { tile_pos } => {
            write_u8(write, 1)?;
            write_i32(write, tile_pos)
        }
        UnitRef::Unit { id } => {
            write_u8(write, 2)?;
            write_i32(write, id)
        }
    }
}

pub fn read_unit_ref<R: Read>(read: &mut R) -> io::Result<UnitRef> {
    let kind = read_u8(read)?;
    let id = read_i32(read)?;
    match kind {
        0 => Ok(UnitRef::Null),
        1 => Ok(UnitRef::Block { tile_pos: id }),
        2 => Ok(UnitRef::Unit { id }),
        _ => Err(invalid_data("unknown unit ref kind")),
    }
}

/// 按上游 `TypeIO.writeUnitContainer(...)` 字段顺序写入 raw 单位同步容器。
pub fn write_unit_container<W: Write>(
    write: &mut W,
    container: &UnitSyncContainer,
) -> io::Result<()> {
    write_i32(write, container.unit_id)?;
    write_u8(write, container.unit_type_id)?;
    write.write_all(&container.sync)
}

/// 按上游 `TypeIO.readUnitContainer(...)` 语义读取 raw 单位同步容器。
///
/// 该 helper 会把当前 packet payload 中剩余全部字节视为 `sync` 数据。
pub fn read_unit_container<R: Read>(read: &mut R) -> io::Result<UnitSyncContainer> {
    let unit_id = read_i32(read)?;
    let unit_type_id = read_u8(read)?;
    let mut sync = Vec::new();
    read.read_to_end(&mut sync)?;
    Ok(UnitSyncContainer::new(unit_id, unit_type_id, sync))
}

pub fn write_payload<W: Write>(write: &mut W, payload: Option<&PayloadRef>) -> io::Result<()> {
    payloads::write_payload_ref(write, payload)
}

pub fn read_payload<R: Read>(read: &mut R) -> io::Result<Option<PayloadRef>> {
    if !read_bool(read)? {
        return Ok(None);
    }
    let payload_type = read_u8(read)?;
    match payload_type {
        PAYLOAD_BLOCK_TYPE => Ok(Some(PayloadRef::Block {
            block: read_i16(read)?,
            version: read_u8(read)?,
            build_bytes: read_remaining_bytes(read)?,
        })),
        PAYLOAD_UNIT_TYPE => Ok(Some(PayloadRef::Unit {
            class_id: read_u8(read)?,
            unit_bytes: read_remaining_bytes(read)?,
        })),
        _ => Err(invalid_data("unknown payload type")),
    }
}

pub fn write_mounts<W: Write>(write: &mut W, mounts: &[MountWire]) -> io::Result<()> {
    if mounts.len() > u8::MAX as usize {
        return Err(invalid_input("mount array too large"));
    }
    write_u8(write, mounts.len() as u8)?;
    for mount in mounts {
        let state = (mount.shoot as u8) | ((mount.rotate as u8) << 1);
        write_u8(write, state)?;
        write_f32(write, mount.aim_x)?;
        write_f32(write, mount.aim_y)?;
    }
    Ok(())
}

pub fn write_weapon_mounts<W: Write>(write: &mut W, mounts: &[WeaponMount]) -> io::Result<()> {
    if mounts.len() > u8::MAX as usize {
        return Err(invalid_input("mount array too large"));
    }
    write_u8(write, mounts.len() as u8)?;
    for mount in mounts {
        let state = (mount.shoot as u8) | ((mount.rotate as u8) << 1);
        write_u8(write, state)?;
        write_f32(write, mount.aim_x)?;
        write_f32(write, mount.aim_y)?;
    }
    Ok(())
}

pub fn read_mounts<R: Read>(read: &mut R) -> io::Result<Vec<MountWire>> {
    let len = read_u8(read)? as usize;
    let mut mounts = Vec::with_capacity(len);
    for _ in 0..len {
        let state = read_u8(read)?;
        mounts.push(MountWire {
            shoot: state & 1 != 0,
            rotate: state & 2 != 0,
            aim_x: read_f32(read)?,
            aim_y: read_f32(read)?,
        });
    }
    Ok(mounts)
}

pub fn read_mounts_into<R: Read>(read: &mut R, mounts: &mut [WeaponMount]) -> io::Result<()> {
    let len = read_u8(read)? as usize;
    for index in 0..len {
        let state = read_u8(read)?;
        let aim_x = read_f32(read)?;
        let aim_y = read_f32(read)?;
        if let Some(mount) = mounts.get_mut(index) {
            mount.shoot = state & 1 != 0;
            mount.rotate = state & 2 != 0;
            mount.aim_x = aim_x;
            mount.aim_y = aim_y;
        }
    }
    Ok(())
}

pub fn skip_mounts<R: Read>(read: &mut R) -> io::Result<()> {
    let len = read_u8(read)? as usize;
    for _ in 0..len {
        let mut buf = [0; 9];
        read.read_exact(&mut buf)?;
    }
    Ok(())
}

pub fn write_abilities<W: Write>(write: &mut W, abilities: &[AbilityWire]) -> io::Result<()> {
    if abilities.len() > u8::MAX as usize {
        return Err(invalid_input("ability array too large"));
    }
    write_u8(write, abilities.len() as u8)?;
    for ability in abilities {
        write_f32(write, ability.data)?;
    }
    Ok(())
}

pub fn write_ability_data<W: Write>(write: &mut W, ability_data: &[f32]) -> io::Result<()> {
    if ability_data.len() > u8::MAX as usize {
        return Err(invalid_input("ability array too large"));
    }
    write_u8(write, ability_data.len() as u8)?;
    for data in ability_data {
        write_f32(write, *data)?;
    }
    Ok(())
}

pub fn read_abilities<R: Read>(read: &mut R) -> io::Result<Vec<AbilityWire>> {
    let len = read_u8(read)? as usize;
    let mut abilities = Vec::with_capacity(len);
    for _ in 0..len {
        abilities.push(AbilityWire {
            data: read_f32(read)?,
        });
    }
    Ok(abilities)
}

pub fn read_ability_data<R: Read>(read: &mut R) -> io::Result<Vec<f32>> {
    let len = read_u8(read)? as usize;
    let mut abilities = Vec::with_capacity(len);
    for _ in 0..len {
        abilities.push(read_f32(read)?);
    }
    Ok(abilities)
}

pub fn skip_abilities<R: Read>(read: &mut R) -> io::Result<()> {
    let len = read_u8(read)? as usize;
    for _ in 0..len {
        let mut buf = [0; 4];
        read.read_exact(&mut buf)?;
    }
    Ok(())
}

pub fn write_statuses<W: Write>(write: &mut W, statuses: &[StatusEntry]) -> io::Result<()> {
    write_i32(write, statuses.len() as i32)?;
    for status in statuses {
        write_status(write, status)?;
    }
    Ok(())
}

pub fn read_statuses<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
) -> io::Result<Vec<StatusEntry>> {
    let len = read_i32(read)?;
    if len < 0 || len as usize > MAX_ARRAY_SIZE {
        return Err(invalid_data("invalid status list length"));
    }
    let mut statuses = Vec::with_capacity(len as usize);
    for _ in 0..len {
        statuses.push(read_status(read, loader)?);
    }
    Ok(statuses)
}

pub fn read_statuses_into<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
    statuses: &mut Vec<StatusEntry>,
) -> io::Result<()> {
    let len = read_i32(read)?;
    if len < 0 || len as usize > MAX_ARRAY_SIZE {
        return Err(invalid_data("invalid status list length"));
    }
    statuses.clear();
    statuses.reserve(len as usize);
    for _ in 0..len {
        statuses.push(read_status(read, loader)?);
    }
    Ok(())
}

pub fn write_fire_sync<W: Write>(write: &mut W, sync: &FireSyncWire) -> io::Result<()> {
    write_f32(write, sync.lifetime)?;
    write_tile_pos(write, sync.tile_pos)?;
    write_f32(write, sync.time)?;
    write_f32(write, sync.x)?;
    write_f32(write, sync.y)
}

pub fn read_fire_sync<R: Read>(read: &mut R) -> io::Result<FireSyncWire> {
    let lifetime = read_f32(read)?;
    let tile_pos = read_tile_pos(read)?;
    let time = read_f32(read)?;
    let x = read_f32(read)?;
    let y = read_f32(read)?;
    Ok(FireSyncWire {
        lifetime,
        tile_pos,
        time,
        x,
        y,
    })
}

pub fn write_decal_sync<W: Write>(write: &mut W, sync: &DecalSyncWire) -> io::Result<()> {
    write_color(write, sync.color)?;
    write_f32(write, sync.lifetime)?;
    // Upstream revision 0 lists `region: TextureRegion` between lifetime and
    // rotation, but the annotation serializer has no TextureRegion TypeIO
    // method. Generated Java sync code therefore logs and skips that field.
    write_f32(write, sync.rotation)?;
    write_f32(write, sync.time)?;
    write_f32(write, sync.x)?;
    write_f32(write, sync.y)
}

pub fn read_decal_sync<R: Read>(read: &mut R) -> io::Result<DecalSyncWire> {
    let color = read_color(read)?;
    let lifetime = read_f32(read)?;
    let rotation = read_f32(read)?;
    let time = read_f32(read)?;
    let x = read_f32(read)?;
    let y = read_f32(read)?;
    Ok(DecalSyncWire {
        color,
        lifetime,
        rotation,
        time,
        x,
        y,
    })
}

pub fn write_bullet_sync<W: Write>(write: &mut W, sync: &BulletSyncWire) -> io::Result<()> {
    write_int_seq(write, &sync.collided)?;
    write_f32(write, sync.damage)?;
    write_object(write, &sync.data)?;
    write_f32(write, sync.fdata)?;
    write_f32(write, sync.lifetime)?;
    write_entity_ref(write, sync.owner)?;
    write_f32(write, sync.rotation)?;
    write_team(write, Some(sync.team))?;
    write_f32(write, sync.time)?;
    write_bullet_type_id(write, sync.bullet_type_id)?;
    write_vec2(write, sync.vel)?;
    write_f32(write, sync.x)?;
    write_f32(write, sync.y)
}

pub fn read_bullet_sync<R: Read>(read: &mut R) -> io::Result<BulletSyncWire> {
    let collided = read_int_seq(read)?;
    let damage = read_f32(read)?;
    let data = read_object(read)?;
    let fdata = read_f32(read)?;
    let lifetime = read_f32(read)?;
    let owner = read_entity_ref(read)?;
    let rotation = read_f32(read)?;
    let team = read_team(read)?;
    let time = read_f32(read)?;
    let bullet_type_id = read_bullet_type_id(read)?;
    let vel = read_vec2(read)?;
    let x = read_f32(read)?;
    let y = read_f32(read)?;
    Ok(BulletSyncWire {
        collided,
        damage,
        data,
        fdata,
        lifetime,
        owner,
        rotation,
        team,
        time,
        bullet_type_id,
        vel,
        x,
        y,
    })
}

pub fn write_puddle_sync<W: Write>(write: &mut W, sync: &PuddleSyncWire) -> io::Result<()> {
    write_f32(write, sync.amount)?;
    write_content_id(write, ContentType::Liquid, sync.liquid_id)?;
    write_tile_pos(write, sync.tile_pos)?;
    write_f32(write, sync.x)?;
    write_f32(write, sync.y)
}

pub fn read_puddle_sync<R: Read>(read: &mut R) -> io::Result<PuddleSyncWire> {
    let amount = read_f32(read)?;
    let liquid_id = read_content_id(read)?;
    let tile_pos = read_tile_pos(read)?;
    let x = read_f32(read)?;
    let y = read_f32(read)?;
    Ok(PuddleSyncWire {
        amount,
        liquid_id,
        tile_pos,
        x,
        y,
    })
}

pub fn write_weather_state_sync<W: Write>(
    write: &mut W,
    sync: &WeatherStateSyncWire,
) -> io::Result<()> {
    write_f32(write, sync.effect_timer)?;
    write_f32(write, sync.intensity)?;
    write_f32(write, sync.life)?;
    write_f32(write, sync.opacity)?;
    write_content_id(write, ContentType::Weather, sync.weather_id)?;
    write_vec2(write, sync.wind_vector)?;
    write_f32(write, sync.x)?;
    write_f32(write, sync.y)
}

pub fn read_weather_state_sync<R: Read>(read: &mut R) -> io::Result<WeatherStateSyncWire> {
    let effect_timer = read_f32(read)?;
    let intensity = read_f32(read)?;
    let life = read_f32(read)?;
    let opacity = read_f32(read)?;
    let weather_id = read_content_id(read)?;
    let wind_vector = read_vec2(read)?;
    let x = read_f32(read)?;
    let y = read_f32(read)?;
    Ok(WeatherStateSyncWire {
        effect_timer,
        intensity,
        life,
        opacity,
        weather_id,
        wind_vector,
        x,
        y,
    })
}

pub fn write_effect_state_sync<W: Write>(
    write: &mut W,
    sync: &EffectStateSyncWire,
) -> io::Result<()> {
    write_color(write, sync.color)?;
    write_object(write, &sync.data)?;
    write_u16(write, sync.effect_id)?;
    write_f32(write, sync.lifetime)?;
    write_f32(write, sync.offset_pos)?;
    write_f32(write, sync.offset_rot)?;
    write_f32(write, sync.offset_x)?;
    write_f32(write, sync.offset_y)?;
    write_entity_ref(write, EntityRef { id: sync.parent_id })?;
    write_bool(write, sync.rot_with_parent)?;
    write_f32(write, sync.rotation)?;
    write_f32(write, sync.time)?;
    write_f32(write, sync.x)?;
    write_f32(write, sync.y)
}

pub fn read_effect_state_sync<R: Read>(read: &mut R) -> io::Result<EffectStateSyncWire> {
    let color = read_color(read)?;
    let data = read_object(read)?;
    let effect_id = read_effect_id(read)?;
    let lifetime = read_f32(read)?;
    let offset_pos = read_f32(read)?;
    let offset_rot = read_f32(read)?;
    let offset_x = read_f32(read)?;
    let offset_y = read_f32(read)?;
    let parent_id = read_entity_ref(read)?.id;
    let rot_with_parent = read_bool(read)?;
    let rotation = read_f32(read)?;
    let time = read_f32(read)?;
    let x = read_f32(read)?;
    let y = read_f32(read)?;
    Ok(EffectStateSyncWire {
        color,
        data,
        effect_id,
        lifetime,
        offset_pos,
        offset_rot,
        offset_x,
        offset_y,
        parent_id,
        rot_with_parent,
        rotation,
        time,
        x,
        y,
    })
}

pub fn write_unit_sync<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    sync: &UnitSyncWire,
) -> io::Result<()> {
    write_abilities(write, &sync.abilities)?;
    write_f32(write, sync.ammo)?;
    write_controller(write, &sync.controller)?;
    write_f32(write, sync.elevation)?;
    write_u64(write, sync.flag.to_bits())?;
    write_f32(write, sync.health)?;
    write_bool(write, sync.is_shooting)?;
    write_tile_pos(write, sync.mine_tile)?;
    write_mounts(write, &sync.mounts)?;
    write_plans_queue_net(write, loader, sync.plans.as_deref())?;
    write_f32(write, sync.rotation)?;
    write_f32(write, sync.shield)?;
    write_bool(write, sync.spawned_by_core)?;
    write_items(write, loader, &sync.stack)?;
    write_statuses(write, &sync.statuses)?;
    write_team(write, Some(sync.team))?;
    write_i16(write, sync.type_id)?;
    write_bool(write, sync.update_building)?;
    write_vec2(write, sync.vel)?;
    write_f32(write, sync.x)?;
    write_f32(write, sync.y)
}

pub fn read_unit_sync<R: Read>(read: &mut R, loader: &ContentLoader) -> io::Result<UnitSyncWire> {
    let abilities = read_abilities(read)?;
    let ammo = read_f32(read)?;
    let controller = read_controller(read)?;
    let elevation = read_f32(read)?;
    let flag = f64::from_bits(read_u64(read)?);
    let health = read_f32(read)?;
    let is_shooting = read_bool(read)?;
    let mine_tile = read_tile_pos(read)?;
    let mounts = read_mounts(read)?;
    let plans = read_plans_queue(read, loader)?;
    let rotation = read_f32(read)?;
    let shield = read_f32(read)?;
    let spawned_by_core = read_bool(read)?;
    let stack = read_items(read, loader)?;
    let statuses = read_statuses(read, loader)?;
    let team = read_team(read)?;
    let type_id = read_i16(read)?;
    let update_building = read_bool(read)?;
    let vel = read_vec2(read)?;
    let x = read_f32(read)?;
    let y = read_f32(read)?;
    Ok(UnitSyncWire {
        abilities,
        ammo,
        controller,
        elevation,
        flag,
        health,
        is_shooting,
        mine_tile,
        mounts,
        plans,
        rotation,
        shield,
        spawned_by_core,
        stack,
        statuses,
        team,
        type_id,
        update_building,
        vel,
        x,
        y,
    })
}

pub fn write_entity_ref<W: Write>(write: &mut W, entity: EntityRef) -> io::Result<()> {
    write_i32(write, entity.id.unwrap_or(-1))
}

pub fn read_entity_ref<R: Read>(read: &mut R) -> io::Result<EntityRef> {
    let id = read_i32(read)?;
    Ok(EntityRef {
        id: (id >= 0).then_some(id),
    })
}

pub fn write_building_ref<W: Write>(write: &mut W, building: BuildingRef) -> io::Result<()> {
    write_i32(write, building.tile_pos.unwrap_or(-1))
}

pub fn read_building_ref<R: Read>(read: &mut R) -> io::Result<BuildingRef> {
    let tile_pos = read_i32(read)?;
    Ok(BuildingRef {
        tile_pos: (tile_pos >= 0).then_some(tile_pos),
    })
}

pub fn write_tile_pos<W: Write>(write: &mut W, tile_pos: Option<i32>) -> io::Result<()> {
    write_i32(write, tile_pos.unwrap_or_else(|| point2_pack(-1, -1)))
}

pub fn read_tile_pos<R: Read>(read: &mut R) -> io::Result<Option<i32>> {
    let pos = read_i32(read)?;
    if pos == point2_pack(-1, -1) {
        Ok(None)
    } else {
        Ok(Some(pos))
    }
}

pub fn write_content_ref<W: Write>(write: &mut W, value: ContentRef) -> io::Result<()> {
    write_u8(write, value.content_type.ordinal())?;
    write_i16(write, value.id)
}

pub fn read_content_ref<R: Read>(read: &mut R) -> io::Result<ContentRef> {
    let ordinal = read_u8(read)?;
    let content_type = ContentType::from_ordinal(ordinal).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid ContentType ordinal {ordinal}"),
        )
    })?;
    Ok(ContentRef::new(content_type, read_i16(read)?))
}

pub fn read_content_ref_resolved<'a, R: Read>(
    read: &mut R,
    loader: &'a ContentLoader,
) -> io::Result<Option<&'a ContentRecord>> {
    Ok(read_content_ref(read)?.resolve(loader))
}

pub fn write_required_content_ref<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    content_type: ContentType,
    name: &str,
) -> io::Result<()> {
    let id = loader
        .get_by_name(content_type, name)
        .ok_or_else(|| invalid_input("unknown content name"))?
        .id;
    write_content_ref(write, ContentRef::new(content_type, id))
}

pub fn read_required_content_name<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
) -> io::Result<(ContentType, String)> {
    let content = read_content_ref(read)?;
    let name = loader
        .get_by_id(content.content_type, content.id)
        .and_then(ContentRecord::name)
        .ok_or_else(|| invalid_data("unknown content id"))?;
    Ok((content.content_type, name.to_string()))
}

pub fn write_nullable_content_id<W: Write>(write: &mut W, id: Option<ContentId>) -> io::Result<()> {
    write_i16(write, id.unwrap_or(-1))
}

pub fn read_nullable_content_id<R: Read>(read: &mut R) -> io::Result<Option<ContentId>> {
    let id = read_i16(read)?;
    if id == -1 {
        Ok(None)
    } else {
        Ok(Some(id))
    }
}

pub fn write_content_id<W: Write>(
    write: &mut W,
    content_type: ContentType,
    id: Option<ContentId>,
) -> io::Result<()> {
    if id.is_some_and(|id| id < 0) {
        return Err(invalid_input("negative content id"));
    }
    let _ = content_type;
    write_nullable_content_id(write, id)
}

pub fn read_content_id<R: Read>(read: &mut R) -> io::Result<Option<ContentId>> {
    read_nullable_content_id(read)
}

pub fn write_content_by_name<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    content_type: ContentType,
    name: Option<&str>,
) -> io::Result<()> {
    let id = match name {
        Some(name) => Some(
            loader
                .get_by_name(content_type, name)
                .ok_or_else(|| invalid_input("unknown content name"))?
                .id,
        ),
        None => None,
    };
    write_content_id(write, content_type, id)
}

pub fn read_content_name<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
    content_type: ContentType,
) -> io::Result<Option<String>> {
    let Some(id) = read_content_id(read)? else {
        return Ok(None);
    };
    loader
        .get_by_id(content_type, id)
        .and_then(|record| record.name().map(str::to_string))
        .map(Some)
        .ok_or_else(|| invalid_data("unknown content id"))
}

pub fn write_block<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    block: Option<&str>,
) -> io::Result<()> {
    write_content_by_name(write, loader, ContentType::Block, block)
}

pub fn read_block<R: Read>(read: &mut R, loader: &ContentLoader) -> io::Result<Option<String>> {
    read_content_name(read, loader, ContentType::Block)
}

fn read_required_block_name<R: Read>(read: &mut R, loader: &ContentLoader) -> io::Result<String> {
    read_block(read, loader)?.ok_or_else(|| invalid_data("null block id"))
}

pub fn write_item<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    item: Option<&str>,
) -> io::Result<()> {
    write_content_by_name(write, loader, ContentType::Item, item)
}

pub fn read_item<R: Read>(read: &mut R, loader: &ContentLoader) -> io::Result<Option<String>> {
    read_content_name(read, loader, ContentType::Item)
}

pub fn write_liquid<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    liquid: Option<&str>,
) -> io::Result<()> {
    write_content_by_name(write, loader, ContentType::Liquid, liquid)
}

pub fn read_liquid<R: Read>(read: &mut R, loader: &ContentLoader) -> io::Result<Option<String>> {
    read_content_name(read, loader, ContentType::Liquid)
}

pub fn write_weather<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    weather: Option<&str>,
) -> io::Result<()> {
    write_content_by_name(write, loader, ContentType::Weather, weather)
}

pub fn read_weather<R: Read>(read: &mut R, loader: &ContentLoader) -> io::Result<Option<String>> {
    read_content_name(read, loader, ContentType::Weather)
}

pub fn write_unit_type<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    unit: &str,
) -> io::Result<()> {
    write_content_by_name(write, loader, ContentType::Unit, Some(unit))
}

pub fn read_unit_type<R: Read>(read: &mut R, loader: &ContentLoader) -> io::Result<String> {
    read_content_name(read, loader, ContentType::Unit)?
        .ok_or_else(|| invalid_data("null unit type id"))
}

pub fn write_effect_id<W: Write>(write: &mut W, id: i16) -> io::Result<()> {
    if id < 0 {
        return Err(invalid_input("negative effect id"));
    }
    write_i16(write, id)
}

pub fn read_effect_id<R: Read>(read: &mut R) -> io::Result<u16> {
    read_u16(read)
}

pub fn write_sound_id<W: Write>(write: &mut W, id: i16) -> io::Result<()> {
    write_i16(write, id)
}

pub fn read_sound_id<R: Read>(read: &mut R) -> io::Result<i16> {
    read_i16(read)
}

pub fn write_bullet_type_id<W: Write>(write: &mut W, id: ContentId) -> io::Result<()> {
    if id < 0 {
        return Err(invalid_input("negative bullet id"));
    }
    write_i16(write, id)
}

pub fn read_bullet_type_id<R: Read>(read: &mut R) -> io::Result<ContentId> {
    let id = read_i16(read)?;
    if id < 0 {
        Err(invalid_data("negative bullet id"))
    } else {
        Ok(id)
    }
}

pub fn write_command_id<W: Write>(write: &mut W, id: Option<ContentId>) -> io::Result<()> {
    write_optional_byte_id(write, id)
}

pub fn read_command_id<R: Read>(read: &mut R) -> io::Result<Option<ContentId>> {
    read_optional_byte_id(read)
}

pub fn read_command<'a, R: Read>(
    read: &mut R,
    loader: &'a ContentLoader,
) -> io::Result<Option<&'a crate::mindustry::ai::unit_command::UnitCommand>> {
    Ok(match read_command_id(read)? {
        Some(id) => loader.unit_command(id),
        None => None,
    })
}

pub fn write_stance_id<W: Write>(write: &mut W, id: Option<ContentId>) -> io::Result<()> {
    write_optional_byte_id(write, id)
}

pub fn read_stance_id_or_stop<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
) -> io::Result<ContentId> {
    let raw = read_u8(read)?;
    if raw == u8::MAX || loader.unit_stance(raw as ContentId).is_none() {
        Ok(0)
    } else {
        Ok(raw as ContentId)
    }
}

pub fn read_stance<'a, R: Read>(
    read: &mut R,
    loader: &'a ContentLoader,
) -> io::Result<&'a crate::mindustry::ai::unit_stance::UnitStance> {
    let id = read_stance_id_or_stop(read, loader)?;
    loader
        .unit_stance(id)
        .ok_or_else(|| invalid_data("missing stop stance"))
}

pub fn write_controller<W: Write>(write: &mut W, controller: &ControllerWire) -> io::Result<()> {
    match controller {
        ControllerWire::Player { player_id } => {
            write_u8(write, 0)?;
            write_i32(write, *player_id)
        }
        ControllerWire::LegacyFormation { id } => {
            write_u8(write, 1)?;
            write_i32(write, *id)
        }
        ControllerWire::Ground => write_u8(write, 2),
        ControllerWire::Logic { controller_pos } => {
            write_u8(write, 3)?;
            write_i32(write, *controller_pos)
        }
        ControllerWire::Assembler => write_u8(write, 5),
        ControllerWire::Command(command) => write_command_controller(write, command),
    }
}

pub fn read_controller<R: Read>(read: &mut R) -> io::Result<ControllerWire> {
    let tag = read_u8(read)?;
    match tag {
        0 => Ok(ControllerWire::Player {
            player_id: read_i32(read)?,
        }),
        1 => Ok(ControllerWire::LegacyFormation {
            id: read_i32(read)?,
        }),
        3 => Ok(ControllerWire::Logic {
            controller_pos: read_i32(read)?,
        }),
        4 | 6 | 7 | 8 | 9 => Ok(ControllerWire::Command(read_command_controller_body(
            read, tag,
        )?)),
        5 => Ok(ControllerWire::Assembler),
        2 => Ok(ControllerWire::Ground),
        _ => Ok(ControllerWire::Ground),
    }
}

fn write_command_controller<W: Write>(write: &mut W, command: &CommandWire) -> io::Result<()> {
    write_u8(write, 9)?;
    write_u8(write, command.attack_target.is_some() as u8)?;
    write_u8(write, command.target_pos.is_some() as u8)?;

    if let Some(target_pos) = command.target_pos {
        write_vec2(write, target_pos)?;
    }

    if let Some(target) = command.attack_target {
        match target {
            ControllerTarget::BuildingPos(pos) => {
                write_u8(write, 1)?;
                write_i32(write, pos)?;
            }
            ControllerTarget::UnitId(id) => {
                write_u8(write, 0)?;
                write_i32(write, id)?;
            }
        }
    }

    write_optional_signed_byte_id(write, command.command_id)?;
    if command.command_queue.len() > u8::MAX as usize {
        return Err(invalid_input("command queue too large"));
    }
    write_u8(write, command.command_queue.len() as u8)?;
    for entry in &command.command_queue {
        match entry {
            CommandQueueEntry::BuildingPos(pos) => {
                write_u8(write, 0)?;
                write_i32(write, *pos)?;
            }
            CommandQueueEntry::UnitId(id) => {
                write_u8(write, 1)?;
                write_i32(write, *id)?;
            }
            CommandQueueEntry::Point(point) => {
                write_u8(write, 2)?;
                write_vec2(write, *point)?;
            }
            CommandQueueEntry::Invalid => {
                write_u8(write, 3)?;
            }
        }
    }

    if command.stances.len() > u8::MAX as usize {
        return Err(invalid_input("stance list too large"));
    }
    write_u8(write, command.stances.len() as u8)?;
    for stance in &command.stances {
        write_stance_id(write, Some(*stance))?;
    }
    Ok(())
}

fn read_command_controller_body<R: Read>(read: &mut R, tag: u8) -> io::Result<CommandWire> {
    let has_attack = read_u8(read)? != 0;
    let has_pos = read_u8(read)? != 0;
    let target_pos = if has_pos {
        Some(read_vec2(read)?)
    } else {
        None
    };
    let attack_target = if has_attack {
        let entity_type = read_u8(read)?;
        let id = read_i32(read)?;
        if entity_type == 1 {
            Some(ControllerTarget::BuildingPos(id))
        } else {
            Some(ControllerTarget::UnitId(id))
        }
    } else {
        None
    };

    let command_id = if matches!(tag, 6 | 7 | 8 | 9) {
        read_optional_signed_byte_id(read)?
    } else {
        None
    };

    let mut command_queue = Vec::new();
    if matches!(tag, 7 | 8 | 9) {
        let len = read_u8(read)? as usize;
        command_queue.reserve(len);
        for _ in 0..len {
            let command_type = read_u8(read)?;
            let entry = match command_type {
                0 => CommandQueueEntry::BuildingPos(read_i32(read)?),
                1 => CommandQueueEntry::UnitId(read_i32(read)?),
                2 => CommandQueueEntry::Point(read_vec2(read)?),
                _ => CommandQueueEntry::Invalid,
            };
            command_queue.push(entry);
        }
    }

    let mut stances = Vec::new();
    if tag == 8 {
        stances.push(read_u8(read)? as ContentId);
    } else if tag == 9 {
        let count = read_u8(read)? as usize;
        stances.reserve(count);
        for _ in 0..count {
            stances.push(read_u8(read)? as ContentId);
        }
    }

    Ok(CommandWire {
        target_pos,
        attack_target,
        command_id,
        command_queue,
        stances,
    })
}

fn write_optional_byte_id<W: Write>(write: &mut W, id: Option<ContentId>) -> io::Result<()> {
    let value = match id {
        Some(id) if (0..=254).contains(&id) => id as u8,
        Some(_) => return Err(invalid_input("byte content id out of range")),
        None => u8::MAX,
    };
    write_u8(write, value)
}

fn write_optional_signed_byte_id<W: Write>(write: &mut W, id: Option<ContentId>) -> io::Result<()> {
    let value = match id {
        Some(id) if (0..=127).contains(&id) => id as u8,
        Some(_) => return Err(invalid_input("signed byte content id out of range")),
        None => u8::MAX,
    };
    write_u8(write, value)
}

fn read_optional_byte_id<R: Read>(read: &mut R) -> io::Result<Option<ContentId>> {
    let value = read_u8(read)?;
    if value == u8::MAX {
        Ok(None)
    } else {
        Ok(Some(value as ContentId))
    }
}

fn read_optional_signed_byte_id<R: Read>(read: &mut R) -> io::Result<Option<ContentId>> {
    let value = read_u8(read)? as i8;
    if value < 0 {
        Ok(None)
    } else {
        Ok(Some(value as ContentId))
    }
}

pub fn write_items<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    stack: &ItemStack,
) -> io::Result<()> {
    let item = if stack.item.is_empty() {
        None
    } else {
        Some(stack.item.as_str())
    };
    write_item(write, loader, item)?;
    write_i32(write, stack.amount)
}

pub fn read_items<R: Read>(read: &mut R, loader: &ContentLoader) -> io::Result<ItemStack> {
    let item = read_item(read, loader)?.unwrap_or_default();
    Ok(ItemStack::new(item, read_i32(read)?))
}

pub fn write_item_stacks<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    stacks: &[ItemStack],
) -> io::Result<()> {
    if stacks.len() > i16::MAX as usize {
        return Err(invalid_input("item stack array too large"));
    }
    write_i16(write, stacks.len() as i16)?;
    for stack in stacks {
        write_items(write, loader, stack)?;
    }
    Ok(())
}

pub fn read_item_stacks<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
) -> io::Result<Vec<ItemStack>> {
    let count = read_i16(read)?;
    if count < 0 || count as usize > MAX_ARRAY_SIZE {
        return Err(invalid_data("invalid item stack count"));
    }
    let mut stacks = Vec::with_capacity(count as usize);
    for _ in 0..count {
        stacks.push(read_items(read, loader)?);
    }
    Ok(stacks)
}

pub fn write_liquid_stack<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    stack: &LiquidStack,
) -> io::Result<()> {
    write_liquid(write, loader, Some(&stack.liquid))?;
    write_u32(write, stack.amount.to_bits())
}

pub fn read_liquid_stack<R: Read>(read: &mut R, loader: &ContentLoader) -> io::Result<LiquidStack> {
    let liquid =
        read_liquid(read, loader)?.ok_or_else(|| invalid_data("null liquid stack liquid"))?;
    Ok(LiquidStack::new(liquid, f32::from_bits(read_u32(read)?)))
}

pub fn write_liquid_stacks<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    stacks: &[LiquidStack],
) -> io::Result<()> {
    if stacks.len() > i16::MAX as usize {
        return Err(invalid_input("liquid stack array too large"));
    }
    write_i16(write, stacks.len() as i16)?;
    for stack in stacks {
        write_liquid_stack(write, loader, stack)?;
    }
    Ok(())
}

pub fn read_liquid_stacks<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
) -> io::Result<Vec<LiquidStack>> {
    let count = read_i16(read)?;
    if count < 0 || count as usize > MAX_ARRAY_SIZE {
        return Err(invalid_data("invalid liquid stack count"));
    }
    let mut stacks = Vec::with_capacity(count as usize);
    for _ in 0..count {
        stacks.push(read_liquid_stack(read, loader)?);
    }
    Ok(stacks)
}

pub fn get_max_plans(plans: &[BuildPlanWire]) -> usize {
    let mut used = plans.len().min(MAX_NET_BUILD_PLANS);
    let mut total_length = 0usize;

    for (index, plan) in plans.iter().take(used).enumerate() {
        total_length += build_plan_config_wire_len(&plan.config);
        if total_length > MAX_NET_BUILD_PLAN_CONFIG_CHARS {
            used = index + 1;
            break;
        }
    }

    used
}

pub fn get_max_build_plans(plans: &[BuildPlan]) -> usize {
    let mut used = plans.len().min(MAX_NET_BUILD_PLANS);
    let mut total_length = 0usize;

    for (index, plan) in plans.iter().take(used).enumerate() {
        total_length += build_plan_config_wire_len(&plan.config);
        if total_length > MAX_NET_BUILD_PLAN_CONFIG_CHARS {
            used = index + 1;
            break;
        }
    }

    used
}

pub fn write_plan<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    plan: &BuildPlanWire,
) -> io::Result<()> {
    write_u8(write, plan.breaking as u8)?;
    write_i32(write, point2_pack(plan.x, plan.y))?;
    if !plan.breaking {
        let block = plan
            .block
            .as_deref()
            .ok_or_else(|| invalid_input("place plan missing block"))?;
        write_block(write, loader, Some(block))?;
        write_u8(write, plan.rotation as u8)?;
        write_u8(write, 1)?;
        write_object(write, &plan.config)?;
    }
    Ok(())
}

pub fn read_plan<R: Read>(read: &mut R, loader: &ContentLoader) -> io::Result<BuildPlanWire> {
    let plan_type = read_u8(read)?;
    let position = read_i32(read)?;
    let x = point2_x(position) as i32;
    let y = point2_y(position) as i32;

    if plan_type == 1 {
        return Ok(BuildPlanWire::new_break(x, y));
    }

    let block = read_required_block_name(read, loader)?;
    let rotation = read_u8(read)? as i32;
    let has_config = read_u8(read)? == 1;
    let config = read_object_safe(read)?;
    Ok(BuildPlanWire {
        x,
        y,
        rotation,
        block: Some(block),
        breaking: false,
        config: if has_config { config } else { TypeValue::Null },
    })
}

pub fn write_build_plan<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    plan: &BuildPlan,
) -> io::Result<()> {
    write_plan(write, loader, &BuildPlanWire::from_build_plan(plan))
}

pub fn read_build_plan<R: Read>(read: &mut R, loader: &ContentLoader) -> io::Result<BuildPlan> {
    read_plan(read, loader)?.to_build_plan()
}

pub fn write_plans<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    plans: Option<&[BuildPlanWire]>,
) -> io::Result<()> {
    let Some(plans) = plans else {
        return write_i16(write, -1);
    };
    if plans.len() > i16::MAX as usize {
        return Err(invalid_input("build plan array too large"));
    }
    write_i16(write, plans.len() as i16)?;
    for plan in plans {
        write_plan(write, loader, plan)?;
    }
    Ok(())
}

pub fn read_plans<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
) -> io::Result<Option<Vec<BuildPlanWire>>> {
    let count = read_i16(read)?;
    if count == -1 {
        return Ok(None);
    }
    if count < -1 || count as usize > MAX_ARRAY_SIZE {
        return Err(invalid_data("invalid build plan count"));
    }
    let mut plans = Vec::with_capacity(count as usize);
    for _ in 0..count {
        plans.push(read_plan(read, loader)?);
    }
    Ok(Some(plans))
}

pub fn write_build_plans<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    plans: Option<&[BuildPlan]>,
) -> io::Result<()> {
    let Some(plans) = plans else {
        return write_i16(write, -1);
    };
    if plans.len() > i16::MAX as usize {
        return Err(invalid_input("build plan array too large"));
    }
    write_i16(write, plans.len() as i16)?;
    for plan in plans {
        write_build_plan(write, loader, plan)?;
    }
    Ok(())
}

pub fn read_build_plans<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
) -> io::Result<Option<Vec<BuildPlan>>> {
    let Some(plans) = read_plans(read, loader)? else {
        return Ok(None);
    };
    plans
        .iter()
        .map(BuildPlanWire::to_build_plan)
        .collect::<io::Result<Vec<_>>>()
        .map(Some)
}

pub fn write_plans_queue_net<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    plans: Option<&[BuildPlanWire]>,
) -> io::Result<()> {
    let Some(plans) = plans else {
        return write_i32(write, -1);
    };
    let used = get_max_plans(plans);
    write_i32(write, used as i32)?;
    for plan in plans.iter().take(used) {
        write_plan(write, loader, plan)?;
    }
    Ok(())
}

pub fn read_plans_queue<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
) -> io::Result<Option<Vec<BuildPlanWire>>> {
    let used = read_i32(read)?;
    if used == -1 {
        return Ok(None);
    }
    if used < -1 || used as usize >= MAX_ARRAY_SIZE {
        return Err(invalid_data("build plan queue too long"));
    }
    let mut out = Vec::with_capacity(used as usize);
    for _ in 0..used {
        out.push(read_plan(read, loader)?);
    }
    Ok(Some(out))
}

pub fn write_build_plans_queue_net<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    plans: Option<&[BuildPlan]>,
) -> io::Result<()> {
    let Some(plans) = plans else {
        return write_i32(write, -1);
    };
    let used = get_max_build_plans(plans);
    write_i32(write, used as i32)?;
    for plan in plans.iter().take(used) {
        write_build_plan(write, loader, plan)?;
    }
    Ok(())
}

pub fn read_build_plans_queue<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
) -> io::Result<Option<Vec<BuildPlan>>> {
    let Some(plans) = read_plans_queue(read, loader)? else {
        return Ok(None);
    };
    plans
        .iter()
        .map(BuildPlanWire::to_build_plan)
        .collect::<io::Result<Vec<_>>>()
        .map(Some)
}

pub fn write_client_plans<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    plans: Option<&[BuildPlanWire]>,
) -> io::Result<()> {
    let Some(plans) = plans else {
        return write_i16(write, 0);
    };
    if plans.len() > i16::MAX as usize {
        return Err(invalid_input("client plan array too large"));
    }
    write_i16(write, plans.len() as i16)?;
    for plan in plans {
        if plan.breaking {
            return Err(invalid_input("breaking client plan"));
        }
        write_i32(write, point2_pack(plan.x, plan.y))?;
        let block_name = plan
            .block
            .as_deref()
            .ok_or_else(|| invalid_input("client plan missing block"))?;
        write_block(write, loader, Some(block_name))?;
        if block_rotates(loader, block_name)? {
            write_u8(write, plan.rotation as u8)?;
        }
        if valid_client_plan_config(&plan.config) {
            write_object(write, &plan.config)?;
        } else {
            write_object(write, &TypeValue::Null)?;
        }
    }
    Ok(())
}

pub fn read_client_plans<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
) -> io::Result<Option<Vec<BuildPlanWire>>> {
    let amount = read_i16(read)?;
    if amount == 0 {
        return Ok(None);
    }
    if amount < 0 || amount as usize > MAX_PLAYER_PREVIEW_PLANS {
        return Err(invalid_data("too many client plans"));
    }

    let mut result = Vec::with_capacity(amount as usize);
    for _ in 0..amount {
        let position = read_i32(read)?;
        let x = point2_x(position) as i32;
        let y = point2_y(position) as i32;
        let block = read_required_block_name(read, loader)?;
        let rotation = if block_rotates(loader, &block)? {
            read_u8(read)? as i32
        } else {
            0
        };
        let config = read_client_plan_config(read)?;
        result.push(BuildPlanWire {
            x,
            y,
            rotation,
            block: Some(block),
            breaking: false,
            config,
        });
    }

    Ok(Some(result))
}

pub fn read_client_plan_config<R: Read>(read: &mut R) -> io::Result<TypeValue> {
    let tag = read_u8(read)?;
    match tag {
        0 => Ok(TypeValue::Null),
        1 => Ok(TypeValue::Int(read_i32(read)?)),
        2 => Ok(TypeValue::Long(read_i64(read)?)),
        3 => Ok(TypeValue::Float(f32::from_bits(read_u32(read)?))),
        5 => Ok(TypeValue::Content(read_content_ref(read)?)),
        10 => Ok(TypeValue::Bool(read_u8(read)? != 0)),
        11 => Ok(TypeValue::Double(f64::from_bits(read_u64(read)?))),
        _ => Err(invalid_data("unknown client plan config object type")),
    }
}

pub fn valid_client_plan_config(value: &TypeValue) -> bool {
    matches!(
        value,
        TypeValue::Null
            | TypeValue::Int(_)
            | TypeValue::Long(_)
            | TypeValue::Float(_)
            | TypeValue::Double(_)
            | TypeValue::Bool(_)
            | TypeValue::Content(_)
    )
}

fn build_plan_config_wire_len(value: &TypeValue) -> usize {
    match value {
        TypeValue::String(value) => value.encode_utf16().count(),
        TypeValue::ByteArray(value) => value.len(),
        _ => 0,
    }
}

fn block_rotates(loader: &ContentLoader, block_name: &str) -> io::Result<bool> {
    let block = loader
        .catalog()
        .blocks
        .get_by_name(block_name)
        .ok_or_else(|| invalid_data("unknown block name"))?;
    Ok(match block {
        BlockDef::Production(block) => block.rotate,
        BlockDef::Turret(block) => block.rotate,
        BlockDef::Distribution(block) => block.rotate,
        BlockDef::Liquid(block) => block.rotate,
        BlockDef::Power(block) => block.rotate,
        BlockDef::Crafting(block) => block.rotate,
        BlockDef::UnitFactory(block) => block.rotate,
        BlockDef::UnitAssembler(block) => block.rotate,
        BlockDef::UnitAssemblerModule(block) => block.rotate,
        BlockDef::Payload(block) => block.rotate,
        BlockDef::PayloadMassDriver(block) => block.rotate,
        BlockDef::PayloadDeconstructor(block) => block.rotate,
        BlockDef::PayloadConstructor(block) => block.rotate,
        BlockDef::PayloadLoader(block) => block.rotate,
        BlockDef::Sandbox(block) => block.rotate,
        _ => false,
    })
}

pub fn write_status<W: Write>(write: &mut W, entry: &StatusEntry) -> io::Result<()> {
    let effect = entry
        .effect
        .as_ref()
        .ok_or_else(|| invalid_input("status entry missing effect"))?;
    write_i16(write, effect.base.mappable.base.id)?;
    write_u32(write, entry.time.to_bits())?;

    if effect.dynamic {
        let flags = dynamic_status_flags(entry);
        write_u8(write, flags)?;
        if flags & (1 << 0) != 0 {
            write_u32(write, entry.damage_multiplier.to_bits())?;
        }
        if flags & (1 << 1) != 0 {
            write_u32(write, entry.health_multiplier.to_bits())?;
        }
        if flags & (1 << 2) != 0 {
            write_u32(write, entry.speed_multiplier.to_bits())?;
        }
        if flags & (1 << 3) != 0 {
            write_u32(write, entry.reload_multiplier.to_bits())?;
        }
        if flags & (1 << 4) != 0 {
            write_u32(write, entry.build_speed_multiplier.to_bits())?;
        }
        if flags & (1 << 5) != 0 {
            write_u32(write, entry.drag_multiplier.to_bits())?;
        }
        if flags & (1 << 6) != 0 {
            write_u32(write, entry.armor_override.to_bits())?;
        }
    }

    Ok(())
}

pub fn read_status<R: Read>(read: &mut R, loader: &ContentLoader) -> io::Result<StatusEntry> {
    let id = read_i16(read)?;
    let time = f32::from_bits(read_u32(read)?);
    let effect = loader
        .catalog()
        .status_effect_by_id(id)
        .cloned()
        .ok_or_else(|| invalid_data("unknown status effect id"))?;
    let dynamic = effect.dynamic;
    let mut entry = StatusEntry::new(effect, time);

    if dynamic {
        let flags = read_u8(read)?;
        if flags & (1 << 0) != 0 {
            entry.damage_multiplier = f32::from_bits(read_u32(read)?);
        }
        if flags & (1 << 1) != 0 {
            entry.health_multiplier = f32::from_bits(read_u32(read)?);
        }
        if flags & (1 << 2) != 0 {
            entry.speed_multiplier = f32::from_bits(read_u32(read)?);
        }
        if flags & (1 << 3) != 0 {
            entry.reload_multiplier = f32::from_bits(read_u32(read)?);
        }
        if flags & (1 << 4) != 0 {
            entry.build_speed_multiplier = f32::from_bits(read_u32(read)?);
        }
        if flags & (1 << 5) != 0 {
            entry.drag_multiplier = f32::from_bits(read_u32(read)?);
        }
        if flags & (1 << 6) != 0 {
            entry.armor_override = f32::from_bits(read_u32(read)?);
        }
    }

    Ok(entry)
}

pub fn dynamic_status_flags(entry: &StatusEntry) -> u8 {
    (if entry.damage_multiplier != 1.0 {
        1 << 0
    } else {
        0
    }) | (if entry.health_multiplier != 1.0 {
        1 << 1
    } else {
        0
    }) | (if entry.speed_multiplier != 1.0 {
        1 << 2
    } else {
        0
    }) | (if entry.reload_multiplier != 1.0 {
        1 << 3
    } else {
        0
    }) | (if entry.build_speed_multiplier != 1.0 {
        1 << 4
    } else {
        0
    }) | (if entry.drag_multiplier != 1.0 {
        1 << 5
    } else {
        0
    }) | (if entry.armor_override >= 0.0 {
        1 << 6
    } else {
        0
    })
}

pub fn write_kick<W: Write>(write: &mut W, reason: KickReason) -> io::Result<()> {
    write_u8(write, reason.ordinal())
}

pub fn read_kick<R: Read>(read: &mut R) -> io::Result<KickReason> {
    let ordinal = read_u8(read)?;
    KickReason::from_ordinal(ordinal).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid KickReason ordinal {ordinal}"),
        )
    })
}

pub fn write_marker_control<W: Write>(write: &mut W, control: LMarkerControl) -> io::Result<()> {
    write_u8(write, control.ordinal())
}

pub fn read_marker_control<R: Read>(read: &mut R) -> io::Result<LMarkerControl> {
    let ordinal = read_u8(read)?;
    LMarkerControl::from_ordinal(ordinal).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid LMarkerControl ordinal {ordinal}"),
        )
    })
}

pub fn write_action<W: Write>(write: &mut W, action: AdminAction) -> io::Result<()> {
    write_u8(write, action.ordinal())
}

pub fn read_action<R: Read>(read: &mut R) -> io::Result<AdminAction> {
    let ordinal = read_u8(read)?;
    AdminAction::from_ordinal(ordinal).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid AdminAction ordinal {ordinal}"),
        )
    })
}

pub fn write_rules_json<W: Write>(write: &mut W, json: &str) -> io::Result<()> {
    write_json_bytes(write, json)
}

pub fn read_rules_json<R: Read>(read: &mut R) -> io::Result<String> {
    read_json_bytes(read, MAX_RULES_BYTES, false, "rules")
}

pub fn write_objectives_json<W: Write>(write: &mut W, json: &str) -> io::Result<()> {
    write_json_bytes(write, json)
}

pub fn read_objectives_json<R: Read>(read: &mut R) -> io::Result<String> {
    read_json_bytes(read, MAX_OBJECTIVES_BYTES, true, "objectives")
}

pub fn write_objective_marker_json<W: Write>(write: &mut W, json: &str) -> io::Result<()> {
    write_json_bytes(write, json)
}

pub fn read_objective_marker_json<R: Read>(read: &mut R) -> io::Result<String> {
    read_json_bytes(read, MAX_BYTE_ARRAY_SIZE, false, "objective marker")
}

fn write_json_bytes<W: Write>(write: &mut W, json: &str) -> io::Result<()> {
    let bytes = json.as_bytes();
    if bytes.len() > i32::MAX as usize {
        return Err(invalid_input("json payload too large"));
    }
    write_i32(write, bytes.len() as i32)?;
    write.write_all(bytes)
}

fn read_json_bytes<R: Read>(
    read: &mut R,
    max_len: usize,
    reject_equal: bool,
    label: &'static str,
) -> io::Result<String> {
    let len = read_i32(read)?;
    if len < 0 {
        return Err(invalid_data("negative json payload length"));
    }
    let len = len as usize;
    if len > max_len || (reject_equal && len == max_len) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{label} json payload too long"),
        ));
    }
    let mut bytes = vec![0; len];
    read.read_exact(&mut bytes)?;
    String::from_utf8(bytes).map_err(|_| invalid_data("invalid UTF-8 json payload"))
}

pub fn write_bytes_short<W: Write>(write: &mut W, values: &[u8]) -> io::Result<()> {
    if values.len() > i16::MAX as usize {
        return Err(invalid_input("byte block too large"));
    }
    write_i16(write, values.len() as i16)?;
    write.write_all(values)
}

pub fn read_bytes_short<R: Read>(read: &mut R) -> io::Result<Vec<u8>> {
    let len = read_i16(read)?;
    if len < 0 {
        return Err(invalid_data("invalid byte block length"));
    }
    let mut bytes = vec![0; len as usize];
    read.read_exact(&mut bytes)?;
    Ok(bytes)
}

pub fn write_bytes<W: Write>(write: &mut W, values: &[u8]) -> io::Result<()> {
    write_bytes_short(write, values)
}

pub fn read_bytes<R: Read>(read: &mut R) -> io::Result<Vec<u8>> {
    read_bytes_short(read)
}

pub fn write_ints<W: Write>(write: &mut W, values: &[i32]) -> io::Result<()> {
    if values.len() > i16::MAX as usize {
        return Err(invalid_input("int array too large"));
    }
    write_i16(write, values.len() as i16)?;
    for value in values {
        write_i32(write, *value)?;
    }
    Ok(())
}

pub fn read_ints<R: Read>(read: &mut R) -> io::Result<Vec<i32>> {
    let len = read_i16(read)?;
    if len < 0 {
        return Err(invalid_data("invalid int array length"));
    }
    let mut values = Vec::with_capacity(len as usize);
    for _ in 0..len {
        values.push(read_i32(read)?);
    }
    Ok(values)
}

pub fn write_int_seq<W: Write>(write: &mut W, values: &[i32]) -> io::Result<()> {
    if values.len() > i32::MAX as usize {
        return Err(invalid_input("int seq too large"));
    }
    write_i32(write, values.len() as i32)?;
    for value in values {
        write_i32(write, *value)?;
    }
    Ok(())
}

pub fn read_int_seq<R: Read>(read: &mut R) -> io::Result<Vec<i32>> {
    let len = read_i32(read)?;
    if len < 0 || len as usize > MAX_ARRAY_SIZE {
        return Err(invalid_data("invalid int seq length"));
    }
    let mut values = Vec::with_capacity(len as usize);
    for _ in 0..len {
        values.push(read_i32(read)?);
    }
    Ok(values)
}

pub fn write_strings<W: Write>(write: &mut W, values: &[Option<&str>]) -> io::Result<()> {
    if values.len() > u8::MAX as usize {
        return Err(invalid_input("string array too large"));
    }
    write.write_all(&[values.len() as u8])?;
    for value in values {
        write_string(write, *value)?;
    }
    Ok(())
}

pub fn write_strings_limited<W: Write>(
    write: &mut W,
    values: &[Option<&str>],
    max_len: usize,
) -> io::Result<()> {
    let len = values.len().min(max_len);
    if len > u8::MAX as usize {
        return Err(invalid_input("string array too large"));
    }
    write.write_all(&[len as u8])?;
    for value in values.iter().take(len) {
        write_string(write, *value)?;
    }
    Ok(())
}

pub fn read_strings<R: Read>(read: &mut R) -> io::Result<Vec<Option<String>>> {
    let len = read_u8(read)? as usize;
    let mut values = Vec::with_capacity(len);
    for _ in 0..len {
        values.push(read_string(read)?);
    }
    Ok(values)
}

pub fn write_string_array<W: Write>(write: &mut W, rows: &[Vec<Option<&str>>]) -> io::Result<()> {
    if rows.len() > u8::MAX as usize {
        return Err(invalid_input("string matrix too many rows"));
    }
    write.write_all(&[rows.len() as u8])?;
    for row in rows {
        if row.len() > u8::MAX as usize {
            return Err(invalid_input("string matrix too many columns"));
        }
        write.write_all(&[row.len() as u8])?;
        for value in row {
            write_string(write, *value)?;
        }
    }
    Ok(())
}

pub fn read_string_array<R: Read>(read: &mut R) -> io::Result<Vec<Vec<Option<String>>>> {
    let rows = read_u8(read)? as usize;
    let mut values = Vec::with_capacity(rows);
    for _ in 0..rows {
        let columns = read_u8(read)? as usize;
        let mut row = Vec::with_capacity(columns);
        for _ in 0..columns {
            row.push(read_string(read)?);
        }
        values.push(row);
    }
    Ok(values)
}

pub fn write_string_data<W: Write>(write: &mut W, string: Option<&str>) -> io::Result<()> {
    match string {
        Some(string) => {
            let bytes = string.as_bytes();
            if bytes.len() > i16::MAX as usize {
                return Err(invalid_input("string data too large"));
            }
            write_i16(write, bytes.len() as i16)?;
            write.write_all(bytes)
        }
        None => write_i16(write, -1),
    }
}

pub fn read_string_data<R: Read>(read: &mut R) -> io::Result<Option<String>> {
    let len = read_i16(read)?;
    if len == -1 {
        return Ok(None);
    }
    if len < -1 {
        return Err(invalid_data("invalid string data length"));
    }
    let mut bytes = vec![0; len as usize];
    read.read_exact(&mut bytes)?;
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| invalid_data("invalid UTF-8 string data"))
}

pub fn write_bytebuffer_string<W: Write>(write: &mut W, string: Option<&str>) -> io::Result<()> {
    write_string_data(write, string)
}

pub fn read_bytebuffer_string<R: Read>(read: &mut R) -> io::Result<Option<String>> {
    read_string_data(read)
}

pub fn write_trace_info<W: Write>(write: &mut W, trace: &TraceInfo) -> io::Result<()> {
    write_string(write, trace.ip.as_deref())?;
    write_string(write, trace.uuid.as_deref())?;
    write_string(write, trace.locale.as_deref())?;
    write_u8(write, trace.modded as u8)?;
    write_u8(write, trace.mobile as u8)?;
    write_i32(write, trace.times_joined)?;
    write_i32(write, trace.times_kicked)?;
    write_strings_limited(
        write,
        &borrow_optional_strings(&trace.ips),
        TraceInfo::MAX_HISTORY_LEN,
    )?;
    write_strings_limited(
        write,
        &borrow_optional_strings(&trace.names),
        TraceInfo::MAX_HISTORY_LEN,
    )
}

pub fn read_trace_info<R: Read>(read: &mut R) -> io::Result<TraceInfo> {
    Ok(TraceInfo {
        ip: read_string(read)?,
        uuid: read_string(read)?,
        locale: read_string(read)?,
        modded: read_u8(read)? == 1,
        mobile: read_u8(read)? == 1,
        times_joined: read_i32(read)?,
        times_kicked: read_i32(read)?,
        ips: read_strings(read)?,
        names: read_strings(read)?,
    })
}

fn borrow_optional_strings(values: &[Option<String>]) -> Vec<Option<&str>> {
    values.iter().map(|value| value.as_deref()).collect()
}

fn invalid_input(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nullable_string_matches_typeio_prefix_and_java_utf() {
        let mut bytes = Vec::new();
        write_string(&mut bytes, None).unwrap();
        assert_eq!(bytes, vec![0]);
        assert_eq!(read_string(&mut bytes.as_slice()).unwrap(), None);

        bytes.clear();
        write_string(&mut bytes, Some("Hi")).unwrap();
        assert_eq!(bytes, vec![1, 0, 2, b'H', b'i']);
        assert_eq!(
            read_string(&mut bytes.as_slice()).unwrap(),
            Some("Hi".to_string())
        );
    }

    #[test]
    fn java_modified_utf_handles_nul_and_supplementary_chars() {
        let value = "A\0水😀";
        let mut bytes = Vec::new();
        write_java_utf(&mut bytes, value).unwrap();
        assert_eq!(read_java_utf(&mut bytes.as_slice()).unwrap(), value);
        assert!(bytes.windows(2).any(|w| w == [0xc0, 0x80]));
    }

    #[test]
    fn tagged_values_roundtrip_basic_subset() {
        let value = TypeValue::ObjectArray(vec![
            TypeValue::Int(7),
            TypeValue::String("abc".into()),
            TypeValue::Bool(true),
        ]);
        let mut bytes = Vec::new();
        write_object(&mut bytes, &value).unwrap();
        assert_eq!(read_object(&mut bytes.as_slice()).unwrap(), value);

        bytes.clear();
        let ints = TypeValue::IntArray(vec![1, 2, 3]);
        write_object(&mut bytes, &ints).unwrap();
        assert_eq!(read_object(&mut bytes.as_slice()).unwrap(), ints);
    }

    #[test]
    fn boxed_object_reader_matches_java_processor_var_limits() {
        let long = TypeValue::String("x".repeat(1201));
        let mut bytes = Vec::new();
        write_object(&mut bytes, &long).unwrap();
        assert_eq!(read_object_boxed(&mut bytes.as_slice()).unwrap(), long);

        let mut ok = vec![21];
        write_i16(&mut ok, 200).unwrap();
        for value in 0..200 {
            write_i32(&mut ok, value).unwrap();
        }
        assert_eq!(
            read_object_boxed(&mut ok.as_slice()).unwrap(),
            TypeValue::IntArray((0..200).collect())
        );

        let mut too_many = vec![21];
        write_i16(&mut too_many, 201).unwrap();
        assert!(read_object_boxed(&mut too_many.as_slice()).is_err());
    }

    #[test]
    fn object_reader_limits_match_java_safe_and_non_safe_modes() {
        let safe_max = TypeValue::String("x".repeat(MAX_SAFE_STRING_CHARS));
        let mut bytes = Vec::new();
        write_object(&mut bytes, &safe_max).unwrap();
        assert_eq!(read_object_safe(&mut bytes.as_slice()).unwrap(), safe_max);

        let safe_too_long = TypeValue::String("x".repeat(MAX_SAFE_STRING_CHARS + 1));
        bytes.clear();
        write_object(&mut bytes, &safe_too_long).unwrap();
        assert!(read_object_safe(&mut bytes.as_slice()).is_err());

        let mut too_many = vec![21];
        write_i16(&mut too_many, (MAX_OBJECT_READ_ARRAY_SIZE + 1) as i16).unwrap();
        assert!(read_object(&mut too_many.as_slice()).is_err());
    }

    #[test]
    fn content_and_unit_command_object_tags_match_java_typeio() {
        let content = TypeValue::Content(ContentRef::new(ContentType::Block, 42));
        let tech = TypeValue::TechNode(ContentRef::new(ContentType::Unit, 7));
        let command = TypeValue::UnitCommand(9);

        let mut bytes = Vec::new();
        write_object(&mut bytes, &content).unwrap();
        assert_eq!(bytes, vec![5, ContentType::Block.ordinal(), 0, 42]);
        assert_eq!(read_object(&mut bytes.as_slice()).unwrap(), content);

        bytes.clear();
        write_object(&mut bytes, &tech).unwrap();
        assert_eq!(bytes, vec![9, ContentType::Unit.ordinal(), 0, 7]);
        assert_eq!(read_object(&mut bytes.as_slice()).unwrap(), tech);

        bytes.clear();
        write_object(&mut bytes, &command).unwrap();
        assert_eq!(bytes, vec![23, 0, 9]);
        assert_eq!(read_object(&mut bytes.as_slice()).unwrap(), command);
    }

    #[test]
    fn content_ref_helpers_resolve_through_content_loader_and_reject_bad_type() {
        let loader = ContentLoader::create_base_content().unwrap();
        let value = ContentRef::new(ContentType::Block, 1);

        let mut bytes = Vec::new();
        write_content_ref(&mut bytes, value).unwrap();
        assert_eq!(bytes, vec![ContentType::Block.ordinal(), 0, 1]);
        assert_eq!(read_content_ref(&mut bytes.as_slice()).unwrap(), value);

        let resolved = read_content_ref_resolved(&mut bytes.as_slice(), &loader)
            .unwrap()
            .unwrap();
        assert_eq!(resolved.content_type, ContentType::Block);
        assert_eq!(resolved.id, 1);
        assert_eq!(
            resolved.name(),
            loader.get_by_id(ContentType::Block, 1).unwrap().name()
        );

        assert!(read_content_ref(&mut [0xff, 0, 1].as_slice()).is_err());
    }

    #[test]
    fn entity_building_tile_and_required_content_refs_match_java_typeio_layout() {
        let loader = ContentLoader::create_base_content().unwrap();
        let mut bytes = Vec::new();

        write_entity_ref(&mut bytes, EntityRef::new(123)).unwrap();
        write_entity_ref(&mut bytes, EntityRef::null()).unwrap();
        assert_eq!(
            bytes,
            [123i32.to_be_bytes(), (-1i32).to_be_bytes()].concat()
        );
        let mut slice = bytes.as_slice();
        assert_eq!(read_entity_ref(&mut slice).unwrap(), EntityRef::new(123));
        assert_eq!(read_entity_ref(&mut slice).unwrap(), EntityRef::null());

        bytes.clear();
        let pos = point2_pack(4, 5);
        write_building_ref(&mut bytes, BuildingRef::new(pos)).unwrap();
        write_building_ref(&mut bytes, BuildingRef::null()).unwrap();
        assert_eq!(bytes, [pos.to_be_bytes(), (-1i32).to_be_bytes()].concat());
        let mut slice = bytes.as_slice();
        assert_eq!(
            read_building_ref(&mut slice).unwrap(),
            BuildingRef::new(pos)
        );
        assert_eq!(read_building_ref(&mut slice).unwrap(), BuildingRef::null());

        bytes.clear();
        write_tile_pos(&mut bytes, Some(pos)).unwrap();
        write_tile_pos(&mut bytes, None).unwrap();
        assert_eq!(
            bytes,
            [pos.to_be_bytes(), point2_pack(-1, -1).to_be_bytes()].concat()
        );
        let mut slice = bytes.as_slice();
        assert_eq!(read_tile_pos(&mut slice).unwrap(), Some(pos));
        assert_eq!(read_tile_pos(&mut slice).unwrap(), None);

        bytes.clear();
        write_required_content_ref(&mut bytes, &loader, ContentType::Item, "copper").unwrap();
        assert_eq!(bytes, vec![ContentType::Item.ordinal(), 0, 0]);
        assert_eq!(
            read_required_content_name(&mut bytes.as_slice(), &loader).unwrap(),
            (ContentType::Item, "copper".to_string())
        );
    }

    #[test]
    fn unit_container_roundtrip_uses_raw_unit_id_type_and_sync_bytes() {
        let container = UnitSyncContainer::new(12345, 7, vec![0xaa, 0xbb, 0xcc, 0xdd]);
        let mut bytes = Vec::new();

        write_unit_container(&mut bytes, &container).unwrap();
        assert_eq!(bytes, vec![0, 0, 0x30, 0x39, 7, 0xaa, 0xbb, 0xcc, 0xdd]);
        assert_eq!(
            read_unit_container(&mut bytes.as_slice()).unwrap(),
            container
        );
    }

    #[test]
    fn extended_object_and_wire_helpers_roundtrip_java_tags() {
        let building = TypeValue::Building(123456);
        let access = TypeValue::LogicAccess(LAccess::Enabled);
        let unit = TypeValue::Unit(7890);
        let legacy = [15, 0xaa];

        let mut bytes = Vec::new();
        write_object(&mut bytes, &building).unwrap();
        assert_eq!(read_object(&mut bytes.as_slice()).unwrap(), building);

        bytes.clear();
        write_object(&mut bytes, &access).unwrap();
        assert_eq!(read_object(&mut bytes.as_slice()).unwrap(), access);

        bytes.clear();
        write_object(&mut bytes, &unit).unwrap();
        assert_eq!(read_object(&mut bytes.as_slice()).unwrap(), unit);

        assert_eq!(
            read_object(&mut legacy.as_slice()).unwrap(),
            TypeValue::Null
        );

        let payload = Some(PayloadRef::Unit {
            class_id: 7,
            unit_bytes: vec![1, 2, 3],
        });
        bytes.clear();
        write_payload(&mut bytes, payload.as_ref()).unwrap();
        assert_eq!(read_payload(&mut bytes.as_slice()).unwrap(), payload);

        let mounts = vec![
            MountWire {
                shoot: true,
                rotate: false,
                aim_x: 1.5,
                aim_y: -2.0,
            },
            MountWire {
                shoot: false,
                rotate: true,
                aim_x: 4.25,
                aim_y: 9.5,
            },
        ];
        bytes.clear();
        write_mounts(&mut bytes, &mounts).unwrap();
        assert_eq!(read_mounts(&mut bytes.as_slice()).unwrap(), mounts);

        let abilities = vec![AbilityWire { data: 1.0 }, AbilityWire { data: -3.5 }];
        bytes.clear();
        write_abilities(&mut bytes, &abilities).unwrap();
        assert_eq!(read_abilities(&mut bytes.as_slice()).unwrap(), abilities);

        let ability_data = vec![0.5, 2.75];
        bytes.clear();
        write_ability_data(&mut bytes, &ability_data).unwrap();
        assert_eq!(
            read_ability_data(&mut bytes.as_slice()).unwrap(),
            ability_data
        );
    }

    #[test]
    fn fire_sync_wire_roundtrips_java_write_sync_shape() {
        let sync = FireSyncWire {
            lifetime: 90.0,
            tile_pos: Some(point2_pack(3, 4)),
            time: 12.5,
            x: 24.0,
            y: 32.0,
        };

        let mut bytes = Vec::new();
        write_fire_sync(&mut bytes, &sync).unwrap();
        assert_eq!(read_fire_sync(&mut bytes.as_slice()).unwrap(), sync);

        let sync_without_tile = FireSyncWire {
            tile_pos: None,
            ..sync
        };
        bytes.clear();
        write_fire_sync(&mut bytes, &sync_without_tile).unwrap();
        assert_eq!(
            read_fire_sync(&mut bytes.as_slice()).unwrap(),
            sync_without_tile
        );
    }

    #[test]
    fn decal_sync_wire_roundtrips_java_write_sync_shape() {
        let sync = DecalSyncWire {
            color: RgbaColor::new(0x88aabbccu32 as i32),
            lifetime: 60.0,
            rotation: 45.0,
            time: 12.5,
            x: 96.0,
            y: 128.0,
        };

        let mut bytes = Vec::new();
        write_decal_sync(&mut bytes, &sync).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&(0x88aabbccu32 as i32).to_be_bytes());
        expected.extend_from_slice(&sync.lifetime.to_be_bytes());
        // DecalComp revision 0 includes `region: TextureRegion`, but upstream
        // generated writeSync cannot serialize TextureRegion and emits no bytes.
        expected.extend_from_slice(&sync.rotation.to_be_bytes());
        expected.extend_from_slice(&sync.time.to_be_bytes());
        expected.extend_from_slice(&sync.x.to_be_bytes());
        expected.extend_from_slice(&sync.y.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(read_decal_sync(&mut bytes.as_slice()).unwrap(), sync);
    }

    #[test]
    fn bullet_sync_wire_roundtrips_java_revision_2_write_sync_shape() {
        let sync = BulletSyncWire {
            collided: vec![7, -9],
            damage: 33.5,
            data: TypeValue::String("pierce".into()),
            fdata: 1.25,
            lifetime: 120.0,
            owner: EntityRef::new(4242),
            rotation: 270.0,
            team: TeamId(6),
            time: 12.0,
            bullet_type_id: 5,
            vel: Vec2::new(-0.5, 2.25),
            x: 100.0,
            y: 200.0,
        };

        let mut bytes = Vec::new();
        write_bullet_sync(&mut bytes, &sync).unwrap();

        let mut expected = Vec::new();
        write_int_seq(&mut expected, &[7, -9]).unwrap();
        expected.extend_from_slice(&sync.damage.to_be_bytes());
        write_object(&mut expected, &TypeValue::String("pierce".into())).unwrap();
        expected.extend_from_slice(&sync.fdata.to_be_bytes());
        expected.extend_from_slice(&sync.lifetime.to_be_bytes());
        expected.extend_from_slice(&4242i32.to_be_bytes());
        expected.extend_from_slice(&sync.rotation.to_be_bytes());
        expected.push(6);
        expected.extend_from_slice(&sync.time.to_be_bytes());
        expected.extend_from_slice(&5i16.to_be_bytes());
        expected.extend_from_slice(&sync.vel.x.to_be_bytes());
        expected.extend_from_slice(&sync.vel.y.to_be_bytes());
        expected.extend_from_slice(&sync.x.to_be_bytes());
        expected.extend_from_slice(&sync.y.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(read_bullet_sync(&mut bytes.as_slice()).unwrap(), sync);

        let sync_without_owner = BulletSyncWire {
            owner: EntityRef::null(),
            data: TypeValue::Null,
            collided: Vec::new(),
            ..sync
        };
        bytes.clear();
        write_bullet_sync(&mut bytes, &sync_without_owner).unwrap();
        assert_eq!(
            read_bullet_sync(&mut bytes.as_slice()).unwrap(),
            sync_without_owner
        );
    }

    #[test]
    fn puddle_sync_wire_roundtrips_java_write_sync_shape() {
        let sync = PuddleSyncWire {
            amount: 42.5,
            liquid_id: Some(2),
            tile_pos: Some(point2_pack(7, 8)),
            x: 56.0,
            y: 64.0,
        };

        let mut bytes = Vec::new();
        write_puddle_sync(&mut bytes, &sync).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&sync.amount.to_be_bytes());
        expected.extend_from_slice(&2i16.to_be_bytes());
        expected.extend_from_slice(&point2_pack(7, 8).to_be_bytes());
        expected.extend_from_slice(&sync.x.to_be_bytes());
        expected.extend_from_slice(&sync.y.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(read_puddle_sync(&mut bytes.as_slice()).unwrap(), sync);

        let sync_without_refs = PuddleSyncWire {
            liquid_id: None,
            tile_pos: None,
            ..sync
        };
        bytes.clear();
        write_puddle_sync(&mut bytes, &sync_without_refs).unwrap();
        assert_eq!(
            read_puddle_sync(&mut bytes.as_slice()).unwrap(),
            sync_without_refs
        );
    }

    #[test]
    fn weather_state_sync_wire_roundtrips_java_write_sync_shape() {
        let sync = WeatherStateSyncWire {
            effect_timer: 11.0,
            intensity: 0.75,
            life: 600.0,
            opacity: 0.5,
            weather_id: Some(1),
            wind_vector: Vec2::new(0.25, -0.75),
            x: 10.0,
            y: 20.0,
        };

        let mut bytes = Vec::new();
        write_weather_state_sync(&mut bytes, &sync).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&sync.effect_timer.to_be_bytes());
        expected.extend_from_slice(&sync.intensity.to_be_bytes());
        expected.extend_from_slice(&sync.life.to_be_bytes());
        expected.extend_from_slice(&sync.opacity.to_be_bytes());
        expected.extend_from_slice(&1i16.to_be_bytes());
        expected.extend_from_slice(&sync.wind_vector.x.to_be_bytes());
        expected.extend_from_slice(&sync.wind_vector.y.to_be_bytes());
        expected.extend_from_slice(&sync.x.to_be_bytes());
        expected.extend_from_slice(&sync.y.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            read_weather_state_sync(&mut bytes.as_slice()).unwrap(),
            sync
        );

        let sync_without_weather = WeatherStateSyncWire {
            weather_id: None,
            ..sync
        };
        bytes.clear();
        write_weather_state_sync(&mut bytes, &sync_without_weather).unwrap();
        assert_eq!(
            read_weather_state_sync(&mut bytes.as_slice()).unwrap(),
            sync_without_weather
        );
    }

    #[test]
    fn effect_state_sync_wire_roundtrips_java_write_sync_shape() {
        let sync = EffectStateSyncWire {
            color: RgbaColor::new(0x336699cc),
            data: TypeValue::String("spark".into()),
            effect_id: 7,
            lifetime: 50.0,
            offset_pos: 1.25,
            offset_rot: -2.5,
            offset_x: 3.0,
            offset_y: 4.0,
            parent_id: Some(1234),
            rot_with_parent: true,
            rotation: 90.0,
            time: 12.0,
            x: 100.0,
            y: 200.0,
        };

        let mut bytes = Vec::new();
        write_effect_state_sync(&mut bytes, &sync).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&0x336699cci32.to_be_bytes());
        write_object(&mut expected, &TypeValue::String("spark".into())).unwrap();
        expected.extend_from_slice(&7u16.to_be_bytes());
        expected.extend_from_slice(&sync.lifetime.to_be_bytes());
        expected.extend_from_slice(&sync.offset_pos.to_be_bytes());
        expected.extend_from_slice(&sync.offset_rot.to_be_bytes());
        expected.extend_from_slice(&sync.offset_x.to_be_bytes());
        expected.extend_from_slice(&sync.offset_y.to_be_bytes());
        expected.extend_from_slice(&1234i32.to_be_bytes());
        expected.push(1);
        expected.extend_from_slice(&sync.rotation.to_be_bytes());
        expected.extend_from_slice(&sync.time.to_be_bytes());
        expected.extend_from_slice(&sync.x.to_be_bytes());
        expected.extend_from_slice(&sync.y.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(read_effect_state_sync(&mut bytes.as_slice()).unwrap(), sync);

        let sync_without_parent = EffectStateSyncWire {
            parent_id: None,
            rot_with_parent: false,
            data: TypeValue::Null,
            ..sync
        };
        bytes.clear();
        write_effect_state_sync(&mut bytes, &sync_without_parent).unwrap();
        assert_eq!(
            read_effect_state_sync(&mut bytes.as_slice()).unwrap(),
            sync_without_parent
        );
    }

    #[test]
    fn unit_sync_wire_roundtrips_the_java_field_order_subset() {
        let loader = ContentLoader::create_base_content().unwrap();
        let wet = loader
            .catalog()
            .status_effect_by_name("wet")
            .unwrap()
            .clone();
        let router_plan = BuildPlanWire::new_place(3, 4, 1, "router");
        let sync = UnitSyncWire {
            abilities: vec![AbilityWire { data: 1.25 }, AbilityWire { data: -4.5 }],
            ammo: 6.75,
            controller: ControllerWire::Ground,
            elevation: 0.5,
            flag: 123.456,
            health: 88.25,
            is_shooting: true,
            mine_tile: Some(point2_pack(11, -7)),
            mounts: vec![MountWire {
                shoot: true,
                rotate: false,
                aim_x: 4.0,
                aim_y: -8.0,
            }],
            plans: Some(vec![router_plan.clone()]),
            rotation: 270.0,
            shield: 12.5,
            spawned_by_core: false,
            stack: ItemStack::new("copper", 123),
            statuses: vec![StatusEntry::new(wet, 4.0)],
            team: TeamId(2),
            type_id: 17,
            update_building: true,
            vel: Vec2::new(1.5, -2.5),
            x: 55.0,
            y: -66.0,
        };

        let mut bytes = Vec::new();
        write_unit_sync(&mut bytes, &loader, &sync).unwrap();
        assert_eq!(
            read_unit_sync(&mut bytes.as_slice(), &loader).unwrap(),
            sync
        );
    }

    #[test]
    fn typed_content_helpers_match_java_short_and_byte_sentinels() {
        let loader = ContentLoader::create_base_content().unwrap();
        let mut bytes = Vec::new();

        write_block(&mut bytes, &loader, Some("router")).unwrap();
        assert_eq!(
            bytes,
            loader
                .get_by_name(ContentType::Block, "router")
                .unwrap()
                .id
                .to_be_bytes()
        );
        assert_eq!(
            read_block(&mut bytes.as_slice(), &loader).unwrap(),
            Some("router".into())
        );

        bytes.clear();
        write_item(&mut bytes, &loader, Some("copper")).unwrap();
        assert_eq!(bytes, vec![0, 0]);
        assert_eq!(
            read_item(&mut bytes.as_slice(), &loader).unwrap(),
            Some("copper".into())
        );

        bytes.clear();
        write_liquid(&mut bytes, &loader, None).unwrap();
        assert_eq!(bytes, vec![0xff, 0xff]);
        assert_eq!(read_liquid(&mut bytes.as_slice(), &loader).unwrap(), None);

        bytes.clear();
        write_unit_type(&mut bytes, &loader, "flare").unwrap();
        assert_eq!(bytes, vec![0, 15]);
        assert_eq!(
            read_unit_type(&mut bytes.as_slice(), &loader).unwrap(),
            "flare"
        );

        bytes.clear();
        write_bullet_type_id(&mut bytes, 5).unwrap();
        assert_eq!(bytes, vec![0, 5]);
        assert_eq!(read_bullet_type_id(&mut bytes.as_slice()).unwrap(), 5);

        bytes.clear();
        write_command_id(&mut bytes, None).unwrap();
        write_command_id(&mut bytes, Some(4)).unwrap();
        assert_eq!(bytes, vec![0xff, 4]);
        let mut slice = bytes.as_slice();
        assert_eq!(read_command_id(&mut slice).unwrap(), None);
        assert_eq!(
            read_command(&mut slice, &loader).unwrap().unwrap().name(),
            "mine"
        );

        bytes.clear();
        write_stance_id(&mut bytes, Some(7)).unwrap();
        assert_eq!(bytes, vec![7]);
        assert_eq!(
            read_stance(&mut bytes.as_slice(), &loader).unwrap().name(),
            "mineauto"
        );
        assert_eq!(
            read_stance(&mut [0xff].as_slice(), &loader).unwrap().name(),
            "stop"
        );
        assert_eq!(
            read_stance(&mut [0xfe].as_slice(), &loader).unwrap().name(),
            "stop"
        );
    }

    #[test]
    fn item_and_liquid_stacks_match_java_typeio_layout() {
        let loader = ContentLoader::create_base_content().unwrap();
        let mut bytes = Vec::new();

        let stack = ItemStack::new("copper", 123);
        write_items(&mut bytes, &loader, &stack).unwrap();
        assert_eq!(bytes, vec![0, 0, 0, 0, 0, 123]);
        assert_eq!(read_items(&mut bytes.as_slice(), &loader).unwrap(), stack);

        bytes.clear();
        let stacks = vec![ItemStack::new("lead", 1), ItemStack::new("scrap", 2)];
        write_item_stacks(&mut bytes, &loader, &stacks).unwrap();
        assert_eq!(&bytes[0..2], &[0, 2]);
        assert_eq!(
            read_item_stacks(&mut bytes.as_slice(), &loader).unwrap(),
            stacks
        );

        bytes.clear();
        let liquid = LiquidStack::new("water", 2.5);
        write_liquid_stack(&mut bytes, &loader, &liquid).unwrap();
        assert_eq!(&bytes[0..2], &[0, 0]);
        assert_eq!(&bytes[2..6], &2.5f32.to_bits().to_be_bytes());
        assert_eq!(
            read_liquid_stack(&mut bytes.as_slice(), &loader).unwrap(),
            liquid
        );

        bytes.clear();
        let liquids = vec![
            LiquidStack::new("water", 1.0),
            LiquidStack::new("slag", 3.25),
        ];
        write_liquid_stacks(&mut bytes, &loader, &liquids).unwrap();
        assert_eq!(&bytes[0..2], &[0, 2]);
        assert_eq!(
            read_liquid_stacks(&mut bytes.as_slice(), &loader).unwrap(),
            liquids
        );
    }

    #[test]
    fn build_plan_wire_matches_java_typeio_layout() {
        let loader = ContentLoader::create_base_content().unwrap();
        let block_id = loader
            .get_by_name(ContentType::Block, "router")
            .unwrap()
            .id
            .to_be_bytes();
        let plan =
            BuildPlanWire::new_place_config(3, 4, 2, "router", TypeValue::String("cfg".into()));
        let mut bytes = Vec::new();

        write_plan(&mut bytes, &loader, &plan).unwrap();
        let mut expected = vec![0];
        expected.extend_from_slice(&point2_pack(3, 4).to_be_bytes());
        expected.extend_from_slice(&block_id);
        expected.extend_from_slice(&[2, 1, 4, 1, 0, 3, b'c', b'f', b'g']);
        assert_eq!(bytes, expected);
        assert_eq!(read_plan(&mut bytes.as_slice(), &loader).unwrap(), plan);

        bytes.clear();
        let breaking = BuildPlanWire::new_break(-1, 2);
        write_plan(&mut bytes, &loader, &breaking).unwrap();
        let mut expected = vec![1];
        expected.extend_from_slice(&point2_pack(-1, 2).to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(read_plan(&mut bytes.as_slice(), &loader).unwrap(), breaking);

        let typed_config = TypeValue::Point2(Point2::new(9, 10));
        let local_plan = BuildPlan::new_config(7, 8, 1, "router", typed_config.clone());
        bytes.clear();
        write_build_plan(&mut bytes, &loader, &local_plan).unwrap();
        let decoded = read_build_plan(&mut bytes.as_slice(), &loader).unwrap();
        assert_eq!(decoded.config, typed_config);
    }

    #[test]
    fn build_plan_arrays_and_net_queue_use_java_sentinels_and_caps() {
        let loader = ContentLoader::create_base_content().unwrap();
        let plan =
            BuildPlanWire::new_place_config(1, 2, 0, "router", TypeValue::String("A".into()));
        let mut bytes = Vec::new();

        write_plans(&mut bytes, &loader, None).unwrap();
        assert_eq!(bytes, vec![0xff, 0xff]);
        assert_eq!(read_plans(&mut bytes.as_slice(), &loader).unwrap(), None);

        bytes.clear();
        write_plans(&mut bytes, &loader, Some(std::slice::from_ref(&plan))).unwrap();
        assert_eq!(&bytes[0..2], &[0, 1]);
        assert_eq!(
            read_plans(&mut bytes.as_slice(), &loader).unwrap(),
            Some(vec![plan.clone()])
        );

        bytes.clear();
        write_plans_queue_net(&mut bytes, &loader, None).unwrap();
        assert_eq!(bytes, (-1i32).to_be_bytes());
        assert_eq!(
            read_plans_queue(&mut bytes.as_slice(), &loader).unwrap(),
            None
        );

        let many: Vec<_> = (0..25)
            .map(|index| BuildPlanWire::new_place(index, index + 1, 0, "router"))
            .collect();
        assert_eq!(get_max_plans(&many), MAX_NET_BUILD_PLANS);
        bytes.clear();
        write_plans_queue_net(&mut bytes, &loader, Some(&many)).unwrap();
        assert_eq!(&bytes[0..4], &(MAX_NET_BUILD_PLANS as i32).to_be_bytes());
        assert_eq!(
            read_plans_queue(&mut bytes.as_slice(), &loader)
                .unwrap()
                .unwrap()
                .len(),
            MAX_NET_BUILD_PLANS
        );

        let capped = vec![
            BuildPlanWire::new_place_config(0, 0, 0, "router", TypeValue::String("a".repeat(250))),
            BuildPlanWire::new_place_config(1, 1, 0, "router", TypeValue::ByteArray(vec![0; 251])),
            BuildPlanWire::new_place(2, 2, 0, "router"),
        ];
        assert_eq!(get_max_plans(&capped), 2);
    }

    #[test]
    fn client_plans_filter_configs_and_elide_non_rotating_rotation() {
        let loader = ContentLoader::create_base_content().unwrap();
        let router_id = loader
            .get_by_name(ContentType::Block, "router")
            .unwrap()
            .id
            .to_be_bytes();
        let duo_id = loader
            .get_by_name(ContentType::Block, "duo")
            .unwrap()
            .id
            .to_be_bytes();
        let plans = vec![
            BuildPlanWire::new_place_config(3, 4, 2, "router", TypeValue::String("drop".into())),
            BuildPlanWire::new_place_config(
                5,
                6,
                1,
                "duo",
                TypeValue::Content(ContentRef::new(ContentType::Item, 0)),
            ),
        ];
        let mut bytes = Vec::new();

        write_client_plans(&mut bytes, &loader, Some(&plans)).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&2i16.to_be_bytes());
        expected.extend_from_slice(&point2_pack(3, 4).to_be_bytes());
        expected.extend_from_slice(&router_id);
        expected.push(0);
        expected.extend_from_slice(&point2_pack(5, 6).to_be_bytes());
        expected.extend_from_slice(&duo_id);
        expected.push(1);
        expected.extend_from_slice(&[5, ContentType::Item.ordinal(), 0, 0]);
        assert_eq!(bytes, expected);

        let decoded = read_client_plans(&mut bytes.as_slice(), &loader)
            .unwrap()
            .unwrap();
        assert_eq!(decoded[0].rotation, 0);
        assert_eq!(decoded[0].config, TypeValue::Null);
        assert_eq!(decoded[1].rotation, 1);
        assert_eq!(
            decoded[1].config,
            TypeValue::Content(ContentRef::new(ContentType::Item, 0))
        );

        bytes.clear();
        write_client_plans(&mut bytes, &loader, None).unwrap();
        assert_eq!(bytes, vec![0, 0]);
        assert_eq!(
            read_client_plans(&mut bytes.as_slice(), &loader).unwrap(),
            None
        );
        assert!(write_client_plans(
            &mut Vec::new(),
            &loader,
            Some(&[BuildPlanWire::new_break(1, 2)])
        )
        .is_err());
        assert!(read_client_plan_config(&mut [4, 1, 0, 1, b'x'].as_slice()).is_err());
    }

    #[test]
    fn controller_wire_simple_tags_match_java_typeio_layout() {
        let mut bytes = Vec::new();
        write_controller(&mut bytes, &ControllerWire::Player { player_id: 123456 }).unwrap();
        let mut expected = vec![0];
        expected.extend_from_slice(&123456i32.to_be_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            read_controller(&mut bytes.as_slice()).unwrap(),
            ControllerWire::Player { player_id: 123456 }
        );

        bytes.clear();
        write_controller(
            &mut bytes,
            &ControllerWire::Logic {
                controller_pos: 0x0102_0304,
            },
        )
        .unwrap();
        assert_eq!(bytes, vec![3, 1, 2, 3, 4]);
        assert_eq!(
            read_controller(&mut bytes.as_slice()).unwrap(),
            ControllerWire::Logic {
                controller_pos: 0x0102_0304
            }
        );

        bytes.clear();
        write_controller(&mut bytes, &ControllerWire::Ground).unwrap();
        assert_eq!(bytes, vec![2]);
        assert_eq!(
            read_controller(&mut bytes.as_slice()).unwrap(),
            ControllerWire::Ground
        );

        bytes.clear();
        write_controller(&mut bytes, &ControllerWire::Assembler).unwrap();
        assert_eq!(bytes, vec![5]);
        assert_eq!(
            read_controller(&mut bytes.as_slice()).unwrap(),
            ControllerWire::Assembler
        );

        bytes.clear();
        write_controller(&mut bytes, &ControllerWire::LegacyFormation { id: -77 }).unwrap();
        assert_eq!(&bytes[0..1], &[1]);
        assert_eq!(
            read_controller(&mut bytes.as_slice()).unwrap(),
            ControllerWire::LegacyFormation { id: -77 }
        );

        assert_eq!(
            read_controller(&mut [0x7f].as_slice()).unwrap(),
            ControllerWire::Ground
        );
    }

    #[test]
    fn command_controller_wire_current_tag_matches_java_layout() {
        let mut bytes = Vec::new();
        let minimal = ControllerWire::Command(CommandWire {
            command_id: Some(7),
            ..CommandWire::new()
        });
        write_controller(&mut bytes, &minimal).unwrap();
        assert_eq!(bytes, vec![9, 0, 0, 7, 0, 0]);
        assert_eq!(read_controller(&mut bytes.as_slice()).unwrap(), minimal);

        bytes.clear();
        let none_command = ControllerWire::Command(CommandWire::new());
        write_controller(&mut bytes, &none_command).unwrap();
        assert_eq!(bytes, vec![9, 0, 0, 0xff, 0, 0]);
        assert_eq!(
            read_controller(&mut bytes.as_slice()).unwrap(),
            none_command
        );

        bytes.clear();
        let full = ControllerWire::Command(CommandWire {
            target_pos: Some(Vec2::new(1.5, -2.25)),
            attack_target: Some(ControllerTarget::BuildingPos(42)),
            command_id: Some(3),
            command_queue: vec![
                CommandQueueEntry::BuildingPos(10),
                CommandQueueEntry::UnitId(11),
                CommandQueueEntry::Point(Vec2::new(12.0, 13.0)),
                CommandQueueEntry::Invalid,
            ],
            stances: vec![1, 7],
        });
        write_controller(&mut bytes, &full).unwrap();
        let mut expected = vec![9, 1, 1];
        expected.extend_from_slice(&1.5f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&(-2.25f32).to_bits().to_be_bytes());
        expected.push(1);
        expected.extend_from_slice(&42i32.to_be_bytes());
        expected.push(3);
        expected.push(4);
        expected.push(0);
        expected.extend_from_slice(&10i32.to_be_bytes());
        expected.push(1);
        expected.extend_from_slice(&11i32.to_be_bytes());
        expected.push(2);
        expected.extend_from_slice(&12.0f32.to_bits().to_be_bytes());
        expected.extend_from_slice(&13.0f32.to_bits().to_be_bytes());
        expected.push(3);
        expected.push(2);
        expected.extend_from_slice(&[1, 7]);
        assert_eq!(bytes, expected);
        assert_eq!(read_controller(&mut bytes.as_slice()).unwrap(), full);
    }

    #[test]
    fn command_controller_legacy_tags_read_compatible_layouts() {
        let mut tag4 = vec![4, 1, 1];
        tag4.extend_from_slice(&5.0f32.to_bits().to_be_bytes());
        tag4.extend_from_slice(&6.0f32.to_bits().to_be_bytes());
        tag4.push(0);
        tag4.extend_from_slice(&99i32.to_be_bytes());
        assert_eq!(
            read_controller(&mut tag4.as_slice()).unwrap(),
            ControllerWire::Command(CommandWire {
                target_pos: Some(Vec2::new(5.0, 6.0)),
                attack_target: Some(ControllerTarget::UnitId(99)),
                command_id: None,
                command_queue: Vec::new(),
                stances: Vec::new(),
            })
        );

        let tag6 = [6, 0, 0, 0xff];
        assert_eq!(
            read_controller(&mut tag6.as_slice()).unwrap(),
            ControllerWire::Command(CommandWire::new())
        );

        let mut tag7 = vec![7, 0, 0, 2, 2];
        tag7.push(2);
        tag7.extend_from_slice(&7.5f32.to_bits().to_be_bytes());
        tag7.extend_from_slice(&8.5f32.to_bits().to_be_bytes());
        tag7.push(99);
        assert_eq!(
            read_controller(&mut tag7.as_slice()).unwrap(),
            ControllerWire::Command(CommandWire {
                target_pos: None,
                attack_target: None,
                command_id: Some(2),
                command_queue: vec![
                    CommandQueueEntry::Point(Vec2::new(7.5, 8.5)),
                    CommandQueueEntry::Invalid,
                ],
                stances: Vec::new(),
            })
        );

        let tag8 = [8, 0, 0, 1, 0, 7];
        assert_eq!(
            read_controller(&mut tag8.as_slice()).unwrap(),
            ControllerWire::Command(CommandWire {
                target_pos: None,
                attack_target: None,
                command_id: Some(1),
                command_queue: Vec::new(),
                stances: vec![7],
            })
        );

        let tag9_with_stop_sentinel = [9, 0, 0, 0xff, 0, 1, 0xff];
        assert_eq!(
            read_controller(&mut tag9_with_stop_sentinel.as_slice()).unwrap(),
            ControllerWire::Command(CommandWire {
                target_pos: None,
                attack_target: None,
                command_id: None,
                command_queue: Vec::new(),
                stances: vec![255],
            })
        );
    }

    #[test]
    fn status_entry_serialization_keeps_dynamic_flag_order() {
        let loader = ContentLoader::create_base_content().unwrap();
        let dynamic = loader
            .catalog()
            .status_effect_by_name("dynamic")
            .unwrap()
            .clone();
        let dynamic_id = dynamic.base.mappable.base.id;
        let mut entry = StatusEntry::new(dynamic, 12.5);
        entry.damage_multiplier = 2.0;
        entry.speed_multiplier = 0.5;
        entry.drag_multiplier = 3.0;
        entry.armor_override = 8.0;

        assert_eq!(
            dynamic_status_flags(&entry),
            (1 << 0) | (1 << 2) | (1 << 5) | (1 << 6)
        );

        let mut bytes = Vec::new();
        write_status(&mut bytes, &entry).unwrap();
        assert_eq!(&bytes[0..2], &dynamic_id.to_be_bytes());
        assert_eq!(&bytes[2..6], &12.5f32.to_bits().to_be_bytes());
        assert_eq!(bytes[6], dynamic_status_flags(&entry));

        let decoded = read_status(&mut bytes.as_slice(), &loader).unwrap();
        assert_eq!(decoded.time, 12.5);
        assert_eq!(decoded.effect.as_ref().unwrap().name(), "dynamic");
        assert_eq!(decoded.damage_multiplier, 2.0);
        assert_eq!(decoded.speed_multiplier, 0.5);
        assert_eq!(decoded.drag_multiplier, 3.0);
        assert_eq!(decoded.armor_override, 8.0);

        let wet = loader
            .catalog()
            .status_effect_by_name("wet")
            .unwrap()
            .clone();
        let normal = StatusEntry::new(wet, 4.0);
        bytes.clear();
        write_status(&mut bytes, &normal).unwrap();
        assert_eq!(bytes.len(), 6);
        let decoded = read_status(&mut bytes.as_slice(), &loader).unwrap();
        assert_eq!(decoded.effect.as_ref().unwrap().name(), "wet");
        assert_eq!(decoded.time, 4.0);

        bytes.clear();
        write_statuses(&mut bytes, &[normal.clone()]).unwrap();
        assert_eq!(
            read_statuses(&mut bytes.as_slice(), &loader).unwrap(),
            vec![normal.clone()]
        );
        let mut statuses = Vec::new();
        read_statuses_into(&mut bytes.as_slice(), &loader, &mut statuses).unwrap();
        assert_eq!(statuses, vec![normal]);
    }

    #[test]
    fn java_length_prefixed_arrays_match_exact_layout() {
        let mut bytes = Vec::new();
        write_bytes_short(&mut bytes, &[0xaa, 0xbb]).unwrap();
        assert_eq!(bytes, vec![0x00, 0x02, 0xaa, 0xbb]);
        assert_eq!(
            read_bytes_short(&mut bytes.as_slice()).unwrap(),
            vec![0xaa, 0xbb]
        );

        bytes.clear();
        write_ints(&mut bytes, &[1, -2]).unwrap();
        assert_eq!(bytes, vec![0x00, 0x02, 0, 0, 0, 1, 0xff, 0xff, 0xff, 0xfe]);
        assert_eq!(read_ints(&mut bytes.as_slice()).unwrap(), vec![1, -2]);

        bytes.clear();
        write_int_seq(&mut bytes, &[3, 4]).unwrap();
        assert_eq!(bytes, vec![0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4]);
        assert_eq!(read_int_seq(&mut bytes.as_slice()).unwrap(), vec![3, 4]);
    }

    #[test]
    fn string_arrays_and_string_data_use_java_prefixes() {
        let mut bytes = Vec::new();
        write_strings(&mut bytes, &[Some("A"), None]).unwrap();
        assert_eq!(bytes, vec![2, 1, 0, 1, b'A', 0]);
        assert_eq!(
            read_strings(&mut bytes.as_slice()).unwrap(),
            vec![Some("A".into()), None]
        );

        bytes.clear();
        write_strings_limited(&mut bytes, &[Some("A"), Some("B"), Some("C")], 2).unwrap();
        assert_eq!(bytes, vec![2, 1, 0, 1, b'A', 1, 0, 1, b'B']);

        bytes.clear();
        write_string_array(&mut bytes, &[vec![Some("x")], vec![None, Some("y")]]).unwrap();
        assert_eq!(
            read_string_array(&mut bytes.as_slice()).unwrap(),
            vec![vec![Some("x".into())], vec![None, Some("y".into())]]
        );

        bytes.clear();
        write_string_data(&mut bytes, Some("A\0中")).unwrap();
        assert_eq!(bytes[0..2], [0, 5]);
        assert_eq!(
            read_string_data(&mut bytes.as_slice()).unwrap(),
            Some("A\0中".into())
        );

        bytes.clear();
        write_string_data(&mut bytes, None).unwrap();
        assert_eq!(bytes, vec![0xff, 0xff]);
        assert_eq!(read_string_data(&mut bytes.as_slice()).unwrap(), None);
    }

    #[test]
    fn nullable_vec_uses_nan_sentinel_like_typeio() {
        let mut bytes = Vec::new();
        write_vec_nullable(&mut bytes, None).unwrap();
        assert_eq!(bytes.len(), 8);
        assert_eq!(read_vec_nullable(&mut bytes.as_slice()).unwrap(), None);

        bytes.clear();
        write_vec_nullable(&mut bytes, Some(Vec2::new(1.25, -2.5))).unwrap();
        assert_eq!(bytes, vec![0x3f, 0xa0, 0, 0, 0xc0, 0x20, 0, 0]);
        assert_eq!(
            read_vec_nullable(&mut bytes.as_slice()).unwrap(),
            Some(Vec2::new(1.25, -2.5))
        );
    }

    #[test]
    fn color_bytebuffer_string_and_vec2_base_follow_typeio_order() {
        let mut bytes = Vec::new();
        write_color(&mut bytes, RgbaColor::new(0x11223344)).unwrap();
        assert_eq!(bytes, vec![0x11, 0x22, 0x33, 0x44]);
        assert_eq!(
            read_color(&mut bytes.as_slice()).unwrap(),
            RgbaColor::new(0x11223344)
        );

        bytes.clear();
        write_bytebuffer_string(&mut bytes, Some("Hi中")).unwrap();
        assert_eq!(&bytes[0..2], &[0, 5]);
        assert_eq!(
            read_bytebuffer_string(&mut bytes.as_slice()).unwrap(),
            Some("Hi中".into())
        );

        bytes.clear();
        write_bytebuffer_string(&mut bytes, None).unwrap();
        assert_eq!(bytes, vec![0xff, 0xff]);
        assert_eq!(read_bytebuffer_string(&mut bytes.as_slice()).unwrap(), None);

        bytes.clear();
        write_vec2(&mut bytes, Vec2::new(-1.5, 3.25)).unwrap();
        let mut base = Vec2::new(99.0, 88.0);
        read_vec2_into(&mut bytes.as_slice(), &mut base).unwrap();
        assert_eq!(base, Vec2::new(-1.5, 3.25));
    }

    #[test]
    fn tagged_array_values_match_low_risk_java_tags() {
        let mut bytes = Vec::new();
        let seq = TypeValue::IntSeq(vec![5, -6]);
        write_object(&mut bytes, &seq).unwrap();
        assert_eq!(bytes, vec![6, 0, 2, 0, 0, 0, 5, 0xff, 0xff, 0xff, 0xfa]);
        assert_eq!(read_object(&mut bytes.as_slice()).unwrap(), seq);

        bytes.clear();
        let points = TypeValue::Point2Array(vec![Point2::new(-1, 2), Point2::new(3, -4)]);
        write_object(&mut bytes, &points).unwrap();
        assert_eq!(bytes[0], 8);
        assert_eq!(bytes[1], 2);
        assert_eq!(read_object(&mut bytes.as_slice()).unwrap(), points);

        bytes.clear();
        let bools = TypeValue::BoolArray(vec![true, false, true]);
        write_object(&mut bytes, &bools).unwrap();
        assert_eq!(bytes, vec![16, 0, 0, 0, 3, 1, 0, 1]);
        assert_eq!(read_object(&mut bytes.as_slice()).unwrap(), bools);

        bytes.clear();
        let vecs = TypeValue::Vec2Array(vec![Vec2::new(1.0, 2.0), Vec2::new(-3.0, 4.5)]);
        write_object(&mut bytes, &vecs).unwrap();
        assert_eq!(bytes[0], 18);
        assert_eq!(&bytes[1..3], &[0, 2]);
        assert_eq!(read_object(&mut bytes.as_slice()).unwrap(), vecs);

        let nested = [22, 0, 0, 0, 1, 16, 0, 0, 0, 1, 1];
        assert!(read_object(&mut nested.as_slice()).is_err());
    }

    #[test]
    fn point2_teamid_vec2_helpers_roundtrip() {
        let point = Point2::new(-12, 34);
        let vec = Vec2::new(1.5, -2.25);
        let team = TeamId(7);

        let mut bytes = Vec::new();
        write_point2(&mut bytes, point).unwrap();
        write_point2_packed(&mut bytes, point).unwrap();
        write_vec2(&mut bytes, vec).unwrap();
        write_team_id(&mut bytes, team).unwrap();

        let mut slice = bytes.as_slice();
        assert_eq!(read_point2(&mut slice).unwrap(), point);
        assert_eq!(read_point2_packed(&mut slice).unwrap(), point);
        assert_eq!(read_vec2(&mut slice).unwrap(), vec);
        assert_eq!(read_team_id(&mut slice).unwrap(), team);
    }

    #[test]
    fn team_effect_sound_and_bytes_match_java_typeio_layout() {
        let mut bytes = Vec::new();

        write_team(&mut bytes, None).unwrap();
        write_team(&mut bytes, Some(TeamId(6))).unwrap();
        assert_eq!(bytes, vec![0, 6]);
        let mut slice = bytes.as_slice();
        assert_eq!(read_team(&mut slice).unwrap(), TeamId(0));
        assert_eq!(read_team(&mut slice).unwrap(), TeamId(6));

        bytes.clear();
        write_effect_id(&mut bytes, 258).unwrap();
        assert_eq!(bytes, vec![0x01, 0x02]);
        assert_eq!(read_effect_id(&mut bytes.as_slice()).unwrap(), 258);
        assert!(write_effect_id(&mut Vec::new(), -1).is_err());

        bytes.clear();
        write_sound_id(&mut bytes, -1).unwrap();
        write_sound_id(&mut bytes, 17).unwrap();
        assert_eq!(bytes, vec![0xff, 0xff, 0x00, 0x11]);
        let mut slice = bytes.as_slice();
        assert_eq!(read_sound_id(&mut slice).unwrap(), -1);
        assert_eq!(read_sound_id(&mut slice).unwrap(), 17);

        bytes.clear();
        write_bytes(&mut bytes, &[0xaa, 0xbb, 0xcc]).unwrap();
        assert_eq!(bytes, vec![0, 3, 0xaa, 0xbb, 0xcc]);
        assert_eq!(
            read_bytes(&mut bytes.as_slice()).unwrap(),
            vec![0xaa, 0xbb, 0xcc]
        );
    }

    #[test]
    fn kick_and_marker_control_match_java_ordinal_bytes() {
        let mut bytes = Vec::new();
        write_kick(&mut bytes, KickReason::Gameover).unwrap();
        assert_eq!(bytes, vec![0x04]);
        assert_eq!(
            read_kick(&mut bytes.as_slice()).unwrap(),
            KickReason::Gameover
        );

        bytes.clear();
        write_kick(&mut bytes, KickReason::ServerRestarting).unwrap();
        assert_eq!(bytes, vec![0x0f]);
        assert_eq!(
            read_kick(&mut bytes.as_slice()).unwrap(),
            KickReason::ServerRestarting
        );
        assert!(read_kick(&mut [0x10].as_slice()).is_err());
        assert!(read_kick(&mut [0xff].as_slice()).is_err());

        bytes.clear();
        write_marker_control(&mut bytes, LMarkerControl::Remove).unwrap();
        assert_eq!(bytes, vec![0x00]);
        assert_eq!(
            read_marker_control(&mut bytes.as_slice()).unwrap(),
            LMarkerControl::Remove
        );

        bytes.clear();
        write_marker_control(&mut bytes, LMarkerControl::TextureSize).unwrap();
        assert_eq!(bytes, vec![0x15]);
        assert_eq!(
            read_marker_control(&mut bytes.as_slice()).unwrap(),
            LMarkerControl::TextureSize
        );

        bytes.clear();
        write_marker_control(&mut bytes, LMarkerControl::Colori).unwrap();
        assert_eq!(bytes, vec![0x18]);
        assert_eq!(
            read_marker_control(&mut bytes.as_slice()).unwrap(),
            LMarkerControl::Colori
        );
        assert!(read_marker_control(&mut [0x19].as_slice()).is_err());
        assert!(read_marker_control(&mut [0xff].as_slice()).is_err());
    }

    #[test]
    fn admin_action_matches_java_ordinal_bytes() {
        assert_eq!(AdminAction::ALL.len(), 5);
        assert_eq!(AdminAction::Kick.ordinal(), 0);
        assert_eq!(AdminAction::SwitchTeam.ordinal(), 4);
        assert_eq!(AdminAction::SwitchTeam.wire_name(), "switchTeam");

        let mut bytes = Vec::new();
        write_action(&mut bytes, AdminAction::Kick).unwrap();
        assert_eq!(bytes, vec![0x00]);
        assert_eq!(
            read_action(&mut bytes.as_slice()).unwrap(),
            AdminAction::Kick
        );

        bytes.clear();
        write_action(&mut bytes, AdminAction::SwitchTeam).unwrap();
        assert_eq!(bytes, vec![0x04]);
        assert_eq!(
            read_action(&mut bytes.as_slice()).unwrap(),
            AdminAction::SwitchTeam
        );
        assert!(read_action(&mut [0x05].as_slice()).is_err());
        assert!(read_action(&mut [0xff].as_slice()).is_err());
    }

    #[test]
    fn rules_and_objective_json_use_java_int_length_utf8_layout() {
        let mut bytes = Vec::new();
        let rules = "{\"wave\":true,\"name\":\"中\"}";
        write_rules_json(&mut bytes, rules).unwrap();
        assert_eq!(&bytes[0..4], &(rules.len() as i32).to_be_bytes());
        assert_eq!(&bytes[4..], rules.as_bytes());
        assert_eq!(read_rules_json(&mut bytes.as_slice()).unwrap(), rules);

        bytes.clear();
        let objectives = "{\"all\":[]}";
        write_objectives_json(&mut bytes, objectives).unwrap();
        assert_eq!(&bytes[0..4], &(objectives.len() as i32).to_be_bytes());
        assert_eq!(
            read_objectives_json(&mut bytes.as_slice()).unwrap(),
            objectives
        );

        bytes.clear();
        let marker = "{\"type\":\"Point\",\"x\":4}";
        write_objective_marker_json(&mut bytes, marker).unwrap();
        assert_eq!(&bytes[0..4], &(marker.len() as i32).to_be_bytes());
        assert_eq!(
            read_objective_marker_json(&mut bytes.as_slice()).unwrap(),
            marker
        );
    }

    #[test]
    fn json_payload_readers_enforce_java_length_limits() {
        assert!(read_rules_json(&mut [0xff, 0xff, 0xff, 0xff].as_slice()).is_err());

        let mut rules = Vec::new();
        rules.extend_from_slice(&((MAX_RULES_BYTES as i32 + 1).to_be_bytes()));
        assert!(read_rules_json(&mut rules.as_slice()).is_err());

        let mut objectives = Vec::new();
        objectives.extend_from_slice(&(MAX_OBJECTIVES_BYTES as i32).to_be_bytes());
        assert!(read_objectives_json(&mut objectives.as_slice()).is_err());

        let mut marker = Vec::new();
        marker.extend_from_slice(&((MAX_BYTE_ARRAY_SIZE as i32 + 1).to_be_bytes()));
        assert!(read_objective_marker_json(&mut marker.as_slice()).is_err());

        let mut invalid_utf8 = Vec::new();
        invalid_utf8.extend_from_slice(&1i32.to_be_bytes());
        invalid_utf8.push(0xff);
        assert!(read_rules_json(&mut invalid_utf8.as_slice()).is_err());
    }

    #[test]
    fn trace_info_matches_java_field_order_and_caps_history() {
        let trace = TraceInfo::new(
            Some("127.0.0.1".into()),
            Some("uuid".into()),
            None,
            true,
            false,
            7,
            2,
            (0..13).map(|index| Some(format!("ip{index}"))).collect(),
            vec![Some("alpha".into()), None, Some("gamma".into())],
        );

        let mut bytes = Vec::new();
        write_trace_info(&mut bytes, &trace).unwrap();

        let mut expected = Vec::new();
        write_string(&mut expected, Some("127.0.0.1")).unwrap();
        write_string(&mut expected, Some("uuid")).unwrap();
        write_string(&mut expected, None).unwrap();
        expected.extend_from_slice(&[1, 0]);
        expected.extend_from_slice(&7i32.to_be_bytes());
        expected.extend_from_slice(&2i32.to_be_bytes());
        expected.push(12);
        for index in 0..12 {
            write_string(&mut expected, Some(&format!("ip{index}"))).unwrap();
        }
        expected.push(3);
        write_string(&mut expected, Some("alpha")).unwrap();
        write_string(&mut expected, None).unwrap();
        write_string(&mut expected, Some("gamma")).unwrap();
        assert_eq!(bytes, expected);

        let decoded = read_trace_info(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded.ip.as_deref(), Some("127.0.0.1"));
        assert_eq!(decoded.uuid.as_deref(), Some("uuid"));
        assert_eq!(decoded.locale, None);
        assert!(decoded.modded);
        assert!(!decoded.mobile);
        assert_eq!(decoded.times_joined, 7);
        assert_eq!(decoded.times_kicked, 2);
        assert_eq!(decoded.ips.len(), TraceInfo::MAX_HISTORY_LEN);
        assert_eq!(decoded.ips[11].as_deref(), Some("ip11"));
        assert_eq!(
            decoded.names,
            vec![Some("alpha".into()), None, Some("gamma".into())]
        );
    }
}
