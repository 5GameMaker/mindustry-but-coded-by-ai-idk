use crate::mindustry::ctype::{ContentId, ContentType, UnlockableContentBase};

#[derive(Debug, Clone, PartialEq)]
pub struct StatusEffect {
    pub base: UnlockableContentBase,
    pub damage_multiplier: f32,
    pub health_multiplier: f32,
    pub speed_multiplier: f32,
    pub reload_multiplier: f32,
    pub build_speed_multiplier: f32,
    pub drag_multiplier: f32,
    pub transition_damage: f32,
    pub disarm: bool,
    pub damage: f32,
    pub interval_damage_time: f32,
    pub interval_damage: f32,
    pub interval_damage_pierce: bool,
    pub effect_chance: f32,
    pub permanent: bool,
    pub reactive: bool,
    pub dynamic: bool,
    pub show: bool,
    pub color_rgba: u32,
    pub opposites: Vec<String>,
    pub affinities: Vec<String>,
    pub outline: bool,
}

impl StatusEffect {
    pub fn new(id: ContentId, name: impl Into<String>) -> Self {
        Self {
            base: UnlockableContentBase::new(id, ContentType::Status, name),
            damage_multiplier: 1.0,
            health_multiplier: 1.0,
            speed_multiplier: 1.0,
            reload_multiplier: 1.0,
            build_speed_multiplier: 1.0,
            drag_multiplier: 1.0,
            transition_damage: 0.0,
            disarm: false,
            damage: 0.0,
            interval_damage_time: 60.0,
            interval_damage: 0.0,
            interval_damage_pierce: true,
            effect_chance: 0.15,
            permanent: false,
            reactive: false,
            dynamic: false,
            show: true,
            color_rgba: 0xffffffff,
            opposites: Vec::new(),
            affinities: Vec::new(),
            outline: true,
        }
    }
}
