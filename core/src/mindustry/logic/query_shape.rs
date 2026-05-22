//! Mirrors upstream `mindustry.logic.QueryShape`.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryShape {
    Circle,
    Rect,
}

impl QueryShape {
    pub const ALL: [QueryShape; 2] = [QueryShape::Circle, QueryShape::Rect];
    pub const WIRE_NAMES: [&'static str; 2] = ["circle", "rect"];

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
    use super::QueryShape;

    #[test]
    fn query_shape_order_and_wire_names_match_java_enum() {
        assert_eq!(QueryShape::ALL, [QueryShape::Circle, QueryShape::Rect]);
        assert_eq!(QueryShape::Circle.ordinal(), 0);
        assert_eq!(QueryShape::Rect.ordinal(), 1);
        assert_eq!(QueryShape::Circle.wire_name(), "circle");
        assert_eq!(QueryShape::Rect.wire_name(), "rect");
        assert_eq!(QueryShape::from_ordinal(2), None);
        assert_eq!(QueryShape::by_wire_name("circle"), Some(QueryShape::Circle));
        assert_eq!(QueryShape::by_wire_name("missing"), None);
    }
}
