//! Mirrors lightweight locate and radar result views used by logic instructions.

#[derive(Debug, Clone, PartialEq)]
pub struct LogicLocateResult {
    pub x: f32,
    pub y: f32,
    pub building: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarUnitView {
    pub x: f32,
    pub y: f32,
    pub health: f32,
    pub shield: f32,
    pub armor: f32,
    pub max_health: f32,
    pub team: u8,
    pub is_player: bool,
    pub can_shoot: bool,
    pub is_flying: bool,
    pub is_boss: bool,
    pub is_grounded: bool,
    pub targetable: bool,
}

impl RadarUnitView {
    pub const fn new(x: f32, y: f32, team: u8) -> Self {
        Self {
            x,
            y,
            health: 0.0,
            shield: 0.0,
            armor: 0.0,
            max_health: 0.0,
            team,
            is_player: false,
            can_shoot: false,
            is_flying: false,
            is_boss: false,
            is_grounded: false,
            targetable: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LogicLocateResult, RadarUnitView};

    #[test]
    fn logic_locate_result_keeps_java_locate_payload_shape() {
        let result = LogicLocateResult {
            x: 16.0,
            y: 24.0,
            building: Some("@core".into()),
        };
        assert_eq!(result.x, 16.0);
        assert_eq!(result.y, 24.0);
        assert_eq!(result.building.as_deref(), Some("@core"));
    }

    #[test]
    fn radar_unit_view_defaults_match_java_radar_target_view() {
        let mut unit = RadarUnitView::new(3.0, 4.0, 2);
        assert_eq!(unit.x, 3.0);
        assert_eq!(unit.y, 4.0);
        assert_eq!(unit.team, 2);
        assert_eq!(unit.health, 0.0);
        assert_eq!(unit.shield, 0.0);
        assert_eq!(unit.armor, 0.0);
        assert_eq!(unit.max_health, 0.0);
        assert!(!unit.is_player);
        assert!(!unit.can_shoot);
        assert!(!unit.is_flying);
        assert!(!unit.is_boss);
        assert!(!unit.is_grounded);
        assert!(unit.targetable);

        unit.health = 100.0;
        unit.can_shoot = true;
        assert_eq!(unit.health, 100.0);
        assert!(unit.can_shoot);
    }
}
