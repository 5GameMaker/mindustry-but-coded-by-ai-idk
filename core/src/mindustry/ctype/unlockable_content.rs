use super::{ContentId, ContentType, MappableContentBase};

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
}
