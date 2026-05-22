#[derive(Debug, Clone, PartialEq)]
pub struct BulletType {
    pub name: String,
    pub lifetime: f32,
    pub life_scale_rand_min: f32,
    pub life_scale_rand_max: f32,
    pub speed: f32,
    pub velocity_scale_rand_min: f32,
    pub velocity_scale_rand_max: f32,
    pub damage: f32,
    pub hit_size: f32,
    pub draw_size: f32,
    pub drag: f32,
    pub hittable: bool,
    pub collides_tiles: bool,
    pub collides_ground: bool,
    pub collides_air: bool,
    pub collides: bool,
    pub keep_velocity: bool,
    pub instant_disappear: bool,
    pub kill_shooter: bool,
    pub scaled_splash_damage: bool,
    pub pierce: bool,
    pub pierce_building: bool,
    pub pierce_cap: i32,
    pub splash_damage: f32,
    pub shield_damage_multiplier: f32,
    pub building_damage_multiplier: f32,
    pub range_override: f32,
    pub max_range: f32,
    pub range_change: f32,
    pub extra_range_margin: f32,
    pub range: f32,
    pub heal_percent: f32,
    pub heal_amount: f32,
    pub frag_bullet_dps: Option<f32>,
    pub frag_bullets: i32,
    pub splash_damage_radius: f32,
    pub lightning: i32,
    pub lightning_length: i32,
    pub lightning_length_rand: i32,
    pub despawn_hit: bool,
    pub set_defaults: bool,
    pub trail_length: i32,
    pub light_radius: f32,
    pub spawn_unit_range: Option<f32>,
    pub despawn_unit_range: Option<f32>,
    pub spawn_unit_dps: Option<f32>,
    pub despawn_unit_dps: Option<f32>,
    pub cached_dps: Option<f32>,
}

