//! Map exception carrying the source map, mirroring upstream `mindustry.maps.MapException`.

use std::fmt;

use super::MapDescriptor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapException {
    pub map: MapDescriptor,
    pub message: String,
}

impl MapException {
    pub fn new(map: MapDescriptor, message: impl Into<String>) -> Self {
        Self {
            map,
            message: message.into(),
        }
    }
}

impl fmt::Display for MapException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MapException {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn map_exception_keeps_map_reference_and_runtime_message() {
        let map = MapDescriptor::new("maps/test.msav", 64, 64, BTreeMap::new(), true, 7, 157);
        let error = MapException::new(map.clone(), "invalid map data");

        assert_eq!(error.map, map);
        assert_eq!(error.message, "invalid map data");
        assert_eq!(error.to_string(), "invalid map data");
    }
}
