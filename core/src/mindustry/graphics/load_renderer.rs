//! Data-oriented migration of upstream `LoadRenderer`.
//!
//! The Java renderer draws the loading screen directly via `Core.graphics`.
//! This Rust counterpart keeps the same concerns as backend-neutral plans:
//! stage selection, progress bar, prompt text, logo/planet/background layers,
//! plus error and completion overlays.

use super::{RenderCommand, RenderPass, RenderPassKind, RenderPoint, RenderRect, RenderTextAlign};
use crate::mindustry::ui::{WarningBar, WarningBarDrawCommand, WarningBarLayout};

const LOAD_PASS_KIND: &str = "load";
const LOAD_GLOW_LAYER: f32 = 1.0;
const LOAD_GRID_LAYER: f32 = 1.5;
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
const LOAD_FRAGMENT_OVERLAY_LAYER: f32 = 4.0;
const LOAD_FRAGMENT_WARNING_LAYER: f32 = 13.0;
const LOAD_FRAGMENT_LABEL_LAYER: f32 = 14.0;
const LOAD_FRAGMENT_BUTTON_LAYER: f32 = 15.0;
const LOAD_FRAGMENT_EDGE_LAYER: f32 = 15.5;
const LOAD_FRAGMENT_TOP_SPACER: f32 = 133.0;
const LOAD_FRAGMENT_WARNING_HEIGHT: f32 = 24.0;
const LOAD_FRAGMENT_LABEL_GAP: f32 = 10.0;
const LOAD_FRAGMENT_LABEL_HEIGHT: f32 = 32.0;
const LOAD_FRAGMENT_BAR_TOP_PAD: f32 = 6.0;
const LOAD_FRAGMENT_CANCEL_WIDTH: f32 = 250.0;
const LOAD_FRAGMENT_CANCEL_HEIGHT: f32 = 70.0;
const LOAD_FRAGMENT_CANCEL_PAD: f32 = 20.0;

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
        let fragment_layout = loading_fragment_layout(width, height, scale, input.cancelable);
        let bar_width = fragment_layout.progress_bar.width;
        let bar_height = (18.0 * scale).max(10.0);
        let bar_x = fragment_layout.progress_bar.x;
        let bar_y = fragment_layout.progress_bar.y;
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
                bounds: LoadRect::new(0.0, 0.0, input.graphics_width, input.graphics_height),
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

        commands.push(LoadRenderCommand::LoadingFragment {
            overlay: LoadRect::new(0.0, 0.0, width, height),
            top_warning: fragment_layout.top_warning,
            bottom_warning: fragment_layout.bottom_warning,
            label: "@loading".to_string(),
            label_center: (
                center_x,
                fragment_layout.label.y + fragment_layout.label.height * 0.5,
            ),
            cancel_button: fragment_layout.cancel_button,
            overlay_color: [0.0, 0.0, 0.0, 0.80],
        });

        if input.progress_visible {
            commands.push(LoadRenderCommand::ProgressBar {
                rect: LoadRect::new(bar_x, bar_y, bar_width, bar_height),
                progress,
                label: input.stage.label().to_string(),
                fill_color: self.theme.accent_rgba,
                track_color: self.theme.accent_alt_rgba,
            });
        }

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
    pub progress_visible: bool,
    pub prompt: Option<String>,
    pub error: Option<String>,
    pub completion: Option<String>,
    pub cancelable: bool,
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
            progress_visible: true,
            prompt: None,
            error: None,
            completion: None,
            cancelable: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadingFragmentState {
    pub visible: bool,
    pub stage: LoadStage,
    pub progress: f32,
    pub progress_visible: bool,
    pub text: String,
    pub error: Option<String>,
    pub completion: Option<String>,
    pub cancelable: bool,
}

impl Default for LoadingFragmentState {
    fn default() -> Self {
        Self {
            visible: false,
            stage: LoadStage::Boot,
            progress: 0.0,
            progress_visible: false,
            text: "@loading".to_string(),
            error: None,
            completion: None,
            cancelable: false,
        }
    }
}

