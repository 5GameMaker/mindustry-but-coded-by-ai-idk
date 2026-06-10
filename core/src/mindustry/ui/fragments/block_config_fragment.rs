//! Configuration table state mirror of upstream `mindustry.ui.fragments.BlockConfigFragment`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigBuildingRef {
    pub id: i32,
    pub block_name: String,
    pub config_tapped: bool,
    pub should_hide_configure: bool,
    pub is_air: bool,
    pub valid: bool,
}

impl ConfigBuildingRef {
    pub fn new(id: i32, block_name: impl Into<String>) -> Self {
        Self {
            id,
            block_name: block_name.into(),
            config_tapped: true,
            should_hide_configure: false,
            is_air: false,
            valid: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockConfigTableAction {
    ScaleTo { x: f32, y: f32, duration: f32 },
    Visible(bool),
    Clear,
    Pack,
    SetTransform(bool),
    SetOriginCenter,
    UpdateTableAlign(i32),
    BuildConfiguration(i32),
    ConfigureClosed(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockConfigShowResult {
    pub shown: bool,
    pub actions: Vec<BlockConfigTableAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockConfigFragment {
    table_visible: bool,
    selected: Option<i32>,
}

impl BlockConfigFragment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&mut self) {
        self.table_visible = false;
    }

    pub fn force_hide(&mut self) {
        self.table_visible = false;
        self.selected = None;
    }

    pub fn is_shown(&self) -> bool {
        self.table_visible && self.selected.is_some()
    }

    pub fn selected(&self) -> Option<i32> {
        self.selected
    }

    pub fn show_config(&mut self, tile: &ConfigBuildingRef) -> BlockConfigShowResult {
        let mut actions = Vec::new();
        if let Some(selected) = self.selected {
            actions.push(BlockConfigTableAction::ConfigureClosed(selected));
        }

        if tile.config_tapped {
            self.selected = Some(tile.id);
            self.table_visible = true;
            actions.extend([
                BlockConfigTableAction::Clear,
                BlockConfigTableAction::BuildConfiguration(tile.id),
                BlockConfigTableAction::Pack,
                BlockConfigTableAction::SetTransform(true),
                BlockConfigTableAction::ScaleTo {
                    x: 0.0,
                    y: 1.0,
                    duration: 0.0,
                },
                BlockConfigTableAction::Visible(true),
                BlockConfigTableAction::ScaleTo {
                    x: 1.0,
                    y: 1.0,
                    duration: 0.07,
                },
            ]);
        }

        BlockConfigShowResult {
            shown: tile.config_tapped,
            actions,
        }
    }

    pub fn update(&mut self, selected: Option<&ConfigBuildingRef>) -> Vec<BlockConfigTableAction> {
        let mut actions = Vec::new();
        if let Some(tile) = selected {
            if self.selected == Some(tile.id) && tile.should_hide_configure {
                actions.extend(self.hide_config());
                return actions;
            }
        }

        actions.push(BlockConfigTableAction::SetOriginCenter);
        if let Some(tile) = selected {
            if self.selected.is_none() || tile.is_air || !tile.valid {
                actions.extend(self.hide_config());
            } else {
                actions.push(BlockConfigTableAction::UpdateTableAlign(tile.id));
            }
        } else {
            actions.extend(self.hide_config());
        }
        actions
    }

    pub fn has_config_mouse(&self, hover_is_table_or_descendant: bool) -> bool {
        hover_is_table_or_descendant
    }

    pub fn hide_config(&mut self) -> Vec<BlockConfigTableAction> {
        let mut actions = Vec::new();
        if let Some(selected) = self.selected {
            actions.push(BlockConfigTableAction::ConfigureClosed(selected));
        }
        self.selected = None;
        self.table_visible = false;
        actions.extend([
            BlockConfigTableAction::ScaleTo {
                x: 0.0,
                y: 1.0,
                duration: 0.06,
            },
            BlockConfigTableAction::Visible(false),
        ]);
        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_config_opens_table_and_runs_java_action_sequence() {
        let mut fragment = BlockConfigFragment::new();
        let tile = ConfigBuildingRef::new(7, "router");

        let result = fragment.show_config(&tile);

        assert!(result.shown);
        assert!(fragment.is_shown());
        assert_eq!(fragment.selected(), Some(7));
        assert_eq!(result.actions[0], BlockConfigTableAction::Clear);
        assert!(result
            .actions
            .contains(&BlockConfigTableAction::BuildConfiguration(7)));
        assert_eq!(
            result.actions.last(),
            Some(&BlockConfigTableAction::ScaleTo {
                x: 1.0,
                y: 1.0,
                duration: 0.07
            })
        );
    }

    #[test]
    fn show_config_closes_previous_selected_building_like_java() {
        let mut fragment = BlockConfigFragment::new();
        fragment.show_config(&ConfigBuildingRef::new(1, "router"));
        let result = fragment.show_config(&ConfigBuildingRef::new(2, "sorter"));

        assert_eq!(
            result.actions.first(),
            Some(&BlockConfigTableAction::ConfigureClosed(1))
        );
        assert_eq!(fragment.selected(), Some(2));
    }

    #[test]
    fn update_hides_when_selected_requests_hide_or_invalid() {
        let mut fragment = BlockConfigFragment::new();
        let mut tile = ConfigBuildingRef::new(3, "switch");
        fragment.show_config(&tile);

        tile.should_hide_configure = true;
        let actions = fragment.update(Some(&tile));

        assert!(!fragment.is_shown());
        assert!(actions.contains(&BlockConfigTableAction::ConfigureClosed(3)));
        assert!(actions.contains(&BlockConfigTableAction::Visible(false)));
    }
}
