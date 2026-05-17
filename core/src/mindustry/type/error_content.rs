use crate::mindustry::ctype::{ContentId, ContentType, UnlockableContentBase};

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContent {
    pub base: UnlockableContentBase,
}

impl ErrorContent {
    pub fn new(id: ContentId, name: impl Into<String>) -> Self {
        Self {
            base: UnlockableContentBase::new(id, ContentType::Error, name),
        }
    }
}
