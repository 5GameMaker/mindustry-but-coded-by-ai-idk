use std::collections::BTreeMap;

use crate::mindustry::input::{Binding, KeyBindingInput, KeyBindingSpec, KeyCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeybindDialogButtonSize {
    pub width: u16,
    pub height: u16,
}

const CATEGORY_DIVIDER_HEIGHT: u16 = 3;
const ACTION_BUTTON_SIZE: KeybindDialogButtonSize = KeybindDialogButtonSize {
    width: 140,
    height: 40,
};
const RESET_ALL_MIN_WIDTH: u16 = 200;
const RESET_ALL_HEIGHT: u16 = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeybindDialogRow {
    Category(&'static str),
    CategoryDivider {
        height: u16,
    },
    Binding {
        spec: KeyBindingSpec,
        button_size: KeybindDialogButtonSize,
    },
    ResetAll {
        min_width: u16,
        height: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeybindDialog {
    android: bool,
    shown: bool,
    search_text: String,
    keyboard_focus_requested: bool,
    rebind_key: Option<&'static str>,
    rebind_axis: bool,
    rebind_min: bool,
    min_key: Option<KeyCode>,
    overrides: BTreeMap<&'static str, KeyBindingInput>,
}

impl KeybindDialog {
    pub fn new(android: bool) -> Self {
        Self {
            android,
            shown: false,
            search_text: String::new(),
            keyboard_focus_requested: false,
            rebind_key: None,
            rebind_axis: false,
            rebind_min: true,
            min_key: None,
            overrides: BTreeMap::new(),
        }
    }

    pub fn show(&mut self) {
        self.shown = true;
        self.search_text.clear();
        self.keyboard_focus_requested = true;
    }

    pub fn hide(&mut self) {
        self.shown = false;
        self.rebind_key = None;
        self.rebind_axis = false;
        self.rebind_min = true;
        self.min_key = None;
    }

    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn search_text(&self) -> &str {
        &self.search_text
    }

    pub fn set_search_text(&mut self, text: impl Into<String>) {
        self.search_text = text.into();
    }

    pub fn keyboard_focus_requested(&self) -> bool {
        self.keyboard_focus_requested
    }

    pub fn clear_keyboard_focus_request(&mut self) {
        self.keyboard_focus_requested = false;
    }

    pub fn rebind_key(&self) -> Option<&'static str> {
        self.rebind_key
    }

    pub fn rebind_min_key(&self) -> Option<KeyCode> {
        self.min_key
    }

    pub fn override_for(&self, name: &'static str) -> Option<KeyBindingInput> {
        self.overrides.get(name).copied()
    }

    pub fn effective_input(&self, spec: KeyBindingSpec) -> KeyBindingInput {
        self.override_for(spec.name).unwrap_or(spec.input)
    }

    pub fn visible_rows<F>(&self, mut localize_name: F) -> Vec<KeybindDialogRow>
    where
        F: FnMut(&str) -> String,
    {
        let needle = self.search_text.to_lowercase();
        let mut rows = Vec::new();
        let mut last_category = None;
        for spec in Binding::defaults(self.android) {
            let localized = localize_name(spec.name);
            if !needle.is_empty() && !localized.to_lowercase().contains(&needle) {
                continue;
            }
            if last_category != Some(spec.category) {
                rows.push(KeybindDialogRow::Category(spec.category));
                rows.push(KeybindDialogRow::CategoryDivider {
                    height: CATEGORY_DIVIDER_HEIGHT,
                });
                last_category = Some(spec.category);
            }
            rows.push(KeybindDialogRow::Binding {
                spec,
                button_size: ACTION_BUTTON_SIZE,
            });
        }
        rows.push(KeybindDialogRow::ResetAll {
            min_width: RESET_ALL_MIN_WIDTH,
            height: RESET_ALL_HEIGHT,
        });
        rows
    }

    pub fn open_rebind(&mut self, name: &'static str) -> bool {
        let Some(spec) = Binding::find(name, self.android) else {
            return false;
        };
        self.rebind_key = Some(spec.name);
        self.rebind_axis = matches!(
            spec.input,
            KeyBindingInput::AxisPair { .. } | KeyBindingInput::AxisSingle(_)
        );
        self.rebind_min = self.rebind_axis;
        self.min_key = None;
        true
    }

    pub fn rebind(&mut self, key: KeyCode) -> bool {
        let Some(name) = self.rebind_key else {
            return false;
        };
        let Some(spec) = Binding::find(name, self.android) else {
            self.hide_rebind();
            return false;
        };
        let is_axis = matches!(
            spec.input,
            KeyBindingInput::AxisPair { .. } | KeyBindingInput::AxisSingle(_)
        );
        if is_axis {
            if key == KeyCode::Scroll || !self.rebind_min {
                let input = if key == KeyCode::Scroll {
                    KeyBindingInput::AxisSingle(KeyCode::Scroll)
                } else if let Some(min) = self.min_key {
                    KeyBindingInput::AxisPair { min, max: key }
                } else {
                    return false;
                };
                self.overrides.insert(name, input);
                self.hide_rebind();
                return true;
            }
            self.min_key = Some(key);
            self.rebind_min = false;
            return true;
        }
        self.overrides.insert(name, KeyBindingInput::Key(key));
        self.hide_rebind();
        true
    }

    pub fn reset_key(&mut self, name: &'static str) {
        self.overrides.remove(name);
        if self.rebind_key == Some(name) {
            self.hide_rebind();
        }
    }

    pub fn reset_all(&mut self) {
        self.overrides.clear();
        self.hide_rebind();
    }

    fn hide_rebind(&mut self) {
        self.rebind_key = None;
        self.rebind_axis = false;
        self.rebind_min = true;
        self.min_key = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keybind_dialog_show_clears_search_and_requests_keyboard_focus_like_java() {
        let mut dialog = KeybindDialog::new(false);
        dialog.set_search_text("move");
        dialog.clear_keyboard_focus_request();

        dialog.show();

        assert!(dialog.shown());
        assert_eq!(dialog.search_text(), "");
        assert!(dialog.keyboard_focus_requested());
    }

    #[test]
    fn keybind_dialog_search_filters_by_localized_name_and_keeps_categories_like_java() {
        let mut dialog = KeybindDialog::new(false);
        dialog.set_search_text("strafe");
        let rows = dialog.visible_rows(|name| match name {
            "move_x" => "Strafe".into(),
            other => other.into(),
        });

        assert_eq!(rows.first(), Some(&KeybindDialogRow::Category("general")));
        assert_eq!(
            rows.get(1),
            Some(&KeybindDialogRow::CategoryDivider {
                height: CATEGORY_DIVIDER_HEIGHT
            })
        );
        if let Some(KeybindDialogRow::Binding { spec, button_size }) = rows.get(2) {
            assert_eq!(spec.name, "move_x");
            assert_eq!(*button_size, ACTION_BUTTON_SIZE);
        } else {
            panic!("expected move_x binding row");
        }
        assert_eq!(
            rows.last(),
            Some(&KeybindDialogRow::ResetAll {
                min_width: RESET_ALL_MIN_WIDTH,
                height: RESET_ALL_HEIGHT
            })
        );
        assert!(!rows.iter().any(
            |row| matches!(row, KeybindDialogRow::Binding { spec, .. } if spec.name == "move_y")
        ));
    }

    #[test]
    fn keybind_dialog_rows_keep_java_category_dividers_and_button_sizes() {
        let mut dialog = KeybindDialog::new(false);
        dialog.set_search_text("strafe");

        let rows = dialog.visible_rows(|name| match name {
            "move_x" => "Strafe".into(),
            other => other.into(),
        });

        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0], KeybindDialogRow::Category("general"));
        assert_eq!(
            rows[1],
            KeybindDialogRow::CategoryDivider {
                height: CATEGORY_DIVIDER_HEIGHT
            }
        );
        match &rows[2] {
            KeybindDialogRow::Binding { spec, button_size } => {
                assert_eq!(spec.name, "move_x");
                assert_eq!(*button_size, ACTION_BUTTON_SIZE);
            }
            other => panic!("expected binding row, got {other:?}"),
        }
        assert_eq!(
            rows[3],
            KeybindDialogRow::ResetAll {
                min_width: RESET_ALL_MIN_WIDTH,
                height: RESET_ALL_HEIGHT
            }
        );
    }

    #[test]
    fn keybind_dialog_rebinds_axis_min_then_max_and_scroll_like_java() {
        let mut dialog = KeybindDialog::new(false);

        assert!(dialog.open_rebind("move_x"));
        assert!(dialog.rebind(KeyCode::Q));
        assert_eq!(dialog.rebind_key(), Some("move_x"));
        assert_eq!(dialog.rebind_min_key(), Some(KeyCode::Q));
        assert!(dialog.rebind(KeyCode::E));
        assert_eq!(
            dialog.override_for("move_x"),
            Some(KeyBindingInput::AxisPair {
                min: KeyCode::Q,
                max: KeyCode::E
            })
        );
        assert_eq!(dialog.rebind_key(), None);

        assert!(dialog.open_rebind("zoom"));
        assert!(dialog.rebind(KeyCode::Scroll));
        assert_eq!(
            dialog.override_for("zoom"),
            Some(KeyBindingInput::AxisSingle(KeyCode::Scroll))
        );
    }
}
