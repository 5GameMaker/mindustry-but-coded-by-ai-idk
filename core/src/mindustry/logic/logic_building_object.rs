//! Mirrors the building-facing logic runtime state used by upstream `LExecutor`.

use std::collections::{BTreeMap, BTreeSet};

use crate::mindustry::world::meta::BlockFlag;

use super::{
    logic_conv, logic_object_name, logic_team_from_name, LAccess, LVarValue, LOGIC_TILE_SIZE,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LogicBuildingObject {
    pub block_name: String,
    pub team: u8,
    pub x: f32,
    pub y: f32,
    pub hit_size: f32,
    pub valid: bool,
    pub flags: BTreeSet<BlockFlag>,
    pub damaged: bool,
    pub block_privileged: bool,
    pub display_commands: Vec<u64>,
    pub message: String,
    pub prop_values: BTreeMap<LAccess, LVarValue>,
    pub content_props: BTreeMap<String, f64>,
}

impl LogicBuildingObject {
    pub fn new(block_name: impl Into<String>, team: u8, x: f32, y: f32) -> Self {
        Self {
            block_name: block_name.into(),
            team,
            x,
            y,
            hit_size: LOGIC_TILE_SIZE,
            valid: true,
            flags: BTreeSet::new(),
            damaged: false,
            block_privileged: false,
            display_commands: Vec::new(),
            message: String::new(),
            prop_values: BTreeMap::new(),
            content_props: BTreeMap::new(),
        }
    }

    pub fn has_flag(&self, flag: BlockFlag) -> bool {
        self.flags.contains(&flag)
    }

    pub(super) fn sense_access(&self, access: LAccess) -> Option<LVarValue> {
        match access {
            LAccess::Health => Some(LVarValue::Number((!self.damaged) as u8 as f64)),
            LAccess::X => Some(LVarValue::Number(logic_conv(self.x) as f64)),
            LAccess::Y => Some(LVarValue::Number(logic_conv(self.y) as f64)),
            LAccess::Team => Some(LVarValue::Number(self.team as f64)),
            LAccess::Type => Some(LVarValue::Object(Some(logic_object_name(&self.block_name)))),
            LAccess::Dead => Some(LVarValue::Number((!self.valid) as u8 as f64)),
            _ => self.prop_values.get(&access).cloned(),
        }
    }

    pub(super) fn sense_content(&self, content_name: &str) -> f64 {
        *self.content_props.get(content_name).unwrap_or(&0.0)
    }

    pub(super) fn set_prop(&mut self, access: LAccess, value: LVarValue) {
        match (&access, &value) {
            (LAccess::Health, LVarValue::Number(value)) => {
                self.damaged = *value <= 0.0;
                self.valid = *value > 0.0;
            }
            (LAccess::Team, LVarValue::Number(value)) => {
                if (0.0..=255.0).contains(value) {
                    self.team = *value as u8;
                }
            }
            (LAccess::Team, LVarValue::Object(Some(value))) => {
                if let Some(team) = logic_team_from_name(value) {
                    self.team = team;
                }
            }
            _ => {}
        }
        self.prop_values.insert(access, value);
    }

    pub(super) fn set_content_prop(&mut self, content_name: impl Into<String>, value: f64) {
        self.content_props
            .insert(logic_object_name(&content_name.into()), value);
    }
}

#[cfg(test)]
mod tests {
    use super::LogicBuildingObject;
    use crate::mindustry::{
        logic::{LAccess, LVarValue, LOGIC_TILE_SIZE},
        world::meta::BlockFlag,
    };

    #[test]
    fn logic_building_object_sense_flags_and_set_prop_follow_java_runtime_shape() {
        let mut building = LogicBuildingObject::new("core-shard", 1, 16.0, 24.0);
        assert_eq!(building.block_name, "core-shard");
        assert_eq!(building.team, 1);
        assert_eq!(building.hit_size, LOGIC_TILE_SIZE);
        assert!(building.valid);
        assert!(!building.damaged);
        assert!(!building.block_privileged);
        assert!(building.display_commands.is_empty());
        assert!(building.message.is_empty());
        assert!(!building.has_flag(BlockFlag::Core));

        building.flags.insert(BlockFlag::Core);
        assert!(building.has_flag(BlockFlag::Core));
        assert_eq!(
            building.sense_access(LAccess::Health),
            Some(LVarValue::Number(1.0))
        );
        assert_eq!(
            building.sense_access(LAccess::X),
            Some(LVarValue::Number(2.0))
        );
        assert_eq!(
            building.sense_access(LAccess::Y),
            Some(LVarValue::Number(3.0))
        );
        assert_eq!(
            building.sense_access(LAccess::Type),
            Some(LVarValue::Object(Some("@core-shard".into())))
        );
        assert_eq!(
            building.sense_access(LAccess::Dead),
            Some(LVarValue::Number(0.0))
        );

        building.set_prop(LAccess::Team, LVarValue::Object(Some("@crux".into())));
        assert_eq!(building.team, 2);
        building.set_prop(LAccess::Health, LVarValue::Number(0.0));
        assert!(building.damaged);
        assert!(!building.valid);
        assert_eq!(
            building.sense_access(LAccess::Health),
            Some(LVarValue::Number(0.0))
        );
        assert_eq!(
            building.sense_access(LAccess::Dead),
            Some(LVarValue::Number(1.0))
        );

        building.set_content_prop("copper", 12.0);
        assert_eq!(building.sense_content("@copper"), 12.0);
        assert_eq!(building.sense_content("@lead"), 0.0);
    }
}
