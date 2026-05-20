use crate::mindustry::ctype::{Content, ContentBase, ContentId, ContentType};

/// Represents a blank type of content that has an error.
///
/// Java `ErrorContent` extends plain `Content`, not `UnlockableContent`; this shell
/// deliberately keeps only the base content fields used by loader/error handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorContent {
    pub base: ContentBase,
}

impl ErrorContent {
    pub const fn new(id: ContentId) -> Self {
        Self {
            base: ContentBase::new(id, ContentType::Error),
        }
    }
}

impl Content for ErrorContent {
    fn id(&self) -> ContentId {
        self.base.id()
    }

    fn content_type(&self) -> ContentType {
        ContentType::Error
    }
}

impl std::fmt::Display for ErrorContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.content_type().wire_name(), self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_content_is_plain_error_content_like_java() {
        let mut content = ErrorContent::new(12);
        assert_eq!(content.id(), 12);
        assert_eq!(content.content_type(), ContentType::Error);
        assert_eq!(content.to_string(), "error#12");
        assert!(content.base.is_vanilla());
        assert!(!content.base.has_errored());

        content.base.error = Some("bad json".into());
        assert!(content.base.has_errored());
    }
}