impl LoadingFragmentState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self) {
        self.show_text("@loading");
    }

    pub fn show_text(&mut self, text: impl Into<String>) {
        self.visible = true;
        self.stage = LoadStage::Boot;
        self.progress_visible = false;
        self.cancelable = false;
        self.error = None;
        self.completion = None;
        self.text = text.into();
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.cancelable = false;
    }

    pub fn set_stage(&mut self, stage: LoadStage) {
        self.stage = stage;
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress;
        self.progress_visible = true;
        if self.stage == LoadStage::Boot {
            self.stage = LoadStage::LoadingAssets;
        }
    }

    pub fn snap_progress(&mut self) {
        self.progress_visible = true;
    }

    pub fn set_button(&mut self) {
        self.cancelable = true;
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn fail(&mut self, message: impl Into<String>) {
        self.visible = true;
        self.stage = LoadStage::Failed;
        self.error = Some(message.into());
        self.completion = None;
        self.progress_visible = true;
    }

    pub fn complete(&mut self, message: impl Into<String>) {
        self.visible = true;
        self.stage = LoadStage::Complete;
        self.progress = 1.0;
        self.error = None;
        self.completion = Some(message.into());
        self.progress_visible = true;
        self.cancelable = false;
    }

    pub fn frame_input(
        &self,
        graphics_width: f32,
        graphics_height: f32,
        scale: f32,
        delta: f32,
    ) -> Option<LoadFrameInput> {
        if !self.visible {
            return None;
        }

        let mut input = LoadFrameInput::new(
            graphics_width,
            graphics_height,
            scale,
            delta,
            self.stage,
            self.progress,
        );
        input.progress_visible = self.progress_visible;
        input.prompt = Some(self.text.clone());
        input.error = self.error.clone();
        input.completion = self.completion.clone();
        input.cancelable = self.cancelable;
        Some(input)
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
        let LoadFramePlan {
            stage,
            stage_text,
            prompt_text,
            commands,
            ..
        } = self;
        let commands = commands
            .into_iter()
            .flat_map(LoadRenderCommand::into_render_commands)
            .collect::<Vec<_>>();

        if commands.is_empty() {
            return Some(Self::empty_fallback_render_pass(
                stage,
                stage_text,
                prompt_text,
            ));
        }

        let mut pass = RenderPass::new(RenderPassKind::Custom(LOAD_PASS_KIND.to_string()));
        pass.extend(commands);
        Some(pass)
    }

    fn empty_fallback_render_pass(
        stage: LoadStage,
        stage_text: String,
        prompt_text: String,
    ) -> RenderPass {
        let mut pass = RenderPass::new(RenderPassKind::Custom(LOAD_PASS_KIND.to_string()));
        pass.push(RenderCommand::clear([0.035, 0.040, 0.052, 1.0]));
        pass.push(RenderCommand::draw_text(
            "load render plan empty",
            RenderPoint::new(24.0, 48.0),
            [0.95, 0.70, 0.28, 1.0],
            18.0,
            0.0,
            RenderTextAlign::Start,
            LOAD_STAGE_TEXT_LAYER,
        ));
        pass.push(RenderCommand::draw_text(
            format!(
                "{} | {}",
                if stage_text.is_empty() {
                    stage.label().to_string()
                } else {
                    stage_text
                },
                if prompt_text.is_empty() {
                    stage.default_prompt().to_string()
                } else {
                    prompt_text
                }
            ),
            RenderPoint::new(24.0, 24.0),
            [0.68, 0.78, 0.86, 1.0],
            12.0,
            0.0,
            RenderTextAlign::Start,
            LOAD_PROMPT_TEXT_LAYER,
        ));
        pass
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
        bounds: LoadRect,
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
    LoadingFragment {
        overlay: LoadRect,
        top_warning: LoadRect,
        bottom_warning: LoadRect,
        label: String,
        label_center: (f32, f32),
        cancel_button: Option<LoadRect>,
        overlay_color: [f32; 4],
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
                bounds,
                spacing,
                stroke,
                color,
            } => {
                let spacing = spacing.max(1.0);
                let stroke = stroke.max(0.25);
                let left = bounds.x;
                let right = bounds.x + bounds.width.max(0.0);
                let bottom = bounds.y;
                let top = bounds.y + bounds.height.max(0.0);

                let mut x = left;
                while x <= right {
                    commands.push(RenderCommand::draw_line(
                        RenderPoint::new(x, bottom),
                        RenderPoint::new(x, top),
                        stroke,
                        color,
                        LOAD_GRID_LAYER,
                    ));
                    x += spacing;
                }

                let mut y = bottom;
                while y <= top {
                    commands.push(RenderCommand::draw_line(
                        RenderPoint::new(left, y),
                        RenderPoint::new(right, y),
                        stroke,
                        color,
                        LOAD_GRID_LAYER,
                    ));
                    y += spacing;
                }
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
            Self::LoadingFragment {
                overlay,
                top_warning,
                bottom_warning,
                label,
                label_center,
                cancel_button,
                overlay_color,
            } => {
                commands.push(RenderCommand::fill_rect(
                    RenderRect::new(overlay.x, overlay.y, overlay.width, overlay.height),
                    overlay_color,
                    LOAD_FRAGMENT_OVERLAY_LAYER,
                ));
                push_warning_bar_render_commands(&mut commands, top_warning);
                push_warning_bar_render_commands(&mut commands, bottom_warning);
                commands.push(RenderCommand::draw_text(
                    label,
                    RenderPoint::new(label_center.0, label_center.1),
                    [0.98, 0.98, 0.98, 1.0],
                    22.0,
                    0.0,
                    RenderTextAlign::Center,
                    LOAD_FRAGMENT_LABEL_LAYER,
                ));
                if let Some(cancel_button) = cancel_button {
                    let rect = RenderRect::new(
                        cancel_button.x,
                        cancel_button.y,
                        cancel_button.width,
                        cancel_button.height,
                    );
                    commands.push(RenderCommand::fill_rect(
                        rect,
                        [0.06, 0.08, 0.10, 0.92],
                        LOAD_FRAGMENT_BUTTON_LAYER,
                    ));
                    commands.push(RenderCommand::stroke_rect(
                        rect,
                        [0.36, 0.58, 0.70, 0.95],
                        2.0,
                        LOAD_FRAGMENT_EDGE_LAYER,
                    ));
                    commands.push(RenderCommand::draw_text(
                        "@cancel",
                        rect.center(),
                        [0.90, 0.96, 1.0, 1.0],
                        18.0,
                        0.0,
                        RenderTextAlign::Center,
                        LOAD_FRAGMENT_EDGE_LAYER + 0.1,
                    ));
                }
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct LoadingFragmentLayout {
    top_warning: LoadRect,
    label: LoadRect,
    bottom_warning: LoadRect,
    progress_bar: LoadRect,
    cancel_button: Option<LoadRect>,
}

fn loading_fragment_layout(
    width: f32,
    height: f32,
    scale: f32,
    cancelable: bool,
) -> LoadingFragmentLayout {
    let scale = scale.max(0.1);
    let top_spacer = LOAD_FRAGMENT_TOP_SPACER * scale;
    let warning_height = (LOAD_FRAGMENT_WARNING_HEIGHT * scale).max(12.0);
    let label_gap = LOAD_FRAGMENT_LABEL_GAP * scale;
    let label_height = (LOAD_FRAGMENT_LABEL_HEIGHT * scale).max(20.0);
    let progress_width = (width * 0.54).clamp(220.0 * scale, 640.0 * scale);
    let progress_height = (18.0 * scale).max(10.0);
    let progress_top_pad = LOAD_FRAGMENT_BAR_TOP_PAD * scale;
    let cancel_width = (LOAD_FRAGMENT_CANCEL_WIDTH * scale).min((width - 40.0 * scale).max(1.0));
    let cancel_height = (LOAD_FRAGMENT_CANCEL_HEIGHT * scale).max(40.0);
    let cancel_pad = LOAD_FRAGMENT_CANCEL_PAD * scale;

    let top_warning_y = (height - top_spacer - warning_height).max(height * 0.56);
    let label_y = top_warning_y - label_gap - label_height;
    let bottom_warning_y = label_y - label_gap - warning_height;
    let progress_y = bottom_warning_y - progress_top_pad - progress_height;
    let cancel_button = cancelable.then_some(LoadRect::new(
        (width - cancel_width) * 0.5,
        (progress_y - cancel_pad - cancel_height).max(12.0 * scale),
        cancel_width,
        cancel_height,
    ));

    LoadingFragmentLayout {
        top_warning: LoadRect::new(0.0, top_warning_y, width, warning_height),
        label: LoadRect::new(0.0, label_y, width, label_height),
        bottom_warning: LoadRect::new(0.0, bottom_warning_y, width, warning_height),
        progress_bar: LoadRect::new(
            (width - progress_width) * 0.5,
            progress_y,
            progress_width,
            progress_height,
        ),
        cancel_button,
    }
}

fn push_warning_bar_render_commands(commands: &mut Vec<RenderCommand>, rect: LoadRect) {
    let warning = WarningBar::new();
    let plan = warning.draw_plan(WarningBarLayout::new(
        rect.x,
        rect.y,
        rect.width,
        rect.height,
    ));
    for command in plan.commands {
        match command {
            WarningBarDrawCommand::Stripe(stripe) => {
                let center = RenderPoint::new(
                    (stripe.quad.x1 + stripe.quad.x3) * 0.5,
                    (stripe.quad.y1 + stripe.quad.y3) * 0.5,
                );
                let from = RenderPoint::new(
                    (stripe.quad.x1 + stripe.quad.x4) * 0.5,
                    (stripe.quad.y1 + stripe.quad.y4) * 0.5,
                );
                let to = RenderPoint::new(
                    (stripe.quad.x2 + stripe.quad.x3) * 0.5,
                    (stripe.quad.y2 + stripe.quad.y3) * 0.5,
                );
                let color = load_rgba_u32_to_f32(stripe.color_rgba, stripe.alpha);
                commands.push(RenderCommand::draw_line(
                    from,
                    to,
                    warning.bar_width,
                    color,
                    LOAD_FRAGMENT_WARNING_LAYER,
                ));
                commands.push(RenderCommand::draw_circle(
                    center,
                    (warning.bar_width * 0.18).max(1.0),
                    color,
                    true,
                    LOAD_FRAGMENT_WARNING_LAYER + 0.01,
                ));
            }
            WarningBarDrawCommand::Line(line) => commands.push(RenderCommand::draw_line(
                RenderPoint::new(line.line.from_x, line.line.from_y),
                RenderPoint::new(line.line.to_x, line.line.to_y),
                line.line.stroke,
                load_rgba_u32_to_f32(line.color_rgba, line.alpha),
                LOAD_FRAGMENT_EDGE_LAYER,
            )),
        }
    }
}

fn load_rgba_u32_to_f32(rgba: u32, alpha_scale: f32) -> [f32; 4] {
    [
        ((rgba >> 24) & 0xff) as f32 / 255.0,
        ((rgba >> 16) & 0xff) as f32 / 255.0,
        ((rgba >> 8) & 0xff) as f32 / 255.0,
        ((rgba & 0xff) as f32 / 255.0) * alpha_scale.clamp(0.0, 1.0),
    ]
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
            progress_visible: true,
            prompt: None,
            error: None,
            completion: None,
            cancelable: false,
        });

        assert_eq!(plan.stage, LoadStage::LoadingContent);
        assert_eq!(plan.progress, 0.42);
        assert_eq!(plan.stage_text, "content  42%");
        assert_eq!(plan.prompt_text, "loading content");
        assert_eq!(plan.commands.len(), 9);

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
            LoadRenderCommand::LoadingFragment {
                cancel_button: None,
                ..
            }
        ));
        assert!(matches!(
            plan.commands[6],
            LoadRenderCommand::ProgressBar { .. }
        ));
        assert!(matches!(
            plan.commands[7],
            LoadRenderCommand::StageLabel { .. }
        ));
        assert!(matches!(
            plan.commands[8],
            LoadRenderCommand::PromptText { .. }
        ));
        assert_eq!(state.planet_rotation_deg, 54.0);
    }

    #[test]
    fn loading_plan_includes_loading_fragment_overlay_warning_bars_and_cancel() {
        let mut state = LoadRendererState::default();
        let mut input = LoadFrameInput::new(1280.0, 720.0, 1.0, 0.0, LoadStage::LoadingAssets, 0.2);
        input.cancelable = true;
        let plan = state.build_plan(input);

        match &plan.commands[5] {
            LoadRenderCommand::LoadingFragment {
                overlay,
                top_warning,
                bottom_warning,
                label,
                cancel_button,
                overlay_color,
                ..
            } => {
                assert_eq!(*overlay, LoadRect::new(0.0, 0.0, 1280.0, 720.0));
                assert_eq!(label, "@loading");
                assert_eq!(*overlay_color, [0.0, 0.0, 0.0, 0.80]);
                assert_eq!(top_warning.width, 1280.0);
                assert_eq!(bottom_warning.width, 1280.0);
                assert!(cancel_button.is_some());
            }
            other => panic!("expected LoadingFragment command, got {other:?}"),
        }

        let pass = plan
            .to_render_pass()
            .expect("load plan with LoadingFragment should render");
        let texts = pass
            .commands
            .iter()
            .filter_map(|command| match command {
                RenderCommand::DrawText { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(texts.contains(&"@loading"));
        assert!(texts.contains(&"@cancel"));
        assert!(pass.commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::FillRect { color, layer, .. }
                    if *color == [0.0, 0.0, 0.0, 0.80]
                        && (*layer - LOAD_FRAGMENT_OVERLAY_LAYER).abs() < f32::EPSILON
            )
        }));
        assert!(
            pass.commands
                .iter()
                .filter(|command| matches!(command, RenderCommand::DrawLine { .. }))
                .count()
                >= 4,
            "LoadingFragment should render the two WarningBar stripe/edge groups"
        );
    }

    #[test]
    fn loading_fragment_state_show_hides_progress_and_cancel_until_setters_like_java() {
        let mut fragment = LoadingFragmentState::new();
        assert_eq!(fragment.frame_input(1280.0, 720.0, 1.0, 0.0), None);

        fragment.show();
        let input = fragment
            .frame_input(1280.0, 720.0, 1.0, 0.0)
            .expect("shown LoadingFragment should produce a load frame input");
        assert_eq!(input.prompt.as_deref(), Some("@loading"));
        assert!(!input.progress_visible);
        assert!(!input.cancelable);

        let mut renderer = LoadRendererState::default();
        let shown_plan = renderer.build_plan(input);
        assert!(!shown_plan
            .commands
            .iter()
            .any(|command| matches!(command, LoadRenderCommand::ProgressBar { .. })));
        let shown_pass = shown_plan
            .to_render_pass()
            .expect("shown LoadingFragment should render a visible pass");
        let shown_texts = shown_pass
            .commands
            .iter()
            .filter_map(|command| match command {
                RenderCommand::DrawText { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(shown_texts.contains(&"@loading"));
        assert!(!shown_texts.contains(&"@cancel"));

        fragment.set_progress(0.37);
        fragment.set_button();
        let input = fragment
            .frame_input(1280.0, 720.0, 1.0, 0.0)
            .expect("visible LoadingFragment should keep producing load input");
        assert_eq!(input.stage, LoadStage::LoadingAssets);
        assert_eq!(input.progress, 0.37);
        assert!(input.progress_visible);
        assert!(input.cancelable);

        let updated_plan = renderer.build_plan(input);
        assert!(updated_plan
            .commands
            .iter()
            .any(|command| matches!(command, LoadRenderCommand::ProgressBar { .. })));
        let updated_pass = updated_plan
            .to_render_pass()
            .expect("updated LoadingFragment should render");
        let updated_texts = updated_pass
            .commands
            .iter()
            .filter_map(|command| match command {
                RenderCommand::DrawText { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(updated_texts.contains(&"@cancel"));

        fragment.hide();
        assert_eq!(fragment.frame_input(1280.0, 720.0, 1.0, 0.0), None);
    }

    #[test]
    fn loading_fragment_state_error_and_completion_emit_terminal_frames() {
        let mut fragment = LoadingFragmentState::new();
        fragment.show_text("loading assets");
        fragment.set_progress(0.4);
        fragment.fail("asset loader crashed");

        let mut renderer = LoadRendererState::default();
        let failed_plan = renderer.build_plan(
            fragment
                .frame_input(960.0, 540.0, 1.0, 0.1)
                .expect("failed LoadingFragment should stay visible"),
        );
        assert_eq!(failed_plan.stage, LoadStage::Failed);
        assert_eq!(failed_plan.stage_text, "asset loader crashed");
        assert!(matches!(
            failed_plan.commands.last().unwrap(),
            LoadRenderCommand::ErrorBanner { .. }
        ));

        fragment.complete("all assets ready");
        let complete_plan = renderer.build_plan(
            fragment
                .frame_input(960.0, 540.0, 1.0, 0.1)
                .expect("complete LoadingFragment should stay visible"),
        );
        assert_eq!(complete_plan.stage, LoadStage::Complete);
        assert_eq!(complete_plan.progress, 1.0);
        assert_eq!(complete_plan.stage_text, "all assets ready");
        assert!(matches!(
            complete_plan.commands.last().unwrap(),
            LoadRenderCommand::CompletionBanner { .. }
        ));
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
        assert_eq!(plan.commands.len(), 6);
        assert!(matches!(plan.commands[0], LoadRenderCommand::Clear { .. }));
        assert!(matches!(
            plan.commands[1],
            LoadRenderCommand::BackgroundGlow { .. }
        ));
        assert!(matches!(
            plan.commands[2],
            LoadRenderCommand::LoadingFragment { .. }
        ));
        assert!(matches!(
            plan.commands[3],
            LoadRenderCommand::ProgressBar { .. }
        ));
        assert!(matches!(
            plan.commands[4],
            LoadRenderCommand::StageLabel { .. }
        ));
        assert!(matches!(
            plan.commands[5],
            LoadRenderCommand::PromptText { .. }
        ));
    }

    #[test]
    fn empty_load_frame_plan_creates_visible_fallback_render_pass() {
        let plan = LoadFramePlan {
            stage: LoadStage::Boot,
            progress: 0.0,
            stage_text: String::new(),
            prompt_text: String::new(),
            commands: Vec::new(),
        };

        let pass = plan
            .to_render_pass()
            .expect("empty load plan should still render a visible fallback pass");
        assert_eq!(
            pass.kind,
            RenderPassKind::Custom(LOAD_PASS_KIND.to_string())
        );
        assert!(matches!(
            pass.commands.first(),
            Some(RenderCommand::Clear { .. })
        ));
        let texts = pass
            .commands
            .iter()
            .filter_map(|command| match command {
                RenderCommand::DrawText { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(texts.contains(&"load render plan empty"));
        assert!(texts.contains(&"boot | booting"));

        assert!(plan.into_render_pass().is_some());
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
                    bounds: LoadRect::new(0.0, 0.0, 1280.0, 720.0),
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
        assert!(owned.commands.len() > 15);

        assert!(matches!(owned.commands[0], RenderCommand::Clear { .. }));
        assert!(matches!(
            owned.commands[1],
            RenderCommand::DrawCircle { .. }
        ));
        assert!(matches!(
            owned.commands[2],
            RenderCommand::DrawLine { layer, .. } if (layer - LOAD_GRID_LAYER).abs() < f32::EPSILON
        ));
        assert!(owned
            .commands
            .iter()
            .any(|command| matches!(command, RenderCommand::DrawSprite { symbol, .. } if symbol == "serpulo")));
        assert!(
            owned
                .commands
                .iter()
                .filter(|command| {
                    matches!(
                        command,
                        RenderCommand::DrawLine { layer, .. }
                            if (*layer - LOAD_GRID_LAYER).abs() < f32::EPSILON
                    )
                })
                .count()
                >= 4
        );
        for expected in [
            "assets",
            "error  25%",
            "loading assets",
            "asset loader crashed",
            "retry or inspect the failure source",
            "all assets ready",
        ] {
            assert!(
                owned.commands.iter().any(|command| {
                    matches!(command, RenderCommand::DrawText { text, .. } if text == expected)
                }),
                "load pass should contain text command {expected:?}"
            );
        }
        assert!(owned.commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::FillRect { color, .. } if *color == [0.32, 0.84, 0.50, 1.0]
            )
        }));
    }
}
