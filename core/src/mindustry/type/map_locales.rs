use std::collections::BTreeMap;

use crate::mindustry::ui::upstream_menu_bundle_value_for_locale_owned;

/// Class for storing map-specific locale bundles.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MapLocales {
    pub locales: BTreeMap<String, BTreeMap<String, String>>,
}

impl MapLocales {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_locale(&mut self, locale: impl Into<String>, values: Vec<(String, String)>) {
        let mut map = BTreeMap::new();
        for (key, value) in values {
            map.insert(key, value);
        }
        self.locales.insert(locale.into(), map);
    }

    pub fn to_json_map(&self) -> BTreeMap<String, BTreeMap<String, String>> {
        self.locales.clone()
    }

    pub fn read_json_map(&mut self, values: &BTreeMap<String, BTreeMap<String, String>>) {
        self.locales = values.clone();
    }

    pub fn from_json_map(values: BTreeMap<String, BTreeMap<String, String>>) -> Self {
        Self { locales: values }
    }

    pub fn put_property(
        &mut self,
        locale: impl Into<String>,
        key: impl Into<String>,
        value: impl Into<String>,
    ) {
        self.locales
            .entry(locale.into())
            .or_default()
            .insert(key.into(), value.into());
    }

    pub fn remove_property(&mut self, locale: &str, key: &str) -> Option<String> {
        self.locales
            .get_mut(locale)
            .and_then(|values| values.remove(key))
    }

    pub fn remove_locale(&mut self, locale: &str) -> Option<BTreeMap<String, String>> {
        self.locales.remove(locale)
    }

    pub fn locale_codes(&self) -> Vec<String> {
        self.locales.keys().cloned().collect()
    }

    pub fn property_count_for(&self, locale: &str) -> usize {
        self.locales.get(locale).map(BTreeMap::len).unwrap_or(0)
    }

    pub fn to_json_string(&self) -> String {
        let mut out = String::from("{");
        for (locale_index, (locale, values)) in self.locales.iter().enumerate() {
            if locale_index > 0 {
                out.push(',');
            }
            push_json_string(&mut out, locale);
            out.push(':');
            out.push('{');
            for (value_index, (key, value)) in values.iter().enumerate() {
                if value_index > 0 {
                    out.push(',');
                }
                push_json_string(&mut out, key);
                out.push(':');
                push_json_string(&mut out, value);
            }
            out.push('}');
        }
        out.push('}');
        out
    }

