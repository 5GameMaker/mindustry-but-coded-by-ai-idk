//! Mirrors the lightweight radar source checks used by upstream logic sensors.

#[derive(Debug, Clone, PartialEq)]
pub struct LogicRadarSource {
    pub x: f32,
    pub y: f32,
    pub team: u8,
    pub range: f32,
    pub block_privileged: bool,
}

impl LogicRadarSource {
    pub const fn new(x: f32, y: f32, team: u8, range: f32) -> Self {
        Self {
            x,
            y,
            team,
            range,
            block_privileged: false,
        }
    }

    pub fn usable_by(&self, exec_privileged: bool, exec_team: u8) -> bool {
        (exec_privileged || self.team == exec_team) && (!self.block_privileged || exec_privileged)
    }
}

#[cfg(test)]
mod tests {
    use super::LogicRadarSource;

    #[test]
    fn logic_radar_source_matches_java_team_and_privilege_checks() {
        let mut source = LogicRadarSource::new(12.0, 24.0, 2, 96.0);
        assert_eq!(
            (source.x, source.y, source.team, source.range),
            (12.0, 24.0, 2, 96.0)
        );
        assert!(!source.block_privileged);

        assert!(source.usable_by(false, 2));
        assert!(!source.usable_by(false, 3));
        assert!(source.usable_by(true, 3));

        source.block_privileged = true;
        assert!(!source.usable_by(false, 2));
        assert!(source.usable_by(true, 3));
    }
}
