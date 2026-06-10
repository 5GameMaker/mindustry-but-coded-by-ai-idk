//! Generic item list display model mirroring upstream `mindustry.ui.ItemsDisplay`.

use crate::mindustry::{
    r#type::{Item, ItemSeq},
    ui::core_items_display::format_amount,
};

pub const ITEMS_DISPLAY_BUTTON_KEY: &str = "@globalitems";
pub const ITEMS_DISPLAY_COLLAPSER_DURATION: f32 = 0.3;
pub const ITEMS_DISPLAY_ICON_SIZE: f32 = 24.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemsDisplayRow {
    pub item_name: String,
    pub localized_name: String,
    pub amount: i32,
    pub amount_label: String,
    pub shine: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemsDisplayModel {
    pub collapsed: bool,
    pub button_key: &'static str,
    pub button_icon: &'static str,
    pub icon_size: f32,
    pub collapser_duration: f32,
    pub rows: Vec<ItemsDisplayRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemsDisplay {
    collapsed: bool,
}

impl ItemsDisplay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn collapsed(&self) -> bool {
        self.collapsed
    }

    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.collapsed = collapsed;
    }

    pub fn toggle(&mut self) {
        self.collapsed = !self.collapsed;
    }

    pub fn rebuild(
        &self,
        content_items: &[Item],
        items: Option<&ItemSeq>,
        shine: Option<&[bool]>,
    ) -> ItemsDisplayModel {
        let mut rows = Vec::new();

        if let Some(items) = items {
            for (index, item) in content_items.iter().enumerate() {
                if !items.has(index) {
                    continue;
                }

                let amount = items.get(index);
                rows.push(ItemsDisplayRow {
                    item_name: item.name().to_string(),
                    localized_name: item.localized_name().to_string(),
                    amount,
                    amount_label: format_amount(amount as i64),
                    shine: shine
                        .and_then(|values| values.get(index))
                        .copied()
                        .unwrap_or(false),
                });
            }
        }

        ItemsDisplayModel {
            collapsed: self.collapsed,
            button_key: ITEMS_DISPLAY_BUTTON_KEY,
            button_icon: if self.collapsed { "upOpen" } else { "downOpen" },
            icon_size: ITEMS_DISPLAY_ICON_SIZE,
            collapser_duration: ITEMS_DISPLAY_COLLAPSER_DURATION,
            rows,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::content::items;

    #[test]
    fn items_display_rebuild_keeps_content_order_and_format_amounts() {
        let content = items::load();
        let names = content.iter().map(Item::name).collect::<Vec<_>>();
        let mut seq = ItemSeq::new(names);
        seq.set(1, 1500);
        seq.set(3, 2);

        let model = ItemsDisplay::new().rebuild(&content, Some(&seq), Some(&[false, true]));

        assert_eq!(model.button_key, "@globalitems");
        assert_eq!(model.button_icon, "downOpen");
        assert_eq!(model.rows[0].item_name, "copper");
        assert_eq!(model.rows[0].amount_label, "1.5[gray]k[]");
        assert!(model.rows[0].shine);
        assert_eq!(model.rows[1].amount, 2);
    }

    #[test]
    fn items_display_collapsed_state_controls_button_icon_like_java_update() {
        let mut display = ItemsDisplay::new();
        let content = items::load();
        let seq = ItemSeq::new(content.iter().map(Item::name));

        display.set_collapsed(true);
        let model = display.rebuild(&content, Some(&seq), None);

        assert!(model.collapsed);
        assert_eq!(model.button_icon, "upOpen");
        display.toggle();
        assert!(!display.collapsed());
    }

    #[test]
    fn items_display_accepts_null_items_like_java_rebuild() {
        let content = items::load();
        let model = ItemsDisplay::new().rebuild(&content, None, None);
        assert!(model.rows.is_empty());
    }
}
