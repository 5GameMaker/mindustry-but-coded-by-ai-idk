//! Mirrors upstream `mindustry.logic.QueryType`.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryType {
    Unit,
    Building,
    Bullet,
}

impl QueryType {
    pub const ALL: [QueryType; 3] = [QueryType::Unit, QueryType::Building, QueryType::Bullet];

    /// Upstream excludes bullets from `queryable` because pooled bullet
    /// references may become stale.
    pub const QUERYABLE: [QueryType; 2] = [QueryType::Unit, QueryType::Building];

    pub const WIRE_NAMES: [&'static str; 3] = ["unit", "building", "bullet"];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }

    pub fn by_wire_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.wire_name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::QueryType;

    #[test]
    fn query_type_order_queryable_and_wire_names_match_java_enum() {
        assert_eq!(
            QueryType::ALL,
            [QueryType::Unit, QueryType::Building, QueryType::Bullet]
        );
        assert_eq!(QueryType::QUERYABLE, [QueryType::Unit, QueryType::Building]);
        assert_eq!(QueryType::Unit.ordinal(), 0);
        assert_eq!(QueryType::Building.ordinal(), 1);
        assert_eq!(QueryType::Bullet.ordinal(), 2);
        assert_eq!(QueryType::Bullet.wire_name(), "bullet");
        assert_eq!(QueryType::from_ordinal(3), None);
        assert_eq!(
            QueryType::by_wire_name("building"),
            Some(QueryType::Building)
        );
        assert_eq!(QueryType::by_wire_name("missing"), None);
    }
}
