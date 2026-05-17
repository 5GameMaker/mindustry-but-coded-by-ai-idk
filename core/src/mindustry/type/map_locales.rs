use std::collections::BTreeMap;

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

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn current_locale() -> String {
        std::env::var("LANG")
            .ok()
            .and_then(|value| value.split('.').next().map(|s| s.to_string()))
            .and_then(|value| value.split('_').next().map(|s| s.to_string()))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "en".to_string())
    }

    pub fn contains_property(&self, key: &str) -> bool {
        let locale = Self::current_locale();
        self.contains_property_for(&locale, key) || self.contains_property_for("en", key)
    }

    pub fn get_property(&self, key: &str) -> String {
        let locale = Self::current_locale();
        self.get_property_for(&locale, key)
            .or_else(|| self.get_property_for("en", key))
            .unwrap_or_else(|| format!("???{key}???"))
    }

    pub fn get_formatted(&self, key: &str, args: &[String]) -> String {
        let mut result = self.get_property(key);

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
