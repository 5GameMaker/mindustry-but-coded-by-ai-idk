use crate::mindustry::ctype::{ContentId, ContentType, UnlockableContentBase};

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub base: UnlockableContentBase,
    pub color_rgba: u32,
    pub explosiveness: f32,
    pub flammability: f32,
    pub radioactivity: f32,
    pub charge: f32,
    pub hardness: i32,
    pub cost: f32,
    pub health_scaling: f32,
    pub low_priority: bool,
    pub frames: i32,
    pub transition_frames: i32,
    pub frame_time: f32,
    pub buildable: bool,
    pub hidden: bool,
}

impl Item {
    pub fn new(id: ContentId, name: impl Into<String>) -> Self {
        Self {
            base: UnlockableContentBase::new(id, ContentType::Item, name),
            color_rgba: 0xffffffff,
            explosiveness: 0.0,
            flammability: 0.0,
            radioactivity: 0.0,
            charge: 0.0,
            hardness: 0,
            cost: 1.0,
            health_scaling: 0.0,
            low_priority: false,
            frames: 0,
            transition_frames: 0,
            frame_time: 5.0,
            buildable: true,
            hidden: false,
        }
    }
}
