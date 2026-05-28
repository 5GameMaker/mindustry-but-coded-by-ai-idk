//! Data-oriented migration of upstream `LoadRenderer`.
//!
//! The Java renderer draws the loading screen directly via `Core.graphics`.
//! This Rust counterpart keeps the same concerns as backend-neutral plans:
//! stage selection, progress bar, prompt text, logo/planet/background layers,
//! plus error and completion overlays.

use super::{
    RenderCommand, RenderPass, RenderPassKind, RenderPoint, RenderProperty, RenderRect,
    RenderTextAlign,
};

const LOAD_PASS_KIND: &str = "load";
const LOAD_GLOW_LAYER: f32 = 1.0;
const LOAD_PLANET_LAYER: f32 = 2.0;
const LOAD_LOGO_LAYER: f32 = 3.0;
const LOAD_PROGRESS_TRACK_LAYER: f32 = 10.0;
const LOAD_PROGRESS_FILL_LAYER: f32 = 11.0;
const LOAD_PROGRESS_TEXT_LAYER: f32 = 12.0;
const LOAD_STAGE_TEXT_LAYER: f32 = 20.0;
const LOAD_PROMPT_TEXT_LAYER: f32 = 21.0;
const LOAD_BANNER_BACKGROUND_LAYER: f32 = 30.0;
const LOAD_BANNER_MESSAGE_LAYER: f32 = 31.0;
const LOAD_BANNER_DETAILS_LAYER: f32 = 32.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadStage {
    Boot,
    LoadingAssets,
    LoadingContent,
    Initializing,
    Running,
    Failed,
    Complete,
}

