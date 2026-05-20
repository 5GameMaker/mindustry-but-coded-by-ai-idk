use crate::mindustry::ctype::{Content, ContentId, ContentType, MappableContentBase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitCommand {
    pub base: MappableContentBase,
    pub icon: String,
    pub controller: Option<String>,
    pub switch_to_move: bool,
    pub draw_target: bool,
    pub reset_target: bool,
    pub snap_to_building: bool,
    pub exact_arrival: bool,
    pub refresh_on_select: bool,
    pub keybind: Option<String>,
    pub extra_stances: Vec<String>,
}

impl UnitCommand {
    pub fn new(
        id: ContentId,
        name: impl Into<String>,
        icon: impl Into<String>,
        keybind: Option<impl Into<String>>,
        controller: Option<impl Into<String>>,
    ) -> Self {
        Self {
            base: MappableContentBase::new(id, ContentType::UnitCommand, name),
            icon: icon.into(),
            controller: controller.map(Into::into),
            switch_to_move: true,
            draw_target: false,
            reset_target: true,
            snap_to_building: false,
            exact_arrival: false,
            refresh_on_select: false,
            keybind: keybind.map(Into::into),
            extra_stances: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.base.name
    }

    pub fn localized_key(&self) -> String {
        format!("command.{}", self.name())
    }

    pub fn to_java_string(&self) -> String {
        format!("UnitCommand:{}", self.name())
    }
}

impl Content for UnitCommand {
    fn id(&self) -> ContentId {
        self.base.base.id
    }

    fn content_type(&self) -> ContentType {
        ContentType::UnitCommand
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_command_defaults_match_java_constructor_shape() {
        let command = UnitCommand::new(
            2,
            "rebuild",
            "hammer",
            Some("unitCommandRebuild"),
            Some("BuilderAI"),
        );

        assert_eq!(command.id(), 2);
        assert_eq!(command.content_type(), ContentType::UnitCommand);
        assert_eq!(command.name(), "rebuild");
        assert_eq!(command.icon, "hammer");
        assert_eq!(command.keybind.as_deref(), Some("unitCommandRebuild"));
        assert_eq!(command.controller.as_deref(), Some("BuilderAI"));
        assert!(command.switch_to_move);
        assert!(!command.draw_target);
        assert!(command.reset_target);
        assert!(!command.snap_to_building);
        assert!(!command.exact_arrival);
        assert!(!command.refresh_on_select);
        assert!(command.extra_stances.is_empty());
    }

    #[test]
    fn unit_command_helpers_match_java_strings() {
        let command = UnitCommand::new(0, "move", "right", None::<String>, None::<String>);

        assert_eq!(command.localized_key(), "command.move");
        assert_eq!(command.to_java_string(), "UnitCommand:move");
    }
}
