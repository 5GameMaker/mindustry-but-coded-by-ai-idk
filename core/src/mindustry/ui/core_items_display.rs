//! Core item HUD model mirroring upstream `mindustry.ui.CoreItemsDisplay`.

use std::collections::BTreeSet;

use crate::mindustry::{
    r#type::{Item, ItemSeq},
    ui::upstream_menu_bundle_value_for_locale,
};

const ICON_SMALL: f32 = 32.0;
const JAVA_CORE_ITEM_COLUMNS: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreItemsDisplayEntry {
    pub item_name: String,
    pub localized_name: String,
    pub icon_region: String,
    pub amount: i32,
    pub amount_label: String,
    pub amount_tooltip: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoreItemsDisplayRow {
    pub entries: Vec<CoreItemsDisplayEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoreItemsDisplayModel {
    pub background_black6: bool,
    pub margin: f32,
    pub icon_size: f32,
    pub rows: Vec<CoreItemsDisplayRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreItemsDisplay {
    used_items: BTreeSet<String>,
}

impl CoreItemsDisplay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset_used(&mut self) {
        self.used_items.clear();
    }

    pub fn used_items(&self) -> &BTreeSet<String> {
        &self.used_items
    }

    pub fn update_from_core_items(&mut self, content_items: &[Item], core_items: &ItemSeq) -> bool {
        let mut changed = false;
        for (index, item) in content_items.iter().enumerate() {
            if core_items.get(index) > 0 && self.used_items.insert(item.name().to_string()) {
                changed = true;
            }
        }
        changed
    }

    pub fn rebuild_model(
        &self,
        content_items: &[Item],
        core_items: &ItemSeq,
    ) -> CoreItemsDisplayModel {
        let mut rows = Vec::new();
        let mut current = Vec::new();

        for (index, item) in content_items.iter().enumerate() {
            if self.used_items.contains(item.name()) {
                let amount = core_items.get(index);
                current.push(CoreItemsDisplayEntry {
                    item_name: item.name().to_string(),
                    localized_name: item.localized_name().to_string(),
                    icon_region: item.name().to_string(),
                    amount,
                    amount_label: format_amount(amount as i64),
                    amount_tooltip: amount.to_string(),
                });

                if current.len() == JAVA_CORE_ITEM_COLUMNS {
                    rows.push(CoreItemsDisplayRow { entries: current });
                    current = Vec::new();
                }
            }
        }

        if !current.is_empty() {
            rows.push(CoreItemsDisplayRow { entries: current });
        }

        CoreItemsDisplayModel {
            background_black6: !self.used_items.is_empty(),
            margin: if self.used_items.is_empty() { 0.0 } else { 4.0 },
            icon_size: ICON_SMALL,
            rows,
        }
    }
}

pub fn format_amount(number: i64) -> String {
    if number == i64::MAX {
        return "∞".to_string();
    }
    if number == i64::MIN {
        return "-∞".to_string();
    }

    let mag = number.abs();
    let sign = if number < 0 { "-" } else { "" };
    let thousands = upstream_menu_bundle_value_for_locale("en", "unit.thousands").unwrap_or("k");
    let millions = upstream_menu_bundle_value_for_locale("en", "unit.millions").unwrap_or("mil");
    let billions = upstream_menu_bundle_value_for_locale("en", "unit.billions").unwrap_or("bil");

    if mag >= 1_000_000_000 {
        format!(
            "{sign}{}[gray]{billions}[]",
            fixed1(mag as f32 / 1_000_000_000.0)
        )
    } else if mag >= 1_000_000 {
        format!(
            "{sign}{}[gray]{millions}[]",
            fixed1(mag as f32 / 1_000_000.0)
        )
    } else if mag >= 10_000 {
        format!("{}[gray]{thousands}[]", number / 1000)
    } else if mag >= 1000 {
        format!("{sign}{}[gray]{thousands}[]", fixed1(mag as f32 / 1000.0))
    } else {
        number.to_string()
    }
}

fn fixed1(value: f32) -> String {
    format!("{value:.1}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::content::items;

    #[test]
    fn update_tracks_used_items_once_and_preserves_java_content_order() {
        let content_items = items::load();
        let names = content_items.iter().map(Item::name).collect::<Vec<_>>();
        let mut core_items = ItemSeq::new(names);
        core_items.set(1, 42);
        core_items.set(2, 12);

        let mut display = CoreItemsDisplay::new();
        assert!(display.update_from_core_items(&content_items, &core_items));
        assert!(!display.update_from_core_items(&content_items, &core_items));

        let model = display.rebuild_model(&content_items, &core_items);
        assert!(model.background_black6);
        assert_eq!(model.margin, 4.0);
        assert_eq!(model.rows.len(), 1);
        assert_eq!(model.rows[0].entries[0].item_name, "copper");
        assert_eq!(model.rows[0].entries[0].amount_label, "42");
        assert_eq!(model.rows[0].entries[1].item_name, "lead");
    }

    #[test]
    fn reset_used_matches_java_reset_used() {
        let content_items = items::load();
        let names = content_items.iter().map(Item::name).collect::<Vec<_>>();
        let mut core_items = ItemSeq::new(names);
        core_items.set(1, 1);

        let mut display = CoreItemsDisplay::new();
        display.update_from_core_items(&content_items, &core_items);
        display.reset_used();

        let model = display.rebuild_model(&content_items, &core_items);
        assert!(!model.background_black6);
        assert!(model.rows.is_empty());
    }

    #[test]
    fn format_amount_matches_java_ui_format_amount_thresholds() {
        assert_eq!(format_amount(999), "999");
        assert_eq!(format_amount(1000), "1.0[gray]k[]");
        assert_eq!(format_amount(9999), "10.0[gray]k[]");
        assert_eq!(format_amount(10_000), "10[gray]k[]");
        assert_eq!(format_amount(1_500_000), "1.5[gray]mil[]");
        assert_eq!(format_amount(2_000_000_000), "2.0[gray]b[]");
        assert_eq!(format_amount(i64::MAX), "∞");
        assert_eq!(format_amount(i64::MIN), "-∞");
    }
}