impl LoadStage {
    pub const fn default_prompt(self) -> &'static str {
        match self {
            Self::Boot => "booting",
            Self::LoadingAssets => "loading assets",
            Self::LoadingContent => "loading content",
            Self::Initializing => "initializing",
            Self::Running => "starting",
            Self::Failed => "loading failed",
            Self::Complete => "ready",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Boot => "boot",
            Self::LoadingAssets => "assets",
            Self::LoadingContent => "content",
            Self::Initializing => "init",
            Self::Running => "run",
            Self::Failed => "error",
            Self::Complete => "done",
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Failed | Self::Complete)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoadRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LoadRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadTheme {
    pub logo_text: String,
    pub planet_name: String,
    pub background_rgba: [f32; 4],
    pub glow_rgba: [f32; 4],
    pub accent_rgba: [f32; 4],
    pub accent_alt_rgba: [f32; 4],
    pub error_rgba: [f32; 4],
    pub success_rgba: [f32; 4],
    pub show_logo: bool,
    pub show_planet: bool,
    pub show_background_grid: bool,
    pub rotation_speed_deg_per_sec: f32,
}

impl Default for LoadTheme {
    fn default() -> Self {
        Self {
            logo_text: "mindustry".to_string(),
            planet_name: "serpulo".to_string(),
            background_rgba: [0.04, 0.04, 0.06, 1.0],
            glow_rgba: [0.20, 0.42, 0.83, 0.18],
            accent_rgba: [0.30, 0.70, 1.00, 1.0],
            accent_alt_rgba: [0.10, 0.18, 0.28, 1.0],
            error_rgba: [0.95, 0.26, 0.22, 1.0],
            success_rgba: [0.32, 0.84, 0.50, 1.0],
            show_logo: true,
            show_planet: true,
            show_background_grid: true,
            rotation_speed_deg_per_sec: 18.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadRendererState {
    pub theme: LoadTheme,
    pub planet_rotation_deg: f32,
}

impl Default for LoadRendererState {
    fn default() -> Self {
        Self {
            theme: LoadTheme::default(),
            planet_rotation_deg: 45.0,
        }
    }
}

impl LoadRendererState {
    pub fn new(theme: LoadTheme) -> Self {
        Self {
            theme,
            planet_rotation_deg: 45.0,
        }
    }

    pub fn build_plan(&mut self, input: LoadFrameInput) -> LoadFramePlan {
        self.planet_rotation_deg = wrap_degrees(
            self.planet_rotation_deg + input.delta.max(0.0) * self.theme.rotation_speed_deg_per_sec,
        );

        let progress = normalize_progress(input.progress, input.stage);
        let width = input.graphics_width.max(1.0);
        let height = input.graphics_height.max(1.0);
        let scale = input.scale.max(0.1);
        let min_side = width.min(height);

        let center_x = width / 2.0;
        let center_y = height / 2.0;
        let glow_radius = min_side * 0.62;
        let planet_radius = (min_side * 0.14).max(18.0 * scale);
        let logo_y = height * 0.24;
        let planet_y = height * 0.40;
        let bar_width = (width * 0.54).clamp(220.0 * scale, 640.0 * scale);
        let bar_height = (18.0 * scale).max(10.0);
        let bar_x = (width - bar_width) / 2.0;
        let bar_y = height * 0.66;
        let prompt_y = (bar_y + bar_height + 26.0 * scale).min(height - 20.0 * scale);
        let status_y = (height - 28.0 * scale).max(prompt_y + 18.0 * scale);
        let panel_height = (54.0 * scale).max(32.0);
        let panel_width = (width * 0.62).clamp(240.0 * scale, 760.0 * scale);
        let panel_x = (width - panel_width) / 2.0;
        let panel_y = (height * 0.78).min(height - panel_height - 10.0 * scale);

        let prompt_text = input
            .prompt
            .clone()
            .unwrap_or_else(|| input.stage.default_prompt().to_string());
        let completion_text = input
            .completion
            .clone()
            .unwrap_or_else(|| "loading complete".to_string());
        let error_text = input
            .error
            .clone()
            .unwrap_or_else(|| input.stage.default_prompt().to_string());

        let status_text = if input.stage == LoadStage::Complete {
            prompt_or(&completion_text, "ready")
        } else if input.stage == LoadStage::Failed {
            error_text.clone()
        } else {
            format!("{} {:>3.0}%", input.stage.label(), progress * 100.0)
        };

        let mut commands = vec![
            LoadRenderCommand::Clear {
                color: self.theme.background_rgba,
            },
            LoadRenderCommand::BackgroundGlow {
                center: (center_x, center_y),
                radius: glow_radius,
                color: self.theme.glow_rgba,
            },
        ];

        if self.theme.show_background_grid {
            commands.push(LoadRenderCommand::BackgroundGrid {
                spacing: (58.0 * scale).max(24.0),
                stroke: (4.0 * scale).max(1.0),
                color: self.theme.accent_alt_rgba,
            });
        }

        if self.theme.show_planet {
            commands.push(LoadRenderCommand::Planet {
                name: self.theme.planet_name.clone(),
                center: (center_x, planet_y),
                radius: planet_radius,
                rotation_deg: self.planet_rotation_deg,
                color: self.theme.accent_rgba,
            });
        }

        if self.theme.show_logo {
            commands.push(LoadRenderCommand::Logo {
                text: self.theme.logo_text.clone(),
                center: (center_x, logo_y),
                scale: (1.0 + scale * 0.12).max(1.0),
                color: self.theme.accent_rgba,
            });
        }

        commands.push(LoadRenderCommand::ProgressBar {
            rect: LoadRect::new(bar_x, bar_y, bar_width, bar_height),
            progress,
            label: input.stage.label().to_string(),
            fill_color: self.theme.accent_rgba,
            track_color: self.theme.accent_alt_rgba,
        });

        commands.push(LoadRenderCommand::StageLabel {
            text: status_text.clone(),
            center: (center_x, status_y),
            color: stage_color(input.stage, &self.theme),
        });

        commands.push(LoadRenderCommand::PromptText {
            text: prompt_text.clone(),
            center: (center_x, prompt_y),
            color: self.theme.accent_rgba,
        });

        if input.stage == LoadStage::Failed || input.error.is_some() {
            commands.push(LoadRenderCommand::ErrorBanner {
                message: error_text.clone(),
                details: Some("retry or inspect the failure source".to_string()),
                rect: LoadRect::new(panel_x, panel_y, panel_width, panel_height),
                color: self.theme.error_rgba,
            });
        }

        if input.stage == LoadStage::Complete || input.completion.is_some() {
            commands.push(LoadRenderCommand::CompletionBanner {
                message: completion_text.clone(),
                rect: LoadRect::new(panel_x, panel_y, panel_width, panel_height),
                color: self.theme.success_rgba,
            });
        }

        LoadFramePlan {
            stage: input.stage,
            progress,
            stage_text: status_text,
            prompt_text,
            commands,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadFrameInput {
    pub graphics_width: f32,
    pub graphics_height: f32,
    pub scale: f32,
    pub delta: f32,
    pub stage: LoadStage,
    pub progress: f32,
    pub prompt: Option<String>,
    pub error: Option<String>,
    pub completion: Option<String>,
}

impl LoadFrameInput {
    pub fn new(
        graphics_width: f32,
        graphics_height: f32,
        scale: f32,
        delta: f32,
        stage: LoadStage,
        progress: f32,
    ) -> Self {
        Self {
            graphics_width,
            graphics_height,
            scale,
            delta,
            stage,
            progress,
            prompt: None,
            error: None,
            completion: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadFramePlan {
    pub stage: LoadStage,
    pub progress: f32,
    pub stage_text: String,
    pub prompt_text: String,
    pub commands: Vec<LoadRenderCommand>,
}

impl LoadFramePlan {
    pub fn to_render_pass(&self) -> Option<RenderPass> {
        self.clone().into_render_pass()
    }

    pub fn into_render_pass(self) -> Option<RenderPass> {
        let commands = self
            .commands
            .into_iter()
            .flat_map(LoadRenderCommand::into_render_commands)
            .collect::<Vec<_>>();

        if commands.is_empty() {
            return None;
        }

        let mut pass = RenderPass::new(RenderPassKind::Custom(LOAD_PASS_KIND.to_string()));
        pass.extend(commands);
        Some(pass)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadRenderCommand {
    Clear {
        color: [f32; 4],
    },
    BackgroundGlow {
        center: (f32, f32),
        radius: f32,
        color: [f32; 4],
    },
    BackgroundGrid {
        spacing: f32,
        stroke: f32,
        color: [f32; 4],
    },
    Logo {
        text: String,
        center: (f32, f32),
        scale: f32,
        color: [f32; 4],
    },
    Planet {
        name: String,
        center: (f32, f32),
        radius: f32,
        rotation_deg: f32,
        color: [f32; 4],
    },
    ProgressBar {
        rect: LoadRect,
        progress: f32,
        label: String,
        fill_color: [f32; 4],
        track_color: [f32; 4],
    },
    StageLabel {
        text: String,
        center: (f32, f32),
        color: [f32; 4],
    },
    PromptText {
        text: String,
        center: (f32, f32),
        color: [f32; 4],
    },
    ErrorBanner {
        message: String,
        details: Option<String>,
        rect: LoadRect,
        color: [f32; 4],
    },
    CompletionBanner {
        message: String,
        rect: LoadRect,
        color: [f32; 4],
    },
}

impl LoadRenderCommand {
    pub fn to_render_commands(&self) -> Vec<RenderCommand> {
        self.clone().into_render_commands()
    }

    pub fn into_render_commands(self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        match self {
            Self::Clear { color } => {
                commands.push(RenderCommand::clear(color));
            }
            Self::BackgroundGlow {
                center,
                radius,
                color,
            } => {
                commands.push(RenderCommand::draw_circle(
                    RenderPoint::new(center.0, center.1),
                    radius,
                    color,
                    true,
                    LOAD_GLOW_LAYER,
                ));
            }
            Self::BackgroundGrid {
                spacing,
                stroke,
                color,
            } => {
                commands.push(RenderCommand::custom(
                    "load-background-grid",
                    vec![
                        RenderProperty::new("spacing", spacing.to_string()),
                        RenderProperty::new("stroke", stroke.to_string()),
                        RenderProperty::new("r", color[0].to_string()),
                        RenderProperty::new("g", color[1].to_string()),
                        RenderProperty::new("b", color[2].to_string()),
                        RenderProperty::new("a", color[3].to_string()),
                    ],
                ));
            }
            Self::Logo {
                text,
                center,
                scale,
                color,
            } => {
                commands.push(RenderCommand::draw_text(
                    text,
                    RenderPoint::new(center.0, center.1),
                    color,
                    (scale * 36.0).max(1.0),
                    0.0,
                    RenderTextAlign::Center,
                    LOAD_LOGO_LAYER,
                ));
            }
            Self::Planet {
                name,
                center,
                radius,
                rotation_deg,
                color,
            } => {
                commands.push(RenderCommand::draw_sprite(
                    name,
                    RenderRect::from_center(
                        RenderPoint::new(center.0, center.1),
                        radius * 2.0,
                        radius * 2.0,
                    ),
                    color,
                    rotation_deg,
                    LOAD_PLANET_LAYER,
                ));
            }
            Self::ProgressBar {
                rect,
                progress,
                label,
                fill_color,
                track_color,
            } => {
                let rect = RenderRect::new(rect.x, rect.y, rect.width, rect.height);
                let progress = clamp_progress(progress);
                let filled_width = rect.width * progress;
                let text_color = contrasting_color(track_color);

                commands.push(RenderCommand::fill_rect(
                    rect,
                    track_color,
                    LOAD_PROGRESS_TRACK_LAYER,
                ));

                if filled_width > 0.0 {
                    commands.push(RenderCommand::fill_rect(
                        RenderRect::new(rect.x, rect.y, filled_width, rect.height),
                        fill_color,
                        LOAD_PROGRESS_FILL_LAYER,
                    ));
                }

                commands.push(RenderCommand::draw_text(
                    label,
                    rect.center(),
                    text_color,
                    (rect.height * 0.78).max(1.0),
                    0.0,
                    RenderTextAlign::Center,
                    LOAD_PROGRESS_TEXT_LAYER,
                ));
            }
            Self::StageLabel {
                text,
                center,
                color,
            } => {
                commands.push(RenderCommand::draw_text(
                    text,
                    RenderPoint::new(center.0, center.1),
                    color,
                    18.0,
                    0.0,
                    RenderTextAlign::Center,
                    LOAD_STAGE_TEXT_LAYER,
                ));
            }
            Self::PromptText {
                text,
                center,
                color,
            } => {
                commands.push(RenderCommand::draw_text(
                    text,
                    RenderPoint::new(center.0, center.1),
                    color,
                    16.0,
                    0.0,
                    RenderTextAlign::Center,
                    LOAD_PROMPT_TEXT_LAYER,
                ));
            }
            Self::ErrorBanner {
                message,
                details,
                rect,
                color,
            } => {
                let banner_rect = RenderRect::new(rect.x, rect.y, rect.width, rect.height);
                let text_color = contrasting_color(color);
                let message_center = RenderPoint::new(
                    banner_rect.x + banner_rect.width / 2.0,
                    banner_rect.y + banner_rect.height * 0.40,
                );
                let details_center = RenderPoint::new(
                    banner_rect.x + banner_rect.width / 2.0,
                    banner_rect.y + banner_rect.height * 0.70,
                );

                commands.push(RenderCommand::fill_rect(
                    banner_rect,
                    color,
                    LOAD_BANNER_BACKGROUND_LAYER,
                ));
                commands.push(RenderCommand::draw_text(
                    message,
                    message_center,
                    text_color,
                    (banner_rect.height * 0.34).max(1.0),
                    0.0,
                    RenderTextAlign::Center,
                    LOAD_BANNER_MESSAGE_LAYER,
                ));

                if let Some(details) = details {
                    commands.push(RenderCommand::draw_text(
                        details,
                        details_center,
                        text_color,
                        (banner_rect.height * 0.22).max(1.0),
                        0.0,
                        RenderTextAlign::Center,
                        LOAD_BANNER_DETAILS_LAYER,
                    ));
                }
            }
            Self::CompletionBanner {
                message,
                rect,
                color,
            } => {
                let banner_rect = RenderRect::new(rect.x, rect.y, rect.width, rect.height);
                let text_color = contrasting_color(color);

                commands.push(RenderCommand::fill_rect(
                    banner_rect,
                    color,
                    LOAD_BANNER_BACKGROUND_LAYER,
                ));
                commands.push(RenderCommand::draw_text(
                    message,
                    banner_rect.center(),
                    text_color,
                    (banner_rect.height * 0.34).max(1.0),
                    0.0,
                    RenderTextAlign::Center,
                    LOAD_BANNER_MESSAGE_LAYER,
                ));
            }
        }

        commands
    }
}

fn stage_color(stage: LoadStage, theme: &LoadTheme) -> [f32; 4] {
    match stage {
        LoadStage::Failed => theme.error_rgba,
        LoadStage::Complete => theme.success_rgba,
        _ => theme.accent_rgba,
    }
}

fn normalize_progress(progress: f32, stage: LoadStage) -> f32 {
    if stage == LoadStage::Complete {
        return 1.0;
    }

    if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn wrap_degrees(value: f32) -> f32 {
    let wrapped = value % 360.0;
    if wrapped < 0.0 {
        wrapped + 360.0
    } else {
        wrapped
    }
}

fn prompt_or(value: &str, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn clamp_progress(progress: f32) -> f32 {
    if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn contrasting_color(color: [f32; 4]) -> [f32; 4] {
    let luminance = color[0] * 0.299 + color[1] * 0.587 + color[2] * 0.114;
    if luminance > 0.5 {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        [1.0, 1.0, 1.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_stage_prompts_match_expected_labels() {
        assert_eq!(LoadStage::Boot.default_prompt(), "booting");
        assert_eq!(LoadStage::LoadingAssets.default_prompt(), "loading assets");
        assert_eq!(LoadStage::Failed.default_prompt(), "loading failed");
        assert!(LoadStage::Complete.is_terminal());
    }

    #[test]
    fn loading_plan_includes_background_logo_planet_progress_and_prompt() {
        let mut state = LoadRendererState::default();
        let plan = state.build_plan(LoadFrameInput {
            graphics_width: 1920.0,
            graphics_height: 1080.0,
            scale: 2.0,
            delta: 0.5,
            stage: LoadStage::LoadingContent,
            progress: 0.42,
            prompt: None,
            error: None,
            completion: None,
        });

        assert_eq!(plan.stage, LoadStage::LoadingContent);
        assert_eq!(plan.progress, 0.42);
        assert_eq!(plan.stage_text, "content  42%");
        assert_eq!(plan.prompt_text, "loading content");
        assert_eq!(plan.commands.len(), 8);

        assert!(matches!(plan.commands[0], LoadRenderCommand::Clear { .. }));
        assert!(matches!(
            plan.commands[1],
            LoadRenderCommand::BackgroundGlow { .. }
        ));
        assert!(matches!(
            plan.commands[2],
            LoadRenderCommand::BackgroundGrid { .. }
        ));
        assert!(matches!(plan.commands[3], LoadRenderCommand::Planet { .. }));
        assert!(matches!(plan.commands[4], LoadRenderCommand::Logo { .. }));
        assert!(matches!(
            plan.commands[5],
            LoadRenderCommand::ProgressBar { .. }
        ));
        assert!(matches!(
            plan.commands[6],
            LoadRenderCommand::StageLabel { .. }
        ));
        assert!(matches!(
            plan.commands[7],
            LoadRenderCommand::PromptText { .. }
        ));
        assert_eq!(state.planet_rotation_deg, 54.0);
    }

    #[test]
    fn failed_state_adds_error_banner_and_clamps_progress() {
        let mut state = LoadRendererState::default();
        let mut input = LoadFrameInput::new(1280.0, 720.0, 1.5, 1.0, LoadStage::Failed, 2.5);
        input.error = Some("asset loader crashed".to_string());
        let plan = state.build_plan(input);

        assert_eq!(plan.progress, 1.0);
        assert_eq!(plan.stage_text, "asset loader crashed");
        assert!(matches!(
            plan.commands.last().unwrap(),
            LoadRenderCommand::ErrorBanner { .. }
        ));

        let error = match plan.commands.last().unwrap() {
            LoadRenderCommand::ErrorBanner {
                message, details, ..
            } => (message.clone(), details.clone()),
            _ => unreachable!(),
        };
        assert_eq!(error.0, "asset loader crashed");
        assert_eq!(
            error.1,
            Some("retry or inspect the failure source".to_string())
        );
    }

    #[test]
    fn completion_state_forces_full_progress_and_completion_banner() {
        let mut state = LoadRendererState::default();
        let mut input = LoadFrameInput::new(960.0, 540.0, 1.0, 0.25, LoadStage::Complete, 0.1);
        input.completion = Some("all assets ready".to_string());
        let plan = state.build_plan(input);

        assert_eq!(plan.progress, 1.0);
        assert_eq!(plan.stage_text, "all assets ready");
        assert_eq!(plan.prompt_text, "ready");
        assert!(matches!(
            plan.commands.last().unwrap(),
            LoadRenderCommand::CompletionBanner { .. }
        ));
    }

    #[test]
    fn prompt_override_is_preserved_even_when_stage_changes() {
        let mut state = LoadRendererState::new(LoadTheme {
            show_background_grid: false,
            show_planet: false,
            show_logo: false,
            ..LoadTheme::default()
        });
        let mut input = LoadFrameInput::new(800.0, 600.0, 1.0, 0.0, LoadStage::Initializing, 0.33);
        input.prompt = Some("warming shaders".to_string());
        let plan = state.build_plan(input);

        assert_eq!(plan.prompt_text, "warming shaders");
        assert_eq!(plan.commands.len(), 5);
        assert!(matches!(plan.commands[0], LoadRenderCommand::Clear { .. }));
        assert!(matches!(
            plan.commands[1],
            LoadRenderCommand::BackgroundGlow { .. }
        ));
        assert!(matches!(
            plan.commands[2],
            LoadRenderCommand::ProgressBar { .. }
        ));
        assert!(matches!(
            plan.commands[3],
            LoadRenderCommand::StageLabel { .. }
        ));
        assert!(matches!(
            plan.commands[4],
            LoadRenderCommand::PromptText { .. }
        ));
    }

    #[test]
    fn empty_load_frame_plan_does_not_create_render_pass() {
        let plan = LoadFramePlan {
            stage: LoadStage::Boot,
            progress: 0.0,
            stage_text: String::new(),
            prompt_text: String::new(),
            commands: Vec::new(),
        };

        assert_eq!(plan.to_render_pass(), None);
        assert_eq!(plan.into_render_pass(), None);
    }

    #[test]
    fn load_frame_plan_maps_commands_into_load_render_pass_in_order() {
        let plan = LoadFramePlan {
            stage: LoadStage::Failed,
            progress: 0.25,
            stage_text: "error  25%".to_string(),
            prompt_text: "loading assets".to_string(),
            commands: vec![
                LoadRenderCommand::Clear {
                    color: [0.04, 0.04, 0.06, 1.0],
                },
                LoadRenderCommand::BackgroundGlow {
                    center: (10.0, 20.0),
                    radius: 30.0,
                    color: [0.20, 0.42, 0.83, 0.18],
                },
                LoadRenderCommand::BackgroundGrid {
                    spacing: 48.0,
                    stroke: 2.0,
                    color: [0.10, 0.18, 0.28, 1.0],
                },
                LoadRenderCommand::Planet {
                    name: "serpulo".to_string(),
                    center: (100.0, 120.0),
                    radius: 24.0,
                    rotation_deg: 45.0,
                    color: [0.30, 0.70, 1.00, 1.0],
                },
                LoadRenderCommand::Logo {
                    text: "mindustry".to_string(),
                    center: (100.0, 40.0),
                    scale: 1.5,
                    color: [0.30, 0.70, 1.00, 1.0],
                },
                LoadRenderCommand::ProgressBar {
                    rect: LoadRect::new(20.0, 140.0, 160.0, 18.0),
                    progress: 0.25,
                    label: "assets".to_string(),
                    fill_color: [0.30, 0.70, 1.00, 1.0],
                    track_color: [0.10, 0.18, 0.28, 1.0],
                },
                LoadRenderCommand::StageLabel {
                    text: "error  25%".to_string(),
                    center: (100.0, 180.0),
                    color: [0.95, 0.26, 0.22, 1.0],
                },
                LoadRenderCommand::PromptText {
                    text: "loading assets".to_string(),
                    center: (100.0, 210.0),
                    color: [0.30, 0.70, 1.00, 1.0],
                },
                LoadRenderCommand::ErrorBanner {
                    message: "asset loader crashed".to_string(),
                    details: Some("retry or inspect the failure source".to_string()),
                    rect: LoadRect::new(16.0, 220.0, 168.0, 54.0),
                    color: [0.95, 0.26, 0.22, 1.0],
                },
                LoadRenderCommand::CompletionBanner {
                    message: "all assets ready".to_string(),
                    rect: LoadRect::new(16.0, 220.0, 168.0, 54.0),
                    color: [0.32, 0.84, 0.50, 1.0],
                },
            ],
        };

        let borrowed = plan.to_render_pass().expect("plan should produce a pass");
        let owned = plan.into_render_pass().expect("plan should produce a pass");

        assert_eq!(borrowed, owned);
        assert_eq!(owned.kind, RenderPassKind::Custom("load".to_string()));
        assert_eq!(
            owned.order,
            RenderPassKind::Custom("load".to_string()).default_order()
        );
        assert_eq!(owned.commands.len(), 15);

        assert!(matches!(owned.commands[0], RenderCommand::Clear { .. }));
        assert!(matches!(
            owned.commands[1],
            RenderCommand::DrawCircle { .. }
        ));
        assert!(matches!(
            owned.commands[2],
            RenderCommand::Custom { ref name, .. } if name == "load-background-grid"
        ));
        assert!(matches!(
            owned.commands[3],
            RenderCommand::DrawSprite { .. }
        ));
        assert!(matches!(owned.commands[4], RenderCommand::DrawText { .. }));
        assert!(matches!(owned.commands[5], RenderCommand::FillRect { .. }));
        assert!(matches!(owned.commands[6], RenderCommand::FillRect { .. }));
        assert!(matches!(owned.commands[7], RenderCommand::DrawText { .. }));
        assert!(matches!(owned.commands[8], RenderCommand::DrawText { .. }));
        assert!(matches!(owned.commands[9], RenderCommand::DrawText { .. }));
        assert!(matches!(owned.commands[10], RenderCommand::FillRect { .. }));
        assert!(matches!(owned.commands[11], RenderCommand::DrawText { .. }));
        assert!(matches!(owned.commands[12], RenderCommand::DrawText { .. }));
        assert!(matches!(owned.commands[13], RenderCommand::FillRect { .. }));
        assert!(matches!(owned.commands[14], RenderCommand::DrawText { .. }));

        match &owned.commands[7] {
            RenderCommand::DrawText { text, .. } => assert_eq!(text, "assets"),
            other => panic!("unexpected progress label command: {other:?}"),
        }

        match &owned.commands[8] {
            RenderCommand::DrawText { text, .. } => assert_eq!(text, "error  25%"),
            other => panic!("unexpected stage label command: {other:?}"),
        }

        match &owned.commands[9] {
            RenderCommand::DrawText { text, .. } => assert_eq!(text, "loading assets"),
            other => panic!("unexpected prompt command: {other:?}"),
        }

        match &owned.commands[11] {
            RenderCommand::DrawText { text, .. } => {
                assert_eq!(text, "asset loader crashed")
            }
            other => panic!("unexpected banner message command: {other:?}"),
        }

        match &owned.commands[12] {
            RenderCommand::DrawText { text, .. } => {
                assert_eq!(text, "retry or inspect the failure source")
            }
            other => panic!("unexpected banner details command: {other:?}"),
        }

        match &owned.commands[13] {
            RenderCommand::FillRect { color, .. } => {
                assert_eq!(*color, [0.32, 0.84, 0.50, 1.0]);
            }
            other => panic!("unexpected completion background command: {other:?}"),
        }

        match &owned.commands[14] {
            RenderCommand::DrawText { text, .. } => {
                assert_eq!(text, "all assets ready")
            }
            other => panic!("unexpected completion message command: {other:?}"),
        }
    }
}