    pub fn from_json_str(input: &str) -> Result<Self, String> {
        let mut parser = MapLocalesJsonParser::new(input);
        let locales = parser.parse_outer_object()?;
        parser.skip_ws();
        if parser.is_done() {
            Ok(Self { locales })
        } else {
            Err("trailing data in map locales json".into())
        }
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn current_locale() -> String {
        Self::current_locale_from_lang(std::env::var("LANG").ok().as_deref())
    }

    pub fn current_locale_from_game_setting(locale: &str) -> String {
        Self::current_locale_from_setting(locale, std::env::var("LANG").ok().as_deref())
    }

    pub fn current_locale_from_setting(locale: &str, system_lang: Option<&str>) -> String {
        let locale = locale.trim();
        if locale.is_empty() || locale == "default" {
            return Self::current_locale_from_lang(system_lang);
        }
        locale.replace('-', "_")
    }

    pub fn current_locale_from_lang(lang: Option<&str>) -> String {
        let Some(lang) = lang.map(str::trim).filter(|value| !value.is_empty()) else {
            return "en".to_string();
        };

        let base = lang
            .split(|ch| ch == '.' || ch == '@')
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("en");

        if base.eq_ignore_ascii_case("c") || base.eq_ignore_ascii_case("posix") {
            return "en".to_string();
        }

        let base = base.replace('-', "_");
        let mut parts = base.split('_').filter(|part| !part.is_empty());
        let Some(language) = parts.next() else {
            return "en".to_string();
        };

        let mut normalized = normalize_locale_segment(language, 0);
        for (index, part) in parts.enumerate() {
            normalized.push('_');
            normalized.push_str(&normalize_locale_segment(part, index + 1));
        }

        if normalized.is_empty() {
            "en".to_string()
        } else {
            normalized
        }
    }

    pub fn contains_property(&self, key: &str) -> bool {
        let locale = Self::current_locale();
        self.contains_property_for(&locale, key) || self.contains_property_for("en", key)
    }

    pub fn contains_property_for_setting(&self, locale: &str, key: &str) -> bool {
        let locale = Self::current_locale_from_game_setting(locale);
        self.contains_property_for(&locale, key) || self.contains_property_for("en", key)
    }

    pub fn get_property(&self, key: &str) -> String {
        let locale = Self::current_locale();
        self.get_property_for_locale_or_global(&locale, key)
    }

    pub fn get_property_for_setting(&self, locale: &str, key: &str) -> String {
        let locale = Self::current_locale_from_game_setting(locale);
        self.get_property_for_locale_or_global(&locale, key)
    }

    pub fn get_property_for_locale_or_global(&self, locale: &str, key: &str) -> String {
        self.get_property_for(locale, key)
            .or_else(|| self.get_property_for("en", key))
            .or_else(|| upstream_menu_bundle_value_for_locale_owned(locale, key))
            .unwrap_or_else(|| format!("???{key}???"))
    }

    pub fn get_formatted(&self, key: &str, args: &[String]) -> String {
        let locale = Self::current_locale();
        self.get_formatted_for(&locale, key, args)
    }

    pub fn get_formatted_for_setting(&self, locale: &str, key: &str, args: &[String]) -> String {
        let locale = Self::current_locale_from_game_setting(locale);
        self.get_formatted_for(&locale, key, args)
    }

    pub fn get_formatted_for(&self, locale: &str, key: &str, args: &[String]) -> String {
        let mut result = self
            .get_property_for(locale, key)
            .or_else(|| self.get_property_for("en", key))
            .unwrap_or_else(|| format!("???{key}???"));

        for arg in args {
            if let Some(index) = result.find('@') {
                result.replace_range(index..index + 1, arg);
            } else {
                break;
            }
        }

        result
    }

    fn contains_property_for(&self, locale: &str, key: &str) -> bool {
        self.locales
            .get(locale)
            .and_then(|map| map.get(key))
            .is_some()
    }

    fn get_property_for(&self, locale: &str, key: &str) -> Option<String> {
        self.locales
            .get(locale)
            .and_then(|map| map.get(key))
            .cloned()
    }
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
}

fn normalize_locale_segment(segment: &str, index: usize) -> String {
    match index {
        0 => segment.to_ascii_lowercase(),
        _ if segment.len() == 2 || segment.len() == 3 => segment.to_ascii_uppercase(),
        _ if segment.len() == 4 => {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => {
                    let mut out = first.to_ascii_uppercase().to_string();
                    let tail = chars.as_str().to_ascii_lowercase();
                    out.push_str(&tail);
                    out
                }
                None => String::new(),
            }
        }
        _ => segment.to_string(),
    }
}

struct MapLocalesJsonParser<'a> {
    chars: Vec<char>,
    index: usize,
    _source: &'a str,
}

