//! Health component mirroring upstream `mindustry.entities.comp.HealthComp`.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealthComp {
    pub health: f32,
    pub hit_time: f32,
    pub max_health: f32,
    pub dead: bool,
    removed: bool,
    killed_calls: usize,
}

impl HealthComp {
    pub const HIT_DURATION: f32 = 9.0;

    pub const fn new(max_health: f32) -> Self {
        Self {
            health: max_health,
            hit_time: 0.0,
            max_health,
            dead: false,
            removed: false,
            killed_calls: 0,
        }
    }

    pub fn is_removed(&self) -> bool {
        self.removed
    }

    pub fn killed_calls(&self) -> usize {
        self.killed_calls
    }

    pub fn is_valid(&self, is_added: bool) -> bool {
        !self.dead && is_added
    }

    pub fn healthf(&self) -> f32 {
        self.health / self.max_health
    }

    pub fn update(&mut self, delta: f32) {
        self.hit_time -= delta / Self::HIT_DURATION;
    }

    pub fn killed(&mut self) {
        self.killed_calls += 1;
    }

    pub fn remove(&mut self) {
        self.removed = true;
    }

    pub fn kill(&mut self) {
        if self.dead {
            return;
        }

        self.health = self.health.min(0.0);
        self.dead = true;
        self.killed();
        self.remove();
    }

    pub fn heal_full(&mut self) {
        self.dead = false;
        self.health = self.max_health;
    }

    pub fn damaged(&self) -> bool {
        self.health < self.max_health - 0.001
    }

    pub fn damage_pierce(&mut self, amount: f32, with_effect: bool) {
        self.damage_with_effect(amount, with_effect);
    }

    pub fn damage_armor_mult(&mut self, amount: f32, _armor_mult: f32, with_effect: bool) {
        self.damage_with_effect(amount, with_effect);
    }

    pub fn damage(&mut self, amount: f32) {
        if self.health.is_nan() {
            self.health = 0.0;
        }

        self.health -= amount;
        self.hit_time = 1.0;
        if self.health <= 0.0 && !self.dead {
            self.kill();
        }
    }

    pub fn damage_with_effect(&mut self, amount: f32, with_effect: bool) {
        let pre = self.hit_time;

        self.damage(amount);

        if !with_effect {
            self.hit_time = pre;
        }
    }

    pub fn damage_continuous(&mut self, amount: f32, delta: f32) {
        self.damage_with_effect(amount * delta, self.hit_time <= -10.0 + Self::HIT_DURATION);
    }

    pub fn damage_continuous_pierce(&mut self, amount: f32, delta: f32) {
        self.damage_pierce(amount * delta, self.hit_time <= -20.0 + Self::HIT_DURATION);
    }

    pub fn damage_continuous_armor_mult(&mut self, amount: f32, armor_mult: f32, delta: f32) {
        self.damage_armor_mult(
            amount * delta,
            armor_mult,
            self.hit_time <= -20.0 + Self::HIT_DURATION,
        );
    }

    pub fn clamp_health(&mut self) {
        if self.health.is_nan() {
            self.health = 0.0;
        } else {
            self.health = self.health.min(self.max_health);
        }
    }

    pub fn heal(&mut self, amount: f32) {
        self.health += amount;
        self.clamp_health();
    }

    pub fn heal_fract(&mut self, amount: f32) {
        self.heal(amount * self.max_health);
    }
}

impl Default for HealthComp {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_component_validity_fraction_and_hit_timer_follow_java_shape() {
        let mut health = HealthComp::new(100.0);

        assert!(health.is_valid(true));
        assert_eq!(health.healthf(), 1.0);

        health.hit_time = 1.0;
        health.update(9.0);
        assert_eq!(health.hit_time, 0.0);
    }

    #[test]
    fn health_component_damage_kill_and_effect_suppression_match_java() {
        let mut health = HealthComp::new(10.0);
        health.hit_time = 0.25;

        health.damage_with_effect(3.0, false);
        assert_eq!(health.health, 7.0);
        assert_eq!(health.hit_time, 0.25);
        assert!(health.damaged());

        health.damage(20.0);
        assert_eq!(health.health, -13.0);
        assert!(health.dead);
        assert!(health.is_removed());
        assert_eq!(health.killed_calls(), 1);

        health.kill();
        assert_eq!(health.killed_calls(), 1);
    }

    #[test]
    fn health_component_heal_clamp_and_continuous_damage_are_deterministic() {
        let mut health = HealthComp::new(20.0);
        health.health = f32::NAN;
        health.clamp_health();
        assert_eq!(health.health, 0.0);

        health.heal(5.0);
        health.heal_fract(1.0);
        assert_eq!(health.health, 20.0);

        health.damage_continuous(4.0, 0.5);
        assert_eq!(health.health, 18.0);
    }

    #[test]
    fn health_component_heal_full_revives_to_max_health() {
        let mut health = HealthComp::new(5.0);
        health.damage(10.0);
        assert!(health.dead);

        health.heal_full();
        assert!(!health.dead);
        assert_eq!(health.health, 5.0);
    }
}
