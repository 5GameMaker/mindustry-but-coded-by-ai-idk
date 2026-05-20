pub const PAL_AMMO_RGBA: u32 = 0xff8947ff;
pub const PAL_POWER_LIGHT_RGBA: u32 = 0xfbd367ff;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AmmoUnit {
    pub ammo: f32,
    pub ammo_capacity: f32,
}

/// Type of ammo that a unit uses.
pub trait AmmoType {
    fn icon(&self) -> String;
    fn color(&self) -> u32;
    fn bar_color(&self) -> u32;
    fn resupply(&self, unit: &mut AmmoUnit);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicAmmoType {
    pub icon_name: String,
    pub color_rgba: u32,
    pub bar_color_rgba: u32,
}

impl BasicAmmoType {
    pub fn new(icon_name: impl Into<String>, color_rgba: u32, bar_color_rgba: u32) -> Self {
        Self {
            icon_name: icon_name.into(),
            color_rgba,
            bar_color_rgba,
        }
    }
}

impl Default for BasicAmmoType {
    fn default() -> Self {
        Self::new("", 0xffffffff, 0xffffffff)
    }
}

impl AmmoType for BasicAmmoType {
    fn icon(&self) -> String {
        self.icon_name.clone()
    }

    fn color(&self) -> u32 {
        self.color_rgba
    }

    fn bar_color(&self) -> u32 {
        self.bar_color_rgba
    }

