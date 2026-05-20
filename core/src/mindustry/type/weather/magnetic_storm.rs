use super::Weather;
use crate::mindustry::ctype::ContentId;

#[derive(Debug, Clone, PartialEq)]
pub struct MagneticStorm {
    pub weather: Weather,
}

impl MagneticStorm {
    pub fn new(id: ContentId, name: impl Into<String>) -> Self {
        Self {
            weather: Weather::new(id, name),
        }
    }

    pub fn name(&self) -> &str {
        self.weather.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::ctype::{Content, ContentType};

    #[test]
    fn magnetic_storm_is_weather_todo_shell_like_java() {
        let storm = MagneticStorm::new(3, "magnetic-storm");

        assert_eq!(storm.name(), "magnetic-storm");
        assert_eq!(storm.weather.content_type(), ContentType::Weather);
        assert_eq!(storm.weather.id(), 3);
        assert_eq!(storm.weather.status, "none");
        assert_eq!(storm.weather.sound, "none");
    }
}
