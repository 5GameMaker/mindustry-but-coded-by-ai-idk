//! Mirrors marker side-effect state used by upstream marker logic instructions.

use super::{logic_unconv, LMarkerControl, LVarValue};

#[derive(Debug, Clone, PartialEq)]
pub struct LogicMarkerControlEvent {
    pub id: i32,
    pub control: LMarkerControl,
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicMarkerEvent {
    Created {
        id: i32,
        type_name: String,
        x: f32,
        y: f32,
        replaced: bool,
    },
    Removed {
        id: i32,
    },
    Controlled(LogicMarkerControlEvent),
    Text {
        id: i32,
        text: String,
        fetch: bool,
    },
    Texture {
        id: i32,
        texture: LVarValue,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicMarkerObject {
    pub type_name: String,
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub text_fetch: bool,
    pub texture: LVarValue,
    pub controls: Vec<LogicMarkerControlEvent>,
}

impl LogicMarkerObject {
    pub fn new(type_name: impl Into<String>, x: f32, y: f32) -> Self {
        Self {
            type_name: type_name.into(),
            x,
            y,
            text: String::new(),
            text_fetch: false,
            texture: LVarValue::Object(None),
            controls: Vec::new(),
        }
    }

    pub fn control(&mut self, event: LogicMarkerControlEvent) {
        if event.control == LMarkerControl::Pos {
            if !event.p1.is_nan() {
                self.x = logic_unconv(event.p1 as f32);
            }
            if !event.p2.is_nan() {
                self.y = logic_unconv(event.p2 as f32);
            }
        }
        self.controls.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::{LogicMarkerControlEvent, LogicMarkerEvent, LogicMarkerObject};
    use crate::mindustry::logic::{LMarkerControl, LVarValue};

    #[test]
    fn logic_marker_object_defaults_and_pos_control_match_java_marker_controls() {
        let mut marker = LogicMarkerObject::new("shape", 8.0, 16.0);
        assert_eq!(marker.type_name, "shape");
        assert_eq!((marker.x, marker.y), (8.0, 16.0));
        assert!(marker.text.is_empty());
        assert!(!marker.text_fetch);
        assert_eq!(marker.texture, LVarValue::Object(None));
        assert!(marker.controls.is_empty());

        marker.control(LogicMarkerControlEvent {
            id: 5,
            control: LMarkerControl::Pos,
            p1: 4.0,
            p2: f64::NAN,
            p3: 0.0,
        });
        assert_eq!(marker.x, 32.0);
        assert_eq!(marker.y, 16.0);
        assert_eq!(marker.controls.len(), 1);

        marker.control(LogicMarkerControlEvent {
            id: 5,
            control: LMarkerControl::Radius,
            p1: 9.0,
            p2: 10.0,
            p3: 11.0,
        });
        assert_eq!((marker.x, marker.y), (32.0, 16.0));
        assert_eq!(marker.controls.len(), 2);
    }

    #[test]
    fn logic_marker_event_variants_preserve_payload_shapes() {
        let controlled = LogicMarkerControlEvent {
            id: 1,
            control: LMarkerControl::Color,
            p1: 0.1,
            p2: 0.2,
            p3: 0.3,
        };
        assert_eq!(
            LogicMarkerEvent::Controlled(controlled.clone()),
            LogicMarkerEvent::Controlled(controlled)
        );
        assert_eq!(
            LogicMarkerEvent::Texture {
                id: 2,
                texture: LVarValue::Object(Some("@icon".into())),
            },
            LogicMarkerEvent::Texture {
                id: 2,
                texture: LVarValue::Object(Some("@icon".into())),
            }
        );
    }
}
