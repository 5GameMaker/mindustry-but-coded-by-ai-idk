/// Cache-layer registry mirrored from upstream `mindustry.graphics.CacheLayer`.
///
/// The Java side keeps a mutable runtime registry. In Rust we model the stable
/// set of cache-layer kinds as an enum and preserve the upstream order for the
/// built-in layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CacheLayer {
    #[default]
    None,
    Water,
    Mud,
    Tar,
    Slag,
    Arkycite,
    Cryofluid,
    Space,
    Normal,
    Walls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheLayerEntry {
    pub name: &'static str,
    pub layer: CacheLayer,
    pub liquid: bool,
}

impl CacheLayerEntry {
    pub const fn new(name: &'static str, layer: CacheLayer, liquid: bool) -> Self {
        Self {
            name,
            layer,
            liquid,
        }
    }
}

impl CacheLayer {
    pub const BUILTIN: [CacheLayerEntry; 9] = [
        CacheLayerEntry::new("water", CacheLayer::Water, true),
        CacheLayerEntry::new("mud", CacheLayer::Mud, true),
        CacheLayerEntry::new("tar", CacheLayer::Tar, true),
        CacheLayerEntry::new("slag", CacheLayer::Slag, true),
        CacheLayerEntry::new("arkycite", CacheLayer::Arkycite, true),
        CacheLayerEntry::new("cryofluid", CacheLayer::Cryofluid, true),
        CacheLayerEntry::new("space", CacheLayer::Space, false),
        CacheLayerEntry::new("normal", CacheLayer::Normal, false),
        CacheLayerEntry::new("walls", CacheLayer::Walls, false),
    ];

    pub const fn is_liquid(self) -> bool {
        matches!(
            self,
            CacheLayer::Water
                | CacheLayer::Mud
                | CacheLayer::Tar
                | CacheLayer::Slag
                | CacheLayer::Arkycite
                | CacheLayer::Cryofluid
        )
    }

    pub const fn name(self) -> &'static str {
        match self {
            CacheLayer::None => "none",
            CacheLayer::Water => "water",
            CacheLayer::Mud => "mud",
            CacheLayer::Tar => "tar",
            CacheLayer::Slag => "slag",
            CacheLayer::Arkycite => "arkycite",
            CacheLayer::Cryofluid => "cryofluid",
            CacheLayer::Space => "space",
            CacheLayer::Normal => "normal",
            CacheLayer::Walls => "walls",
        }
    }

    pub const fn builtin_entries() -> &'static [CacheLayerEntry; 9] {
        &Self::BUILTIN
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::builtin_entries()
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.layer)
    }
}

#[cfg(test)]
mod tests {
    use super::CacheLayer;

    #[test]
    fn cache_layer_defaults_and_lookup_match_upstream_order() {
        assert_eq!(CacheLayer::default(), CacheLayer::None);
        assert!(!CacheLayer::None.is_liquid());
        assert!(CacheLayer::Water.is_liquid());
        assert_eq!(CacheLayer::Walls.name(), "walls");
        assert_eq!(
            CacheLayer::from_name("cryofluid"),
            Some(CacheLayer::Cryofluid)
        );
        assert_eq!(CacheLayer::from_name("missing"), None);

        let expected = [
            "water",
            "mud",
            "tar",
            "slag",
            "arkycite",
            "cryofluid",
            "space",
            "normal",
            "walls",
        ];
        for (entry, expected_name) in CacheLayer::builtin_entries().iter().zip(expected) {
            assert_eq!(entry.name, expected_name);
            assert_eq!(entry.layer.name(), expected_name);
            assert_eq!(entry.liquid, entry.layer.is_liquid());
        }
    }
}
