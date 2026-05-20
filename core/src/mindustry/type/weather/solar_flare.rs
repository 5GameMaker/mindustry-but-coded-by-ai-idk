use super::Weather;
use crate::mindustry::ctype::ContentId;

#[derive(Debug, Clone, PartialEq)]
pub struct SolarFlare {
    pub weather: Weather,
}

impl SolarFlare {
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
    fn solar_flare_is_weather_todo_shell_like_java() {
        let flare = SolarFlare::new(4, "solar-flare");

        assert_eq!(flare.name(), "solar-flare");
        assert_eq!(flare.weather.content_type(), ContentType::Weather);
        assert_eq!(flare.weather.id(), 4);
        assert_eq!(flare.weather.status, "none");
        assert_eq!(flare.weather.sound, "none");
    }
}
