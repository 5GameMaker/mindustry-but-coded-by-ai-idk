//! Battery shell mirroring upstream `mindustry.world.blocks.power.Battery`.

use crate::mindustry::world::{
    meta::{BlockFlag, BlockGroup, Env},
    Block,
};

use super::PowerDistributor;

#[derive(Debug, Clone, PartialEq)]
pub struct Battery {
    pub base: Block,
    pub drawer: String,
    pub empty_light_color: String,
    pub full_light_color: String,
    pub can_overdrive: bool,
}

impl Battery {
    pub fn new(name: impl Into<String>) -> Self {
        let mut base = PowerDistributor::new(name).base;
        base.group = BlockGroup::Power;
        base.outputs_power = true;
        base.consumes_power = true;
        base.destructible = true;
        base.update = false;
        base.env_enabled |= Env::SPACE;
        base.flags.push(BlockFlag::Battery);

        Self {
            base,
            drawer: "DrawMulti(DrawDefault, DrawPower, DrawRegion(-top))".into(),
            empty_light_color: "f8c266".into(),
            full_light_color: "fb9567".into(),
            can_overdrive: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_sets_java_defaults_and_visual_colors() {
        let battery = Battery::new("battery");

        assert_eq!(battery.base.name, "battery");
        assert!(battery.base.solid);
        assert!(battery.base.has_power);
        assert_eq!(battery.base.group, BlockGroup::Power);
        assert!(battery.base.outputs_power);
        assert!(battery.base.consumes_power);
        assert!(battery.base.destructible);
        assert!(!battery.base.update);
        assert_eq!(battery.base.env_enabled & Env::SPACE, Env::SPACE);
        assert!(battery.base.flags.contains(&BlockFlag::Battery));

        assert_eq!(
            battery.drawer,
            "DrawMulti(DrawDefault, DrawPower, DrawRegion(-top))"
        );
        assert_eq!(battery.empty_light_color, "f8c266");
        assert_eq!(battery.full_light_color, "fb9567");
        assert!(!battery.can_overdrive);
    }
}
