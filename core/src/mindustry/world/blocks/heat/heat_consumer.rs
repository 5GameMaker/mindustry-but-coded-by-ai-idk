//! Heat-consuming block interface mirroring upstream `mindustry.world.blocks.heat.HeatConsumer`.

use super::{calculate_heat, heat_consumer_requirement_met};

pub trait HeatConsumer {
    fn side_heat(&self) -> [f32; 4];

    fn heat_requirement(&self) -> f32;

    fn total_heat(&self) -> f32 {
        calculate_heat(&self.side_heat())
    }

    fn requirement_met(&self) -> bool {
        heat_consumer_requirement_met(self.total_heat(), self.heat_requirement())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Consumer {
        side_heat: [f32; 4],
        requirement: f32,
    }

    impl HeatConsumer for Consumer {
        fn side_heat(&self) -> [f32; 4] {
            self.side_heat
        }

        fn heat_requirement(&self) -> f32 {
            self.requirement
        }
    }

    #[test]
    fn heat_consumer_exposes_side_heat_and_requirement() {
        let consumer = Consumer {
            side_heat: [1.0, 2.0, 3.0, 4.0],
            requirement: 10.0,
        };

        assert_eq!(consumer.side_heat(), [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(consumer.heat_requirement(), 10.0);
        assert_eq!(consumer.total_heat(), 10.0);
        assert!(consumer.requirement_met());
    }

    #[test]
    fn zero_or_negative_heat_requirement_is_always_met_like_existing_helper() {
        let consumer = Consumer {
            side_heat: [0.0; 4],
            requirement: 0.0,
        };
        assert!(consumer.requirement_met());

        let consumer = Consumer {
            side_heat: [0.0; 4],
            requirement: -1.0,
        };
        assert!(consumer.requirement_met());
    }
}
