use std::collections::{BTreeSet, HashMap};
use std::io::{self, Cursor, Read, Write};

use base64::{engine::general_purpose, Engine as _};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

use crate::mindustry::io::type_io::{
    read_i16, read_i32, read_java_utf, read_object, read_u8, write_i16, write_i32, write_java_utf,
    write_object, write_u8, TypeValue,
};
use crate::mindustry::world::{point2_pack, point2_x, point2_y};

pub const SCHEMATIC_HEADER: &[u8; 4] = b"msch";
pub const SCHEMATIC_VERSION: u8 = 1;
pub const MAX_SCHEMATIC_SIZE: i32 = 128;

#[derive(Debug, Clone, PartialEq)]
pub struct Schematic {
    pub tiles: Vec<SchematicTile>,
    pub labels: Vec<String>,
    pub tags: HashMap<String, String>,
    pub width: i32,
    pub height: i32,
    pub file: Option<String>,
    pub r#mod: Option<String>,
}

impl Schematic {
    pub fn new(
        tiles: Vec<SchematicTile>,
        tags: HashMap<String, String>,
        width: i32,
        height: i32,
    ) -> Self {
        Self {
            tiles,
            labels: Vec::new(),
            tags,
            width,
            height,
            file: None,
            r#mod: None,
        }
    }

    pub fn name(&self) -> String {
        self.tags
            .get("name")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string())
    }

    pub fn description(&self) -> String {
        self.tags.get("description").cloned().unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicTile {
    pub block: String,
    pub x: i16,
    pub y: i16,
    pub config: TypeValue,
    pub rotation: u8,
}

impl SchematicTile {
    pub fn new(
        block: impl Into<String>,
        x: i32,
        y: i32,
        config: Option<String>,
        rotation: u8,
    ) -> Self {
        let config = config.map(TypeValue::String).unwrap_or(TypeValue::Null);
        Self::with_config_value(block, x, y, config, rotation)
    }

    pub fn with_config_value(
        block: impl Into<String>,
        x: i32,
        y: i32,
        config: TypeValue,
        rotation: u8,
    ) -> Self {
        Self {
            block: block.into(),
            x: x as i16,
            y: y as i16,
            config,
            rotation,
        }
    }

    pub fn set(&mut self, other: &Self) -> &mut Self {
        self.block = other.block.clone();
        self.x = other.x;
        self.y = other.y;
        self.config = other.config.clone();
        self.rotation = other.rotation;
        self
    }

    pub fn copy(&self) -> Self {
        Self {
            block: self.block.clone(),
            x: self.x,
            y: self.y,
            config: self.config.clone(),
            rotation: self.rotation,
        }
    }
}

pub fn write_schematic(schematic: &Schematic) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    out.extend_from_slice(SCHEMATIC_HEADER);
    out.push(SCHEMATIC_VERSION);

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    write_schematic_body(&mut encoder, schematic)?;
    out.extend_from_slice(&encoder.finish()?);
    Ok(out)
}

pub fn read_schematic(bytes: &[u8]) -> io::Result<Schematic> {
    if bytes.len() < SCHEMATIC_HEADER.len() + 1 {
        return Err(invalid_data("schematic is too short"));
    }
    if &bytes[..SCHEMATIC_HEADER.len()] != SCHEMATIC_HEADER {
        return Err(invalid_data("Not a schematic file (missing header)."));
    }

    let version = bytes[SCHEMATIC_HEADER.len()];
    if version > SCHEMATIC_VERSION {
        return Err(invalid_data(format!(
            "Unknown schematic version: {version}"
        )));
    }

    let compressed = &bytes[SCHEMATIC_HEADER.len() + 1..];
    let mut decoder = ZlibDecoder::new(compressed);
    let mut body = Vec::new();
    decoder.read_to_end(&mut body)?;

    let mut cursor = Cursor::new(body);
    read_schematic_body(&mut cursor, version)
}

pub fn write_schematic_base64(schematic: &Schematic) -> io::Result<String> {
    Ok(general_purpose::STANDARD.encode(write_schematic(schematic)?))
}

pub fn read_schematic_base64(schematic: &str) -> io::Result<Schematic> {
    let bytes = general_purpose::STANDARD
        .decode(schematic.trim())
        .map_err(|_| invalid_data("invalid schematic base64"))?;
    read_schematic(&bytes)
}

