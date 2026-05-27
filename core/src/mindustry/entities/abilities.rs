#[derive(Debug, Clone, PartialEq)]
pub struct BasicAbility {
    pub visible: bool,
    pub data: f32,
}

impl Default for BasicAbility {
    fn default() -> Self {
        Self {
            visible: true,
            data: 0.0,
        }
    }
}

pub trait Ability: Clone {
    fn is_visible(&self) -> bool {
        true
    }

    fn data(&self) -> f32 {
        0.0
    }

    fn set_data(&mut self, _data: f32) {}

    fn update<U>(&mut self, _unit: &mut U) {}

    fn draw<U>(&self, _unit: &U) {}

    fn death<U>(&self, _unit: &mut U) {}

    fn created<U>(&self, _unit: &mut U) {}

    fn init<T>(&mut self, _type: &mut T) {}

    fn display_bars<U, T>(&self, _unit: &U, _bars: &mut T) {}

    fn display<T>(&self, _table: &mut T) {}

    fn add_stats<T>(&self, _table: &mut T) {}

    fn ability_stat(&self, stat: &str, values: &[String]) -> String {
        if values.is_empty() {
            format!("ability.stat.{stat}")
        } else {
            format!("ability.stat.{stat}: {}", values.join(", "))
        }
    }

    fn copy(&self) -> Self
    where
        Self: Sized,
    {
        self.clone()
    }

    fn localized(&self) -> String {
        self.get_bundle()
    }

    fn get_bundle(&self) -> String {
        let type_name = std::any::type_name::<Self>();
        let simple = type_name.rsplit("::").next().unwrap_or(type_name);
        let base = simple.strip_suffix("Ability").unwrap_or(simple);
        format!("ability.{}", base.to_lowercase())
    }
}

impl Ability for BasicAbility {
    fn is_visible(&self) -> bool {
        self.visible
    }

    fn data(&self) -> f32 {
        self.data
    }

    fn set_data(&mut self, data: f32) {
        self.data = data;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegenAbility {
    pub base: BasicAbility,
    /// Amount healed as percent per tick.
    pub percent_amount: f32,
    /// Amount healed as a flat amount per tick.
    pub amount: f32,
}

impl Default for RegenAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            percent_amount: 0.0,
            amount: 0.0,
        }
    }
}

impl RegenAbility {
    pub fn heal_per_delta(&self, max_health: f32) -> f32 {
        max_health * self.percent_amount / 100.0 + self.amount
    }

    pub fn heal_amount(&self, max_health: f32, delta: f32) -> f32 {
        self.heal_per_delta(max_health) * delta
    }
}

impl Ability for RegenAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpawnDeathSpawnPlan {
    pub offset_x: f32,
    pub offset_y: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnDeathAbility {
    pub base: BasicAbility,
    pub amount: i32,
    pub rand_amount: i32,
    /// Random spread of units away from the spawned.
    pub spread: f32,
    /// If true, units spawned face outwards from the middle.
    pub face_outwards: bool,
}

impl Default for SpawnDeathAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            amount: 1,
            rand_amount: 0,
            spread: 8.0,
            face_outwards: true,
        }
    }
}

impl SpawnDeathAbility {
    pub fn planned_spawn_count(&self, random_bonus: i32) -> i32 {
        (self.amount + random_bonus.clamp(0, self.rand_amount.max(0))).max(0)
    }

    pub fn planned_spawn_offset(&self, angle_degrees: f32, distance: f32) -> (f32, f32) {
        let radians = angle_degrees.to_radians();
        (radians.cos() * distance, radians.sin() * distance)
    }

    pub fn planned_spawn_offset_from_fraction(
        &self,
        angle_degrees: f32,
        fraction: f32,
    ) -> (f32, f32) {
        self.planned_spawn_offset(
            angle_degrees,
            self.spread.max(0.0) * fraction.clamp(0.0, 1.0),
        )
    }

    pub fn planned_spawn_rotation(
        &self,
        unit_rotation: f32,
        offset_angle: f32,
        rotation_jitter: f32,
    ) -> f32 {
        if self.face_outwards {
            offset_angle
        } else {
            unit_rotation + rotation_jitter
        }
    }

    pub fn planned_spawn(
        &self,
        unit_rotation: f32,
        angle_degrees: f32,
        fraction: f32,
        rotation_jitter: f32,
    ) -> SpawnDeathSpawnPlan {
        let (offset_x, offset_y) = self.planned_spawn_offset_from_fraction(angle_degrees, fraction);
        let rotation = self.planned_spawn_rotation(unit_rotation, angle_degrees, rotation_jitter);

        SpawnDeathSpawnPlan {
            offset_x,
            offset_y,
            rotation,
        }
    }
}

impl Ability for SpawnDeathAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidExplodeAbility {
    pub base: BasicAbility,
    pub liquid_name: String,
    pub amount: f32,
    pub rad_amount_scale: f32,
    pub rad_scale: f32,
    pub noise_mag: f32,
    pub noise_scl: f32,
}

impl Default for LiquidExplodeAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            liquid_name: String::new(),
            amount: 120.0,
            rad_amount_scale: 5.0,
            rad_scale: 1.0,
            noise_mag: 6.5,
            noise_scl: 5.0,
        }
    }
}

impl LiquidExplodeAbility {
    pub fn planned_radius(&self, hit_size: f32, tile_size: f32) -> i32 {
        ((hit_size / tile_size) * self.rad_scale).max(1.0) as i32
    }

    pub fn planned_noise_radius(&self, hit_size: f32) -> f32 {
        hit_size / self.noise_mag
    }

    pub fn planned_deposit_amount(&self, distance_from_center: f32, radius: i32) -> f32 {
        let radius = radius.max(1) as f32;
        let scaling = (1.0 - distance_from_center / radius).max(0.0) * self.rad_amount_scale;
        self.amount * scaling
    }
}

impl Ability for LiquidExplodeAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidRegenAbility {
    pub base: BasicAbility,
    pub liquid_name: String,
    pub slurp_speed: f32,
    pub regen_per_slurp: f32,
    pub slurp_effect_chance: f32,
}

impl Default for LiquidRegenAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            liquid_name: String::new(),
            slurp_speed: 5.0,
            regen_per_slurp: 6.0,
            slurp_effect_chance: 0.4,
        }
    }
}

impl LiquidRegenAbility {
    pub fn planned_heal_amount(&self, slurped_amount: f32) -> f32 {
        slurped_amount * self.regen_per_slurp
    }
}

impl Ability for LiquidRegenAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceFieldUpdate {
    pub shield: f32,
    pub radius_scale: f32,
    pub alpha: f32,
    pub was_broken: bool,
    pub broke_this_tick: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceFieldHit {
    pub shield_after: f32,
    pub alpha: f32,
    pub absorbed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForceFieldAbility {
    pub base: BasicAbility,
    pub radius: f32,
    pub regen: f32,
    pub max: f32,
    pub cooldown: f32,
    pub sides: i32,
    pub rotation: f32,
    pub radius_scale: f32,
    pub alpha: f32,
    pub was_broken: bool,
}

impl Default for ForceFieldAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            radius: 60.0,
            regen: 0.1,
            max: 200.0,
            cooldown: 60.0 * 5.0,
            sides: 6,
            rotation: 0.0,
            radius_scale: 0.0,
            alpha: 0.0,
            was_broken: true,
        }
    }
}

impl ForceFieldAbility {
    pub fn new(radius: f32, regen: f32, max: f32, cooldown: f32) -> Self {
        Self {
            radius,
            regen,
            max,
            cooldown,
            ..Default::default()
        }
    }

    pub fn with_polygon(
        radius: f32,
        regen: f32,
        max: f32,
        cooldown: f32,
        sides: i32,
        rotation: f32,
    ) -> Self {
        Self {
            radius,
            regen,
            max,
            cooldown,
            sides,
            rotation,
            ..Default::default()
        }
    }

    pub fn created_shield(&self) -> f32 {
        self.max
    }

    pub fn real_radius(&self) -> f32 {
        self.radius_scale * self.radius
    }

