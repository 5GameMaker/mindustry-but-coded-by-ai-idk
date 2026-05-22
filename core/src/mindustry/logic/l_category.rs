//! Mirrors upstream `mindustry.logic.LCategory`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LCategory {
    pub id: u8,
    pub name: &'static str,
    pub color_rgba: u32,
    pub icon: Option<&'static str>,
}

impl LCategory {
    pub const ALL: [LCategory; 7] = [
        LCategory {
            id: 0,
            name: "unknown",
            color_rgba: 0x4c4c4cff,
            icon: None,
        },
        LCategory {
            id: 1,
            name: "io",
            color_rgba: 0xa08a8aff,
            icon: Some("logicSmall"),
        },
        LCategory {
            id: 2,
            name: "block",
            color_rgba: 0xd4816bff,
            icon: Some("effectSmall"),
        },
        LCategory {
            id: 3,
            name: "operation",
            color_rgba: 0x877badff,
            icon: Some("settingsSmall"),
        },
        LCategory {
            id: 4,
            name: "control",
            color_rgba: 0x6bb2b2ff,
            icon: Some("rotateSmall"),
        },
        LCategory {
            id: 5,
            name: "unit",
            color_rgba: 0xc7b59dff,
            icon: Some("unitsSmall"),
        },
        LCategory {
            id: 6,
            name: "world",
            color_rgba: 0x6b84d4ff,
            icon: Some("terrainSmall"),
        },
    ];

    pub fn by_name(name: &str) -> Option<&'static LCategory> {
        Self::ALL.iter().find(|category| category.name == name)
    }

    pub fn localized_key(self) -> String {
        format!("lcategory.{}", self.name)
    }

    pub fn description_key(self) -> String {
        format!("lcategory.{}.description", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::LCategory;

    #[test]
    fn l_category_static_table_matches_java_order_and_values() {
        assert_eq!(LCategory::ALL.len(), 7);
        assert_eq!(
            LCategory::ALL
                .iter()
                .map(|category| (category.id, category.name, category.icon))
                .collect::<Vec<_>>(),
            vec![
                (0, "unknown", None),
                (1, "io", Some("logicSmall")),
                (2, "block", Some("effectSmall")),
                (3, "operation", Some("settingsSmall")),
                (4, "control", Some("rotateSmall")),
                (5, "unit", Some("unitsSmall")),
                (6, "world", Some("terrainSmall"))
            ]
        );
        assert_eq!(LCategory::ALL[0].color_rgba, 0x4c4c4cff);
        assert_eq!(LCategory::ALL[1].color_rgba, 0xa08a8aff);
        assert_eq!(LCategory::ALL[2].color_rgba, 0xd4816bff);
        assert_eq!(LCategory::ALL[3].color_rgba, 0x877badff);
        assert_eq!(LCategory::ALL[4].color_rgba, 0x6bb2b2ff);
        assert_eq!(LCategory::ALL[5].color_rgba, 0xc7b59dff);
        assert_eq!(LCategory::ALL[6].color_rgba, 0x6b84d4ff);
    }

    #[test]
    fn l_category_lookup_and_keys_match_java_behavior() {
        let unit = LCategory::by_name("unit").unwrap();
        assert_eq!(unit.id, 5);
        assert_eq!(unit.name, "unit");
        assert_eq!(unit.localized_key(), "lcategory.unit");
        assert_eq!(unit.description_key(), "lcategory.unit.description");
        assert_eq!(LCategory::by_name("missing"), None);
    }
}
