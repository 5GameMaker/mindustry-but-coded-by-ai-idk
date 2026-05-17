#[derive(Debug, Clone, PartialEq)]
pub struct LiquidStack {
    pub liquid: String,
    pub amount: f32,
}

impl LiquidStack {
    pub const EMPTY: [LiquidStack; 0] = [];

    pub fn new(liquid: impl Into<String>, amount: f32) -> Self {
        Self {
            liquid: liquid.into(),
            amount,
        }
    }

    pub fn mult(&self, amount: f32) -> Self {
        Self {
            liquid: self.liquid.clone(),
            amount: self.amount * amount,
        }
    }
}