    pub fn update_state(&mut self, shield: f32, delta: f32) -> ForceFieldUpdate {
        let mut shield = shield;
        let mut broke_this_tick = false;

        if shield <= 0.0 && !self.was_broken {
            shield -= self.cooldown * self.regen;
            broke_this_tick = true;
        }

        self.was_broken = shield <= 0.0;

        if shield < self.max {
            shield += delta * self.regen;
        }

        self.alpha = (self.alpha - delta / 10.0).max(0.0);

        if shield > 0.0 {
            self.radius_scale = lerp_delta(self.radius_scale, 1.0, 0.06, delta);
        } else {
            self.radius_scale = 0.0;
        }

        ForceFieldUpdate {
            shield,
            radius_scale: self.radius_scale,
            alpha: self.alpha,
            was_broken: self.was_broken,
            broke_this_tick,
        }
    }

    pub fn absorb_bullet(
        &mut self,
        shield: f32,
        unit_team: i32,
        bullet_team: i32,
        bullet_absorbable: bool,
        bullet_pos: (f32, f32),
        unit_pos: (f32, f32),
        shield_damage: f32,
    ) -> Option<ForceFieldHit> {
        if bullet_team != unit_team
            && bullet_absorbable
            && shield > 0.0
            && point_in_regular_polygon(
                self.sides,
                unit_pos.0,
                unit_pos.1,
                self.real_radius(),
                self.rotation,
                bullet_pos.0,
                bullet_pos.1,
            )
        {
            self.alpha = 1.0;
            Some(ForceFieldHit {
                shield_after: shield - shield_damage,
                alpha: self.alpha,
                absorbed: true,
            })
        } else {
            None
        }
    }
}

impl Ability for ForceFieldAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepairFieldTarget {
    pub damaged: bool,
    pub max_health: f32,
    pub same_type: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepairFieldPulse {
    pub heals: Vec<f32>,
    pub active_effect: bool,
    pub timer: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepairFieldAbility {
    pub base: BasicAbility,
    pub amount: f32,
    pub reload: f32,
    pub range: f32,
    pub heal_percent: f32,
    pub parentize_effects: bool,
    /// Multiplies healing to units of the same type by this amount.
    pub same_type_heal_mult: f32,
    pub timer: f32,
    pub was_healed: bool,
}

impl Default for RepairFieldAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            amount: 1.0,
            reload: 100.0,
            range: 60.0,
            heal_percent: 0.0,
            parentize_effects: false,
            same_type_heal_mult: 1.0,
            timer: 0.0,
            was_healed: false,
        }
    }
}

impl RepairFieldAbility {
    pub fn new(amount: f32, reload: f32, range: f32) -> Self {
        Self {
            amount,
            reload,
            range,
            ..Default::default()
        }
    }

    pub fn with_percent(amount: f32, reload: f32, range: f32, heal_percent: f32) -> Self {
        Self {
            amount,
            reload,
            range,
            heal_percent,
            ..Default::default()
        }
    }

    pub fn heal_amount_for(&self, target: RepairFieldTarget) -> f32 {
        let heal_mult = if target.same_type {
            self.same_type_heal_mult
        } else {
            1.0
        };
        (self.amount + self.heal_percent / 100.0 * target.max_health) * heal_mult
    }

    pub fn update_targets(
        &mut self,
        delta: f32,
        targets: &[RepairFieldTarget],
    ) -> Option<RepairFieldPulse> {
        self.timer += delta;

        if self.timer < self.reload {
            return None;
        }

        self.was_healed = targets.iter().any(|target| target.damaged);
        let heals = targets
            .iter()
            .copied()
            .map(|target| self.heal_amount_for(target))
            .collect::<Vec<_>>();
        self.timer = 0.0;

        Some(RepairFieldPulse {
            heals,
            active_effect: self.was_healed,
            timer: self.timer,
        })
    }

    pub fn repairs_per_second(&self) -> f32 {
        self.amount * 60.0 / self.reload
    }

    pub fn repair_percent_per_second(&self) -> f32 {
        self.heal_percent * 60.0 / self.reload
    }
}

impl Ability for RepairFieldAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShieldRegenFieldTarget {
    pub shield: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShieldRegenFieldPulse {
    pub shields: Vec<f32>,
    pub active_effect: bool,
    pub timer: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShieldRegenFieldAbility {
    pub base: BasicAbility,
    pub amount: f32,
    pub max: f32,
    pub reload: f32,
    pub range: f32,
    pub parentize_effects: bool,
    pub timer: f32,
    pub applied: bool,
}

impl Default for ShieldRegenFieldAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            amount: 1.0,
            max: 100.0,
            reload: 100.0,
            range: 60.0,
            parentize_effects: false,
            timer: 0.0,
            applied: false,
        }
    }
}

impl ShieldRegenFieldAbility {
    pub fn new(amount: f32, max: f32, reload: f32, range: f32) -> Self {
        Self {
            amount,
            max,
            reload,
            range,
            ..Default::default()
        }
    }

    pub fn shield_after_pulse(&self, shield: f32) -> f32 {
        if shield < self.max {
            (shield + self.amount).min(self.max)
        } else {
            shield
        }
    }

    pub fn update_targets(
        &mut self,
        delta: f32,
        targets: &[ShieldRegenFieldTarget],
    ) -> Option<ShieldRegenFieldPulse> {
        self.timer += delta;

        if self.timer < self.reload {
            return None;
        }

        self.applied = false;
        let shields = targets
            .iter()
            .map(|target| {
                let next = self.shield_after_pulse(target.shield);
                if next > target.shield {
                    self.applied = true;
                }
                next
            })
            .collect::<Vec<_>>();
        self.timer = 0.0;

        Some(ShieldRegenFieldPulse {
            shields,
            active_effect: self.applied,
            timer: self.timer,
        })
    }

    pub fn pulses_per_second(&self) -> f32 {
        60.0 / self.reload
    }

    pub fn regen_per_second(&self) -> f32 {
        self.amount * 60.0 / self.reload
    }
}

impl Ability for ShieldRegenFieldAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusFieldPulse {
    pub effect: String,
    pub duration: f32,
    pub target_count: usize,
    pub active_x: f32,
    pub active_y: f32,
    pub active_param: f32,
    pub timer: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusFieldAbility {
    pub base: BasicAbility,
    pub effect: String,
    pub duration: f32,
    pub reload: f32,
    pub range: f32,
    pub on_shoot: bool,
    pub effect_x: f32,
    pub effect_y: f32,
    pub parentize_effects: bool,
    pub effect_size_param: bool,
    pub color: String,
    pub timer: f32,
}

impl Default for StatusFieldAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            effect: String::new(),
            duration: 60.0,
            reload: 100.0,
            range: 20.0,
            on_shoot: false,
            effect_x: 0.0,
            effect_y: 0.0,
            parentize_effects: false,
            effect_size_param: true,
            color: "accent".into(),
            timer: 0.0,
        }
    }
}

impl StatusFieldAbility {
    pub fn new(effect: impl Into<String>, duration: f32, reload: f32, range: f32) -> Self {
        Self {
            effect: effect.into(),
            duration,
            reload,
            range,
            ..Default::default()
        }
    }

    pub fn update_targets(
        &mut self,
        delta: f32,
        is_shooting: bool,
        unit_x: f32,
        unit_y: f32,
        unit_rotation: f32,
        target_count: usize,
    ) -> Option<StatusFieldPulse> {
        self.timer += delta;

        if self.timer < self.reload || (self.on_shoot && !is_shooting) {
            return None;
        }

        let (offset_x, offset_y) =
            rotated_effect_offset(unit_rotation, self.effect_y, self.effect_x);
        let active_param = if self.effect_size_param {
            self.range
        } else {
            unit_rotation
        };
        self.timer = 0.0;

        Some(StatusFieldPulse {
            effect: self.effect.clone(),
            duration: self.duration,
            target_count,
            active_x: unit_x + offset_x,
            active_y: unit_y + offset_y,
            active_param,
            timer: self.timer,
        })
    }

    pub fn pulses_per_second(&self) -> f32 {
        60.0 / self.reload
    }
}

