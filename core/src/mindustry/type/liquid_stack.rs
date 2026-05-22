use std::fmt;

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

    pub fn set(&mut self, liquid: impl Into<String>, amount: f32) -> &mut Self {
        self.liquid = liquid.into();
        self.amount = amount;
        self
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn mult(&self, amount: f32) -> Self {
        Self {
            liquid: self.liquid.clone(),
            amount: self.amount * amount,
        }
    }

    pub fn mult_all(stacks: &[Self], amount: f32) -> Vec<Self> {
        stacks.iter().map(|stack| stack.mult(amount)).collect()
    }

    pub fn with_pairs<I, S>(items: I) -> Vec<Self>
    where
        I: IntoIterator<Item = (S, f32)>,
        S: Into<String>,
    {
        items
            .into_iter()
            .map(|(liquid, amount)| Self::new(liquid, amount))
            .collect()
    }

    pub fn list<I, S>(items: I) -> Vec<Self>
    where
        I: IntoIterator<Item = (S, f32)>,
        S: Into<String>,
    {
        Self::with_pairs(items)
    }
}

impl fmt::Display for LiquidStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LiquidStack{{liquid={}, amount={}}}",
            self.liquid, self.amount
        )
    }
}

impl PartialOrd for LiquidStack {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.liquid.cmp(&other.liquid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liquid_stack_mutators_copy_and_display_match_java_shape() {
        let mut stack = LiquidStack::new("water", 10.0);
        assert_eq!(stack.to_string(), "LiquidStack{liquid=water, amount=10}");

        stack.set("slag", 2.5);
        let copied = stack.copy();

        assert_eq!(stack, LiquidStack::new("slag", 2.5));
        assert_eq!(copied, stack);
    }

    #[test]
    fn liquid_stack_mult_keeps_float_precision_like_java() {
        let stacks = vec![LiquidStack::new("water", 3.0), LiquidStack::new("oil", 5.0)];
        assert_eq!(
            LiquidStack::mult_all(&stacks, 1.5),
            vec![LiquidStack::new("water", 4.5), LiquidStack::new("oil", 7.5)]
        );
        assert_eq!(stacks[0].mult(0.25), LiquidStack::new("water", 0.75));
    }

    #[test]
    fn liquid_stack_with_and_list_build_pairs_in_order() {
        assert_eq!(
            LiquidStack::with_pairs([("water", 1.0), ("oil", 2.0)]),
            vec![LiquidStack::new("water", 1.0), LiquidStack::new("oil", 2.0)]
        );
        assert_eq!(
            LiquidStack::list([("slag", 3.0)]),
            vec![LiquidStack::new("slag", 3.0)]
        );
        assert_eq!(
            LiquidStack::new("water", 1.0).partial_cmp(&LiquidStack::new("water", 999.0)),
            Some(std::cmp::Ordering::Equal)
        );
        assert!(LiquidStack::new("oil", 999.0) < LiquidStack::new("water", 1.0));
    }
}
