//! Hover-display interface mirroring upstream `mindustry.ui.Displayable`.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DisplayTable {
    entries: Vec<String>,
}

impl DisplayTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, entry: impl Into<String>) {
        self.entries.push(entry.into());
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}

pub trait Displayable {
    fn displayable(&self) -> bool {
        true
    }

    fn display(&self, table: &mut DisplayTable);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ItemDisplay;

    impl Displayable for ItemDisplay {
        fn display(&self, table: &mut DisplayTable) {
            table.add("copper");
            table.add("lead");
        }
    }

    struct HiddenDisplay;

    impl Displayable for HiddenDisplay {
        fn displayable(&self) -> bool {
            false
        }

        fn display(&self, table: &mut DisplayTable) {
            table.add("hidden");
        }
    }

    #[test]
    fn displayable_default_is_true_like_java_interface() {
        let display = ItemDisplay;
        assert!(display.displayable());
    }

    #[test]
    fn display_writes_to_table_abstraction() {
        let display = ItemDisplay;
        let mut table = DisplayTable::new();

        display.display(&mut table);

        assert_eq!(table.entries(), &["copper".to_string(), "lead".to_string()]);
    }

    #[test]
    fn implementors_can_override_displayable() {
        assert!(!HiddenDisplay.displayable());
    }
}