impl Ability for StatusFieldAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuppressionFieldPulse {
    pub x: f32,
    pub y: f32,
    pub range: f32,
    pub reload: f32,
    pub max_delay: f32,
    pub apply_particle_chance: f32,
    pub timer: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuppressionFieldAbility {
    pub base: BasicAbility,
    pub reload: f32,
    pub max_delay: f32,
    pub range: f32,
    pub x: f32,
    pub y: f32,
    pub active: bool,
    pub apply_particle_chance: f32,
    pub timer: f32,
}

impl Default for SuppressionFieldAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            reload: 60.0 * 1.5,
            max_delay: 60.0 * 1.5,
            range: 200.0,
            x: 0.0,
            y: 0.0,
            active: true,
            apply_particle_chance: 13.0,
            timer: 0.0,
        }
    }
}

impl SuppressionFieldAbility {
    pub fn update_state(
        &mut self,
        delta: f32,
        unit_x: f32,
        unit_y: f32,
        unit_rotation: f32,
    ) -> Option<SuppressionFieldPulse> {
        if !self.active {
            return None;
        }

        self.timer += delta;
        if self.timer < self.max_delay {
            return None;
        }

        let (offset_x, offset_y) = rotate_offset(self.x, self.y, unit_rotation - 90.0);
        self.timer = 0.0;

        Some(SuppressionFieldPulse {
            x: unit_x + offset_x,
            y: unit_y + offset_y,
            range: self.range,
            reload: self.reload,
            max_delay: self.max_delay,
            apply_particle_chance: self.apply_particle_chance,
            timer: self.timer,
        })
    }

    pub fn duration_seconds(&self) -> f32 {
        self.reload / 60.0
    }
}

impl Ability for SuppressionFieldAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveEffectPlan {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub amount: i32,
    pub timer: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoveEffectAbility {
    pub base: BasicAbility,
    pub min_velocity: f32,
    pub interval: f32,
    pub chance: f32,
    pub amount: i32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub range_x: f32,
    pub range_y: f32,
    pub range_length_min: f32,
    pub range_length_max: f32,
    pub rotate_effect: bool,
    pub effect_param: f32,
    pub team_color: bool,
    pub parentize_effects: bool,
    pub counter: f32,
}

impl Default for MoveEffectAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility {
                visible: false,
                data: 0.0,
            },
            min_velocity: 0.08,
            interval: 3.0,
            chance: 0.0,
            amount: 1,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            range_x: 0.0,
            range_y: 0.0,
            range_length_min: 0.0,
            range_length_max: 0.0,
            rotate_effect: false,
            effect_param: 3.0,
            team_color: false,
            parentize_effects: false,
            counter: 0.0,
        }
    }
}

impl MoveEffectAbility {
    pub fn new(x: f32, y: f32, interval: f32) -> Self {
        Self {
            x,
            y,
            interval,
            ..Default::default()
        }
    }

    pub fn update_plan(
        &mut self,
        delta: f32,
        velocity_len2: f32,
        in_fog: bool,
        chance_triggered: bool,
        unit_x: f32,
        unit_y: f32,
        unit_rotation: f32,
        random_offset: (f32, f32),
    ) -> Option<MoveEffectPlan> {
        self.counter += delta;

        let moving = velocity_len2 >= self.min_velocity * self.min_velocity;
        let timed = self.counter >= self.interval;
        let chance = self.chance > 0.0 && chance_triggered;
        if !moving || !(timed || chance) || in_fog {
            return None;
        }

        let (local_x, local_y) = if self.range_length_max > 0.0 {
            (self.x + random_offset.0, self.y + random_offset.1)
        } else {
            (
                self.x + random_offset.0.clamp(-self.range_x, self.range_x),
                self.y + random_offset.1.clamp(-self.range_y, self.range_y),
            )
        };
        let (offset_x, offset_y) = rotate_offset(local_x, local_y, unit_rotation - 90.0);
        self.counter %= self.interval;

        Some(MoveEffectPlan {
            x: unit_x + offset_x,
            y: unit_y + offset_y,
            rotation: (if self.rotate_effect {
                unit_rotation
            } else {
                self.effect_param
            }) + self.rotation,
            amount: self.amount,
            timer: self.counter,
        })
    }
}

impl Ability for MoveEffectAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveLightningPlan {
    pub x: f32,
    pub y: f32,
    pub lightning_x: f32,
    pub lightning_y: f32,
    pub rotation: f32,
    pub damage: f32,
    pub length: i32,
    pub bullet_rotation: f32,
    pub side_after: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoveLightningAbility {
    pub base: BasicAbility,
    pub damage: f32,
    pub chance: f32,
    pub length: i32,
    pub min_speed: f32,
    pub max_speed: f32,
    pub y: f32,
    pub x: f32,
    pub alternate: bool,
    pub heat_region: String,
    pub bullet_angle: f32,
    pub bullet_spread: f32,
    pub parentize_effects: bool,
    pub side: f32,
}

impl Default for MoveLightningAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            damage: 35.0,
            chance: 0.15,
            length: 12,
            min_speed: 0.8,
            max_speed: 1.2,
            y: 0.0,
            x: 0.0,
            alternate: true,
            heat_region: "error".into(),
            bullet_angle: 0.0,
            bullet_spread: 0.0,
            parentize_effects: false,
            side: 1.0,
        }
    }
}

impl MoveLightningAbility {
    pub fn new(
        damage: f32,
        length: i32,
        chance: f32,
        y: f32,
        min_speed: f32,
        max_speed: f32,
    ) -> Self {
        Self {
            damage,
            length,
            chance,
            y,
            min_speed,
            max_speed,
            ..Default::default()
        }
    }

    pub fn speed_scale(&self, velocity_len: f32) -> f32 {
        if (self.max_speed - self.min_speed).abs() <= f32::EPSILON {
            if velocity_len >= self.max_speed {
                1.0
            } else {
                0.0
            }
        } else {
            ((velocity_len - self.min_speed) / (self.max_speed - self.min_speed)).clamp(0.0, 1.0)
        }
    }

    pub fn trigger_probability(&self, delta: f32, velocity_len: f32) -> f32 {
        (delta * self.chance * self.speed_scale(velocity_len)).clamp(0.0, 1.0)
    }

    pub fn update_plan(
        &mut self,
        delta: f32,
        velocity: (f32, f32),
        unit_pos: (f32, f32),
        unit_rotation: f32,
        chance_triggered: bool,
        bullet_spread_offset: f32,
    ) -> Option<MoveLightningPlan> {
        let velocity_len = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();
        if self.trigger_probability(delta, velocity_len) <= 0.0 || !chance_triggered {
            return None;
        }

        let (offset_x, offset_y) = rotated_effect_offset(unit_rotation, self.y, self.x * self.side);
        let x = unit_pos.0 + offset_x;
        let y = unit_pos.1 + offset_y;
        if self.alternate {
            self.side *= -1.0;
        }

        Some(MoveLightningPlan {
            x,
            y,
            lightning_x: x + velocity.0,
            lightning_y: y + velocity.1,
            rotation: unit_rotation,
            damage: self.damage,
            length: self.length,
            bullet_rotation: unit_rotation + self.bullet_angle + bullet_spread_offset,
            side_after: self.side,
        })
    }
}

impl Ability for MoveLightningAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmorPlateUpdate {
    pub warmup: f32,
    pub health_multiplier_bonus: f32,
    pub should_draw: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArmorPlateAbility {
    pub base: BasicAbility,
    pub plate_suffix: String,
    pub shine_suffix: String,
    pub shine_speed: f32,
    pub z: f32,
    pub draw_plate: bool,
    pub draw_shine: bool,
    pub health_multiplier: f32,
    pub warmup: f32,
}

impl Default for ArmorPlateAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            plate_suffix: "-armor".into(),
            shine_suffix: "-shine".into(),
            shine_speed: 1.0,
            z: -1.0,
            draw_plate: true,
            draw_shine: true,
            health_multiplier: 0.2,
            warmup: 0.0,
        }
    }
}

impl ArmorPlateAbility {
    pub fn update_state(&mut self, is_shooting: bool, delta: f32) -> ArmorPlateUpdate {
        self.warmup = lerp_delta(self.warmup, if is_shooting { 1.0 } else { 0.0 }, 0.1, delta);
        ArmorPlateUpdate {
            warmup: self.warmup,
            health_multiplier_bonus: self.warmup * self.health_multiplier,
            should_draw: (self.draw_plate || self.draw_shine) && self.warmup > 0.001,
        }
    }

