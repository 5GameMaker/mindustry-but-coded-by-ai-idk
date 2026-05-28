/// 数据化的 OverlayRenderer 状态/计划层，对应 upstream `mindustry.graphics.OverlayRenderer`。
///
/// 这里不绑定任何 GPU 或具体绘制后端，只保留语义化的 overlay 指令：
/// build placement、selection、power/liquid/item overlays，以及 spawn/core/target/logic 等。
use std::mem;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlayPoint {
    pub x: f32,
    pub y: f32,
}

impl OverlayPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OverlayTile {
    pub x: i32,
    pub y: i32,
}

impl OverlayTile {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverlayAnchor {
    Point(OverlayPoint),
    Tile(OverlayTile),
    Entity {
        id: u64,
        position: OverlayPoint,
        hit_size: f32,
    },
}

impl OverlayAnchor {
    pub const fn point(x: f32, y: f32) -> Self {
        Self::Point(OverlayPoint::new(x, y))
    }

    pub const fn tile(x: i32, y: i32) -> Self {
        Self::Tile(OverlayTile::new(x, y))
    }

    pub const fn entity(id: u64, x: f32, y: f32, hit_size: f32) -> Self {
        Self::Entity {
            id,
            position: OverlayPoint::new(x, y),
            hit_size,
        }
    }

