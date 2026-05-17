use super::{ContentBase, ContentId, ContentType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappableContentBase {
    pub base: ContentBase,
    pub name: String,
}

impl MappableContentBase {
    pub fn new(id: ContentId, content_type: ContentType, name: impl Into<String>) -> Self {
        Self {
            base: ContentBase::new(id, content_type),
            name: name.into(),
        }
    }
}
