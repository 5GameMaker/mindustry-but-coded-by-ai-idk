//! Planet dialog core model mirroring upstream `mindustry.ui.dialogs.PlanetDialog`.
//!
//! This module keeps the campaign-entry decision logic pure. Rendering details
//! (`PlanetRenderer`, 3D ray picking and launch cutscenes) are represented as
//! data/actions so frontends can reproduce the Java flow without embedding engine
//! side effects in the model.

pub const PLANET_DIALOG_INITIAL_TITLE: &str = "";
pub const PLANET_DIALOG_STYLE: &str = "Styles.fullDialog";
pub const PLANET_DIALOG_SECTOR_SHOW_DURATION: f32 = 60.0 * 2.4;
pub const PLANET_DIALOG_DEFAULT_PLANET: &str = "serpulo";
pub const PLANET_DIALOG_BACK_BUTTON_TEXT: &str = "@back";
pub const PLANET_DIALOG_BACK_ICON: &str = "left";
pub const PLANET_DIALOG_TECH_TREE_TEXT: &str = "@techtree";
pub const PLANET_DIALOG_TECH_TREE_ICON: &str = "tree";
pub const PLANET_DIALOG_BOTTOM_BUTTON_SIZE: (f32, f32) = (200.0, 54.0);
pub const PLANET_DIALOG_CAMPAIGN_RULES_TEXT: &str = "@campaign.difficulty";
pub const PLANET_DIALOG_CAMPAIGN_RULES_ICON: &str = "bookSmall";
pub const PLANET_DIALOG_CAMPAIGN_RULES_SIZE: (f32, f32) = (208.0, 40.0);
pub const PLANET_DIALOG_PLANET_BUTTON_STYLE: &str = "Styles.flatTogglet";
pub const PLANET_DIALOG_PLANET_BUTTON_SIZE: (f32, f32) = (200.0, 40.0);
pub const PLANET_DIALOG_SELECTED_BACKGROUND: &str = "Styles.black6";
pub const PLANET_DIALOG_SELECTED_PLAY_HEIGHT: f32 = 54.0;
pub const PLANET_DIALOG_SELECTED_PLAY_MIN_WIDTH: f32 = 170.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetDialogMode {
    Look,
    Select,
    PlanetLaunch,
}

