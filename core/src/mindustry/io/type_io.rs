use std::io::{self, Read, Write};

use crate::mindustry::core::content_loader::{ContentLoader, ContentRecord};
use crate::mindustry::ctype::{ContentId, ContentType};
use crate::mindustry::logic::LMarkerControl;
use crate::mindustry::net::{AdminAction, KickReason, TraceInfo};
use crate::mindustry::world::{point2_pack, point2_x, point2_y};

pub const MAX_ARRAY_SIZE: usize = 1000;
pub const MAX_BYTE_ARRAY_SIZE: usize = 40_000;

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

fn read_object_inner<R: Read>(read: &mut R, allow_arrays: bool) -> io::Result<TypeValue> {
    let tag = read_u8(read)?;
    match tag {
        0 => Ok(TypeValue::Null),
        1 => Ok(TypeValue::Int(read_i32(read)?)),
        2 => Ok(TypeValue::Long(read_i64(read)?)),
        3 => Ok(TypeValue::Float(f32::from_bits(read_u32(read)?))),
        4 => Ok(match read_string(read)? {
            Some(value) => TypeValue::String(value),
            None => TypeValue::Null,
        }),
        5 => Ok(TypeValue::Content(read_content_ref(read)?)),
        10 => Ok(TypeValue::Bool(read_u8(read)? != 0)),
        11 => Ok(TypeValue::Double(f64::from_bits(read_u64(read)?))),
        7 => Ok(TypeValue::Point2(Point2::new(
            read_i32(read)?,
            read_i32(read)?,
        ))),
        9 => Ok(TypeValue::TechNode(read_content_ref(read)?)),
        6 => {
            ensure_arrays_allowed(allow_arrays)?;
            let len = read_i16(read)?;
            if len < 0 || len as usize > MAX_ARRAY_SIZE {
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
            if len < 0 || len as usize > MAX_ARRAY_SIZE {
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
            if len < 0 || len as usize > MAX_ARRAY_SIZE {
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
            if len < 0 || len as usize > MAX_ARRAY_SIZE {
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
            if len < 0 || len as usize > MAX_ARRAY_SIZE {
                return Err(invalid_data("invalid object array length"));
            }
            let mut values = Vec::with_capacity(len as usize);
            for _ in 0..len {
                values.push(read_object_inner(read, false)?);
            }
            Ok(TypeValue::ObjectArray(values))
        }
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
