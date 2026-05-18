use std::io::{self, Read, Write};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

use crate::mindustry::io::{read_java_utf, write_java_utf};

pub const MESSAGE_MAX_TEXT_LENGTH: usize = 300;
pub const MESSAGE_MAX_NEWLINES: usize = 24;
pub const LOGIC_MAX_BYTE_LEN: usize = 1024 * 100;
pub const LOGIC_MAX_LINKS: usize = 6000;
pub const LOGIC_MAX_NAME_LENGTH: usize = 32;
pub const DISPLAY_DRAW_TYPE: i32 = 30;
pub const DISPLAY_SCALE_STEP: f32 = 0.05;
pub const TILE_DISPLAY_TILE_SIZE: i32 = 32;
pub const TILE_DISPLAY_FRAME_SIZE: i32 = 6;
pub const TILE_DISPLAY_MAX_DIMENSIONS: i32 = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicLink {
    pub x: i16,
    pub y: i16,
    pub name: String,
    pub valid: bool,
}

impl LogicLink {
    pub fn new(x: i16, y: i16, name: impl Into<String>, valid: bool) -> Self {
        Self {
            x,
            y,
            name: name.into(),
            valid,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicConfig {
    pub version: u8,
    pub code: Vec<u8>,
    pub links: Vec<LogicLink>,
}

impl LogicConfig {
    pub fn from_code(code: impl AsRef<[u8]>, links: Vec<LogicLink>) -> Self {
        Self {
            version: 1,
            code: code.as_ref().to_vec(),
            links,
        }
    }
}

pub fn sanitize_message_config(input: &str) -> Option<String> {
    sanitize_message_config_with_limits(input, MESSAGE_MAX_TEXT_LENGTH, MESSAGE_MAX_NEWLINES)
}

pub fn sanitize_message_config_with_limits(
    input: &str,
    max_text_length: usize,
    max_newlines: usize,
) -> Option<String> {
    if input.encode_utf16().count() > max_text_length {
        return None;
    }

    let text = trim_java_ascii(input);
    let mut out = String::with_capacity(text.len());
    let mut newlines = 0usize;
    for ch in text.chars() {
        if ch == '\n' {
            if newlines <= max_newlines {
                out.push('\n');
            }
            newlines += 1;
        } else {
            out.push(ch);
        }
    }
    Some(out)
}

fn trim_java_ascii(input: &str) -> &str {
    input.trim_matches(|ch| ch <= '\u{20}')
}

pub fn write_switch_enabled<W: Write>(write: &mut W, enabled: bool) -> io::Result<()> {
    write.write_all(&[u8::from(enabled)])
}

pub fn read_switch_enabled<R: Read>(read: &mut R, revision: u8, current: bool) -> io::Result<bool> {
    if revision == 1 {
        Ok(read_u8(read)? != 0)
    } else {
        Ok(current)
    }
}

pub fn write_logic_config<W: Write>(write: W, config: &LogicConfig) -> io::Result<()> {
    if config.code.len() > LOGIC_MAX_BYTE_LEN {
        return Err(invalid_input(
            "logic code exceeds upstream maximum byte length",
        ));
    }
    let mut encoder = ZlibEncoder::new(write, Compression::default());
    encoder.write_all(&[config.version])?;
    write_i32(&mut encoder, config.code.len() as i32)?;
    encoder.write_all(&config.code)?;
    write_i32(&mut encoder, config.links.len() as i32)?;
    for link in &config.links {
        write_java_utf(&mut encoder, &link.name)?;
        write_i16(&mut encoder, link.x)?;
        write_i16(&mut encoder, link.y)?;
    }
    encoder.finish()?;
    Ok(())
}

pub fn read_logic_config<R: Read>(
    read: R,
    relative_origin: Option<(i16, i16)>,
) -> io::Result<LogicConfig> {
    let mut decoder = ZlibDecoder::new(read);
    let version = read_u8(&mut decoder)?;
    let byte_len = read_i32(&mut decoder)?;
    if byte_len < 0 || byte_len as usize > LOGIC_MAX_BYTE_LEN {
        return Err(invalid_data("malformed logic code byte length"));
    }

    let mut code = vec![0; byte_len as usize];
    decoder.read_exact(&mut code)?;

    let total = read_i32(&mut decoder)?.max(0) as usize;
    let total = total.min(LOGIC_MAX_LINKS);
    let mut links = Vec::with_capacity(total);

    if version == 0 {
        for _ in 0..total {
            let _ = read_i32(&mut decoder)?;
        }
    } else {
        for _ in 0..total {
            let name = read_java_utf(&mut decoder)?;
            let mut x = read_i16(&mut decoder)?;
            let mut y = read_i16(&mut decoder)?;
            if let Some((origin_x, origin_y)) = relative_origin {
                x = x.wrapping_add(origin_x);
                y = y.wrapping_add(origin_y);
            }
            links.push(LogicLink::new(x, y, name, false));
        }
    }

    Ok(LogicConfig {
        version,
        code,
        links,
    })
}

pub fn transform_logic_config_points(
    config: &LogicConfig,
    mut transform: impl FnMut(i16, i16) -> (i16, i16),
) -> LogicConfig {
    let mut transformed = config.clone();
    for link in &mut transformed.links {
        let (x, y) = transform(link.x, link.y);
        link.x = x;
        link.y = y;
        link.valid = true;
    }
    transformed
}

pub fn memory_read(memory: &[f64], address: i32) -> f64 {
    if address < 0 {
        f64::NAN
    } else {
        memory.get(address as usize).copied().unwrap_or(f64::NAN)
    }
}

pub fn memory_write(memory: &mut [f64], address: i32, value: f64) {
    if address >= 0 {
        if let Some(slot) = memory.get_mut(address as usize) {
            *slot = value;
        }
    }
}

pub fn display_dimension(tiles: i32, frame_size: i32) -> i32 {
    tiles * TILE_DISPLAY_TILE_SIZE - frame_size * 2
}

pub fn canvas_bits_per_pixel(palette_len: usize) -> i32 {
    let mut power = 1usize;
    let mut bits = 0i32;
    while power < palette_len {
        power <<= 1;
        bits += 1;
    }
    bits
}

pub fn canvas_data_len(canvas_size: i32, bits_per_pixel: i32) -> usize {
    ((canvas_size * canvas_size * bits_per_pixel) as f32 / 8.0).ceil() as usize
}

pub fn canvas_get_index(data: &[u8], bit_offset: usize, bits_per_pixel: i32) -> u32 {
    let mut result = 0u32;
    for i in 0..bits_per_pixel as usize {
        let word = (i + bit_offset) >> 3;
        let bit = (i + bit_offset) & 7;
        if data.get(word).copied().unwrap_or(0) & (1 << bit) != 0 {
            result |= 1 << i;
        }
    }
    result
}

pub fn canvas_set_index(data: &mut [u8], bit_offset: usize, bits_per_pixel: i32, value: u32) {
    for i in 0..bits_per_pixel as usize {
        let word = (i + bit_offset) >> 3;
        let bit = (i + bit_offset) & 7;
        if let Some(byte) = data.get_mut(word) {
            if (value >> i) & 1 == 0 {
                *byte &= !(1 << bit);
            } else {
                *byte |= 1 << bit;
            }
        }
    }
}

pub fn canvas_get_pixel(data: &[u8], canvas_size: i32, bits_per_pixel: i32, pos: i32) -> f64 {
    if pos < 0 || pos >= canvas_size * canvas_size {
        return f64::NAN;
    }
    canvas_get_index(data, pos as usize * bits_per_pixel as usize, bits_per_pixel) as f64
}

pub fn canvas_set_pixel(
    data: &mut [u8],
    canvas_size: i32,
    palette_len: usize,
    bits_per_pixel: i32,
    pos: i32,
    index: i32,
) -> bool {
    if pos < 0 || pos >= canvas_size * canvas_size || index < 0 || index as usize >= palette_len {
        return false;
    }
    canvas_set_index(
        data,
        pos as usize * bits_per_pixel as usize,
        bits_per_pixel,
        index as u32,
    );
    true
}

fn read_u8<R: Read>(read: &mut R) -> io::Result<u8> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_i16<R: Read>(read: &mut R) -> io::Result<i16> {
    let mut buf = [0; 2];
    read.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

fn write_i16<W: Write>(write: &mut W, value: i16) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_i32<R: Read>(read: &mut R) -> io::Result<i32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

fn write_i32<W: Write>(write: &mut W, value: i32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
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
    fn message_config_trims_and_keeps_java_newline_limit() {
        let input = format!("  a{}b  ", "\n".repeat(MESSAGE_MAX_NEWLINES + 3));
        let out = sanitize_message_config(&input).unwrap();
        assert!(out.starts_with('a'));
        assert!(out.ends_with('b'));
        assert_eq!(out.matches('\n').count(), MESSAGE_MAX_NEWLINES + 1);
        assert!(sanitize_message_config(&"x".repeat(MESSAGE_MAX_TEXT_LENGTH + 1)).is_none());
    }

    #[test]
    fn switch_revision_one_reads_boolean_like_java_build() {
        let mut bytes = Vec::new();
        write_switch_enabled(&mut bytes, true).unwrap();
        assert_eq!(
            read_switch_enabled(&mut bytes.as_slice(), 1, false).unwrap(),
            true
        );
        assert_eq!(
            read_switch_enabled(&mut [].as_slice(), 0, true).unwrap(),
            true
        );
    }

    #[test]
    fn logic_config_roundtrips_deflater_stream_and_relative_links() {
        let config = LogicConfig::from_code(
            b"print \"hi\"",
            vec![LogicLink::new(2, -3, "conveyor1", true)],
        );
        let mut bytes = Vec::new();
        write_logic_config(&mut bytes, &config).unwrap();

        let decoded = read_logic_config(bytes.as_slice(), Some((10, 20))).unwrap();
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.code, b"print \"hi\"");
        assert_eq!(
            decoded.links,
            vec![LogicLink {
                x: 12,
                y: 17,
                name: "conveyor1".into(),
                valid: false
            }]
        );

        let transformed = transform_logic_config_points(&decoded, |x, y| (x - 1, y + 1));
        assert_eq!(transformed.links[0].x, 11);
        assert_eq!(transformed.links[0].y, 18);
        assert!(transformed.links[0].valid);
    }

    #[test]
    fn memory_read_write_returns_nan_out_of_bounds() {
        let mut memory = vec![0.0; 2];
        memory_write(&mut memory, 1, 42.5);
        memory_write(&mut memory, 9, 1.0);
        assert_eq!(memory_read(&memory, 1), 42.5);
        assert!(memory_read(&memory, -1).is_nan());
        assert!(memory_read(&memory, 2).is_nan());
    }

    #[test]
    fn tile_display_dimensions_account_for_frame() {
        assert_eq!(
            display_dimension(1, TILE_DISPLAY_FRAME_SIZE),
            TILE_DISPLAY_TILE_SIZE - TILE_DISPLAY_FRAME_SIZE * 2
        );
        assert_eq!(display_dimension(TILE_DISPLAY_MAX_DIMENSIONS, 6), 500);
    }

    #[test]
    fn canvas_bit_packing_matches_java_little_bit_order() {
        let bpp = canvas_bits_per_pixel(16);
        assert_eq!(bpp, 4);
        assert_eq!(canvas_data_len(24, bpp), 288);

        let mut data = vec![0; canvas_data_len(4, bpp)];
        assert!(canvas_set_pixel(&mut data, 4, 16, bpp, 0, 7));
        assert!(canvas_set_pixel(&mut data, 4, 16, bpp, 1, 12));
        assert!(!canvas_set_pixel(&mut data, 4, 16, bpp, 2, 16));
        assert_eq!(canvas_get_pixel(&data, 4, bpp, 0), 7.0);
        assert_eq!(canvas_get_pixel(&data, 4, bpp, 1), 12.0);
        assert!(canvas_get_pixel(&data, 4, bpp, 16).is_nan());
    }
}
