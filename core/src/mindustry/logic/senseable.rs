//! Mirrors upstream `mindustry.logic.Senseable`.

use super::LAccess;

pub trait Senseable {
    type Content;
    type Object;

    fn sense(&self, sensor: LAccess) -> f64;

    fn sense_content(&self, _content: &Self::Content) -> f64 {
        0.0
    }

    /// Java returns a sentinel `noSensed` object by default; Rust models the
    /// same "no sensed object" result as `None`.
    fn sense_object(&self, _sensor: LAccess) -> Option<Self::Object> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Senseable;
    use crate::mindustry::logic::LAccess;

    struct Sensor;

    impl Senseable for Sensor {
        type Content = &'static str;
        type Object = &'static str;

        fn sense(&self, sensor: LAccess) -> f64 {
            match sensor {
                LAccess::Health => 75.0,
                _ => f64::NAN,
            }
        }
    }

    #[test]
    fn senseable_trait_keeps_java_numeric_and_default_object_contract() {
        let sensor = Sensor;

        assert_eq!(sensor.sense(LAccess::Health), 75.0);
        assert_eq!(sensor.sense_content(&"copper"), 0.0);
        assert_eq!(sensor.sense_object(LAccess::Config), None);
    }
}
