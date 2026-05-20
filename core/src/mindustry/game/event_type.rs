#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    Shock,
    CannotUpgrade,
    OpenConsole,
    BlastFreeze,
    ImpactPower,
    BlastGenerator,
    ShockwaveTowerUse,
    ForceProjectorBreak,
    ThoriumReactorOverheat,
    NeoplasmReact,
    FireExtinguish,
    AcceleratorUse,
    NewGame,
    TutorialComplete,
    FlameAmmo,
    ResupplyTurret,
    TurretCool,
    EnablePixelation,
    ExclusionDeath,
    SuicideBomb,
    OpenWiki,
    TeamCoreDamage,
    SocketConfigChanged,
    Update,
    BeforeGameUpdate,
    AfterGameUpdate,
    UnitCommandChange,
    UnitCommandPosition,
    UnitCommandAttack,
    UnitCommandBoost,
    ImportMod,
    Draw,
    DrawOver,
    PreDraw,
    PostDraw,
    UiDrawBegin,
    UiDrawEnd,
    UniverseDrawBegin,
    UniverseDraw,
    UniverseDrawEnd,
}

impl Trigger {
    pub const fn java_name(self) -> &'static str {
        match self {
            Trigger::Shock => "shock",
            Trigger::CannotUpgrade => "cannotUpgrade",
            Trigger::OpenConsole => "openConsole",
            Trigger::BlastFreeze => "blastFreeze",
            Trigger::ImpactPower => "impactPower",
            Trigger::BlastGenerator => "blastGenerator",
            Trigger::ShockwaveTowerUse => "shockwaveTowerUse",
            Trigger::ForceProjectorBreak => "forceProjectorBreak",
            Trigger::ThoriumReactorOverheat => "thoriumReactorOverheat",
            Trigger::NeoplasmReact => "neoplasmReact",
            Trigger::FireExtinguish => "fireExtinguish",
            Trigger::AcceleratorUse => "acceleratorUse",
            Trigger::NewGame => "newGame",
            Trigger::TutorialComplete => "tutorialComplete",
            Trigger::FlameAmmo => "flameAmmo",
            Trigger::ResupplyTurret => "resupplyTurret",
            Trigger::TurretCool => "turretCool",
            Trigger::EnablePixelation => "enablePixelation",
            Trigger::ExclusionDeath => "exclusionDeath",
            Trigger::SuicideBomb => "suicideBomb",
            Trigger::OpenWiki => "openWiki",
            Trigger::TeamCoreDamage => "teamCoreDamage",
            Trigger::SocketConfigChanged => "socketConfigChanged",
            Trigger::Update => "update",
            Trigger::BeforeGameUpdate => "beforeGameUpdate",
            Trigger::AfterGameUpdate => "afterGameUpdate",
            Trigger::UnitCommandChange => "unitCommandChange",
            Trigger::UnitCommandPosition => "unitCommandPosition",
            Trigger::UnitCommandAttack => "unitCommandAttack",
            Trigger::UnitCommandBoost => "unitCommandBoost",
            Trigger::ImportMod => "importMod",
            Trigger::Draw => "draw",
            Trigger::DrawOver => "drawOver",
            Trigger::PreDraw => "preDraw",
            Trigger::PostDraw => "postDraw",
            Trigger::UiDrawBegin => "uiDrawBegin",
            Trigger::UiDrawEnd => "uiDrawEnd",
            Trigger::UniverseDrawBegin => "universeDrawBegin",
            Trigger::UniverseDraw => "universeDraw",
            Trigger::UniverseDrawEnd => "universeDrawEnd",
        }
    }

    pub fn from_java_name(name: &str) -> Option<Self> {
        Some(match name {
            "shock" => Trigger::Shock,
            "cannotUpgrade" => Trigger::CannotUpgrade,
            "openConsole" => Trigger::OpenConsole,
            "blastFreeze" => Trigger::BlastFreeze,
            "impactPower" => Trigger::ImpactPower,
            "blastGenerator" => Trigger::BlastGenerator,
            "shockwaveTowerUse" => Trigger::ShockwaveTowerUse,
            "forceProjectorBreak" => Trigger::ForceProjectorBreak,
            "thoriumReactorOverheat" => Trigger::ThoriumReactorOverheat,
            "neoplasmReact" => Trigger::NeoplasmReact,
            "fireExtinguish" => Trigger::FireExtinguish,
            "acceleratorUse" => Trigger::AcceleratorUse,
            "newGame" => Trigger::NewGame,
            "tutorialComplete" => Trigger::TutorialComplete,
            "flameAmmo" => Trigger::FlameAmmo,
            "resupplyTurret" => Trigger::ResupplyTurret,
            "turretCool" => Trigger::TurretCool,
            "enablePixelation" => Trigger::EnablePixelation,
            "exclusionDeath" => Trigger::ExclusionDeath,
            "suicideBomb" => Trigger::SuicideBomb,
            "openWiki" => Trigger::OpenWiki,
            "teamCoreDamage" => Trigger::TeamCoreDamage,
            "socketConfigChanged" => Trigger::SocketConfigChanged,
            "update" => Trigger::Update,
            "beforeGameUpdate" => Trigger::BeforeGameUpdate,
            "afterGameUpdate" => Trigger::AfterGameUpdate,
            "unitCommandChange" => Trigger::UnitCommandChange,
            "unitCommandPosition" => Trigger::UnitCommandPosition,
            "unitCommandAttack" => Trigger::UnitCommandAttack,
            "unitCommandBoost" => Trigger::UnitCommandBoost,
            "importMod" => Trigger::ImportMod,
            "draw" => Trigger::Draw,
            "drawOver" => Trigger::DrawOver,
            "preDraw" => Trigger::PreDraw,
            "postDraw" => Trigger::PostDraw,
            "uiDrawBegin" => Trigger::UiDrawBegin,
            "uiDrawEnd" => Trigger::UiDrawEnd,
            "universeDrawBegin" => Trigger::UniverseDrawBegin,
            "universeDraw" => Trigger::UniverseDraw,
            "universeDrawEnd" => Trigger::UniverseDrawEnd,
            _ => return None,
        })
    }
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

    #[test]
    fn trigger_names_cover_upstream_java_trigger_enum_order() {
        let names: Vec<&str> = [
            Trigger::Shock,
            Trigger::CannotUpgrade,
            Trigger::OpenConsole,
            Trigger::BlastFreeze,
            Trigger::ImpactPower,
            Trigger::BlastGenerator,
            Trigger::ShockwaveTowerUse,
            Trigger::ForceProjectorBreak,
            Trigger::ThoriumReactorOverheat,
            Trigger::NeoplasmReact,
            Trigger::FireExtinguish,
            Trigger::AcceleratorUse,
            Trigger::NewGame,
            Trigger::TutorialComplete,
            Trigger::FlameAmmo,
            Trigger::ResupplyTurret,
            Trigger::TurretCool,
            Trigger::EnablePixelation,
            Trigger::ExclusionDeath,
            Trigger::SuicideBomb,
            Trigger::OpenWiki,
            Trigger::TeamCoreDamage,
            Trigger::SocketConfigChanged,
            Trigger::Update,
            Trigger::BeforeGameUpdate,
            Trigger::AfterGameUpdate,
            Trigger::UnitCommandChange,
            Trigger::UnitCommandPosition,
            Trigger::UnitCommandAttack,
            Trigger::UnitCommandBoost,
            Trigger::ImportMod,
            Trigger::Draw,
            Trigger::DrawOver,
            Trigger::PreDraw,
            Trigger::PostDraw,
            Trigger::UiDrawBegin,
            Trigger::UiDrawEnd,
            Trigger::UniverseDrawBegin,
            Trigger::UniverseDraw,
            Trigger::UniverseDrawEnd,
        ]
        .into_iter()
        .map(Trigger::java_name)
        .collect();

        assert_eq!(
            names,
            vec![
                "shock",
                "cannotUpgrade",
                "openConsole",
                "blastFreeze",
                "impactPower",
                "blastGenerator",
                "shockwaveTowerUse",
                "forceProjectorBreak",
                "thoriumReactorOverheat",
                "neoplasmReact",
                "fireExtinguish",
                "acceleratorUse",
                "newGame",
                "tutorialComplete",
                "flameAmmo",
                "resupplyTurret",
                "turretCool",
                "enablePixelation",
                "exclusionDeath",
                "suicideBomb",
                "openWiki",
                "teamCoreDamage",
                "socketConfigChanged",
                "update",
                "beforeGameUpdate",
                "afterGameUpdate",
                "unitCommandChange",
                "unitCommandPosition",
                "unitCommandAttack",
                "unitCommandBoost",
                "importMod",
                "draw",
                "drawOver",
                "preDraw",
                "postDraw",
                "uiDrawBegin",
                "uiDrawEnd",
                "universeDrawBegin",
                "universeDraw",
                "universeDrawEnd",
            ]
        );
    }

    #[test]
    fn trigger_java_names_roundtrip_known_values() {
        for trigger in [
            Trigger::Shock,
            Trigger::NewGame,
            Trigger::Update,
            Trigger::UniverseDrawEnd,
        ] {
            assert_eq!(Trigger::from_java_name(trigger.java_name()), Some(trigger));
        }
        assert_eq!(Trigger::from_java_name("missing"), None);
    }
}
