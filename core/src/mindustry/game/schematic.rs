use std::collections::HashMap;

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
    pub config: Option<String>,
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