fn write_schematic_body<W: Write>(write: &mut W, schematic: &Schematic) -> io::Result<()> {
    validate_schematic_bounds(schematic.width, schematic.height, schematic.tiles.len())?;

    write_i16(write, schematic.width as i16)?;
    write_i16(write, schematic.height as i16)?;

    let mut tags = schematic.tags.clone();
    tags.insert("labels".into(), labels_to_json(&schematic.labels));
    tags.insert("contentMap".into(), "{}".into());
    if tags.len() > u8::MAX as usize {
        return Err(invalid_input("too many schematic tags"));
    }

    write_u8(write, tags.len() as u8)?;
    let mut entries: Vec<_> = tags.iter().collect();
    entries.sort_by(|left, right| left.0.cmp(right.0));
    for (key, value) in entries {
        write_java_utf(write, key)?;
        write_java_utf(write, value)?;
    }

    let dictionary = block_dictionary(&schematic.tiles)?;
    write_u8(write, dictionary.len() as u8)?;
    for block in &dictionary {
        write_java_utf(write, block)?;
    }

    write_i32(write, schematic.tiles.len() as i32)?;
    for tile in &schematic.tiles {
        let block_index = dictionary
            .iter()
            .position(|block| block == &tile.block)
            .ok_or_else(|| invalid_data("schematic tile block missing from dictionary"))?;
        write_u8(write, block_index as u8)?;
        write_i32(write, point2_pack(tile.x as i32, tile.y as i32))?;
        write_object(write, &tile.config)?;
        write_u8(write, tile.rotation)?;
    }

    Ok(())
}

fn read_schematic_body<R: Read>(read: &mut R, version: u8) -> io::Result<Schematic> {
    let width = read_i16(read)? as i32;
    let height = read_i16(read)? as i32;
    validate_schematic_bounds(width, height, 0)?;

    let tag_count = read_u8(read)? as usize;
    let mut tags = HashMap::new();
    for _ in 0..tag_count {
        let key = read_java_utf(read)?;
        let value = read_java_utf(read)?;
        tags.insert(key, value);
    }

    let labels = tags
        .get("labels")
        .map(|value| labels_from_json(value))
        .transpose()?
        .unwrap_or_default();

    let block_count = read_u8(read)? as usize;
    let mut blocks = Vec::with_capacity(block_count);
    for _ in 0..block_count {
        blocks.push(read_java_utf(read)?);
    }

    let total = read_i32(read)?;
    if total < 0 || total > MAX_SCHEMATIC_SIZE * MAX_SCHEMATIC_SIZE {
        return Err(invalid_data("Invalid schematic: Too many blocks."));
    }

    let mut tiles = Vec::with_capacity(total as usize);
    for _ in 0..total {
        let block_index = read_u8(read)? as usize;
        let block = blocks
            .get(block_index)
            .cloned()
            .unwrap_or_else(|| "air".into());
        let position = read_i32(read)?;
        let config = if version == 0 {
            let _legacy = read_i32(read)?;
            TypeValue::Null
        } else {
            read_object(read)?
        };
        let rotation = read_u8(read)?;

        if block != "air" {
            tiles.push(SchematicTile::with_config_value(
                block,
                point2_x(position) as i32,
                point2_y(position) as i32,
                config,
                rotation,
            ));
        }
    }

    let mut schematic = Schematic::new(tiles, tags, width, height);
    schematic.labels = labels;
    Ok(schematic)
}

fn validate_schematic_bounds(width: i32, height: i32, tiles: usize) -> io::Result<()> {
    if width < 0 || height < 0 || width > MAX_SCHEMATIC_SIZE || height > MAX_SCHEMATIC_SIZE {
        return Err(invalid_data(
            "Invalid schematic: Too large (max possible size is 128x128)",
        ));
    }
    if tiles > (MAX_SCHEMATIC_SIZE * MAX_SCHEMATIC_SIZE) as usize {
        return Err(invalid_data("Invalid schematic: Too many blocks."));
    }
    Ok(())
}

fn block_dictionary(tiles: &[SchematicTile]) -> io::Result<Vec<String>> {
    let mut seen = BTreeSet::new();
    let mut blocks = Vec::new();
    for tile in tiles {
        if seen.insert(tile.block.clone()) {
            blocks.push(tile.block.clone());
        }
    }

    if blocks.len() > u8::MAX as usize {
        return Err(invalid_input("too many block types in schematic"));
    }
    Ok(blocks)
}

fn labels_to_json(labels: &[String]) -> String {
    let mut out = String::from("[");
    for (index, label) in labels.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('"');
        for ch in label.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                ch if ch < ' ' => out.push_str(&format!("\\u{:04x}", ch as u32)),
                ch => out.push(ch),
            }
        }
        out.push('"');
    }
    out.push(']');
    out
}

