//! Mirrors the dynamic runtime object dispatch used by upstream `LExecutor`.

use super::{
    logic_conv, logic_utf16_len, read_logic_sequence, read_logic_text, LAccess, LVar, LVarValue,
    LogicBuildingObject, LogicBulletEvent, LogicControllableObject, LogicMemoryObject,
    LogicRadarSource, LogicSenseObject, LogicUnitObject,
};

#[derive(Debug, Clone, PartialEq)]
pub enum LogicRuntimeObject {
    Text(String),
    Sequence(Vec<LVarValue>),
    Memory(LogicMemoryObject),
    Senseable(LogicSenseObject),
    Controllable(LogicControllableObject),
    RadarSource(LogicRadarSource),
    Unit(LogicUnitObject),
    Building(LogicBuildingObject),
    Bullet(LogicBulletEvent),
    QueryResult(Vec<String>),
}

impl LogicRuntimeObject {
    pub(super) fn read_runtime(
        &self,
        exec_privileged: bool,
        exec_team: u8,
        position: &LVar,
        output: &mut LVar,
    ) -> bool {
        match self {
            LogicRuntimeObject::Text(value) => {
                read_logic_text(value, position, output);
                true
            }
            LogicRuntimeObject::Sequence(values) => {
                read_logic_sequence(values, position, output);
                true
            }
            LogicRuntimeObject::Memory(memory) => {
                if memory.readable_by(exec_privileged, exec_team) {
                    memory.read(position, output);
                } else {
                    output.set_obj(None);
                }
                true
            }
            LogicRuntimeObject::Senseable(_) => false,
            LogicRuntimeObject::Controllable(_)
            | LogicRuntimeObject::RadarSource(_)
            | LogicRuntimeObject::Unit(_)
            | LogicRuntimeObject::Building(_)
            | LogicRuntimeObject::Bullet(_) => false,
            LogicRuntimeObject::QueryResult(values) => {
                read_logic_sequence(
                    &values
                        .iter()
                        .cloned()
                        .map(|value| LVarValue::Object(Some(value)))
                        .collect::<Vec<_>>(),
                    position,
                    output,
                );
                true
            }
        }
    }

    pub(super) fn write_runtime(
        &mut self,
        exec_privileged: bool,
        exec_team: u8,
        position: &LVar,
        value: &LVar,
    ) -> bool {
        match self {
            LogicRuntimeObject::Memory(memory) => {
                if memory.readable_by(exec_privileged, exec_team) {
                    memory.write(position, value);
                }
                true
            }
            _ => false,
        }
    }

    pub(super) fn sense_access(&self, access: LAccess) -> Option<LVarValue> {
        match self {
            LogicRuntimeObject::Text(value) => match access {
                LAccess::Size | LAccess::BufferSize => {
                    Some(LVarValue::Number(logic_utf16_len(value) as f64))
                }
                _ => None,
            },
            LogicRuntimeObject::Sequence(values) => match access {
                LAccess::Size | LAccess::BufferSize => Some(LVarValue::Number(values.len() as f64)),
                _ => None,
            },
            LogicRuntimeObject::Memory(memory) => match access {
                LAccess::MemoryCapacity => Some(LVarValue::Number(memory.memory.len() as f64)),
                _ => None,
            },
            LogicRuntimeObject::Senseable(senseable) => {
                if let Some(value) = senseable.object_senses.get(&access) {
                    Some(LVarValue::Object(value.clone()))
                } else {
                    Some(LVarValue::Number(
                        *senseable.numeric_senses.get(&access).unwrap_or(&0.0),
                    ))
                }
            }
            LogicRuntimeObject::Controllable(controllable) => match access {
                LAccess::Enabled => Some(LVarValue::Number(controllable.enabled as u8 as f64)),
                _ => None,
            },
            LogicRuntimeObject::RadarSource(_) => None,
            LogicRuntimeObject::Unit(unit) => unit.sense_access(access),
            LogicRuntimeObject::Building(building) => building.sense_access(access),
            LogicRuntimeObject::Bullet(bullet) => match access {
                LAccess::X => Some(LVarValue::Number(logic_conv(bullet.x) as f64)),
                LAccess::Y => Some(LVarValue::Number(logic_conv(bullet.y) as f64)),
                LAccess::Rotation => Some(LVarValue::Number(bullet.rotation as f64)),
                LAccess::Team => Some(LVarValue::Number(bullet.team as f64)),
                LAccess::Health => Some(LVarValue::Number(bullet.damage as f64)),
                LAccess::BulletLifetime => Some(LVarValue::Number(bullet.life_scl as f64)),
                _ => None,
            },
            LogicRuntimeObject::QueryResult(values) => match access {
                LAccess::Size => Some(LVarValue::Number(values.len() as f64)),
                _ => None,
            },
        }
    }

