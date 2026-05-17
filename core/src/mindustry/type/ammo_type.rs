#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AmmoUnit {
    pub ammo: i32,
    pub ammo_capacity: i32,
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
