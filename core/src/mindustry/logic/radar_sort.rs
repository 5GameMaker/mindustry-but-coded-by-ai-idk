//! Mirrors upstream `mindustry.logic.RadarSort`.

use super::RadarUnitView;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadarSort {
    Distance,
    Health,
    Shield,
    Armor,
    MaxHealth,
}

impl RadarSort {
    pub const ALL: [RadarSort; 5] = [
        RadarSort::Distance,
        RadarSort::Health,
        RadarSort::Shield,
        RadarSort::Armor,
        RadarSort::MaxHealth,
    ];

    pub const WIRE_NAMES: [&'static str; 5] =
        ["distance", "health", "shield", "armor", "maxHealth"];

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

    pub fn score(self, origin_x: f32, origin_y: f32, other: &RadarUnitView) -> f32 {
        match self {
            RadarSort::Distance => {
                let dx = origin_x - other.x;
                let dy = origin_y - other.y;
                -(dx * dx + dy * dy)
            }
            RadarSort::Health => other.health,
            RadarSort::Shield => other.shield,
            RadarSort::Armor => other.armor,
            RadarSort::MaxHealth => other.max_health,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RadarSort;
    use crate::mindustry::logic::RadarUnitView;

    #[test]
    fn radar_sort_matches_java_order_and_scores() {
        assert_eq!(RadarSort::ALL.len(), 5);
        assert_eq!(
            RadarSort::ALL,
            [
                RadarSort::Distance,
                RadarSort::Health,
                RadarSort::Shield,
                RadarSort::Armor,
                RadarSort::MaxHealth
            ]
        );
        assert_eq!(
            RadarSort::ALL
                .iter()
                .map(|sort| sort.wire_name())
                .collect::<Vec<_>>(),
            vec!["distance", "health", "shield", "armor", "maxHealth"]
        );
        assert_eq!(RadarSort::MaxHealth.ordinal(), 4);
        assert_eq!(RadarSort::from_ordinal(5), None);
        assert_eq!(
            RadarSort::by_wire_name("maxHealth"),
            Some(RadarSort::MaxHealth)
        );
        assert_eq!(RadarSort::by_wire_name("missing"), None);

        let mut unit = RadarUnitView::new(3.0, 4.0, 2);
        unit.health = 10.0;
        unit.shield = 5.0;
        unit.armor = 2.5;
        unit.max_health = 30.0;
        assert_eq!(RadarSort::Distance.score(0.0, 0.0, &unit), -25.0);
        assert_eq!(RadarSort::Health.score(0.0, 0.0, &unit), 10.0);
        assert_eq!(RadarSort::Shield.score(0.0, 0.0, &unit), 5.0);
        assert_eq!(RadarSort::Armor.score(0.0, 0.0, &unit), 2.5);
        assert_eq!(RadarSort::MaxHealth.score(0.0, 0.0, &unit), 30.0);
    }
}
