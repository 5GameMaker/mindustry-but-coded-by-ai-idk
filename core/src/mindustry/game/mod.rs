pub mod attack_indicators;
pub mod campaign_rules;
pub mod campaign_stats;
pub mod difficulty;
pub mod event_type;
pub mod game_stats;
pub mod gamemode;
pub mod map_markers;
pub mod map_objectives;
pub mod objectives;
pub mod rules;
pub mod schematic;
pub mod sector_info;
pub mod spawn_group;
pub mod team;
pub mod teams;
pub mod waves;

pub use attack_indicators::{AttackIndicator, AttackIndicators, ATTACK_INDICATOR_DURATION};
pub use campaign_rules::{CampaignPlanetRules, CampaignRules, CampaignRulesApplyResult};
pub use campaign_stats::CampaignStats;
pub use difficulty::Difficulty;
pub use event_type::Trigger;
pub use game_stats::GameStats;
pub use gamemode::Gamemode;
pub use map_markers::MapMarkers;
pub use map_objectives::{
    marker_type_by_java_name, CompleteObjectiveEvent, MapObjective, MapObjectiveCommon,
    MapObjectiveContext, MapObjectiveKind, MapObjectives, MarkerCommon, ObjectiveMarker, Point2,
    Vec2,
};
pub use objectives::{
    Objective, ObjectiveContent, ObjectiveKind, PlanetObjectiveState, SectorObjectiveState,
};
pub use rules::{Rules, TeamRule, TeamRules};
pub use schematic::{Schematic, SchematicTile};
pub use sector_info::{ExportStat, SectorInfo};
pub use spawn_group::SpawnGroup;
pub use team::{
    vanilla_teams, Team, TeamRegistry, BASE_TEAM_COUNT, TEAM_BLUE, TEAM_COUNT, TEAM_CRUX,
    TEAM_DERELICT, TEAM_GREEN, TEAM_MALIS, TEAM_NEOPLASTIC, TEAM_SHARDED,
};
pub use teams::{BlockPlan, CoreInfo, TeamData, Teams};
pub use waves::{default_spawn_groups, generate, generate_with_seed, Waves, WAVE_VERSION};
