//! Weapons component mirroring upstream `mindustry.entities.comp.WeaponsComp`.

use crate::mindustry::entities::units::WeaponMount;
use crate::mindustry::r#type::Weapon;

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponsComp {
    pub x: f32,
    pub y: f32,
    pub disarmed: bool,
    pub ammo_capacity: f32,
    pub aim_dst: f32,
    pub mounts: Vec<WeaponMount>,
    pub is_rotate: bool,
    pub aim_x: f32,
    pub aim_y: f32,
    pub is_shooting: bool,
    pub ammo: f32,
    pub mount_update_calls: usize,
}

impl WeaponsComp {
    pub fn new(ammo_capacity: f32, aim_dst: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            disarmed: false,
            ammo_capacity,
            aim_dst,
            mounts: Vec::new(),
            is_rotate: false,
            aim_x: 0.0,
            aim_y: 0.0,
            is_shooting: false,
            ammo: 0.0,
            mount_update_calls: 0,
        }
    }

    pub fn ammof(&self) -> f32 {
        self.ammo / self.ammo_capacity
    }

    pub fn set_weapon_rotation(&mut self, rotation: f32) {
        for mount in &mut self.mounts {
            mount.rotation = rotation;
        }
    }

    pub fn setup_weapons(&mut self, weapons: impl IntoIterator<Item = Weapon>) {
        self.mounts = weapons.into_iter().map(WeaponMount::new).collect();
    }

    pub fn control_weapons_same(&mut self, rotate_shoot: bool) {
        self.control_weapons(rotate_shoot, rotate_shoot);
    }

    pub fn control_weapons(&mut self, rotate: bool, shoot: bool) {
        for mount in &mut self.mounts {
            if mount.weapon.controllable {
                mount.rotate = rotate;
                mount.shoot = shoot;
            }
        }
        self.is_rotate = rotate;
        self.is_shooting = shoot;
    }

    pub fn aim(&mut self, x: f32, y: f32) {
        let mut dx = x - self.x;
        let mut dy = y - self.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < self.aim_dst && len > 0.0 {
            let scale = self.aim_dst / len;
            dx *= scale;
            dy *= scale;
        }

        let aim_x = dx + self.x;
        let aim_y = dy + self.y;

        for mount in &mut self.mounts {
            if mount.weapon.controllable {
                mount.aim_x = aim_x;
                mount.aim_y = aim_y;
            }
        }

        self.aim_x = aim_x;
        self.aim_y = aim_y;
    }

    pub fn can_shoot(&self) -> bool {
        !self.disarmed
    }

    pub fn remove(&mut self) {
        for mount in &mut self.mounts {
            if mount.weapon.continuous && mount.bullet.is_some() {
                mount.bullet = None;
            }

            if mount.sound.is_some() {
                mount.sound = None;
            }
        }
    }

    pub fn update(&mut self) {
        self.mount_update_calls += self.mounts.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weapon(name: &str, controllable: bool, continuous: bool) -> Weapon {
        let mut weapon = Weapon::new(name);
        weapon.controllable = controllable;
        weapon.continuous = continuous;
        weapon.base_rotation = 15.0;
        weapon
    }

    #[test]
    fn weapons_component_sets_up_mounts_and_ammo_fraction() {
        let mut comp = WeaponsComp::new(100.0, 10.0);
        comp.ammo = 25.0;
        comp.setup_weapons([weapon("a", true, false), weapon("b", false, false)]);

        assert_eq!(comp.ammof(), 0.25);
        assert_eq!(comp.mounts.len(), 2);
        assert_eq!(comp.mounts[0].rotation, 15.0);
    }

    #[test]
    fn weapons_component_control_and_aim_only_affect_controllable_mounts() {
        let mut comp = WeaponsComp::new(100.0, 10.0);
        comp.x = 0.0;
        comp.y = 0.0;
        comp.setup_weapons([weapon("a", true, false), weapon("b", false, false)]);

        comp.control_weapons(true, true);
        assert!(comp.mounts[0].rotate);
        assert!(comp.mounts[0].shoot);
        assert!(!comp.mounts[1].rotate);
        assert!(!comp.mounts[1].shoot);

        comp.aim(3.0, 4.0);
        assert_eq!((comp.aim_x, comp.aim_y), (6.0, 8.0));
        assert_eq!((comp.mounts[0].aim_x, comp.mounts[0].aim_y), (6.0, 8.0));
        assert_eq!((comp.mounts[1].aim_x, comp.mounts[1].aim_y), (0.0, 0.0));
    }

    #[test]
    fn weapons_component_remove_stops_continuous_bullets_and_sounds() {
        let mut comp = WeaponsComp::new(100.0, 10.0);
        comp.setup_weapons([
            weapon("continuous", true, true),
            weapon("single", true, false),
        ]);
        comp.mounts[0].bullet = Some("beam".into());
        comp.mounts[0].sound = Some("loop".into());
        comp.mounts[1].bullet = Some("shot".into());
        comp.mounts[1].sound = Some("sound".into());

        comp.remove();

        assert_eq!(comp.mounts[0].bullet, None);
        assert_eq!(comp.mounts[0].sound, None);
        assert_eq!(comp.mounts[1].bullet, Some("shot".into()));
        assert_eq!(comp.mounts[1].sound, None);
    }
}
