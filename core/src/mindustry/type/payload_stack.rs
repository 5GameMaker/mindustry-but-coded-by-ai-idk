use std::fmt;

use crate::mindustry::ctype::{ContentId, ContentType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadStack {
    pub content_type: ContentType,
    pub id: ContentId,
    pub name: String,
    pub amount: i32,
}

impl PayloadStack {
    pub fn new(
        content_type: ContentType,
        id: ContentId,
        name: impl Into<String>,
        amount: i32,
    ) -> Self {
        Self {
            content_type,
            id,
            name: name.into(),
            amount,
        }
    }

    pub fn router_default() -> Self {
        Self::new(ContentType::Block, 0, "router", 1)
    }

    pub fn single(content_type: ContentType, id: ContentId, name: impl Into<String>) -> Self {
        Self::new(content_type, id, name, 1)
    }

    pub fn with(pairs: &[(ContentType, ContentId, &str, i32)]) -> Vec<Self> {
        pairs
            .iter()
            .map(|(content_type, id, name, amount)| Self::new(*content_type, *id, *name, *amount))
            .collect()
    }

    pub fn list(pairs: &[(ContentType, ContentId, &str, i32)]) -> Vec<Self> {
        Self::with(pairs)
    }
}

impl fmt::Display for PayloadStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BlockStack{{item={}, amount={}}}",
            self.name, self.amount
        )
    }
}

impl PartialOrd for PayloadStack {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PayloadStack {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.content_type.ordinal(), self.id, &self.name).cmp(&(
            other.content_type.ordinal(),
            other.id,
            &other.name,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_stack_defaults_with_and_order_follow_content_identity() {
        let default = PayloadStack::router_default();
        assert_eq!(default.content_type, ContentType::Block);
        assert_eq!(default.name, "router");
        assert_eq!(default.amount, 1);
        assert_eq!(default.to_string(), "BlockStack{item=router, amount=1}");
        assert_eq!(
            PayloadStack::single(ContentType::Unit, 3, "dagger"),
            PayloadStack::new(ContentType::Unit, 3, "dagger", 1)
        );

        let stacks = PayloadStack::with(&[
            (ContentType::Unit, 3, "dagger", 2),
            (ContentType::Block, 5, "router", 4),
        ]);
        assert_eq!(stacks.len(), 2);
        assert_eq!(stacks[0].amount, 2);
        assert!(stacks[1] < stacks[0]);
        assert_eq!(
            PayloadStack::list(&[(ContentType::Block, 5, "router", 4)]).len(),
            1
        );
    }
}
