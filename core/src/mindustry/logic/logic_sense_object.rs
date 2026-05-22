//! Mirrors the generic senseable runtime object state used by upstream `LExecutor`.

use std::collections::BTreeMap;

use super::LAccess;

#[derive(Debug, Clone, PartialEq)]
pub struct LogicSenseObject {
    pub numeric_senses: BTreeMap<LAccess, f64>,
    pub object_senses: BTreeMap<LAccess, Option<String>>,
    pub content_senses: BTreeMap<String, f64>,
}

impl Default for LogicSenseObject {
    fn default() -> Self {
        Self {
            numeric_senses: BTreeMap::new(),
            object_senses: BTreeMap::new(),
            content_senses: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LogicSenseObject;
    use crate::mindustry::logic::LAccess;

    #[test]
    fn logic_sense_object_defaults_are_empty_java_maps() {
        let mut object = LogicSenseObject::default();
        assert!(object.numeric_senses.is_empty());
        assert!(object.object_senses.is_empty());
        assert!(object.content_senses.is_empty());

        object.numeric_senses.insert(LAccess::Health, 42.0);
        object
            .object_senses
            .insert(LAccess::Name, Some("router".into()));
        object.content_senses.insert("@copper".into(), 12.0);

        assert_eq!(object.numeric_senses[&LAccess::Health], 42.0);
        assert_eq!(
            object.object_senses.get(&LAccess::Name),
            Some(&Some("router".into()))
        );
        assert_eq!(object.content_senses["@copper"], 12.0);
    }
}
