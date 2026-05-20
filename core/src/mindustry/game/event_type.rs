#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    NewGame,
    SaveLoad,
    ClientCreate,
    WorldLoad,
    WorldDrawBegin,
    WorldDrawEnd,
    PostDraw,
    UiDrawBegin,
    UiDrawEnd,
    Update,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentPatchLoadEvent {
    pub patches: Vec<String>,
}

impl ContentPatchLoadEvent {
    pub fn new(patches: Vec<String>) -> Self {
        Self { patches }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveLoadEvent {
    pub is_map: bool,
}

impl SaveLoadEvent {
    pub const fn new(is_map: bool) -> Self {
        Self { is_map }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientServerConnectEvent {
    pub ip: String,
    pub port: i32,
}

impl ClientServerConnectEvent {
    pub fn new(ip: impl Into<String>, port: i32) -> Self {
        Self {
            ip: ip.into(),
            port,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientChatEvent {
    pub message: String,
}

impl ClientChatEvent {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_load_event_keeps_java_is_map_field() {
        assert!(SaveLoadEvent::new(true).is_map);
        assert!(!SaveLoadEvent::new(false).is_map);
    }

    #[test]
    fn client_server_connect_event_keeps_java_ip_and_port_fields() {
        let event = ClientServerConnectEvent::new("127.0.0.1", 6567);
        assert_eq!(event.ip, "127.0.0.1");
        assert_eq!(event.port, 6567);
    }

    #[test]
    fn content_patch_load_event_keeps_mutable_patch_sequence() {
        let mut event = ContentPatchLoadEvent::new(vec!["base".into()]);
        event.patches.push("modded".into());
        assert_eq!(event.patches, vec!["base", "modded"]);
    }

    #[test]
    fn client_chat_event_is_clientside_message_payload() {
        let event = ClientChatEvent::new("hello");
        assert_eq!(event.message, "hello");
    }
}
