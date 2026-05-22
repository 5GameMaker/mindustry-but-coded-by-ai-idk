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
    pub reflectable: bool,
    pub collides_tiles: bool,
    pub collides_ground: bool,
    pub collides_air: bool,
    pub collides: bool,
    pub keep_velocity: bool,
    pub scale_life: bool,
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
    pub trail_chance: f32,
    pub trail_rotation: bool,
    pub homing_power: f32,
    pub hit_shake: f32,
    pub light_radius: f32,
    pub light_opacity: f32,
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
            reflectable: true,
            collides_tiles: true,
            collides_ground: true,
            collides_air: true,
            collides: true,
            keep_velocity: true,
            scale_life: false,
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
            trail_chance: -0.0001,
            trail_rotation: false,
            homing_power: 0.0,
            hit_shake: 0.0,
            light_radius: -1.0,
            light_opacity: 0.0,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BasicBulletDrawPlan {
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub mix_t: f32,
    pub draw_back: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicBulletType {
    pub base: BulletType,
    pub width: f32,
    pub height: f32,
    pub shrink_x: f32,
    pub shrink_y: f32,
    pub spin: f32,
    pub rotation_offset: f32,
    pub sprite: String,
    pub back_sprite: Option<String>,
}

impl Default for BasicBulletType {
    fn default() -> Self {
        Self::new(1.0, 1.0, "bullet")
    }
}

impl BasicBulletType {
    pub fn new(speed: f32, damage: f32, sprite: impl Into<String>) -> Self {
        Self {
            base: BulletType {
                speed,
                damage,
                ..Default::default()
            },
            width: 5.0,
            height: 7.0,
            shrink_x: 0.0,
            shrink_y: 0.5,
            spin: 0.0,
            rotation_offset: 0.0,
            sprite: sprite.into(),
            back_sprite: None,
        }
    }

    pub fn default_with_sprite(speed: f32, damage: f32) -> Self {
        Self::new(speed, damage, "bullet")
    }

    pub fn back_region_name(&self) -> String {
        self.back_sprite
            .clone()
            .unwrap_or_else(|| format!("{}-back", self.sprite))
    }

    pub fn draw_plan(
        &self,
        fin: f32,
        fout: f32,
        bullet_rotation: f32,
        bullet_time: f32,
        spin_seed_degrees: f32,
        back_region_found: bool,
    ) -> BasicBulletDrawPlan {
        let shrink = fout.clamp(0.0, 1.0);
        let height = self.height * ((1.0 - self.shrink_y) + self.shrink_y * shrink);
        let width = self.width * ((1.0 - self.shrink_x) + self.shrink_x * shrink);
        let offset = -90.0
            + if self.spin != 0.0 {
                spin_seed_degrees + bullet_time * self.spin
            } else {
                0.0
            }
            + self.rotation_offset;

        BasicBulletDrawPlan {
            width,
            height,
            rotation: bullet_rotation + offset,
            mix_t: fin.clamp(0.0, 1.0),
            draw_back: back_region_found,
        }
    }
}

pub fn bomb_bullet_type(damage: f32, radius: f32, sprite: impl Into<String>) -> BasicBulletType {
    let mut bullet = BasicBulletType::new(0.7, 0.0, sprite);
    bullet.base.splash_damage_radius = radius;
    bullet.base.splash_damage = damage;
    bullet.base.collides_tiles = false;
    bullet.base.collides = false;
    bullet.shrink_y = 0.7;
    bullet.base.lifetime = 30.0;
    bullet.base.drag = 0.05;
    bullet.base.keep_velocity = false;
    bullet.base.collides_air = false;
    bullet
}

pub fn missile_bullet_type(speed: f32, damage: f32, sprite: impl Into<String>) -> BasicBulletType {
    let mut bullet = BasicBulletType::new(speed, damage, sprite);
    bullet.base.homing_power = 0.08;
    bullet.shrink_y = 0.0;
    bullet.width = 8.0;
    bullet.height = 8.0;
    bullet.base.trail_chance = 0.2;
    bullet.base.lifetime = 52.0;
    bullet
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaserBoltDrawPlan {
    pub line_width: f32,
    pub back_length: f32,
    pub front_length: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaserBoltBulletType {
    pub base: BasicBulletType,
    pub line_width: f32,
    pub line_height: f32,
}

impl Default for LaserBoltBulletType {
    fn default() -> Self {
        Self::new(1.0, 1.0)
    }
}

impl LaserBoltBulletType {
    pub fn new(speed: f32, damage: f32) -> Self {
        let mut base = BasicBulletType::default_with_sprite(speed, damage);
        base.base.hittable = false;
        base.base.reflectable = false;
        base.base.light_opacity = 0.6;
        Self {
            base,
            line_width: 2.0,
            line_height: 7.0,
        }
    }

    pub fn draw_plan(&self, bullet_rotation: f32) -> LaserBoltDrawPlan {
        LaserBoltDrawPlan {
            line_width: self.line_width,
            back_length: self.line_height,
            front_length: self.line_height / 2.0,
            rotation: bullet_rotation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArtilleryTrailPlan {
    pub interval: f32,
    pub param: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArtilleryBulletType {
    pub base: BasicBulletType,
    pub trail_mult: f32,
    pub trail_size: f32,
}

impl Default for ArtilleryBulletType {
    fn default() -> Self {
        Self::new(1.0, 1.0, "shell")
    }
}

impl ArtilleryBulletType {
    pub fn new(speed: f32, damage: f32, sprite: impl Into<String>) -> Self {
        let mut base = BasicBulletType::new(speed, damage, sprite);
        base.base.collides_tiles = false;
        base.base.collides = false;
        base.base.collides_air = false;
        base.base.scale_life = true;
        base.base.hit_shake = 1.0;
        base.shrink_x = 0.15;
        base.shrink_y = 0.5;
        Self {
            base,
            trail_mult: 1.0,
            trail_size: 4.0,
        }
    }

    pub fn trail_plan(&self, fslope: f32, rotation: f32) -> ArtilleryTrailPlan {
        ArtilleryTrailPlan {
            interval: (3.0 + fslope * 2.0) * self.trail_mult,
            param: if self.base.base.trail_rotation {
                rotation
            } else {
                fslope * self.trail_size
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlakUpdatePlan {
    pub prime: bool,
    pub explode_delay: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlakBulletType {
    pub base: BasicBulletType,
    pub explode_range: f32,
    pub explode_delay: f32,
    pub flak_delay: f32,
    pub flak_interval: f32,
}

impl Default for FlakBulletType {
    fn default() -> Self {
        Self::new(1.0, 1.0)
    }
}

impl FlakBulletType {
    pub fn new(speed: f32, damage: f32) -> Self {
        let mut base = BasicBulletType::new(speed, damage, "shell");
        base.base.splash_damage = 15.0;
        base.base.splash_damage_radius = 34.0;
        base.width = 8.0;
        base.height = 10.0;
        base.base.collides_ground = false;
        Self {
            base,
            explode_range: 30.0,
            explode_delay: 5.0,
            flak_delay: 0.0,
            flak_interval: 6.0,
        }
    }

    pub fn update_plan(
        &self,
        bullet_time: f32,
        fdata: f32,
        timer_ready: bool,
        target_within_range: bool,
    ) -> FlakUpdatePlan {
        let prime =
            bullet_time >= self.flak_delay && fdata >= 0.0 && timer_ready && target_within_range;
        FlakUpdatePlan {
            prime,
            explode_delay: self.explode_delay,
        }
    }

    pub fn target_radius(&self, unit_hit_size: f32) -> f32 {
        self.explode_range + unit_hit_size / 2.0
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

    #[test]
    fn basic_bullet_draw_plan_matches_shrink_spin_and_sprite_rules() {
        let mut basic = BasicBulletType::new(3.0, 9.0, "shell");
        basic.width = 10.0;
        basic.height = 20.0;
        basic.shrink_x = 0.5;
        basic.shrink_y = 0.25;
        basic.spin = 2.0;
        basic.rotation_offset = 5.0;

        assert_eq!(basic.base.speed, 3.0);
        assert_eq!(basic.base.damage, 9.0);
        assert_eq!(basic.back_region_name(), "shell-back");

        let plan = basic.draw_plan(0.25, 0.5, 45.0, 10.0, 30.0, true);
        assert_eq!(plan.width, 7.5);
        assert_eq!(plan.height, 17.5);
        assert_eq!(plan.rotation, 10.0);
        assert_eq!(plan.mix_t, 0.25);
        assert!(plan.draw_back);

        basic.back_sprite = Some("custom-back".into());
        assert_eq!(basic.back_region_name(), "custom-back");
    }

    #[test]
    fn bomb_and_missile_bullet_constructors_match_upstream_presets() {
        let bomb = bomb_bullet_type(60.0, 24.0, "shell");
        assert_eq!(bomb.base.speed, 0.7);
        assert_eq!(bomb.base.damage, 0.0);
        assert_eq!(bomb.base.splash_damage, 60.0);
        assert_eq!(bomb.base.splash_damage_radius, 24.0);
        assert!(!bomb.base.collides_tiles);
        assert!(!bomb.base.collides);
        assert!(!bomb.base.keep_velocity);
        assert!(!bomb.base.collides_air);
        assert_eq!(bomb.shrink_y, 0.7);
        assert_eq!(bomb.base.lifetime, 30.0);
        assert_eq!(bomb.base.drag, 0.05);

        let missile = missile_bullet_type(3.0, 10.0, "missile");
        assert_eq!(missile.base.speed, 3.0);
        assert_eq!(missile.base.damage, 10.0);
        assert_eq!(missile.base.homing_power, 0.08);
        assert_eq!(missile.shrink_y, 0.0);
        assert_eq!((missile.width, missile.height), (8.0, 8.0));
        assert_eq!(missile.base.trail_chance, 0.2);
        assert_eq!(missile.base.lifetime, 52.0);
    }

    #[test]
    fn laser_bolt_constructor_and_draw_plan_cover_line_overlay() {
        let laser = LaserBoltBulletType::new(5.0, 12.0);
        assert_eq!(laser.base.base.speed, 5.0);
        assert_eq!(laser.base.base.damage, 12.0);
        assert!(!laser.base.base.hittable);
        assert!(!laser.base.base.reflectable);
        assert_eq!(laser.base.base.light_opacity, 0.6);

        let plan = laser.draw_plan(45.0);
        assert_eq!(plan.line_width, 2.0);
        assert_eq!(plan.back_length, 7.0);
        assert_eq!(plan.front_length, 3.5);
        assert_eq!(plan.rotation, 45.0);
    }

    #[test]
    fn artillery_bullet_constructor_and_trail_plan_match_update_formula() {
        let mut artillery = ArtilleryBulletType::new(2.5, 20.0, "shell");
        assert!(!artillery.base.base.collides_tiles);
        assert!(!artillery.base.base.collides);
        assert!(!artillery.base.base.collides_air);
        assert!(artillery.base.base.scale_life);
        assert_eq!(artillery.base.base.hit_shake, 1.0);
        assert_eq!(artillery.base.shrink_x, 0.15);
        assert_eq!(artillery.base.shrink_y, 0.5);

        artillery.trail_mult = 2.0;
        artillery.trail_size = 5.0;
        let plan = artillery.trail_plan(0.5, 90.0);
        assert_eq!(plan.interval, 8.0);
        assert_eq!(plan.param, 2.5);

        artillery.base.base.trail_rotation = true;
        assert_eq!(artillery.trail_plan(0.5, 90.0).param, 90.0);
    }

    #[test]
    fn flak_bullet_primes_only_after_delay_timer_and_target_match() {
        let mut flak = FlakBulletType::new(3.0, 7.0);
        flak.flak_delay = 5.0;
        assert_eq!(flak.base.base.splash_damage, 15.0);
        assert_eq!(flak.base.base.splash_damage_radius, 34.0);
        assert_eq!((flak.base.width, flak.base.height), (8.0, 10.0));
        assert!(!flak.base.base.collides_ground);
        assert_eq!(flak.target_radius(12.0), 36.0);

        assert!(!flak.update_plan(0.0, 0.0, true, true).prime);
        assert!(!flak.update_plan(10.0, -1.0, true, true).prime);
        assert!(!flak.update_plan(10.0, 0.0, false, true).prime);
        assert!(!flak.update_plan(10.0, 0.0, true, false).prime);

        let plan = flak.update_plan(10.0, 0.0, true, true);
        assert!(plan.prime);
        assert_eq!(plan.explode_delay, 5.0);
    }
}
