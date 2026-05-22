use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Mutex, MutexGuard, OnceLock};

const VANILLA_ATTRIBUTE_NAMES: [&str; 7] =
    ["heat", "spores", "water", "oil", "light", "sand", "steam"];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attribute {
    pub id: usize,
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AttributeRegistry {
    all: Vec<Attribute>,
    map: BTreeMap<String, usize>,
}

pub trait AttributeEnvironment {
    fn attribute_value(&self, attr: &Attribute) -> f32;
}

impl Attribute {
    pub fn new(id: usize, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    pub fn vanilla() -> Vec<Self> {
        VANILLA_ATTRIBUTE_NAMES
            .into_iter()
            .enumerate()
            .map(|(id, name)| Self::new(id, name))
            .collect()
    }

    pub fn all() -> Vec<Self> {
        lock_global_registry().all()
    }

    pub fn map() -> BTreeMap<String, Self> {
        lock_global_registry().map()
    }

    /// Never returns `None`; mirrors Java `Attribute.get`, which throws when missing.
    pub fn get(name: &str) -> Self {
        let attr = lock_global_registry().get(name);
        attr.unwrap_or_else(|| panic!("Unknown Attribute type: {name}"))
    }

    pub fn get_or_null(name: &str) -> Option<Self> {
        lock_global_registry().get(name)
    }

    pub fn exists(name: &str) -> bool {
        lock_global_registry().exists(name)
    }

    /// Automatically registers this attribute for use. Do not call after mod init.
    pub fn add(name: impl Into<String>) -> Self {
        lock_global_registry().add(name)
    }

    /// Rust-side stand-in for Java `Attribute.env()` before the global `Vars.state` singleton
    /// is migrated. Java returns 0 when `Vars.state == null`, which is the only globally
    /// knowable value here.
    pub fn env(&self) -> f32 {
        0.0
    }

    /// Reads this attribute from an explicit environment attribute bag, matching the
    /// `Vars.state.envAttrs.get(this)` branch of Java `Attribute.env()`.
    pub fn env_from(&self, env: &impl AttributeEnvironment) -> f32 {
        env.attribute_value(self)
    }

    pub fn heat() -> Self {
        Self::get("heat")
    }

    pub fn spores() -> Self {
        Self::get("spores")
    }

    pub fn water() -> Self {
        Self::get("water")
    }

    pub fn oil() -> Self {
        Self::get("oil")
    }

    pub fn light() -> Self {
        Self::get("light")
    }

    pub fn sand() -> Self {
        Self::get("sand")
    }

    pub fn steam() -> Self {
        Self::get("steam")
    }
}

impl AttributeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_vanilla() -> Self {
        let mut registry = Self::new();
        for name in VANILLA_ATTRIBUTE_NAMES {
            registry.add(name);
        }
        registry
    }

    pub fn all(&self) -> Vec<Attribute> {
        self.all.clone()
    }

    pub fn map(&self) -> BTreeMap<String, Attribute> {
        self.map
            .iter()
            .filter_map(|(name, id)| self.all.get(*id).cloned().map(|attr| (name.clone(), attr)))
            .collect()
    }

    pub fn get(&self, name: &str) -> Option<Attribute> {
        self.map.get(name).and_then(|id| self.all.get(*id)).cloned()
    }

    pub fn exists(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    pub fn add(&mut self, name: impl Into<String>) -> Attribute {
        let name = name.into();
        let attr = Attribute::new(self.all.len(), name.clone());
        self.all.push(attr.clone());
        self.map.insert(name, attr.id);
        attr
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

fn global_registry() -> &'static Mutex<AttributeRegistry> {
    static REGISTRY: OnceLock<Mutex<AttributeRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(AttributeRegistry::with_vanilla()))
}

fn lock_global_registry() -> MutexGuard<'static, AttributeRegistry> {
    global_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::{Attribute, AttributeRegistry};
    use crate::mindustry::world::blocks::Attributes;

    #[test]
    fn vanilla_attributes_match_java_static_registration_order() {
        let attrs = Attribute::vanilla();
        let names: Vec<_> = attrs.iter().map(|attr| attr.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["heat", "spores", "water", "oil", "light", "sand", "steam"]
        );
        for (index, attr) in attrs.iter().enumerate() {
            assert_eq!(attr.id, index);
            assert_eq!(attr.to_string(), attr.name);
        }
    }

    #[test]
    fn registry_get_exists_map_and_add_match_java_attribute_api() {
        let mut registry = AttributeRegistry::with_vanilla();
        assert!(registry.exists("heat"));
        assert_eq!(registry.get("heat").unwrap().id, 0);
        assert_eq!(registry.get("missing"), None);

        let added = registry.add("radiation");
        assert_eq!(added.id, 7);
        assert_eq!(added.name, "radiation");
        assert!(registry.exists("radiation"));
        assert_eq!(registry.get("radiation"), Some(added.clone()));
        assert_eq!(registry.all().len(), 8);
        assert_eq!(registry.map().get("radiation"), Some(&added));
    }

    #[test]
    fn global_attribute_helpers_match_java_static_fields_and_throwing_get() {
        assert_eq!(Attribute::heat().id, 0);
        assert_eq!(Attribute::spores().name, "spores");
        assert_eq!(Attribute::water().name, "water");
        assert_eq!(Attribute::oil().name, "oil");
        assert_eq!(Attribute::light().name, "light");
        assert_eq!(Attribute::sand().name, "sand");
        assert_eq!(Attribute::steam().name, "steam");
        assert!(Attribute::exists("heat"));
        assert!(Attribute::get_or_null("heat").is_some());
        assert_eq!(Attribute::get_or_null("missing"), None);
        assert!(std::panic::catch_unwind(|| Attribute::get("missing")).is_err());

        let name = format!("test-attr-{}", std::process::id());
        let before = Attribute::all().len();
        let added = Attribute::add(&name);
        assert_eq!(added.id, before);
        assert_eq!(Attribute::get(&name), added);
        assert_eq!(Attribute::map().get(&name), Some(&added));
    }

    #[test]
    fn env_returns_zero_without_global_state_and_reads_explicit_env_attrs() {
        let heat = Attribute::heat();
        assert_eq!(heat.env(), 0.0);

        let all = Attribute::all();
        let mut env_attrs = Attributes::from_attributes(&all);
        env_attrs.set(&heat, 0.8);

        assert_eq!(heat.env_from(&env_attrs), 0.8);
    }
}
