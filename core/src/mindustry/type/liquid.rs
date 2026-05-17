use crate::mindustry::ctype::{ContentId, ContentType, UnlockableContentBase};

#[derive(Debug, Clone, PartialEq)]
pub struct Liquid {
    pub base: UnlockableContentBase,
    pub gas: bool,
    pub color_rgba: u32,
    pub gas_color_rgba: u32,
    pub bar_color_rgba: Option<u32>,
    pub light_color_rgba: u32,
    pub flammability: f32,
    pub temperature: f32,
    pub heat_capacity: f32,
    pub viscosity: f32,
    pub explosiveness: f32,
    pub block_reactive: bool,
    pub coolant: bool,
    pub move_through_blocks: bool,
    pub incinerable: bool,
    pub effect: Option<String>,
    pub boil_point: f32,
    pub cap_puddles: bool,
    pub hidden: bool,
}

impl Liquid {
    pub const ANIMATION_FRAMES: i32 = 50;
    pub const ANIMATION_SCALE_GAS: f32 = 190.0;
    pub const ANIMATION_SCALE_LIQUID: f32 = 230.0;

    pub fn new(id: ContentId, name: impl Into<String>) -> Self {
        Self {
            base: UnlockableContentBase::new(id, ContentType::Liquid, name),
            gas: false,
            color_rgba: 0xffffffff,
            gas_color_rgba: 0xffffffff,
            bar_color_rgba: None,
            light_color_rgba: 0x00000000,
            flammability: 0.0,
            temperature: 0.5,
            heat_capacity: 0.5,
            viscosity: 0.5,
            explosiveness: 0.0,
            block_reactive: true,
            coolant: false,
            move_through_blocks: false,
            incinerable: true,
            effect: Some("none".to_string()),
            boil_point: 2.0,
            cap_puddles: true,
            hidden: false,
        }
    }
}
