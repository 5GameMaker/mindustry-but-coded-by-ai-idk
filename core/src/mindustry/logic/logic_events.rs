//! Mirrors the data-only side effects recorded by upstream logic instructions.

use super::{LVarValue, MessageType};

#[derive(Debug, Clone, PartialEq)]
pub struct LogicSpawnEvent {
    pub unit_name: String,
    pub type_name: String,
    pub team: u8,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicEffectEvent {
    pub type_name: String,
    pub effect_name: String,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub color: f64,
    pub data: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicExplosionEvent {
    pub team: Option<u8>,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub damage: f32,
    pub air: bool,
    pub ground: bool,
    pub pierce: bool,
    pub effect: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicMessageEvent {
    pub type_: MessageType,
    pub text: String,
    pub duration: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicCutsceneState {
    pub active: bool,
    pub pan_x: f32,
    pub pan_y: f32,
    pub speed: f32,
    pub zoom: f32,
}

impl Default for LogicCutsceneState {
    fn default() -> Self {
        Self {
            active: false,
            pan_x: 0.0,
            pan_y: 0.0,
            speed: 0.0,
            zoom: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LogicMessageState {
    pub announcement_active: bool,
    pub toast_active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicWeatherState {
    pub active: bool,
    pub life: f32,
}

impl Default for LogicWeatherState {
    fn default() -> Self {
        Self {
            active: false,
            life: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicWeatherEvent {
    pub weather_name: String,
    pub active: bool,
    pub life: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicBulletEvent {
    pub bullet_name: String,
    pub from_name: String,
    pub weapon: LVarValue,
    pub team: u8,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub owner: Option<String>,
    pub damage: f32,
    pub velocity_scl: f32,
    pub life_scl: f32,
    pub aim_x: f32,
    pub aim_y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicClientDataEvent {
    pub channel: String,
    pub value: LVarValue,
    pub reliable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicSyncEvent {
    pub variable_id: i32,
    pub value: LVarValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicSoundEvent {
    pub positional: bool,
    pub sound_id: i32,
    pub sound_name: Option<String>,
    pub volume: f32,
    pub pitch: f32,
    pub pan: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub limit: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        LogicBulletEvent, LogicClientDataEvent, LogicCutsceneState, LogicEffectEvent,
        LogicExplosionEvent, LogicMessageEvent, LogicMessageState, LogicSoundEvent,
        LogicSpawnEvent, LogicSyncEvent, LogicWeatherEvent, LogicWeatherState,
    };
    use crate::mindustry::logic::{LVarValue, MessageType};

    #[test]
    fn logic_event_records_keep_java_runtime_side_effect_payloads() {
        let spawn = LogicSpawnEvent {
            unit_name: "@dagger#1".into(),
            type_name: "@dagger".into(),
            team: 1,
            x: 8.0,
            y: 16.0,
            rotation: 90.0,
        };
        assert_eq!(spawn.team, 1);
        assert_eq!(spawn.rotation, 90.0);

        let effect = LogicEffectEvent {
            type_name: "@shootSmall".into(),
            effect_name: "@shootSmall".into(),
            x: 1.0,
            y: 2.0,
            rotation: 3.0,
            color: 4.0,
            data: Some("@router".into()),
        };
        assert_eq!(effect.data.as_deref(), Some("@router"));

        let explosion = LogicExplosionEvent {
            team: Some(2),
            x: 1.0,
            y: 2.0,
            radius: 3.0,
            damage: 4.0,
            air: true,
            ground: false,
            pierce: true,
            effect: false,
        };
        assert_eq!(explosion.team, Some(2));
        assert!(explosion.air);
        assert!(explosion.pierce);

        let message = LogicMessageEvent {
            type_: MessageType::Toast,
            text: "hello".into(),
            duration: 5.0,
        };
        assert_eq!(message.type_, MessageType::Toast);

        let weather = LogicWeatherEvent {
            weather_name: "@rain".into(),
            active: true,
            life: 120.0,
        };
        assert!(weather.active);

        let bullet = LogicBulletEvent {
            bullet_name: "@basic".into(),
            from_name: "@duo".into(),
            weapon: LVarValue::Object(Some("@weapon".into())),
            team: 1,
            x: 2.0,
            y: 3.0,
            rotation: 4.0,
            owner: Some("@dagger#1".into()),
            damage: 5.0,
            velocity_scl: 6.0,
            life_scl: 7.0,
            aim_x: 8.0,
            aim_y: 9.0,
        };
        assert_eq!(bullet.owner.as_deref(), Some("@dagger#1"));
        assert_eq!(bullet.weapon, LVarValue::Object(Some("@weapon".into())));

        let client_data = LogicClientDataEvent {
            channel: "chan".into(),
            value: LVarValue::Number(42.0),
            reliable: true,
        };
        assert!(client_data.reliable);

        let sync = LogicSyncEvent {
            variable_id: 7,
            value: LVarValue::Object(Some("@copper".into())),
        };
        assert_eq!(sync.variable_id, 7);

        let sound = LogicSoundEvent {
            positional: true,
            sound_id: 3,
            sound_name: Some("@boom".into()),
            volume: 0.5,
            pitch: 1.5,
            pan: -0.25,
            x: Some(10.0),
            y: Some(20.0),
            limit: true,
        };
        assert!(sound.positional);
        assert_eq!(sound.x, Some(10.0));
    }

    #[test]
    fn logic_event_states_keep_java_defaults() {
        let cutscene = LogicCutsceneState::default();
        assert!(!cutscene.active);
        assert_eq!(cutscene.zoom, 1.0);

        let message = LogicMessageState::default();
        assert!(!message.announcement_active);
        assert!(!message.toast_active);

        let weather = LogicWeatherState::default();
        assert!(!weather.active);
        assert_eq!(weather.life, 0.0);
    }
}