    fn resupply(&self, unit: &mut AmmoUnit) {
        unit.ammo = unit.ammo_capacity;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemAmmoType {
    pub range: f32,
    pub ammo_per_item: i32,
    pub item_name: String,
    pub item_color_rgba: u32,
    pub item_icon: String,
    pub bar_color_rgba: u32,
}

impl ItemAmmoType {
    pub const DEFAULT_RANGE: f32 = 85.0;
    pub const DEFAULT_AMMO_PER_ITEM: i32 = 15;

    pub fn new(item_name: impl Into<String>, item_color_rgba: u32) -> Self {
        Self::with_ammo_per_item(item_name, item_color_rgba, Self::DEFAULT_AMMO_PER_ITEM)
    }

    pub fn with_ammo_per_item(
        item_name: impl Into<String>,
        item_color_rgba: u32,
        ammo_per_item: i32,
    ) -> Self {
        let item_name = item_name.into();
        Self {
            range: Self::DEFAULT_RANGE,
            ammo_per_item,
            item_icon: item_name.clone(),
            item_name,
            item_color_rgba,
            bar_color_rgba: PAL_AMMO_RGBA,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.item_icon = icon.into();
        self
    }

    pub fn resupply_range(&self, unit_hit_size: f32) -> f32 {
        unit_hit_size + self.range
    }

    pub fn can_accept_item_resupply(&self, unit: &AmmoUnit) -> bool {
        unit.ammo_capacity - unit.ammo >= self.ammo_per_item as f32
    }

    pub fn resupply_from_available(&self, unit: &mut AmmoUnit, available_items: &mut i32) -> bool {
        if *available_items <= 0 || !self.can_accept_item_resupply(unit) {
            return false;
        }

        unit.ammo = (unit.ammo + self.ammo_per_item as f32).min(unit.ammo_capacity);
        *available_items -= 1;
        true
    }
}

impl AmmoType for ItemAmmoType {
    fn icon(&self) -> String {
        self.item_icon.clone()
    }

    fn color(&self) -> u32 {
        self.item_color_rgba
    }

    fn bar_color(&self) -> u32 {
        self.bar_color_rgba
    }

    fn resupply(&self, _unit: &mut AmmoUnit) {
        // Java searches the world for a nearby building with the configured item.
        // The world/provider lookup is represented by `resupply_from_available`.
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PowerAmmoType {
    pub range: f32,
    pub total_power: f32,
    pub icon_name: String,
    pub color_rgba: u32,
    pub bar_color_rgba: u32,
}

impl PowerAmmoType {
    pub const DEFAULT_RANGE: f32 = 85.0;
    pub const DEFAULT_TOTAL_POWER: f32 = 1000.0;

    pub fn new(total_power: f32) -> Self {
        Self {
            range: Self::DEFAULT_RANGE,
            total_power,
            icon_name: "power".to_string(),
            color_rgba: PAL_POWER_LIGHT_RGBA,
            bar_color_rgba: PAL_POWER_LIGHT_RGBA,
        }
    }

    pub fn resupply_range(&self, unit_hit_size: f32) -> f32 {
        unit_hit_size + self.range
    }

    pub fn power_per_ammo(&self, ammo_capacity: f32) -> f32 {
        self.total_power / ammo_capacity
    }

    pub fn resupply_from_buffer(
        &self,
        unit: &mut AmmoUnit,
        power_status: &mut f32,
        power_capacity: f32,
    ) -> f32 {
        if unit.ammo_capacity <= 0.0 || power_capacity <= 0.0 {
            return 0.0;
        }

        let amount = *power_status * power_capacity;
        let power_per_ammo = self.power_per_ammo(unit.ammo_capacity);
        let ammo_required = unit.ammo_capacity - unit.ammo;
        let power_required = ammo_required * power_per_ammo;
        let power_taken = amount.min(power_required);

        if power_taken > 1.0 {
            *power_status -= power_taken / power_capacity;
            unit.ammo += power_taken / power_per_ammo;
            power_taken
        } else {
            0.0
        }
    }

    pub fn transfer_effect_scale(power_taken: f32) -> f32 {
        (power_taken / 100.0).max(1.0)
    }
}

impl Default for PowerAmmoType {
    fn default() -> Self {
        Self::new(Self::DEFAULT_TOTAL_POWER)
    }
}

impl AmmoType for PowerAmmoType {
    fn icon(&self) -> String {
        self.icon_name.clone()
    }

    fn color(&self) -> u32 {
        self.color_rgba
    }

    fn bar_color(&self) -> u32 {
        self.bar_color_rgba
    }

    fn resupply(&self, _unit: &mut AmmoUnit) {
        // Java searches the world for a nearby buffered power consumer.
        // The power-transfer math is represented by `resupply_from_buffer`.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_ammo_type_fills_unit_capacity() {
        let ammo_type = BasicAmmoType::new("ammo", 0x112233ff, 0x445566ff);
        let mut unit = AmmoUnit {
            ammo: 2.5,
            ammo_capacity: 10.0,
        };

        assert_eq!(ammo_type.icon(), "ammo");
        assert_eq!(ammo_type.color(), 0x112233ff);
        assert_eq!(ammo_type.bar_color(), 0x445566ff);

        ammo_type.resupply(&mut unit);
        assert_eq!(unit.ammo, 10.0);
    }

    #[test]
    fn item_ammo_type_defaults_match_java_constructors() {
        let item = ItemAmmoType::new("copper", 0xd99d73ff);
        assert_eq!(item.range, 85.0);
        assert_eq!(item.ammo_per_item, 15);
        assert_eq!(item.item_name, "copper");
        assert_eq!(item.icon(), "copper");
        assert_eq!(item.color(), 0xd99d73ff);
        assert_eq!(item.bar_color(), PAL_AMMO_RGBA);
        assert_eq!(item.resupply_range(12.0), 97.0);

        let graphite =
            ItemAmmoType::with_ammo_per_item("graphite", 0xb2c6d2ff, 30).with_icon("\u{f83a}");
        assert_eq!(graphite.ammo_per_item, 30);
        assert_eq!(graphite.icon(), "\u{f83a}");
    }

    #[test]
    fn item_ammo_resupply_uses_one_item_only_when_missing_capacity_is_large_enough() {
        let item = ItemAmmoType::new("copper", 0xd99d73ff);
        let mut unit = AmmoUnit {
            ammo: 80.0,
            ammo_capacity: 100.0,
        };
        let mut available = 1;

        assert!(item.can_accept_item_resupply(&unit));
        assert!(item.resupply_from_available(&mut unit, &mut available));
        assert_eq!(unit.ammo, 95.0);
        assert_eq!(available, 0);

        let mut nearly_full = AmmoUnit {
            ammo: 90.1,
            ammo_capacity: 100.0,
        };
        let mut available = 1;
        assert!(!item.can_accept_item_resupply(&nearly_full));
        assert!(!item.resupply_from_available(&mut nearly_full, &mut available));
        assert_eq!(nearly_full.ammo, 90.1);
        assert_eq!(available, 1);
    }

    #[test]
    fn power_ammo_type_defaults_and_buffer_math_match_java_resupply_formula() {
        let power = PowerAmmoType::default();
        assert_eq!(power.range, 85.0);
        assert_eq!(power.total_power, 1000.0);
        assert_eq!(power.icon(), "power");
        assert_eq!(power.color(), PAL_POWER_LIGHT_RGBA);
        assert_eq!(power.bar_color(), PAL_POWER_LIGHT_RGBA);
        assert_eq!(power.resupply_range(10.0), 95.0);
        assert_eq!(power.power_per_ammo(40.0), 25.0);

        let mut unit = AmmoUnit {
            ammo: 10.0,
            ammo_capacity: 40.0,
        };
        let mut power_status = 1.0;
        let taken = power.resupply_from_buffer(&mut unit, &mut power_status, 1000.0);
        assert_eq!(taken, 750.0);
        assert_eq!(unit.ammo, 40.0);
        assert_eq!(power_status, 0.25);
        assert_eq!(PowerAmmoType::transfer_effect_scale(taken), 7.5);
    }

    #[test]
    fn power_ammo_resupply_ignores_tiny_transfers_like_java_threshold() {
        let power = PowerAmmoType::default();
        let mut unit = AmmoUnit {
            ammo: 39.99,
            ammo_capacity: 40.0,
        };
        let mut power_status = 1.0;

        let taken = power.resupply_from_buffer(&mut unit, &mut power_status, 1000.0);
        assert_eq!(taken, 0.0);
        assert_eq!(unit.ammo, 39.99);
        assert_eq!(power_status, 1.0);
    }
}
