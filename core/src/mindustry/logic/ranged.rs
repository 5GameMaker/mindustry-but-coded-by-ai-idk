//! Mirrors upstream `mindustry.logic.Ranged`.

/// Java `Ranged` extends `Posc` and `Teamc`; Rust keeps the required position
/// and team accessors directly on the trait until the generated entity traits
/// are fully split out.
pub trait Ranged {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn team(&self) -> u8;
    fn range(&self) -> f32;
}

#[cfg(test)]
mod tests {
    use super::Ranged;

    struct TurretRange {
        x: f32,
        y: f32,
        team: u8,
        range: f32,
    }

    impl Ranged for TurretRange {
        fn x(&self) -> f32 {
            self.x
        }

        fn y(&self) -> f32 {
            self.y
        }

        fn team(&self) -> u8 {
            self.team
        }

        fn range(&self) -> f32 {
            self.range
        }
    }

    #[test]
    fn ranged_trait_exposes_position_team_and_range_contract() {
        let ranged = TurretRange {
            x: 12.0,
            y: 24.0,
            team: 3,
            range: 160.0,
        };

        assert_eq!(ranged.x(), 12.0);
        assert_eq!(ranged.y(), 24.0);
        assert_eq!(ranged.team(), 3);
        assert_eq!(ranged.range(), 160.0);
    }
}
