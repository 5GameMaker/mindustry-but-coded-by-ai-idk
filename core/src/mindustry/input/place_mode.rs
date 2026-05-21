//! Placement mode enum mirroring upstream `mindustry.input.PlaceMode`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum PlaceMode {
    None,
    Breaking,
    Placing,
    SchematicSelect,
    RebuildSelect,
}

impl PlaceMode {
    pub const ALL: [PlaceMode; 5] = [
        PlaceMode::None,
        PlaceMode::Breaking,
        PlaceMode::Placing,
        PlaceMode::SchematicSelect,
        PlaceMode::RebuildSelect,
    ];

    pub fn name(self) -> &'static str {
        match self {
            PlaceMode::None => "none",
            PlaceMode::Breaking => "breaking",
            PlaceMode::Placing => "placing",
            PlaceMode::SchematicSelect => "schematicSelect",
            PlaceMode::RebuildSelect => "rebuildSelect",
        }
    }

    pub fn ordinal(self) -> usize {
        self as usize
    }

    pub fn from_ordinal(ordinal: usize) -> Option<Self> {
        Self::ALL.get(ordinal).copied()
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_mode_order_and_names_match_upstream_enum() {
        assert_eq!(
            PlaceMode::ALL,
            [
                PlaceMode::None,
                PlaceMode::Breaking,
                PlaceMode::Placing,
                PlaceMode::SchematicSelect,
                PlaceMode::RebuildSelect
            ]
        );

        let names: Vec<_> = PlaceMode::ALL.iter().map(|mode| mode.name()).collect();
        assert_eq!(
            names,
            vec![
                "none",
                "breaking",
                "placing",
                "schematicSelect",
                "rebuildSelect"
            ]
        );
        assert_eq!(PlaceMode::Breaking.ordinal(), 1);
        assert_eq!(PlaceMode::from_ordinal(4), Some(PlaceMode::RebuildSelect));
        assert_eq!(PlaceMode::from_ordinal(5), None);
        assert_eq!(
            PlaceMode::from_name("schematicSelect"),
            Some(PlaceMode::SchematicSelect)
        );
        assert_eq!(PlaceMode::from_name("missing"), None);
    }
}
