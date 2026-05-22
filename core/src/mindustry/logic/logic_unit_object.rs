//! Mirrors the unit-facing logic runtime state used by upstream `LExecutor`.

use std::collections::BTreeMap;

use super::{
    logic_conv, logic_object_name, logic_team_from_name, logic_unconv, LAccess, LUnitControl,
    LVarValue, LogicRadarSource, RadarUnitView, LOGIC_CTRL_PROCESSOR,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LogicUnitObject {
    pub type_name: String,
    pub logic_controllable: bool,
    pub team: u8,
    pub valid: bool,
    pub x: f32,
    pub y: f32,
    pub range: f32,
    pub health: f32,
    pub shield: f32,
    pub armor: f32,
    pub max_health: f32,
    pub is_player: bool,
    pub can_shoot: bool,
    pub is_flying: bool,
    pub is_boss: bool,
    pub is_grounded: bool,
    pub targetable: bool,
    pub control: Option<LUnitControl>,
    pub move_x: f32,
    pub move_y: f32,
    pub move_rad: f32,
    pub aim_control: Option<LUnitControl>,
    pub target_x: f32,
    pub target_y: f32,
    pub main_target: Option<String>,
    pub shoot: bool,
    pub boost: bool,
    pub flag: f64,
    pub statuses: BTreeMap<String, f32>,
    pub mine_x: Option<f32>,
    pub mine_y: Option<f32>,
    pub mine_cleared: bool,
    pub building_cleared: bool,
    pub controller_reset: bool,
    pub control_timer_refreshed: bool,
    pub prop_values: BTreeMap<LAccess, LVarValue>,
    pub content_props: BTreeMap<String, f64>,
}

impl LogicUnitObject {
    pub fn new(type_name: impl Into<String>, team: u8, x: f32, y: f32) -> Self {
        Self {
            type_name: type_name.into(),
            logic_controllable: true,
            team,
            valid: true,
            x,
            y,
            range: 0.0,
            health: 0.0,
            shield: 0.0,
            armor: 0.0,
            max_health: 0.0,
            is_player: false,
            can_shoot: false,
            is_flying: false,
            is_boss: false,
            is_grounded: false,
            targetable: true,
            control: None,
            move_x: 0.0,
            move_y: 0.0,
            move_rad: 0.0,
            aim_control: None,
            target_x: 0.0,
            target_y: 0.0,
            main_target: None,
            shoot: false,
            boost: false,
            flag: 0.0,
            statuses: BTreeMap::new(),
            mine_x: None,
            mine_y: None,
            mine_cleared: false,
            building_cleared: false,
            controller_reset: false,
            control_timer_refreshed: false,
            prop_values: BTreeMap::new(),
            content_props: BTreeMap::new(),
        }
    }

    pub fn controllable_by(&self, exec_privileged: bool, exec_team: u8) -> bool {
        self.valid && self.logic_controllable && (exec_privileged || self.team == exec_team)
    }

    pub fn radar_source(&self) -> LogicRadarSource {
        LogicRadarSource::new(self.x, self.y, self.team, self.range)
    }

    pub fn radar_view(&self) -> RadarUnitView {
        RadarUnitView {
            x: self.x,
            y: self.y,
            health: self.health,
            shield: self.shield,
            armor: self.armor,
            max_health: self.max_health,
            team: self.team,
            is_player: self.is_player,
            can_shoot: self.can_shoot,
            is_flying: self.is_flying,
            is_boss: self.is_boss,
            is_grounded: self.is_grounded,
            targetable: self.valid && self.targetable,
        }
    }

    pub fn clear_unit_action(&mut self) {
        self.mine_x = None;
        self.mine_y = None;
        self.mine_cleared = true;
        self.building_cleared = true;
    }

    pub(super) fn sense_access(&self, access: LAccess) -> Option<LVarValue> {
        match access {
            LAccess::Health => Some(LVarValue::Number(self.health as f64)),
            LAccess::MaxHealth => Some(LVarValue::Number(self.max_health as f64)),
            LAccess::Shield => Some(LVarValue::Number(self.shield as f64)),
            LAccess::Armor => Some(LVarValue::Number(self.armor as f64)),
            LAccess::X => Some(LVarValue::Number(logic_conv(self.x) as f64)),
            LAccess::Y => Some(LVarValue::Number(logic_conv(self.y) as f64)),
            LAccess::Dead => Some(LVarValue::Number((!self.valid) as u8 as f64)),
            LAccess::Range => Some(LVarValue::Number(logic_conv(self.range) as f64)),
            LAccess::Shooting => Some(LVarValue::Number(self.shoot as u8 as f64)),
            LAccess::Boosting => Some(LVarValue::Number(self.boost as u8 as f64)),
            LAccess::Team => Some(LVarValue::Number(self.team as f64)),
            LAccess::Type => Some(LVarValue::Object(Some(logic_object_name(&self.type_name)))),
            LAccess::Flag => Some(LVarValue::Number(self.flag)),
            LAccess::Controlled => Some(LVarValue::Number(
                self.control_timer_refreshed as u8 as f64 * LOGIC_CTRL_PROCESSOR as f64,
            )),
            LAccess::MineX => Some(LVarValue::Number(
                self.mine_x.map_or(-1.0, |value| logic_conv(value)) as f64,
            )),
            LAccess::MineY => Some(LVarValue::Number(
                self.mine_y.map_or(-1.0, |value| logic_conv(value)) as f64,
            )),
            LAccess::Mining => Some(LVarValue::Number(
                (self.mine_x.is_some() && self.mine_y.is_some()) as u8 as f64,
            )),
            _ => self.prop_values.get(&access).cloned(),
        }
    }

    pub(super) fn sense_content(&self, content_name: &str) -> f64 {
        *self.content_props.get(content_name).unwrap_or(&0.0)
    }

    pub(super) fn set_prop(&mut self, access: LAccess, value: LVarValue) {
        match (&access, &value) {
            (LAccess::Health, LVarValue::Number(value)) => {
                self.health = (*value as f32).clamp(0.0, self.max_health.max(0.0));
                self.valid = self.health > 0.0 || self.max_health <= 0.0;
            }
            (LAccess::Shield, LVarValue::Number(value)) => self.shield = (*value as f32).max(0.0),
            (LAccess::Armor, LVarValue::Number(value)) => self.armor = (*value as f32).max(0.0),
            (LAccess::X, LVarValue::Number(value)) => self.x = logic_unconv(*value as f32),
            (LAccess::Y, LVarValue::Number(value)) => self.y = logic_unconv(*value as f32),
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
            (LAccess::Flag, LVarValue::Number(value)) => self.flag = *value,
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
    use super::LogicUnitObject;
    use crate::mindustry::logic::{LAccess, LVarValue, LOGIC_CTRL_PROCESSOR};

    #[test]
    fn logic_unit_object_sense_control_and_set_prop_follow_java_runtime_shape() {
        let mut unit = LogicUnitObject::new("dagger", 1, 16.0, 24.0);
        unit.range = 80.0;
        unit.max_health = 100.0;
        unit.health = 50.0;
        unit.shield = 12.0;
        unit.armor = 3.0;
        unit.shoot = true;
        unit.boost = true;
        unit.flag = 7.0;
        unit.mine_x = Some(32.0);
        unit.mine_y = Some(40.0);
        unit.control_timer_refreshed = true;

        assert!(unit.controllable_by(false, 1));
        assert!(!unit.controllable_by(false, 2));
        assert!(unit.controllable_by(true, 2));
        assert_eq!(unit.radar_source().team, 1);
        assert!(unit.radar_view().targetable);

        assert_eq!(
            unit.sense_access(LAccess::Health),
            Some(LVarValue::Number(50.0))
        );
        assert_eq!(unit.sense_access(LAccess::X), Some(LVarValue::Number(2.0)));
        assert_eq!(unit.sense_access(LAccess::Y), Some(LVarValue::Number(3.0)));
        assert_eq!(
            unit.sense_access(LAccess::Type),
            Some(LVarValue::Object(Some("@dagger".into())))
        );
        assert_eq!(
            unit.sense_access(LAccess::Controlled),
            Some(LVarValue::Number(LOGIC_CTRL_PROCESSOR as f64))
        );
        assert_eq!(
            unit.sense_access(LAccess::MineX),
            Some(LVarValue::Number(4.0))
        );
        assert_eq!(
            unit.sense_access(LAccess::MineY),
            Some(LVarValue::Number(5.0))
        );
        assert_eq!(
            unit.sense_access(LAccess::Mining),
            Some(LVarValue::Number(1.0))
        );

        unit.clear_unit_action();
        assert_eq!(
            unit.sense_access(LAccess::MineX),
            Some(LVarValue::Number(-1.0))
        );
        assert_eq!(
            unit.sense_access(LAccess::Mining),
            Some(LVarValue::Number(0.0))
        );
        assert!(unit.mine_cleared);
        assert!(unit.building_cleared);

        unit.set_prop(LAccess::X, LVarValue::Number(9.0));
        unit.set_prop(LAccess::Team, LVarValue::Object(Some("@crux".into())));
        unit.set_prop(LAccess::Health, LVarValue::Number(-5.0));
        unit.set_content_prop("copper", 42.0);

        assert_eq!(unit.x, 72.0);
        assert_eq!(unit.team, 2);
        assert_eq!(unit.health, 0.0);
        assert!(!unit.valid);
        assert_eq!(unit.sense_content("@copper"), 42.0);
        assert_eq!(unit.sense_content("@lead"), 0.0);
    }
}
