//! Icon selection dialog model mirroring upstream `mindustry.ui.dialogs.IconSelectDialog`.

pub const ICON_SELECT_BUTTON_SIZE: f32 = 48.0;
pub const ICON_SELECT_COLUMN_WIDTH: f32 = 52.0;
pub const ICON_SELECT_BACK_BUTTON_SIZE: (f32, f32) = (210.0, 64.0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconSelectContentIcon {
    pub content_type: String,
    pub name: String,
    pub emoji_char: i32,
    pub hidden: bool,
    pub unlocked: bool,
}

impl IconSelectContentIcon {
    pub fn new(
        content_type: impl Into<String>,
        name: impl Into<String>,
        emoji_char: i32,
        hidden: bool,
        unlocked: bool,
    ) -> Self {
        Self {
            content_type: content_type.into(),
            name: name.into(),
            emoji_char,
            hidden,
            unlocked,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconSelectCell {
    NoneIcon {
        code: i32,
    },
    AccessibleIcon {
        name: String,
        code: i32,
    },
    ContentIcon {
        content_type: String,
        name: String,
        code: i32,
    },
    Separator {
        content_type: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct IconSelectDialogModel {
    pub allow_locked: bool,
    pub fill_parent: bool,
    pub columns: i32,
    pub button_size: f32,
    pub back_button_size: (f32, f32),
    pub cells: Vec<IconSelectCell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconSelectDialog {
    allow_locked: bool,
}

impl Default for IconSelectDialog {
    fn default() -> Self {
        Self { allow_locked: true }
    }
}

impl IconSelectDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_allow_locked(allow_locked: bool) -> Self {
        Self { allow_locked }
    }

    pub fn layout(
        &self,
        graphics_width: f32,
        scl: f32,
        accessible_icons: &[(String, i32)],
        content_icons: &[IconSelectContentIcon],
        default_content_types: &[String],
    ) -> IconSelectDialogModel {
        let columns = ((graphics_width / (ICON_SELECT_COLUMN_WIDTH * scl)).min(20.0) as i32).max(1);
        let mut cells = Vec::new();
        cells.push(IconSelectCell::NoneIcon { code: 0 });

        for (name, code) in accessible_icons {
            cells.push(IconSelectCell::AccessibleIcon {
                name: name.clone(),
                code: *code,
            });
        }

        for content_type in default_content_types {
            cells.push(IconSelectCell::Separator {
                content_type: content_type.clone(),
            });
            for icon in content_icons
                .iter()
                .filter(|icon| icon.content_type == *content_type)
            {
                if !icon.hidden && (self.allow_locked || icon.unlocked) {
                    cells.push(IconSelectCell::ContentIcon {
                        content_type: icon.content_type.clone(),
                        name: icon.name.clone(),
                        code: icon.emoji_char,
                    });
                }
            }
        }

        IconSelectDialogModel {
            allow_locked: self.allow_locked,
            fill_parent: true,
            columns,
            button_size: ICON_SELECT_BUTTON_SIZE,
            back_button_size: ICON_SELECT_BACK_BUTTON_SIZE,
            cells,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_select_columns_match_java_width_formula() {
        let model = IconSelectDialog::new().layout(1040.0, 1.0, &[], &[], &[]);
        assert_eq!(model.columns, 20);

        let narrow = IconSelectDialog::new().layout(10.0, 1.0, &[], &[], &[]);
        assert_eq!(narrow.columns, 1);
    }

    #[test]
    fn icon_select_filters_hidden_and_locked_content_like_java() {
        let content = vec![
            IconSelectContentIcon::new("item", "copper", 1, false, true),
            IconSelectContentIcon::new("item", "lead", 2, false, false),
            IconSelectContentIcon::new("item", "hidden", 3, true, true),
        ];
        let types = vec!["item".to_string()];
        let model =
            IconSelectDialog::with_allow_locked(false).layout(520.0, 1.0, &[], &content, &types);

        assert!(model.cells.contains(&IconSelectCell::NoneIcon { code: 0 }));
        assert!(model.cells.contains(&IconSelectCell::Separator {
            content_type: "item".into()
        }));
        assert!(model.cells.contains(&IconSelectCell::ContentIcon {
            content_type: "item".into(),
            name: "copper".into(),
            code: 1
        }));
        assert!(!model.cells.iter().any(|cell| matches!(
            cell,
            IconSelectCell::ContentIcon { name, .. } if name == "lead" || name == "hidden"
        )));
    }
}
