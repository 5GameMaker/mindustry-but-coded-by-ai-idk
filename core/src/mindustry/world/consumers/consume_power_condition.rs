//! Conditional power consumer mirroring upstream `mindustry.world.consumers.ConsumePowerCondition`.

use super::consume_power_condition_requested;

pub struct ConsumePowerCondition<F> {
    pub usage: f32,
    consume: F,
}

impl<F> ConsumePowerCondition<F> {
    pub fn new(usage: f32, consume: F) -> Self {
        Self { usage, consume }
    }
}

impl<F> ConsumePowerCondition<F> {
    pub fn requested_power<Building>(&self, building: &Building) -> f32
    where
        F: Fn(&Building) -> bool,
    {
        consume_power_condition_requested(self.usage, (self.consume)(building))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Building {
        active: bool,
    }

    #[test]
    fn requested_power_returns_usage_only_when_condition_matches() {
        let consumer = ConsumePowerCondition::new(3.5, |building: &Building| building.active);

        assert_eq!(consumer.requested_power(&Building { active: true }), 3.5);
        assert_eq!(consumer.requested_power(&Building { active: false }), 0.0);
    }
}
