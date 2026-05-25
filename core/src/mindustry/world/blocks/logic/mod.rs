use std::io::{self, Read, Write};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

use crate::mindustry::io::type_io::read_object_safe;
use crate::mindustry::io::{
    read_java_utf, read_string, write_java_utf, write_object, write_string, TypeValue,
};

pub const MESSAGE_MAX_TEXT_LENGTH: usize = 300;
pub const MESSAGE_MAX_NEWLINES: usize = 24;
pub const LOGIC_MAX_BYTE_LEN: usize = 1024 * 100;
pub const LOGIC_MAX_LINKS: usize = 6000;
pub const LOGIC_MAX_NAME_LENGTH: usize = 32;
pub const LOGIC_MAX_COMPRESSED_LEN: usize = 16_000;
/// `arc.math.Mat` is a 3x3 matrix (`val.length == 9`) in upstream Arc.
pub const LOGIC_DISPLAY_TRANSFORM_LEN: usize = 9;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBlockState {
    pub message: String,
}

impl MessageBlockState {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub fn write_message_state<W: Write>(write: &mut W, state: &MessageBlockState) -> io::Result<()> {
    write_java_utf(write, &state.message)
}

pub fn read_message_state<R: Read>(read: &mut R) -> io::Result<MessageBlockState> {
    read_java_utf(read).map(MessageBlockState::new)
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LogicDisplayState {
    pub transform: Option<[f32; LOGIC_DISPLAY_TRANSFORM_LEN]>,
}

impl LogicDisplayState {
    pub fn with_transform(transform: [f32; LOGIC_DISPLAY_TRANSFORM_LEN]) -> Self {
        Self {
            transform: Some(transform),
        }
    }
}

pub fn write_logic_display_state<W: Write>(
    write: &mut W,
    state: &LogicDisplayState,
) -> io::Result<()> {
    if let Some(transform) = state.transform.as_ref() {
        write.write_all(&[1])?;
        for value in transform {
            write_f32(write, *value)?;
        }
    } else {
        write.write_all(&[0])?;
    }
    Ok(())
}

pub fn read_logic_display_state<R: Read>(
    read: &mut R,
    revision: u8,
) -> io::Result<LogicDisplayState> {
    if revision < 1 {
        return Ok(LogicDisplayState::default());
    }

    if read_u8(read)? == 0 {
        return Ok(LogicDisplayState::default());
    }

    let mut transform = [0.0; LOGIC_DISPLAY_TRANSFORM_LEN];
    for value in &mut transform {
        *value = read_f32(read)?;
    }
    Ok(LogicDisplayState::with_transform(transform))
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBlockState {
    pub memory: Vec<f64>,
}

impl MemoryBlockState {
    pub fn new(capacity: usize) -> Self {
        Self {
            memory: vec![0.0; capacity],
        }
    }

    pub fn from_values(values: impl Into<Vec<f64>>) -> Self {
        Self {
            memory: values.into(),
        }
    }
}

pub fn write_memory_state<W: Write>(write: &mut W, state: &MemoryBlockState) -> io::Result<()> {
    write_i32(write, state.memory.len() as i32)?;
    for value in &state.memory {
        write_f64(write, *value)?;
    }
    Ok(())
}

pub fn read_memory_state<R: Read>(read: &mut R, capacity: usize) -> io::Result<MemoryBlockState> {
    let mut state = MemoryBlockState::new(capacity);
    let amount = read_i32(read)?;
    if amount <= 0 {
        return Ok(state);
    }

    for i in 0..amount as usize {
        let value = read_f64(read)?;
        if let Some(slot) = state.memory.get_mut(i) {
            *slot = value;
        }
    }
    Ok(state)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasBlockState {
    pub data: Vec<u8>,
}

impl CanvasBlockState {
    pub fn new(data_len: usize) -> Self {
        Self {
            data: vec![0; data_len],
        }
    }

    pub fn from_data(data: impl Into<Vec<u8>>) -> Self {
        Self { data: data.into() }
    }
}

pub fn write_canvas_state<W: Write>(write: &mut W, state: &CanvasBlockState) -> io::Result<()> {
    write_i32(write, state.data.len() as i32)?;
    write.write_all(&state.data)
}

pub fn read_canvas_state<R: Read>(
    read: &mut R,
    expected_len: usize,
) -> io::Result<CanvasBlockState> {
    let len = read_i32(read)?;
    if len < 0 {
        return Err(invalid_data("negative canvas data length"));
    }

    let len = len as usize;
    if len == expected_len {
        let mut state = CanvasBlockState::new(expected_len);
        read.read_exact(&mut state.data)?;
        Ok(state)
    } else {
        skip_bytes(read, len)?;
        Ok(CanvasBlockState::new(expected_len))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicProcessorVariableState {
    pub name: String,
    pub value: TypeValue,
}

impl LogicProcessorVariableState {
    pub fn new(name: impl Into<String>, value: TypeValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicProcessorWaitState {
    pub index: u16,
    pub value: f32,
}

impl LogicProcessorWaitState {
    pub fn new(index: u16, value: f32) -> Self {
        Self { index, value }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LogicProcessorState {
    pub compressed_code: Option<Vec<u8>>,
    pub config: Option<LogicConfig>,
    pub legacy_code: Option<String>,
    pub legacy_link_positions: Vec<i32>,
    pub variables: Vec<LogicProcessorVariableState>,
    pub legacy_memory_slots: i32,
    pub ipt: Option<i16>,
    pub tag: Option<String>,
    pub icon_tag: u16,
    pub waits: Vec<LogicProcessorWaitState>,
    pub accumulator: f32,
}

impl LogicProcessorState {
    pub fn from_config(config: LogicConfig) -> io::Result<Self> {
        let mut compressed = Vec::new();
        write_logic_config(&mut compressed, &config)?;
        Ok(Self {
            compressed_code: Some(compressed),
            config: Some(config),
            ..Self::default()
        })
    }
}

pub fn write_logic_processor_state<W: Write>(
    write: &mut W,
    state: &LogicProcessorState,
    revision: u8,
    privileged: bool,
) -> io::Result<()> {
    if revision >= 1 {
        let compressed = if let Some(compressed) = state.compressed_code.as_ref() {
            compressed.clone()
        } else if let Some(config) = state.config.as_ref() {
            let mut compressed = Vec::new();
            write_logic_config(&mut compressed, config)?;
            compressed
        } else {
            Vec::new()
        };
        if compressed.len() > i32::MAX as usize {
            return Err(invalid_input("processor compressed code too large"));
        }
        write_i32(write, compressed.len() as i32)?;
        write.write_all(&compressed)?;
    } else {
        write_java_utf(write, state.legacy_code.as_deref().unwrap_or(""))?;
        if state.legacy_link_positions.len() > i16::MAX as usize {
            return Err(invalid_input("legacy processor link count too large"));
        }
        write_i16(write, state.legacy_link_positions.len() as i16)?;
        for pos in &state.legacy_link_positions {
            write_i32(write, *pos)?;
        }
    }

    if state.variables.len() > i32::MAX as usize {
        return Err(invalid_input("processor variable count too large"));
    }
    write_i32(write, state.variables.len() as i32)?;
    for variable in &state.variables {
        write_java_utf(write, &variable.name)?;
        write_object(write, &variable.value)?;
    }

    write_i32(write, state.legacy_memory_slots)?;
    if state.legacy_memory_slots > 0 {
        for _ in 0..state.legacy_memory_slots {
            write_f64(write, 0.0)?;
        }
    }

    if privileged && revision >= 2 {
        write_i16(write, state.ipt.unwrap_or(1))?;
    }

    if revision >= 3 {
        write_string(write, state.tag.as_deref())?;
        write_u16(write, state.icon_tag)?;
    }

    if revision >= 4 {
        if state.waits.len() > u16::MAX as usize {
            return Err(invalid_input("processor wait count too large"));
        }
        write_u16(write, state.waits.len() as u16)?;
        for wait in &state.waits {
            write_u16(write, wait.index)?;
            write_f32(write, wait.value)?;
        }
        write_f32(write, state.accumulator)?;
    }

    Ok(())
}

pub fn read_logic_processor_state<R: Read>(
    read: &mut R,
    revision: u8,
    privileged: bool,
    max_instructions_per_tick: i16,
) -> io::Result<LogicProcessorState> {
    let mut state = LogicProcessorState::default();

    if revision >= 1 {
        let compressed_len = read_i32(read)?;
        if compressed_len < 0 {
            return Err(invalid_data("negative processor compressed length"));
        }
        let mut compressed = vec![0; compressed_len as usize];
        read.read_exact(&mut compressed)?;
        state.config = read_logic_config(compressed.as_slice(), None).ok();
        state.compressed_code = Some(compressed);
    } else {
        state.legacy_code = Some(read_java_utf(read)?);
        let total = read_i16(read)?;
        if total > 0 {
            state.legacy_link_positions.reserve(total as usize);
            for _ in 0..total {
                state.legacy_link_positions.push(read_i32(read)?);
            }
        }
    }

    let var_count = read_i32(read)?;
    if var_count < 0 {
        return Err(invalid_data("negative processor variable count"));
    }
    state.variables.reserve(var_count as usize);
    for _ in 0..var_count {
        let name = read_java_utf(read)?;
        let value = read_object_safe(read)?;
        state
            .variables
            .push(LogicProcessorVariableState::new(name, value));
    }

    state.legacy_memory_slots = read_i32(read)?;
    if state.legacy_memory_slots > 0 {
        skip_bytes(read, state.legacy_memory_slots as usize * 8)?;
    }

    if privileged && revision >= 2 {
        let ipt = read_i16(read)?;
        state.ipt = Some(ipt.clamp(1, max_instructions_per_tick.max(1)));
    }

    if revision >= 3 {
        state.tag = read_string(read)?;
        state.icon_tag = read_u16(read)?;
    }

    if revision >= 4 {
        let waits = read_u16(read)? as usize;
        state.waits.reserve(waits);
        for _ in 0..waits {
            let index = read_u16(read)?;
            let value = read_f32(read)?;
            state.waits.push(LogicProcessorWaitState::new(index, value));
        }
        state.accumulator = read_f32(read)?;
    }

    Ok(state)
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

fn read_u16<R: Read>(read: &mut R) -> io::Result<u16> {
    let mut buf = [0; 2];
    read.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn write_u16<W: Write>(write: &mut W, value: u16) -> io::Result<()> {
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

fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_f64<R: Read>(read: &mut R) -> io::Result<f64> {
    let mut buf = [0; 8];
    read.read_exact(&mut buf)?;
    Ok(f64::from_be_bytes(buf))
}

fn write_f64<W: Write>(write: &mut W, value: f64) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn skip_bytes<R: Read>(read: &mut R, mut len: usize) -> io::Result<()> {
    let mut buf = [0; 1024];
    while len > 0 {
        let take = len.min(buf.len());
        read.read_exact(&mut buf[..take])?;
        len -= take;
    }
    Ok(())
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
    fn message_state_roundtrips_java_utf_payload() {
        let state = MessageBlockState::new("alpha\nbeta");
        let mut bytes = Vec::new();
        write_message_state(&mut bytes, &state).unwrap();
        assert_eq!(read_message_state(&mut bytes.as_slice()).unwrap(), state);
    }

    #[test]
    fn logic_display_state_roundtrips_java_mat_payload() {
        let state =
            LogicDisplayState::with_transform([1.0, 0.0, 2.5, 0.0, 1.0, -3.25, 0.0, 0.0, 1.0]);
        let mut bytes = Vec::new();
        write_logic_display_state(&mut bytes, &state).unwrap();
        assert_eq!(bytes.len(), 1 + LOGIC_DISPLAY_TRANSFORM_LEN * 4);
        assert_eq!(
            read_logic_display_state(&mut bytes.as_slice(), 1).unwrap(),
            state
        );

        let mut empty = Vec::new();
        write_logic_display_state(&mut empty, &LogicDisplayState::default()).unwrap();
        assert_eq!(empty, vec![0]);
        assert_eq!(
            read_logic_display_state(&mut empty.as_slice(), 1).unwrap(),
            LogicDisplayState::default()
        );
    }

    #[test]
    fn logic_display_state_revision_zero_does_not_consume_payload() {
        let mut legacy = [0xaa, 0xbb].as_slice();
        assert_eq!(
            read_logic_display_state(&mut legacy, 0).unwrap(),
            LogicDisplayState::default()
        );
        assert_eq!(legacy, [0xaa, 0xbb].as_slice());
    }

    #[test]
    fn memory_state_roundtrips_java_double_array_payload() {
        let state = MemoryBlockState::from_values(vec![1.0, -2.5, 42.25]);
        let mut bytes = Vec::new();
        write_memory_state(&mut bytes, &state).unwrap();
        assert_eq!(bytes.len(), 4 + state.memory.len() * 8);
        assert_eq!(read_memory_state(&mut bytes.as_slice(), 3).unwrap(), state);
    }

    #[test]
    fn memory_state_read_consumes_extra_and_keeps_default_tail_like_java() {
        let mut bytes = Vec::new();
        write_memory_state(
            &mut bytes,
            &MemoryBlockState::from_values(vec![7.0, 8.0, 9.0]),
        )
        .unwrap();
        let mut read = bytes.as_slice();
        assert_eq!(
            read_memory_state(&mut read, 2).unwrap(),
            MemoryBlockState::from_values(vec![7.0, 8.0])
        );
        assert!(read.is_empty());

        let mut short = Vec::new();
        write_memory_state(&mut short, &MemoryBlockState::from_values(vec![4.5])).unwrap();
        assert_eq!(
            read_memory_state(&mut short.as_slice(), 3).unwrap(),
            MemoryBlockState::from_values(vec![4.5, 0.0, 0.0])
        );
    }

    #[test]
    fn canvas_state_roundtrips_java_length_prefixed_bytes() {
        let state = CanvasBlockState::from_data(vec![0x12, 0x34, 0x56]);
        let mut bytes = Vec::new();
        write_canvas_state(&mut bytes, &state).unwrap();
        assert_eq!(bytes.len(), 4 + state.data.len());
        assert_eq!(read_canvas_state(&mut bytes.as_slice(), 3).unwrap(), state);
    }

    #[test]
    fn canvas_state_mismatched_length_is_consumed_and_keeps_default_data_like_java() {
        let mut bytes = Vec::new();
        write_canvas_state(&mut bytes, &CanvasBlockState::from_data(vec![1, 2, 3])).unwrap();
        let mut read = bytes.as_slice();
        assert_eq!(
            read_canvas_state(&mut read, 2).unwrap(),
            CanvasBlockState::new(2)
        );
        assert!(read.is_empty());
    }

    #[test]
    fn logic_processor_state_reads_current_revision_metadata_without_unboxing() {
        let config =
            LogicConfig::from_code(b"print \"hi\"", vec![LogicLink::new(2, -3, "cell1", false)]);
        let mut state = LogicProcessorState::from_config(config.clone()).unwrap();
        state.variables = vec![
            LogicProcessorVariableState::new("@counter", TypeValue::Double(12.5)),
            LogicProcessorVariableState::new("@unit", TypeValue::Unit(77)),
        ];
        state.ipt = Some(25);
        state.tag = Some("core-loop".into());
        state.icon_tag = 'A' as u16;
        state.waits = vec![LogicProcessorWaitState::new(3, 0.75)];
        state.accumulator = 0.5;

        let mut bytes = Vec::new();
        write_logic_processor_state(&mut bytes, &state, 4, true).unwrap();
        let decoded = read_logic_processor_state(&mut bytes.as_slice(), 4, true, 40).unwrap();
        assert_eq!(decoded, state);
        assert_eq!(decoded.config, Some(config));
    }

    #[test]
    fn logic_processor_state_revision_gates_and_skips_legacy_memory() {
        let state = LogicProcessorState {
            legacy_memory_slots: 2,
            ..LogicProcessorState::default()
        };
        let mut bytes = Vec::new();
        write_logic_processor_state(&mut bytes, &state, 1, true).unwrap();
        let mut read = bytes.as_slice();
        let decoded = read_logic_processor_state(&mut read, 1, true, 40).unwrap();

        assert_eq!(decoded.compressed_code, Some(Vec::new()));
        assert_eq!(decoded.config, None);
        assert_eq!(decoded.legacy_memory_slots, 2);
        assert_eq!(decoded.ipt, None);
        assert_eq!(decoded.tag, None);
        assert_eq!(decoded.icon_tag, 0);
        assert!(decoded.waits.is_empty());
        assert_eq!(decoded.accumulator, 0.0);
        assert!(read.is_empty());
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
