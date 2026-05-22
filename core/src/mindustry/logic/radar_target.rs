//! Mirrors upstream `mindustry.logic.RadarTarget`.

use super::RadarUnitView;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadarTarget {
    Any,
    Enemy,
    Ally,
    Player,
    Attacker,
    Flying,
    Boss,
    Ground,
}

impl RadarTarget {
    /// Upstream `Team.derelict` has id 0.
    pub const DERELICT_TEAM: u8 = 0;

    pub const ALL: [RadarTarget; 8] = [
        RadarTarget::Any,
        RadarTarget::Enemy,
        RadarTarget::Ally,
        RadarTarget::Player,
        RadarTarget::Attacker,
        RadarTarget::Flying,
        RadarTarget::Boss,
        RadarTarget::Ground,
    ];

    pub const WIRE_NAMES: [&'static str; 8] = [
        "any", "enemy", "ally", "player", "attacker", "flying", "boss", "ground",
    ];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
    }

    pub fn matches(self, team: u8, other: &RadarUnitView) -> bool {
        match self {
            RadarTarget::Any => true,
            RadarTarget::Enemy => team != other.team && other.team != Self::DERELICT_TEAM,
            RadarTarget::Ally => team == other.team,
            RadarTarget::Player => other.is_player,
            RadarTarget::Attacker => other.can_shoot,
            RadarTarget::Flying => other.is_flying,
            RadarTarget::Boss => other.is_boss,
            RadarTarget::Ground => other.is_grounded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RadarTarget;
    use crate::mindustry::logic::RadarUnitView;

    #[test]
    fn radar_target_order_names_and_ordinals_match_java_enum() {
        assert_eq!(
            RadarTarget::ALL,
            [
                RadarTarget::Any,
                RadarTarget::Enemy,
                RadarTarget::Ally,
                RadarTarget::Player,
                RadarTarget::Attacker,
                RadarTarget::Flying,
                RadarTarget::Boss,
                RadarTarget::Ground,
            ]
        );
        assert_eq!(
            RadarTarget::WIRE_NAMES,
            ["any", "enemy", "ally", "player", "attacker", "flying", "boss", "ground",]
        );
        assert_eq!(RadarTarget::Attacker.ordinal(), 4);
        assert_eq!(RadarTarget::from_ordinal(7), Some(RadarTarget::Ground));
        assert_eq!(RadarTarget::from_ordinal(8), None);
        assert_eq!(
            RadarTarget::by_wire_name("flying"),
            Some(RadarTarget::Flying)
        );
        assert_eq!(RadarTarget::by_wire_name("missing"), None);
    }

    #[test]
    fn radar_target_predicates_follow_java_team_and_unit_rules() {
        let mut unit = RadarUnitView::new(0.0, 0.0, 2);

        assert!(RadarTarget::Any.matches(1, &unit));
        assert!(RadarTarget::Enemy.matches(1, &unit));
        assert!(!RadarTarget::Ally.matches(1, &unit));
        assert!(RadarTarget::Ally.matches(2, &unit));

        unit.team = RadarTarget::DERELICT_TEAM;
        assert!(!RadarTarget::Enemy.matches(1, &unit));

        unit.team = 2;
        unit.is_player = true;
        unit.can_shoot = true;
        unit.is_flying = true;
        unit.is_boss = true;
        unit.is_grounded = true;

        assert!(RadarTarget::Player.matches(1, &unit));
        assert!(RadarTarget::Attacker.matches(1, &unit));
        assert!(RadarTarget::Flying.matches(1, &unit));
        assert!(RadarTarget::Boss.matches(1, &unit));
        assert!(RadarTarget::Ground.matches(1, &unit));
    }
}
