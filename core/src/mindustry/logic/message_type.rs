//! Mirrors upstream `mindustry.logic.MessageType`.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageType {
    Notify,
    Announce,
    Toast,
    Mission,
}

impl MessageType {
    pub const ALL: [MessageType; 4] = [
        MessageType::Notify,
        MessageType::Announce,
        MessageType::Toast,
        MessageType::Mission,
    ];
    pub const WIRE_NAMES: [&'static str; 4] = ["notify", "announce", "toast", "mission"];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::MessageType;

    #[test]
    fn message_type_order_and_wire_names_match_java_enum() {
        assert_eq!(
            MessageType::ALL,
            [
                MessageType::Notify,
                MessageType::Announce,
                MessageType::Toast,
                MessageType::Mission
            ]
        );
        assert_eq!(MessageType::Notify.ordinal(), 0);
        assert_eq!(MessageType::Mission.ordinal(), 3);
        assert_eq!(MessageType::Toast.wire_name(), "toast");
        assert_eq!(MessageType::from_ordinal(4), None);
        assert_eq!(
            MessageType::by_wire_name("announce"),
            Some(MessageType::Announce)
        );
        assert_eq!(MessageType::by_wire_name("missing"), None);
    }
}
