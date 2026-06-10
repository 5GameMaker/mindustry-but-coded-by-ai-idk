//! Effects browser dialog model mirroring upstream `mindustry.ui.dialogs.EffectsDialog`.

use crate::mindustry::logic::{LogicEffectEntry, LogicEffectRegistry};

pub const EFFECTS_DIALOG_TITLE: &str = "Effects";
pub const EFFECTS_DIALOG_CELL_SIZE: f32 = 280.0;
pub const EFFECTS_DIALOG_COLUMN_PAD: f32 = 12.0;
pub const EFFECTS_DIALOG_BORDER_STROKE: f32 = 3.0;
pub const EFFECTS_DIALOG_BACKGROUND_LIGHTNESS_HOVER: f32 = 0.4;
pub const EFFECTS_DIALOG_BACKGROUND_LIGHTNESS_NORMAL: f32 = 0.5;

#[derive(Debug, Clone, PartialEq)]
pub struct EffectsDialogCell {
    pub entry: LogicEffectEntry,
    pub preview_size: f32,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectsDialogModel {
    pub title: &'static str,
    pub columns: i32,
    pub cell_size: f32,
    pub scroll_x: bool,
    pub listener_present: bool,
    pub cells: Vec<EffectsDialogCell>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EffectsDialogAction {
    Selected(String),
    Hide,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectPreviewState {
    pub entry_name: String,
    pub id: i32,
    pub time: f32,
    pub lifetime: f32,
    pub rotation: f32,
    pub data: Option<&'static str>,
    pub size: f32,
}

impl EffectPreviewState {
    pub fn new(entry: &LogicEffectEntry, lifetime: f32, size: f32) -> Self {
        Self {
            entry_name: entry.name.clone(),
            id: 1,
            time: 0.0,
            lifetime,
            rotation: 1.0,
            data: entry.data,
            size,
        }
    }

    pub fn act(&mut self, delta: f32) {
        self.time += delta;
        if self.time >= self.lifetime {
            self.id += 1;
        }
        self.time %= self.lifetime;
    }

    pub fn draw_plan(
        &self,
        width: f32,
        hovered: bool,
        listener_present: bool,
    ) -> EffectPreviewDrawPlan {
        EffectPreviewDrawPlan {
            background_lightness: if hovered && listener_present {
                EFFECTS_DIALOG_BACKGROUND_LIGHTNESS_HOVER
            } else {
                EFFECTS_DIALOG_BACKGROUND_LIGHTNESS_NORMAL
            },
            border_stroke: EFFECTS_DIALOG_BORDER_STROKE,
            scale: width / self.size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectPreviewDrawPlan {
    pub background_lightness: f32,
    pub border_stroke: f32,
    pub scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectsDialog {
    entries: Vec<LogicEffectEntry>,
    listener_present: bool,
    visible: bool,
}

impl Default for EffectsDialog {
    fn default() -> Self {
        Self::new(LogicEffectRegistry::new().entries().to_vec())
    }
}

impl EffectsDialog {
    pub fn new(entries: Vec<LogicEffectEntry>) -> Self {
        Self {
            entries,
            listener_present: false,
            visible: false,
        }
    }

    pub fn from_logic_registry(registry: &LogicEffectRegistry) -> Self {
        Self::new(registry.entries().to_vec())
    }

    pub fn with_all_effects(entries: Vec<LogicEffectEntry>) -> Self {
        Self::new(entries)
    }

    pub fn show_with_listener(&mut self) {
        self.listener_present = true;
        self.visible = true;
    }

    pub fn show_browse(&mut self) {
        self.listener_present = false;
        self.visible = true;
    }

    pub fn setup_with_bounds(
        &mut self,
        graphics_width: f32,
        scl: f32,
        mut estimator: impl FnMut(&LogicEffectEntry) -> f32,
    ) -> EffectsDialogModel {
        let columns = (graphics_width
            / (scl * (EFFECTS_DIALOG_CELL_SIZE + EFFECTS_DIALOG_COLUMN_PAD)))
            .max(1.0) as i32;
        let mut cells = Vec::new();
        for entry in &mut self.entries {
            let bounds = calculate_size(entry, &mut estimator);
            if bounds <= 0.0 {
                continue;
            }
            cells.push(EffectsDialogCell {
                entry: entry.clone(),
                preview_size: bounds + 1.0,
                label: entry.name.clone(),
            });
        }

        EffectsDialogModel {
            title: EFFECTS_DIALOG_TITLE,
            columns,
            cell_size: EFFECTS_DIALOG_CELL_SIZE,
            scroll_x: false,
            listener_present: self.listener_present,
            cells,
        }
    }

    pub fn click_cell(&mut self, entry_name: &str) -> Vec<EffectsDialogAction> {
        if self.listener_present {
            self.visible = false;
            vec![
                EffectsDialogAction::Selected(entry_name.to_string()),
                EffectsDialogAction::Hide,
            ]
        } else {
            Vec::new()
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }
}

pub fn calculate_size(
    entry: &mut LogicEffectEntry,
    estimator: &mut impl FnMut(&LogicEffectEntry) -> f32,
) -> f32 {
    if entry.bounds >= 0.0 {
        return entry.bounds;
    }

    let max = estimator(entry);
    if max <= 0.0 {
        -1.0
    } else {
        entry.bounds = max * 2.0;
        entry.bounds
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EffectsBoundsBatch {
    pub max: f32,
}

impl EffectsBoundsBatch {
    pub fn reset(&mut self) {
        self.max = 0.0;
    }

    pub fn max_values(&mut self, values: &[f32]) {
        for value in values {
            if !value.is_nan() {
                self.max = self.max.max(value.abs());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_uses_java_column_formula_and_logic_effect_order() {
        let mut dialog = EffectsDialog::default();
        dialog.show_browse();
        let model = dialog.setup_with_bounds(900.0, 1.0, |_| 20.0);

        assert_eq!(model.title, "Effects");
        assert_eq!(model.columns, 3);
        assert!(!model.scroll_x);
        assert!(!model.listener_present);
        assert_eq!(model.cells.first().unwrap().entry.name, "warn");
    }

    #[test]
    fn calculate_size_uses_cached_bounds_or_stores_estimated_diameter() {
        let mut cached = LogicEffectEntry::new("cached", "fx").bounds(42.0);
        assert_eq!(calculate_size(&mut cached, &mut |_| 5.0), 42.0);

        let mut dynamic = LogicEffectEntry::new("dynamic", "fx");
        assert_eq!(calculate_size(&mut dynamic, &mut |_| 9.0), 18.0);
        assert_eq!(dynamic.bounds, 18.0);
    }

    #[test]
    fn clicking_cell_selects_and_hides_only_when_listener_exists() {
        let mut dialog = EffectsDialog::default();
        dialog.show_browse();
        assert!(dialog.click_cell("warn").is_empty());

        dialog.show_with_listener();
        assert_eq!(
            dialog.click_cell("warn"),
            vec![
                EffectsDialogAction::Selected("warn".to_string()),
                EffectsDialogAction::Hide
            ]
        );
        assert!(!dialog.visible());
    }

    #[test]
    fn preview_state_loops_time_and_increments_seed_id() {
        let entry = LogicEffectEntry::new("warn", "unitCapKill").bounds(20.0);
        let mut state = EffectPreviewState::new(&entry, 10.0, 21.0);

        state.act(12.0);

        assert_eq!(state.id, 2);
        assert_eq!(state.time, 2.0);
        assert_eq!(
            state.draw_plan(280.0, true, true),
            EffectPreviewDrawPlan {
                background_lightness: 0.4,
                border_stroke: 3.0,
                scale: 280.0 / 21.0,
            }
        );
    }

    #[test]
    fn bounds_batch_tracks_absolute_non_nan_max_like_java_batch() {
        let mut batch = EffectsBoundsBatch::default();
        batch.max_values(&[-2.0, 3.0, f32::NAN, -7.0]);

        assert_eq!(batch.max, 7.0);
    }
}
