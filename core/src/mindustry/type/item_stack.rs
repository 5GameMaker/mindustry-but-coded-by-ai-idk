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

    pub fn mult(&self, amount: i32) -> Self {
        Self {
            item: self.item.clone(),
            amount: self.amount * amount,
        }
    }
}
