//! Lightweight Rust mirror of upstream `mindustry.game.MapObjectives`.
//!
//! Rendering, UI localization and generated Call dispatch are intentionally
//! represented as pure state/events here. This keeps the game-objective data
//! usable by server/network code while the full client renderer is migrated.

use crate::mindustry::logic::{double_bits_to_rgba, LMarkerControl, LOGIC_TILE_SIZE};

use std::collections::{BTreeMap, BTreeSet};

pub const OBJECTIVE_MARKER_DRAW_LAYER_OVERLAY_UI: f32 = 120.0;
pub const WORLD_LABEL_FLAG_BACKGROUND: u8 = 1;
pub const WORLD_LABEL_FLAG_OUTLINE: u8 = 2;
pub const ALIGN_CENTER: i32 = 1;

pub const MAP_OBJECTIVE_TYPE_NAMES: [&str; 13] = [
    "research",
    "produce",
    "item",
    "coreItem",
    "buildCount",
    "unitCount",
    "destroyUnits",
    "timer",
    "destroyBlock",
    "destroyBlocks",
    "destroyCore",
    "commandMode",
    "flag",
];

pub const OBJECTIVE_MARKER_TYPE_NAMES: [&str; 7] = [
    "shapeText",
    "point",
    "shape",
    "text",
    "line",
    "texture",
    "quad",
];

