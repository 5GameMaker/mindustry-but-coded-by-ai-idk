//! World label component mirroring upstream `mindustry.entities.comp.WorldLabelComp`.

use crate::mindustry::{entities::EntityPosition, io::WorldLabelSyncWire};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldLabelAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldLabelDrawPlan {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub layer: f32,
    pub flags: u8,
    pub font_size: f32,
    pub align: WorldLabelAlign,
    pub line_align: WorldLabelAlign,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldLabelComp {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub font_size: f32,
    pub z: f32,
    pub flags: u8,
    pub parent_id: Option<i32>,
    pub parent: Option<(f32, f32)>,
    removed: bool,
}

impl WorldLabelComp {
    pub const DEFAULT_Z: f32 = 151.0;
    pub const FLAG_BACKGROUND: u8 = 1;
    pub const FLAG_OUTLINE: u8 = 2;
    pub const FLAG_ALIGN_LEFT: u8 = 4;
    pub const FLAG_ALIGN_RIGHT: u8 = 8;
    pub const FLAG_AUTOSCALE: u8 = 16;
    pub const FLAG_ONLY_PARENT_VISIBLE: u8 = 32;

    pub fn new(id: i32, x: f32, y: f32) -> Self {
        Self {
            id,
            x,
            y,
            text: "sample text".into(),
            font_size: 1.0,
            z: Self::DEFAULT_Z,
            flags: Self::FLAG_BACKGROUND | Self::FLAG_OUTLINE,
            parent_id: None,
            parent: None,
            removed: false,
        }
    }

    pub fn is_removed(&self) -> bool {
        self.removed
    }

    pub fn clip_size(&self) -> f32 {
        self.text.len() as f32 * 10.0 * self.font_size
    }

    pub fn draw(&self) -> WorldLabelDrawPlan {
        let (mut x, mut y) = (self.x, self.y);
        if let Some((px, py)) = self.parent {
            x += px;
            y += py;
        }
        Self::draw_at_plan(&self.text, x, y, self.z, self.flags, self.font_size)
    }

    pub fn draw_at_plan(
        text: &str,
        x: f32,
        y: f32,
        layer: f32,
        flags: u8,
        font_size: f32,
    ) -> WorldLabelDrawPlan {
        let line_align = if flags & Self::FLAG_ALIGN_LEFT != 0 {
            WorldLabelAlign::Left
        } else if flags & Self::FLAG_ALIGN_RIGHT != 0 {
            WorldLabelAlign::Right
        } else {
            WorldLabelAlign::Center
        };

        WorldLabelDrawPlan {
            text: text.into(),
            x,
            y,
            layer,
            flags,
            font_size,
            align: WorldLabelAlign::Center,
            line_align,
        }
    }

    pub fn apply_sync_wire(&mut self, sync: &WorldLabelSyncWire) {
        self.flags = sync.flags;
        self.font_size = sync.font_size;
        self.parent_id = sync.parent_id;
        self.text = sync.text.clone().unwrap_or_default();
        self.x = sync.x;
        self.y = sync.y;
        self.z = sync.z;
    }

    /// Java: must be called instead of `remove()`; also calls
    /// `Call.removeWorldLabel(id)`. The returned ID is that network call plan.
    pub fn hide(&mut self) -> i32 {
        self.removed = true;
        self.id
    }
}

impl EntityPosition for WorldLabelComp {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_label_defaults_match_java_fields_and_clip_size() {
        let label = WorldLabelComp::new(5, 10.0, 20.0);

        assert_eq!(label.text, "sample text");
        assert_eq!(label.font_size, 1.0);
        assert_eq!(
            label.flags,
            WorldLabelComp::FLAG_BACKGROUND | WorldLabelComp::FLAG_OUTLINE
        );
        assert_eq!(label.clip_size(), 110.0);
    }

    #[test]
    fn world_label_draw_uses_parent_offsets_and_alignment_flags() {
        let mut label = WorldLabelComp::new(5, 10.0, 20.0);
        label.parent = Some((2.0, 3.0));
        label.flags |= WorldLabelComp::FLAG_ALIGN_RIGHT;

        let plan = label.draw();

        assert_eq!((plan.x, plan.y), (12.0, 23.0));
        assert_eq!(plan.align, WorldLabelAlign::Center);
        assert_eq!(plan.line_align, WorldLabelAlign::Right);
    }

    #[test]
    fn world_label_applies_revision_1_sync_wire_fields() {
        let mut label = WorldLabelComp::new(5, 10.0, 20.0);
        let sync = WorldLabelSyncWire {
            flags: WorldLabelComp::FLAG_ALIGN_LEFT | WorldLabelComp::FLAG_AUTOSCALE,
            font_size: 1.75,
            parent_id: Some(42),
            text: Some("hello".into()),
            x: 30.0,
            y: 40.0,
            z: 155.0,
        };

        label.apply_sync_wire(&sync);

        assert_eq!(
            label.flags,
            WorldLabelComp::FLAG_ALIGN_LEFT | WorldLabelComp::FLAG_AUTOSCALE
        );
        assert_eq!(label.font_size, 1.75);
        assert_eq!(label.parent_id, Some(42));
        assert_eq!(label.text, "hello");
        assert_eq!((label.x, label.y, label.z), (30.0, 40.0, 155.0));
    }

    #[test]
    fn world_label_hide_marks_removed_and_returns_network_remove_id() {
        let mut label = WorldLabelComp::new(9, 0.0, 0.0);

        assert_eq!(label.hide(), 9);
        assert!(label.is_removed());
    }
}
