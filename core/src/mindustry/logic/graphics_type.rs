//! Mirrors upstream `mindustry.world.blocks.logic.LogicDisplay.GraphicsType`.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphicsType {
    Clear,
    Color,
    Col,
    Stroke,
    Line,
    Rect,
    LineRect,
    Poly,
    LinePoly,
    Triangle,
    Image,
    Print,
    Translate,
    Scale,
    Rotate,
    Reset,
}

impl GraphicsType {
    pub const ALL: [GraphicsType; 16] = [
        GraphicsType::Clear,
        GraphicsType::Color,
        GraphicsType::Col,
        GraphicsType::Stroke,
        GraphicsType::Line,
        GraphicsType::Rect,
        GraphicsType::LineRect,
        GraphicsType::Poly,
        GraphicsType::LinePoly,
        GraphicsType::Triangle,
        GraphicsType::Image,
        GraphicsType::Print,
        GraphicsType::Translate,
        GraphicsType::Scale,
        GraphicsType::Rotate,
        GraphicsType::Reset,
    ];

    pub const WIRE_NAMES: [&'static str; 16] = [
        "clear",
        "color",
        "col",
        "stroke",
        "line",
        "rect",
        "lineRect",
        "poly",
        "linePoly",
        "triangle",
        "image",
        "print",
        "translate",
        "scale",
        "rotate",
        "reset",
    ];

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
    use super::GraphicsType;

    #[test]
    fn graphics_type_order_and_wire_names_match_logic_display_enum() {
        assert_eq!(GraphicsType::ALL.len(), 16);
        assert_eq!(GraphicsType::Clear.ordinal(), 0);
        assert_eq!(GraphicsType::LineRect.ordinal(), 6);
        assert_eq!(GraphicsType::Reset.ordinal(), 15);
        assert_eq!(
            GraphicsType::ALL
                .iter()
                .map(|kind| kind.wire_name())
                .collect::<Vec<_>>(),
            GraphicsType::WIRE_NAMES.to_vec()
        );
        assert_eq!(
            GraphicsType::by_wire_name("print"),
            Some(GraphicsType::Print)
        );
        assert_eq!(GraphicsType::from_ordinal(16), None);
        assert_eq!(GraphicsType::by_wire_name("missing"), None);
    }
}
