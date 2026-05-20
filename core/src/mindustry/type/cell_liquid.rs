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

    pub fn with_spread_target(mut self, target: impl Into<String>) -> Self {
        self.spread_target = Some(target.into());
        self
    }

    pub fn react(&self, other: &Liquid, amount: f32) -> f32 {
        if self.spread_target.as_deref() == Some(other.name()) {
            amount
        } else {
            0.0
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_liquid_defaults_match_java_field_initializers() {
        let cell = CellLiquid::with_color("neoplasm", 0xff795eff);
        assert_eq!(cell.liquid.name(), "neoplasm");
        assert_eq!(cell.liquid.color_rgba, 0xff795eff);
        assert_eq!(cell.liquid.gas_color_rgba, 0xff795eff);
        assert_eq!(cell.color_from_rgba, 0xffffffff);
        assert_eq!(cell.color_to_rgba, 0xffffffff);
        assert_eq!(cell.cells, 6);
        assert_eq!(cell.spread_target, None);
        assert_eq!(cell.max_spread, 0.75);
        assert_eq!(cell.spread_conversion, 1.2);
        assert_eq!(cell.spread_damage, 0.11);
        assert_eq!(cell.remove_scaling, 0.25);
    }

    #[test]
    fn cell_liquid_react_consumes_only_the_configured_spread_target() {
        let cell = CellLiquid::new("neoplasm").with_spread_target("water");
        let water = Liquid::new(1, "water");
        let oil = Liquid::new(2, "oil");

        assert_eq!(cell.react(&water, 9.5), 9.5);
        assert_eq!(cell.react(&oil, 9.5), 0.0);
        assert_eq!(CellLiquid::new("neoplasm").react(&water, 9.5), 0.0);
    }
}
