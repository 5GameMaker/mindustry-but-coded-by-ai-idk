//! Low-allocation integer bundle formatter mirroring upstream `mindustry.ui.IntFormat`.

use super::upstream_menu_bundle_format_for_locale;

const JAVA_INTEGER_MIN_VALUE: i32 = i32::MIN;

pub struct IntFormat {
    text: String,
    builder: String,
    last_value: i32,
    last_value2: i32,
    converter: Box<dyn Fn(i32) -> String + Send + Sync>,
}

impl IntFormat {
    pub fn new(text: impl Into<String>) -> Self {
        Self::with_converter(text, |value| value.to_string())
    }

    pub fn with_converter(
        text: impl Into<String>,
        converter: impl Fn(i32) -> String + Send + Sync + 'static,
    ) -> Self {
        Self {
            text: text.into(),
            builder: String::new(),
            last_value: JAVA_INTEGER_MIN_VALUE,
            last_value2: JAVA_INTEGER_MIN_VALUE,
            converter: Box::new(converter),
        }
    }

    pub fn get(&mut self, value: i32) -> &str {
        self.get_for_locale("en", value)
    }

    pub fn get_for_locale(&mut self, locale: &str, value: i32) -> &str {
        if self.last_value != value {
            let converted = (self.converter)(value);
            self.builder = format_bundle_text(locale, &self.text, &[converted.as_str()]);
        }
        self.last_value = value;
        &self.builder
    }

    pub fn get2(&mut self, value1: i32, value2: i32) -> &str {
        self.get2_for_locale("en", value1, value2)
    }

    pub fn get2_for_locale(&mut self, locale: &str, value1: i32, value2: i32) -> &str {
        if self.last_value != value1 || self.last_value2 != value2 {
            let first = value1.to_string();
            let second = value2.to_string();
            self.builder =
                format_bundle_text(locale, &self.text, &[first.as_str(), second.as_str()]);
        }
        self.last_value = value1;
        self.last_value2 = value2;
        &self.builder
    }

    pub fn cached_text(&self) -> &str {
        &self.builder
    }
}

fn format_bundle_text(locale: &str, text: &str, args: &[&str]) -> String {
    upstream_menu_bundle_format_for_locale(locale, text, args)
        .unwrap_or_else(|| replace_placeholders(text, args))
}

fn replace_placeholders(text: &str, args: &[&str]) -> String {
    let mut value = text.to_string();
    for (index, arg) in args.iter().enumerate() {
        value = value.replace(&format!("{{{index}}}"), arg);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_value_uses_java_bundle_format_key_and_caches_until_value_changes() {
        let mut format = IntFormat::new("players");

        assert_eq!(format.get(3), "3 players");
        let cached = format.cached_text().as_ptr();
        assert_eq!(format.get(3), "3 players");
        assert_eq!(format.cached_text().as_ptr(), cached);
        assert_eq!(format.get(1), "1 players");
    }

    #[test]
    fn converter_matches_java_constructor_overload() {
        let mut format = IntFormat::with_converter("save.mode", |value| format!("mode-{value}"));

        assert_eq!(format.get(2), "Gamemode: mode-2");
    }

    #[test]
    fn two_value_format_replaces_both_placeholders_like_core_bundle_format() {
        let mut format = IntFormat::new("server.versions");

        assert_eq!(
            format.get2(158, 157),
            "Your version:[accent] 158[]\nServer version:[accent] 157[]"
        );
    }
}