    pub(super) fn sense_content(&self, content_name: &str) -> Option<f64> {
        match self {
            LogicRuntimeObject::Senseable(senseable) => {
                Some(*senseable.content_senses.get(content_name).unwrap_or(&0.0))
            }
            LogicRuntimeObject::Unit(unit) => Some(unit.sense_content(content_name)),
            LogicRuntimeObject::Building(building) => Some(building.sense_content(content_name)),
            LogicRuntimeObject::Bullet(_) | LogicRuntimeObject::QueryResult(_) => Some(0.0),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LogicRuntimeObject;
    use crate::mindustry::logic::{
        LAccess, LVar, LVarValue, LogicBulletEvent, LogicControllableObject, LogicMemoryObject,
        LogicSenseObject,
    };

    #[test]
    fn logic_runtime_object_read_write_and_sense_dispatch_follow_java_l_executor_shape() {
        let mut position = LVar::new("pos");
        let mut output = LVar::new("out");

        position.set_num(1.0);
        assert!(LogicRuntimeObject::Text("ab💥".into()).read_runtime(
            false,
            1,
            &position,
            &mut output
        ));
        assert_eq!(output.value(), LVarValue::Number('b' as u32 as f64));

        let sequence = LogicRuntimeObject::Sequence(vec![
            LVarValue::Number(1.0),
            LVarValue::Object(Some("@copper".into())),
        ]);
        assert!(sequence.read_runtime(false, 1, &position, &mut output));
        assert_eq!(output.value(), LVarValue::Object(Some("@copper".into())));
        assert_eq!(
            sequence.sense_access(LAccess::Size),
            Some(LVarValue::Number(2.0))
        );

        let mut memory = LogicRuntimeObject::Memory(LogicMemoryObject::new(2, 1));
        let mut value = LVar::new("value");
        value.set_num(42.0);
        assert!(memory.write_runtime(false, 1, &position, &value));
        assert!(memory.read_runtime(false, 1, &position, &mut output));
        assert_eq!(output.value(), LVarValue::Number(42.0));
        assert_eq!(
            memory.sense_access(LAccess::MemoryCapacity),
            Some(LVarValue::Number(2.0))
        );

        let blocked_memory = LogicRuntimeObject::Memory(LogicMemoryObject::new(1, 2));
        assert!(blocked_memory.read_runtime(false, 1, &position, &mut output));
        assert_eq!(output.value(), LVarValue::Object(None));

        let mut senseable = LogicSenseObject::default();
        senseable.numeric_senses.insert(LAccess::Health, 99.0);
        senseable.content_senses.insert("@copper".into(), 5.0);
        let runtime = LogicRuntimeObject::Senseable(senseable);
        assert_eq!(
            runtime.sense_access(LAccess::Health),
            Some(LVarValue::Number(99.0))
        );
        assert_eq!(runtime.sense_content("@copper"), Some(5.0));

        let mut controllable = LogicControllableObject::new(1);
        controllable.enabled = false;
        assert_eq!(
            LogicRuntimeObject::Controllable(controllable).sense_access(LAccess::Enabled),
            Some(LVarValue::Number(0.0))
        );

        let bullet = LogicRuntimeObject::Bullet(LogicBulletEvent {
            bullet_name: "@basic".into(),
            from_name: "@duo".into(),
            weapon: LVarValue::Object(None),
            team: 2,
            x: 16.0,
            y: 24.0,
            rotation: 45.0,
            owner: None,
            damage: 8.0,
            velocity_scl: 1.0,
            life_scl: 2.0,
            aim_x: 0.0,
            aim_y: 0.0,
        });
        assert_eq!(
            bullet.sense_access(LAccess::X),
            Some(LVarValue::Number(2.0))
        );
        assert_eq!(
            bullet.sense_access(LAccess::Team),
            Some(LVarValue::Number(2.0))
        );
        assert_eq!(
            bullet.sense_access(LAccess::BulletLifetime),
            Some(LVarValue::Number(2.0))
        );

        let query = LogicRuntimeObject::QueryResult(vec!["@a".into(), "@b".into()]);
        assert_eq!(query.sense_content("@copper"), Some(0.0));
        assert_eq!(
            query.sense_access(LAccess::Size),
            Some(LVarValue::Number(2.0))
        );
    }
}
