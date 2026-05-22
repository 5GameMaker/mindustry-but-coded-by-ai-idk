use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item: String,
    pub amount: i32,
}

impl ItemStack {
    pub const EMPTY: [ItemStack; 0] = [];

    pub fn new(item: impl Into<String>, amount: i32) -> Self {
        Self {
            item: item.into(),
            amount,
        }
    }

    pub fn set(&mut self, item: impl Into<String>, amount: i32) -> &mut Self {
        self.item = item.into();
        self.amount = amount;
        self
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn mult(&self, amount: f32) -> Self {
        Self {
            item: self.item.clone(),
            amount: (self.amount as f32 * amount).round() as i32,
        }
    }

    pub fn mult_all(stacks: &[Self], amount: f32) -> Vec<Self> {
        stacks.iter().map(|stack| stack.mult(amount)).collect()
    }

    pub fn with_pairs<I, S>(items: I) -> Vec<Self>
    where
        I: IntoIterator<Item = (S, i32)>,
        S: Into<String>,
    {
        items
            .into_iter()
            .map(|(item, amount)| Self::new(item, amount))
            .collect()
    }

    pub fn list<I, S>(items: I) -> Vec<Self>
    where
        I: IntoIterator<Item = (S, i32)>,
        S: Into<String>,
    {
        Self::with_pairs(items)
    }

    pub fn copy_all(stacks: &[Self]) -> Vec<Self> {
        stacks.to_vec()
    }
}

impl fmt::Display for ItemStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.item, self.amount)
    }
}

impl PartialOrd for ItemStack {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemStack {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.item.cmp(&other.item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_stack_mutators_copy_and_display_match_java_shape() {
        let mut stack = ItemStack::new("copper", 10);
        assert_eq!(stack.to_string(), "copper: 10");

        stack.set("lead", 20);
        let copied = stack.copy();

        assert_eq!(stack, ItemStack::new("lead", 20));
        assert_eq!(copied, stack);
    }

    #[test]
    fn item_stack_mult_rounds_amounts_like_java_mathf_round() {
        let stacks = vec![ItemStack::new("copper", 3), ItemStack::new("lead", 5)];
        assert_eq!(
            ItemStack::mult_all(&stacks, 1.5),
            vec![ItemStack::new("copper", 5), ItemStack::new("lead", 8)]
        );
        assert_eq!(stacks[0].mult(0.25), ItemStack::new("copper", 1));
    }

    #[test]
    fn item_stack_with_list_copy_and_order_follow_item_identity_name() {
        let stacks = ItemStack::with_pairs([("lead", 2), ("copper", 1)]);
        assert_eq!(
            stacks,
            vec![ItemStack::new("lead", 2), ItemStack::new("copper", 1)]
        );
        assert_eq!(
            ItemStack::list([("graphite", 3)]),
            vec![ItemStack::new("graphite", 3)]
        );
        assert_eq!(ItemStack::copy_all(&stacks), stacks);
        assert!(ItemStack::new("copper", 99) < ItemStack::new("lead", 1));
        assert_eq!(
            ItemStack::new("copper", 1).cmp(&ItemStack::new("copper", 999)),
            std::cmp::Ordering::Equal
        );
    }
}
