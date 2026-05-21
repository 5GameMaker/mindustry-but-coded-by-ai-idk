//! Item-carrying component mirroring upstream `mindustry.entities.comp.ItemsComp`.

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStackSlot {
    pub item: Option<String>,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemsComp {
    pub stack: ItemStackSlot,
    pub item_time: f32,
    capacity: i32,
}

impl ItemsComp {
    pub fn new(capacity: i32) -> Self {
        Self {
            stack: ItemStackSlot::default(),
            item_time: 0.0,
            capacity,
        }
    }

    pub fn item_capacity(&self) -> i32 {
        self.capacity
    }

    pub fn set_item_capacity(&mut self, capacity: i32) {
        self.capacity = capacity;
        self.stack.amount = self.stack.amount.clamp(0, self.capacity);
    }

    pub fn update(&mut self) {
        self.stack.amount = self.stack.amount.clamp(0, self.item_capacity());
        self.item_time = lerp_delta(
            self.item_time,
            if self.has_item() { 1.0 } else { 0.0 },
            0.05,
        );
    }

    pub fn item(&self) -> Option<&str> {
        self.stack.item.as_deref()
    }

    pub fn clear_item(&mut self) {
        self.stack.amount = 0;
    }

    pub fn accepts_item(&self, item: &str) -> bool {
        !self.has_item()
            || (self.stack.item.as_deref() == Some(item)
                && self.stack.amount + 1 <= self.item_capacity())
    }

    pub fn has_item(&self) -> bool {
        self.stack.amount > 0
    }

    pub fn add_item(&mut self, item: impl Into<String>) {
        self.add_item_amount(item, 1);
    }

    pub fn add_item_amount(&mut self, item: impl Into<String>, amount: i32) {
        let item = item.into();
        self.stack.amount = if self.stack.item.as_deref() == Some(item.as_str()) {
            self.stack.amount + amount
        } else {
            amount
        };
        self.stack.item = Some(item);
        self.stack.amount = self.stack.amount.clamp(0, self.item_capacity());
    }

    pub fn max_accepted(&self, item: &str) -> i32 {
        if self.stack.item.as_deref() != Some(item) && self.stack.amount > 0 {
            0
        } else {
            self.item_capacity() - self.stack.amount
        }
    }
}

impl Default for ItemsComp {
    fn default() -> Self {
        Self::new(0)
    }
}

fn lerp_delta(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_component_adds_accepts_and_clamps_single_stack() {
        let mut items = ItemsComp::new(3);

        assert!(items.accepts_item("copper"));
        items.add_item_amount("copper", 2);
        assert_eq!(items.item(), Some("copper"));
        assert_eq!(items.stack.amount, 2);
        assert_eq!(items.max_accepted("copper"), 1);
        assert_eq!(items.max_accepted("lead"), 0);

        assert!(items.accepts_item("copper"));
        items.add_item_amount("copper", 5);
        assert_eq!(items.stack.amount, 3);
        assert!(!items.accepts_item("copper"));

        items.clear_item();
        assert!(!items.has_item());
        assert!(items.accepts_item("lead"));
    }

    #[test]
    fn items_component_update_clamps_amount_and_tracks_item_time() {
        let mut items = ItemsComp::new(10);
        items.stack.item = Some("lead".into());
        items.stack.amount = 20;

        items.update();
        assert_eq!(items.stack.amount, 10);
        assert_eq!(items.item_time, 0.05);

        items.clear_item();
        items.update();
        assert_eq!(items.item_time, 0.0475);
    }

    #[test]
    fn items_component_replaces_stack_item_like_java_item_stack() {
        let mut items = ItemsComp::new(5);
        items.add_item_amount("copper", 4);
        items.add_item("lead");

        assert_eq!(items.item(), Some("lead"));
        assert_eq!(items.stack.amount, 1);
    }
}