    pub fn damage_reduction_stat_percent(&self) -> f32 {
        -self.health_multiplier * 100.0
    }
}

impl Ability for ArmorPlateAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitSpawnPlan {
    pub unit: String,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub timer: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitSpawnAbility {
    pub base: BasicAbility,
    pub unit: String,
    pub spawn_time: f32,
    pub spawn_x: f32,
    pub spawn_y: f32,
    pub parentize_effects: bool,
    pub timer: f32,
}

impl Default for UnitSpawnAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            unit: String::new(),
            spawn_time: 60.0,
            spawn_x: 0.0,
            spawn_y: 0.0,
            parentize_effects: false,
            timer: 0.0,
        }
    }
}

impl UnitSpawnAbility {
    pub fn new(unit: impl Into<String>, spawn_time: f32, spawn_x: f32, spawn_y: f32) -> Self {
        Self {
            unit: unit.into(),
            spawn_time,
            spawn_x,
            spawn_y,
            ..Default::default()
        }
    }

    pub fn from_descriptor(descriptor: &str) -> Option<Self> {
        let descriptor = descriptor.trim();
        let args = descriptor.strip_prefix("UnitSpawnAbility:").or_else(|| {
            descriptor
                .strip_prefix("UnitSpawnAbility(")
                .and_then(|rest| rest.strip_suffix(')'))
        })?;
        let mut parts = args
            .split([',', ':'])
            .map(str::trim)
            .filter(|part| !part.is_empty());

        let unit = parts.next()?;
        let spawn_time = parts.next()?.parse().ok()?;
        let spawn_x = parts.next()?.parse().ok()?;
        let spawn_y = parts.next()?.parse().ok()?;
        let mut ability = Self::new(unit, spawn_time, spawn_x, spawn_y);
        if let Some(parentize_effects) = parts.next() {
            ability.parentize_effects = matches!(parentize_effects, "true" | "1" | "parent");
        }
        Some(ability)
    }

    pub fn update_state(
        &mut self,
        delta: f32,
        unit_build_speed: f32,
        can_create: bool,
        parent_x: f32,
        parent_y: f32,
        parent_rotation: f32,
    ) -> Option<UnitSpawnPlan> {
        self.timer += delta * unit_build_speed;

        if self.timer < self.spawn_time || !can_create {
            return None;
        }

        let (offset_x, offset_y) =
            rotated_effect_offset(parent_rotation, self.spawn_y, -self.spawn_x);
        self.timer = 0.0;

        Some(UnitSpawnPlan {
            unit: self.unit.clone(),
            x: parent_x + offset_x,
            y: parent_y + offset_y,
            rotation: parent_rotation,
            timer: self.timer,
        })
    }

    pub fn build_progress(&self) -> f32 {
        if self.spawn_time <= 0.0 {
            1.0
        } else {
            (self.timer / self.spawn_time).clamp(0.0, 1.0)
        }
    }

    pub fn build_time_seconds(&self) -> f32 {
        self.spawn_time / 60.0
    }
}

impl Ability for UnitSpawnAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyFieldAction {
    Heal,
    Damage,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyFieldTarget {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub air: bool,
    pub targetable: bool,
    pub same_team: bool,
    pub damaged: bool,
    pub max_health: f32,
    pub same_type: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnergyFieldHit {
    pub id: u32,
    pub action: EnergyFieldAction,
    pub amount: f32,
    pub angle: f32,
    pub status: Option<String>,
    pub status_duration: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnergyFieldPulse {
    pub x: f32,
    pub y: f32,
    pub hits: Vec<EnergyFieldHit>,
    pub any_nearby: bool,
    pub ammo_after: i32,
    pub timer: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnergyFieldAbility {
    pub base: BasicAbility,
    pub damage: f32,
    pub reload: f32,
    pub range: f32,
    pub status: String,
    pub status_duration: f32,
    pub x: f32,
    pub y: f32,
    pub target_ground: bool,
    pub target_air: bool,
    pub hit_buildings: bool,
    pub hit_units: bool,
    pub max_targets: usize,
    pub heal_percent: f32,
    pub same_type_heal_mult: f32,
    pub display_heal: bool,
    pub use_ammo: bool,
    pub timer: f32,
    pub cur_stroke: f32,
    pub any_nearby: bool,
}

impl Default for EnergyFieldAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            damage: 1.0,
            reload: 100.0,
            range: 60.0,
            status: "electrified".into(),
            status_duration: 60.0 * 6.0,
            x: 0.0,
            y: 0.0,
            target_ground: true,
            target_air: true,
            hit_buildings: true,
            hit_units: true,
            max_targets: 25,
            heal_percent: 3.0,
            same_type_heal_mult: 1.0,
            display_heal: true,
            use_ammo: true,
            timer: 0.0,
            cur_stroke: 0.0,
            any_nearby: false,
        }
    }
}

impl EnergyFieldAbility {
    pub fn new(damage: f32, reload: f32, range: f32) -> Self {
        Self {
            damage,
            reload,
            range,
            ..Default::default()
        }
    }

    pub fn from_descriptor(descriptor: &str) -> Option<Self> {
        let descriptor = descriptor.trim();
        let args = descriptor.strip_prefix("EnergyFieldAbility:").or_else(|| {
            descriptor
                .strip_prefix("EnergyFieldAbility(")
                .and_then(|rest| rest.strip_suffix(')'))
        })?;
        let mut parts = args
            .split([',', ':'])
            .map(str::trim)
            .filter(|part| !part.is_empty());

        let damage = parts.next()?.parse().ok()?;
        let reload = parts.next()?.parse().ok()?;
        let range = parts.next()?.parse().ok()?;
        let mut ability = Self::new(damage, reload, range);
        if let Some(heal_percent) = parts.next().and_then(|value| value.parse().ok()) {
            ability.heal_percent = heal_percent;
        }
        if let Some(same_type_heal_mult) = parts.next().and_then(|value| value.parse().ok()) {
            ability.same_type_heal_mult = same_type_heal_mult;
        }
        if let Some(max_targets) = parts.next().and_then(|value| value.parse().ok()) {
            ability.max_targets = max_targets;
        }
        if let Some(status_duration) = parts.next().and_then(|value| value.parse().ok()) {
            ability.status_duration = status_duration;
        }
        if let Some(status) = parts.next() {
            ability.status = status.to_string();
        }
        Some(ability)
    }

    pub fn center(&self, unit_x: f32, unit_y: f32, unit_rotation: f32) -> (f32, f32) {
        let (offset_x, offset_y) = rotate_offset(self.x, self.y, unit_rotation - 90.0);
        (unit_x + offset_x, unit_y + offset_y)
    }

    pub fn update_targets(
        &mut self,
        delta: f32,
        unit_x: f32,
        unit_y: f32,
        unit_rotation: f32,
        unit_damage_scale: f32,
        ammo: i32,
        unit_ammo_rule: bool,
        targets: &[EnergyFieldTarget],
    ) -> Option<EnergyFieldPulse> {
        self.cur_stroke = lerp_delta(
            self.cur_stroke,
            if self.any_nearby { 1.0 } else { 0.0 },
            0.09,
            delta,
        );
        self.timer += delta;

        if self.timer < self.reload || (self.use_ammo && ammo <= 0 && unit_ammo_rule) {
            return None;
        }

        let (cx, cy) = self.center(unit_x, unit_y, unit_rotation);
        let mut sorted = targets
            .iter()
            .copied()
            .filter(|target| distance2(cx, cy, target.x, target.y) <= self.range * self.range)
            .filter(|target| target.targetable)
            .filter(|target| {
                if target.air {
                    self.target_air
                } else {
                    self.target_ground
                }
            })
            .collect::<Vec<_>>();
        sorted.sort_by(|a, b| {
            distance2(cx, cy, a.x, a.y)
                .partial_cmp(&distance2(cx, cy, b.x, b.y))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.any_nearby = false;
        let mut hits = Vec::new();
        for target in sorted.into_iter().take(self.max_targets) {
            if target.same_team {
                if target.damaged {
                    self.any_nearby = true;
                    let heal_mult = if target.same_type {
                        self.same_type_heal_mult
                    } else {
                        1.0
                    };
                    hits.push(EnergyFieldHit {
                        id: target.id,
                        action: EnergyFieldAction::Heal,
                        amount: self.heal_percent / 100.0 * target.max_health * heal_mult,
                        angle: angle_to(cx, cy, target.x, target.y),
                        status: None,
                        status_duration: 0.0,
                    });
                }
            } else {
                self.any_nearby = true;
                hits.push(EnergyFieldHit {
                    id: target.id,
                    action: EnergyFieldAction::Damage,
                    amount: self.damage * unit_damage_scale,
                    angle: angle_to(cx, cy, target.x, target.y),
                    status: (!self.status.is_empty()).then(|| self.status.clone()),
                    status_duration: self.status_duration,
                });
            }
        }

        let ammo_after = if self.any_nearby && self.use_ammo && unit_ammo_rule {
            ammo - 1
        } else {
            ammo
        };
        self.timer = 0.0;

        Some(EnergyFieldPulse {
            x: cx,
            y: cy,
            hits,
            any_nearby: self.any_nearby,
            ammo_after,
            timer: self.timer,
        })
    }

    pub fn firing_rate_per_second(&self) -> f32 {
        60.0 / self.reload
    }
}

impl Ability for EnergyFieldAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.base.data
    }

