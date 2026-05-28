//! Data-oriented migration of upstream `LoadRenderer`.
//!
//! The Java renderer draws the loading screen directly via `Core.graphics`.
//! This Rust counterpart keeps the same concerns as backend-neutral plans:
//! stage selection, progress bar, prompt text, logo/planet/background layers,
//! plus error and completion overlays.

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
}
