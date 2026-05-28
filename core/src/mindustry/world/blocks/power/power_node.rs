//! Power node shell mirroring upstream `mindustry.world.blocks.power.PowerNode`.

use crate::mindustry::world::{
    meta::{BlockGroup, Env},
    Block,
};

use super::PowerBlock;

#[derive(Debug, Clone, PartialEq)]
pub struct PowerNode {
    pub base: Block,
    pub laser: String,
    pub laser_end: String,
    pub laser_range: f32,
    pub max_nodes: i32,
    pub autolink: bool,
    pub draw_range: bool,
    pub same_block_connection: bool,
    pub laser_scale: f32,
    pub power_layer: String,
    pub laser_color1: String,
    pub laser_color2: String,
    pub can_overdrive: bool,
    pub draw_disabled: bool,
    pub schematic_priority: i32,
    pub swap_diagonal_placement: bool,
    pub delay_landing_config: bool,
}

impl PowerNode {
    pub fn new(name: impl Into<String>) -> Self {
        let mut base = PowerBlock::new(name).base;
        base.group = BlockGroup::Power;
        base.configurable = true;
        base.ignore_resize_config = true;
        base.consumes_power = false;
        base.outputs_power = false;
        base.destructible = true;
        base.update = false;
        base.env_enabled |= Env::SPACE;

        Self {
            base,
            laser: "laser".into(),
            laser_end: "laser-end".into(),
            laser_range: 6.0,
            max_nodes: 3,
            autolink: true,
            draw_range: true,
            same_block_connection: false,
            laser_scale: 0.25,
            power_layer: "Layer.power".into(),
            laser_color1: "white".into(),
            laser_color2: "powerLight".into(),
            can_overdrive: false,
            draw_disabled: false,
            schematic_priority: -10,
            swap_diagonal_placement: true,
            delay_landing_config: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_node_sets_java_wiring_defaults() {
        let node = PowerNode::new("power-node");

        assert_eq!(node.base.name, "power-node");
        assert_eq!(node.base.group, BlockGroup::Power);
        assert!(node.base.solid);
        assert!(node.base.has_power);
        assert!(node.base.configurable);
        assert!(node.base.ignore_resize_config);
        assert!(!node.base.consumes_power);
        assert!(!node.base.outputs_power);
        assert!(node.base.destructible);
        assert!(!node.base.update);
        assert_eq!(node.base.env_enabled & Env::SPACE, Env::SPACE);

        assert_eq!(node.laser, "laser");
        assert_eq!(node.laser_end, "laser-end");
        assert_eq!(node.laser_range, 6.0);
        assert_eq!(node.max_nodes, 3);
        assert!(node.autolink);
        assert!(node.draw_range);
        assert!(!node.same_block_connection);
        assert_eq!(node.laser_scale, 0.25);
        assert_eq!(node.power_layer, "Layer.power");
        assert_eq!(node.laser_color1, "white");
        assert_eq!(node.laser_color2, "powerLight");
        assert!(!node.can_overdrive);
        assert!(!node.draw_disabled);
        assert_eq!(node.schematic_priority, -10);
        assert!(node.swap_diagonal_placement);
        assert!(node.delay_landing_config);
    }
}