    fn set_data(&mut self, data: f32) {
        self.base.data = data;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShieldArcUpdate {
    pub data: f32,
    pub width_scale: f32,
    pub alpha: f32,
    pub active: bool,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShieldArcHitAction {
    None,
    Absorb,
    Deflect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShieldArcHit {
    pub action: ShieldArcHitAction,
    pub data_after: f32,
    pub alpha: f32,
    pub broke: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShieldArcAbility {
    pub base: BasicAbility,
    pub radius: f32,
    pub regen: f32,
    pub max: f32,
    pub cooldown: f32,
    pub angle: f32,
    pub angle_offset: f32,
    pub x: f32,
    pub y: f32,
    pub when_shooting: bool,
    pub width: f32,
    pub chance_deflect: f32,
    pub reflect_building_damage: f32,
    pub reflect_vel: f32,
    pub reflect_time: f32,
    pub missile_unit_multiplier: f32,
    pub draw_arc: bool,
    pub offset_region: bool,
    pub push_units: bool,
    pub data: f32,
    pub width_scale: f32,
    pub alpha: f32,
}

impl Default for ShieldArcAbility {
    fn default() -> Self {
        Self {
            base: BasicAbility::default(),
            radius: 60.0,
            regen: 0.1,
            max: 200.0,
            cooldown: 60.0 * 5.0,
            angle: 80.0,
            angle_offset: 0.0,
            x: 0.0,
            y: 0.0,
            when_shooting: true,
            width: 6.0,
            chance_deflect: -1.0,
            reflect_building_damage: 1.0,
            reflect_vel: 1.0,
            reflect_time: 1.0 - 0.5,
            missile_unit_multiplier: 2.0,
            draw_arc: true,
            offset_region: false,
            push_units: true,
            data: 0.0,
            width_scale: 0.0,
            alpha: 0.0,
        }
    }
}

impl ShieldArcAbility {
    pub fn created_shield(&mut self) -> f32 {
        self.data = self.max;
        self.data
    }

    pub fn shield_position(&self, unit_x: f32, unit_y: f32, unit_rotation: f32) -> (f32, f32) {
        let (offset_x, offset_y) = rotate_offset(self.x, self.y, unit_rotation - 90.0);
        (unit_x + offset_x, unit_y + offset_y)
    }

    pub fn update_state(
        &mut self,
        delta: f32,
        is_shooting: bool,
        unit_x: f32,
        unit_y: f32,
        unit_rotation: f32,
    ) -> ShieldArcUpdate {
        if self.data < self.max {
            self.data += delta * self.regen;
        }

        let active = self.data > 0.0 && (is_shooting || !self.when_shooting);
        self.alpha = (self.alpha - delta / 10.0).max(0.0);
        self.width_scale = if active {
            lerp_delta(self.width_scale, 1.0, 0.06, delta)
        } else {
            lerp_delta(self.width_scale, 0.0, 0.11, delta)
        };
        let (x, y) = self.shield_position(unit_x, unit_y, unit_rotation);

        ShieldArcUpdate {
            data: self.data,
            width_scale: self.width_scale,
            alpha: self.alpha,
            active,
            x,
            y,
        }
    }

    pub fn contains_point(
        &self,
        shield_x: f32,
        shield_y: f32,
        unit_rotation: f32,
        point_x: f32,
        point_y: f32,
    ) -> bool {
        let dist = distance2(shield_x, shield_y, point_x, point_y).sqrt();
        let in_ring = dist >= self.radius - self.width && dist <= self.radius + self.width;
        let angle = angle_to(shield_x, shield_y, point_x, point_y);
        in_ring && angle_within(angle, unit_rotation + self.angle_offset, self.angle / 2.0)
    }

    pub fn apply_bullet_hit(
        &mut self,
        bullet_damage: f32,
        shield_damage: f32,
        reflectable: bool,
        velocity_len: f32,
        deflect_chance_passed: bool,
    ) -> ShieldArcHit {
        if self.data <= 0.0 {
            return ShieldArcHit {
                action: ShieldArcHitAction::None,
                data_after: self.data,
                alpha: self.alpha,
                broke: false,
            };
        }

        let action = if self.chance_deflect > 0.0
            && velocity_len >= 0.1
            && reflectable
            && deflect_chance_passed
        {
            ShieldArcHitAction::Deflect
        } else {
            ShieldArcHitAction::Absorb
        };

        let broke = self.data <= bullet_damage;
        if broke {
            self.data -= self.cooldown * self.regen;
        }
        self.data -= shield_damage;
        self.alpha = 1.0;

        ShieldArcHit {
            action,
            data_after: self.data,
            alpha: self.alpha,
            broke,
        }
    }
}

impl Ability for ShieldArcAbility {
    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn data(&self) -> f32 {
        self.data
    }

    fn set_data(&mut self, data: f32) {
        self.data = data;
    }
}

fn lerp_delta(from: f32, to: f32, alpha: f32, delta: f32) -> f32 {
    let scaled = 1.0 - (1.0 - alpha).powf(delta.max(0.0));
    from + (to - from) * scaled
}

fn distance2(x: f32, y: f32, tx: f32, ty: f32) -> f32 {
    let dx = tx - x;
    let dy = ty - y;
    dx * dx + dy * dy
}

fn angle_to(x: f32, y: f32, tx: f32, ty: f32) -> f32 {
    (ty - y).atan2(tx - x).to_degrees()
}

fn angle_within(angle: f32, target: f32, margin: f32) -> bool {
    let mut diff = (angle - target).rem_euclid(360.0).abs();
    if diff > 180.0 {
        diff = 360.0 - diff;
    }
    diff <= margin
}

fn rotate_offset(x: f32, y: f32, rotation: f32) -> (f32, f32) {
    let radians = rotation.to_radians();
    (
        x * radians.cos() - y * radians.sin(),
        x * radians.sin() + y * radians.cos(),
    )
}

fn rotated_effect_offset(rotation: f32, forward: f32, sideways: f32) -> (f32, f32) {
    let radians = rotation.to_radians();
    (
        radians.cos() * forward - radians.sin() * sideways,
        radians.sin() * forward + radians.cos() * sideways,
    )
}

fn point_in_regular_polygon(
    sides: i32,
    center_x: f32,
    center_y: f32,
    radius: f32,
    rotation: f32,
    x: f32,
    y: f32,
) -> bool {
    if radius <= 0.0 || sides < 3 {
        return false;
    }

    let mut inside = false;
    let sides = sides as usize;
    let mut prev = polygon_vertex(sides - 1, sides, center_x, center_y, radius, rotation);
    for i in 0..sides {
        let current = polygon_vertex(i, sides, center_x, center_y, radius, rotation);
        let intersects = ((current.1 > y) != (prev.1 > y))
            && (x < (prev.0 - current.0) * (y - current.1) / (prev.1 - current.1) + current.0);
        if intersects {
            inside = !inside;
        }
        prev = current;
    }
    inside
}

fn polygon_vertex(
    index: usize,
    sides: usize,
    center_x: f32,
    center_y: f32,
    radius: f32,
    rotation: f32,
) -> (f32, f32) {
    let angle = rotation.to_radians() + index as f32 * std::f32::consts::TAU / sides as f32;
    (
        center_x + angle.cos() * radius,
        center_y + angle.sin() * radius,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        Ability, ArmorPlateAbility, BasicAbility, EnergyFieldAbility, EnergyFieldAction,
        EnergyFieldTarget, ForceFieldAbility, LiquidExplodeAbility, LiquidRegenAbility,
        MoveEffectAbility, MoveLightningAbility, RegenAbility, RepairFieldAbility,
        RepairFieldTarget, ShieldArcAbility, ShieldArcHitAction, ShieldRegenFieldAbility,
        ShieldRegenFieldTarget, SpawnDeathAbility, StatusFieldAbility, SuppressionFieldAbility,
        UnitSpawnAbility,
    };

    #[derive(Clone)]
    struct DemoAbility;

    impl Ability for DemoAbility {}

    #[test]
    fn basic_ability_stores_visibility_and_data() {
        let mut ability = BasicAbility::default();
        assert!(ability.is_visible());
        assert_eq!(ability.data(), 0.0);

        ability.set_data(2.5);
        assert_eq!(ability.data(), 2.5);
    }

    #[test]
    fn trait_defaults_provide_bundle_name_and_copy() {
        let ability = DemoAbility;
        assert_eq!(ability.get_bundle(), "ability.demo");
        assert_eq!(ability.localized(), "ability.demo");
        assert_eq!(ability.copy().get_bundle(), "ability.demo");
        assert_eq!(
            ability.ability_stat("reload", &["1".to_string()]),
            "ability.stat.reload: 1"
        );
    }

    #[test]
    fn regen_ability_uses_max_health_and_delta_in_pure_formula() {
        let ability = RegenAbility {
            percent_amount: 2.5,
            amount: 3.0,
            ..Default::default()
        };

        assert!((ability.heal_per_delta(200.0) - 8.0).abs() < 0.0001);
        assert!((ability.heal_amount(200.0, 0.5) - 4.0).abs() < 0.0001);
    }

    #[test]
    fn spawn_death_plans_count_offset_and_rotation_without_world_state() {
        let ability = SpawnDeathAbility {
            amount: 2,
            rand_amount: 3,
            spread: 8.0,
            face_outwards: true,
            ..Default::default()
        };

        assert_eq!(ability.planned_spawn_count(2), 4);
        assert_eq!(ability.planned_spawn_count(8), 5);

        let (offset_x, offset_y) = ability.planned_spawn_offset_from_fraction(90.0, 0.5);
        assert!(offset_x.abs() < 0.0001);
        assert!((offset_y - 4.0).abs() < 0.0001);

        let plan = ability.planned_spawn(30.0, 90.0, 0.5, 2.0);
        assert!(plan.offset_x.abs() < 0.0001);
        assert!((plan.offset_y - 4.0).abs() < 0.0001);
        assert_eq!(plan.rotation, 90.0);
    }

    #[test]
    fn liquid_explode_plans_radius_and_deposit_amount() {
        let ability = LiquidExplodeAbility::default();

        assert_eq!(ability.planned_radius(48.0, 8.0), 6);
        assert!((ability.planned_noise_radius(13.0) - 2.0).abs() < 0.0001);
        assert!((ability.planned_deposit_amount(0.0, 6) - 600.0).abs() < 0.0001);
        assert!((ability.planned_deposit_amount(3.0, 6) - 300.0).abs() < 0.0001);
    }

    #[test]
    fn liquid_regen_is_a_lightweight_data_shell() {
        let mut ability = LiquidRegenAbility::default();
        ability.set_data(7.5);
        assert!(ability.is_visible());
        assert_eq!(ability.data(), 7.5);
        assert!((ability.planned_heal_amount(2.0) - 12.0).abs() < 0.0001);
    }

    #[test]
    fn force_field_regenerates_breaks_and_scales_radius_like_java_update() {
        let mut ability = ForceFieldAbility::new(60.0, 0.1, 200.0, 300.0);
        ability.was_broken = false;
        ability.alpha = 1.0;

        let broken = ability.update_state(0.0, 1.0);
        assert!(broken.broke_this_tick);
        assert!(broken.shield < 0.0);
        assert_eq!(broken.radius_scale, 0.0);
        assert!((broken.alpha - 0.9).abs() < 0.0001);

        let mut active = ForceFieldAbility::new(60.0, 0.1, 200.0, 300.0);
        let update = active.update_state(100.0, 1.0);
        assert!(!update.broke_this_tick);
        assert!((update.shield - 100.1).abs() < 0.0001);
        assert!((update.radius_scale - 0.06).abs() < 0.0001);
        assert!((active.real_radius() - 3.6).abs() < 0.0001);
        assert_eq!(active.created_shield(), 200.0);
    }

    #[test]
    fn force_field_absorbs_only_enemy_absorbable_bullets_inside_polygon() {
        let mut ability = ForceFieldAbility::with_polygon(10.0, 0.1, 100.0, 60.0, 6, 0.0);
        ability.radius_scale = 1.0;

        let hit = ability
            .absorb_bullet(50.0, 1, 2, true, (3.0, 0.0), (0.0, 0.0), 7.0)
            .expect("bullet inside shield should be absorbed");
        assert_eq!(hit.shield_after, 43.0);
        assert_eq!(hit.alpha, 1.0);
        assert!(hit.absorbed);

        assert!(ability
            .absorb_bullet(50.0, 1, 1, true, (3.0, 0.0), (0.0, 0.0), 7.0)
            .is_none());
        assert!(ability
            .absorb_bullet(50.0, 1, 2, false, (3.0, 0.0), (0.0, 0.0), 7.0)
            .is_none());
        assert!(ability
            .absorb_bullet(50.0, 1, 2, true, (30.0, 0.0), (0.0, 0.0), 7.0)
            .is_none());
    }

    #[test]
    fn repair_field_pulses_after_reload_and_heals_all_nearby_targets() {
        let mut ability = RepairFieldAbility::with_percent(4.0, 10.0, 60.0, 5.0);
        ability.same_type_heal_mult = 2.0;
        let targets = [
            RepairFieldTarget {
                damaged: true,
                max_health: 100.0,
                same_type: false,
            },
            RepairFieldTarget {
                damaged: false,
                max_health: 200.0,
                same_type: true,
            },
        ];

        assert!(ability.update_targets(9.0, &targets).is_none());
        let pulse = ability
            .update_targets(1.0, &targets)
            .expect("reload threshold should fire a pulse");

        assert_eq!(pulse.heals, vec![9.0, 28.0]);
        assert!(pulse.active_effect);
        assert_eq!(pulse.timer, 0.0);
        assert!(ability.was_healed);
        assert_eq!(ability.repairs_per_second(), 24.0);
        assert_eq!(ability.repair_percent_per_second(), 30.0);
    }

    #[test]
    fn repair_field_can_pulse_without_active_effect_when_no_target_was_damaged() {
        let mut ability = RepairFieldAbility::new(3.0, 5.0, 40.0);
        let pulse = ability
            .update_targets(
                5.0,
                &[RepairFieldTarget {
                    damaged: false,
                    max_health: 50.0,
                    same_type: false,
                }],
            )
            .expect("reload threshold should fire");

        assert_eq!(pulse.heals, vec![3.0]);
        assert!(!pulse.active_effect);
        assert!(!ability.was_healed);
    }

    #[test]
    fn shield_regen_field_caps_shields_and_reports_active_effect() {
        let mut ability = ShieldRegenFieldAbility::new(25.0, 100.0, 10.0, 60.0);
        let targets = [
            ShieldRegenFieldTarget { shield: 10.0 },
            ShieldRegenFieldTarget { shield: 90.0 },
            ShieldRegenFieldTarget { shield: 100.0 },
        ];

        assert!(ability.update_targets(9.0, &targets).is_none());
        let pulse = ability
            .update_targets(1.0, &targets)
            .expect("reload threshold should fire a pulse");

        assert_eq!(pulse.shields, vec![35.0, 100.0, 100.0]);
        assert!(pulse.active_effect);
        assert!(ability.applied);
        assert_eq!(pulse.timer, 0.0);
        assert_eq!(ability.pulses_per_second(), 6.0);
        assert_eq!(ability.regen_per_second(), 150.0);
    }

    #[test]
    fn shield_regen_field_can_pulse_without_applying_when_everyone_is_full() {
        let mut ability = ShieldRegenFieldAbility::new(10.0, 50.0, 5.0, 40.0);
        let pulse = ability
            .update_targets(5.0, &[ShieldRegenFieldTarget { shield: 50.0 }])
            .expect("reload threshold should fire");

        assert_eq!(pulse.shields, vec![50.0]);
        assert!(!pulse.active_effect);
        assert!(!ability.applied);
    }

    #[test]
    fn status_field_pulses_after_reload_and_offsets_active_effect() {
        let mut ability = StatusFieldAbility::new("overdrive", 120.0, 10.0, 30.0);
        ability.effect_x = 2.0;
        ability.effect_y = 4.0;

        assert!(ability
            .update_targets(9.0, false, 10.0, 20.0, 90.0, 3)
            .is_none());
        let pulse = ability
            .update_targets(1.0, false, 10.0, 20.0, 90.0, 3)
            .expect("reload threshold should fire");

        assert_eq!(pulse.effect, "overdrive");
        assert_eq!(pulse.duration, 120.0);
        assert_eq!(pulse.target_count, 3);
        assert!((pulse.active_x - 8.0).abs() < 0.0001);
        assert!((pulse.active_y - 24.0).abs() < 0.0001);
        assert_eq!(pulse.active_param, 30.0);
        assert_eq!(pulse.timer, 0.0);
        assert_eq!(ability.pulses_per_second(), 6.0);
    }

    #[test]
    fn status_field_can_require_shooting_and_use_rotation_as_effect_param() {
        let mut ability = StatusFieldAbility::new("boost", 30.0, 5.0, 10.0);
        ability.on_shoot = true;
        ability.effect_size_param = false;

        assert!(ability
            .update_targets(5.0, false, 0.0, 0.0, 45.0, 1)
            .is_none());
        let pulse = ability
            .update_targets(0.0, true, 0.0, 0.0, 45.0, 1)
            .expect("stored timer should fire once shooting starts");

        assert_eq!(pulse.active_param, 45.0);
    }

    #[test]
    fn suppression_field_triggers_after_delay_at_rotated_offset() {
        let mut ability = SuppressionFieldAbility {
            x: 0.0,
            y: 10.0,
            max_delay: 5.0,
            reload: 30.0,
            range: 80.0,
            ..Default::default()
        };

        assert!(ability.update_state(4.0, 100.0, 200.0, 90.0).is_none());
        let pulse = ability
            .update_state(1.0, 100.0, 200.0, 90.0)
            .expect("max delay should trigger suppression");

        assert!((pulse.x - 100.0).abs() < 0.0001);
        assert!((pulse.y - 210.0).abs() < 0.0001);
        assert_eq!(pulse.range, 80.0);
        assert_eq!(pulse.reload, 30.0);
        assert_eq!(pulse.max_delay, 5.0);
        assert_eq!(pulse.apply_particle_chance, 13.0);
        assert_eq!(pulse.timer, 0.0);
        assert_eq!(ability.duration_seconds(), 0.5);
    }

    #[test]
    fn suppression_field_does_not_tick_when_inactive() {
        let mut ability = SuppressionFieldAbility {
            active: false,
            max_delay: 1.0,
            ..Default::default()
        };

        assert!(ability.update_state(10.0, 0.0, 0.0, 0.0).is_none());
        assert_eq!(ability.timer, 0.0);
    }

    #[test]
    fn move_effect_emits_when_moving_and_interval_elapsed() {
        let mut ability = MoveEffectAbility::new(0.0, 10.0, 3.0);
        ability.rotate_effect = true;
        ability.rotation = 5.0;
        ability.amount = 2;

        assert!(ability
            .update_plan(2.0, 1.0, false, false, 100.0, 200.0, 90.0, (0.0, 0.0))
            .is_none());

        let plan = ability
            .update_plan(1.0, 1.0, false, false, 100.0, 200.0, 90.0, (0.0, 0.0))
            .expect("elapsed interval should emit effect");
        assert!((plan.x - 100.0).abs() < 0.0001);
        assert!((plan.y - 210.0).abs() < 0.0001);
        assert_eq!(plan.rotation, 95.0);
        assert_eq!(plan.amount, 2);
        assert_eq!(plan.timer, 0.0);
    }

    #[test]
    fn move_effect_respects_velocity_fog_chance_and_range_offsets() {
        let mut ability = MoveEffectAbility::new(1.0, 2.0, 100.0);
        ability.chance = 1.0;
        ability.range_x = 3.0;
        ability.range_y = 4.0;

        assert!(ability
            .update_plan(1.0, 0.0, false, true, 0.0, 0.0, 0.0, (0.0, 0.0))
            .is_none());
        assert!(ability
            .update_plan(1.0, 1.0, true, true, 0.0, 0.0, 0.0, (0.0, 0.0))
            .is_none());

        let plan = ability
            .update_plan(1.0, 1.0, false, true, 10.0, 20.0, 0.0, (99.0, -99.0))
            .expect("chance trigger should emit");
        assert!((plan.x - 8.0).abs() < 0.0001);
        assert!((plan.y - 16.0).abs() < 0.0001);
        assert_eq!(plan.rotation, 3.0);
    }

    #[test]
    fn move_lightning_scales_chance_with_speed_and_alternates_side() {
        let mut ability = MoveLightningAbility::new(40.0, 8, 0.5, 10.0, 1.0, 3.0);
        ability.x = 2.0;
        ability.bullet_angle = 5.0;

        assert_eq!(ability.speed_scale(0.5), 0.0);
        assert_eq!(ability.speed_scale(2.0), 0.5);
        assert_eq!(ability.trigger_probability(2.0, 2.0), 0.5);
        assert!(ability
            .update_plan(1.0, (0.5, 0.0), (100.0, 200.0), 90.0, true, 0.0)
            .is_none());

        let plan = ability
            .update_plan(1.0, (3.0, 4.0), (100.0, 200.0), 90.0, true, -1.0)
            .expect("chance trigger at speed should create lightning");
        assert!((plan.x - 98.0).abs() < 0.0001);
        assert!((plan.y - 210.0).abs() < 0.0001);
        assert!((plan.lightning_x - 101.0).abs() < 0.0001);
        assert!((plan.lightning_y - 214.0).abs() < 0.0001);
        assert_eq!(plan.rotation, 90.0);
        assert_eq!(plan.damage, 40.0);
        assert_eq!(plan.length, 8);
        assert_eq!(plan.bullet_rotation, 94.0);
        assert_eq!(plan.side_after, -1.0);

        let second = ability
            .update_plan(1.0, (3.0, 4.0), (100.0, 200.0), 90.0, true, 0.0)
            .expect("second trigger should use opposite side");
        assert!((second.x - 102.0).abs() < 0.0001);
        assert_eq!(second.side_after, 1.0);
    }

    #[test]
    fn move_lightning_can_disable_alternating() {
        let mut ability = MoveLightningAbility {
            alternate: false,
            chance: 1.0,
            min_speed: 0.0,
            max_speed: 1.0,
            ..Default::default()
        };
        let plan = ability
            .update_plan(1.0, (1.0, 0.0), (0.0, 0.0), 0.0, true, 0.0)
            .expect("always-on chance should create plan");

        assert_eq!(plan.side_after, 1.0);
        assert_eq!(ability.side, 1.0);
    }

    #[test]
    fn armor_plate_warms_up_while_shooting_and_adds_health_multiplier() {
        let mut ability = ArmorPlateAbility::default();

        let update = ability.update_state(true, 1.0);
        assert!((update.warmup - 0.1).abs() < 0.0001);
        assert!((update.health_multiplier_bonus - 0.02).abs() < 0.0001);
        assert!(update.should_draw);
        assert_eq!(ability.damage_reduction_stat_percent(), -20.0);

        let cooldown = ability.update_state(false, 1.0);
        assert!((cooldown.warmup - 0.09).abs() < 0.0001);
    }

    #[test]
    fn armor_plate_respects_draw_flags() {
        let mut ability = ArmorPlateAbility {
            draw_plate: false,
            draw_shine: false,
            ..Default::default()
        };

        let update = ability.update_state(true, 1.0);
        assert!(!update.should_draw);
    }

    #[test]
    fn unit_spawn_accumulates_with_build_speed_and_spawns_at_rotated_offset() {
        let mut ability = UnitSpawnAbility::new("flare", 10.0, 2.0, 4.0);

        assert!(ability
            .update_state(4.0, 2.0, true, 100.0, 200.0, 90.0)
            .is_none());
        assert_eq!(ability.build_progress(), 0.8);

        let plan = ability
            .update_state(1.0, 2.0, true, 100.0, 200.0, 90.0)
            .expect("timer should reach spawn time");
        assert_eq!(plan.unit, "flare");
        assert!((plan.x - 102.0).abs() < 0.0001);
        assert!((plan.y - 204.0).abs() < 0.0001);
        assert_eq!(plan.rotation, 90.0);
        assert_eq!(plan.timer, 0.0);
        assert_eq!(ability.build_time_seconds(), 10.0 / 60.0);
    }

    #[test]
    fn unit_spawn_descriptor_parses_runtime_ability_entries() {
        let ability = UnitSpawnAbility::from_descriptor("UnitSpawnAbility:flare:60:2:-4:true")
            .expect("colon descriptor should parse");
        assert_eq!(ability.unit, "flare");
        assert_eq!(ability.spawn_time, 60.0);
        assert_eq!(ability.spawn_x, 2.0);
        assert_eq!(ability.spawn_y, -4.0);
        assert!(ability.parentize_effects);

        let call_style = UnitSpawnAbility::from_descriptor("UnitSpawnAbility(mono, 30, 0, 8)")
            .expect("call-style descriptor should parse");
        assert_eq!(call_style.unit, "mono");
        assert_eq!(call_style.spawn_time, 30.0);
        assert_eq!(call_style.spawn_y, 8.0);
        assert!(UnitSpawnAbility::from_descriptor("RepairFieldAbility").is_none());
    }

    #[test]
    fn unit_spawn_waits_when_unit_cap_prevents_creation() {
        let mut ability = UnitSpawnAbility::new("mono", 5.0, 0.0, 0.0);

        assert!(ability
            .update_state(5.0, 1.0, false, 0.0, 0.0, 0.0)
            .is_none());
        assert_eq!(ability.build_progress(), 1.0);

        assert!(ability
            .update_state(0.0, 1.0, true, 0.0, 0.0, 0.0)
            .is_some());
    }

    #[test]
    fn energy_field_sorts_targets_and_creates_heal_damage_hits() {
        let mut ability = EnergyFieldAbility::new(12.0, 10.0, 100.0);
        ability.x = 0.0;
        ability.y = 10.0;
        ability.heal_percent = 5.0;
        ability.same_type_heal_mult = 2.0;
        ability.max_targets = 3;
        let targets = [
            EnergyFieldTarget {
                id: 1,
                x: 100.0,
                y: 210.0,
                air: false,
                targetable: true,
                same_team: false,
                damaged: false,
                max_health: 100.0,
                same_type: false,
            },
            EnergyFieldTarget {
                id: 2,
                x: 100.0,
                y: 212.0,
                air: false,
                targetable: true,
                same_team: true,
                damaged: true,
                max_health: 200.0,
                same_type: true,
            },
            EnergyFieldTarget {
                id: 3,
                x: 500.0,
                y: 500.0,
                air: false,
                targetable: true,
                same_team: false,
                damaged: false,
                max_health: 100.0,
                same_type: false,
            },
        ];

        assert!(ability
            .update_targets(9.0, 100.0, 200.0, 90.0, 1.5, 4, true, &targets)
            .is_none());
        let pulse = ability
            .update_targets(1.0, 100.0, 200.0, 90.0, 1.5, 4, true, &targets)
            .expect("reload threshold should fire");

        assert!((pulse.x - 100.0).abs() < 0.0001);
        assert!((pulse.y - 210.0).abs() < 0.0001);
        assert!(pulse.any_nearby);
        assert_eq!(pulse.ammo_after, 3);
        assert_eq!(pulse.hits.len(), 2);
        assert_eq!(pulse.hits[0].id, 1);
        assert_eq!(pulse.hits[0].action, EnergyFieldAction::Damage);
        assert_eq!(pulse.hits[0].amount, 18.0);
        assert_eq!(pulse.hits[0].status.as_deref(), Some("electrified"));
        assert_eq!(pulse.hits[0].status_duration, 60.0 * 6.0);
        assert_eq!(pulse.hits[1].id, 2);
        assert_eq!(pulse.hits[1].action, EnergyFieldAction::Heal);
        assert_eq!(pulse.hits[1].amount, 20.0);
        assert_eq!(ability.firing_rate_per_second(), 6.0);
    }

    #[test]
    fn energy_field_descriptor_parses_aegires_runtime_entry() {
        let ability =
            EnergyFieldAbility::from_descriptor("EnergyFieldAbility:40:65:180:1.5:0.5:25")
                .expect("aegires descriptor should parse");
        assert_eq!(ability.damage, 40.0);
        assert_eq!(ability.reload, 65.0);
        assert_eq!(ability.range, 180.0);
        assert_eq!(ability.heal_percent, 1.5);
        assert_eq!(ability.same_type_heal_mult, 0.5);
        assert_eq!(ability.max_targets, 25);
        assert_eq!(ability.status, "electrified");
        assert!(EnergyFieldAbility::from_descriptor("ForceFieldAbility").is_none());
    }

    #[test]
    fn energy_field_waits_for_ammo_when_rules_require_it() {
        let mut ability = EnergyFieldAbility::new(10.0, 1.0, 50.0);
        assert!(ability
            .update_targets(1.0, 0.0, 0.0, 0.0, 1.0, 0, true, &[])
            .is_none());

        let pulse = ability
            .update_targets(0.0, 0.0, 0.0, 0.0, 1.0, 0, false, &[])
            .expect("ammo is ignored when unit ammo rule is disabled");
        assert!(!pulse.any_nearby);
        assert_eq!(pulse.ammo_after, 0);
    }

    #[test]
    fn shield_arc_regenerates_activates_and_scales_width() {
        let mut ability = ShieldArcAbility::default();
        ability.data = 50.0;
        ability.x = 0.0;
        ability.y = 10.0;

        let update = ability.update_state(1.0, true, 100.0, 200.0, 90.0);
        assert!((update.data - 50.1).abs() < 0.0001);
        assert!(update.active);
        assert!((update.width_scale - 0.06).abs() < 0.0001);
        assert!((update.x - 100.0).abs() < 0.0001);
        assert!((update.y - 210.0).abs() < 0.0001);

        ability.when_shooting = true;
        let inactive = ability.update_state(1.0, false, 100.0, 200.0, 90.0);
        assert!(!inactive.active);
        assert!(inactive.width_scale < update.width_scale);

        assert_eq!(ability.created_shield(), ability.max);
    }

    #[test]
    fn shield_arc_contains_points_in_ring_and_angle() {
        let ability = ShieldArcAbility {
            radius: 10.0,
            width: 2.0,
            angle: 90.0,
            angle_offset: 0.0,
            ..Default::default()
        };

        assert!(ability.contains_point(0.0, 0.0, 0.0, 10.0, 0.0));
        assert!(!ability.contains_point(0.0, 0.0, 0.0, 0.0, 10.0));
        assert!(!ability.contains_point(0.0, 0.0, 0.0, 20.0, 0.0));
    }

    #[test]
    fn shield_arc_absorbs_deflects_and_breaks_from_bullet_hits() {
        let mut ability = ShieldArcAbility {
            data: 20.0,
            chance_deflect: 1.0,
            cooldown: 100.0,
            regen: 0.1,
            ..Default::default()
        };

        let deflect = ability.apply_bullet_hit(5.0, 3.0, true, 1.0, true);
        assert_eq!(deflect.action, ShieldArcHitAction::Deflect);
        assert_eq!(deflect.data_after, 17.0);
        assert_eq!(deflect.alpha, 1.0);
        assert!(!deflect.broke);

        let absorb = ability.apply_bullet_hit(30.0, 2.0, false, 1.0, false);
        assert_eq!(absorb.action, ShieldArcHitAction::Absorb);
        assert!(absorb.broke);
        assert_eq!(absorb.data_after, 5.0);
    }
}
