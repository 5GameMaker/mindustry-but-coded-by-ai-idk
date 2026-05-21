//! Shield component mirroring upstream `mindustry.entities.comp.ShieldComp`.

pub const MIN_ARMOR_DAMAGE: f32 = 0.1;

pub fn apply_armor(damage: f32, armor: f32) -> f32 {
    (damage - armor).max(MIN_ARMOR_DAMAGE * damage)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShieldComp {
    pub health: f32,
    pub hit_time: f32,
    pub x: f32,
    pub y: f32,
    pub health_multiplier: f32,
    pub armor_override: f32,
    pub dead: bool,
    pub killable: bool,
    pub shield: f32,
    pub armor: f32,
    pub shield_alpha: f32,
    killed: bool,
    shield_break_effects: usize,
}

impl ShieldComp {
    pub const fn new(health: f32) -> Self {
        Self {
            health,
            hit_time: 0.0,
            x: 0.0,
            y: 0.0,
            health_multiplier: 1.0,
            armor_override: -1.0,
            dead: false,
            killable: true,
            shield: 0.0,
            armor: 0.0,
            shield_alpha: 0.0,
            killed: false,
            shield_break_effects: 0,
        }
    }

    pub fn killed(&self) -> bool {
        self.killed
    }

    pub fn shield_break_effects(&self) -> usize {
        self.shield_break_effects
    }

    pub fn kill(&mut self) {
        self.dead = true;
        self.killed = true;
    }

    pub fn effective_armor(&self) -> f32 {
        if self.armor_override >= 0.0 {
            self.armor_override
        } else {
            self.armor
        }
    }

    pub fn damage(&mut self, amount: f32, unit_health_rule: f32) {
        let scaled =
            apply_armor(amount, self.effective_armor()) / self.health_multiplier / unit_health_rule;
        self.raw_damage(scaled);
    }

    pub fn damage_pierce(&mut self, amount: f32, with_effect: bool, unit_health_rule: f32) {
        let pre = self.hit_time;

        self.raw_damage(amount / self.health_multiplier / unit_health_rule);

        if !with_effect {
            self.hit_time = pre;
        }
    }

    pub fn damage_armor_mult(
        &mut self,
        amount: f32,
        armor_mult: f32,
        with_effect: bool,
        unit_health_rule: f32,
    ) {
        let pre = self.hit_time;
        let armor = if self.armor_override >= 0.0 {
            self.armor_override * armor_mult
        } else {
            self.armor * armor_mult
        };

        self.raw_damage(apply_armor(amount, armor) / self.health_multiplier / unit_health_rule);

        if !with_effect {
            self.hit_time = pre;
        }
    }

    pub fn raw_damage(&mut self, mut amount: f32) {
        let had_shields = self.shield > 0.0001;

        if self.health.is_nan() {
            self.health = 0.0;
        }

        if had_shields {
            self.shield_alpha = 1.0;
        }

        let shield_damage = self.shield.max(0.0).min(amount);
        self.shield -= shield_damage;
        self.hit_time = 1.0;
        amount -= shield_damage;

        if amount > 0.0 && self.killable {
            self.health -= amount;
            if self.health <= 0.0 && !self.dead {
                self.kill();
            }

            if had_shields && self.shield <= 0.0001 {
                self.shield_break_effects += 1;
            }
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.shield_alpha -= delta / 15.0;
        if self.shield_alpha < 0.0 {
            self.shield_alpha = 0.0;
        }
    }
}

impl Default for ShieldComp {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_armor_matches_java_min_armor_damage_formula() {
        assert_eq!(apply_armor(100.0, 30.0), 70.0);
        assert_eq!(apply_armor(100.0, 95.0), 10.0);
    }

    #[test]
    fn shield_component_absorbs_shield_then_health_and_records_break() {
        let mut shield = ShieldComp::new(50.0);
        shield.shield = 10.0;

        shield.raw_damage(12.0);

        assert_eq!(shield.shield, 0.0);
        assert_eq!(shield.health, 48.0);
        assert_eq!(shield.hit_time, 1.0);
        assert_eq!(shield.shield_alpha, 1.0);
        assert_eq!(shield.shield_break_effects(), 1);
        assert!(!shield.dead);
    }

    #[test]
    fn shield_component_damage_applies_armor_multiplier_and_unit_health_rule() {
        let mut shield = ShieldComp::new(100.0);
        shield.armor = 5.0;
        shield.health_multiplier = 2.0;

        shield.damage(25.0, 2.0);
        assert_eq!(shield.health, 95.0);

        shield.damage_armor_mult(25.0, 2.0, false, 1.0);
        assert_eq!(shield.health, 87.5);
        assert_eq!(shield.hit_time, 1.0);
    }

    #[test]
    fn shield_component_can_kill_and_fade_shield_alpha() {
        let mut shield = ShieldComp::new(5.0);

        shield.raw_damage(10.0);
        assert!(shield.dead);
        assert!(shield.killed());

        shield.shield_alpha = 1.0;
        shield.update(15.0);
        assert_eq!(shield.shield_alpha, 0.0);
    }
}
