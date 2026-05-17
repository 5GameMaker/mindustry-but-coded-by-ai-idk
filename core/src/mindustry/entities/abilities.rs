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

#[cfg(test)]
mod tests {
    use super::{
        Ability, BasicAbility, LiquidExplodeAbility, LiquidRegenAbility, RegenAbility,
        SpawnDeathAbility,
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
}
