use std::io::{self, Read, Write};

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
pub struct TeamId(pub u8);

#[derive(Debug, Clone, PartialEq)]
pub enum TypeValue {
    Null,
    Int(i32),
    Long(i64),
    Float(f32),
    String(String),
    Bool(bool),
    Double(f64),
    Point2(Point2),
    Vec2(Vec2),
    Team(u8),
    IntArray(Vec<i32>),
    ByteArray(Vec<u8>),
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
        TypeValue::Team(value) => {
            write.write_all(&[20])?;
            write.write_all(&[*value])
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
        10 => Ok(TypeValue::Bool(read_u8(read)? != 0)),
        11 => Ok(TypeValue::Double(f64::from_bits(read_u64(read)?))),
        7 => Ok(TypeValue::Point2(Point2::new(
            read_i32(read)?,
            read_i32(read)?,
        ))),
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

pub fn write_team_id<W: Write>(write: &mut W, value: TeamId) -> io::Result<()> {
    write.write_all(&[value.0])
}

pub fn read_team_id<R: Read>(read: &mut R) -> io::Result<TeamId> {
    Ok(TeamId(read_u8(read)?))
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
}