impl Default for PlanetDialogMode {
    fn default() -> Self {
        Self::Look
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetDialog {
    pub mode: PlanetDialogMode,
    pub planet_name: String,
    pub selected_sector_id: Option<i32>,
    pub hovered_sector_id: Option<i32>,
    pub launch_sector_id: Option<i32>,
    pub launch_sector_planet: Option<String>,
    pub launch_candidates: Vec<String>,
    pub launching: bool,
    pub zoom: f32,
    pub state_zoom: f32,
    pub ui_alpha: f32,
    pub search_text: String,
    pub sectors_shown: bool,
    pub preset_show: f32,
    pub showed: bool,
}

impl Default for PlanetDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanetDialog {
    pub fn new() -> Self {
        Self {
            mode: PlanetDialogMode::Look,
            planet_name: PLANET_DIALOG_DEFAULT_PLANET.into(),
            selected_sector_id: None,
            hovered_sector_id: None,
            launch_sector_id: None,
            launch_sector_planet: None,
            launch_candidates: Vec::new(),
            launching: false,
            zoom: 1.0,
            state_zoom: 1.0,
            ui_alpha: 0.0,
            search_text: String::new(),
            sectors_shown: false,
            preset_show: 0.0,
            showed: false,
        }
    }

    pub fn show_plan(&mut self, context: &PlanetDialogContext) -> Vec<PlanetDialogAction> {
        if context.net_client {
            return vec![PlanetDialogAction::ShowInfo {
                text: "@map.multiplayer",
            }];
        }

        if let Some(planet) = &context.rules_sector_planet {
            self.planet_name = planet.clone();
        }

        self.mode = PlanetDialogMode::Look;
        self.selected_sector_id = None;
        self.hovered_sector_id = None;
        self.launching = false;
        self.zoom = 1.0;
        self.state_zoom = 1.0;
        self.ui_alpha = 0.0;
        self.launch_sector_id = if context.game_over {
            None
        } else {
            context.current_sector_id
        };
        self.launch_sector_planet = if context.game_over {
            None
        } else {
            context.current_sector_planet.clone()
        };
        self.preset_show = 0.0;
        self.showed = false;
        self.launch_candidates.clear();

        vec![
            PlanetDialogAction::RebuildButtons,
            PlanetDialogAction::SetMode {
                mode: PlanetDialogMode::Look,
            },
            PlanetDialogAction::ClearOtherCamera,
            PlanetDialogAction::UpdateSelected,
            PlanetDialogAction::ShowDialog,
        ]
    }

    pub fn show_planet_launch(
        &mut self,
        source: &PlanetDialogSector,
        candidates: &[String],
        destination_start_sector: Option<&PlanetDialogSector>,
    ) -> Vec<PlanetDialogAction> {
        self.selected_sector_id = None;
        self.hovered_sector_id = None;
        self.launching = false;
        self.launch_candidates = candidates.to_vec();
        self.launch_sector_id = Some(source.id);
        self.launch_sector_planet = Some(source.planet_name.clone());
        self.zoom = 1.0;
        self.state_zoom = 1.0;
        self.ui_alpha = 0.0;
        self.mode = PlanetDialogMode::PlanetLaunch;

        let mut actions = vec![PlanetDialogAction::SetMode {
            mode: PlanetDialogMode::PlanetLaunch,
        }];

        if self.launch_candidates.len() == 1 {
            self.planet_name = self.launch_candidates[0].clone();
            actions.push(PlanetDialogAction::SetPlanet {
                planet_name: self.planet_name.clone(),
            });
            actions.push(PlanetDialogAction::SetOtherCameraFromPlanet {
                planet_name: source.planet_name.clone(),
            });
            if let Some(sector) = destination_start_sector {
                self.selected_sector_id = Some(sector.id);
                actions.push(PlanetDialogAction::UnlockPreset {
                    sector_id: sector.id,
                });
                actions.push(PlanetDialogAction::SelectSector {
                    sector_id: sector.id,
                });
            }
        }

        actions.extend([
            PlanetDialogAction::UpdateSelected,
            PlanetDialogAction::RebuildExpand,
            PlanetDialogAction::ShowDialog,
        ]);
        actions
    }

    pub fn show_select(&mut self, source: &PlanetDialogSector) -> Vec<PlanetDialogAction> {
        self.selected_sector_id = None;
        self.hovered_sector_id = None;
        self.launching = false;
        self.launch_sector_id = Some(source.id);
        self.launch_sector_planet = Some(source.planet_name.clone());
        self.zoom = 1.0;
        self.state_zoom = 1.0;
        self.ui_alpha = 0.0;
        self.mode = PlanetDialogMode::Select;

        vec![
            PlanetDialogAction::LookAtSector {
                sector_id: source.id,
            },
            PlanetDialogAction::SetMode {
                mode: PlanetDialogMode::Select,
            },
            PlanetDialogAction::ShowDialog,
        ]
    }

    pub fn tap_plan(
        &mut self,
        hovered: Option<&PlanetDialogSector>,
        sectors: &[PlanetDialogSector],
        count: i32,
        button: PlanetDialogPointerButton,
        context: &PlanetDialogContext,
    ) -> Vec<PlanetDialogAction> {
        if context.showing_preset || button != PlanetDialogPointerButton::MouseLeft {
            return Vec::new();
        }

        let mut actions = Vec::new();
        if let Some(sector) = hovered {
            if self.selected_sector_id == Some(sector.id)
                && count == 2
                && can_play_sector(self.mode, sector, self.launch_sector_id, sectors)
            {
                actions.extend(self.play_selected_plan(sector, sectors, context));
            }

            if can_select_sector(
                self.mode,
                sector,
                self.launch_sector_id,
                self.launch_sector_planet.as_deref(),
                context.debug_select,
            ) || context.debug_select
            {
                self.selected_sector_id = Some(sector.id);
                actions.push(PlanetDialogAction::SelectSector {
                    sector_id: sector.id,
                });
                actions.push(PlanetDialogAction::UpdateSelected);
            }
        }
        actions
    }

    pub fn selected_panel_model(
        &self,
        sector: Option<&PlanetDialogSector>,
        sectors: &[PlanetDialogSector],
        context: &PlanetDialogContext,
    ) -> PlanetSelectedPanelModel {
        let Some(sector) = sector else {
            return PlanetSelectedPanelModel {
                visible: false,
                ..PlanetSelectedPanelModel::default()
            };
        };

        let locked = is_locked(sector);
        let no_candidate = has_no_candidate(self.mode, sector, self.launch_sector_id, sectors);
        let show_play = (sector.has_base && self.mode == PlanetDialogMode::Look)
            || can_select_sector(
                self.mode,
                sector,
                self.launch_sector_id,
                self.launch_sector_planet.as_deref(),
                context.debug_select,
            )
            || sector.preset_always_unlocked
            || context.debug_select;

        PlanetSelectedPanelModel {
            visible: true,
            background: PLANET_DIALOG_SELECTED_BACKGROUND,
            title: selected_title(sector, context.debug_select),
            show_rename: sector.preset_name.is_none() || !sector.preset_require_unlock,
            show_threat: !locked && !sector.has_base,
            show_enemy_base: !sector.has_base && sector.has_enemy_base,
            show_stats: sector.has_base,
            play_button: show_play.then(|| PlanetSelectedPlayButton {
                text: selected_play_text(self.mode, sector, locked, no_candidate),
                icon: if locked { "lock" } else { "play" },
                disabled: locked || no_candidate,
                height: PLANET_DIALOG_SELECTED_PLAY_HEIGHT,
                min_width: PLANET_DIALOG_SELECTED_PLAY_MIN_WIDTH,
            }),
        }
    }

    pub fn play_selected_plan(
        &self,
        sector: &PlanetDialogSector,
        sectors: &[PlanetDialogSector],
        context: &PlanetDialogContext,
    ) -> Vec<PlanetDialogAction> {
        if sector.is_being_played {
            return vec![PlanetDialogAction::HideDialog];
        }

        if is_locked(sector) {
            return Vec::new();
        }

        let mut actions = Vec::new();
        if context.has_current_save
            && context.state_is_game
            && self.mode != PlanetDialogMode::Select
        {
            actions.push(PlanetDialogAction::SaveCurrentGame);
        }

        if self.mode == PlanetDialogMode::Look && !sector.has_base {
            if let Some(from) = find_launcher(self.mode, sector, self.launch_sector_id, sectors) {
                actions.push(PlanetDialogAction::ShowLaunchLoadout {
                    from_sector_id: from,
                    to_sector_id: sector.id,
                });
            } else {
                actions.push(PlanetDialogAction::ClearLoadoutInfo);
                actions.push(PlanetDialogAction::PlaySector {
                    from_sector_id: None,
                    to_sector_id: sector.id,
                });
            }
        } else if self.mode == PlanetDialogMode::Select
            || self.mode == PlanetDialogMode::PlanetLaunch
        {
            actions.push(PlanetDialogAction::NotifySectorListener {
                sector_id: sector.id,
            });
            actions.push(PlanetDialogAction::HideDialog);
        } else {
            actions.push(PlanetDialogAction::PlaySector {
                from_sector_id: None,
                to_sector_id: sector.id,
            });
            actions.push(PlanetDialogAction::HideDialog);
        }

        actions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanetDialogContext {
    pub net_client: bool,
    pub game_over: bool,
    pub rules_sector_planet: Option<String>,
    pub rules_sector_id: Option<i32>,
    pub current_sector_planet: Option<String>,
    pub current_sector_id: Option<i32>,
    pub debug_select: bool,
    pub showing_preset: bool,
    pub has_current_save: bool,
    pub state_is_game: bool,
}

impl Default for PlanetDialogContext {
    fn default() -> Self {
        Self {
            net_client: false,
            game_over: false,
            rules_sector_planet: None,
            rules_sector_id: None,
            current_sector_planet: None,
            current_sector_id: None,
            debug_select: false,
            showing_preset: false,
            has_current_save: false,
            state_is_game: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanetDialogPlanet {
    pub name: String,
    pub localized_name: String,
    pub always_unlocked: bool,
    pub is_landable: bool,
    pub has_base: bool,
    pub allow_self_sector_launch: bool,
    pub allow_campaign_rules: bool,
}

impl PlanetDialogPlanet {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            localized_name: name.clone(),
            name,
            always_unlocked: false,
            is_landable: false,
            has_base: false,
            allow_self_sector_launch: false,
            allow_campaign_rules: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetDialogSector {
    pub id: i32,
    pub planet_name: String,
    pub name: String,
    pub has_base: bool,
    pub is_start_sector: bool,
    pub is_being_played: bool,
    pub is_attacked: bool,
    pub is_shielded: bool,
    pub has_enemy_base: bool,
    pub preset_name: Option<String>,
    pub preset_require_unlock: bool,
    pub preset_always_unlocked: bool,
    pub preset_add_starting_items: bool,
    pub preset_locked: bool,
    pub preset_has_tech_node: bool,
    pub preset_parent_available: bool,
    pub generator_present: bool,
    pub allow_landing: bool,
    pub allow_accelerator_landing: bool,
    pub launch_candidate_id: Option<i32>,
}

impl PlanetDialogSector {
    pub fn new(id: i32, planet_name: impl Into<String>) -> Self {
        Self {
            id,
            planet_name: planet_name.into(),
            name: id.to_string(),
            has_base: false,
            is_start_sector: false,
            is_being_played: false,
            is_attacked: false,
            is_shielded: false,
            has_enemy_base: false,
            preset_name: None,
            preset_require_unlock: false,
            preset_always_unlocked: false,
            preset_add_starting_items: false,
            preset_locked: false,
            preset_has_tech_node: false,
            preset_parent_available: true,
            generator_present: true,
            allow_landing: false,
            allow_accelerator_landing: false,
            launch_candidate_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetSelectedPanelModel {
    pub visible: bool,
    pub background: &'static str,
    pub title: String,
    pub show_rename: bool,
    pub show_threat: bool,
    pub show_enemy_base: bool,
    pub show_stats: bool,
    pub play_button: Option<PlanetSelectedPlayButton>,
}

impl Default for PlanetSelectedPanelModel {
    fn default() -> Self {
        Self {
            visible: false,
            background: PLANET_DIALOG_SELECTED_BACKGROUND,
            title: String::new(),
            show_rename: false,
            show_threat: false,
            show_enemy_base: false,
            show_stats: false,
            play_button: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetSelectedPlayButton {
    pub text: &'static str,
    pub icon: &'static str,
    pub disabled: bool,
    pub height: f32,
    pub min_width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetDialogPointerButton {
    MouseLeft,
    MouseRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanetDialogAction {
    ShowInfo {
        text: &'static str,
    },
    ShowDialog,
    HideDialog,
    RebuildButtons,
    RebuildExpand,
    ClearOtherCamera,
    UpdateSelected,
    SetMode {
        mode: PlanetDialogMode,
    },
    SetPlanet {
        planet_name: String,
    },
    SetOtherCameraFromPlanet {
        planet_name: String,
    },
    LookAtSector {
        sector_id: i32,
    },
    SelectSector {
        sector_id: i32,
    },
    UnlockPreset {
        sector_id: i32,
    },
    SaveCurrentGame,
    ClearLoadoutInfo,
    ShowLaunchLoadout {
        from_sector_id: i32,
        to_sector_id: i32,
    },
    PlaySector {
        from_sector_id: Option<i32>,
        to_sector_id: i32,
    },
    NotifySectorListener {
        sector_id: i32,
    },
}

pub fn selectable_planet(
    mode: PlanetDialogMode,
    current_planet_name: &str,
    planet: &PlanetDialogPlanet,
    launch_sector_planet: Option<&str>,
    launch_candidates: &[String],
    debug_select: bool,
) -> bool {
    if mode == PlanetDialogMode::Select {
        return planet.name == current_planet_name;
    }
    if mode == PlanetDialogMode::PlanetLaunch {
        return launch_sector_planet.is_some()
            && (launch_candidates
                .iter()
                .any(|candidate| candidate == &planet.name)
                || (launch_sector_planet == Some(planet.name.as_str())
                    && planet.allow_self_sector_launch));
    }
    (planet.always_unlocked && planet.is_landable) || planet.has_base || debug_select
}

pub fn can_select_sector(
    mode: PlanetDialogMode,
    sector: &PlanetDialogSector,
    launch_sector_id: Option<i32>,
    launch_sector_planet: Option<&str>,
    debug_select: bool,
) -> bool {
    if debug_select {
        return true;
    }
    if mode == PlanetDialogMode::Select {
        return sector.has_base
            && launch_sector_id.is_some()
            && launch_sector_planet == Some(sector.planet_name.as_str());
    }

    if mode == PlanetDialogMode::PlanetLaunch
        && (sector.has_base || sector.preset_add_starting_items)
    {
        return false;
    }

    if !sector.generator_present || sector.is_shielded {
        return false;
    }

    if sector.has_base || sector.is_start_sector {
        return true;
    }

    if sector.preset_require_unlock {
        return !sector.preset_locked || sector.preset_parent_available;
    }

    if mode == PlanetDialogMode::PlanetLaunch {
        sector.allow_accelerator_landing
    } else {
        sector.allow_landing
    }
}

pub fn find_launcher(
    mode: PlanetDialogMode,
    to: &PlanetDialogSector,
    launch_sector_id: Option<i32>,
    sectors: &[PlanetDialogSector],
) -> Option<i32> {
    if mode == PlanetDialogMode::PlanetLaunch || !to.generator_present {
        return launch_sector_id;
    }

    let actual = launch_sector_id.and_then(|id| {
        sectors
            .iter()
            .find(|sector| sector.id == id && sector.planet_name == to.planet_name)
    });

    if let Some(candidate) = to.launch_candidate_id {
        return Some(candidate);
    }

    actual
        .filter(|sector| !sector.is_attacked)
        .map(|sector| sector.id)
}

pub fn has_no_candidate(
    mode: PlanetDialogMode,
    sector: &PlanetDialogSector,
    launch_sector_id: Option<i32>,
    sectors: &[PlanetDialogSector],
) -> bool {
    !sector.is_start_sector
        && !sector.has_base
        && find_launcher(mode, sector, launch_sector_id, sectors).is_none()
}

pub fn is_locked(sector: &PlanetDialogSector) -> bool {
    sector.preset_require_unlock
        && sector.preset_locked
        && sector.preset_has_tech_node
        && !sector.has_base
}

pub fn can_play_sector(
    mode: PlanetDialogMode,
    sector: &PlanetDialogSector,
    launch_sector_id: Option<i32>,
    sectors: &[PlanetDialogSector],
) -> bool {
    !is_locked(sector) && !has_no_candidate(mode, sector, launch_sector_id, sectors)
}

fn selected_title(sector: &PlanetDialogSector, debug_select: bool) -> String {
    if debug_select && sector.preset_name.is_some() {
        format!("[accent]{} [lightgray]({})", sector.name, sector.id)
    } else {
        format!("[accent]{}", sector.name)
    }
}

fn selected_play_text(
    mode: PlanetDialogMode,
    sector: &PlanetDialogSector,
    locked: bool,
    no_candidate: bool,
) -> &'static str {
    if mode == PlanetDialogMode::Select {
        "@sectors.select"
    } else if sector.is_being_played {
        "@sectors.resume"
    } else if sector.has_base {
        "@sectors.go"
    } else if locked {
        "@locked"
    } else if no_candidate {
        "@sectors.nolaunchcandidate"
    } else {
        "@sectors.launch"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(id: i32) -> PlanetDialogSector {
        let mut sector = PlanetDialogSector::new(id, "serpulo");
        sector.name = format!("Sector {id}");
        sector.has_base = true;
        sector
    }

    fn empty(id: i32) -> PlanetDialogSector {
        PlanetDialogSector::new(id, "serpulo")
    }

    #[test]
    fn constructor_matches_java_initial_state_shape() {
        let dialog = PlanetDialog::new();

        assert_eq!(PLANET_DIALOG_INITIAL_TITLE, "");
        assert_eq!(PLANET_DIALOG_STYLE, "Styles.fullDialog");
        assert_eq!(PLANET_DIALOG_SECTOR_SHOW_DURATION, 144.0);
        assert_eq!(dialog.mode, PlanetDialogMode::Look);
        assert_eq!(dialog.planet_name, "serpulo");
        assert_eq!(dialog.zoom, 1.0);
        assert_eq!(dialog.state_zoom, 1.0);
        assert_eq!(dialog.ui_alpha, 0.0);
        assert!(!dialog.launching);
    }

    #[test]
    fn show_plan_rejects_net_client_and_otherwise_resets_look_state() {
        let mut dialog = PlanetDialog::new();
        let blocked = dialog.show_plan(&PlanetDialogContext {
            net_client: true,
            ..PlanetDialogContext::default()
        });
        assert_eq!(
            blocked,
            vec![PlanetDialogAction::ShowInfo {
                text: "@map.multiplayer"
            }]
        );

        dialog.mode = PlanetDialogMode::Select;
        dialog.selected_sector_id = Some(5);
        let actions = dialog.show_plan(&PlanetDialogContext {
            rules_sector_planet: Some("erekir".into()),
            current_sector_planet: Some("erekir".into()),
            current_sector_id: Some(10),
            ..PlanetDialogContext::default()
        });

        assert_eq!(dialog.mode, PlanetDialogMode::Look);
        assert_eq!(dialog.planet_name, "erekir");
        assert_eq!(dialog.selected_sector_id, None);
        assert_eq!(dialog.launch_sector_id, Some(10));
        assert_eq!(
            actions,
            vec![
                PlanetDialogAction::RebuildButtons,
                PlanetDialogAction::SetMode {
                    mode: PlanetDialogMode::Look
                },
                PlanetDialogAction::ClearOtherCamera,
                PlanetDialogAction::UpdateSelected,
                PlanetDialogAction::ShowDialog,
            ]
        );
    }

    #[test]
    fn selectable_planet_matches_mode_branches() {
        let mut planet = PlanetDialogPlanet::new("serpulo");
        planet.always_unlocked = true;
        planet.is_landable = true;
        assert!(selectable_planet(
            PlanetDialogMode::Look,
            "serpulo",
            &planet,
            None,
            &[],
            false,
        ));

        planet.always_unlocked = false;
        planet.has_base = true;
        assert!(selectable_planet(
            PlanetDialogMode::Look,
            "serpulo",
            &planet,
            None,
            &[],
            false,
        ));

        assert!(selectable_planet(
            PlanetDialogMode::Select,
            "serpulo",
            &planet,
            None,
            &[],
            false,
        ));
        assert!(!selectable_planet(
            PlanetDialogMode::Select,
            "erekir",
            &planet,
            None,
            &[],
            false,
        ));

        let candidates = vec!["erekir".into()];
        assert!(!selectable_planet(
            PlanetDialogMode::PlanetLaunch,
            "serpulo",
            &planet,
            Some("serpulo"),
            &candidates,
            false,
        ));
        planet.allow_self_sector_launch = true;
        assert!(selectable_planet(
            PlanetDialogMode::PlanetLaunch,
            "serpulo",
            &planet,
            Some("serpulo"),
            &candidates,
            false,
        ));
    }

    #[test]
    fn can_select_sector_matches_select_planet_launch_and_generator_rules() {
        let launch = base(1);
        let mut sector = base(2);
        assert!(can_select_sector(
            PlanetDialogMode::Select,
            &sector,
            Some(launch.id),
            Some("serpulo"),
            false,
        ));

        sector.planet_name = "erekir".into();
        assert!(!can_select_sector(
            PlanetDialogMode::Select,
            &sector,
            Some(launch.id),
            Some("serpulo"),
            false,
        ));

        let mut target = empty(3);
        target.preset_add_starting_items = true;
        assert!(!can_select_sector(
            PlanetDialogMode::PlanetLaunch,
            &target,
            Some(1),
            Some("serpulo"),
            false,
        ));

        target.preset_add_starting_items = false;
        target.allow_accelerator_landing = true;
        assert!(can_select_sector(
            PlanetDialogMode::PlanetLaunch,
            &target,
            Some(1),
            Some("serpulo"),
            false,
        ));

        target.is_shielded = true;
        assert!(!can_select_sector(
            PlanetDialogMode::Look,
            &target,
            Some(1),
            Some("serpulo"),
            false,
        ));
    }

    #[test]
    fn launcher_candidate_and_no_candidate_follow_java_fallback() {
        let mut launch = base(1);
        launch.is_attacked = false;
        let mut target = empty(2);
        target.allow_landing = true;
        let sectors = vec![launch.clone(), target.clone()];

        assert_eq!(
            find_launcher(PlanetDialogMode::Look, &target, Some(1), &sectors),
            Some(1)
        );
        assert!(!has_no_candidate(
            PlanetDialogMode::Look,
            &target,
            Some(1),
            &sectors
        ));

        launch.is_attacked = true;
        let sectors = vec![launch, target.clone()];
        assert_eq!(
            find_launcher(PlanetDialogMode::Look, &target, Some(1), &sectors),
            None
        );
        assert!(has_no_candidate(
            PlanetDialogMode::Look,
            &target,
            Some(1),
            &sectors
        ));

        target.launch_candidate_id = Some(9);
        assert_eq!(
            find_launcher(PlanetDialogMode::Look, &target, Some(1), &sectors),
            Some(9)
        );
    }

    #[test]
    fn selected_panel_button_text_and_disabled_state_match_update_selected_order() {
        let mut dialog = PlanetDialog::new();
        let sectors = vec![base(1), empty(2)];

        let base_panel = dialog.selected_panel_model(
            Some(&sectors[0]),
            &sectors,
            &PlanetDialogContext::default(),
        );
        assert_eq!(base_panel.play_button.unwrap().text, "@sectors.go");
        assert!(base_panel.show_stats);

        dialog.mode = PlanetDialogMode::Select;
        dialog.launch_sector_id = Some(1);
        dialog.launch_sector_planet = Some("serpulo".into());
        let select_panel = dialog.selected_panel_model(
            Some(&sectors[0]),
            &sectors,
            &PlanetDialogContext::default(),
        );
        assert_eq!(select_panel.play_button.unwrap().text, "@sectors.select");

        let mut locked = empty(3);
        locked.preset_require_unlock = true;
        locked.preset_locked = true;
        locked.preset_has_tech_node = true;
        locked.preset_always_unlocked = true;
        let locked_panel = PlanetDialog::new().selected_panel_model(
            Some(&locked),
            &[locked.clone()],
            &PlanetDialogContext::default(),
        );
        let button = locked_panel.play_button.unwrap();
        assert_eq!(button.text, "@locked");
        assert_eq!(button.icon, "lock");
        assert!(button.disabled);
    }

    #[test]
    fn play_selected_handles_being_played_locked_free_launch_loadout_listener_and_resume() {
        let dialog = PlanetDialog::new();
        let mut playing = base(1);
        playing.is_being_played = true;
        assert_eq!(
            dialog.play_selected_plan(
                &playing,
                &[playing.clone()],
                &PlanetDialogContext::default()
            ),
            vec![PlanetDialogAction::HideDialog]
        );

        let mut locked = empty(2);
        locked.preset_require_unlock = true;
        locked.preset_locked = true;
        locked.preset_has_tech_node = true;
        assert!(dialog
            .play_selected_plan(&locked, &[locked.clone()], &PlanetDialogContext::default())
            .is_empty());

        let free = empty(3);
        assert_eq!(
            dialog.play_selected_plan(&free, &[free.clone()], &PlanetDialogContext::default()),
            vec![
                PlanetDialogAction::ClearLoadoutInfo,
                PlanetDialogAction::PlaySector {
                    from_sector_id: None,
                    to_sector_id: 3,
                }
            ]
        );

        let launch = base(4);
        let mut target = empty(5);
        target.launch_candidate_id = Some(4);
        assert_eq!(
            dialog.play_selected_plan(
                &target,
                &[launch.clone(), target.clone()],
                &PlanetDialogContext::default()
            ),
            vec![PlanetDialogAction::ShowLaunchLoadout {
                from_sector_id: 4,
                to_sector_id: 5,
            }]
        );

        let mut select_dialog = PlanetDialog::new();
        select_dialog.mode = PlanetDialogMode::Select;
        assert_eq!(
            select_dialog.play_selected_plan(
                &launch,
                &[launch.clone()],
                &PlanetDialogContext::default()
            ),
            vec![
                PlanetDialogAction::NotifySectorListener { sector_id: 4 },
                PlanetDialogAction::HideDialog,
            ]
        );

        assert_eq!(
            dialog.play_selected_plan(&launch, &[launch.clone()], &PlanetDialogContext::default()),
            vec![
                PlanetDialogAction::PlaySector {
                    from_sector_id: None,
                    to_sector_id: 4,
                },
                PlanetDialogAction::HideDialog,
            ]
        );
    }

    #[test]
    fn double_click_plays_then_single_click_selects_hovered_sector() {
        let mut dialog = PlanetDialog::new();
        dialog.selected_sector_id = Some(1);
        let sector = base(1);

        let actions = dialog.tap_plan(
            Some(&sector),
            &[sector.clone()],
            2,
            PlanetDialogPointerButton::MouseLeft,
            &PlanetDialogContext::default(),
        );

        assert_eq!(
            actions,
            vec![
                PlanetDialogAction::PlaySector {
                    from_sector_id: None,
                    to_sector_id: 1,
                },
                PlanetDialogAction::HideDialog,
                PlanetDialogAction::SelectSector { sector_id: 1 },
                PlanetDialogAction::UpdateSelected,
            ]
        );
    }

    #[test]
    fn show_planet_launch_auto_selects_single_candidate_start_sector() {
        let source = base(1);
        let mut dest = PlanetDialogSector::new(10, "erekir");
        dest.preset_name = Some("onset".into());
        let mut dialog = PlanetDialog::new();

        let actions = dialog.show_planet_launch(&source, &["erekir".into()], Some(&dest));

        assert_eq!(dialog.mode, PlanetDialogMode::PlanetLaunch);
        assert_eq!(dialog.planet_name, "erekir");
        assert_eq!(dialog.launch_sector_id, Some(1));
        assert_eq!(dialog.selected_sector_id, Some(10));
        assert_eq!(
            actions,
            vec![
                PlanetDialogAction::SetMode {
                    mode: PlanetDialogMode::PlanetLaunch,
                },
                PlanetDialogAction::SetPlanet {
                    planet_name: "erekir".into(),
                },
                PlanetDialogAction::SetOtherCameraFromPlanet {
                    planet_name: "serpulo".into(),
                },
                PlanetDialogAction::UnlockPreset { sector_id: 10 },
                PlanetDialogAction::SelectSector { sector_id: 10 },
                PlanetDialogAction::UpdateSelected,
                PlanetDialogAction::RebuildExpand,
                PlanetDialogAction::ShowDialog,
            ]
        );
    }
}
