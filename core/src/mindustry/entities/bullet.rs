use crate::mindustry::r#type::{Item, Liquid};

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
    pub absorbable: bool,
    pub collides_tiles: bool,
    pub collides_ground: bool,
    pub collides_air: bool,
    pub collides: bool,
    pub keep_velocity: bool,
    pub scale_life: bool,
    pub instant_disappear: bool,
    pub kill_shooter: bool,
    pub scaled_splash_damage: bool,
    pub impact: bool,
    pub status: String,
    pub status_duration: f32,
    pub pierce: bool,
    pub pierce_building: bool,
    pub remove_after_pierce: bool,
    pub laser_absorb: bool,
    pub optimal_life_fract: f32,
    pub ammo_multiplier: f32,
    pub pierce_armor: bool,
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
            absorbable: true,
            collides_tiles: true,
            collides_ground: true,
            collides_air: true,
            collides: true,
            keep_velocity: true,
            scale_life: false,
            instant_disappear: false,
            kill_shooter: false,
            scaled_splash_damage: false,
            impact: false,
            status: "none".into(),
            status_duration: 60.0 * 8.0,
            pierce: false,
            pierce_building: false,
            remove_after_pierce: true,
            laser_absorb: true,
            optimal_life_fract: 0.0,
            ammo_multiplier: 2.0,
            pierce_armor: false,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmpFriendlyBuildingPlan {
    pub apply_boost: bool,
    pub boost_scale: f32,
    pub boost_duration: f32,
    pub heal_amount: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmpEnemyPowerPlan {
    pub apply_slowdown: bool,
    pub slowdown_scale: f32,
    pub slowdown_duration: f32,
    pub damage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmpUnitHitPlan {
    pub damage: f32,
    pub apply_status: bool,
    pub status_duration: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmpBulletType {
    pub base: BasicBulletType,
    pub radius: f32,
    pub time_increase: f32,
    pub time_duration: f32,
    pub power_damage_scl: f32,
    pub power_scl_decrease: f32,
    pub hit_power_effect: String,
    pub chain_effect: String,
    pub apply_effect: String,
    pub hit_units: bool,
    pub unit_damage_scl: f32,
}

impl Default for EmpBulletType {
    fn default() -> Self {
        Self {
            base: BasicBulletType::default(),
            radius: 100.0,
            time_increase: 2.5,
            time_duration: 60.0 * 10.0,
            power_damage_scl: 2.0,
            power_scl_decrease: 0.2,
            hit_power_effect: "hitEmpSpark".into(),
            chain_effect: "chainEmp".into(),
            apply_effect: "heal".into(),
            hit_units: true,
            unit_damage_scl: 0.7,
        }
    }
}

impl EmpBulletType {
    pub fn new(speed: f32, damage: f32, sprite: impl Into<String>) -> Self {
        let mut out = Self::default();
        out.base = BasicBulletType::new(speed, damage, sprite);
        out
    }

    pub fn friendly_building_plan(
        &self,
        has_power: bool,
        can_overdrive: bool,
        time_scale: f32,
        damaged: bool,
        max_health: f32,
    ) -> EmpFriendlyBuildingPlan {
        EmpFriendlyBuildingPlan {
            apply_boost: has_power && can_overdrive && time_scale < self.time_increase,
            boost_scale: self.time_increase,
            boost_duration: self.time_duration,
            heal_amount: if has_power && damaged {
                self.base.base.heal_percent / 100.0 * max_health + self.base.base.heal_amount
            } else {
                0.0
            },
        }
    }

    pub fn enemy_power_plan(
        &self,
        has_power_graph: bool,
        last_power_produced: f32,
    ) -> Option<EmpEnemyPowerPlan> {
        (has_power_graph && last_power_produced > 0.0).then_some(EmpEnemyPowerPlan {
            apply_slowdown: true,
            slowdown_scale: self.power_scl_decrease,
            slowdown_duration: self.time_duration,
            damage: self.base.base.damage * self.power_damage_scl,
        })
    }

    pub fn enemy_unit_plan(
        &self,
        hittable: bool,
        absorbed_by_shield: bool,
    ) -> Option<EmpUnitHitPlan> {
        (self.hit_units && hittable && !absorbed_by_shield).then_some(EmpUnitHitPlan {
            damage: self.base.base.damage * self.unit_damage_scl,
            apply_status: self.base.base.status != "none",
            status_duration: self.base.base.status_duration,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MassDriverUpdatePlan {
    pub hit: bool,
    pub keep_flying: bool,
    pub snap_position: Option<(f32, f32)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MassDriverDropPlan {
    pub item_index: usize,
    pub amount_dropped: i32,
    pub angle: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MassDriverExplosionPlan {
    pub flammability: f32,
    pub explosiveness: f32,
    pub power: f32,
    pub radius_scl: f32,
    pub damage_explosions: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MassDriverBolt {
    pub base: BasicBulletType,
    pub despawn_effect: String,
    pub hit_effect: String,
}

impl Default for MassDriverBolt {
    fn default() -> Self {
        let mut base = BasicBulletType::new(1.0, 75.0, "shell");
        base.base.collides_tiles = false;
        base.base.lifetime = 1.0;
        base.width = 11.0;
        base.height = 13.0;
        base.shrink_y = 0.0;
        Self {
            base,
            despawn_effect: "smeltsmoke".into(),
            hit_effect: "hitBulletBig".into(),
        }
    }
}

impl MassDriverBolt {
    pub fn invalid_data_plan(&self) -> MassDriverUpdatePlan {
        MassDriverUpdatePlan {
            hit: true,
            keep_flying: false,
            snap_position: None,
        }
    }

    pub fn update_plan(
        &self,
        bullet_x: f32,
        bullet_y: f32,
        from_x: f32,
        from_y: f32,
        to_x: f32,
        to_y: f32,
        target_dead: bool,
    ) -> MassDriverUpdatePlan {
        if target_dead {
            return MassDriverUpdatePlan {
                hit: false,
                keep_flying: true,
                snap_position: None,
            };
        }

        let hit_dst = 7.0;
        let base_dst = dst(from_x, from_y, to_x, to_y);
        let dst1 = dst(bullet_x, bullet_y, from_x, from_y);
        let dst2 = dst(bullet_x, bullet_y, to_x, to_y);
        let mut snap_position = None;
        let mut hit = false;

        if dst1 > base_dst {
            let angle_to_target = angle_to_degrees(bullet_x, bullet_y, to_x, to_y);
            let base_angle = angle_to_degrees(to_x, to_y, from_x, from_y);
            if angle_near(angle_to_target, base_angle, 2.0) {
                hit = true;
                snap_position = Some((
                    to_x + trnsx(base_angle, hit_dst),
                    to_y + trnsy(base_angle, hit_dst),
                ));
            }
        }

        if (dst1 + dst2 - base_dst).abs() < 4.0 && dst2 <= hit_dst {
            hit = true;
        }

        MassDriverUpdatePlan {
            hit,
            keep_flying: false,
            snap_position,
        }
    }

    pub fn despawn_drop_plans(
        &self,
        item_amounts: &[i32],
        random_amounts: &[i32],
        bullet_rotation: f32,
        random_angle_offsets: &[f32],
    ) -> Vec<MassDriverDropPlan> {
        item_amounts
            .iter()
            .enumerate()
            .filter_map(|(index, amount)| {
                let random_amount = random_amounts.get(index).copied().unwrap_or(0);
                let amount_dropped = random_amount.clamp(0, (*amount).max(0));
                (amount_dropped > 0).then(|| MassDriverDropPlan {
                    item_index: index,
                    amount_dropped,
                    angle: bullet_rotation
                        + random_angle_offsets.get(index).copied().unwrap_or(0.0),
                })
            })
            .collect()
    }

    pub fn dynamic_explosion_plan(
        &self,
        items: &[Item],
        item_amounts: &[i32],
        damage_explosions: bool,
    ) -> MassDriverExplosionPlan {
        let mut explosiveness = 0.0;
        let mut flammability = 0.0;
        let mut power = 0.0;

        for (index, amount) in item_amounts.iter().copied().enumerate() {
            let Some(item) = items.get(index) else {
                continue;
            };
            let amount = amount.max(0) as f32;
            explosiveness += item.explosiveness * amount;
            flammability += item.flammability * amount;
            power += item.charge * amount.powf(1.1) * 25.0;
        }

        MassDriverExplosionPlan {
            flammability: flammability / 10.0,
            explosiveness: explosiveness / 10.0,
            power,
            radius_scl: 1.0,
            damage_explosions,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidUpdatePlan {
    pub vaporize: bool,
    pub extinguish_fire: bool,
    pub extinguish_intensity: f32,
    pub remove: bool,
    pub hit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidDrawPlan {
    pub color_rgba: u32,
    pub mix_color_rgba: u32,
    pub mix: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidHitPlan {
    pub hit_effect: String,
    pub deposit_puddle: bool,
    pub puddle_size: f32,
    pub extinguish_intensity: Option<f32>,
    pub extinguish_offsets: Vec<(i32, i32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidBulletType {
    pub base: BulletType,
    pub liquid: Option<Liquid>,
    pub puddle_size: f32,
    pub orb_size: f32,
    pub boil_time: f32,
    pub hit_color_rgba: u32,
    pub light_color_rgba: u32,
    pub despawn_effect: String,
    pub hit_effect: String,
    pub smoke_effect: String,
    pub shoot_effect: String,
    pub knockback: f32,
    pub display_ammo_multiplier: bool,
}

impl Default for LiquidBulletType {
    fn default() -> Self {
        Self::new(None)
    }
}

impl LiquidBulletType {
    pub fn new(liquid: Option<Liquid>) -> Self {
        let mut base = BulletType {
            speed: 3.5,
            damage: 0.0,
            ammo_multiplier: 1.0,
            lifetime: 34.0,
            status_duration: 60.0 * 2.0,
            drag: 0.001,
            ..Default::default()
        };
        let mut hit_color_rgba = 0xffff_ffff;
        let mut light_color_rgba = 0x0000_0000;
        if let Some(liquid) = liquid.as_ref() {
            base.status = liquid.effect.clone().unwrap_or_else(|| "none".into());
            hit_color_rgba = liquid.color_rgba;
            light_color_rgba = liquid.light_color_rgba;
            base.light_opacity = rgba_alpha(light_color_rgba);
        }

        Self {
            base,
            liquid,
            puddle_size: 6.0,
            orb_size: 3.0,
            boil_time: 5.0,
            hit_color_rgba,
            light_color_rgba,
            despawn_effect: "none".into(),
            hit_effect: "hitLiquid".into(),
            smoke_effect: "none".into(),
            shoot_effect: "none".into(),
            knockback: 0.55,
            display_ammo_multiplier: false,
        }
    }

    pub fn update_plan(
        &self,
        heat_env: f32,
        time: f32,
        random_boil_time: f32,
        tile_has_fire: bool,
    ) -> LiquidUpdatePlan {
        let Some(liquid) = self.liquid.as_ref() else {
            return LiquidUpdatePlan {
                vaporize: false,
                extinguish_fire: false,
                extinguish_intensity: 0.0,
                remove: false,
                hit: false,
            };
        };

        if liquid.will_boil(heat_env) && time >= random_boil_time {
            return LiquidUpdatePlan {
                vaporize: true,
                extinguish_fire: false,
                extinguish_intensity: 0.0,
                remove: true,
                hit: false,
            };
        }

        if liquid.can_extinguish() && tile_has_fire {
            return LiquidUpdatePlan {
                vaporize: false,
                extinguish_fire: true,
                extinguish_intensity: 100.0,
                remove: true,
                hit: true,
            };
        }

        LiquidUpdatePlan {
            vaporize: false,
            extinguish_fire: false,
            extinguish_intensity: 0.0,
            remove: false,
            hit: false,
        }
    }

    pub fn draw_plan(
        &self,
        heat_env: f32,
        time: f32,
        random_boil_time: f32,
        fin: f32,
        fout: f32,
    ) -> Option<LiquidDrawPlan> {
        let liquid = self.liquid.as_ref()?;
        if liquid.will_boil(heat_env) {
            Some(LiquidDrawPlan {
                color_rgba: liquid.color_rgba,
                mix_color_rgba: with_rgba_alpha(liquid.gas_color_rgba, 0x66),
                mix: safe_div(time, random_boil_time),
                radius: self.orb_size * (fin * 1.1 + 1.0),
            })
        } else {
            Some(LiquidDrawPlan {
                color_rgba: liquid.color_rgba,
                mix_color_rgba: 0xffff_ffff,
                mix: fout / 100.0,
                radius: self.orb_size,
            })
        }
    }

    pub fn despawn_effect_plan(&self, heat_env: f32) -> Option<String> {
        let liquid = self.liquid.as_ref()?;
        (!liquid.will_boil(heat_env)).then(|| self.hit_effect.clone())
    }

    pub fn hit_plan(&self) -> Option<LiquidHitPlan> {
        let liquid = self.liquid.as_ref()?;
        let extinguishes_nearby = liquid.temperature <= 0.5 && liquid.flammability < 0.3;
        Some(LiquidHitPlan {
            hit_effect: self.hit_effect.clone(),
            deposit_puddle: true,
            puddle_size: self.puddle_size,
            extinguish_intensity: extinguishes_nearby.then_some(400.0 * self.puddle_size / 6.0),
            extinguish_offsets: if extinguishes_nearby {
                vec![(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)]
            } else {
                Vec::new()
            },
        })
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterceptorHitPlan {
    pub hit_x: f32,
    pub hit_y: f32,
    pub other_damage_after: f32,
    pub remove_other: bool,
    pub remove_self: bool,
    pub clear_data: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterceptorBulletType {
    pub base: BasicBulletType,
}

impl Default for InterceptorBulletType {
    fn default() -> Self {
        Self {
            base: BasicBulletType::default(),
        }
    }
}

impl InterceptorBulletType {
    pub fn new(speed: f32, damage: f32, sprite: impl Into<String>) -> Self {
        Self {
            base: BasicBulletType::new(speed, damage, sprite),
        }
    }

    pub fn resolve_intercept(
        &self,
        other_added: bool,
        collision_point: Option<(f32, f32)>,
        other_damage: f32,
        self_damage: f32,
    ) -> Option<InterceptorHitPlan> {
        if !other_added {
            return Some(InterceptorHitPlan {
                hit_x: 0.0,
                hit_y: 0.0,
                other_damage_after: other_damage,
                remove_other: false,
                remove_self: false,
                clear_data: true,
            });
        }

        let (hit_x, hit_y) = collision_point?;
        let remove_other = other_damage <= self_damage;
        Some(InterceptorHitPlan {
            hit_x,
            hit_y,
            other_damage_after: if remove_other {
                0.0
            } else {
                other_damage - self_damage
            },
            remove_other,
            remove_self: true,
            clear_data: false,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointBulletType {
    pub base: BulletType,
    pub trail_spacing: f32,
}

impl Default for PointBulletType {
    fn default() -> Self {
        Self {
            base: BulletType {
                scale_life: true,
                lifetime: 100.0,
                collides: false,
                reflectable: false,
                keep_velocity: false,
                ..Default::default()
            },
            trail_spacing: 10.0,
        }
    }
}

impl PointBulletType {
    pub fn end_position(&self, x: f32, y: f32, vel_x: f32, vel_y: f32) -> (f32, f32) {
        (
            x + self.base.lifetime * vel_x,
            y + self.base.lifetime * vel_y,
        )
    }

    pub fn trail_points(&self, x: f32, y: f32, end_x: f32, end_y: f32) -> Vec<(f32, f32)> {
        let dx = end_x - x;
        let dy = end_y - y;
        let len = (dx * dx + dy * dy).sqrt();
        if len <= f32::EPSILON || self.trail_spacing <= 0.0 {
            return vec![(x, y)];
        }

        let steps = (len / self.trail_spacing).floor() as i32;
        (0..=steps)
            .map(|i| {
                let t = (i as f32 * self.trail_spacing / len).min(1.0);
                (x + dx * t, y + dy * t)
            })
            .chain(std::iter::once((end_x, end_y)))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpaceLiquidBulletType {
    pub base: BulletType,
    pub orb_size: f32,
}

impl Default for SpaceLiquidBulletType {
    fn default() -> Self {
        Self {
            base: BulletType {
                speed: 3.5,
                damage: 0.0,
                collides: false,
                lifetime: 90.0,
                drag: 0.002,
                hittable: false,
                ..Default::default()
            },
            orb_size: 5.5,
        }
    }
}

impl SpaceLiquidBulletType {
    pub fn draw_radius(&self, fslope: f32) -> f32 {
        pow3_out(fslope.clamp(0.0, 1.0)) * self.orb_size
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLaserUpdatePlan {
    pub collide_point: bool,
    pub beam_effect: bool,
    pub shake: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointLaserBulletType {
    pub base: BulletType,
    pub sprite: String,
    pub beam_effect_interval: f32,
    pub beam_effect_size: f32,
    pub osc_scl: f32,
    pub osc_mag: f32,
    pub damage_interval: f32,
    pub shake: f32,
}

impl Default for PointLaserBulletType {
    fn default() -> Self {
        Self {
            base: BulletType {
                remove_after_pierce: false,
                speed: 0.0,
                lifetime: 20.0,
                impact: true,
                keep_velocity: false,
                collides: false,
                pierce: true,
                hittable: false,
                absorbable: false,
                optimal_life_fract: 0.5,
                draw_size: 1000.0,
                ..Default::default()
            },
            sprite: "point-laser".into(),
            beam_effect_interval: 3.0,
            beam_effect_size: 3.5,
            osc_scl: 2.0,
            osc_mag: 0.3,
            damage_interval: 5.0,
            shake: 0.0,
        }
    }
}

impl PointLaserBulletType {
    pub fn continuous_damage(&self) -> f32 {
        self.base.damage / self.damage_interval * 60.0
    }

    pub fn estimate_dps(&self) -> f32 {
        self.base.damage * 100.0 / self.damage_interval * 3.0
    }

    pub fn laser_width_scale(&self, fslope: f32, absin: f32) -> f32 {
        fslope * (1.0 - self.osc_mag + absin)
    }

    pub fn update_plan(&self, damage_timer: bool, beam_timer: bool) -> PointLaserUpdatePlan {
        PointLaserUpdatePlan {
            collide_point: damage_timer,
            beam_effect: beam_timer,
            shake: self.shake,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContinuousDamagePlan {
    pub length: f32,
    pub damage: f32,
    pub large_hit: bool,
    pub laser_absorb: bool,
    pub pierce_cap: i32,
    pub shake: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContinuousBulletType {
    pub base: BulletType,
    pub length: f32,
    pub shake: f32,
    pub damage_interval: f32,
    pub large_hit: bool,
    pub continuous: bool,
    pub timescale_damage: bool,
}

impl Default for ContinuousBulletType {
    fn default() -> Self {
        Self {
            base: BulletType {
                remove_after_pierce: false,
                pierce_cap: -1,
                speed: 0.0,
                lifetime: 16.0,
                impact: true,
                keep_velocity: false,
                collides: false,
                pierce: true,
                hittable: false,
                absorbable: false,
                ..Default::default()
            },
            length: 220.0,
            shake: 0.0,
            damage_interval: 5.0,
            large_hit: false,
            continuous: true,
            timescale_damage: false,
        }
    }
}

impl ContinuousBulletType {
    pub fn continuous_damage(&self) -> f32 {
        if self.continuous {
            self.base.damage / self.damage_interval * 60.0
        } else {
            -1.0
        }
    }

    pub fn estimate_dps(&mut self) -> f32 {
        if self.continuous {
            self.base.damage * 100.0 / self.damage_interval * 3.0
        } else {
            self.base.estimate_dps()
        }
    }

    pub fn calculate_range(&self) -> f32 {
        self.length.max(self.base.max_range)
    }

    pub fn init_defaults(&mut self) {
        self.base.init_defaults();
        self.base.draw_size = self.base.draw_size.max(self.length * 2.0);
    }

    pub fn current_length(&self) -> f32 {
        self.length
    }

    pub fn damage_plan(&self, owner_timescale: f32) -> ContinuousDamagePlan {
        ContinuousDamagePlan {
            length: self.current_length(),
            damage: if self.timescale_damage {
                self.base.damage * owner_timescale
            } else {
                self.base.damage
            },
            large_hit: self.large_hit,
            laser_absorb: self.base.laser_absorb,
            pierce_cap: self.base.pierce_cap,
            shake: self.shake,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContinuousLaserBulletType {
    pub base: ContinuousBulletType,
    pub fade_time: f32,
    pub light_stroke: f32,
    pub divisions: i32,
    pub stroke_from: f32,
    pub stroke_to: f32,
    pub pointy_scaling: f32,
    pub back_length: f32,
    pub front_length: f32,
    pub width: f32,
    pub osc_scl: f32,
    pub osc_mag: f32,
}

impl Default for ContinuousLaserBulletType {
    fn default() -> Self {
        let mut base = ContinuousBulletType::default();
        base.shake = 1.0;
        base.large_hit = true;
        base.base.hit_size = 4.0;
        base.base.draw_size = 420.0;
        base.base.lifetime = 16.0;
        base.base.light_opacity = 0.7;
        Self {
            base,
            fade_time: 16.0,
            light_stroke: 40.0,
            divisions: 13,
            stroke_from: 2.0,
            stroke_to: 0.5,
            pointy_scaling: 0.75,
            back_length: 7.0,
            front_length: 35.0,
            width: 9.0,
            osc_scl: 0.8,
            osc_mag: 1.5,
        }
    }
}

impl ContinuousLaserBulletType {
    pub fn new(damage: f32) -> Self {
        let mut out = Self::default();
        out.base.base.damage = damage;
        out
    }

    pub fn fade_out(&self, time: f32) -> f32 {
        if time > self.base.base.lifetime - self.fade_time {
            (1.0 - (time - (self.base.base.lifetime - self.fade_time)) / self.fade_time)
                .clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    pub fn current_length(&self, time: f32) -> f32 {
        self.base.length * self.fade_out(time)
    }

    pub fn stroke(&self, color_fin: f32, fout: f32, absin: f32) -> f32 {
        let base_stroke = self.stroke_from + (self.stroke_to - self.stroke_from) * color_fin;
        (self.width + absin) * fout * base_stroke
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContinuousFlameBulletType {
    pub base: ContinuousBulletType,
    pub light_stroke: f32,
    pub width: f32,
    pub osc_scl: f32,
    pub osc_mag: f32,
    pub divisions: i32,
    pub draw_flare: bool,
    pub flare_width: f32,
    pub flare_length: f32,
    pub flare_rot_speed: f32,
    pub rotate_flare: bool,
    pub length_width_pans: Vec<(f32, f32, f32)>,
}

impl Default for ContinuousFlameBulletType {
    fn default() -> Self {
        let mut base = ContinuousBulletType::default();
        base.base.optimal_life_fract = 0.5;
        base.length = 120.0;
        base.base.hit_size = 4.0;
        base.base.draw_size = 420.0;
        base.base.lifetime = 16.0;
        base.base.light_opacity = 0.7;
        base.base.laser_absorb = false;
        base.base.ammo_multiplier = 1.0;
        base.base.pierce_armor = true;
        Self {
            base,
            light_stroke: 40.0,
            width: 3.7,
            osc_scl: 1.2,
            osc_mag: 0.02,
            divisions: 25,
            draw_flare: true,
            flare_width: 3.0,
            flare_length: 40.0,
            flare_rot_speed: 1.2,
            rotate_flare: false,
            length_width_pans: vec![
                (1.12, 1.3, 0.32),
                (1.0, 1.0, 0.3),
                (0.8, 0.9, 0.2),
                (0.5, 0.8, 0.15),
                (0.25, 0.7, 0.1),
            ],
        }
    }
}

impl ContinuousFlameBulletType {
    pub fn new(damage: f32) -> Self {
        let mut out = Self::default();
        out.base.base.damage = damage;
        out
    }

    pub fn current_length(&self, fin_slope: f32) -> f32 {
        self.base.length * fin_slope
    }

    pub fn flame_segment(
        &self,
        index: usize,
        real_length: f32,
        mult: f32,
        sin: f32,
    ) -> (f32, f32, f32) {
        let (length_scl, width_scl, pan) = self.length_width_pans[index];
        (
            real_length * length_scl * (1.0 - sin),
            self.width * width_scl * mult * (1.0 + sin),
            pan,
        )
    }

    pub fn flare_angle(&self, time: f32, rotation: f32) -> f32 {
        time * self.flare_rot_speed + if self.rotate_flare { rotation } else { 0.0 }
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

fn dst(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

fn angle_to_degrees(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    (y2 - y1).atan2(x2 - x1).to_degrees()
}

fn angle_near(a: f32, b: f32, margin: f32) -> bool {
    let delta = (a - b + 180.0).rem_euclid(360.0) - 180.0;
    delta.abs() <= margin
}

fn trnsx(angle: f32, len: f32) -> f32 {
    angle.to_radians().cos() * len
}

fn trnsy(angle: f32, len: f32) -> f32 {
    angle.to_radians().sin() * len
}

fn rgba_alpha(rgba: u32) -> f32 {
    (rgba & 0xff) as f32 / 255.0
}

fn with_rgba_alpha(rgba: u32, alpha: u8) -> u32 {
    (rgba & 0xffff_ff00) | alpha as u32
}

fn safe_div(a: f32, b: f32) -> f32 {
    if b.abs() <= f32::EPSILON {
        0.0
    } else {
        a / b
    }
}

fn pow3_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
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
    fn emp_bullet_plans_building_power_and_unit_effects() {
        let mut emp = EmpBulletType::new(4.0, 30.0, "emp");
        emp.base.base.heal_percent = 5.0;
        emp.base.base.heal_amount = 10.0;
        emp.base.base.status = "shocked".into();

        assert_eq!(emp.base.base.speed, 4.0);
        assert_eq!(emp.base.base.damage, 30.0);
        assert_eq!(emp.radius, 100.0);
        assert_eq!(emp.time_increase, 2.5);
        assert_eq!(emp.time_duration, 600.0);
        assert_eq!(emp.power_damage_scl, 2.0);
        assert_eq!(emp.power_scl_decrease, 0.2);
        assert_eq!(emp.hit_power_effect, "hitEmpSpark");
        assert_eq!(emp.chain_effect, "chainEmp");
        assert_eq!(emp.apply_effect, "heal");
        assert!(emp.hit_units);
        assert_eq!(emp.unit_damage_scl, 0.7);

        let friendly = emp.friendly_building_plan(true, true, 1.0, true, 400.0);
        assert!(friendly.apply_boost);
        assert_eq!(friendly.boost_scale, 2.5);
        assert_eq!(friendly.boost_duration, 600.0);
        assert_eq!(friendly.heal_amount, 30.0);

        let enemy = emp
            .enemy_power_plan(true, 12.0)
            .expect("powered enemy building should be affected");
        assert!(enemy.apply_slowdown);
        assert_eq!(enemy.slowdown_scale, 0.2);
        assert_eq!(enemy.slowdown_duration, 600.0);
        assert_eq!(enemy.damage, 60.0);
        assert!(emp.enemy_power_plan(true, 0.0).is_none());

        let unit = emp
            .enemy_unit_plan(true, false)
            .expect("hittable enemy unit without absorber should be affected");
        assert_eq!(unit.damage, 21.0);
        assert!(unit.apply_status);
        assert_eq!(unit.status_duration, 480.0);
        assert!(emp.enemy_unit_plan(true, true).is_none());
    }

    #[test]
    fn mass_driver_bolt_intersection_drops_and_explosion_stats_are_pure() {
        let bolt = MassDriverBolt::default();
        assert_eq!(bolt.base.base.speed, 1.0);
        assert_eq!(bolt.base.base.damage, 75.0);
        assert!(!bolt.base.base.collides_tiles);
        assert_eq!(bolt.base.base.lifetime, 1.0);
        assert_eq!((bolt.base.width, bolt.base.height), (11.0, 13.0));
        assert_eq!(bolt.base.shrink_y, 0.0);
        assert_eq!(bolt.base.sprite, "shell");
        assert_eq!(bolt.despawn_effect, "smeltsmoke");
        assert_eq!(bolt.hit_effect, "hitBulletBig");

        assert!(bolt.invalid_data_plan().hit);
        let dead_target = bolt.update_plan(95.0, 0.0, 0.0, 0.0, 100.0, 0.0, true);
        assert!(!dead_target.hit);
        assert!(dead_target.keep_flying);

        let in_range = bolt.update_plan(95.0, 0.0, 0.0, 0.0, 100.0, 0.0, false);
        assert!(in_range.hit);
        assert_eq!(in_range.snap_position, None);

        let overshot = bolt.update_plan(108.0, 0.0, 0.0, 0.0, 100.0, 0.0, false);
        assert!(overshot.hit);
        let snap = overshot
            .snap_position
            .expect("overshot hit should snap back");
        assert!((snap.0 - 93.0).abs() < f32::EPSILON);
        assert!(snap.1.abs() < 0.0001);

        let drops = bolt.despawn_drop_plans(&[3, 0, 5], &[2, 4, 7], 10.0, &[1.0, 2.0, 3.0]);
        assert_eq!(
            drops,
            vec![
                MassDriverDropPlan {
                    item_index: 0,
                    amount_dropped: 2,
                    angle: 11.0,
                },
                MassDriverDropPlan {
                    item_index: 2,
                    amount_dropped: 5,
                    angle: 13.0,
                },
            ]
        );

        let mut coal = Item::new(0, "coal");
        coal.explosiveness = 0.2;
        coal.flammability = 0.4;
        coal.charge = 0.0;
        let mut thorium = Item::new(1, "thorium");
        thorium.explosiveness = 0.1;
        thorium.flammability = 0.0;
        thorium.charge = 0.5;
        let explosion = bolt.dynamic_explosion_plan(&[coal, thorium], &[4, 2], true);
        assert_eq!(explosion.flammability, 0.16);
        assert_eq!(explosion.explosiveness, 0.1);
        assert!((explosion.power - 0.5 * 2.0_f32.powf(1.1) * 25.0).abs() < 0.0001);
        assert_eq!(explosion.radius_scl, 1.0);
        assert!(explosion.damage_explosions);
    }

    #[test]
    fn liquid_bullet_plans_boil_extinguish_draw_and_puddle_effects() {
        let mut water = Liquid::new(0, "water");
        water.effect = Some("wet".into());
        water.color_rgba = 0x3366_ccff;
        water.gas_color_rgba = 0xaabb_ccff;
        water.light_color_rgba = 0x1122_3380;
        water.boil_point = 0.7;
        water.temperature = 0.5;
        water.flammability = 0.0;

        let liquid = LiquidBulletType::new(Some(water));
        assert_eq!(liquid.base.speed, 3.5);
        assert_eq!(liquid.base.damage, 0.0);
        assert_eq!(liquid.base.ammo_multiplier, 1.0);
        assert_eq!(liquid.base.lifetime, 34.0);
        assert_eq!(liquid.base.status, "wet");
        assert_eq!(liquid.base.status_duration, 120.0);
        assert_eq!(liquid.base.drag, 0.001);
        assert_eq!(liquid.puddle_size, 6.0);
        assert_eq!(liquid.orb_size, 3.0);
        assert_eq!(liquid.boil_time, 5.0);
        assert_eq!(liquid.hit_color_rgba, 0x3366_ccff);
        assert_eq!(liquid.light_color_rgba, 0x1122_3380);
        assert!((liquid.base.light_opacity - 128.0 / 255.0).abs() < 0.0001);
        assert_eq!(liquid.hit_effect, "hitLiquid");
        assert_eq!(liquid.knockback, 0.55);
        assert!(!liquid.display_ammo_multiplier);

        let boil = liquid.update_plan(1.0, 4.0, 3.5, true);
        assert!(boil.vaporize);
        assert!(boil.remove);
        assert!(!boil.hit);
        assert!(!boil.extinguish_fire);

        let extinguish = liquid.update_plan(0.0, 1.0, 3.5, true);
        assert!(!extinguish.vaporize);
        assert!(extinguish.extinguish_fire);
        assert_eq!(extinguish.extinguish_intensity, 100.0);
        assert!(extinguish.remove);
        assert!(extinguish.hit);

        let boiling_draw = liquid
            .draw_plan(1.0, 2.0, 4.0, 0.5, 0.25)
            .expect("liquid draw plan");
        assert_eq!(boiling_draw.color_rgba, 0x3366_ccff);
        assert_eq!(boiling_draw.mix_color_rgba, 0xaabb_cc66);
        assert_eq!(boiling_draw.mix, 0.5);
        assert!((boiling_draw.radius - 4.65).abs() < 0.0001);

        let normal_draw = liquid
            .draw_plan(0.0, 2.0, 4.0, 0.5, 25.0)
            .expect("normal liquid draw plan");
        assert_eq!(normal_draw.mix_color_rgba, 0xffff_ffff);
        assert_eq!(normal_draw.mix, 0.25);
        assert_eq!(normal_draw.radius, 3.0);
        assert!(liquid.despawn_effect_plan(1.0).is_none());
        assert_eq!(liquid.despawn_effect_plan(0.0), Some("hitLiquid".into()));

        let hit = liquid.hit_plan().expect("liquid hit plan");
        assert!(hit.deposit_puddle);
        assert_eq!(hit.puddle_size, 6.0);
        assert_eq!(hit.extinguish_intensity, Some(400.0));
        assert_eq!(
            hit.extinguish_offsets,
            vec![(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)]
        );
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

    #[test]
    fn interceptor_bullet_resolves_overlap_damage_and_removal() {
        let interceptor = InterceptorBulletType::new(4.0, 10.0, "point");
        assert_eq!(interceptor.base.base.speed, 4.0);
        assert_eq!(interceptor.base.base.damage, 10.0);

        assert!(interceptor
            .resolve_intercept(true, None, 20.0, 5.0)
            .is_none());

        let damaged = interceptor
            .resolve_intercept(true, Some((3.0, 4.0)), 20.0, 5.0)
            .expect("collision point should produce hit");
        assert_eq!((damaged.hit_x, damaged.hit_y), (3.0, 4.0));
        assert_eq!(damaged.other_damage_after, 15.0);
        assert!(!damaged.remove_other);
        assert!(damaged.remove_self);

        let removed = interceptor
            .resolve_intercept(true, Some((0.0, 0.0)), 5.0, 5.0)
            .expect("equal damage removes target bullet");
        assert!(removed.remove_other);
        assert_eq!(removed.other_damage_after, 0.0);

        let cleared = interceptor
            .resolve_intercept(false, None, 7.0, 5.0)
            .expect("missing target clears bullet data");
        assert!(cleared.clear_data);
        assert!(!cleared.remove_self);
    }

    #[test]
    fn point_bullet_computes_instant_end_position_and_trail_points() {
        let point = PointBulletType::default();
        assert!(point.base.scale_life);
        assert_eq!(point.base.lifetime, 100.0);
        assert!(!point.base.collides);
        assert!(!point.base.reflectable);
        assert!(!point.base.keep_velocity);

        let end = point.end_position(0.0, 0.0, 1.0, 0.0);
        assert_eq!(end, (100.0, 0.0));

        let points = point.trail_points(0.0, 0.0, 25.0, 0.0);
        assert_eq!(
            points,
            vec![(0.0, 0.0), (10.0, 0.0), (20.0, 0.0), (25.0, 0.0)]
        );
    }

    #[test]
    fn space_liquid_bullet_defaults_and_orb_radius_match_pow3out() {
        let liquid = SpaceLiquidBulletType::default();
        assert_eq!(liquid.base.speed, 3.5);
        assert_eq!(liquid.base.damage, 0.0);
        assert!(!liquid.base.collides);
        assert_eq!(liquid.base.lifetime, 90.0);
        assert_eq!(liquid.base.drag, 0.002);
        assert!(!liquid.base.hittable);
        assert_eq!(liquid.draw_radius(0.5), 4.8125);
    }

    #[test]
    fn point_laser_defaults_damage_and_update_plan_are_pure() {
        let mut laser = PointLaserBulletType::default();
        laser.base.damage = 20.0;
        laser.shake = 2.0;

        assert!(!laser.base.remove_after_pierce);
        assert_eq!(laser.base.speed, 0.0);
        assert_eq!(laser.base.lifetime, 20.0);
        assert!(laser.base.impact);
        assert!(!laser.base.keep_velocity);
        assert!(!laser.base.collides);
        assert!(laser.base.pierce);
        assert!(!laser.base.hittable);
        assert!(!laser.base.absorbable);
        assert_eq!(laser.base.optimal_life_fract, 0.5);
        assert_eq!(laser.base.draw_size, 1000.0);

        assert_eq!(laser.continuous_damage(), 240.0);
        assert_eq!(laser.estimate_dps(), 1200.0);
        assert_eq!(laser.laser_width_scale(0.5, 0.3), 0.5);
        let plan = laser.update_plan(true, false);
        assert!(plan.collide_point);
        assert!(!plan.beam_effect);
        assert_eq!(plan.shake, 2.0);
    }

    #[test]
    fn continuous_bullet_defaults_damage_range_and_timescale_plan_match_java() {
        let mut continuous = ContinuousBulletType::default();
        continuous.base.damage = 10.0;
        assert!(!continuous.base.remove_after_pierce);
        assert_eq!(continuous.base.pierce_cap, -1);
        assert_eq!(continuous.base.speed, 0.0);
        assert_eq!(continuous.base.lifetime, 16.0);
        assert!(continuous.base.impact);
        assert!(!continuous.base.keep_velocity);
        assert!(!continuous.base.collides);
        assert!(continuous.base.pierce);
        assert!(!continuous.base.hittable);
        assert!(!continuous.base.absorbable);

        assert_eq!(continuous.continuous_damage(), 120.0);
        assert_eq!(continuous.estimate_dps(), 600.0);
        assert_eq!(continuous.calculate_range(), 220.0);
        continuous.init_defaults();
        assert_eq!(continuous.base.draw_size, 440.0);

        continuous.timescale_damage = true;
        continuous.large_hit = true;
        continuous.shake = 2.0;
        let plan = continuous.damage_plan(1.5);
        assert_eq!(plan.length, 220.0);
        assert_eq!(plan.damage, 15.0);
        assert!(plan.large_hit);
        assert!(plan.laser_absorb);
        assert_eq!(plan.pierce_cap, -1);
        assert_eq!(plan.shake, 2.0);
    }

    #[test]
    fn continuous_laser_fades_length_and_stroke() {
        let laser = ContinuousLaserBulletType::new(25.0);
        assert_eq!(laser.base.base.damage, 25.0);
        assert_eq!(laser.base.shake, 1.0);
        assert!(laser.base.large_hit);
        assert_eq!(laser.base.base.draw_size, 420.0);
        assert_eq!(laser.base.base.light_opacity, 0.7);

        assert_eq!(laser.fade_out(0.0), 1.0);
        assert_eq!(laser.fade_out(8.0), 0.5);
        assert_eq!(laser.current_length(8.0), 110.0);
        assert_eq!(laser.stroke(0.5, 0.5, 1.0), 6.25);
    }

    #[test]
    fn continuous_flame_defaults_length_segments_and_flare_angle() {
        let mut flame = ContinuousFlameBulletType::new(30.0);
        assert_eq!(flame.base.base.damage, 30.0);
        assert_eq!(flame.base.base.optimal_life_fract, 0.5);
        assert_eq!(flame.base.length, 120.0);
        assert!(!flame.base.base.laser_absorb);
        assert_eq!(flame.base.base.ammo_multiplier, 1.0);
        assert!(flame.base.base.pierce_armor);
        assert_eq!(flame.current_length(0.5), 60.0);
        let segment = flame.flame_segment(0, 100.0, 0.5, 0.1);
        assert!((segment.0 - 100.799995).abs() < f32::EPSILON);
        assert!((segment.1 - 2.6455).abs() < f32::EPSILON);
        assert!((segment.2 - 0.32).abs() < f32::EPSILON);
        assert_eq!(flame.flare_angle(10.0, 90.0), 12.0);
        flame.rotate_flare = true;
        assert_eq!(flame.flare_angle(10.0, 90.0), 102.0);
    }
}
