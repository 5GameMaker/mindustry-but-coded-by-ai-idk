#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Category {
    Turret = 0,
    Production = 1,
    Distribution = 2,
    Liquid = 3,
    Power = 4,
    Defense = 5,
    Crafting = 6,
    Units = 7,
    Effect = 8,
    Logic = 9,
}

impl Category {
    pub const ALL: [Category; 10] = [
        Category::Turret,
        Category::Production,
        Category::Distribution,
        Category::Liquid,
        Category::Power,
        Category::Defense,
        Category::Crafting,
        Category::Units,
        Category::Effect,
        Category::Logic,
    ];

    pub const fn ordinal(self) -> usize {
        self as usize
    }

    pub fn prev(self) -> Self {
        Self::ALL[(self.ordinal() + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn next(self) -> Self {
        Self::ALL[(self.ordinal() + 1) % Self::ALL.len()]
    }

    pub const fn wire_name(self) -> &'static str {
        match self {
            Category::Turret => "turret",
            Category::Production => "production",
            Category::Distribution => "distribution",
            Category::Liquid => "liquid",
            Category::Power => "power",
            Category::Defense => "defense",
            Category::Crafting => "crafting",
            Category::Units => "units",
            Category::Effect => "effect",
            Category::Logic => "logic",
        }
    }
}
