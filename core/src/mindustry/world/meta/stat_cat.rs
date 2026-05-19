#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatCat {
    General,
    Power,
    Liquids,
    Items,
    Crafting,
    Function,
    Optional,
}

impl StatCat {
    pub const ALL: [StatCat; 7] = [
        StatCat::General,
        StatCat::Power,
        StatCat::Liquids,
        StatCat::Items,
        StatCat::Crafting,
        StatCat::Function,
        StatCat::Optional,
    ];

    pub const fn id(self) -> usize {
        self as usize
    }

    pub const fn name(self) -> &'static str {
        match self {
            StatCat::General => "general",
            StatCat::Power => "power",
            StatCat::Liquids => "liquids",
            StatCat::Items => "items",
            StatCat::Crafting => "crafting",
            StatCat::Function => "function",
            StatCat::Optional => "optional",
        }
    }

    pub fn bundle_key(self) -> String {
        format!("category.{}", self.name())
    }
}

impl core::fmt::Display for StatCat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_categories_keep_java_registration_order_and_keys() {
        let names: Vec<_> = StatCat::ALL.iter().map(|cat| cat.name()).collect();
        assert_eq!(
            names,
            vec!["general", "power", "liquids", "items", "crafting", "function", "optional"]
        );
        assert_eq!(StatCat::General.id(), 0);
        assert_eq!(StatCat::Optional.id(), 6);
        assert_eq!(StatCat::Power.bundle_key(), "category.power");
    }
}
