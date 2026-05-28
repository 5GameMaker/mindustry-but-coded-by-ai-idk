use core::fmt;

use crate::mindustry::entities::{ShootPattern, ShootSpread, Shot};

#[derive(Debug, Clone, PartialEq)]
pub struct Weapon {
    pub name: String,
    pub bullet: String,
    pub bullet_kill_shooter: bool,
    /// Mirrors Java `bullet.range` while the Rust shell still stores the
    /// projectile by name instead of an owned `BulletType`.
    pub bullet_range: f32,
    pub eject_effect: String,
    pub display: bool,
    pub use_ammo: bool,
    pub mirror: bool,
    pub flip_sprite: bool,
    pub alternate: bool,
    pub rotate: bool,
    pub show_stat_sprite: bool,
    pub base_rotation: f32,
    pub top: bool,
    pub continuous: bool,
    pub always_continuous: bool,
    pub aim_change_speed: f32,
    pub controllable: bool,
    pub ai_controllable: bool,
    pub always_shooting: bool,
    pub auto_target: bool,
    pub predict_target: bool,
    pub use_attack_range: bool,
    pub target_interval: f32,
    pub target_switch_interval: f32,
    pub rotate_speed: f32,
    pub reload: f32,
    pub inaccuracy: f32,
    pub shake: f32,
    pub recoil: f32,
    pub recoils: i32,
    pub recoil_time: f32,
    pub recoil_pow: f32,
    pub cooldown_time: f32,
    pub shoot_x: f32,
    pub shoot_y: f32,
    pub x: f32,
    pub y: f32,
    pub x_rand: f32,
    pub y_rand: f32,
    pub shoot_pattern: String,
    pub shoot_shots: i32,
    pub shoot_spread: f32,
    pub shoot_first_shot_delay: f32,
    pub shoot_shot_delay: f32,
    pub shadow: f32,
    pub velocity_rnd: f32,
    pub extra_velocity: f32,
    pub shoot_cone: f32,
    pub rotation_limit: f32,
    pub min_warmup: f32,
    pub shoot_warmup_speed: f32,
    pub smooth_reload_speed: f32,
    pub linear_warmup: bool,
    pub sound_pitch_min: f32,
    pub sound_pitch_max: f32,
    pub ignore_rotation: bool,
    pub no_attack: bool,
    pub min_shoot_velocity: f32,
    pub parentize_effects: bool,
    pub other_side: i32,
    pub layer_offset: f32,
    pub active_sound: String,
    pub active_sound_volume: f32,
    pub shoot_sound: String,
    pub shoot_sound_volume: f32,
    pub initial_shoot_sound: String,
    pub charge_sound: String,
    pub region: Option<String>,
    pub heat_region: Option<String>,
    pub cell_region: Option<String>,
    pub outline_region: Option<String>,
    pub heat_color_rgba: u32,
    pub shoot_status: String,
    pub mount_type: String,
    pub shoot_status_duration: f32,
    pub shoot_on_death: bool,
    pub shoot_on_death_effect: Option<String>,
    pub parts: Vec<String>,
}

