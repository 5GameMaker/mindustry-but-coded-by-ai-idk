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

    pub fn from_ordinal(ordinal: usize) -> Option<Self> {
        Self::ALL.get(ordinal).copied()
    }

    pub fn from_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|category| category.wire_name() == name)
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

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.wire_name())
    }
}

#[cfg(test)]
mod tests {
    use super::Category;

    #[test]
    fn category_order_and_wire_names_match_java_enum() {
        let names: Vec<_> = Category::ALL
            .iter()
            .map(|category| category.wire_name())
            .collect();
        assert_eq!(
            names,
            vec![
                "turret",
                "production",
                "distribution",
                "liquid",
                "power",
                "defense",
                "crafting",
                "units",
                "effect",
                "logic"
            ]
        );

        for (index, category) in Category::ALL.iter().copied().enumerate() {
            assert_eq!(category.ordinal(), index);
            assert_eq!(Category::from_ordinal(index), Some(category));
            assert_eq!(
                Category::from_wire_name(category.wire_name()),
                Some(category)
            );
            assert_eq!(category.to_string(), category.wire_name());
        }
        assert_eq!(Category::from_ordinal(Category::ALL.len()), None);
        assert_eq!(Category::from_wire_name("missing"), None);
    }

    #[test]
    fn category_prev_and_next_wrap_like_java_modulo_helpers() {
        assert_eq!(Category::Turret.prev(), Category::Logic);
        assert_eq!(Category::Turret.next(), Category::Production);
        assert_eq!(Category::Logic.next(), Category::Turret);
        assert_eq!(Category::Logic.prev(), Category::Effect);
    }
}