pub fn marker_type_by_java_name(name: &str) -> Option<&'static str> {
    match name {
        "ShapeText" | "shapeText" => Some("shapeText"),
        "Point" | "point" | "Minimap" | "minimap" => Some("point"),
        "Shape" | "shape" => Some("shape"),
        "Text" | "text" => Some("text"),
        "Line" | "line" => Some("line"),
        "Texture" | "texture" => Some("texture"),
        "Quad" | "quad" => Some("quad"),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn set(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Point2 {
    pub x: i32,
    pub y: i32,
}

impl Point2 {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectiveBuildingView {
    pub block: String,
    pub team: i32,
}

impl ObjectiveBuildingView {
    pub fn new(block: impl Into<String>, team: i32) -> Self {
        Self {
            block: block.into(),
            team,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapObjectiveContext {
    pub is_client: bool,
    pub headless: bool,
    pub delta: f32,
    pub objective_timer_multiplier: f32,
    pub objective_flags: BTreeSet<String>,
    pub unlocked_content: BTreeSet<String>,
    pub core_items: BTreeMap<String, i32>,
    pub core_item_counts: BTreeMap<String, i32>,
    pub placed_block_counts: BTreeMap<String, i32>,
    pub unit_counts: BTreeMap<String, i32>,
    pub enemy_units_destroyed: i32,
    pub enemy_core_count: usize,
    pub command_mode_used: bool,
    pub buildings: BTreeMap<Point2, ObjectiveBuildingView>,
    pub map_locales: BTreeMap<String, String>,
}

impl Default for MapObjectiveContext {
    fn default() -> Self {
        Self {
            is_client: false,
            headless: false,
            delta: 1.0,
            objective_timer_multiplier: 1.0,
            objective_flags: BTreeSet::new(),
            unlocked_content: BTreeSet::new(),
            core_items: BTreeMap::new(),
            core_item_counts: BTreeMap::new(),
            placed_block_counts: BTreeMap::new(),
            unit_counts: BTreeMap::new(),
            enemy_units_destroyed: 0,
            enemy_core_count: 1,
            command_mode_used: false,
            buildings: BTreeMap::new(),
            map_locales: BTreeMap::new(),
        }
    }
}

impl MapObjectiveContext {
    pub fn item_count(&self, item: &str) -> i32 {
        *self.core_items.get(item).unwrap_or(&0)
    }

    pub fn core_item_count(&self, item: &str) -> i32 {
        *self.core_item_counts.get(item).unwrap_or(&0)
    }

    pub fn placed_block_count(&self, block: &str) -> i32 {
        *self.placed_block_counts.get(block).unwrap_or(&0)
    }

    pub fn unit_count(&self, unit: &str) -> i32 {
        *self.unit_counts.get(unit).unwrap_or(&0)
    }

    pub fn block_destroyed(&self, pos: Point2, block: &str, team: i32) -> bool {
        self.buildings
            .get(&pos)
            .is_none_or(|building| building.team != team || building.block != block)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteObjectiveEvent {
    pub index: usize,
    pub completion_logic_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapObjectives {
    pub all: Vec<MapObjective>,
}

impl MapObjectives {
    pub fn new(all: Vec<MapObjective>) -> Self {
        let mut objectives = Self { all: Vec::new() };
        objectives.add_many(all);
        objectives
    }

    pub fn add(&mut self, objective: MapObjective) {
        self.flatten(objective);
    }

    pub fn add_many(&mut self, objectives: impl IntoIterator<Item = MapObjective>) {
        for objective in objectives {
            self.add(objective);
        }
    }

    fn flatten(&mut self, mut objective: MapObjective) {
        let parent_index = self.all.len() + objective.flattened_children_len();
        let children = std::mem::take(&mut objective.children);
        for mut child in children {
            child.common.parent_indices.push(parent_index);
            self.flatten(child);
        }
        self.all.push(objective);
    }

    pub fn get(&self, index: usize) -> Option<&MapObjective> {
        self.all.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut MapObjective> {
        self.all.get_mut(index)
    }

    pub fn any(&self) -> bool {
        self.all
            .iter()
            .enumerate()
            .any(|(index, _)| self.qualified(index))
    }

    pub fn clear(&mut self) {
        self.all.clear();
    }

    pub fn qualified(&self, index: usize) -> bool {
        let Some(objective) = self.all.get(index) else {
            return false;
        };
        !objective.common.completed
            && objective
                .common
                .parent_indices
                .iter()
                .all(|&parent| self.all.get(parent).is_some_and(MapObjective::is_completed))
    }

    pub fn running_indices(&self) -> Vec<usize> {
        (0..self.all.len())
            .filter(|&index| self.qualified(index))
            .collect()
    }

    pub fn update(&mut self, ctx: &mut MapObjectiveContext) -> Vec<CompleteObjectiveEvent> {
        let mut completed = Vec::new();
        for index in self.running_indices() {
            if self.all[index].update(ctx) && !ctx.is_client {
                if let Some(event) = self.complete(index, ctx) {
                    completed.push(event);
                }
            }
        }
        completed
    }

    pub fn complete(
        &mut self,
        index: usize,
        ctx: &mut MapObjectiveContext,
    ) -> Option<CompleteObjectiveEvent> {
        let objective = self.all.get_mut(index)?;
        if objective.common.completed {
            return None;
        }
        let completion_logic_code = objective.done(ctx);
        Some(CompleteObjectiveEvent {
            index,
            completion_logic_code,
        })
    }
}

impl Default for MapObjectives {
    fn default() -> Self {
        Self { all: Vec::new() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapObjectiveCommon {
    pub hidden: bool,
    pub details: Option<String>,
    pub completion_logic_code: Option<String>,
    pub flags_added: Vec<String>,
    pub flags_removed: Vec<String>,
    pub markers: Vec<ObjectiveMarker>,
    pub parent_indices: Vec<usize>,
    pub editor_x: i32,
    pub editor_y: i32,
    pub completed: bool,
}

impl Default for MapObjectiveCommon {
    fn default() -> Self {
        Self {
            hidden: false,
            details: None,
            completion_logic_code: None,
            flags_added: Vec::new(),
            flags_removed: Vec::new(),
            markers: Vec::new(),
            parent_indices: Vec::new(),
            editor_x: -999,
            editor_y: -999,
            completed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapObjective {
    pub common: MapObjectiveCommon,
    pub kind: MapObjectiveKind,
    pub children: Vec<MapObjective>,
    countup: f32,
}

impl MapObjective {
    pub fn new(kind: MapObjectiveKind) -> Self {
        Self {
            common: MapObjectiveCommon::default(),
            kind,
            children: Vec::new(),
            countup: 0.0,
        }
    }

    pub fn child(mut self, child: MapObjective) -> Self {
        self.children.push(child);
        self
    }

    pub fn details(mut self, details: impl Into<String>) -> Self {
        self.common.details = Some(details.into());
        self
    }

    pub fn flags_added(mut self, flags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.common.flags_added = flags.into_iter().map(Into::into).collect();
        self
    }

    pub fn flags_removed(mut self, flags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.common.flags_removed = flags.into_iter().map(Into::into).collect();
        self
    }

    pub fn markers(mut self, markers: impl IntoIterator<Item = ObjectiveMarker>) -> Self {
        self.common.markers = markers.into_iter().collect();
        self
    }

    pub fn completion_logic_code(mut self, code: impl Into<String>) -> Self {
        self.common.completion_logic_code = Some(code.into());
        self
    }

    pub fn is_completed(&self) -> bool {
        self.common.completed
    }

    pub fn reset(&mut self) {
        if matches!(self.kind, MapObjectiveKind::Timer { .. }) {
            self.countup = 0.0;
        }
    }

    pub fn update(&mut self, ctx: &MapObjectiveContext) -> bool {
        match &self.kind {
            MapObjectiveKind::Research { content } | MapObjectiveKind::Produce { content } => {
                ctx.unlocked_content.contains(content)
            }
            MapObjectiveKind::Item { item, amount } => ctx.item_count(item) >= *amount,
            MapObjectiveKind::CoreItem { item, amount } => ctx.core_item_count(item) >= *amount,
            MapObjectiveKind::BuildCount { block, count } => {
                ctx.placed_block_count(block) >= *count
            }
            MapObjectiveKind::UnitCount { unit, count } => ctx.unit_count(unit) >= *count,
            MapObjectiveKind::DestroyUnits { count } => ctx.enemy_units_destroyed >= *count,
            MapObjectiveKind::Timer { duration, .. } => {
                self.countup += ctx.delta;
                self.countup >= *duration * ctx.objective_timer_multiplier
            }
            MapObjectiveKind::DestroyBlock { pos, team, block } => {
                ctx.block_destroyed(*pos, block, *team)
            }
            MapObjectiveKind::DestroyBlocks {
                positions,
                team,
                block,
            } => positions
                .iter()
                .all(|&pos| ctx.block_destroyed(pos, block, *team)),
            MapObjectiveKind::CommandMode => ctx.headless || ctx.command_mode_used,
            MapObjectiveKind::Flag { flag, .. } => ctx.objective_flags.contains(flag),
            MapObjectiveKind::DestroyCore => ctx.enemy_core_count == 0,
        }
    }

    pub fn done(&mut self, ctx: &mut MapObjectiveContext) -> Option<String> {
        for flag in &self.common.flags_removed {
            ctx.objective_flags.remove(flag);
        }
        for flag in &self.common.flags_added {
            ctx.objective_flags.insert(flag.clone());
        }
        self.common.completed = true;
        self.common.completion_logic_code.clone()
    }

    pub fn text_token(&self, ctx: &MapObjectiveContext) -> Option<String> {
        match &self.kind {
            MapObjectiveKind::Research { content } => Some(format!("objective.research:{content}")),
            MapObjectiveKind::Produce { content } => Some(format!("objective.produce:{content}")),
            MapObjectiveKind::Item { item, amount } => Some(format!(
                "objective.item:{}/{amount}:{item}",
                ctx.item_count(item)
            )),
            MapObjectiveKind::CoreItem { item, amount } => Some(format!(
                "objective.coreitem:{}/{amount}:{item}",
                ctx.core_item_count(item)
            )),
            MapObjectiveKind::BuildCount { block, count } => Some(format!(
                "objective.build:{}:{block}",
                count - ctx.placed_block_count(block)
            )),
            MapObjectiveKind::UnitCount { unit, count } => Some(format!(
                "objective.buildunit:{}:{unit}",
                count - ctx.unit_count(unit)
            )),
            MapObjectiveKind::DestroyUnits { count } => Some(format!(
                "objective.destroyunits:{}",
                count - ctx.enemy_units_destroyed
            )),
            MapObjectiveKind::Timer { text, duration } => text.as_ref().map(|text| {
                let remaining =
                    ((*duration * ctx.objective_timer_multiplier - self.countup) / 60.0) as i32;
                let time = format_timer_seconds(remaining.max(0));
                format_objective_text(text, &time, &ctx.map_locales)
            }),
            MapObjectiveKind::DestroyBlock { block, .. } => {
                Some(format!("objective.destroyblock:{block}"))
            }
            MapObjectiveKind::DestroyBlocks {
                positions, block, ..
            } => Some(format!(
                "objective.destroyblocks:{}/{}:{block}",
                self.destroy_blocks_progress(ctx),
                positions.len()
            )),
            MapObjectiveKind::CommandMode => Some("objective.command".into()),
            MapObjectiveKind::Flag { text, .. } => text
                .as_ref()
                .map(|text| format_objective_text(text, "", &ctx.map_locales)),
            MapObjectiveKind::DestroyCore => Some("objective.destroycore".into()),
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.kind.type_name()
    }

    pub fn validate(&mut self) {
        self.kind.validate();
    }

    fn destroy_blocks_progress(&self, ctx: &MapObjectiveContext) -> usize {
        match &self.kind {
            MapObjectiveKind::DestroyBlocks {
                positions,
                team,
                block,
            } => positions
                .iter()
                .filter(|&&pos| ctx.block_destroyed(pos, block, *team))
                .count(),
            _ => 0,
        }
    }

    fn flattened_len(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(MapObjective::flattened_len)
            .sum::<usize>()
    }

    fn flattened_children_len(&self) -> usize {
        self.children.iter().map(MapObjective::flattened_len).sum()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapObjectiveKind {
    Research {
        content: String,
    },
    Produce {
        content: String,
    },
    Item {
        item: String,
        amount: i32,
    },
    CoreItem {
        item: String,
        amount: i32,
    },
    BuildCount {
        block: String,
        count: i32,
    },
    UnitCount {
        unit: String,
        count: i32,
    },
    DestroyUnits {
        count: i32,
    },
    Timer {
        text: Option<String>,
        duration: f32,
    },
    DestroyBlock {
        pos: Point2,
        team: i32,
        block: String,
    },
    DestroyBlocks {
        positions: Vec<Point2>,
        team: i32,
        block: String,
    },
    CommandMode,
    Flag {
        flag: String,
        text: Option<String>,
    },
    DestroyCore,
}

impl MapObjectiveKind {
    pub fn research(content: impl Into<String>) -> Self {
        Self::Research {
            content: content.into(),
        }
    }

    pub fn produce(content: impl Into<String>) -> Self {
        Self::Produce {
            content: content.into(),
        }
    }

    pub fn item(item: impl Into<String>, amount: i32) -> Self {
        Self::Item {
            item: item.into(),
            amount,
        }
    }

    pub fn core_item(item: impl Into<String>, amount: i32) -> Self {
        Self::CoreItem {
            item: item.into(),
            amount,
        }
    }

    pub fn build_count(block: impl Into<String>, count: i32) -> Self {
        Self::BuildCount {
            block: block.into(),
            count,
        }
    }

    pub fn unit_count(unit: impl Into<String>, count: i32) -> Self {
        Self::UnitCount {
            unit: unit.into(),
            count,
        }
    }

    pub fn destroy_units(count: i32) -> Self {
        Self::DestroyUnits { count }
    }

    pub fn timer(text: impl Into<Option<String>>, duration: f32) -> Self {
        Self::Timer {
            text: text.into(),
            duration,
        }
    }

    pub fn destroy_block(block: impl Into<String>, x: i32, y: i32, team: i32) -> Self {
        Self::DestroyBlock {
            pos: Point2::new(x, y),
            team,
            block: block.into(),
        }
    }

    pub fn destroy_blocks(
        block: impl Into<String>,
        team: i32,
        positions: impl IntoIterator<Item = Point2>,
    ) -> Self {
        Self::DestroyBlocks {
            positions: positions.into_iter().collect(),
            team,
            block: block.into(),
        }
    }

    pub fn flag(flag: impl Into<String>, text: impl Into<Option<String>>) -> Self {
        Self::Flag {
            flag: flag.into(),
            text: text.into(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            MapObjectiveKind::Research { .. } => "Research",
            MapObjectiveKind::Produce { .. } => "Produce",
            MapObjectiveKind::Item { .. } => "Item",
            MapObjectiveKind::CoreItem { .. } => "CoreItem",
            MapObjectiveKind::BuildCount { .. } => "BuildCount",
            MapObjectiveKind::UnitCount { .. } => "UnitCount",
            MapObjectiveKind::DestroyUnits { .. } => "DestroyUnits",
            MapObjectiveKind::Timer { .. } => "Timer",
            MapObjectiveKind::DestroyBlock { .. } => "DestroyBlock",
            MapObjectiveKind::DestroyBlocks { .. } => "DestroyBlocks",
            MapObjectiveKind::CommandMode => "CommandMode",
            MapObjectiveKind::Flag { .. } => "Flag",
            MapObjectiveKind::DestroyCore => "DestroyCore",
        }
    }

    pub fn validate(&mut self) {
        match self {
            MapObjectiveKind::Research { content } | MapObjectiveKind::Produce { content } => {
                if content.is_empty() {
                    *content = "copper".into();
                }
            }
            MapObjectiveKind::Item { item, .. } | MapObjectiveKind::CoreItem { item, .. } => {
                if item.is_empty() {
                    *item = "copper".into();
                }
            }
            MapObjectiveKind::BuildCount { block, .. }
            | MapObjectiveKind::DestroyBlock { block, .. }
            | MapObjectiveKind::DestroyBlocks { block, .. } => {
                if block.is_empty() {
                    *block = "router".into();
                }
            }
            MapObjectiveKind::UnitCount { unit, .. } => {
                if unit.is_empty() {
                    *unit = "dagger".into();
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarkerCommon {
    pub array_index: i32,
    pub world: bool,
    pub minimap: bool,
    pub autoscale: bool,
    pub draw_layer: f32,
}

impl Default for MarkerCommon {
    fn default() -> Self {
        Self {
            array_index: 0,
            world: true,
            minimap: false,
            autoscale: false,
            draw_layer: OBJECTIVE_MARKER_DRAW_LAYER_OVERLAY_UI,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectiveMarker {
    ShapeText(ShapeTextMarker),
    Point(PointMarker),
    Shape(ShapeMarker),
    Text(TextMarker),
    Line(LineMarker),
    Texture(TextureMarker),
    Quad(QuadMarker),
}

impl ObjectiveMarker {
    pub fn type_name(&self) -> &'static str {
        match self {
            ObjectiveMarker::ShapeText(_) => "ShapeText",
            ObjectiveMarker::Point(_) => "Point",
            ObjectiveMarker::Shape(_) => "Shape",
            ObjectiveMarker::Text(_) => "Text",
            ObjectiveMarker::Line(_) => "Line",
            ObjectiveMarker::Texture(_) => "Texture",
            ObjectiveMarker::Quad(_) => "Quad",
        }
    }

    pub fn common(&self) -> &MarkerCommon {
        match self {
            ObjectiveMarker::ShapeText(value) => &value.common,
            ObjectiveMarker::Point(value) => &value.common,
            ObjectiveMarker::Shape(value) => &value.common,
            ObjectiveMarker::Text(value) => &value.common,
            ObjectiveMarker::Line(value) => &value.common,
            ObjectiveMarker::Texture(value) => &value.common,
            ObjectiveMarker::Quad(value) => &value.common,
        }
    }

    pub fn common_mut(&mut self) -> &mut MarkerCommon {
        match self {
            ObjectiveMarker::ShapeText(value) => &mut value.common,
            ObjectiveMarker::Point(value) => &mut value.common,
            ObjectiveMarker::Shape(value) => &mut value.common,
            ObjectiveMarker::Text(value) => &mut value.common,
            ObjectiveMarker::Line(value) => &mut value.common,
            ObjectiveMarker::Texture(value) => &mut value.common,
            ObjectiveMarker::Quad(value) => &mut value.common,
        }
    }

    pub fn control(&mut self, type_: LMarkerControl, p1: f64, p2: f64, p3: f64) {
        control_common(self.common_mut(), type_, p1);
        match self {
            ObjectiveMarker::ShapeText(value) => value.control(type_, p1, p2, p3),
            ObjectiveMarker::Point(value) => value.control(type_, p1, p2, p3),
            ObjectiveMarker::Shape(value) => value.control(type_, p1, p2, p3),
            ObjectiveMarker::Text(value) => value.control(type_, p1, p2, p3),
            ObjectiveMarker::Line(value) => value.control(type_, p1, p2, p3),
            ObjectiveMarker::Texture(value) => value.control(type_, p1, p2, p3),
            ObjectiveMarker::Quad(value) => value.control(type_, p1, p2, p3),
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>, fetch: bool) {
        let text = text.into();
        match self {
            ObjectiveMarker::ShapeText(value) => value.set_text(text, fetch),
            ObjectiveMarker::Text(value) => value.set_text(text, fetch),
            _ => {}
        }
    }

    pub fn set_texture(&mut self, texture: TextureHolder) {
        match self {
            ObjectiveMarker::Texture(value) => value.texture = texture,
            ObjectiveMarker::Quad(value) => value.texture = texture,
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PosMarkerFields {
    pub common: MarkerCommon,
    pub pos: Vec2,
}

impl Default for PosMarkerFields {
    fn default() -> Self {
        Self {
            common: MarkerCommon::default(),
            pos: Vec2::default(),
        }
    }
}

impl PosMarkerFields {
    fn control_pos(&mut self, type_: LMarkerControl, p1: f64, p2: f64) {
        if type_ == LMarkerControl::Pos {
            if !p1.is_nan() {
                self.pos.x = p1 as f32 * LOGIC_TILE_SIZE;
            }
            if !p2.is_nan() {
                self.pos.y = p2 as f32 * LOGIC_TILE_SIZE;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapeTextMarker {
    pub common: MarkerCommon,
    pub pos: Vec2,
    pub text: String,
    pub font_size: f32,
    pub text_height: f32,
    pub flags: u8,
    pub text_align: i32,
    pub line_align: i32,
    pub radius: f32,
    pub rotation: f32,
    pub sides: i32,
    pub color: u32,
    pub fetched_text: Option<String>,
}

impl Default for ShapeTextMarker {
    fn default() -> Self {
        Self {
            common: MarkerCommon::default(),
            pos: Vec2::default(),
            text: "frog".into(),
            font_size: 1.0,
            text_height: 7.0,
            flags: WORLD_LABEL_FLAG_BACKGROUND | WORLD_LABEL_FLAG_OUTLINE,
            text_align: ALIGN_CENTER,
            line_align: ALIGN_CENTER,
            radius: 6.0,
            rotation: 0.0,
            sides: 4,
            color: 0xffd37fff,
            fetched_text: None,
        }
    }
}

impl ShapeTextMarker {
    pub fn new(text: impl Into<String>, x: f32, y: f32) -> Self {
        Self {
            text: text.into(),
            pos: Vec2::new(x, y),
            ..Self::default()
        }
    }

    fn control(&mut self, type_: LMarkerControl, p1: f64, p2: f64, _p3: f64) {
        PosMarkerFields {
            common: self.common.clone(),
            pos: self.pos,
        }
        .control_pos(type_, p1, p2);
        if type_ == LMarkerControl::Pos {
            if !p1.is_nan() {
                self.pos.x = p1 as f32 * LOGIC_TILE_SIZE;
            }
            if !p2.is_nan() {
                self.pos.y = p2 as f32 * LOGIC_TILE_SIZE;
            }
        }
        if !p1.is_nan() {
            match type_ {
                LMarkerControl::FontSize => self.font_size = p1 as f32,
                LMarkerControl::TextHeight => self.text_height = p1 as f32,
                LMarkerControl::TextAlign => self.text_align = p1 as i32,
                LMarkerControl::LineAlign => self.line_align = p1 as i32,
                LMarkerControl::Outline => {
                    set_flag(&mut self.flags, WORLD_LABEL_FLAG_OUTLINE, logic_bool(p1))
                }
                LMarkerControl::LabelFlags => {
                    set_flag(&mut self.flags, WORLD_LABEL_FLAG_BACKGROUND, logic_bool(p1))
                }
                LMarkerControl::Radius => self.radius = p1 as f32,
                LMarkerControl::Rotation => self.rotation = p1 as f32,
                LMarkerControl::Color => self.color = logic_color_to_rgba(p1),
                LMarkerControl::Shape => self.sides = p1 as i32,
                _ => {}
            }
        }
        if !p2.is_nan() && type_ == LMarkerControl::LabelFlags {
            set_flag(&mut self.flags, WORLD_LABEL_FLAG_OUTLINE, logic_bool(p2));
        }
    }

    fn set_text(&mut self, text: String, fetch: bool) {
        self.text = text;
        self.fetched_text = Some(if fetch {
            fetch_marker_text(&self.text, &BTreeMap::new(), false)
        } else {
            self.text.clone()
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointMarker {
    pub common: MarkerCommon,
    pub pos: Vec2,
    pub radius: f32,
    pub stroke: f32,
    pub color: u32,
}

impl Default for PointMarker {
    fn default() -> Self {
        Self {
            common: MarkerCommon::default(),
            pos: Vec2::default(),
            radius: 5.0,
            stroke: 11.0,
            color: 0xf25555ff,
        }
    }
}

impl PointMarker {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            ..Self::default()
        }
    }

    fn control(&mut self, type_: LMarkerControl, p1: f64, p2: f64, _p3: f64) {
        control_pos(&mut self.pos, type_, p1, p2);
        if !p1.is_nan() {
            match type_ {
                LMarkerControl::Radius => self.radius = p1 as f32,
                LMarkerControl::Stroke => self.stroke = p1 as f32,
                LMarkerControl::Color => self.color = logic_color_to_rgba(p1),
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapeMarker {
    pub common: MarkerCommon,
    pub pos: Vec2,
    pub radius: f32,
    pub rotation: f32,
    pub stroke: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub fill: bool,
    pub outline: bool,
    pub sides: i32,
    pub color: u32,
}

impl Default for ShapeMarker {
    fn default() -> Self {
        Self {
            common: MarkerCommon::default(),
            pos: Vec2::default(),
            radius: 8.0,
            rotation: 0.0,
            stroke: 1.0,
            start_angle: 0.0,
            end_angle: 360.0,
            fill: false,
            outline: true,
            sides: 4,
            color: 0xffd37fff,
        }
    }
}

impl ShapeMarker {
    fn control(&mut self, type_: LMarkerControl, p1: f64, p2: f64, p3: f64) {
        control_pos(&mut self.pos, type_, p1, p2);
        if !p1.is_nan() {
            match type_ {
                LMarkerControl::Radius => self.radius = p1 as f32,
                LMarkerControl::Stroke => self.stroke = p1 as f32,
                LMarkerControl::Outline => self.outline = logic_bool(p1),
                LMarkerControl::Rotation => self.rotation = p1 as f32,
                LMarkerControl::Color => self.color = logic_color_to_rgba(p1),
                LMarkerControl::Shape => self.sides = p1 as i32,
                LMarkerControl::Arc => self.start_angle = p1 as f32,
                _ => {}
            }
        }
        if !p2.is_nan() {
            match type_ {
                LMarkerControl::Shape => self.fill = logic_bool(p2),
                LMarkerControl::Arc => self.end_angle = p2 as f32,
                _ => {}
            }
        }
        if !p3.is_nan() && type_ == LMarkerControl::Shape {
            self.outline = logic_bool(p3);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextMarker {
    pub common: MarkerCommon,
    pub pos: Vec2,
    pub text: String,
    pub font_size: f32,
    pub flags: u8,
    pub text_align: i32,
    pub line_align: i32,
    pub fetched_text: Option<String>,
}

impl Default for TextMarker {
    fn default() -> Self {
        Self {
            common: MarkerCommon::default(),
            pos: Vec2::default(),
            text: "uwu".into(),
            font_size: 1.0,
            flags: WORLD_LABEL_FLAG_BACKGROUND | WORLD_LABEL_FLAG_OUTLINE,
            text_align: ALIGN_CENTER,
            line_align: ALIGN_CENTER,
            fetched_text: None,
        }
    }
}

impl TextMarker {
    fn control(&mut self, type_: LMarkerControl, p1: f64, p2: f64, _p3: f64) {
        control_pos(&mut self.pos, type_, p1, p2);
        if !p1.is_nan() {
            match type_ {
                LMarkerControl::FontSize => self.font_size = p1 as f32,
                LMarkerControl::TextAlign => self.text_align = p1 as i32,
                LMarkerControl::LineAlign => self.line_align = p1 as i32,
                LMarkerControl::Outline => {
                    set_flag(&mut self.flags, WORLD_LABEL_FLAG_OUTLINE, logic_bool(p1))
                }
                LMarkerControl::LabelFlags => {
                    set_flag(&mut self.flags, WORLD_LABEL_FLAG_BACKGROUND, logic_bool(p1))
                }
                _ => {}
            }
        }
        if !p2.is_nan() && type_ == LMarkerControl::LabelFlags {
            set_flag(&mut self.flags, WORLD_LABEL_FLAG_OUTLINE, logic_bool(p2));
        }
    }

    fn set_text(&mut self, text: String, fetch: bool) {
        self.text = text;
        self.fetched_text = Some(if fetch {
            fetch_marker_text(&self.text, &BTreeMap::new(), false)
        } else {
            self.text.clone()
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineMarker {
    pub common: MarkerCommon,
    pub pos: Vec2,
    pub end_pos: Vec2,
    pub stroke: f32,
    pub outline: bool,
    pub color1: u32,
    pub color2: u32,
}

impl Default for LineMarker {
    fn default() -> Self {
        Self {
            common: MarkerCommon::default(),
            pos: Vec2::default(),
            end_pos: Vec2::default(),
            stroke: 1.0,
            outline: true,
            color1: 0xffd37fff,
            color2: 0xffd37fff,
        }
    }
}

impl LineMarker {
    fn control(&mut self, type_: LMarkerControl, p1: f64, p2: f64, p3: f64) {
        control_pos(&mut self.pos, type_, p1, p2);
        if !p1.is_nan() {
            match type_ {
                LMarkerControl::EndPos => self.end_pos.x = p1 as f32 * LOGIC_TILE_SIZE,
                LMarkerControl::Stroke => self.stroke = p1 as f32,
                LMarkerControl::Color => {
                    let color = logic_color_to_rgba(p1);
                    self.color1 = color;
                    self.color2 = color;
                }
                LMarkerControl::Outline => self.outline = logic_bool(p1),
                _ => {}
            }
        }
        if !p2.is_nan() && type_ == LMarkerControl::EndPos {
            self.end_pos.y = p2 as f32 * LOGIC_TILE_SIZE;
        }
        if !p1.is_nan() && !p2.is_nan() {
            match type_ {
                LMarkerControl::Posi => {
                    set_indexed_pos(&mut self.pos, &mut self.end_pos, p1, Some(p2), None)
                }
                LMarkerControl::Colori => {
                    set_indexed_color(&mut self.color1, &mut self.color2, p1, p2)
                }
                _ => {}
            }
        }
        if !p1.is_nan() && !p3.is_nan() && type_ == LMarkerControl::Posi {
            set_indexed_pos(&mut self.pos, &mut self.end_pos, p1, None, Some(p3));
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureMarker {
    pub common: MarkerCommon,
    pub pos: Vec2,
    pub rotation: f32,
    pub width: f32,
    pub height: f32,
    pub texture: TextureHolder,
    pub color: u32,
}

impl Default for TextureMarker {
    fn default() -> Self {
        Self {
            common: MarkerCommon::default(),
            pos: Vec2::default(),
            rotation: 0.0,
            width: 0.0,
            height: 0.0,
            texture: TextureHolder::default(),
            color: 0xffffffff,
        }
    }
}

impl TextureMarker {
    fn control(&mut self, type_: LMarkerControl, p1: f64, p2: f64, _p3: f64) {
        control_pos(&mut self.pos, type_, p1, p2);
        if !p1.is_nan() {
            match type_ {
                LMarkerControl::Rotation => self.rotation = p1 as f32,
                LMarkerControl::TextureSize => self.width = p1 as f32 * LOGIC_TILE_SIZE,
                LMarkerControl::Color => self.color = logic_color_to_rgba(p1),
                _ => {}
            }
        }
        if !p2.is_nan() && type_ == LMarkerControl::TextureSize {
            self.height = p2 as f32 * LOGIC_TILE_SIZE;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuadMarker {
    pub common: MarkerCommon,
    pub texture: TextureHolder,
    pub vertices: Vec<f32>,
    pub map_region: bool,
}

impl Default for QuadMarker {
    fn default() -> Self {
        let mut vertices = vec![0.0; 24];
        for index in 0..4 {
            vertices[index * 6 + 2] = color_float_bits(0xffffffff);
            vertices[index * 6 + 5] = color_float_bits(0x00000000);
        }
        Self {
            common: MarkerCommon::default(),
            texture: TextureHolder::default(),
            vertices,
            map_region: true,
        }
    }
}

impl QuadMarker {
    fn control(&mut self, type_: LMarkerControl, p1: f64, p2: f64, p3: f64) {
        if !p1.is_nan() {
            match type_ {
                LMarkerControl::Color => {
                    let color = color_float_bits(logic_color_to_rgba(p1));
                    for index in 0..4 {
                        self.vertices[index * 6 + 2] = color;
                    }
                }
                LMarkerControl::Pos => self.vertices[0] = p1 as f32 * LOGIC_TILE_SIZE,
                LMarkerControl::Posi => self.set_pos(p1 as i32, p2, p3),
                LMarkerControl::Uvi => self.set_uv(p1 as i32, p2, p3),
                _ => {}
            }
        }
        if !p2.is_nan() && type_ == LMarkerControl::Pos {
            // Mirrors the upstream Java implementation, including its p1 use.
            self.vertices[1] = p1 as f32 * LOGIC_TILE_SIZE;
        }
        if !p1.is_nan() && !p2.is_nan() && type_ == LMarkerControl::Colori {
            self.set_color(p1 as i32, p2);
        }
    }

    fn set_pos(&mut self, index: i32, x: f64, y: f64) {
        if (0..4).contains(&index) {
            let offset = index as usize * 6;
            if !x.is_nan() {
                self.vertices[offset] = x as f32 * LOGIC_TILE_SIZE;
            }
            if !y.is_nan() {
                self.vertices[offset + 1] = y as f32 * LOGIC_TILE_SIZE;
            }
        }
    }

    fn set_color(&mut self, index: i32, color: f64) {
        if (0..4).contains(&index) {
            self.vertices[index as usize * 6 + 2] = color_float_bits(logic_color_to_rgba(color));
        }
    }

    fn set_uv(&mut self, index: i32, u: f64, v: f64) {
        if (0..4).contains(&index) {
            let offset = index as usize * 6;
            if !u.is_nan() {
                self.vertices[offset + 3] = u as f32;
            }
            if !v.is_nan() {
                self.vertices[offset + 4] = 1.0 - v as f32;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextureHolder {
    String(String),
    Content(String),
    Building(i32),
}

impl Default for TextureHolder {
    fn default() -> Self {
        Self::String("white".into())
    }
}

pub fn format_timer_seconds(seconds: i32) -> String {
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    if minutes > 0 {
        format!("{minutes}:{seconds:02}")
    } else {
        seconds.to_string()
    }
}

pub fn format_objective_text(
    text: &str,
    argument: &str,
    map_locales: &BTreeMap<String, String>,
) -> String {
    if let Some(key) = text.strip_prefix('@') {
        map_locales
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("{key}:{argument}"))
    } else if argument.is_empty() {
        text.to_string()
    } else {
        text.replace("{0}", argument)
    }
}

pub fn fetch_marker_text(
    text: &str,
    map_locales: &BTreeMap<String, String>,
    mobile: bool,
) -> String {
    let Some(key) = text.strip_prefix('@') else {
        return text.to_string();
    };
    if mobile {
        map_locales
            .get(&format!("{key}.mobile"))
            .or_else(|| map_locales.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    } else {
        map_locales
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}

fn control_common(common: &mut MarkerCommon, type_: LMarkerControl, p1: f64) {
    if p1.is_nan() {
        return;
    }
    match type_ {
        LMarkerControl::World => common.world = logic_bool(p1),
        LMarkerControl::Minimap => common.minimap = logic_bool(p1),
        LMarkerControl::Autoscale => common.autoscale = logic_bool(p1),
        LMarkerControl::DrawLayer => common.draw_layer = p1 as f32,
        _ => {}
    }
}

fn control_pos(pos: &mut Vec2, type_: LMarkerControl, p1: f64, p2: f64) {
    if type_ == LMarkerControl::Pos {
        if !p1.is_nan() {
            pos.x = p1 as f32 * LOGIC_TILE_SIZE;
        }
        if !p2.is_nan() {
            pos.y = p2 as f32 * LOGIC_TILE_SIZE;
        }
    }
}

fn set_indexed_pos(pos: &mut Vec2, end_pos: &mut Vec2, index: f64, x: Option<f64>, y: Option<f64>) {
    let target = if index as i32 == 0 {
        Some(pos)
    } else if index as i32 == 1 {
        Some(end_pos)
    } else {
        None
    };
    if let Some(target) = target {
        if let Some(x) = x.filter(|value| !value.is_nan()) {
            target.x = x as f32 * LOGIC_TILE_SIZE;
        }
        if let Some(y) = y.filter(|value| !value.is_nan()) {
            target.y = y as f32 * LOGIC_TILE_SIZE;
        }
    }
}

fn set_indexed_color(color1: &mut u32, color2: &mut u32, index: f64, color: f64) {
    match index as i32 {
        0 => *color1 = logic_color_to_rgba(color),
        1 => *color2 = logic_color_to_rgba(color),
        _ => {}
    }
}

fn set_flag(flags: &mut u8, flag: u8, enabled: bool) {
    if enabled {
        *flags |= flag;
    } else {
        *flags &= !flag;
    }
}

fn logic_bool(value: f64) -> bool {
    value.abs() >= 0.00001
}

fn logic_color_to_rgba(value: f64) -> u32 {
    double_bits_to_rgba(value)
}

fn color_float_bits(rgba: u32) -> f32 {
    f32::from_bits(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::logic::rgba_to_double_bits;

    #[test]
    fn objective_and_marker_registries_match_java_order_and_aliases() {
        assert_eq!(
            MAP_OBJECTIVE_TYPE_NAMES,
            [
                "research",
                "produce",
                "item",
                "coreItem",
                "buildCount",
                "unitCount",
                "destroyUnits",
                "timer",
                "destroyBlock",
                "destroyBlocks",
                "destroyCore",
                "commandMode",
                "flag"
            ]
        );
        assert_eq!(
            OBJECTIVE_MARKER_TYPE_NAMES,
            [
                "shapeText",
                "point",
                "shape",
                "text",
                "line",
                "texture",
                "quad"
            ]
        );
        assert_eq!(marker_type_by_java_name("Minimap"), Some("point"));
        assert_eq!(marker_type_by_java_name("shapeText"), Some("shapeText"));
        assert_eq!(marker_type_by_java_name("missing"), None);
    }

    #[test]
    fn objectives_flatten_children_after_dependencies_and_complete_flags() {
        let parent = MapObjective::new(MapObjectiveKind::flag("ready".to_string(), None))
            .flags_added(["opened"])
            .child(MapObjective::new(MapObjectiveKind::destroy_units(1)).flags_removed(["locked"]));
        let mut objectives = MapObjectives::default();
        objectives.add(parent);

        assert_eq!(objectives.all.len(), 2);
        assert_eq!(objectives.all[0].common.parent_indices, vec![1]);
        assert!(!objectives.qualified(0));
        assert!(objectives.qualified(1));

        let mut ctx = MapObjectiveContext::default();
        ctx.objective_flags.insert("ready".into());
        ctx.objective_flags.insert("locked".into());
        let completed = objectives.update(&mut ctx);
        assert_eq!(completed[0].index, 1);
        assert!(ctx.objective_flags.contains("opened"));
        assert!(!objectives.qualified(1));
        assert!(objectives.qualified(0));

        ctx.enemy_units_destroyed = 1;
        let completed = objectives.update(&mut ctx);
        assert_eq!(completed[0].index, 0);
        assert!(!ctx.objective_flags.contains("locked"));
    }

    #[test]
    fn objective_runtime_variants_follow_java_update_conditions() {
        let mut ctx = MapObjectiveContext::default();
        ctx.unlocked_content.insert("copper".into());
        ctx.core_items.insert("lead".into(), 4);
        ctx.core_item_counts.insert("coal".into(), 2);
        ctx.placed_block_counts.insert("router".into(), 3);
        ctx.unit_counts.insert("dagger".into(), 1);
        ctx.enemy_units_destroyed = 5;
        ctx.enemy_core_count = 0;
        ctx.command_mode_used = true;
        ctx.buildings
            .insert(Point2::new(1, 2), ObjectiveBuildingView::new("router", 2));

        assert!(MapObjective::new(MapObjectiveKind::research("copper")).update(&ctx));
        assert!(MapObjective::new(MapObjectiveKind::item("lead", 4)).update(&ctx));
        assert!(MapObjective::new(MapObjectiveKind::core_item("coal", 2)).update(&ctx));
        assert!(MapObjective::new(MapObjectiveKind::build_count("router", 3)).update(&ctx));
        assert!(MapObjective::new(MapObjectiveKind::unit_count("dagger", 1)).update(&ctx));
        assert!(MapObjective::new(MapObjectiveKind::destroy_units(5)).update(&ctx));
        assert!(MapObjective::new(MapObjectiveKind::DestroyCore).update(&ctx));
        assert!(MapObjective::new(MapObjectiveKind::CommandMode).update(&ctx));
        assert!(
            !MapObjective::new(MapObjectiveKind::destroy_block("router", 1, 2, 2)).update(&ctx)
        );
        assert!(MapObjective::new(MapObjectiveKind::destroy_block("router", 1, 2, 1)).update(&ctx));
    }

    #[test]
    fn timer_objective_counts_delta_and_formats_text_like_java() {
        let mut objective =
            MapObjective::new(MapObjectiveKind::timer(Some("T-{0}".into()), 60.0 * 120.0));
        let ctx = MapObjectiveContext {
            delta: 60.0 * 60.0,
            ..MapObjectiveContext::default()
        };
        assert!(!objective.update(&ctx));
        assert_eq!(objective.text_token(&ctx), Some("T-1:00".into()));
        assert!(objective.update(&ctx));
        objective.reset();
        assert_eq!(objective.text_token(&ctx), Some("T-2:00".into()));
    }

    #[test]
    fn marker_controls_mutate_marker_state_like_logic_marker_controls() {
        let mut marker = ObjectiveMarker::Shape(ShapeMarker::default());
        marker.control(LMarkerControl::World, 0.0, f64::NAN, f64::NAN);
        marker.control(LMarkerControl::Pos, 2.0, 3.0, f64::NAN);
        marker.control(LMarkerControl::Shape, 6.0, 1.0, 0.0);
        marker.control(
            LMarkerControl::Color,
            rgba_to_double_bits(0xff, 0x00, 0xaa, 0xff),
            f64::NAN,
            f64::NAN,
        );

        match marker {
            ObjectiveMarker::Shape(shape) => {
                assert!(!shape.common.world);
                assert_eq!(shape.pos, Vec2::new(16.0, 24.0));
                assert_eq!(shape.sides, 6);
                assert!(shape.fill);
                assert!(!shape.outline);
                assert_eq!(shape.color, 0xff00aaff);
            }
            _ => unreachable!(),
        }

        let mut line = ObjectiveMarker::Line(LineMarker::default());
        line.control(LMarkerControl::Posi, 1.0, 4.0, 5.0);
        line.control(
            LMarkerControl::Colori,
            1.0,
            rgba_to_double_bits(0x11, 0x22, 0x33, 0x44),
            f64::NAN,
        );
        match line {
            ObjectiveMarker::Line(line) => {
                assert_eq!(line.end_pos, Vec2::new(32.0, 40.0));
                assert_eq!(line.color2, 0x11223344);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn quad_marker_keeps_java_vertex_layout_and_indexed_controls() {
        let mut quad = QuadMarker::default();
        assert_eq!(quad.vertices.len(), 24);
        quad.control(LMarkerControl::Posi, 2.0, 7.0, 8.0);
        quad.control(LMarkerControl::Uvi, 2.0, 0.25, 0.75);
        assert_eq!(quad.vertices[12], 56.0);
        assert_eq!(quad.vertices[13], 64.0);
        assert_eq!(quad.vertices[15], 0.25);
        assert_eq!(quad.vertices[16], 0.25);
    }
}
