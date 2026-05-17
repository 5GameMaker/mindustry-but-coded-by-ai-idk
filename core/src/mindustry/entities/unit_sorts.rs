#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SortTarget {
    pub x: f32,
    pub y: f32,
    pub max_health: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildingPriorityTarget {
    pub priority: i32,
    pub has_liquids: bool,
    pub liquid_water: f32,
}

pub type Sortf = fn(&SortTarget, f32, f32) -> f32;
pub type BuildingPriorityf = fn(&BuildingPriorityTarget) -> i32;

#[inline]
fn dst2(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    dx * dx + dy * dy
}

pub fn closest(unit: &SortTarget, x: f32, y: f32) -> f32 {
    dst2(unit.x, unit.y, x, y)
}

pub fn farthest(unit: &SortTarget, x: f32, y: f32) -> f32 {
    -dst2(unit.x, unit.y, x, y)
}

pub fn strongest(unit: &SortTarget, x: f32, y: f32) -> f32 {
    -unit.max_health + dst2(unit.x, unit.y, x, y) / 6400.0
}

pub fn weakest(unit: &SortTarget, x: f32, y: f32) -> f32 {
    unit.max_health + dst2(unit.x, unit.y, x, y) / 6400.0
}

pub fn building_default(building: &BuildingPriorityTarget) -> i32 {
    building.priority
}

pub fn building_water(building: &BuildingPriorityTarget) -> i32 {
    building.priority
        + if building.has_liquids && building.liquid_water > 5.0 {
            10
        } else {
            0
        }
}

#[cfg(test)]
mod tests {
    use super::{
        building_default, building_water, closest, farthest, strongest, weakest,
        BuildingPriorityTarget, SortTarget,
    };

    #[test]
    fn unit_sorts_match_expected_formulae() {
        let unit = SortTarget {
            x: 0.0,
            y: 0.0,
            max_health: 80.0,
        };

        assert_eq!(closest(&unit, 3.0, 4.0), 25.0);
        assert_eq!(farthest(&unit, 3.0, 4.0), -25.0);
        assert_eq!(strongest(&unit, 3.0, 4.0), -79.99609375);
        assert_eq!(weakest(&unit, 3.0, 4.0), 80.00390625);
    }

    #[test]
    fn building_sorts_apply_water_bonus_only_when_needed() {
        let base = BuildingPriorityTarget {
            priority: 3,
            has_liquids: false,
            liquid_water: 0.0,
        };
        let water = BuildingPriorityTarget {
            priority: 3,
            has_liquids: true,
            liquid_water: 6.0,
        };

        assert_eq!(building_default(&base), 3);
        assert_eq!(building_water(&base), 3);
        assert_eq!(building_water(&water), 13);
    }
}