impl<'a> MapLocalesJsonParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().collect(),
            index: 0,
            _source: source,
        }
    }

    fn is_done(&self) -> bool {
        self.index >= self.chars.len()
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.index += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.index += 1;
        Some(ch)
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        self.skip_ws();
        match self.next() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(format!("expected '{expected}', found '{ch}'")),
            None => Err(format!("expected '{expected}', found end of input")),
        }
    }

    fn parse_outer_object(&mut self) -> Result<BTreeMap<String, BTreeMap<String, String>>, String> {
        self.expect('{')?;
        let mut out = BTreeMap::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.index += 1;
            return Ok(out);
        }

        loop {
            let locale = self.parse_string()?;
            self.expect(':')?;
            let values = self.parse_string_map()?;
            out.insert(locale, values);
            self.skip_ws();
            match self.next() {
                Some(',') => continue,
                Some('}') => return Ok(out),
                Some(ch) => return Err(format!("expected ',' or '}}', found '{ch}'")),
                None => return Err("unterminated map locales json object".into()),
            }
        }
    }

    fn parse_string_map(&mut self) -> Result<BTreeMap<String, String>, String> {
        self.expect('{')?;
        let mut out = BTreeMap::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.index += 1;
            return Ok(out);
        }

        loop {
            let key = self.parse_string()?;
            self.expect(':')?;
            let value = self.parse_string()?;
            out.insert(key, value);
            self.skip_ws();
            match self.next() {
                Some(',') => continue,
                Some('}') => return Ok(out),
                Some(ch) => return Err(format!("expected ',' or '}}', found '{ch}'")),
                None => return Err("unterminated map locale json object".into()),
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.skip_ws();
        if self.next() != Some('"') {
            return Err("expected json string".into());
        }

        let mut out = String::new();
        loop {
            match self.next() {
                Some('"') => return Ok(out),
                Some('\\') => out.push(self.parse_escape()?),
                Some(ch) => out.push(ch),
                None => return Err("unterminated json string".into()),
            }
        }
    }

    fn parse_escape(&mut self) -> Result<char, String> {
        match self.next() {
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('/') => Ok('/'),
            Some('b') => Ok('\u{08}'),
            Some('f') => Ok('\u{0c}'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('u') => {
                let mut value = 0u32;
                for _ in 0..4 {
                    let ch = self
                        .next()
                        .ok_or_else(|| "incomplete unicode escape".to_string())?;
                    value = value * 16
                        + ch.to_digit(16)
                            .ok_or_else(|| "invalid unicode escape".to_string())?;
                }
                char::from_u32(value).ok_or_else(|| "invalid unicode scalar".into())
            }
            Some(ch) => Err(format!("invalid json escape '\\{ch}'")),
            None => Err("incomplete json escape".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MapLocales;
    use std::collections::BTreeMap;

    fn sample_locales() -> MapLocales {
        let mut locales = MapLocales::new();
        locales.insert_locale(
            "en",
            vec![
                ("name".into(), "Sector @".into()),
                ("desc".into(), "Line\nTwo".into()),
            ],
        );
        locales.insert_locale(
            "zh",
            vec![
                ("name".into(), "区域@".into()),
                ("quote".into(), "\"引号\" 与 \\".into()),
            ],
        );
        locales
    }

    #[test]
    fn map_locales_copy_is_deep_like_java_copy() {
        let original = sample_locales();
        let mut copied = original.copy();
        copied
            .locales
            .get_mut("en")
            .unwrap()
            .insert("name".into(), "Changed".into());

        assert_eq!(original.locales["en"]["name"], "Sector @");
        assert_eq!(copied.locales["en"]["name"], "Changed");
    }

    #[test]
    fn map_locales_property_lookup_uses_locale_then_en_then_missing_marker() {
        let locales = sample_locales();

        assert!(locales.contains_property_for("zh", "name"));
        assert!(locales.contains_property_for("en", "desc"));
        assert!(!locales.contains_property_for("zh", "desc"));
        assert_eq!(locales.get_property_for("zh", "name").unwrap(), "区域@");
        assert_eq!(locales.get_property_for("zh", "desc"), None);
        assert_eq!(locales.get_property_for("en", "desc").unwrap(), "Line\nTwo");
        assert_eq!(locales.get_property_for("en", "missing"), None);
        assert_eq!(
            locales.get_property("definitely.missing.map.locale.key"),
            "???definitely.missing.map.locale.key???"
        );
    }

    #[test]
    fn map_locales_formatted_replaces_at_placeholders_in_order() {
        let locales = sample_locales();

        assert_eq!(
            locales.get_formatted_for("en", "name", &[String::from("42"), String::from("ignored")]),
            "Sector 42"
        );
        assert_eq!(
            locales.get_formatted_for("zh", "name", &[String::from("42")]),
            "区域42"
        );

        let mut custom = MapLocales::new();
        custom.insert_locale("en", vec![("objective".into(), "@ captured @ of @".into())]);
        assert_eq!(
            custom.get_formatted_for(
                "en",
                "objective",
                &[String::from("alpha"), String::from("2"), String::from("3")]
            ),
            "alpha captured 2 of 3"
        );
    }

    #[test]
    fn map_locales_json_map_roundtrips_java_write_read_shape() {
        let locales = sample_locales();
        let map = locales.to_json_map();
        let mut read = MapLocales::new();
        read.read_json_map(&map);

        assert_eq!(read, locales);
        assert_eq!(MapLocales::from_json_map(map), locales);
    }

    #[test]
    fn map_locales_editor_mutations_persist_java_json_shape() {
        let mut locales = sample_locales();

        locales.put_property("fr", "name", "Secteur @");
        locales.put_property("en", "desc", "Updated");
        assert_eq!(locales.property_count_for("fr"), 1);
        assert_eq!(
            locales.locale_codes(),
            vec!["en".to_string(), "fr".to_string(), "zh".to_string()],
            "MapLocalesDialog should expose deterministic locale rows while editing"
        );
        assert_eq!(
            locales.remove_property("zh", "quote"),
            Some("\"引号\" 与 \\".to_string())
        );
        assert_eq!(locales.property_count_for("zh"), 1);
        assert!(locales.remove_property("zh", "missing").is_none());
        let removed = locales
            .remove_locale("zh")
            .expect("delete locale should return its property map for undo/confirmation UI");
        assert_eq!(removed["name"], "区域@");
        assert_eq!(locales.property_count_for("zh"), 0);

        let json = locales.to_json_string();
        assert_eq!(
            json,
            "{\"en\":{\"desc\":\"Updated\",\"name\":\"Sector @\"},\"fr\":{\"name\":\"Secteur @\"}}",
            "MapLocalesDialog apply should write the same nested locale/property JSON shape as Java JsonIO.write(locales)"
        );
        assert_eq!(
            MapLocales::from_json_str(&json).unwrap(),
            locales,
            "edited locale JSON must round-trip before desktop UI writes it into editor.tags"
        );
    }

    #[test]
    fn map_locales_json_string_roundtrips_nested_string_object() {
        let locales = sample_locales();
        let json = locales.to_json_string();
        assert_eq!(
            json,
            "{\"en\":{\"desc\":\"Line\\nTwo\",\"name\":\"Sector @\"},\"zh\":{\"name\":\"区域@\",\"quote\":\"\\\"引号\\\" 与 \\\\\"}}"
        );

        let parsed = MapLocales::from_json_str(&json).unwrap();
        assert_eq!(parsed, locales);

        let pretty = r#"{
            "en": {"name": "Launch @", "desc": "A\/B"},
            "fr": {}
        }"#;
        let parsed = MapLocales::from_json_str(pretty).unwrap();
        assert_eq!(parsed.locales["en"]["name"], "Launch @");
        assert_eq!(parsed.locales["en"]["desc"], "A/B");
        assert!(parsed.locales["fr"].is_empty());
    }

    #[test]
    fn map_locales_json_string_rejects_invalid_shapes() {
        assert!(MapLocales::from_json_str("{\"en\":[]}").is_err());
        assert!(MapLocales::from_json_str("{\"en\":{\"name\":1}}").is_err());
        assert!(MapLocales::from_json_str("{\"en\":{\"name\":\"ok\"}} trailing").is_err());
    }

    #[test]
    fn map_locales_current_locale_parses_lang_like_runtime_fallback() {
        let parsed = MapLocales::current_locale_from_lang(Some("zh_CN.UTF-8"));
        assert_eq!(parsed, "zh_CN");
        assert_eq!(MapLocales::current_locale_from_lang(Some("pt_BR.UTF-8")), "pt_BR");
        assert_eq!(MapLocales::current_locale_from_lang(Some("")), "en");
        assert_eq!(MapLocales::current_locale_from_lang(None), "en");
    }

    #[test]
    fn map_locales_current_locale_prefers_game_settings_locale_over_env_like_java() {
        assert_eq!(
            MapLocales::current_locale_from_setting("zh_CN", Some("ja_JP.UTF-8")),
            "zh_CN",
            "Java MapLocales.currentLocale() reads settings.locale first"
        );
        assert_eq!(
            MapLocales::current_locale_from_setting("zh-CN", Some("ja_JP.UTF-8")),
            "zh_CN",
            "LanguageDialog stores Locale.toString()-style underscores; Rust normalizes hyphenated inputs to the same key"
        );
        assert_eq!(
            MapLocales::current_locale_from_setting("default", Some("ja_JP.UTF-8")),
            "ja_JP",
            "Java Vars loads the default locale as a full Locale, so region codes should survive the LANG fallback"
        );
    }

    #[test]
    fn map_locales_get_property_for_setting_falls_back_to_global_bundle_like_java() {
        let locales = sample_locales();

        assert_eq!(
            locales.get_property_for_setting("zh", "name"),
            "区域@",
            "selected game locale should beat OS locale"
        );
        assert_eq!(
            locales.get_property_for_locale_or_global("fr", "desc"),
            "Line\nTwo",
            "Java MapLocales falls back to map English before Core.bundle"
        );
        assert_eq!(
            locales.get_property_for_locale_or_global("fr", "editor"),
            "Editor",
            "Java MapLocales.getProperty falls back to Core.bundle when map locale and English map bundle miss"
        );
        assert_eq!(
            locales.get_property_for_locale_or_global("fr", "definitely.missing.map.locale.key"),
            "???definitely.missing.map.locale.key???"
        );
    }

    #[test]
    fn map_locales_can_read_explicit_btreemap_shape() {
        let values = BTreeMap::from([(
            String::from("en"),
            BTreeMap::from([(String::from("name"), String::from("Test"))]),
        )]);
        let locales = MapLocales::from_json_map(values);
        assert_eq!(locales.get_property_for("en", "name").unwrap(), "Test");
    }
}
