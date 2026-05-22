//! Mirrors upstream `mindustry.logic.Controllable`.

use super::LAccess;

/// An object that can be controlled with logic.
pub trait Controllable {
    type Object;

    fn control(&mut self, access: LAccess, p1: f64, p2: f64, p3: f64, p4: f64);
    fn control_object(&mut self, access: LAccess, p1: Self::Object, p2: f64, p3: f64, p4: f64);
    fn team(&self) -> u8;
}

#[cfg(test)]
mod tests {
    use super::Controllable;
    use crate::mindustry::logic::LAccess;

    #[derive(Default)]
    struct Controlled {
        last_access: Option<LAccess>,
        last_numbers: [f64; 4],
        last_object: Option<String>,
        team: u8,
    }

    impl Controllable for Controlled {
        type Object = String;

        fn control(&mut self, access: LAccess, p1: f64, p2: f64, p3: f64, p4: f64) {
            self.last_access = Some(access);
            self.last_numbers = [p1, p2, p3, p4];
        }

        fn control_object(&mut self, access: LAccess, p1: Self::Object, p2: f64, p3: f64, p4: f64) {
            self.last_access = Some(access);
            self.last_object = Some(p1);
            self.last_numbers = [p2, p3, p4, 0.0];
        }

        fn team(&self) -> u8 {
            self.team
        }
    }

    #[test]
    fn controllable_trait_exposes_numeric_object_and_team_contract() {
        let mut controlled = Controlled {
            team: 3,
            ..Controlled::default()
        };
        controlled.control(LAccess::Shoot, 1.0, 2.0, 3.0, 4.0);
        assert_eq!(controlled.last_access, Some(LAccess::Shoot));
        assert_eq!(controlled.last_numbers, [1.0, 2.0, 3.0, 4.0]);

        controlled.control_object(LAccess::Config, "payload".into(), 5.0, 6.0, 7.0);
        assert_eq!(controlled.last_access, Some(LAccess::Config));
        assert_eq!(controlled.last_object.as_deref(), Some("payload"));
        assert_eq!(controlled.last_numbers, [5.0, 6.0, 7.0, 0.0]);
        assert_eq!(controlled.team(), 3);
    }
}