fn labels_from_json(json: &str) -> io::Result<Vec<String>> {
    let mut chars = json.trim().chars().peekable();
    if chars.next() != Some('[') {
        return Err(invalid_data("invalid schematic labels json"));
    }

    let mut labels = Vec::new();
    loop {
        while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
            chars.next();
        }
        match chars.peek().copied() {
            Some(']') => {
                chars.next();
                break;
            }
            Some('"') => {
                chars.next();
                let mut value = String::new();
                while let Some(ch) = chars.next() {
                    match ch {
                        '"' => break,
                        '\\' => match chars.next() {
                            Some('"') => value.push('"'),
                            Some('\\') => value.push('\\'),
                            Some('/') => value.push('/'),
                            Some('b') => value.push('\u{0008}'),
                            Some('f') => value.push('\u{000c}'),
                            Some('n') => value.push('\n'),
                            Some('r') => value.push('\r'),
                            Some('t') => value.push('\t'),
                            Some('u') => {
                                let mut code = 0u32;
                                for _ in 0..4 {
                                    let Some(hex) = chars.next().and_then(|c| c.to_digit(16))
                                    else {
                                        return Err(invalid_data(
                                            "invalid unicode escape in labels json",
                                        ));
                                    };
                                    code = (code << 4) | hex;
                                }
                                let Some(ch) = char::from_u32(code) else {
                                    return Err(invalid_data(
                                        "invalid unicode scalar in labels json",
                                    ));
                                };
                                value.push(ch);
                            }
                            _ => return Err(invalid_data("invalid escape in labels json")),
                        },
                        ch => value.push(ch),
                    }
                }
                labels.push(value);
            }
            _ => return Err(invalid_data("invalid schematic labels json")),
        }

        while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
            chars.next();
        }
        match chars.peek().copied() {
            Some(',') => {
                chars.next();
            }
            Some(']') => continue,
            _ => return Err(invalid_data("invalid schematic labels json")),
        }
    }

    while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
        chars.next();
    }
    if chars.peek().is_some() {
        return Err(invalid_data("trailing data in schematic labels json"));
    }
    Ok(labels)
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schematic_base_properties_follow_java_tag_defaults() {
        let schematic = Schematic::new(Vec::new(), HashMap::new(), 1, 1);
        assert_eq!(schematic.name(), "unknown");
        assert_eq!(schematic.description(), "");
    }

    #[test]
    fn schematic_binary_roundtrips_java_msch_shape() {
        let mut tags = HashMap::new();
        tags.insert("name".into(), "drill".into());
        tags.insert("description".into(), "mines copper".into());
        let mut schematic = Schematic::new(
            vec![
                SchematicTile::with_config_value("duo", 1, 2, TypeValue::Null, 3),
                SchematicTile::with_config_value(
                    "message",
                    -1,
                    5,
                    TypeValue::String("hello".into()),
                    0,
                ),
                SchematicTile::with_config_value("switch", 3, -2, TypeValue::Bool(true), 1),
            ],
            tags,
            8,
            9,
        );
        schematic.labels = vec!["attack".into(), "core\"logic".into()];

        let bytes = write_schematic(&schematic).unwrap();
        assert_eq!(&bytes[..4], SCHEMATIC_HEADER);
        assert_eq!(bytes[4], SCHEMATIC_VERSION);

        let decoded = read_schematic(&bytes).unwrap();
        assert_eq!(decoded.width, 8);
        assert_eq!(decoded.height, 9);
        assert_eq!(decoded.name(), "drill");
        assert_eq!(decoded.description(), "mines copper");
        assert_eq!(decoded.labels, schematic.labels);
        assert_eq!(decoded.tiles, schematic.tiles);
        assert!(decoded.tags.contains_key("contentMap"));
    }

    #[test]
    fn schematic_base64_uses_trimmed_java_payload() {
        let schematic = Schematic::new(
            vec![SchematicTile::new("core-shard", 0, 0, None, 0)],
            HashMap::new(),
            3,
            3,
        );
        let base64 = write_schematic_base64(&schematic).unwrap();
        let decoded = read_schematic_base64(&format!(" \n{base64}\n ")).unwrap();
        assert_eq!(decoded.tiles.len(), 1);
        assert_eq!(decoded.tiles[0].block, "core-shard");
    }

    #[test]
    fn schematic_reader_rejects_bad_header_version_and_size() {
        assert!(read_schematic(b"xxxx\x01").is_err());

        let mut bytes = Vec::from(*SCHEMATIC_HEADER);
        bytes.push(SCHEMATIC_VERSION + 1);
        bytes.extend_from_slice(&[0x78, 0x9c]);
        assert!(read_schematic(&bytes).is_err());

        let too_large = Schematic::new(Vec::new(), HashMap::new(), 129, 1);
        assert!(write_schematic(&too_large).is_err());
    }

    #[test]
    fn labels_json_handles_java_style_string_arrays() {
        let labels = vec![
            "alpha".to_string(),
            "quote\"slash\\".to_string(),
            "line\nbreak".to_string(),
        ];
        let json = labels_to_json(&labels);
        assert_eq!(labels_from_json(&json).unwrap(), labels);
        assert_eq!(labels_from_json("[]").unwrap(), Vec::<String>::new());
    }
}