impl Weapon {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bullet: String::new(),
            bullet_kill_shooter: false,
            bullet_range: 0.0,
            eject_effect: String::new(),
            display: true,
            use_ammo: true,
            mirror: true,
            flip_sprite: false,
            alternate: true,
            rotate: false,
            show_stat_sprite: true,
            base_rotation: 0.0,
            top: true,
            continuous: false,
            always_continuous: false,
            aim_change_speed: f32::INFINITY,
            controllable: true,
            ai_controllable: true,
            always_shooting: false,
            auto_target: false,
            predict_target: true,
            use_attack_range: true,
            target_interval: 40.0,
            target_switch_interval: 70.0,
            rotate_speed: 20.0,
            reload: 1.0,
            inaccuracy: 0.0,
            shake: 0.0,
            recoil: 1.5,
            recoils: -1,
            recoil_time: -1.0,
            recoil_pow: 1.8,
            cooldown_time: 20.0,
            shoot_x: 0.0,
            shoot_y: 3.0,
            x: 5.0,
            y: 0.0,
            x_rand: 0.0,
            y_rand: 0.0,
            shoot_pattern: String::new(),
            shoot_shots: 1,
            shoot_spread: 0.0,
            shoot_first_shot_delay: 0.0,
            shoot_shot_delay: 0.0,
            shadow: -1.0,
            velocity_rnd: 0.0,
            extra_velocity: 0.0,
            shoot_cone: 5.0,
            rotation_limit: 361.0,
            min_warmup: 0.0,
            shoot_warmup_speed: 0.1,
            smooth_reload_speed: 0.15,
            linear_warmup: false,
            sound_pitch_min: 0.8,
            sound_pitch_max: 1.0,
            ignore_rotation: false,
            no_attack: false,
            min_shoot_velocity: -1.0,
            parentize_effects: false,
            other_side: -1,
            layer_offset: 0.0,
            active_sound: String::new(),
            active_sound_volume: 1.0,
            shoot_sound: String::new(),
            shoot_sound_volume: 1.0,
            initial_shoot_sound: String::new(),
            charge_sound: String::new(),
            region: None,
            heat_region: None,
            cell_region: None,
            outline_region: None,
            heat_color_rgba: 0xffa3f2ff,
            shoot_status: String::new(),
            mount_type: String::new(),
            shoot_status_duration: 300.0,
            shoot_on_death: false,
            shoot_on_death_effect: None,
            parts: Vec::new(),
        }
    }

    pub fn range(&self) -> f32 {
        self.bullet_range
    }

    pub fn shots_per_sec(&self, shots: f32) -> f32 {
        if self.reload <= 0.0 {
            0.0
        } else {
            shots * 60.0 / self.reload
        }
    }

    pub fn shoot_shots(&self) -> i32 {
        self.shoot_shots.max(1)
    }

    pub fn shoot_pattern_shots(&self, total_shots: i32) -> Vec<Shot> {
        let mut shots = Vec::new();
        if self.shoot_pattern == "ShootSpread" {
            let pattern = ShootSpread::new(self.shoot_shots(), self.shoot_spread);
            pattern.shoot(total_shots, &mut |shot| shots.push(shot), None);
        } else {
            let mut pattern = ShootPattern::new();
            pattern.shots = self.shoot_shots();
            pattern.first_shot_delay = self.shoot_first_shot_delay;
            pattern.shot_delay = self.shoot_shot_delay;
            pattern.shoot(total_shots, &mut |shot| shots.push(shot), None);
        }
        shots
    }

    pub fn dps(&self, bullet_damage: f32, shots: f32) -> f32 {
        if self.reload <= 0.0 {
            0.0
        } else {
            (bullet_damage / self.reload) * shots * 60.0
        }
    }

    pub fn flip(&mut self) {
        self.x = -self.x;
        self.shoot_x = -self.shoot_x;
        self.base_rotation = -self.base_rotation;
        self.flip_sprite = !self.flip_sprite;
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn init(&mut self) {
        if self.always_continuous {
            self.continuous = true;
        }
    }

    pub fn load(&mut self) {
        self.region = Some(self.name.clone());
        self.heat_region = Some(format!("{}-heat", self.name));
        self.cell_region = Some(format!("{}-cell", self.name));
        self.outline_region = Some(format!("{}-outline", self.name));
    }
}

impl Default for Weapon {
    fn default() -> Self {
        Self::new("")
    }
}

impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.name.is_empty() {
            write!(f, "Weapon")
        } else {
            write!(f, "Weapon: {}", self.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_range_uses_bullet_range_not_shoot_cone_like_java() {
        let mut weapon = Weapon::new("scattershot");
        weapon.bullet_range = 175.0;
        weapon.shoot_cone = 12.0;

        assert_eq!(weapon.range(), 175.0);
        assert!(!weapon.bullet_kill_shooter);
    }

    #[test]
    fn weapon_shoot_shots_mirrors_java_shoot_pattern_minimum() {
        let mut weapon = Weapon::new("burst");
        assert_eq!(weapon.shoot_shots(), 1);
        assert_eq!(weapon.shoot_spread, 0.0);

        weapon.shoot_shots = 3;
        weapon.shoot_spread = 12.0;
        assert_eq!(weapon.shoot_shots(), 3);
        assert_eq!(weapon.shoot_spread, 12.0);

        weapon.shoot_shots = 0;
        assert_eq!(weapon.shoot_shots(), 1);
    }

    #[test]
    fn weapon_shoot_pattern_shots_reuses_core_spread_pattern() {
        let mut weapon = Weapon::new("spread");
        weapon.shoot_pattern = "ShootSpread".into();
        weapon.shoot_shots = 3;
        weapon.shoot_spread = 10.0;

        let shots = weapon.shoot_pattern_shots(0);

        assert_eq!(shots.len(), 3);
        assert_eq!(shots[0].rotation, -10.0);
        assert_eq!(shots[1].rotation, 0.0);
        assert_eq!(shots[2].rotation, 10.0);
    }

    #[test]
    fn weapon_flip_keeps_bullet_range_and_mirrors_offsets_like_java_shell() {
        let mut weapon = Weapon::new("left");
        weapon.x = 6.0;
        weapon.shoot_x = 1.5;
        weapon.base_rotation = 20.0;
        weapon.bullet_range = 120.0;

        weapon.flip();

        assert_eq!(weapon.x, -6.0);
        assert_eq!(weapon.shoot_x, -1.5);
        assert_eq!(weapon.base_rotation, -20.0);
        assert!(weapon.flip_sprite);
        assert_eq!(weapon.bullet_range, 120.0);
    }

    #[test]
    fn weapon_load_populates_regions_like_java_load() {
        let mut weapon = Weapon::new("duo");

        weapon.load();

        assert_eq!(weapon.region.as_deref(), Some("duo"));
        assert_eq!(weapon.heat_region.as_deref(), Some("duo-heat"));
        assert_eq!(weapon.cell_region.as_deref(), Some("duo-cell"));
        assert_eq!(weapon.outline_region.as_deref(), Some("duo-outline"));
    }

    #[test]
    fn weapon_display_matches_java_tostring() {
        assert_eq!(Weapon::new("").to_string(), "Weapon");
        assert_eq!(Weapon::new("duo").to_string(), "Weapon: duo");
    }
}