    pub fn position(self) -> OverlayPoint {
        match self {
            Self::Point(point) => point,
            Self::Tile(tile) => OverlayPoint::new(tile.x as f32, tile.y as f32),
            Self::Entity { position, .. } => position,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectionTargetId {
    Player(u64),
    Unit(u64),
    Building(u64),
    Logic(u64),
    Tile(OverlayTile),
}

impl SelectionTargetId {
    pub const fn player(id: u64) -> Self {
        Self::Player(id)
    }

    pub const fn unit(id: u64) -> Self {
        Self::Unit(id)
    }

    pub const fn building(id: u64) -> Self {
        Self::Building(id)
    }

    pub const fn logic(id: u64) -> Self {
        Self::Logic(id)
    }

    pub const fn tile(x: i32, y: i32) -> Self {
        Self::Tile(OverlayTile::new(x, y))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildPlacementSource {
    Player,
    OtherPlayer,
    Schematic,
    Scripted,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayBuildPlan {
    pub tile: OverlayTile,
    pub rotation: i32,
    pub block: Option<String>,
    pub breaking: bool,
    pub config: Option<String>,
    pub progress: f32,
    pub initialized: bool,
    pub stuck: bool,
    pub cached_valid: bool,
    pub world_context: bool,
    pub anim_scale: f32,
}

impl OverlayBuildPlan {
    pub fn new_place(tile: OverlayTile, rotation: i32, block: impl Into<String>) -> Self {
        Self {
            tile,
            rotation,
            block: Some(block.into()),
            breaking: false,
            config: None,
            progress: 0.0,
            initialized: false,
            stuck: false,
            cached_valid: false,
            world_context: true,
            anim_scale: 0.0,
        }
    }

    pub fn new_break(tile: OverlayTile) -> Self {
        Self {
            tile,
            rotation: -1,
            block: None,
            breaking: true,
            config: None,
            progress: 0.0,
            initialized: false,
            stuck: false,
            cached_valid: false,
            world_context: true,
            anim_scale: 0.0,
        }
    }

    pub fn with_config(mut self, config: impl Into<String>) -> Self {
        self.config = Some(config.into());
        self
    }
}

impl Default for OverlayBuildPlan {
    fn default() -> Self {
        Self::new_break(OverlayTile::new(0, 0))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildPlacementOverlay {
    pub plan: OverlayBuildPlan,
    pub source: BuildPlacementSource,
    pub visible: bool,
    pub highlighted: bool,
}

impl BuildPlacementOverlay {
    pub fn new(plan: OverlayBuildPlan) -> Self {
        Self {
            plan,
            source: BuildPlacementSource::Player,
            visible: true,
            highlighted: false,
        }
    }

    pub fn sourced(plan: OverlayBuildPlan, source: BuildPlacementSource) -> Self {
        Self {
            plan,
            source,
            visible: true,
            highlighted: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceOverlayKind {
    Power,
    Liquid,
    Item,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceOverlay {
    pub kind: ResourceOverlayKind,
    pub source: OverlayAnchor,
    pub target: Option<OverlayAnchor>,
    pub amount: Option<f32>,
    pub radius: Option<f32>,
    pub valid: bool,
    pub label: Option<String>,
}

impl ResourceOverlay {
    pub fn new(kind: ResourceOverlayKind, source: OverlayAnchor) -> Self {
        Self {
            kind,
            source,
            target: None,
            amount: None,
            radius: None,
            valid: true,
            label: None,
        }
    }

    pub fn with_target(mut self, target: OverlayAnchor) -> Self {
        self.target = Some(target);
        self
    }

    pub fn with_amount(mut self, amount: f32) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionOverlay {
    pub id: SelectionTargetId,
    pub anchor: OverlayAnchor,
    pub hit_size: f32,
    pub fade: f32,
    pub accent: bool,
    pub arrow_count: u8,
    pub icon: Option<String>,
}

impl SelectionOverlay {
    pub fn new(id: SelectionTargetId, anchor: OverlayAnchor, hit_size: f32) -> Self {
        Self {
            id,
            anchor,
            hit_size,
            fade: 1.0,
            accent: false,
            arrow_count: 4,
            icon: None,
        }
    }

    pub fn with_fade(mut self, fade: f32) -> Self {
        self.fade = fade;
        self
    }

    pub fn with_accent(mut self, accent: bool) -> Self {
        self.accent = accent;
        self
    }

    pub fn with_arrow_count(mut self, arrow_count: u8) -> Self {
        self.arrow_count = arrow_count;
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnOverlay {
    pub tile: OverlayTile,
    pub center: OverlayPoint,
    pub drop_zone_radius: f32,
    pub alpha: f32,
}

impl SpawnOverlay {
    pub fn new(tile: OverlayTile, center: OverlayPoint, drop_zone_radius: f32) -> Self {
        Self {
            tile,
            center,
            drop_zone_radius,
            alpha: 1.0,
        }
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoreOverlay {
    pub center: OverlayPoint,
    pub build_radius: f32,
    pub team: Option<String>,
    pub accent_strength: f32,
    pub highlighted: bool,
}

impl CoreOverlay {
    pub fn new(center: OverlayPoint, build_radius: f32) -> Self {
        Self {
            center,
            build_radius,
            team: None,
            accent_strength: 0.5,
            highlighted: true,
        }
    }

    pub fn with_team(mut self, team: impl Into<String>) -> Self {
        self.team = Some(team.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TargetOverlay {
    pub origin: OverlayPoint,
    pub target: OverlayPoint,
    pub stroke: f32,
    pub radius: f32,
    pub visible: bool,
    pub label: Option<String>,
}

impl TargetOverlay {
    pub fn new(origin: OverlayPoint, target: OverlayPoint, stroke: f32) -> Self {
        Self {
            origin,
            target,
            stroke,
            radius: 0.0,
            visible: true,
            label: None,
        }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicOverlay {
    pub unit: OverlayAnchor,
    pub controller: OverlayAnchor,
    pub privileged: bool,
    pub valid: bool,
    pub arrow_length: f32,
    pub label: Option<String>,
}

impl LogicOverlay {
    pub fn new(unit: OverlayAnchor, controller: OverlayAnchor) -> Self {
        Self {
            unit,
            controller,
            privileged: false,
            valid: true,
            arrow_length: 28.0,
            label: None,
        }
    }

    pub fn privileged(mut self, privileged: bool) -> Self {
        self.privileged = privileged;
        self
    }

    pub fn invalid(mut self) -> Self {
        self.valid = false;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndicatorOverlay {
    pub origin: OverlayPoint,
    pub direction: f32,
    pub length: f32,
    pub thickness: f32,
    pub color: Option<String>,
}

impl IndicatorOverlay {
    pub fn new(origin: OverlayPoint, direction: f32, length: f32, thickness: f32) -> Self {
        Self {
            origin,
            direction,
            length,
            thickness,
            color: None,
        }
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelOverlay {
    pub position: OverlayPoint,
    pub text: String,
    pub visible: bool,
}

impl LabelOverlay {
    pub fn new(position: OverlayPoint, text: impl Into<String>) -> Self {
        Self {
            position,
            text: text.into(),
            visible: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OverlayCommand {
    Selection(SelectionOverlay),
    Resource(ResourceOverlay),
    Spawn(SpawnOverlay),
    Core(CoreOverlay),
    Target(TargetOverlay),
    Logic(LogicOverlay),
    Indicator(IndicatorOverlay),
    Label(LabelOverlay),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoreEdgeOverlay {
    pub start: OverlayPoint,
    pub end: OverlayPoint,
    pub team_a: Option<String>,
    pub team_b: Option<String>,
    pub display_team: Option<String>,
}

impl CoreEdgeOverlay {
    pub fn new(start: OverlayPoint, end: OverlayPoint) -> Self {
        Self {
            start,
            end,
            team_a: None,
            team_b: None,
            display_team: None,
        }
    }

    pub fn with_teams(mut self, team_a: impl Into<String>, team_b: impl Into<String>) -> Self {
        self.team_a = Some(team_a.into());
        self.team_b = Some(team_b.into());
        self
    }

    pub fn with_display_team(mut self, team: impl Into<String>) -> Self {
        self.display_team = Some(team.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayRendererPlan {
    pub build_fade: f32,
    pub unit_fade: f32,
    pub last_select: Option<SelectionTargetId>,
    pub updated_cores: bool,
    pub core_edges: Vec<CoreEdgeOverlay>,
    pub build_placements: Vec<BuildPlacementOverlay>,
    pub commands: Vec<OverlayCommand>,
}

impl Default for OverlayRendererPlan {
    fn default() -> Self {
        Self {
            build_fade: 0.0,
            unit_fade: 0.0,
            last_select: None,
            updated_cores: false,
            core_edges: Vec::new(),
            build_placements: Vec::new(),
            commands: Vec::new(),
        }
    }
}

impl OverlayRendererPlan {
    pub fn is_empty(&self) -> bool {
        self.build_placements.is_empty() && self.commands.is_empty() && self.core_edges.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayRendererState {
    pub build_fade: f32,
    pub unit_fade: f32,
    pub last_select: Option<SelectionTargetId>,
    pub updated_cores: bool,
    pub core_edges: Vec<CoreEdgeOverlay>,
    pub build_placements: Vec<BuildPlacementOverlay>,
    pub commands: Vec<OverlayCommand>,
}

impl Default for OverlayRendererState {
    fn default() -> Self {
        Self {
            build_fade: 0.0,
            unit_fade: 0.0,
            last_select: None,
            updated_cores: true,
            core_edges: Vec::new(),
            build_placements: Vec::new(),
            commands: Vec::new(),
        }
    }
}

impl OverlayRendererState {
    pub fn set_build_fade(&mut self, build_fade: f32) -> &mut Self {
        self.build_fade = clamp01(build_fade);
        self
    }

    pub fn set_unit_fade(&mut self, unit_fade: f32) -> &mut Self {
        self.unit_fade = clamp01(unit_fade);
        self
    }

    pub fn remember_selection(&mut self, selection: SelectionTargetId) {
        self.last_select = Some(selection);
    }

    pub fn clear_selection(&mut self) {
        self.last_select = None;
    }

    pub fn mark_core_edges_dirty(&mut self) {
        self.updated_cores = true;
    }

    pub fn replace_core_edges<I>(&mut self, edges: I)
    where
        I: IntoIterator<Item = CoreEdgeOverlay>,
    {
        self.core_edges = edges.into_iter().collect();
        self.updated_cores = false;
    }

    pub fn push_build_placement(&mut self, overlay: BuildPlacementOverlay) -> &mut Self {
        self.build_placements.push(overlay);
        self
    }

    pub fn push_command(&mut self, command: OverlayCommand) -> &mut Self {
        self.commands.push(command);
        self
    }

    pub fn push_selection(&mut self, overlay: SelectionOverlay) -> &mut Self {
        self.push_command(OverlayCommand::Selection(overlay))
    }

    pub fn push_resource(&mut self, overlay: ResourceOverlay) -> &mut Self {
        self.push_command(OverlayCommand::Resource(overlay))
    }

    pub fn push_spawn(&mut self, overlay: SpawnOverlay) -> &mut Self {
        self.push_command(OverlayCommand::Spawn(overlay))
    }

    pub fn push_core(&mut self, overlay: CoreOverlay) -> &mut Self {
        self.push_command(OverlayCommand::Core(overlay))
    }

    pub fn push_target(&mut self, overlay: TargetOverlay) -> &mut Self {
        self.push_command(OverlayCommand::Target(overlay))
    }

    pub fn push_logic(&mut self, overlay: LogicOverlay) -> &mut Self {
        self.push_command(OverlayCommand::Logic(overlay))
    }

    pub fn push_indicator(&mut self, overlay: IndicatorOverlay) -> &mut Self {
        self.push_command(OverlayCommand::Indicator(overlay))
    }

    pub fn push_label(&mut self, overlay: LabelOverlay) -> &mut Self {
        self.push_command(OverlayCommand::Label(overlay))
    }

    pub fn drain_plan(&mut self) -> OverlayRendererPlan {
        let updated_cores = self.updated_cores;
        self.updated_cores = false;

        OverlayRendererPlan {
            build_fade: self.build_fade,
            unit_fade: self.unit_fade,
            last_select: self.last_select,
            updated_cores,
            core_edges: self.core_edges.clone(),
            build_placements: mem::take(&mut self.build_placements),
            commands: mem::take(&mut self.commands),
        }
    }
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_build_plan_and_selection_helpers_preserve_semantics() {
        let plan =
            OverlayBuildPlan::new_place(OverlayTile::new(3, 4), 2, "duo").with_config("power=1");
        assert_eq!(plan.tile, OverlayTile::new(3, 4));
        assert_eq!(plan.rotation, 2);
        assert_eq!(plan.block.as_deref(), Some("duo"));
        assert_eq!(plan.config.as_deref(), Some("power=1"));
        assert!(!plan.breaking);

        let break_plan = OverlayBuildPlan::new_break(OverlayTile::new(5, 6));
        assert!(break_plan.breaking);
        assert_eq!(break_plan.block, None);

        let selection = SelectionOverlay::new(
            SelectionTargetId::unit(9),
            OverlayAnchor::entity(9, 16.0, 24.0, 8.0),
            8.0,
        )
        .with_fade(0.75)
        .with_accent(true)
        .with_arrow_count(6)
        .with_icon("select-arrow");

        assert_eq!(selection.id, SelectionTargetId::unit(9));
        assert_eq!(selection.anchor.position(), OverlayPoint::new(16.0, 24.0));
        assert_eq!(selection.fade, 0.75);
        assert!(selection.accent);
        assert_eq!(selection.arrow_count, 6);
        assert_eq!(selection.icon.as_deref(), Some("select-arrow"));
    }

    #[test]
    fn overlay_renderer_state_drains_build_and_overlay_commands() {
        let mut state = OverlayRendererState::default();
        state
            .set_build_fade(0.9)
            .set_unit_fade(0.4)
            .remember_selection(SelectionTargetId::building(42));

        state.push_build_placement(BuildPlacementOverlay::sourced(
            OverlayBuildPlan::new_place(OverlayTile::new(1, 2), 1, "router"),
            BuildPlacementSource::Schematic,
        ));
        state.push_selection(
            SelectionOverlay::new(
                SelectionTargetId::building(42),
                OverlayAnchor::tile(10, 11),
                12.0,
            )
            .with_accent(true),
        );
        state.push_resource(
            ResourceOverlay::new(ResourceOverlayKind::Power, OverlayAnchor::point(2.0, 3.0))
                .with_target(OverlayAnchor::point(4.0, 5.0))
                .with_amount(0.5)
                .with_radius(7.0)
                .with_label("power-link"),
        );
        state.push_spawn(SpawnOverlay::new(
            OverlayTile::new(7, 8),
            OverlayPoint::new(56.0, 64.0),
            48.0,
        ));
        state.push_core(CoreOverlay::new(OverlayPoint::new(100.0, 120.0), 32.0).with_team("blue"));
        state.push_target(
            TargetOverlay::new(
                OverlayPoint::new(1.0, 1.0),
                OverlayPoint::new(9.0, 9.0),
                3.0,
            )
            .with_radius(11.0)
            .with_label("logic-target"),
        );
        state.push_logic(
            LogicOverlay::new(
                OverlayAnchor::entity(7, 16.0, 16.0, 10.0),
                OverlayAnchor::tile(12, 13),
            )
            .privileged(true),
        );
        state.push_indicator(IndicatorOverlay::new(
            OverlayPoint::new(5.0, 6.0),
            90.0,
            14.0,
            2.0,
        ));
        state.push_label(LabelOverlay::new(
            OverlayPoint::new(13.0, 14.0),
            "only core deposit",
        ));

        let plan = state.drain_plan();
        assert_eq!(plan.build_fade, 0.9);
        assert_eq!(plan.unit_fade, 0.4);
        assert_eq!(plan.last_select, Some(SelectionTargetId::building(42)));
        assert_eq!(plan.build_placements.len(), 1);
        assert_eq!(plan.commands.len(), 8);
        assert!(matches!(plan.commands[0], OverlayCommand::Selection(_)));
        assert!(matches!(plan.commands[1], OverlayCommand::Resource(_)));
        assert!(matches!(plan.commands[2], OverlayCommand::Spawn(_)));
        assert!(matches!(plan.commands[3], OverlayCommand::Core(_)));
        assert!(matches!(plan.commands[4], OverlayCommand::Target(_)));
        assert!(matches!(plan.commands[5], OverlayCommand::Logic(_)));
        assert!(matches!(plan.commands[6], OverlayCommand::Indicator(_)));
        assert!(matches!(plan.commands[7], OverlayCommand::Label(_)));
        assert_eq!(plan.core_edges.len(), 0);
        assert!(!plan.is_empty());

        assert!(state.build_placements.is_empty());
        assert!(state.commands.is_empty());
        assert_eq!(state.last_select, Some(SelectionTargetId::building(42)));
    }

    #[test]
    fn overlay_renderer_state_preserves_core_edges_and_dirty_flag() {
        let mut state = OverlayRendererState::default();
        state.replace_core_edges(vec![CoreEdgeOverlay::new(
            OverlayPoint::new(0.0, 0.0),
            OverlayPoint::new(10.0, 0.0),
        )
        .with_teams("red", "blue")
        .with_display_team("blue")]);
        state.mark_core_edges_dirty();

        let plan = state.drain_plan();
        assert!(plan.updated_cores);
        assert_eq!(plan.core_edges.len(), 1);
        assert_eq!(state.core_edges.len(), 1);
        assert!(!state.updated_cores);

        let edge = &plan.core_edges[0];
        assert_eq!(edge.start, OverlayPoint::new(0.0, 0.0));
        assert_eq!(edge.end, OverlayPoint::new(10.0, 0.0));
        assert_eq!(edge.team_a.as_deref(), Some("red"));
        assert_eq!(edge.team_b.as_deref(), Some("blue"));
        assert_eq!(edge.display_team.as_deref(), Some("blue"));
    }
}
