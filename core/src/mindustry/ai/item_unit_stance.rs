use std::collections::HashMap;

use crate::mindustry::ctype::ContentId;

use super::unit_stance::UnitStance;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemUnitStance {
    pub item_name: String,
    pub item_localized_name: String,
    pub stance: UnitStance,
}

impl ItemUnitStance {
    pub fn new(
        id: ContentId,
        item_name: impl Into<String>,
        item_localized_name: impl Into<String>,
        mine_auto_incompatible_stances: &[String],
    ) -> Self {
        let item_name = item_name.into();
        let mut stance = UnitStance::item(id, item_name.clone());
        for incompatible in mine_auto_incompatible_stances {
            if !stance.incompatible_stances.contains(incompatible) {
                stance.incompatible_stances.push(incompatible.clone());
            }
        }
        Self {
            item_name,
            item_localized_name: item_localized_name.into(),
            stance,
        }
    }

    pub fn localized(&self) -> String {
        format!("stance.mine:{}", self.item_localized_name)
    }

    pub fn icon(&self) -> String {
        self.item_name.clone()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemUnitStanceRegistry {
    by_item: HashMap<String, ItemUnitStance>,
    order: Vec<String>,
}

impl ItemUnitStanceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, stance: ItemUnitStance) -> bool {
        if self.by_item.contains_key(&stance.item_name) {
            return false;
        }
        self.order.push(stance.item_name.clone());
        self.by_item.insert(stance.item_name.clone(), stance);
        true
    }

    pub fn get_by_item(&self, item_name: Option<&str>) -> Option<&ItemUnitStance> {
        item_name.and_then(|name| self.by_item.get(name))
    }

    pub fn all(&self) -> Vec<&ItemUnitStance> {
        self.order
            .iter()
            .filter_map(|name| self.by_item.get(name))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.by_item.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_item.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_unit_stance_wraps_unit_stance_name_icon_and_localization() {
        let stance = ItemUnitStance::new(
            12,
            "copper",
            "Copper",
            &["mineAutoExtra".to_string(), "mineauto".to_string()],
        );

        assert_eq!(stance.stance.name(), "item-copper");
        assert_eq!(stance.stance.icon, "item-copper");
        assert_eq!(stance.item_name, "copper");
        assert_eq!(stance.localized(), "stance.mine:Copper");
        assert_eq!(stance.icon(), "copper");
        assert!(stance
            .stance
            .incompatible_stances
            .iter()
            .any(|entry| entry == "mineauto"));
        assert!(stance
            .stance
            .incompatible_stances
            .iter()
            .any(|entry| entry == "mineAutoExtra"));
    }

    #[test]
    fn item_unit_stance_registry_get_by_item_and_all_follow_java_shape() {
        let mut registry = ItemUnitStanceRegistry::new();
        let copper = ItemUnitStance::new(1, "copper", "Copper", &[]);
        let lead = ItemUnitStance::new(2, "lead", "Lead", &[]);

        assert!(registry.register(copper));
        assert!(registry.register(lead));
        assert!(!registry.register(ItemUnitStance::new(3, "lead", "Lead", &[])));

        assert_eq!(registry.len(), 2);
        assert_eq!(
            registry.get_by_item(Some("copper")).unwrap().localized(),
            "stance.mine:Copper"
        );
        assert!(registry.get_by_item(None).is_none());
        assert!(registry.get_by_item(Some("titanium")).is_none());
        assert_eq!(
            registry
                .all()
                .iter()
                .map(|stance| stance.item_name.as_str())
                .collect::<Vec<_>>(),
            vec!["copper", "lead"]
        );
    }
}
