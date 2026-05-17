use super::Liquid;

#[derive(Debug, Clone, PartialEq)]
pub struct CellLiquid {
    pub liquid: Liquid,
    pub color_from_rgba: u32,
    pub color_to_rgba: u32,
    pub cells: i32,
    pub spread_target: Option<String>,
    pub max_spread: f32,
    pub spread_conversion: f32,
    pub spread_damage: f32,
    pub remove_scaling: f32,
}

impl CellLiquid {
    pub fn new(name: impl Into<String>) -> Self {
        Self::with_color(name, 0xffffffff)
    }

    pub fn with_color(name: impl Into<String>, color_rgba: u32) -> Self {
        let liquid = Liquid::new(0, name);
        Self {
            liquid,
            color_from_rgba: 0xffffffff,
            color_to_rgba: 0xffffffff,
            cells: 6,
            spread_target: None,
            max_spread: 0.75,
            spread_conversion: 1.2,
            spread_damage: 0.11,
            remove_scaling: 0.25,
        }
        .with_base_color(color_rgba)
    }

    pub fn with_base_color(mut self, color_rgba: u32) -> Self {
        self.liquid.color_rgba = color_rgba;
        self.liquid.gas_color_rgba = color_rgba;
        self
    }
}

impl From<Liquid> for CellLiquid {
    fn from(liquid: Liquid) -> Self {
        Self {
            liquid,
            color_from_rgba: 0xffffffff,
            color_to_rgba: 0xffffffff,
            cells: 6,
            spread_target: None,
            max_spread: 0.75,
            spread_conversion: 1.2,
            spread_damage: 0.11,
            remove_scaling: 0.25,
        }
    }
}
