use super::{ContentId, ContentType, MappableContentBase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlockableContentIconCandidates {
    pub generate_icons: bool,
    pub full_candidates: Vec<String>,
    pub ui_candidates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnlockableContentBase {
    pub mappable: MappableContentBase,
    pub localized_name: Option<String>,
    pub description: Option<String>,
    pub details: Option<String>,
    pub always_unlocked: bool,
    pub inline_description: bool,
    pub hide_details: bool,
    pub hide_database: bool,
    pub generate_icons: bool,
    pub selection_size: f32,
    pub unlocked: bool,
}

impl UnlockableContentBase {
    pub fn new(id: ContentId, content_type: ContentType, name: impl Into<String>) -> Self {
        Self {
            mappable: MappableContentBase::new(id, content_type, name),
            localized_name: None,
            description: None,
            details: None,
            always_unlocked: false,
            inline_description: false,
            hide_details: true,
            hide_database: false,
            generate_icons: true,
            selection_size: 24.0,
            unlocked: false,
        }
    }

    pub fn unlock(&mut self) {
        self.unlocked = true;
    }

    pub fn clear_unlock(&mut self) {
        self.unlocked = false;
    }

    pub fn unlocked(&self) -> bool {
        self.always_unlocked || self.unlocked
    }

    /// Pure data representation of the Java `loadIcon()` fallback chain.
    ///
    /// - `full_candidates` mirrors the full-icon lookup order.
    /// - `ui_candidates` is the UI-icon lookup order; it is intended to be tried
    ///   before falling back to `full_candidates`.
    /// - `generate_icons` preserves the icon-generation gate.
    pub fn icon_candidates(&self, full_override: Option<&str>) -> UnlockableContentIconCandidates {
        let mut full_candidates = Vec::with_capacity(6);

        if let Some(full_override) = full_override.filter(|name| !name.is_empty()) {
            full_candidates.push(full_override.to_string());
        }

        let content_type = self.mappable.base.content_type.wire_name();
        let name = self.mappable.name.as_str();

        full_candidates.push(format!("{content_type}-{name}-full"));
        full_candidates.push(format!("{name}-full"));
        full_candidates.push(name.to_string());
        full_candidates.push(format!("{content_type}-{name}"));
        full_candidates.push(format!("{name}1"));

        UnlockableContentIconCandidates {
            generate_icons: self.generate_icons,
            full_candidates,
            ui_candidates: vec![format!("{content_type}-{name}-ui")],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ContentType, UnlockableContentBase};

    #[test]
    fn icon_candidates_match_java_fallback_order() {
        let content = UnlockableContentBase::new(7, ContentType::Block, "router");

        let candidates = content.icon_candidates(Some("router-custom-full"));

        assert!(candidates.generate_icons);
        assert_eq!(
            candidates.full_candidates,
            vec![
                "router-custom-full",
                "block-router-full",
                "router-full",
                "router",
                "block-router",
                "router1",
            ]
        );
        assert_eq!(candidates.ui_candidates, vec!["block-router-ui"]);
    }

    #[test]
    fn icon_candidates_keep_generate_icons_gate_without_altering_names() {
        let mut content = UnlockableContentBase::new(8, ContentType::Item, "copper");
        content.generate_icons = false;

        let candidates = content.icon_candidates(Some(""));

        assert!(!candidates.generate_icons);
        assert_eq!(
            candidates.full_candidates,
            vec![
                "item-copper-full",
                "copper-full",
                "copper",
                "item-copper",
                "copper1",
            ]
        );
        assert_eq!(candidates.ui_candidates, vec!["item-copper-ui"]);
    }
}
