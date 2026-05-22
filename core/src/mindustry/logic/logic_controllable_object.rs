//! Mirrors the generic controllable runtime state used by upstream `LExecutor`.

use super::LAccess;

#[derive(Debug, Clone, PartialEq)]
pub enum LogicControlCall {
    Numeric {
        access: LAccess,
        p1: f64,
        p2: f64,
        p3: f64,
        p4: f64,
    },
    Object {
        access: LAccess,
        p1: Option<String>,
        p2: f64,
        p3: f64,
        p4: f64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicControllableObject {
    pub team: u8,
    pub valid_link: bool,
    pub enabled: bool,
    pub no_sleep_calls: usize,
    pub disabled_by_processor: bool,
    pub calls: Vec<LogicControlCall>,
}

impl LogicControllableObject {
    pub fn new(team: u8) -> Self {
        Self {
            team,
            valid_link: true,
            enabled: true,
            no_sleep_calls: 0,
            disabled_by_processor: false,
            calls: Vec::new(),
        }
    }

    pub fn controllable_by(&self, exec_privileged: bool) -> bool {
        exec_privileged || self.valid_link
    }
}

#[cfg(test)]
mod tests {
    use super::{LogicControlCall, LogicControllableObject};
    use crate::mindustry::logic::LAccess;

    #[test]
    fn logic_controllable_object_matches_java_link_and_call_defaults() {
        let mut object = LogicControllableObject::new(3);
        assert_eq!(object.team, 3);
        assert!(object.valid_link);
        assert!(object.enabled);
        assert_eq!(object.no_sleep_calls, 0);
        assert!(!object.disabled_by_processor);
        assert!(object.calls.is_empty());
        assert!(object.controllable_by(false));
        assert!(object.controllable_by(true));

        object.valid_link = false;
        assert!(!object.controllable_by(false));
        assert!(object.controllable_by(true));

        object.calls.push(LogicControlCall::Numeric {
            access: LAccess::Enabled,
            p1: 1.0,
            p2: 0.0,
            p3: 0.0,
            p4: 0.0,
        });
        object.calls.push(LogicControlCall::Object {
            access: LAccess::Config,
            p1: Some("@copper".into()),
            p2: 2.0,
            p3: 3.0,
            p4: 4.0,
        });

        assert_eq!(object.calls.len(), 2);
    }
}
