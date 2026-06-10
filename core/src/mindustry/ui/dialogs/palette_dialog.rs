//! Palette dialog model mirroring upstream `mindustry.ui.dialogs.PaletteDialog`.

pub const PALETTE_DIALOG_TITLE: &str = "";
pub const PALETTE_BUTTON_DRAWABLE: &str = "Tex.whiteui";
pub const PALETTE_BUTTON_STYLE: &str = "Styles.squareTogglei";
pub const PALETTE_BUTTON_IMAGE_SIZE: f32 = 34.0;
pub const PALETTE_BUTTON_SIZE: f32 = 48.0;
pub const PALETTE_COLUMNS: usize = 4;

pub const PLAYER_COLORS: [u32; 16] = [
    0x82759aff, 0xc0c1c5ff, 0xffffffff, 0x7d2953ff, 0xff074eff, 0xff072aff, 0xff76a6ff, 0xa95238ff,
    0xffa108ff, 0xfeeb2cff, 0xffcaa8ff, 0x008551ff, 0x00e339ff, 0x423c7bff, 0x4b5ef1ff, 0x2cabfeff,
];

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteDialogModel {
    pub title: &'static str,
    pub close_on_back: bool,
    pub buttons: Vec<PaletteColorButton>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteColorButton {
    pub index: usize,
    pub color_rgba: u32,
    pub drawable: &'static str,
    pub style: &'static str,
    pub image_size: f32,
    pub size: f32,
    pub checked: bool,
    pub row_after: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteDialogAction {
    AcceptColor { color_rgba: u32 },
    HideDialog,
}

pub struct PaletteDialog;

impl PaletteDialog {
    pub fn model(current_color_rgba: u32) -> PaletteDialogModel {
        PaletteDialogModel {
            title: PALETTE_DIALOG_TITLE,
            close_on_back: true,
            buttons: PLAYER_COLORS
                .iter()
                .copied()
                .enumerate()
                .map(|(index, color_rgba)| PaletteColorButton {
                    index,
                    color_rgba,
                    drawable: PALETTE_BUTTON_DRAWABLE,
                    style: PALETTE_BUTTON_STYLE,
                    image_size: PALETTE_BUTTON_IMAGE_SIZE,
                    size: PALETTE_BUTTON_SIZE,
                    checked: current_color_rgba == color_rgba,
                    row_after: index % PALETTE_COLUMNS == PALETTE_COLUMNS - 1,
                })
                .collect(),
        }
    }

    pub fn select_plan(index: usize) -> Vec<PaletteDialogAction> {
        vec![
            PaletteDialogAction::AcceptColor {
                color_rgba: PLAYER_COLORS[index],
            },
            PaletteDialogAction::HideDialog,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_builds_player_color_grid_like_java_palette() {
        let model = PaletteDialog::model(0xff074eff);

        assert_eq!(model.title, "");
        assert!(model.close_on_back);
        assert_eq!(model.buttons.len(), 16);
        assert_eq!(model.buttons[0].color_rgba, 0x82759aff);
        assert_eq!(model.buttons[0].drawable, "Tex.whiteui");
        assert_eq!(model.buttons[0].style, "Styles.squareTogglei");
        assert_eq!(model.buttons[0].image_size, 34.0);
        assert_eq!(model.buttons[0].size, 48.0);
        assert!(!model.buttons[0].checked);
        assert!(!model.buttons[0].row_after);
        assert!(model.buttons[3].row_after);
        assert!(model.buttons[4].checked);
        assert_eq!(model.buttons[15].color_rgba, 0x2cabfeff);
        assert!(model.buttons[15].row_after);
    }

    #[test]
    fn selecting_color_invokes_consumer_then_hides_dialog() {
        assert_eq!(
            PaletteDialog::select_plan(14),
            vec![
                PaletteDialogAction::AcceptColor {
                    color_rgba: 0x4b5ef1ff,
                },
                PaletteDialogAction::HideDialog,
            ]
        );
    }
}
