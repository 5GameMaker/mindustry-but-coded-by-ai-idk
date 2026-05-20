use crate::mindustry::ctype::{Content, ContentId, ContentType, MappableContentBase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitStanceKind {
    Standard,
    Item { item: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitStance {
    pub base: MappableContentBase,
    pub icon: String,
    pub keybind: Option<String>,
    pub incompatible_commands: Vec<String>,
    pub incompatible_stances: Vec<String>,
    pub incompatible_command_bits: u64,
    pub incompatible_stance_bits: u64,
    pub toggle: bool,
    pub kind: UnitStanceKind,
}

impl UnitStance {
    pub fn new(
        id: ContentId,
        name: impl Into<String>,
        icon: impl Into<String>,
        keybind: Option<impl Into<String>>,
        toggle: bool,
    ) -> Self {
        Self {
            base: MappableContentBase::new(id, ContentType::UnitStance, name),
            icon: icon.into(),
            keybind: keybind.map(Into::into),
            incompatible_commands: Vec::new(),
            incompatible_stances: Vec::new(),
            incompatible_command_bits: 0,
            incompatible_stance_bits: 0,
            toggle,
            kind: UnitStanceKind::Standard,
        }
    }

    pub fn item(id: ContentId, item: impl Into<String>) -> Self {
        let item = item.into();
        let mut stance = Self::new(
            id,
            format!("item-{item}"),
            format!("item-{item}"),
            None::<String>,
            true,
        );
        stance.incompatible_stances.push("mineauto".into());
        stance.kind = UnitStanceKind::Item { item };
        stance
    }

    pub fn name(&self) -> &str {
        &self.base.name
    }

    pub fn localized_key(&self) -> String {
        match &self.kind {
            UnitStanceKind::Standard => format!("stance.{}", self.name()),
            UnitStanceKind::Item { item } => format!("stance.mine:{item}"),
        }
    }

    pub fn to_java_string(&self) -> String {
        format!("UnitStance:{}", self.name())
    }

    pub fn is_compatible_with_command_id(&self, command_id: ContentId) -> bool {
        command_id < 0 || (self.incompatible_command_bits & (1u64 << command_id as u32)) == 0
    }
}

impl Content for UnitStance {
    fn id(&self) -> ContentId {
        self.base.base.id
    }

    fn content_type(&self) -> ContentType {
        ContentType::UnitStance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_stance_defaults_match_java_constructor_shape() {
        let stance = UnitStance::new(0, "stop", "cancel", Some("cancelOrders"), false);

        assert_eq!(stance.id(), 0);
        assert_eq!(stance.content_type(), ContentType::UnitStance);
        assert_eq!(stance.name(), "stop");
        assert_eq!(stance.icon, "cancel");
        assert_eq!(stance.keybind.as_deref(), Some("cancelOrders"));
        assert!(!stance.toggle);
        assert!(stance.incompatible_commands.is_empty());
        assert!(stance.incompatible_stances.is_empty());
        assert_eq!(stance.incompatible_command_bits, 0);
        assert_eq!(stance.incompatible_stance_bits, 0);
    }

    #[test]
    fn item_unit_stance_matches_java_name_icon_and_localization_shape() {
        let stance = UnitStance::item(8, "copper");

        assert_eq!(stance.id(), 8);
        assert_eq!(stance.name(), "item-copper");
        assert_eq!(stance.icon, "item-copper");
        assert!(stance.toggle);
        assert_eq!(
            stance.kind,
            UnitStanceKind::Item {
                item: "copper".into()
            }
        );
        assert!(stance
            .incompatible_stances
            .iter()
            .any(|entry| entry == "mineauto"));
        assert_eq!(stance.localized_key(), "stance.mine:copper");
        assert_eq!(stance.to_java_string(), "UnitStance:item-copper");
    }

    #[test]
    fn stance_command_compatibility_uses_java_id_bitset_shape() {
        let mut stance = UnitStance::new(3, "patrol", "refresh", None::<String>, true);
        stance.incompatible_command_bits = (1 << 1) | (1 << 2);

        assert!(stance.is_compatible_with_command_id(0));
        assert!(!stance.is_compatible_with_command_id(1));
        assert!(!stance.is_compatible_with_command_id(2));
        assert!(stance.is_compatible_with_command_id(3));
    }
}
