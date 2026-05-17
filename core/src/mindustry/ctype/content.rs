use super::ContentType;

pub type ContentId = i16;

pub trait Content {
    fn id(&self) -> ContentId;
    fn content_type(&self) -> ContentType;

    fn init(&mut self) {}
    fn post_init(&mut self) {}
    fn after_patch(&mut self) {}
    fn load(&mut self) {}
    fn load_icon(&mut self) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentBase {
    pub id: ContentId,
    pub content_type: ContentType,
    pub source: Option<String>,
    pub error: Option<String>,
    pub base_error: Option<String>,
}

impl ContentBase {
    pub const fn new(id: ContentId, content_type: ContentType) -> Self {
        Self {
            id,
            content_type,
            source: None,
            error: None,
            base_error: None,
        }
    }

    pub fn has_errored(&self) -> bool {
        self.error.is_some()
    }

    pub fn is_vanilla(&self) -> bool {
        self.source.is_none()
    }

    pub fn is_modded(&self) -> bool {
        self.source.is_some()
    }
}

impl Content for ContentBase {
    fn id(&self) -> ContentId {
        self.id
    }

    fn content_type(&self) -> ContentType {
        self.content_type
    }
}
