//! Build-plan configuration model mirroring upstream `mindustry.ui.fragments.PlanConfigFragment`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanConfigBlock {
    pub name: String,
    pub configurable: bool,
    pub selection_rows: i32,
    pub selection_columns: i32,
    pub size: i32,
}

impl PlanConfigBlock {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            configurable: true,
            selection_rows: 1,
            selection_columns: 1,
            size: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildPlanUiRef {
    pub id: i32,
    pub block: Option<PlanConfigBlock>,
    pub config: Option<String>,
    pub options: Vec<String>,
    pub done: bool,
    pub selectable_or_player_can_configure: bool,
    pub draw_x: f32,
    pub draw_y: f32,
}

impl BuildPlanUiRef {
    pub fn new(id: i32, block: PlanConfigBlock, options: Vec<String>) -> Self {
        Self {
            id,
            block: Some(block),
            config: None,
            options,
            done: false,
            selectable_or_player_can_configure: true,
            draw_x: 0.0,
            draw_y: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlanConfigTableAction {
    Clear,
    BuildItemSelection {
        options: Vec<String>,
        selected: Option<String>,
        rows: i32,
        columns: i32,
    },
    Pack,
    SetTransform(bool),
    Visible(bool),
    ScaleTo {
        x: f32,
        y: f32,
        duration: f32,
    },
    SetOriginCenter,
    SetPositionTop {
        x: f32,
        y: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlanConfigShowResult {
    HiddenBecauseSameOrMissingBlock,
    IgnoredBecauseNotConfigurable,
    SelectedWithoutOptions,
    Shown(PlanConfigTableModel),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanConfigTableModel {
    pub plan_id: i32,
    pub options: Vec<String>,
    pub selected: Option<String>,
    pub rows: i32,
    pub columns: i32,
    pub actions: Vec<PlanConfigTableAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlanConfigFragment {
    table_visible: bool,
    selected: Option<i32>,
}

impl PlanConfigFragment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&mut self) {
        self.table_visible = false;
    }

    pub fn show_config(&mut self, plan: &BuildPlanUiRef) -> PlanConfigShowResult {
        if self.selected == Some(plan.id) || plan.block.is_none() {
            self.hide();
            return PlanConfigShowResult::HiddenBecauseSameOrMissingBlock;
        }

        let block = plan.block.as_ref().unwrap();
        if !block.configurable {
            return PlanConfigShowResult::IgnoredBecauseNotConfigurable;
        }

        self.selected = Some(plan.id);
        let mut actions = vec![PlanConfigTableAction::Clear];
        if plan.options.is_empty() {
            return PlanConfigShowResult::SelectedWithoutOptions;
        }

        actions.extend([
            PlanConfigTableAction::BuildItemSelection {
                options: plan.options.clone(),
                selected: plan.config.clone(),
                rows: block.selection_rows,
                columns: block.selection_columns,
            },
            PlanConfigTableAction::Pack,
            PlanConfigTableAction::SetTransform(true),
            PlanConfigTableAction::Visible(true),
            PlanConfigTableAction::ScaleTo {
                x: 0.0,
                y: 1.0,
                duration: 0.0,
            },
            PlanConfigTableAction::Visible(true),
            PlanConfigTableAction::ScaleTo {
                x: 1.0,
                y: 1.0,
                duration: 0.07,
            },
        ]);
        self.table_visible = true;

        PlanConfigShowResult::Shown(PlanConfigTableModel {
            plan_id: plan.id,
            options: plan.options.clone(),
            selected: plan.config.clone(),
            rows: block.selection_rows,
            columns: block.selection_columns,
            actions,
        })
    }

    pub fn update(
        &mut self,
        plan: &BuildPlanUiRef,
        tile_size: f32,
        screen_position: (f32, f32),
    ) -> Vec<PlanConfigTableAction> {
        let mut actions = vec![PlanConfigTableAction::SetOriginCenter];
        if plan.done || !plan.selectable_or_player_can_configure {
            actions.extend(self.hide());
            return actions;
        }

        let block = plan.block.as_ref().unwrap();
        let y = screen_position.1 - block.size as f32 * tile_size / 2.0 - 1.0;
        actions.push(PlanConfigTableAction::SetPositionTop {
            x: screen_position.0,
            y,
        });
        actions
    }

    pub fn force_hide(&mut self) {
        self.table_visible = false;
        self.selected = None;
    }

    pub fn hide(&mut self) -> Vec<PlanConfigTableAction> {
        self.selected = None;
        self.table_visible = false;
        vec![
            PlanConfigTableAction::ScaleTo {
                x: 0.0,
                y: 1.0,
                duration: 0.06,
            },
            PlanConfigTableAction::Visible(false),
        ]
    }

    pub fn is_shown(&self) -> bool {
        self.table_visible && self.selected.is_some()
    }

    pub fn selected(&self) -> Option<i32> {
        self.selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_config_builds_item_selection_when_block_has_options() {
        let mut fragment = PlanConfigFragment::new();
        let mut block = PlanConfigBlock::new("sorter");
        block.selection_rows = 2;
        block.selection_columns = 4;
        let mut plan = BuildPlanUiRef::new(9, block, vec!["copper".into(), "lead".into()]);
        plan.config = Some("lead".into());

        let result = fragment.show_config(&plan);

        assert!(fragment.is_shown());
        assert_eq!(fragment.selected(), Some(9));
        let PlanConfigShowResult::Shown(model) = result else {
            panic!("expected shown model");
        };
        assert_eq!(model.rows, 2);
        assert_eq!(model.columns, 4);
        assert!(matches!(
            model.actions[1],
            PlanConfigTableAction::BuildItemSelection { .. }
        ));
        assert_eq!(
            model.actions.last(),
            Some(&PlanConfigTableAction::ScaleTo {
                x: 1.0,
                y: 1.0,
                duration: 0.07
            })
        );
    }

    #[test]
    fn same_plan_or_missing_block_hides_like_java() {
        let mut fragment = PlanConfigFragment::new();
        let plan = BuildPlanUiRef::new(
            1,
            PlanConfigBlock::new("sorter"),
            vec!["copper".to_string()],
        );
        fragment.show_config(&plan);

        assert_eq!(
            fragment.show_config(&plan),
            PlanConfigShowResult::HiddenBecauseSameOrMissingBlock
        );
        assert!(!fragment.is_shown());
    }

    #[test]
    fn update_positions_table_above_plan_using_block_size_and_tilesize() {
        let mut fragment = PlanConfigFragment::new();
        let mut block = PlanConfigBlock::new("sorter");
        block.size = 3;
        let plan = BuildPlanUiRef::new(5, block, vec!["copper".to_string()]);
        fragment.show_config(&plan);

        let actions = fragment.update(&plan, 8.0, (100.0, 200.0));

        assert_eq!(
            actions.last(),
            Some(&PlanConfigTableAction::SetPositionTop { x: 100.0, y: 187.0 })
        );
    }
}