impl Default for BulletType {
    fn default() -> Self {
        Self {
            name: String::new(),
            lifetime: 40.0,
            life_scale_rand_min: 1.0,
            life_scale_rand_max: 1.0,
            speed: 1.0,
            velocity_scale_rand_min: 1.0,
            velocity_scale_rand_max: 1.0,
            damage: 1.0,
            hit_size: 4.0,
            draw_size: 40.0,
            drag: 0.0,
            hittable: true,
            collides_tiles: true,
            collides_ground: true,
            collides_air: true,
            collides: true,
            keep_velocity: true,
            instant_disappear: false,
            kill_shooter: false,
            scaled_splash_damage: false,
            pierce: false,
            pierce_building: false,
            pierce_cap: -1,
            splash_damage: 0.0,
            shield_damage_multiplier: 1.0,
            building_damage_multiplier: 1.0,
            range_override: -1.0,
            max_range: -1.0,
            range_change: 0.0,
            extra_range_margin: 0.0,
            range: 0.0,
            heal_percent: 0.0,
            heal_amount: 0.0,
            frag_bullet_dps: None,
            frag_bullets: 9,
            splash_damage_radius: -1.0,
            lightning: 0,
            lightning_length: 5,
            lightning_length_rand: 0,
            despawn_hit: false,
            set_defaults: true,
            trail_length: -1,
            light_radius: -1.0,
            spawn_unit_range: None,
            despawn_unit_range: None,
            spawn_unit_dps: None,
            despawn_unit_dps: None,
            cached_dps: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FireBulletUpdatePlan {
    pub create_fire: bool,
    pub trail_effect: bool,
    pub secondary_trail_effect: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FireBulletType {
    pub base: BulletType,
    pub radius: f32,
    pub vel_min: f32,
    pub vel_max: f32,
    pub fire_trail_chance: f32,
    pub fire_effect_chance: f32,
    pub fire_effect_chance2: f32,
}

impl Default for FireBulletType {
    fn default() -> Self {
        let base = BulletType {
            pierce: true,
            collides_tiles: false,
            collides: false,
            drag: 0.03,
            ..Default::default()
        };
        Self {
            base,
            radius: 3.0,
            vel_min: 0.6,
            vel_max: 2.6,
            fire_trail_chance: 0.04,
            fire_effect_chance: 0.1,
            fire_effect_chance2: 0.1,
        }
    }
}

impl FireBulletType {
    pub fn new(speed: f32, damage: f32) -> Self {
        let mut out = Self::default();
        out.base.speed = speed;
        out.base.damage = damage;
        out
    }

    pub fn initial_velocity_len(&self, random_len: f32) -> f32 {
        random_len.clamp(self.vel_min, self.vel_max)
    }

    pub fn draw_radius(&self, fout: f32) -> f32 {
        self.radius * fout.clamp(0.0, 1.0)
    }

    pub fn update_plan(
        &self,
        fire_trail_triggered: bool,
        fire_effect_triggered: bool,
        second_effect_triggered: bool,
    ) -> FireBulletUpdatePlan {
        FireBulletUpdatePlan {
            create_fire: fire_trail_triggered,
            trail_effect: fire_effect_triggered,
            secondary_trail_effect: second_effect_triggered,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightningBulletType {
    pub base: BulletType,
}

impl Default for LightningBulletType {
    fn default() -> Self {
        Self {
            base: BulletType {
                damage: 1.0,
                speed: 0.0,
                lifetime: 1.0,
                keep_velocity: false,
                hittable: false,
                lightning_length: 25,
                lightning_length_rand: 0,
                ..Default::default()
            },
        }
    }
}

impl LightningBulletType {
    pub fn calculate_range(&self) -> f32 {
        (self.base.lightning_length as f32 + self.base.lightning_length_rand as f32 / 2.0) * 6.0
    }

    pub fn estimate_dps(&mut self) -> f32 {
        self.base.estimate_dps() * (self.base.lightning_length as f32 / 10.0).max(1.0)
    }

    pub fn lightning_length(&self, random_length: i32) -> i32 {
        self.base.lightning_length + random_length.clamp(0, self.base.lightning_length_rand.max(0))
    }
}

pub fn empty_bullet_type() -> BulletType {
    BulletType {
        hittable: false,
        collides_ground: false,
        collides_air: false,
        collides_tiles: false,
        speed: 0.0,
        keep_velocity: false,
        ..Default::default()
    }
}

pub fn explosion_bullet_type(splash_damage: f32, splash_damage_radius: f32) -> BulletType {
    BulletType {
        splash_damage,
        splash_damage_radius,
        hittable: false,
        lifetime: 1.0,
        speed: 0.0,
        range_override: 20.0_f32.max(splash_damage_radius * 2.0 / 3.0),
        instant_disappear: true,
        scaled_splash_damage: true,
        kill_shooter: true,
        collides: false,
        keep_velocity: false,
        ..Default::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BulletCreatePlan {
    pub angle: f32,
    pub damage: f32,
    pub velocity_scale: f32,
    pub lifetime_scale: f32,
    pub lifetime: f32,
    pub speed: f32,
}

impl BulletType {
    pub fn init_defaults(&mut self) {
        if self.pierce_cap >= 1 {
            self.pierce = true;
        }

        if self.set_defaults
            && (self.frag_bullet_dps.is_some()
                || self.splash_damage_radius > 0.0
                || self.lightning > 0)
        {
            self.despawn_hit = true;
        }

        if self.light_radius <= -1.0 {
            self.light_radius = 18.0_f32.max(self.hit_size * 5.0);
        }

        self.draw_size = self
            .draw_size
            .max(self.trail_length.max(0) as f32 * self.speed * 2.0);
        self.range = self.calculate_range();
    }

    pub fn calculate_range(&self) -> f32 {
        if self.range_override > 0.0 {
            return self.range_override;
        }
        if let Some(range) = self.spawn_unit_range {
            return range;
        }
        if let Some(range) = self.despawn_unit_range {
            return range;
        }

        let travel = if self.drag.abs() <= f32::EPSILON {
            self.speed * self.lifetime
        } else {
            self.speed * (1.0 - (1.0 - self.drag).powf(self.lifetime)) / self.drag
        };
        travel.max(self.max_range)
    }

    pub fn estimate_dps(&mut self) -> f32 {
        if let Some(cached) = self.cached_dps {
            return cached;
        }
        if let Some(dps) = self.spawn_unit_dps {
            self.cached_dps = Some(dps);
            return dps;
        }
        if let Some(dps) = self.despawn_unit_dps {
            self.cached_dps = Some(dps);
            return dps;
        }

        let pierce_multiplier = if self.pierce {
            if self.pierce_cap == -1 {
                2.0
            } else {
                self.pierce_cap.clamp(1, 2) as f32
            }
        } else {
            1.0
        };
        let mut sum = (self.damage + self.splash_damage * 0.75) * pierce_multiplier;
        if let Some(frag_dps) = self.frag_bullet_dps {
            sum += frag_dps * self.frag_bullets as f32 / 2.0;
        }
        self.cached_dps = Some(sum);
        sum
    }

    pub fn heals(&self) -> bool {
        self.heal_percent > 0.0 || self.heal_amount > 0.0
    }

    pub fn building_damage(&self, bullet_damage: f32) -> f32 {
        bullet_damage * self.building_damage_multiplier
    }

    pub fn shield_damage(&self, bullet_damage: f32) -> f32 {
        bullet_damage * self.shield_damage_multiplier
    }

    pub fn create_plan(
        &self,
        angle: f32,
        angle_offset: f32,
        random_angle_offset: f32,
        damage_override: Option<f32>,
        velocity_scale: f32,
        lifetime_scale: f32,
        ignore_spawn_angle: bool,
    ) -> BulletCreatePlan {
        let angle = if ignore_spawn_angle {
            0.0
        } else {
            angle + angle_offset + random_angle_offset
        };
        let damage = damage_override.unwrap_or(self.damage);
        BulletCreatePlan {
            angle,
            damage,
            velocity_scale,
            lifetime_scale,
            lifetime: self.lifetime * lifetime_scale,
            speed: self.speed * velocity_scale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bullet_range_matches_drag_override_and_unit_range_rules() {
        let mut bullet = BulletType {
            speed: 3.0,
            lifetime: 10.0,
            ..Default::default()
        };
        assert_eq!(bullet.calculate_range(), 30.0);

        bullet.drag = 0.1;
        let dragged = bullet.calculate_range();
        assert!(dragged < 30.0);

        bullet.range_override = 50.0;
        assert_eq!(bullet.calculate_range(), 50.0);

        bullet.range_override = -1.0;
        bullet.spawn_unit_range = Some(80.0);
        assert_eq!(bullet.calculate_range(), 80.0);
    }

    #[test]
    fn bullet_init_sets_pierce_despawn_hit_light_and_range_defaults() {
        let mut bullet = BulletType {
            pierce_cap: 2,
            splash_damage_radius: 12.0,
            trail_length: 20,
            speed: 2.0,
            hit_size: 5.0,
            ..Default::default()
        };

        bullet.init_defaults();

        assert!(bullet.pierce);
        assert!(bullet.despawn_hit);
        assert_eq!(bullet.light_radius, 25.0);
        assert_eq!(bullet.draw_size, 80.0);
        assert_eq!(bullet.range, 80.0);
    }

    #[test]
    fn bullet_estimate_dps_uses_splash_pierce_and_frags() {
        let mut bullet = BulletType {
            damage: 10.0,
            splash_damage: 8.0,
            pierce: true,
            pierce_cap: 2,
            frag_bullet_dps: Some(4.0),
            frag_bullets: 6,
            ..Default::default()
        };

        assert_eq!(bullet.estimate_dps(), (10.0 + 8.0 * 0.75) * 2.0 + 12.0);
        bullet.damage = 100.0;
        assert_eq!(bullet.estimate_dps(), 44.0);
    }

    #[test]
    fn bullet_damage_helpers_and_create_plan_are_pure() {
        let bullet = BulletType {
            damage: 7.0,
            speed: 3.0,
            lifetime: 20.0,
            building_damage_multiplier: 0.5,
            shield_damage_multiplier: 2.0,
            heal_amount: 1.0,
            ..Default::default()
        };

        assert!(bullet.heals());
        assert_eq!(bullet.building_damage(10.0), 5.0);
        assert_eq!(bullet.shield_damage(10.0), 20.0);

        let plan = bullet.create_plan(30.0, 5.0, -2.0, None, 2.0, 0.5, false);
        assert_eq!(plan.angle, 33.0);
        assert_eq!(plan.damage, 7.0);
        assert_eq!(plan.speed, 6.0);
        assert_eq!(plan.lifetime, 10.0);

        let ignored = bullet.create_plan(30.0, 5.0, -2.0, Some(9.0), 1.0, 1.0, true);
        assert_eq!(ignored.angle, 0.0);
        assert_eq!(ignored.damage, 9.0);
    }

    #[test]
    fn empty_and_explosion_bullets_apply_upstream_constructor_defaults() {
        let empty = empty_bullet_type();
        assert!(!empty.hittable);
        assert!(!empty.collides_ground);
        assert!(!empty.collides_air);
        assert!(!empty.collides_tiles);
        assert_eq!(empty.speed, 0.0);
        assert!(!empty.keep_velocity);

        let explosion = explosion_bullet_type(100.0, 90.0);
        assert_eq!(explosion.splash_damage, 100.0);
        assert_eq!(explosion.splash_damage_radius, 90.0);
        assert_eq!(explosion.range_override, 60.0);
        assert!(explosion.instant_disappear);
        assert!(explosion.scaled_splash_damage);
        assert!(explosion.kill_shooter);
        assert!(!explosion.collides);
    }

    #[test]
    fn fire_bullet_plans_velocity_radius_and_update_effects() {
        let fire = FireBulletType::new(2.0, 5.0);
        assert!(fire.base.pierce);
        assert!(!fire.base.collides_tiles);
        assert!(!fire.base.collides);
        assert_eq!(fire.base.drag, 0.03);
        assert_eq!(fire.initial_velocity_len(9.0), fire.vel_max);
        assert_eq!(fire.initial_velocity_len(0.1), fire.vel_min);
        assert_eq!(fire.draw_radius(0.5), 1.5);

        let plan = fire.update_plan(true, false, true);
        assert!(plan.create_fire);
        assert!(!plan.trail_effect);
        assert!(plan.secondary_trail_effect);
    }

    #[test]
    fn lightning_bullet_range_dps_and_length_match_overrides() {
        let mut lightning = LightningBulletType::default();
        lightning.base.damage = 4.0;
        lightning.base.lightning_length = 25;
        lightning.base.lightning_length_rand = 6;

        assert_eq!(lightning.calculate_range(), 168.0);
        assert_eq!(lightning.estimate_dps(), 10.0);
        assert_eq!(lightning.lightning_length(4), 29);
        assert_eq!(lightning.lightning_length(99), 31);
    }
}
